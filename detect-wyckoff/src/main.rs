// ============================================================================
// WYCKOFF V4 — AXUM HTTP API
// ============================================================================
// GET /api/wyckoff?symbol=BTCUSDT&interval=15m&limit=200
//
// Binance Futures REST API'den veri çeker, WyckoffAnalyst'i besler ve
// son bar'a ait tam WyckoffInsight raporu döner.
//
// Port: 3006 (detect-ms=3002, detect-sr=3001, detect-trend=3003,
//              detect-pattern=3004, detect-liquidity=3005 sıralamasında)
// ============================================================================

use axum::{extract::Query, routing::get, Json, Router};
use ohlcv_engine::client::BinanceClient;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

mod analyst;
mod events;
mod narrative;
mod probability;
mod structure;
mod types;

use analyst::WyckoffAnalyst;

#[derive(Deserialize)]
struct Params {
    symbol: String,
    interval: String,
    /// Kaç bar çekileceği (EWMA penceresi dolması için önerilen: ≥200)
    limit: Option<usize>,
}

struct AppState {
    client: BinanceClient,
}

#[tokio::main]
async fn main() {
    println!("══════════════════════════════════════════════════════");
    println!("  🌊  WYCKOFF V4 — FAZ ANALİZ MOTORU");
    println!("      EWMA Faz Geçişleri | POC | Spring/SOS/UT");
    println!("      Kalibrasyon: v4.1.0 | decay_factor=0.85");
    println!("══════════════════════════════════════════════════════");
    println!();
    println!("  Katman 1: EWMA Faz Ağırlıkları (Accumulation/Markup/Distribution/Markdown)");
    println!("  Katman 2: Anlık Faz Skoru (Kural Matrisi)");
    println!("  Katman 3: Yapısal Konum (POC + Range + Spread)");
    println!("  Katman 4: Wyckoff Olayları (Spring / SOS / UT)");
    println!("  Katman 5: Olasılık Tahmini (Breakout/Breakdown/Range)");
    println!("  Katman 6: Türkçe Naratif + Yön Tavsiyesi (Bias)");
    println!();

    let state = Arc::new(AppState {
        client: BinanceClient::new(),
    });

    let app = Router::new()
        .route("/api/wyckoff", get(get_wyckoff))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3006));
    println!(
        "  API: http://{}/api/wyckoff?symbol=BTCUSDT&interval=15m&limit=200",
        addr
    );
    println!("══════════════════════════════════════════════════════");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// GET /api/wyckoff
///
/// Parametreler:
/// - `symbol`: Binance Futures sembolü (örn. BTCUSDT)
/// - `interval`: Mum aralığı (örn. 15m, 1h, 4h)
/// - `limit`: Bar sayısı (varsayılan 200; EWMA için en az 150 önerilir)
async fn get_wyckoff(
    Query(params): Query<Params>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(200).clamp(50, 1500);

    // Binance Futures REST'ten veri çek
    let klines = match state
        .client
        .fetch_klines(&params.symbol, &params.interval, limit)
        .await
    {
        Ok(k) => k,
        Err(e) => {
            return Json(serde_json::json!({
                "error": format!("Binance veri çekme hatası: {}", e),
                "symbol": params.symbol,
                "interval": params.interval,
            }))
        }
    };

    if klines.is_empty() {
        return Json(serde_json::json!({
            "error": "Veri bulunamadı",
            "symbol": params.symbol,
            "interval": params.interval,
        }));
    }

    // WyckoffAnalyst'i EWMA penceresi = limit olarak başlat
    let mut analyst = WyckoffAnalyst::new(limit);

    // Tüm bar'ları besle (pencere dolsun ve EWMA stabilize olsun)
    // Son bar hariç tümünü sessizce işle
    let (last, rest) = klines.split_last().unwrap();
    for kline in rest {
        let _ = analyst.feed(kline);
    }

    // Son bar için tam rapor al
    let insight = analyst.feed(last);

    // JSON olarak döndür
    match serde_json::to_value(&insight) {
        Ok(v) => Json(v),
        Err(e) => Json(serde_json::json!({
            "error": format!("Serializasyon hatası: {}", e),
        })),
    }
}
