//! REST API katmanı (axum).
//!
//! Tüm yazma işlemleri actor'e komut olarak gönderilir (idempotent), okuma
//! işlemleri paylaşılan snapshot'tan yapılır. JWT ile korunur.

use crate::idempotency::{CachedResponse, IdempotencyCache};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use execution_engine::order::{OrderRequest, OrderSide, OrderType};
use execution_engine::paper::actor::{ActorCommand, OrderRejectReason};
use execution_engine::paper::snapshot::PaperSnapshot;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use parking_lot::RwLock;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

// ── Auth ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub struct AuthState {
    pub secret: String,
    pub admin_user: String,
    pub admin_pass_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

fn jwt_secret() -> String {
    std::env::var("PAPER_JWT_SECRET").unwrap_or_else(|_| "paper-dev-secret-change-me".to_string())
}

fn make_token(claims: &Claims, secret: &str) -> String {
    jsonwebtoken::encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("jwt encode")
}

fn verify_token(token: &str, secret: &str) -> Option<Claims> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|d| d.claims)
}

async fn auth_login(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> impl IntoResponse {
    if req.username != state.auth.admin_user {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid credentials"}))).into_response();
    }
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;
    let parsed = match PasswordHash::new(&state.auth.admin_pass_hash) {
        Ok(p) => p,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid credentials"}))).into_response(),
    };
    if Argon2::default().verify_password(req.password.as_bytes(), &parsed).is_err() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid credentials"}))).into_response();
    }

    let now = now_epoch();
    let access = Claims { sub: req.username.clone(), role: "ADMIN".into(), exp: now + 3600 };
    let refresh = Claims { sub: req.username.clone(), role: "REFRESH".into(), exp: now + 86_400 };
    let resp = TokenResponse {
        access_token: make_token(&access, &jwt_secret()),
        refresh_token: make_token(&refresh, &jwt_secret()),
    };
    (StatusCode::OK, Json(resp)).into_response()
}

async fn auth_refresh(State(_state): State<Arc<AppState>>, Json(body): Json<RefreshRequest>) -> impl IntoResponse {
    let secret = jwt_secret();
    match verify_token(&body.refresh_token, &secret) {
        Some(claims) if claims.role == "REFRESH" => {
            let access = Claims { sub: claims.sub, role: "ADMIN".into(), exp: now_epoch() + 3600 };
            (StatusCode::OK, Json(serde_json::json!({"access_token": make_token(&access, &secret)}))).into_response()
        }
        _ => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid refresh token"}))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

fn now_epoch() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

// ── Engine handle ───────────────────────────────────────────────

#[derive(Clone)]
pub struct EngineHandle {
    pub cmd_tx: mpsc::UnboundedSender<ActorCommand>,
    pub snapshot: Arc<RwLock<PaperSnapshot>>,
    pub idempotency: Arc<dyn IdempotencyCache>,
}

impl EngineHandle {
    pub fn snapshot(&self) -> PaperSnapshot {
        self.snapshot.read().clone()
    }

    pub async fn submit_order(&self, order: OrderRequest) -> Result<execution_engine::paper::actor::OrderAck, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.cmd_tx
            .send(ActorCommand::SubmitOrder { order, response_tx: resp_tx })
            .map_err(|e| format!("actor channel closed: {e}"))?;
        match resp_rx.await {
            Ok(Ok(ack)) => Ok(ack),
            Ok(Err(OrderRejectReason::InsufficientFunds)) => Err("insufficient funds".into()),
            Ok(Err(OrderRejectReason::MarketUnavailable)) => Err("market unavailable".into()),
            Ok(Err(OrderRejectReason::InsufficientDepth)) => Err("insufficient depth".into()),
            Ok(Err(OrderRejectReason::RiskRejected(m))) => Err(m),
            Err(_) => Err("actor response dropped".into()),
        }
    }
}

// ── App state & router ──────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub engine: EngineHandle,
    pub auth: Arc<AuthState>,
    pub metrics: Arc<crate::metrics::Metrics>,
}

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
}

async fn auth_middleware(headers: HeaderMap, state: &AppState) -> Result<Claims, StatusCode> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    verify_token(token, &state.auth.secret).ok_or(StatusCode::UNAUTHORIZED)
}

async fn place_order(State(state): State<Arc<AppState>>, headers: HeaderMap, Json(req): Json<PlaceOrderRequest>) -> impl IntoResponse {
    if auth_middleware(headers, &state).await.is_err() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "unauthorized"}))).into_response();
    }

    // Idempotency: aynı client_order_id → eski sonuç
    if let Some(cached) = state.engine.idempotency.get(&req.client_order_id) {
        return (StatusCode::from_u16(cached.http_status).unwrap_or(StatusCode::OK), Json(cached.body)).into_response();
    }

    let side = match req.side.to_uppercase().as_str() {
        "BUY" => OrderSide::Buy,
        "SELL" => OrderSide::Sell,
        _ => {
            let body = serde_json::json!({"error": "side must be BUY or SELL"});
            let resp = CachedResponse { http_status: 400, body: body.clone() };
            state.engine.idempotency.set(&req.client_order_id, resp);
            return (StatusCode::BAD_REQUEST, Json(body)).into_response();
        }
    };
    let order_type = match req.order_type.to_uppercase().as_str() {
        "MARKET" => OrderType::Market,
        "LIMIT" => OrderType::Limit,
        _ => {
            let body = serde_json::json!({"error": "order_type must be MARKET or LIMIT"});
            let resp = CachedResponse { http_status: 400, body: body.clone() };
            state.engine.idempotency.set(&req.client_order_id, resp);
            return (StatusCode::BAD_REQUEST, Json(body)).into_response();
        }
    };

    let order = OrderRequest {
        symbol: req.symbol,
        side,
        order_type,
        quantity: req.quantity,
        price: req.price,
        time_in_force: None,
    };

    match state.engine.submit_order(order).await {
        Ok(ack) => {
            state.metrics.record_order(true);
            state.metrics.record_fill();
            let body = serde_json::json!({
                "order_id": ack.order_id,
                "avg_price": ack.avg_price.to_string(),
                "executed_qty": ack.executed_qty.to_string(),
            });
            let resp = CachedResponse { http_status: 200, body: body.clone() };
            state.engine.idempotency.set(&req.client_order_id, resp);
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(msg) => {
            state.metrics.record_order(false);
            let body = serde_json::json!({"error": msg});
            let resp = CachedResponse { http_status: 400, body: body.clone() };
            state.engine.idempotency.set(&req.client_order_id, resp);
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        }
    }
}

async fn get_balance(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    if auth_middleware(headers, &state).await.is_err() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "unauthorized"}))).into_response();
    }
    let snap = state.engine.snapshot();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "cash_balance": snap.cash_balance.to_string(),
            "equity": snap.equity.to_string(),
            "realized_pnl": snap.realized_pnl.to_string(),
            "total_commission": snap.total_commission.to_string(),
            "risk_status": snap.risk_status,
        })),
    )
        .into_response()
}

async fn get_positions(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    if auth_middleware(headers, &state).await.is_err() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "unauthorized"}))).into_response();
    }
    let snap = state.engine.snapshot();
    (StatusCode::OK, Json(serde_json::json!({ "positions": snap.positions }))).into_response()
}

async fn get_orders(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    if auth_middleware(headers, &state).await.is_err() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "unauthorized"}))).into_response();
    }
    let snap = state.engine.snapshot();
    (StatusCode::OK, Json(serde_json::json!({ "open_orders": snap.open_orders }))).into_response()
}

async fn get_trade_history(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    if auth_middleware(headers, &state).await.is_err() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "unauthorized"}))).into_response();
    }
    let snap = state.engine.snapshot();
    (StatusCode::OK, Json(serde_json::json!({ "trades": snap.recent_trades }))).into_response()
}

async fn get_liquidation_price(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(symbol): Path<String>,
) -> impl IntoResponse {
    if auth_middleware(headers, &state).await.is_err() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "unauthorized"}))).into_response();
    }
    let snap = state.engine.snapshot();
    let liq = snap.positions.iter().find(|p| p.symbol == symbol).and_then(|p| p.liquidation_price);
    match liq {
        Some(price) => (StatusCode::OK, Json(serde_json::json!({"symbol": symbol, "liquidation_price": price.to_string()}))).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "no position"}))).into_response(),
    }
}

async fn get_health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "engine_inbox_alive": !state.engine.cmd_tx.is_closed(),
            "last_price": snap.last_price.to_string(),
        })),
    )
}

async fn get_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        state.metrics.render(snap.cash_balance.to_string()),
    )
}

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/auth/login", post(auth_login))
        .route("/api/v1/auth/refresh", post(auth_refresh))
        .route("/api/v1/system/health", get(get_health))
        .route("/api/v1/order", post(place_order))
        .route("/api/v1/orders", get(get_orders))
        .route("/api/v1/account/balance", get(get_balance))
        .route("/api/v1/account/trade-history", get(get_trade_history))
        .route("/api/v1/account/positions", get(get_positions))
        .route("/api/v1/risk/liquidation-price/{symbol}", get(get_liquidation_price))
        .route("/metrics", get(get_metrics))
        .route("/", get(|| async { "🛡️ Paper Service API v2.0 — running" }))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}

pub async fn serve(addr: &str, state: Arc<AppState>) {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind api");
    tracing::info!("REST API dinleniyor: http://{addr}");
    axum::serve(listener, app).await.expect("serve api");
}
