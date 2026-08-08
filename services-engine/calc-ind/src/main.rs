//! calc-ind servisi — indikatör hesaplama motoru.
//!
//! POST /api/calc
//!   { symbol, interval, start_ms?, end_ms?, indicator, params{} }
//!   → ohlcv-engine'den veri çeker, ferro_ta_core ile indikatör hesaplar,
//!     sonucu binary olarak `/dev/shm/cycle_finance_calc` ring'ine yazar,
//!     { request_id } döndürür.
//!
//! GET /api/health → { status: "ok" }

use axum::{extract::State, routing::post, Json, Router};
use calc_ind::indicators::{self, IndicatorSeries};
use calc_ind::{CalcKline, CalcResult, IndRequest, codec};
use ohlcv_engine::client::BinanceClient;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use transport::calc_ring::CalcRingBuffer;

const RING_NAME: &str = "/cycle_finance_calc";
const RING_CAPACITY: usize = 64;

struct AppState {
    client: BinanceClient,
    ring: CalcRingBuffer,
    next_id: AtomicU64,
}

#[tokio::main]
async fn main() {
    println!("══════════════════════════════════════════════════");
    println!("  🧮 CALC-IND — İNDİKATÖR HESAPLAMA MOTORU");
    println!("  ferro_ta_core | ring: {RING_NAME}");
    println!("  API: http://127.0.0.1:3007/api/calc");
    println!("══════════════════════════════════════════════════");

    let state = Arc::new(AppState {
        client: BinanceClient::new(),
        ring: CalcRingBuffer::with_name(RING_NAME, RING_CAPACITY),
        next_id: AtomicU64::new(1),
    });

    let app = Router::new()
        .route("/api/calc", post(handle_calc))
        .route("/api/health", axum::routing::get(|| async { Json(serde_json::json!({"status": "ok"})) }))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3007));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_calc(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IndRequest>,
) -> Json<serde_json::Value> {
    let limit = 1000; // tek istekte maks. kline
    let klines = match state
        .client
        .fetch_klines_range(&req.symbol, &req.interval, req.start_ms, req.end_ms, limit)
        .await
    {
        Ok(k) if !k.is_empty() => k,
        Ok(_) => {
            return Json(serde_json::json!({"error": "Veri bulunamadı"}));
        }
        Err(e) => {
            return Json(serde_json::json!({"error": format!("OHLCV çekilemedi: {e}")}));
        }
    };

    let series: IndicatorSeries = match indicators::calc_indicator(&req.indicator, &klines, &req.params) {
        Ok(s) => s,
        Err(e) => return Json(serde_json::json!({"error": e})),
    };

    let request_id = state.next_id.fetch_add(1, Ordering::SeqCst);

    let result = CalcResult {
        request_id,
        symbol: req.symbol.clone(),
        interval: req.interval.clone(),
        indicator: req.indicator.clone(),
        klines: klines.iter().map(to_calc_kline).collect(),
        series,
    };

    // Binary olarak ring'e yayınla
    state.ring.push(&codec::encode(&result));

    Json(serde_json::json!({
        "status": "success",
        "request_id": request_id,
        "count": klines.len(),
        "series": result.series.keys().collect::<Vec<_>>(),
    }))
}

fn to_calc_kline(k: &ohlcv_engine::Kline) -> CalcKline {
    CalcKline {
        open_time: k.open_time,
        open: k.open,
        high: k.high,
        low: k.low,
        close: k.close,
        volume: k.volume,
    }
}
