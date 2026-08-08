use axum::{
    extract::Query,
    routing::get,
    Router, Json,
};
use ohlcv_engine::client::BinanceClient;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

pub mod algorithms;

#[derive(Deserialize)]
struct Params {
    symbol: String,
    interval: String,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct APIResponse {
    status: String,
    symbol: String,
    interval: String,
    current_price: Decimal,
    liquidity: algorithms::LiquidityResult,
}

struct AppState {
    client: BinanceClient,
}

#[tokio::main]
async fn main() {
    println!("==================================================");
    println!("🩸 LİKİDİTE AVCISI (SMC) MOTORU BAŞLATILDI");
    println!("==================================================");
    
    let state = Arc::new(AppState {
        client: BinanceClient::new(),
    });

    let app = Router::new()
        .route("/api/liquidity", get(get_liquidity))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3003));
    println!("API Sunucusu http://{} üzerinde dinleniyor.", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_liquidity(
    Query(params): Query<Params>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(500);
    
    match state.client.fetch_klines(&params.symbol, &params.interval, limit).await {
        Ok(klines) => {
            if klines.is_empty() { return Json(serde_json::json!({"error": "No data"})); }
            let current_price = klines.last().unwrap().close;
            let lq = algorithms::analyze_liquidity(&klines);

            let response = APIResponse {
                status: "success".into(),
                symbol: params.symbol,
                interval: params.interval,
                current_price,
                liquidity: lq,
            };
            Json(serde_json::to_value(response).unwrap())
        },
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}
