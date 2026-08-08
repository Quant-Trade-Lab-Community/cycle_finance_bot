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
    detected_patterns: Vec<algorithms::PatternDetection>,
}

struct AppState {
    client: BinanceClient,
}

#[tokio::main]
async fn main() {
    println!("==================================================");
    println!("👁️ FORMASYON TARAYICI (PATTERN) MOTORU BAŞLATILDI");
    println!("==================================================");
    
    let state = Arc::new(AppState {
        client: BinanceClient::new(),
    });

    let app = Router::new()
        .route("/api/pattern", get(get_patterns))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3004));
    println!("API Sunucusu http://{} üzerinde dinleniyor.", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_patterns(
    Query(params): Query<Params>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(100); // 100 is enough for pattern scanning usually
    
    match state.client.fetch_klines(&params.symbol, &params.interval, limit).await {
        Ok(klines) => {
            if klines.is_empty() { return Json(serde_json::json!({"error": "No data"})); }
            let current_price = klines.last().unwrap().close;
            
            // Tüm formasyonları tara
            let mut patterns = algorithms::scan_patterns(&klines);
            
            // Kullanıcı API'yi çağırdığında tüm listeyi görmek yerine en son olanlarla daha çok ilgilenir.
            // Fakat analiz için son 20 mumda (veya tüm limitte) oluşanları tutmak çok değerlidir.
            // Bu yüzden Hepsini dönüyoruz, ama index'e göre reverse (yeniden eskiye) sıralamak faydalı olabilir.
            patterns.sort_by(|a, b| b.index.cmp(&a.index));

            let response = APIResponse {
                status: "success".into(),
                symbol: params.symbol,
                interval: params.interval,
                current_price,
                detected_patterns: patterns,
            };
            Json(serde_json::to_value(response).unwrap())
        },
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}
