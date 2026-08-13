//! force-orders servisi — Binance Futures canlı likidasyon (forceOrder) izleyici.
//!
//! Akış:
//!
//! ```text
//! Binance Futures WS (!forceOrder@arr ya da <symbol>@forceOrder)
//!   └── ws pump task (tokio, reconnect + backoff)
//!        └── ForceOrder parse + filtre (min qty / min notional, opsiyonel)
//!             ├── /dev/shm/cycle_finance_force_orders ring'e binary yayın
//!             ├── HTTP cache (son N likidasyon) + özet istatistik
//!             └── stdout canlı tablo (tmux canlı görünüm)
//! ```
//!
//! Ring yayını, codebase'in kanonik `Liquidation` event'ini (`transport::wire`)
//! taşır — tüketici servisler `wire::decode` ile okuyabilir. Ayrıca
//! `force_orders::codec` ile bu servise özgü tam kayıt (`ForceOrder`) da
//! yayınlanmaz; ring sadece kanonik event'i taşır, tüm ayrıntı HTTP API'den alınır.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use force_orders::{ForceOrder, RING_CAPACITY, RING_NAME};
use serde::Deserialize;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_tungstenite::tungstenite::Message;

const DEFAULT_PORT: u16 = 3012;
const CACHE_MAX: usize = 10_000;
const WS_BASE: &str = "wss://fstream.binance.com/market/ws";

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ── Paylaşılan durum ─────────────────────────────────────────
struct AppState {
    ring: Arc<transport::stream_ring::StreamRingBuffer>,
    ring_lock: Arc<Mutex<()>>,
    orders: Arc<Mutex<VecDeque<ForceOrder>>>,
    received: AtomicU64,
    started: AtomicU64,
    ws_connected: AtomicU64,
    ws_stream: Mutex<String>,
}

/// forceOrder event gövdesi (WS ham JSON'dan parse edilir).
#[derive(Deserialize)]
struct RawForceOrder {
    #[serde(rename = "E")]
    event_ts: u64,
    #[serde(rename = "o")]
    order: RawOrder,
}

/// Binance sayısal alanları hem `"0.014"` (string) hem `0.014` (sayı) olarak
/// gönderebilir — her ikisini de f64'e çözer.
fn de_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    struct V;
    impl serde::de::Visitor<'_> for V {
        type Value = f64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("sayısal değer (sayı veya string)")
        }
        fn visit_f64<E: Error>(self, v: f64) -> Result<f64, E> {
            Ok(v)
        }
        fn visit_i64<E: Error>(self, v: i64) -> Result<f64, E> {
            Ok(v as f64)
        }
        fn visit_u64<E: Error>(self, v: u64) -> Result<f64, E> {
            Ok(v as f64)
        }
        fn visit_str<E: Error>(self, v: &str) -> Result<f64, E> {
            v.parse().map_err(|_| E::custom(format!("geçersiz sayı: {v}")))
        }
    }
    deserializer.deserialize_any(V)
}

#[derive(Deserialize)]
struct RawOrder {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "S")]
    side: String,
    #[serde(rename = "o")]
    order_type: String,
    #[serde(rename = "q", deserialize_with = "de_f64")]
    qty: f64,
    #[serde(rename = "p", deserialize_with = "de_f64")]
    price: f64,
    #[serde(rename = "ap", deserialize_with = "de_f64")]
    avg_price: f64,
    #[serde(rename = "z", deserialize_with = "de_f64")]
    filled: f64,
    #[serde(rename = "T")]
    trade_ts: u64,
}

// ── Yardımcılar ──────────────────────────────────────────────
fn fmt_time(ms: u64) -> String {
    chrono::DateTime::from_timestamp_millis(ms as i64)
        .map(|dt| dt.format("%H:%M:%S%.3f").to_string())
        .unwrap_or_else(|| ms.to_string())
}

/// Ring'e compact binary `ForceOrder` kaydı yayınlar (force_orders::codec).
fn publish(app: &AppState, o: &ForceOrder) {
    let bytes = force_orders::codec::encode(o);
    let _g = app.ring_lock.lock().unwrap();
    app.ring.push(&bytes);
}

/// Tek likidasyonu stdout'a tek satır olarak stream eder (tmux canlı görünüm).
fn stream_line(o: &ForceOrder) {
    println!(
        "[{}] {:<14} {:<5} {:>12.2} {:>12.4} {:>14.2}  {}",
        fmt_time(o.event_ts),
        o.symbol,
        o.side,
        o.price,
        o.qty,
        o.notional,
        o.order_type
    );
}

fn on_force_order(app: &Arc<AppState>, min_qty: f64, min_notional: f64, raw: &RawForceOrder) {
    let o = ForceOrder {
        symbol: raw.order.symbol.to_uppercase(),
        side: raw.order.side.to_uppercase(),
        order_type: raw.order.order_type.clone(),
        price: raw.order.price,
        avg_price: raw.order.avg_price,
        qty: raw.order.qty,
        filled: raw.order.filled,
        notional: raw.order.avg_price * raw.order.qty,
        event_ts: raw.event_ts,
        trade_ts: raw.order.trade_ts,
    };
    if min_qty > 0.0 && o.qty < min_qty {
        return;
    }
    if min_notional > 0.0 && o.notional < min_notional {
        return;
    }
    app.received.fetch_add(1, Ordering::SeqCst);
    {
        let mut cache = app.orders.lock().unwrap();
        cache.push_back(o.clone());
        if cache.len() > CACHE_MAX {
            cache.pop_front();
        }
    }
    publish(app, &o);
    stream_line(&o);
}

// ── WS pump (reconnect + backoff) ────────────────────────────
async fn ws_pump(app: Arc<AppState>, min_qty: f64, min_notional: f64) {
    use futures_util::{SinkExt, StreamExt};

    let symbol = std::env::var("FORCE_ORDERS_SYMBOL")
        .ok()
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty());

    let url = match &symbol {
        Some(s) => format!("{WS_BASE}/{s}@forceOrder"),
        None => format!("{WS_BASE}/!forceOrder@arr"),
    };

    let mut backoff = 1u64;
    loop {
        match tokio_tungstenite::connect_async(&url).await {
            Ok((mut ws, _)) => {
                backoff = 1;
                *app.ws_stream.lock().unwrap() = url.clone();
                app.ws_connected.store(now_ms(), Ordering::SeqCst);
                println!(
                    "✅ forceOrder stream'ine bağlanıldı: {url}\n\n\
                     {:^24} {:<14} {:<5} {:>12} {:>12} {:>14}",
                    "TIME (UTC)", "SYMBOL", "SIDE", "PRICE", "QTY", "NOTIONAL"
                );

                while let Some(msg) = ws.next().await {
                    let text = match msg {
                        Ok(Message::Text(t)) => t,
                        Ok(Message::Ping(p)) => {
                            let _ = ws.send(Message::Pong(p)).await;
                            continue;
                        }
                        Ok(Message::Pong(_)) => continue,
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("[FORCE] WS okuma hatası: {e}");
                            break;
                        }
                    };
                    let parsed: serde_json::Value = match serde_json::from_str(&text) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("[FORCE] JSON parse hatası: {e}");
                            continue;
                        }
                    };
                    if parsed.get("e").and_then(|e| e.as_str()) != Some("forceOrder") {
                        continue;
                    }
                    let raw: RawForceOrder = match serde_json::from_value(parsed) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("[FORCE] forceOrder parse hatası: {e}");
                            continue;
                        }
                    };
                    on_force_order(&app, min_qty, min_notional, &raw);
                }
                app.ws_connected.store(0, Ordering::SeqCst);
                println!("[FORCE] WS bağlantısı kapandı.");
            }
            Err(e) => {
                eprintln!("[FORCE] WS bağlantı hatası: {e}");
            }
        }

        println!("[FORCE] {backoff} sn sonra yeniden bağlanılıyor...");
        tokio::time::sleep(Duration::from_secs(backoff)).await;
        backoff = (backoff * 2).min(60);
    }
}

// ── HTTP API ─────────────────────────────────────────────────
#[derive(Deserialize)]
struct ListParams {
    limit: Option<usize>,
    symbol: Option<String>,
    side: Option<String>,
    min_notional: Option<f64>,
}

async fn api_health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let connected = state.ws_connected.load(Ordering::SeqCst) != 0;
    Json(serde_json::json!({
        "status": "ok",
        "time": Utc::now().to_rfc3339(),
        "ring": RING_NAME,
        "ws_connected": connected,
        "ws_stream": state.ws_stream.lock().unwrap().clone(),
        "received": state.received.load(Ordering::SeqCst),
        "cached": state.orders.lock().unwrap().len(),
        "started_ms": state.started.load(Ordering::SeqCst),
    }))
}

async fn api_liquidations(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(50).min(CACHE_MAX);
    let symbol = params.symbol.map(|s| s.to_uppercase());
    let side = params.side.map(|s| s.to_uppercase());

    let cache = state.orders.lock().unwrap();
    let mut out: Vec<&ForceOrder> = cache
        .iter()
        .rev()
        .filter(|o| symbol.as_ref().map_or(true, |s| &o.symbol == s))
        .filter(|o| side.as_ref().map_or(true, |s| &o.side == s))
        .filter(|o| {
            params
                .min_notional
                .map_or(true, |mn| o.notional >= mn)
        })
        .take(limit)
        .collect();
    out.sort_by_key(|o| std::cmp::Reverse(o.event_ts));
    Json(serde_json::json!({
        "count": out.len(),
        "liquidations": out,
    }))
}

async fn api_liquidations_symbol(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<ListParams>,
) -> Json<serde_json::Value> {
    let sym = symbol.to_uppercase();
    let limit = params.limit.unwrap_or(50).min(CACHE_MAX);
    let cache = state.orders.lock().unwrap();
    let mut out: Vec<&ForceOrder> = cache
        .iter()
        .rev()
        .filter(|o| o.symbol == sym)
        .take(limit)
        .collect();
    out.sort_by_key(|o| std::cmp::Reverse(o.event_ts));
    Json(serde_json::json!({
        "symbol": sym,
        "count": out.len(),
        "liquidations": out,
    }))
}

async fn api_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let cache = state.orders.lock().unwrap();
    let mut total_notional = 0.0f64;
    let mut buy = (0u64, 0.0f64);
    let mut sell = (0u64, 0.0f64);
    for o in cache.iter() {
        total_notional += o.notional;
        if o.side == "BUY" {
            buy.0 += 1;
            buy.1 += o.notional;
        } else {
            sell.0 += 1;
            sell.1 += o.notional;
        }
    }
    let last = cache.iter().rev().take(10).cloned().collect::<Vec<_>>();
    drop(cache);
    Json(serde_json::json!({
        "received_total": state.received.load(Ordering::SeqCst),
        "cached": state.orders.lock().unwrap().len(),
        "total_notional": total_notional,
        "long_liq_sell": { "count": sell.0, "notional": sell.1 },
        "short_liq_buy": { "count": buy.0, "notional": buy.1 },
        "last": last,
    }))
}

#[tokio::main]
async fn main() {
    let _ = infra::util::single_instance("force-orders");
    let port: u16 = std::env::var("FORCE_ORDERS_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let min_qty: f64 = std::env::var("FORCE_ORDERS_MIN_QTY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let min_notional: f64 = std::env::var("FORCE_ORDERS_MIN_NOTIONAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);

    println!("══════════════════════════════════════════════════");
    println!("  💥 FORCE-ORDERS — Canlı Likidasyon İzleyici");
    println!("  Ring : {RING_NAME} (RAM, binary)");
    println!("  Filtre: min_qty={min_qty}  min_notional={min_notional}");
    println!("  API  : http://127.0.0.1:{port}/api/liquidations");
    println!("         http://127.0.0.1:{port}/api/stats");
    println!("══════════════════════════════════════════════════");

    let ring = Arc::new(transport::stream_ring::StreamRingBuffer::with_name(
        RING_NAME,
        RING_CAPACITY,
    ));
    let state = Arc::new(AppState {
        ring,
        ring_lock: Arc::new(Mutex::new(())),
        orders: Arc::new(Mutex::new(VecDeque::new())),
        received: AtomicU64::new(0),
        started: AtomicU64::new(now_ms()),
        ws_connected: AtomicU64::new(0),
        ws_stream: Mutex::new(String::new()),
    });

    tokio::spawn(ws_pump(state.clone(), min_qty, min_notional));

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/liquidations", get(api_liquidations))
        .route("/api/liquidations/{symbol}", get(api_liquidations_symbol))
        .route("/api/stats", get(api_stats))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = infra::util::bind_or_exit(addr, "force-orders").await;
    axum::serve(listener, app).await.expect("serve");
}
