//! stream-ohlcv servisi — sembol + başlangıç zamanı + interval (sn) ile
//! canlı OHLCV mum akışı üreten servis.
//!
//! Akış:
//!   istek (POST /api/stream: {symbol, start_ms, interval_secs})
//!     → interval >= 60s ise ohlcv-engine (Binance klines) ile start_ms'ten
//!       bugüne kadar geçmiş mumları çek ve ring'e yayınla
//!     → lastprice flow ring'inden (`/cycle_finance_lastprice`) anlık fiyatı düzenli oku
//!     → canlı mumu güncelle, mum kapanınca binary olarak
//!       `/dev/shm/cycle_finance_stream_ohlcv` ring'ine yayınla
//!
//! Tüm istekler ve cevaplar HTTP API üzerinden gider; eşzamanlı istekler
//! her biri kendi stream görevi ile (tokio task) yanıtlanır.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use ohlcv_engine::Kline;
use rust_decimal::prelude::ToPrimitive;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use stream_ohlcv::codec;
use stream_ohlcv::{StreamCandle, StreamMeta, StreamRequest, StreamStatus, binance_interval};
use transport::stream_ring::{StreamRingBuffer, STREAM_DEFAULT_CAPACITY};

const DEFAULT_PORT: u16 = 3008;
const RING_NAME: &str = "/cycle_finance_stream_ohlcv";
const HISTORY_PAGE: usize = 1000;
const HISTORY_MAX_PAGES: usize = 200;
const CACHE_MAX: usize = 500;

// ── Paylaşılan durum ─────────────────────────────────────────
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

struct AppState {
    client: ohlcv_engine::client::BinanceClient,
    ring: Arc<StreamRingBuffer>,
    /// Ring'e push'ları seri hale getirir (çok sayıda stream eşzamanlı yazabilir).
    ring_lock: Arc<Mutex<()>>,
    streams: Arc<tokio::sync::RwLock<HashMap<u64, Arc<Stream>>>>,
    by_key: Arc<tokio::sync::RwLock<HashMap<String, u64>>>,
    next_id: AtomicU64,
}

struct Stream {
    id: u64,
    symbol: String,
    interval_secs: u64,
    start_ms: u64,
    created: u64,
    stop: Arc<AtomicBool>,
    task: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    state: tokio::sync::Mutex<StreamState>,
}

struct StreamState {
    status: StreamStatus,
    published: u64,
    last_price: Option<f64>,
    current: Option<StreamCandle>,
    /// Kapanan son mumlar (API/status için).
    cache: VecDeque<StreamCandle>,
}

fn stream_key(symbol: &str, interval_secs: u64) -> String {
    format!("{}:{}", symbol.to_uppercase(), interval_secs)
}

// ── Yardımcılar ──────────────────────────────────────────────
fn to_stream_candle(stream_id: u64, symbol: &str, interval_secs: u64, k: &Kline, closed: u8) -> StreamCandle {
    let f = |d: rust_decimal::Decimal| d.to_f64().unwrap_or(0.0);
    StreamCandle {
        stream_id,
        symbol: symbol.to_uppercase(),
        interval_secs,
        open_time: k.open_time,
        close_time: k.close_time,
        open: f(k.open),
        high: f(k.high),
        low: f(k.low),
        close: f(k.close),
        volume: f(k.volume),
        closed,
    }
}

fn new_candle(stream_id: u64, symbol: &str, interval_secs: u64, bucket: u64, price: f64) -> StreamCandle {
    StreamCandle {
        stream_id,
        symbol: symbol.to_uppercase(),
        interval_secs,
        open_time: bucket,
        close_time: bucket + interval_secs * 1000 - 1,
        open: price,
        high: price,
        low: price,
        close: price,
        volume: 0.0,
        closed: 0,
    }
}

fn publish(app: &AppState, candle: &StreamCandle) {
    let bytes = codec::encode(candle);
    let _g = app.ring_lock.lock().unwrap();
    app.ring.push(&bytes);
}

/// Son fiyatı lastprice flow ring'inden (RAM paylaşımlı bellek) okur.
/// `FundingRate` event'inin `mark_price` alanı last price olarak taşınır.
fn fetch_last_price(symbol: &str) -> Option<f64> {
    let sym = symbol.to_ascii_uppercase();
    let mut buf = [0u8; 16];
    let b = sym.as_bytes();
    let len = b.len().min(16);
    buf[..len].copy_from_slice(&b[..len]);

    let ring = transport::ring_buffer::GenerationalRingBuffer::with_name("/cycle_finance_lastprice", 1);
    let head = ring.get_head();
    let start = head.saturating_sub(256);
    for seq in (start..head).rev() {
        if let Some(slot) = ring.read_slot(seq) {
            if let Some(ev) = transport::wire::decode(&slot.data[..slot.len as usize]) {
                if &ev.symbol == &buf {
                    if let transport::events::EventType::FundingRate { mark_price, .. } = ev.payload {
                        if let Some(f) = mark_price.to_f64() {
                            if f > 0.0 {
                                return Some(f);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Binance klines'ı `start_ms`'ten bugüne kadar sayfalayarak çeker.
/// `stop` her sayfa arasında kontrol edilir → stream silinince erken çıkar.
async fn fetch_history(
    app: &AppState,
    symbol: &str,
    interval: &str,
    start_ms: u64,
    now: u64,
    stop: &Arc<AtomicBool>,
) -> Vec<Kline> {
    let mut all = Vec::new();
    let mut start = start_ms;
    for _ in 0..HISTORY_MAX_PAGES {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        match app
            .client
            .fetch_klines_range(symbol, interval, Some(start), None, HISTORY_PAGE)
            .await
        {
            Ok(klines) if !klines.is_empty() => {
                let last_close = klines.last().unwrap().close_time;
                all.extend(klines);
                if last_close >= now {
                    break;
                }
                start = last_close + 1;
                if start > now {
                    break;
                }
            }
            _ => break,
        }
    }
    all
}

// ── Stream görevi ────────────────────────────────────────────
async fn run_stream(app: Arc<AppState>, stream: Arc<Stream>) {
    let now = now_ms();
    let interval_ms = stream.interval_secs * 1000;
    let mut published = 0u64;

    {
        let mut state = stream.state.lock().await;
        state.status = StreamStatus::Running;
    }

    // 1) Geçmiş: interval >= 1m ise ohlcv'den start_ms'ten bugüne mumları çek.
    if stream.interval_secs >= 60 {
        if let Some(iv) = binance_interval(stream.interval_secs) {
            match fetch_history(&app, &stream.symbol, iv, stream.start_ms, now, &stream.stop).await {
                history if !history.is_empty() => {
                    let mut forming = None;
                    for k in &history {
                        if k.close_time < now {
                            let c = to_stream_candle(stream.id, &stream.symbol, stream.interval_secs, k, 1);
                            publish(&app, &c);
                            published += 1;
                        } else {
                            forming = Some(to_stream_candle(stream.id, &stream.symbol, stream.interval_secs, k, 0));
                        }
                    }
                    let mut state = stream.state.lock().await;
                    if let Some(f) = forming {
                        state.current = Some(f);
                    }
                }
                _ => {
                    eprintln!("[STREAM-{}] geçmiş OHLCV çekilemedi veya boş", stream.id);
                }
            }
        }
    }

    {
        let mut state = stream.state.lock().await;
        state.published = published;
    }
    println!(
        "[STREAM-{}] {} | {}s | start={} | geçmiş yayınlandı: {}",
        stream.id,
        stream.symbol,
        stream.interval_secs,
        stream.start_ms,
        published
    );

    if stream.stop.load(Ordering::SeqCst) {
        let mut state = stream.state.lock().await;
        state.status = StreamStatus::Stopped;
        drop(state);
        println!("[STREAM-{}] durduruldu (geçmiş çekilirken)", stream.id);
        return;
    }

    // 2) Canlı döngü: lastprice flow ring'i ile mumları güncelle/kapat.
    let poll_ms = if stream.interval_secs < 60 { 500u64 } else { 1000u64 };
    let mut last_report = SystemTime::now();
    loop {
        if stream.stop.load(Ordering::SeqCst) {
            break;
        }

        if let Some(price) = fetch_last_price(&stream.symbol) {
            let bucket = now_ms() - (now_ms() % interval_ms);
            let now = now_ms();
            let mut state = stream.state.lock().await;
            state.last_price = Some(price);
            let should_close = match state.current.as_ref() {
                Some(c) => c.open_time != bucket,
                None => true,
            };
            if should_close {
                if let Some(c) = state.current.take() {
                    let mut closed = c.clone();
                    closed.closed = 1;
                    closed.close_time = now;
                    publish(&app, &closed);
                    state.cache.push_back(closed);
                    if state.cache.len() > CACHE_MAX {
                        state.cache.pop_front();
                    }
                    state.published += 1;
                }
                let nc = new_candle(stream.id, &stream.symbol, stream.interval_secs, bucket, price);
                state.current = Some(nc);
            } else if let Some(c) = state.current.as_mut() {
                c.close = price;
                c.high = c.high.max(price);
                c.low = c.low.min(price);
            }
            drop(state);
        }

        if last_report.elapsed().unwrap_or_default().as_secs() >= 30 {
            let state = stream.state.lock().await;
            println!(
                "[STREAM-{}] {} | {}s | published={} | last={:?} | open_time={:?}",
                stream.id,
                stream.symbol,
                stream.interval_secs,
                state.published,
                state.last_price,
                state.current.as_ref().map(|c| c.open_time)
            );
            drop(state);
            last_report = SystemTime::now();
        }

        tokio::time::sleep(Duration::from_millis(poll_ms)).await;
    }

    let mut state = stream.state.lock().await;
    state.status = StreamStatus::Stopped;
    drop(state);
    println!("[STREAM-{}] durduruldu", stream.id);
}

async fn start_stream(app: Arc<AppState>, req: StreamRequest) -> StreamMeta {
    let key = stream_key(&req.symbol, req.interval_secs);
    let id;

    {
        let by_key = app.by_key.read().await;
        if let Some(existing_id) = by_key.get(&key) {
            let streams = app.streams.read().await;
            if let Some(existing) = streams.get(existing_id) {
                let state = existing.state.lock().await;
                return meta_of(existing, &state);
            }
        }
    }

    id = app.next_id.fetch_add(1, Ordering::SeqCst);

    let stream = Arc::new(Stream {
        id,
        symbol: req.symbol.to_uppercase(),
        interval_secs: req.interval_secs,
        start_ms: req.start_ms,
        created: now_ms(),
        stop: Arc::new(AtomicBool::new(false)),
        task: tokio::sync::Mutex::new(None),
        state: tokio::sync::Mutex::new(StreamState {
            status: StreamStatus::Starting,
            published: 0,
            last_price: None,
            current: None,
            cache: VecDeque::new(),
        }),
    });

    let handle = tokio::spawn(run_stream(app.clone(), stream.clone()));
    *stream.task.lock().await = Some(handle);

    app.streams.write().await.insert(id, stream.clone());
    app.by_key.write().await.insert(key, id);

    let state = stream.state.lock().await;
    meta_of(&stream, &state)
}

fn meta_of(stream: &Stream, state: &StreamState) -> StreamMeta {
    StreamMeta {
        stream_id: stream.id,
        symbol: stream.symbol.clone(),
        start_ms: stream.start_ms,
        interval_secs: stream.interval_secs,
        created: stream.created,
        status: state.status.clone(),
        published: state.published,
        last_price: state.last_price,
        current: state.current.clone(),
    }
}

async fn stop_stream(app: &AppState, stream_id: u64) -> bool {
    let removed = {
        let mut streams = app.streams.write().await;
        let mut by_key = app.by_key.write().await;
        match streams.remove(&stream_id) {
            Some(s) => {
                by_key.remove(&stream_key(&s.symbol, s.interval_secs));
                s.stop.store(true, Ordering::SeqCst);
                let handle = s.task.lock().await.take();
                drop(streams);
                drop(by_key);
                if let Some(h) = handle {
                    let _ = tokio::time::timeout(Duration::from_secs(3), h).await;
                }
                true
            }
            None => false,
        }
    };
    removed
}

// ── HTTP API ─────────────────────────────────────────────────
#[derive(Deserialize)]
struct CandlesParams {
    limit: Option<usize>,
}

async fn api_stream_start(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StreamRequest>,
) -> Json<serde_json::Value> {
    if req.symbol.trim().is_empty() {
        return Json(serde_json::json!({"error": "sembol boş olamaz"}));
    }
    if req.interval_secs == 0 {
        return Json(serde_json::json!({"error": "interval_secs > 0 olmalı"}));
    }
    if req.interval_secs < 60 && binance_interval(req.interval_secs).is_none() {
        return Json(serde_json::json!({"error": format!("desteklenmeyen interval (sn): {}", req.interval_secs)}));
    }
    let meta = start_stream(state.clone(), req).await;
    Json(serde_json::json!(meta))
}

async fn api_streams(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let streams = state.streams.read().await;
    let mut metas = Vec::new();
    for s in streams.values() {
        let st = s.state.lock().await;
        metas.push(meta_of(s, &st));
    }
    Json(serde_json::json!(metas))
}

async fn api_stream_get(
    State(state): State<Arc<AppState>>,
    Path(stream_id): Path<u64>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let streams = state.streams.read().await;
    match streams.get(&stream_id) {
        Some(s) => {
            let st = s.state.lock().await;
            Ok(Json(serde_json::json!(meta_of(s, &st))))
        }
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("bilinmeyen stream: {stream_id}")})),
        )),
    }
}

async fn api_stream_candles(
    State(state): State<Arc<AppState>>,
    Path(stream_id): Path<u64>,
    Query(params): Query<CandlesParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let streams = state.streams.read().await;
    match streams.get(&stream_id) {
        Some(s) => {
            let st = s.state.lock().await;
            let limit = params.limit.unwrap_or(50).min(CACHE_MAX);
            let candles: Vec<StreamCandle> = st.cache.iter().rev().take(limit).cloned().collect();
            let mut current = st.current.clone();
            if let Some(c) = current.as_mut() {
                c.close_time = now_ms();
            }
            Ok(Json(serde_json::json!({
                "stream_id": stream_id,
                "current": current,
                "count": candles.len(),
                "candles": candles,
            })))
        }
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("bilinmeyen stream: {stream_id}")})),
        )),
    }
}

async fn api_stream_stop(
    State(state): State<Arc<AppState>>,
    Path(stream_id): Path<u64>,
) -> Json<serde_json::Value> {
    if stop_stream(&state, stream_id).await {
        Json(serde_json::json!({"status": "stopped", "stream_id": stream_id}))
    } else {
        Json(serde_json::json!({"error": format!("bilinmeyen stream: {stream_id}")}))
    }
}

async fn api_health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let streams = state.streams.read().await;
    let mut detail = Vec::new();
    for s in streams.values() {
        let st = s.state.lock().await;
        detail.push(serde_json::json!({
            "stream_id": s.id,
            "symbol": s.symbol,
            "interval_secs": s.interval_secs,
            "status": st.status,
            "published": st.published,
        }));
    }
    Json(serde_json::json!({
        "status": "ok",
        "time": Utc::now().to_rfc3339(),
        "ring": RING_NAME,
        "stream_count": streams.len(),
        "streams": detail,
    }))
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("STREAM_OHLCV_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    println!("══════════════════════════════════════════════════");
    println!("  📡 STREAM-OHLCV — Canlı OHLCV Mum Akışı");
    println!("  Ring : {RING_NAME} (RAM, binary)");
    println!("  Fiyat: lastprice flow ring (RAM)");
    println!("  API  : http://127.0.0.1:{port}/api/stream");
    println!("══════════════════════════════════════════════════");

    let ring = Arc::new(StreamRingBuffer::with_name(RING_NAME, STREAM_DEFAULT_CAPACITY));

    let state = Arc::new(AppState {
        client: ohlcv_engine::client::BinanceClient::new(),
        ring,
        ring_lock: Arc::new(Mutex::new(())),
        streams: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        by_key: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        next_id: AtomicU64::new(1),
    });

    let app = Router::new()
        .route("/api/stream", post(api_stream_start))
        .route("/api/streams", get(api_streams))
        .route("/api/stream/{stream_id}", get(api_stream_get).delete(api_stream_stop))
        .route("/api/stream/{stream_id}/candles", get(api_stream_candles))
        .route("/api/health", get(api_health))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("port bind");
    axum::serve(listener, app).await.expect("serve");
}
