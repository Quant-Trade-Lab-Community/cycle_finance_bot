//! Execution motoru hata modeli.
//!
//! Binance hataları hem HTTP durum koduna hem de `code` alanına göre ayrıştırılır
//! (ör. `-1021` timestamp drift, `-2015` yetki, `-2019` marj yetersiz).
//! Rate-limit ve ağ hataları yeniden denenebilir (`retryable`).

use std::fmt;

#[derive(Debug)]
pub enum ExecError {
    /// reqwest taşıma hatası (bağlantı, TLS, pool).
    Http(reqwest::Error),
    /// Binance REST API hatası (HTTP 4xx/5xx + JSON `{code, msg}`).
    Binance { http_status: u16, code: i64, msg: String },
    /// 429 / 418 — ağırlık limiti. `retry_after_ms` sunucunun istediği bekleme.
    RateLimit { retry_after_ms: u64 },
    /// İstek zaman aşımı.
    Timeout,
    /// JSON ayrıştırma hatası.
    Json(serde_json::Error),
    /// WS user-data stream hatası.
    WebSocket(String),
    /// Beklenmedik yanıt biçimi.
    InvalidResponse(String),
    /// Pre-trade doğrulama reddi (filtre, precizyon, notional, mod).
    Preflight(String),
    /// Risk katmanı reddi (kill switch, limit, blocklist).
    Risk(String),
    /// Hesap state'i henüz borsa ile eşitlenmedi — yazma kabul edilmez.
    NotReady(String),
    /// Actor kanalı kapalı.
    ChannelClosed,
    /// Config / çevre değişkeni eksik.
    Config(String),
    /// Diğer.
    Other(String),
}

pub type Result<T> = std::result::Result<T, ExecError>;

impl ExecError {
    /// Ağ/5xx/429/418 ve -1021 tarzı hatalar yeniden denenebilir.
    pub fn is_retryable(&self) -> bool {
        match self {
            ExecError::Http(e) if e.is_timeout() || e.is_connect() => true,
            ExecError::RateLimit { .. } => true,
            ExecError::Timeout => true,
            ExecError::Binance { http_status, code, .. } => {
                *http_status == 429
                    || *http_status == 418
                    || *http_status >= 500
                    || *code == -1001
                    || *code == -1021
                    || *code == -2015
            }
            _ => false,
        }
    }
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecError::Http(e) => write!(f, "http error: {e}"),
            ExecError::Binance { http_status, code, msg } => {
                write!(f, "binance error (http {http_status}, code {code}): {msg}")
            }
            ExecError::RateLimit { retry_after_ms } => {
                write!(f, "rate limited; retry after {retry_after_ms}ms")
            }
            ExecError::Timeout => write!(f, "request timeout"),
            ExecError::Json(e) => write!(f, "json error: {e}"),
            ExecError::WebSocket(m) => write!(f, "websocket error: {m}"),
            ExecError::InvalidResponse(m) => write!(f, "invalid response: {m}"),
            ExecError::Preflight(m) => write!(f, "preflight rejected: {m}"),
            ExecError::Risk(m) => write!(f, "risk rejected: {m}"),
            ExecError::NotReady(m) => write!(f, "engine not ready: {m}"),
            ExecError::ChannelClosed => write!(f, "internal channel closed"),
            ExecError::Config(m) => write!(f, "config error: {m}"),
            ExecError::Other(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ExecError {}

impl From<reqwest::Error> for ExecError {
    fn from(e: reqwest::Error) -> Self {
        ExecError::Http(e)
    }
}

impl From<serde_json::Error> for ExecError {
    fn from(e: serde_json::Error) -> Self {
        ExecError::Json(e)
    }
}

impl From<tokio::time::error::Elapsed> for ExecError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        ExecError::Timeout
    }
}

impl From<flume::RecvError> for ExecError {
    fn from(_: flume::RecvError) -> Self {
        ExecError::ChannelClosed
    }
}
