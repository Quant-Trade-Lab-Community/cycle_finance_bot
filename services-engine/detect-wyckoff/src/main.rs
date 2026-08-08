// ============================================================================
// detect-wyckoff — REST API Servisi (:3005)
// /api/wyckoff?symbol=BTCUSDT&interval=1h&limit=300
// ============================================================================

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{extract::Query, routing::get, Json, Router};
use detect_wyckoff::analyst::{self, AnalysisConfig};
use ohlcv_engine::client::BinanceClient;
use serde::Deserialize;

#[derive(Deserialize)]
struct Params {
    symbol: String,
    interval: String,
    limit: Option<usize>,
}

struct AppState {
    client: BinanceClient,
}

#[tokio::main]
async fn main() {
    println!("══════════════════════════════════════════════════════");
    println!("  🏛️  WYCKOFF ANALİZ MOTORU — The Iron Crucible v3.0");
    println!("      WyckoffAnalyst v4.1.4 | Faz + POC + Bayesian");
    println!("══════════════════════════════════════════════════════");
    println!();

    let state = Arc::new(AppState {
        client: BinanceClient::new(),
    });

    let app = Router::new()
        .route("/api/wyckoff", get(get_wyckoff))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3005));
    println!("  API: http://{}/api/wyckoff?symbol=BTCUSDT&interval=1h", addr);
    println!("══════════════════════════════════════════════════════");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_wyckoff(
    Query(params): Query<Params>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(500);

    match state.client.fetch_klines(&params.symbol, &params.interval, limit).await {
        Ok(klines) => {
            if klines.is_empty() {
                return Json(serde_json::json!({"status": "error", "message": "No data received"}));
            }
            let current_price = klines.last().unwrap().close;
            let cfg = AnalysisConfig::default();
            match analyst::analyze(&klines, &cfg) {
                Ok(insight) => Json(serde_json::json!({
                    "status": "success",
                    "symbol": params.symbol,
                    "interval": params.interval,
                    "current_price": current_price,
                    "insight": insight,
                })),
                Err(e) => Json(serde_json::json!({
                    "status": "error",
                    "message": e,
                })),
            }
        }
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}