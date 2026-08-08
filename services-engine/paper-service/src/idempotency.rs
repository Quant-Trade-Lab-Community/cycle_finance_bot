//! Idempotency ve cache katmanı.
//!
//! `client_order_id -> OrderResponse` eşlemesi, çift emir gönderimini önler.
//! Tam set (`--features full`) ile Redis kullanılır; aksi halde in-memory.

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CachedResponse {
    pub http_status: u16,
    pub body: serde_json::Value,
}

pub trait IdempotencyCache: Send + Sync {
    /// Eğer bu `client_oid` daha önce işlendiyse önbellekteki yanıtı döner.
    fn get(&self, client_oid: &str) -> Option<CachedResponse>;
    /// İşlenen isteği TTL ile saklar.
    fn set(&self, client_oid: &str, response: CachedResponse);
}

pub struct InMemoryIdempotencyCache {
    inner: Mutex<HashMap<String, CachedResponse>>,
}

impl InMemoryIdempotencyCache {
    pub fn new() -> Self {
        Self { inner: Mutex::new(HashMap::new()) }
    }
}

impl IdempotencyCache for InMemoryIdempotencyCache {
    fn get(&self, client_oid: &str) -> Option<CachedResponse> {
        self.inner.lock().unwrap().get(client_oid).cloned()
    }

    fn set(&self, client_oid: &str, response: CachedResponse) {
        self.inner.lock().unwrap().insert(client_oid.to_string(), response);
    }
}
