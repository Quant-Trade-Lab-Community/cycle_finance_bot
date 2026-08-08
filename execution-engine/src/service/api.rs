//! REST API katmanı (axum).
//!
//! Tüm yazma işlemleri actor'e komut olarak gider (tek-yazıcı); okumalar
//! paylaşılan snapshot'tan yapılır. Salt-okunur borsa sorguları (income,
//! funding, forceOrders, exchange-info...) `client` üzerinden direkt yapılır.

use crate::client::BinanceClient;
use crate::error::ExecError;
use crate::gateway::EngineHandle;
use crate::metrics::Metrics;
use crate::order::{OrderPositionSide, OrderRequest, OrderSide, OrderType, TimeInForce, WorkingType};
use crate::types::account::MarginType;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
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

fn now_epoch() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

fn require_auth(headers: &HeaderMap, secret: &str) -> Result<Claims, StatusCode> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    verify_token(token, secret).ok_or(StatusCode::UNAUTHORIZED)
}

// ── App state ───────────────────────────────────────────────────

pub struct AppState {
    pub engine: EngineHandle,
    pub auth: Arc<AuthState>,
    pub metrics: Arc<Metrics>,
    /// Salt-okunur borsa sorguları için (paper modda None).
    pub client: Option<Arc<BinanceClient>>,
}

pub fn router(app: Arc<AppState>) -> Router {
    let protected = Router::new()
        .route("/api/v1/orders", post(place_order))
        .route("/api/v1/orders/batch", post(place_batch))
        .route("/api/v1/orders", get(list_orders))
        .route("/api/v1/orders/cancel", post(cancel_order))
        .route("/api/v1/orders/open", delete(cancel_all_open))
        .route("/api/v1/orders/{cid}", delete(cancel_by_cid))
        .route("/api/v1/orders/{cid}", put(modify_order_route))
        .route("/api/v1/orders/query", get(query_order))
        .route("/api/v1/account", get(get_account))
        .route("/api/v1/positions", get(get_positions))
        .route("/api/v1/positions/{symbol}", get(get_position_symbol))
        .route("/api/v1/positions/close", post(close_positions))
        .route("/api/v1/balances", get(get_balances))
        .route("/api/v1/income", get(get_income))
        .route("/api/v1/funding", get(get_funding))
        .route("/api/v1/force-orders", get(get_force_orders))
        .route("/api/v1/commission-rate/{symbol}", get(get_commission_rate))
        .route("/api/v1/adl/{symbol}", get(get_adl))
        .route("/api/v1/trading-status", get(get_trading_status))
        .route("/api/v1/exchange-info/{symbol}", get(get_exchange_info))
        .route("/api/v1/symbols/{symbol}/leverage", put(set_leverage))
        .route("/api/v1/symbols/{symbol}/margin-type", put(set_margin_type))
        .route("/api/v1/symbols/{symbol}/margin", post(adjust_margin))
        .route("/api/v1/position-mode", put(set_position_mode))
        .route("/api/v1/position-mode", get(get_position_mode))
        .route("/api/v1/multi-assets", put(set_multi_assets))
        .route("/api/v1/multi-assets", get(get_multi_assets))
        .route("/api/v1/risk", get(get_risk))
        .route("/api/v1/risk/kill-switch", put(set_kill_switch))
        .route("/api/v1/mode", get(get_mode))
        .route("/api/v1/healthz", get(healthz))
        .route("/metrics", get(get_metrics))
        .layer(axum::middleware::from_fn_with_state(app.clone(), auth_middleware));

    let public = Router::new()
        .route("/api/v1/auth/login", post(auth_login))
        .route("/api/v1/auth/refresh", post(auth_refresh));

    public.merge(protected).with_state(app)
}

// ── Orta katman ─────────────────────────────────────────────────

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let claims = require_auth(req.headers(), &state.auth.secret)?;
    let mut req = req;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn api_err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (status, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}

fn to_err(e: &ExecError) -> StatusCode {
    match e {
        ExecError::Preflight(_) | ExecError::Risk(_) | ExecError::NotReady(_) => StatusCode::BAD_REQUEST,
        ExecError::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
        ExecError::Binance { http_status, .. } => {
            StatusCode::from_u16(*http_status).unwrap_or(StatusCode::BAD_GATEWAY)
        }
        _ => StatusCode::BAD_GATEWAY,
    }
}

// ── Auth handlers ───────────────────────────────────────────────

async fn auth_login(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> impl IntoResponse {
    if req.username != state.auth.admin_user {
        return api_err(StatusCode::UNAUTHORIZED, "invalid credentials");
    }
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;
    let parsed = match PasswordHash::new(&state.auth.admin_pass_hash) {
        Ok(p) => p,
        Err(_) => return api_err(StatusCode::UNAUTHORIZED, "invalid credentials"),
    };
    if Argon2::default().verify_password(req.password.as_bytes(), &parsed).is_err() {
        return api_err(StatusCode::UNAUTHORIZED, "invalid credentials");
    }
    let now = now_epoch();
    let access = Claims { sub: req.username.clone(), role: "ADMIN".into(), exp: now + 3_600 };
    let refresh = Claims { sub: req.username.clone(), role: "REFRESH".into(), exp: now + 86_400 };
    let resp = TokenResponse {
        access_token: make_token(&access, &state.auth.secret),
        refresh_token: make_token(&refresh, &state.auth.secret),
    };
    (StatusCode::OK, Json(resp)).into_response()
}

async fn auth_refresh(State(state): State<Arc<AppState>>, Json(body): Json<RefreshRequest>) -> impl IntoResponse {
    match verify_token(&body.refresh_token, &state.auth.secret) {
        Some(claims) if claims.role == "REFRESH" => {
            let access = Claims { sub: claims.sub, role: "ADMIN".into(), exp: now_epoch() + 3_600 };
            (StatusCode::OK, Json(serde_json::json!({ "access_token": make_token(&access, &state.auth.secret) }))).into_response()
        }
        _ => api_err(StatusCode::UNAUTHORIZED, "invalid refresh token"),
    }
}

// ── Emir ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub quantity: Decimal,
    /// MARKET emirlerde USDT bazlı büyüklük (quantity yerine quoteOrderQty).
    #[serde(default)]
    pub quote_order_qty: Option<Decimal>,
    #[serde(default)]
    pub price: Option<Decimal>,
    #[serde(default)]
    pub stop_price: Option<Decimal>,
    #[serde(default)]
    pub time_in_force: Option<String>,
    #[serde(default)]
    pub position_side: Option<String>,
    #[serde(default)]
    pub reduce_only: Option<bool>,
    #[serde(default)]
    pub close_position: Option<bool>,
    #[serde(default)]
    pub working_type: Option<String>,
    #[serde(default)]
    pub activation_price: Option<Decimal>,
    #[serde(default)]
    pub callback_rate: Option<Decimal>,
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, String> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(|e| e.to_string())
}

fn build_order(req: PlaceOrderRequest) -> Result<OrderRequest, String> {
    Ok(OrderRequest {
        symbol: req.symbol.to_uppercase(),
        side: parse_enum::<OrderSide>(&req.side)?,
        order_type: parse_enum::<OrderType>(&req.order_type)?,
        quantity: req.quantity,
        quote_order_qty: req.quote_order_qty,
        price: req.price,
        stop_price: req.stop_price,
        time_in_force: req
            .time_in_force
            .as_deref()
            .map(parse_enum::<TimeInForce>)
            .transpose()?,
        position_side: req
            .position_side
            .as_deref()
            .map(parse_enum::<OrderPositionSide>)
            .transpose()?
            .unwrap_or(OrderPositionSide::Both),
        client_order_id: req.client_order_id,
        reduce_only: req.reduce_only,
        close_position: req.close_position,
        working_type: req
            .working_type
            .as_deref()
            .map(parse_enum::<WorkingType>)
            .transpose()?,
        activation_price: req.activation_price,
        callback_rate: req.callback_rate,
        new_order_resp_type: None,
        price_protect: None,
        self_trade_prevention_mode: None,
        recv_window: None,
    })
}

async fn place_order(State(state): State<Arc<AppState>>, Json(req): Json<PlaceOrderRequest>) -> impl IntoResponse {
    let order = match build_order(req) {
        Ok(o) => o,
        Err(e) => return api_err(StatusCode::BAD_REQUEST, format!("geçersiz emir: {e}")),
    };
    match state.engine.submit_order(order).await {
        Ok(ack) => (StatusCode::OK, Json(ack)).into_response(),
        Err(e) => {
            state.metrics.record_order(false);
            api_err(StatusCode::BAD_REQUEST, e)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BatchOrderRequest {
    pub orders: Vec<PlaceOrderRequest>,
}

async fn place_batch(State(state): State<Arc<AppState>>, Json(req): Json<BatchOrderRequest>) -> impl IntoResponse {
    let mut orders = Vec::with_capacity(req.orders.len());
    for r in req.orders {
        match build_order(r) {
            Ok(o) => orders.push(o),
            Err(e) => return api_err(StatusCode::BAD_REQUEST, format!("geçersiz emir: {e}")),
        }
    }
    if orders.len() > 5 {
        return api_err(StatusCode::BAD_REQUEST, "batch en fazla 5 emir alır");
    }
    match state.engine.submit_batch(orders).await {
        Ok(acks) => (StatusCode::OK, Json(acks)).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize)]
pub struct OrderQueryParams {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub order_id: Option<i64>,
    #[serde(default)]
    pub client_order_id: Option<String>,
}

async fn list_orders(State(state): State<Arc<AppState>>, Query(q): Query<OrderQueryParams>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    let orders = match &q.symbol {
        Some(s) => snap
            .open_orders
            .iter()
            .filter(|o| o.symbol == *s)
            .cloned()
            .collect::<Vec<_>>(),
        None => snap.open_orders.clone(),
    };
    (StatusCode::OK, Json(orders)).into_response()
}

async fn query_order(State(state): State<Arc<AppState>>, Query(q): Query<OrderQueryParams>) -> impl IntoResponse {
    let symbol = q.symbol.clone().unwrap_or_default();
    if symbol.is_empty() {
        return api_err(StatusCode::BAD_REQUEST, "symbol zorunlu");
    }
    match state
        .engine
        .query_order(&symbol, q.order_id, q.client_order_id.as_deref())
        .await
    {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

async fn cancel_order(State(state): State<Arc<AppState>>, Query(q): Query<OrderQueryParams>) -> impl IntoResponse {
    let symbol = q.symbol.clone().unwrap_or_default();
    if symbol.is_empty() {
        return api_err(StatusCode::BAD_REQUEST, "symbol zorunlu");
    }
    match state
        .engine
        .cancel_order(&symbol, q.order_id, q.client_order_id.as_deref())
        .await
    {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

async fn cancel_by_cid(State(state): State<Arc<AppState>>, Path(cid): Path<String>, Query(q): Query<OrderQueryParams>) -> impl IntoResponse {
    let symbol = q.symbol.clone().unwrap_or_default();
    if symbol.is_empty() {
        return api_err(StatusCode::BAD_REQUEST, "symbol zorunlu");
    }
    match state.engine.cancel_order(&symbol, None, Some(&cid)).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

async fn cancel_all_open(State(state): State<Arc<AppState>>, Query(q): Query<OrderQueryParams>) -> impl IntoResponse {
    let symbol = q.symbol.clone().unwrap_or_default();
    if symbol.is_empty() {
        return api_err(StatusCode::BAD_REQUEST, "symbol zorunlu");
    }
    match state.engine.cancel_all(&symbol).await {
        Ok(n) => (StatusCode::OK, Json(serde_json::json!({ "cancelled": n }))).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize)]
pub struct ModifyOrderRequest {
    pub symbol: String,
    #[serde(default)]
    pub order_id: Option<i64>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub quantity: Option<Decimal>,
    #[serde(default)]
    pub price: Option<Decimal>,
    #[serde(default)]
    pub stop_price: Option<Decimal>,
}

async fn modify_order_route(State(state): State<Arc<AppState>>, Path(cid): Path<String>, Json(req): Json<ModifyOrderRequest>) -> impl IntoResponse {
    match state
        .engine
        .modify_order(
            &req.symbol,
            req.order_id,
            req.client_order_id.as_deref().or(Some(&cid)),
            req.quantity,
            req.price,
            req.stop_price,
        )
        .await
    {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

// ── Hesap / pozisyon (snapshot) ─────────────────────────────────

async fn get_account(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    (StatusCode::OK, Json(snap)).into_response()
}

async fn get_positions(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    (StatusCode::OK, Json(snap.positions)).into_response()
}

async fn get_position_symbol(State(state): State<Arc<AppState>>, Path(symbol): Path<String>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    let sym = symbol.to_uppercase();
    let pos: Vec<_> = snap
        .positions
        .iter()
        .filter(|p| p.symbol == sym)
        .cloned()
        .collect();
    (StatusCode::OK, Json(pos)).into_response()
}

async fn get_balances(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    (StatusCode::OK, Json(snap.account.assets)).into_response()
}

// ── Pozisyon kapatma ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ClosePositionsRequest {
    /// Boşsa TÜM açık pozisyonlar kapatılır.
    #[serde(default)]
    pub symbol: Option<String>,
    /// Hedge modda taraf: LONG | SHORT (opsiyonel, boşsa her iki taraf).
    #[serde(default)]
    pub position_side: Option<String>,
}

async fn close_positions(State(state): State<Arc<AppState>>, Json(req): Json<ClosePositionsRequest>) -> impl IntoResponse {
    let res = match req.symbol.as_deref() {
        Some(sym) => {
            state.engine.close_symbol(sym, req.position_side.as_deref()).await
        }
        None => {
            if req.position_side.is_some() {
                return api_err(
                    StatusCode::BAD_REQUEST,
                    "position_side yalnızca symbol ile birlikte kullanılır",
                );
            }
            state.engine.close_all().await
        }
    };
    match res {
        Ok(closed) => (StatusCode::OK, Json(serde_json::json!({ "closed": closed }))).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

// ── Borsa salt-okunur sorgular ──────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct IncomeParams {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    #[serde(default)]
    pub start_time: Option<u64>,
    #[serde(default)]
    pub end_time: Option<u64>,
    #[serde(default)]
    pub limit: Option<u32>,
}

async fn get_income(State(state): State<Arc<AppState>>, Query(q): Query<IncomeParams>) -> impl IntoResponse {
    let Some(client) = &state.client else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "paper modda income kapalı");
    };
    match client
        .income(
            q.symbol.as_deref(),
            q.type_.as_deref(),
            q.start_time,
            q.end_time,
            q.limit,
        )
        .await
    {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => api_err(to_err(&e), e.to_string()),
    }
}

async fn get_funding(State(state): State<Arc<AppState>>, Query(q): Query<OrderQueryParams>) -> impl IntoResponse {
    let Some(client) = &state.client else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "paper modda funding kapalı");
    };
    let symbol = q.symbol.clone().unwrap_or_default();
    if symbol.is_empty() {
        return api_err(StatusCode::BAD_REQUEST, "symbol zorunlu");
    }
    match client.funding_rate(&symbol, Some(10)).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => api_err(to_err(&e), e.to_string()),
    }
}

async fn get_force_orders(State(state): State<Arc<AppState>>, Query(q): Query<OrderQueryParams>) -> impl IntoResponse {
    let Some(client) = &state.client else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "paper modda forceOrders kapalı");
    };
    match client.force_orders(q.symbol.as_deref()).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => api_err(to_err(&e), e.to_string()),
    }
}

async fn get_commission_rate(State(state): State<Arc<AppState>>, Path(symbol): Path<String>) -> impl IntoResponse {
    let Some(client) = &state.client else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "paper modda commissionRate kapalı");
    };
    match client.commission_rate(&symbol).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => api_err(to_err(&e), e.to_string()),
    }
}

async fn get_adl(State(state): State<Arc<AppState>>, Path(symbol): Path<String>) -> impl IntoResponse {
    let Some(client) = &state.client else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "paper modda ADL kapalı");
    };
    match client.position_adl_quantile(Some(&symbol)).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => api_err(to_err(&e), e.to_string()),
    }
}

async fn get_trading_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Some(client) = &state.client else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "paper modda apiTradingStatus kapalı");
    };
    match client.api_trading_status().await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => api_err(to_err(&e), e.to_string()),
    }
}

async fn get_exchange_info(State(state): State<Arc<AppState>>, Path(symbol): Path<String>) -> impl IntoResponse {
    let Some(client) = &state.client else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "paper modda exchangeInfo kapalı");
    };
    match client.exchange_info().await {
        Ok(info) => {
            let sym = symbol.to_uppercase();
            match info.symbol(&sym) {
                Some(s) => (StatusCode::OK, Json(s)).into_response(),
                None => api_err(StatusCode::NOT_FOUND, format!("{sym} bulunamadı")),
            }
        }
        Err(e) => api_err(to_err(&e), e.to_string()),
    }
}

// ── Kontrol ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LeverageRequest {
    pub leverage: u32,
}

async fn set_leverage(State(state): State<Arc<AppState>>, Path(symbol): Path<String>, Json(req): Json<LeverageRequest>) -> impl IntoResponse {
    match state.engine.set_leverage(&symbol, req.leverage).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "symbol": symbol, "leverage": req.leverage }))).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize)]
pub struct MarginTypeRequest {
    pub margin_type: String,
}

async fn set_margin_type(State(state): State<Arc<AppState>>, Path(symbol): Path<String>, Json(req): Json<MarginTypeRequest>) -> impl IntoResponse {
    let mt = match req.margin_type.to_uppercase().as_str() {
        "ISOLATED" | "ISOLATE" => MarginType::Isolated,
        "CROSSED" | "CROSS" => MarginType::Crossed,
        _ => return api_err(StatusCode::BAD_REQUEST, "margin_type ISOLATED veya CROSSED olmalı"),
    };
    match state.engine.set_margin_type(&symbol, mt).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "symbol": symbol, "margin_type": mt.binance_str() }))).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize)]
pub struct AdjustMarginRequest {
    pub amount: Decimal,
    /// 1 = ekle, 2 = çek.
    pub direction: u8,
}

async fn adjust_margin(State(state): State<Arc<AppState>>, Path(symbol): Path<String>, Json(req): Json<AdjustMarginRequest>) -> impl IntoResponse {
    match state.engine.adjust_margin(&symbol, req.amount, req.direction).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "symbol": symbol, "amount": req.amount, "direction": req.direction }))).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

#[derive(Debug, Deserialize)]
pub struct PositionModeRequest {
    pub dual: bool,
}

async fn set_position_mode(State(state): State<Arc<AppState>>, Json(req): Json<PositionModeRequest>) -> impl IntoResponse {
    match state.engine.set_position_mode(req.dual).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "dual_side_position": req.dual }))).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

async fn get_position_mode(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    (StatusCode::OK, Json(serde_json::json!({ "dual_side_position": snap.position_mode }))).into_response()
}

#[derive(Debug, Deserialize)]
pub struct MultiAssetsRequest {
    pub enabled: bool,
}

async fn set_multi_assets(State(state): State<Arc<AppState>>, Json(req): Json<MultiAssetsRequest>) -> impl IntoResponse {
    match state.engine.set_multi_assets(req.enabled).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "multi_assets_margin": req.enabled }))).into_response(),
        Err(e) => api_err(StatusCode::BAD_REQUEST, e),
    }
}

async fn get_multi_assets(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Some(client) = &state.client else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "paper modda multiAssets kapalı");
    };
    match client.get_multi_assets().await {
        Ok(v) => (StatusCode::OK, Json(serde_json::json!({ "multi_assets_margin": v }))).into_response(),
        Err(e) => api_err(to_err(&e), e.to_string()),
    }
}

// ── Risk / metrikler ────────────────────────────────────────────

async fn get_risk(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    let ks = state.engine.kill_switch.is_open();
    let body = serde_json::json!({
        "kill_switch": ks,
        "ready": snap.ready,
        "mode": state.engine.mode().as_str(),
        "dry_run": state.engine.dry_run(),
        "max_notional_usdt": state.engine.config.max_notional_usdt.to_string(),
        "max_orders_per_min": state.engine.config.max_orders_per_min,
        "open_positions": snap.open_position_count(),
        "open_orders": snap.open_orders.len(),
    });
    (StatusCode::OK, Json(body)).into_response()
}

#[derive(Debug, Deserialize)]
pub struct KillSwitchRequest {
    pub enabled: bool,
}

async fn set_kill_switch(State(state): State<Arc<AppState>>, Json(req): Json<KillSwitchRequest>) -> impl IntoResponse {
    let res = if req.enabled {
        state.engine.kill_switch.engage()
    } else {
        state.engine.kill_switch.release()
    };
    match res {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "kill_switch": req.enabled }))).into_response(),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, format!("kill switch hatası: {e}")),
    }
}

async fn get_mode(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "mode": state.engine.mode().as_str(),
        "dry_run": state.engine.dry_run(),
    }))).into_response()
}

async fn healthz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snap = state.engine.snapshot();
    let healthy = snap.ready;
    let status = if healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, Json(serde_json::json!({
        "status": if healthy { "ok" } else { "not_ready" },
        "ready": snap.ready,
        "mode": state.engine.mode().as_str(),
    }))).into_response()
}

async fn get_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let body = state.metrics.render_prometheus();
    let mut resp = axum::http::Response::new(axum::body::Body::from(body));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4"),
    );
    resp
}
