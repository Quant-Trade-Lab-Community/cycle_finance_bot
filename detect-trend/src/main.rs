use axum::{
    extract::Query,
    routing::get,
    Router, Json,
};
use ohlcv_engine::client::BinanceClient;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

pub mod algorithms;

#[derive(Deserialize)]
struct TrendParams {
    symbol: String,
    interval: String,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct APIResponse {
    status: String,
    symbol: String,
    interval: String,
    current_price: f64,
    results: Vec<algorithms::TrendResult>,
}

struct AppState {
    client: BinanceClient,
}

#[tokio::main]
async fn main() {
    println!("==================================================");
    println!("📈 QUANT TREND & REJİM ANALİZ MOTORU (API)");
    println!("==================================================");
    
    let state = Arc::new(AppState {
        client: BinanceClient::new(),
    });

    let app = Router::new()
        .route("/api/trend", get(get_trends))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("API Sunucusu http://{} üzerinde dinleniyor.", addr);
    println!("Örnek kullanım: http://127.0.0.1:3001/api/trend?symbol=HEIUSDT&interval=1h&limit=500\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_trends(
    Query(params): Query<TrendParams>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(500);
    
    match state.client.fetch_klines(&params.symbol, &params.interval, limit).await {
        Ok(klines) => {
            if klines.is_empty() {
                return Json(serde_json::json!({"status": "error", "message": "No data received"}));
            }
            
            let current_price = klines.last().unwrap().close;
            let mut results = Vec::new();

            results.push(algorithms::sma_ema_crossover(&klines));
            results.push(algorithms::linear_regression(&klines));
            results.push(algorithms::adx(&klines));
            results.push(algorithms::supertrend(&klines));
            results.push(algorithms::dow_theory(&klines));
            results.push(algorithms::hurst_exponent(&klines));
            results.push(algorithms::hmm_simplified(&klines));
            results.push(algorithms::fourier_trend(&klines));
            results.push(algorithms::parabolic_sar(&klines));
            results.push(algorithms::ichimoku(&klines));

            let response = APIResponse {
                status: "success".into(),
                symbol: params.symbol,
                interval: params.interval,
                current_price,
                results,
            };

            Json(serde_json::to_value(response).unwrap())
        },
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}
