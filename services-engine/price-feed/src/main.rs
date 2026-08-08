//! Price Feed — Binance Futures WS'ten mark/index/last price çeken daemon.
//!
//! Mimari, DATA terminaliyle birebir aynıdır:
//!   Binance WS → simd_json EventParser → GenerationalRingBuffer (/dev/shm)
//!
//! Fark: kendi ring buffer'ını kullanır (/cycle_finance_pricefeed) ve ayrıca
//! HTTP API + JSON dosya ile son fiyatları diğer katmanlara sunar.
//!
//! Abonelikler (fstream.binance.com):
//!   {SYM}@markPrice@1s  → mark + index price (FundingRate event)
//!   {SYM}@bookTicker@1s → best bid/ask (BookTicker event)
//!
//! HTTP:
//!   GET /api/lastprice            → tüm semboller {last, mark, index}
//!   GET /api/lastprice/{SYMBOL}   → tek sembol
//!   GET /health

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use flume::Sender;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use transport::ring_buffer::GenerationalRingBuffer;
use contracts::events::{EventType, OwnedEvent};
use proje_core::tick::EventParser;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::connect_async;

const WS_URL: &str = "wss://fstream.binance.com/stream";
const DEFAULT_PORT: u16 = 3004;
const RING_NAME: &str = "/cycle_finance_pricefeed";
const RING_CAPACITY: usize = 20_000;
const OUT_FILE: &str = "/tmp/price_feed.json";

// ── Semboller ────────────────────────────────────────────────
fn load_symbols() -> Vec<String> {
    let mut syms: Vec<String> = Vec::new();
    if let Ok(content) = std::fs::read_to_string("alerts.toml") {
        for line in content.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("symbol") {
                if let Some(eq) = rest.find('=') {
                    let s = rest[eq + 1..].trim().trim_matches('"').trim_matches('\'').trim().to_string();
                    if !s.is_empty() && !syms.contains(&s) {
                        syms.push(s);
                    }
                }
            }
        }
    }
    if !syms.contains(&"HEIUSDT".to_string()) {
        syms.push("HEIUSDT".to_string());
    }
    syms
}

fn resolve_symbols() -> Vec<String> {
    if let Ok(v) = std::env::var("PRICE_FEED_SYMBOLS") {
        let s: Vec<String> = v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
        if !s.is_empty() {
            return s;
        }
    }
    load_symbols()
}

// ── Paylaşılan durum ─────────────────────────────────────────
#[derive(Debug, Clone, Default, Serialize)]
struct PriceEntry {
    last: f64,
    mark: f64,
    index: f64,
    bid: f64,
    ask: f64,
    ts: u64,
}

#[derive(Debug, Default)]
struct FeedState {
    prices: HashMap<String, PriceEntry>,
    symbols: Vec<String>,
}

fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

// ── WS bağlantısı — DATA terminaliyle aynı desen ────────────
async fn ws_pump(tx: Sender<Vec<u8>>, symbols: Vec<String>) {
    let streams: Vec<String> = symbols
        .iter()
        .flat_map(|s| {
            let s = s.to_lowercase();
            vec![
                format!("{}@trade", s),
                format!("{}@bookTicker", s),
            ]
        })
        .collect();

    loop {
        println!("[PRICE-FEED] WS bağlanıyor ({} stream)...", streams.len());
        match connect_async(WS_URL).await {
            Ok((ws, _)) => {
                let (mut write, mut read) = ws.split();
                let sub = serde_json::json!({
                    "method": "SUBSCRIBE",
                    "params": streams,
                    "id": 1
                });
                if write.send(tokio_tungstenite::tungstenite::Message::Text(sub.to_string())).await.is_err() {
                    eprintln!("[PRICE-FEED] Abonelik gönderilemedi");
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    continue;
                }
                println!("[PRICE-FEED] Bağlandı ve abone olundu.");

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                            let bytes = text.into_bytes();
                            // Bounded kuyruk → geri basınç (asla RAM taşmaz).
                            if tx.send_async(bytes).await.is_err() {
                                eprintln!("[PRICE-FEED] Kuyruk kapandı, çıkılıyor.");
                                return;
                            }
                        }
                        Ok(tokio_tungstenite::tungstenite::Message::Ping(p)) => {
                            let _ = write.send(tokio_tungstenite::tungstenite::Message::Pong(p)).await;
                        }
                        Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
                        _ => {}
                    }
                }
                println!("[PRICE-FEED] Bağlantı koptu, yeniden bağlanılıyor...");
            }
            Err(e) => {
                eprintln!("[PRICE-FEED] Bağlantı hatası: {}", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

// ── Mark/Index REST çekimi (fstream WS markPrice sessiz → premiumIndex) ──
async fn fetch_premium_index(client: &reqwest::Client, symbols: &[String], state: Arc<RwLock<FeedState>>) {
    for sym in symbols {
        let url = format!("https://fapi.binance.com/fapi/v1/premiumIndex?symbol={}", sym);
        let resp = client.get(&url).send().await;
        if let Ok(r) = resp {
            if let Ok(doc) = r.json::<serde_json::Value>().await {
                let mark = doc.get("markPrice").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let index = doc.get("indexPrice").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let ts = now_ts();
                let mut st = state.write();
                let e = st.prices.entry(sym.clone()).or_default();
                e.ts = ts;
                if let Some(m) = mark {
                    e.mark = m;
                }
                if let Some(i) = index {
                    e.index = i;
                }
            }
        }
    }
}

// ── Parser + ring buffer + state güncelleme — DATA ile aynı ─
fn ingest(rx: flume::Receiver<Vec<u8>>, ring: Arc<GenerationalRingBuffer>, state: Arc<RwLock<FeedState>>) {
    let mut validator = proje_core::validator::DataValidator::new();
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut last_report = std::time::Instant::now();
    let mut frame_buf = [0u8; contracts::wire::MAX_FRAME_SIZE];

    while let Ok(mut bytes) = rx.recv() {
        if let Some(ev) = EventParser::parse(&mut bytes) {
            if !validator.is_valid(&ev) {
                continue;
            }
            // DATA ile aynı: ring'e typed binary yazılır (ham JSON değil).
            if let Some(len) = contracts::wire::encode(&ev, &mut frame_buf) {
                ring.push(&frame_buf[..len]);
            }
            update_state(&state, &ev);
            ok += 1;
        }
        total += 1;

        if last_report.elapsed().as_secs() >= 1 {
            println!("[PRICE-FEED] ticks/s: {} | parsed: {}", total, ok);
            total = 0;
            ok = 0;
            last_report = std::time::Instant::now();
        }
    }
}

fn update_state(state: &Arc<RwLock<FeedState>>, ev: &OwnedEvent) {
    let sym = std::str::from_utf8(&ev.symbol)
        .unwrap_or("")
        .trim_end_matches('\0')
        .to_uppercase();
    if sym.is_empty() {
        return;
    }
    let ts = now_ts();
    let mut st = state.write();
    let e = st.prices.entry(sym.clone()).or_default();
    e.ts = ts;
    match ev.payload {
        EventType::Trade { price, .. } => {
            e.last = price.to_f64().unwrap_or(0.0);
        }
        EventType::FundingRate { mark_price, index_price, .. } => {
            e.mark = mark_price.to_f64().unwrap_or(0.0);
            e.index = index_price.to_f64().unwrap_or(0.0);
        }
        EventType::BookTicker { best_bid_price, best_ask_price, .. } => {
            e.bid = best_bid_price.to_f64().unwrap_or(0.0);
            e.ask = best_ask_price.to_f64().unwrap_or(0.0);
        }
        _ => {}
    }
    drop(st);
}

// ── HTTP API ─────────────────────────────────────────────────
#[derive(Serialize)]
struct ApiAll {
    updated: u64,
    symbols: Vec<String>,
    prices: HashMap<String, PriceEntry>,
}

#[derive(Serialize)]
struct ApiOne {
    symbol: String,
    price: PriceEntry,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    symbols: Vec<String>,
    prices: HashMap<String, f64>,
}

async fn api_all(State(state): State<Arc<RwLock<FeedState>>>) -> Json<ApiAll> {
    let st = state.read();
    Json(ApiAll {
        updated: now_ts(),
        symbols: st.symbols.clone(),
        prices: st.prices.clone(),
    })
}

async fn api_one(
    State(state): State<Arc<RwLock<FeedState>>>,
    Path(symbol): Path<String>,
) -> Result<Json<ApiOne>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let st = state.read();
    let key = symbol.to_uppercase();
    match st.prices.get(&key) {
        Some(e) => Ok(Json(ApiOne { symbol: key, price: e.clone() })),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("bilinmeyen sembol: {key}"), "available": st.symbols})),
        )),
    }
}

async fn api_health(State(state): State<Arc<RwLock<FeedState>>>) -> Json<Health> {
    let st = state.read();
    Json(Health {
        status: "ok",
        symbols: st.symbols.clone(),
        prices: st.prices.iter().map(|(k, v)| (k.clone(), v.mark.max(v.last))).collect(),
    })
}

#[tokio::main]
async fn main() {
    let symbols = resolve_symbols();
    let port: u16 = std::env::var("PRICE_FEED_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_PORT);

    println!("========================================");
    println!("  💹  PRICE FEED — Anlık LastPrice Daemon");
    println!("  Mimari : WS → EventParser → RingBuffer (/dev/shm)");
    println!("  Semboller : {}", symbols.join(", "));
    println!("  HTTP API  : http://127.0.0.1:{}/api/lastprice", port);
    println!("  JSON çıktı: {}", OUT_FILE);
    println!("========================================");

    let ring = Arc::new(GenerationalRingBuffer::with_name(RING_NAME, RING_CAPACITY));
    let state = Arc::new(RwLock::new(FeedState {
        symbols: symbols.clone(),
        ..Default::default()
    }));

    let (tx, rx) = flume::bounded::<Vec<u8>>(262_144);

    // WS pump task
    let symbols_ws = symbols.clone();
    tokio::spawn(async move { ws_pump(tx, symbols_ws).await });

    // Mark/Index REST döngüsü — Binance fstream markPrice WS stream'i sessiz
    // olduğundan premiumIndex'i çok sık (200ms) çekerek pratikte gecikmesiz.
    {
        let client = reqwest::Client::new();
        let state = state.clone();
        let symbols_rest = symbols.clone();
        tokio::spawn(async move {
            loop {
                fetch_premium_index(&client, &symbols_rest, state.clone()).await;
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        });
    }

    // Ingest thread (parser + ring + state)
    {
        let ring = ring.clone();
        let state = state.clone();
        std::thread::spawn(move || ingest(rx, ring, state));
    }

    // JSON dosya yazıcı (periyodik)
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let doc = {
                    let st = state.read();
                    serde_json::json!({
                        "updated": now_ts(),
                        "symbols": st.symbols.clone(),
                        "prices": st.prices.clone(),
                    })
                };
                let _ = std::fs::write(OUT_FILE, serde_json::to_string_pretty(&doc).unwrap_or_default());
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    let app = Router::new()
        .route("/api/lastprice", get(api_all))
        .route("/api/lastprice/{symbol}", get(api_one))
        .route("/health", get(api_health))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("port bind");
    axum::serve(listener, app).await.expect("serve");
}
