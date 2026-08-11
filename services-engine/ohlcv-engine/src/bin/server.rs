use axum::{
    extract::Query,
    routing::get,
    Router, Json,
};
use ohlcv_engine::client::BinanceClient;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Deserialize)]
struct KlineParams {
    symbol: String,
    interval: String,
    limit: Option<usize>,
}

struct AppState {
    client: BinanceClient,
}

#[tokio::main]
async fn main() {
    println!("OHLCV API Sunucusu Başlatılıyor...");
    
    let state = Arc::new(AppState {
        client: BinanceClient::new(),
    });

    let app = Router::new()
        .route("/api/klines", get(get_klines))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("API Sunucusu http://{} üzerinde dinleniyor.", addr);
    println!("Örnek kullanım: http://127.0.0.1:3000/api/klines?symbol=VELVETUSDT&interval=15m&limit=100");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_klines(
    Query(params): Query<KlineParams>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(100);
    
    match state.client.fetch_klines(&params.symbol, &params.interval, limit).await {
        Ok(klines) => Json(serde_json::json!({
            "status": "success",
            "symbol": params.symbol,
            "interval": params.interval,
            "count": klines.len(),
            "data": klines
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}
