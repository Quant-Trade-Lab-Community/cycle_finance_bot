//! trade-ohlcv servisi — trade data'dan 1 saniyelik OHLCV mumu üreten daemon.
//!
//! Akış:
//!
//! ```text
//! /dev/shm/cycle_finance_trades (flow ring, core DATA terminali üretir)
//!   └── trade kaynak thread (std thread, spin-loop)
//!        └── flume bounded kanal (Trade event'leri)
//!             └── toplayıcı task (tokio) → SecondAggregator → 1s mum
//!                  ├── kapanan mum → /dev/shm/cycle_finance_trade_ohlcv ring'e yayınla
//!                  └── her kapalı mum stdout'a stream edilir (tmux canlı görünüm)
//! ```
//!
//! Her sembol için trade'lerin timestamp'i 1s dilimine (bucket) bölünür; dilim
//! değişince bar kapanır ve binary olarak yayınlanır. HTTP API son N kapalı
//! mumu + oluşan mumu döndürür.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use rust_decimal::prelude::ToPrimitive;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use trade_ohlcv::codec;
use trade_ohlcv::{RING_CAPACITY, RING_NAME, SecondAggregator, TradeCandle};
use transport::events::EventType;
use transport::flow::FlowKind;
use transport::ring_buffer::GenerationalRingBuffer;
use transport::stream_ring::StreamRingBuffer;
use transport::wire;

const DEFAULT_PORT: u16 = 3009;
const TRADES_RING: &str = "/cycle_finance_trades";
const CHANNEL_CAP: usize = 262_144;
const CACHE_MAX: usize = 500;
const REPORT_EVERY: Duration = Duration::from_secs(30);

// ── Paylaşılan durum ─────────────────────────────────────────
struct AppState {
    ring: Arc<StreamRingBuffer>,
    ring_lock: Arc<Mutex<()>>,
    symbols: Arc<tokio::sync::RwLock<HashMap<String, SymbolState>>>,
    published: AtomicU64,
}

struct SymbolState {
    /// Kapanan son mumlar (API/status için).
    candles: VecDeque<TradeCandle>,
    /// Şu an oluşan mum (canlı güncellenen).
    current: Option<TradeCandle>,
}

impl Default for SymbolState {
    fn default() -> Self {
        Self {
            candles: VecDeque::new(),
            current: None,
        }
    }
}

/// Ring okuyucudan toplayıcıya geçen tek trade.
struct IngestTrade {
    symbol: String,
    price: f64,
    quantity: f64,
    ts_ms: u64,
    is_buyer_maker: bool,
}

fn decode_symbol(buf: &[u8; 16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(16);
    String::from_utf8_lossy(&buf[..len]).to_string()
}

// ── Trade kaynak thread (spin-loop, flow ring okuyucu) ───────
/// `seq`'deki slot'u okur, Trade event'ini kanala iletir. Slot yoksa `false`.
fn ingest_slot(
    ring: &GenerationalRingBuffer,
    seq: u64,
    tx: &flume::Sender<IngestTrade>,
) -> bool {
    let Some(slot) = ring.read_slot(seq) else {
        return false;
    };
    // Yeni (sıfırlanmış) ring'de seq=0 slot'ları boş görünür ve `read_slot`
    // seq=0 için yanlışlıkla Some döner — içerik doğrulaması şart.
    if slot.len == 0 {
        return false;
    }
    let Some(ev) = wire::decode(&slot.data[..slot.len as usize]) else {
        return false;
    };
    if let EventType::Trade {
        price,
        quantity,
        timestamp,
        is_buyer_maker,
    } = ev.payload
    {
        let sym = decode_symbol(&ev.symbol);
        if !sym.is_empty() {
            let price_f = price.to_f64().unwrap_or(0.0);
            if price_f > 0.0 {
                let qty_f = quantity.to_f64().unwrap_or(0.0);
                let _ = tx.send(IngestTrade {
                    symbol: sym,
                    price: price_f,
                    quantity: qty_f,
                    ts_ms: timestamp,
                    is_buyer_maker,
                });
            }
        }
    }
    true
}

fn spawn_trade_source(tx: flume::Sender<IngestTrade>) {
    std::thread::spawn(move || {
        let ring = GenerationalRingBuffer::with_name(TRADES_RING, FlowKind::Trade.ring_capacity());
        let mut cursor = ring.get_head();

        loop {
            if ingest_slot(&ring, cursor, &tx) {
                cursor += 1;
                continue;
            }

            // Slot henüz yazılmamış olabilir (üretici 0.5µs-ms ölçeğinde yazıyor).
            // Körlemesine head'e atlamak yeni event'i kaçırır; önce kısa bir
            // retry penceresiyle bekleyip tekrar dene.
            let mut advanced = false;
            for _ in 0..40 {
                std::thread::sleep(Duration::from_micros(100));
                if ingest_slot(&ring, cursor, &tx) {
                    cursor += 1;
                    advanced = true;
                    break;
                }
            }
            if advanced {
                continue;
            }

            // ~4ms boyunca okunamadı ve üretici bizim önümüzdeyse slot
            // overwrite olmuştur (üretici hızlı) — cursor'ı güncel konuma taşı.
            let head = ring.get_head();
            if head > cursor {
                cursor = head;
            } else {
                std::thread::sleep(Duration::from_micros(500));
            }
        }
    });
}

// ── Toplayıcı task ───────────────────────────────────────────
fn publish(app: &AppState, candle: &TradeCandle) {
    let bytes = codec::encode(candle);
    let _g = app.ring_lock.lock().unwrap();
    app.ring.push(&bytes);
    app.published.fetch_add(1, Ordering::SeqCst);
}

fn fmt_time(ms: u64) -> String {
    chrono::DateTime::from_timestamp_millis(ms as i64)
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| ms.to_string())
}

/// 8 ondalığa yuvarlayarak kayan nokta gürültüsünü (0.43000000000000005) temizler.
fn fmt_num(v: f64) -> String {
    let r = (v * 1e8).round() / 1e8;
    format!("{r}")
}

/// Kapanan her mumu tek satır olarak stdout'a stream eder (tmux canlı görünüm).
fn stream_line(c: &TradeCandle) {
    println!(
        "[{}] {}  1s  O={}  H={}  L={}  C={}  V={}  TB={}  n={}",
        fmt_time(c.open_time),
        c.symbol,
        fmt_num(c.open),
        fmt_num(c.high),
        fmt_num(c.low),
        fmt_num(c.close),
        fmt_num(c.volume),
        fmt_num(c.taker_buy_volume),
        c.trades
    );
}

async fn run_aggregator(rx: flume::Receiver<IngestTrade>, app: Arc<AppState>) {
    let mut agg = SecondAggregator::new();
    let mut last_report = SystemTime::now();

    while let Ok(t) = rx.recv_async().await {
        let closed = agg.on_trade(&t.symbol, t.price, t.quantity, t.ts_ms, t.is_buyer_maker);

        let key = closed
            .as_ref()
            .map(|c| c.symbol.clone())
            .unwrap_or_else(|| t.symbol.clone());

        let forming = agg.forming(&key);
        {
            let mut symbols = app.symbols.write().await;
            let entry = symbols.entry(key.clone()).or_default();
            entry.current = forming;
            if let Some(c) = closed {
                entry.candles.push_back(c.clone());
                if entry.candles.len() > CACHE_MAX {
                    entry.candles.pop_front();
                }
                publish(&app, &c);
                stream_line(&c);
            }
        }

        if last_report.elapsed().unwrap_or_default() >= REPORT_EVERY {
            let symbols = app.symbols.read().await;
            println!(
                "── durum: sembol={} formi={} yayinlanan={} ring_head={} ──",
                symbols.len(),
                agg.symbol_count(),
                app.published.load(Ordering::SeqCst),
                app.ring.get_head(),
            );
            drop(symbols);
            last_report = SystemTime::now();
        }
    }
}

// ── HTTP API ─────────────────────────────────────────────────
#[derive(Deserialize)]
struct CandlesParams {
    limit: Option<usize>,
}

async fn api_health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let symbols = state.symbols.read().await;
    Json(serde_json::json!({
        "status": "ok",
        "time": Utc::now().to_rfc3339(),
        "ring": RING_NAME,
        "trades_ring": TRADES_RING,
        "published": state.published.load(Ordering::SeqCst),
        "symbol_count": symbols.len(),
        "symbols": symbols.keys().collect::<Vec<_>>(),
    }))
}

async fn api_symbols(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let symbols = state.symbols.read().await;
    let mut out = Vec::new();
    for (sym, st) in symbols.iter() {
        out.push(serde_json::json!({
            "symbol": sym,
            "current": st.current,
            "closed": st.candles.len(),
        }));
    }
    Json(serde_json::json!(out))
}

async fn api_candles(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<CandlesParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let sym = symbol.to_ascii_uppercase();
    let symbols = state.symbols.read().await;
    match symbols.get(&sym) {
        Some(st) => {
            let limit = params.limit.unwrap_or(50).min(CACHE_MAX);
            let candles: Vec<TradeCandle> = st.candles.iter().rev().take(limit).cloned().collect();
            Ok(Json(serde_json::json!({
                "symbol": sym,
                "current": st.current,
                "count": candles.len(),
                "candles": candles,
            })))
        }
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("bilinmeyen sembol: {sym}")})),
        )),
    }
}

#[tokio::main]
async fn main() {
    let _ = infra::util::single_instance("trade-ohlcv");
    let port: u16 = std::env::var("TRADE_OHLCV_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    println!("══════════════════════════════════════════════════");
    println!("  ⏱  TRADE-OHLCV — Trade Data → 1s OHLCV Mum Akışı");
    println!("  Kaynak: {TRADES_RING} (flow ring, RAM)");
    println!("  Ring  : {RING_NAME} (RAM, binary)");
    println!("  API   : http://127.0.0.1:{port}/api/candles/{{symbol}}");
    println!("══════════════════════════════════════════════════");

    let ring = Arc::new(StreamRingBuffer::with_name(RING_NAME, RING_CAPACITY));
    let state = Arc::new(AppState {
        ring,
        ring_lock: Arc::new(Mutex::new(())),
        symbols: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        published: AtomicU64::new(0),
    });

    let (tx, rx) = flume::bounded::<IngestTrade>(CHANNEL_CAP);
    spawn_trade_source(tx);
    tokio::spawn(run_aggregator(rx, state.clone()));

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/symbols", get(api_symbols))
        .route("/api/candles/{symbol}", get(api_candles))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = infra::util::bind_or_exit(addr, "trade-ohlcv").await;
    axum::serve(listener, app).await.expect("serve");
}
