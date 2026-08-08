//! Havada (in-flight) emir kaydı.
//!
//! Emir borsaya gönderildiği andan user-data stream ile kesin durumu
//! alınana kadar izlenir. Zaman aşımında `GET /fapi/v1/order` ile uzlaştırılır.

use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct InFlightOrder {
    pub client_order_id: String,
    /// Borsa emir numarası (ACK sonrası bilinir).
    pub order_id: Option<i64>,
    pub symbol: String,
    pub sent_at: Instant,
    pub timeout_ms: u64,
}

impl InFlightOrder {
    pub fn is_expired(&self, now: Instant) -> bool {
        now.duration_since(self.sent_at).as_millis() as u64 > self.timeout_ms
    }
}

pub struct InFlightRegistry {
    inner: HashMap<String, InFlightOrder>,
    default_timeout_ms: u64,
    max_size: usize,
}

impl InFlightRegistry {
    pub fn new(default_timeout_ms: u64, max_size: usize) -> Self {
        Self {
            inner: HashMap::new(),
            default_timeout_ms,
            max_size,
        }
    }

    pub fn insert(&mut self, client_order_id: String, symbol: String, order_id: Option<i64>, timeout_ms: Option<u64>) -> bool {
        if self.inner.len() >= self.max_size {
            // En eski emri düşür.
            if let Some(oldest) = self.inner.keys().next().cloned() {
                self.inner.remove(&oldest);
            }
        }
        let prev = self.inner.insert(
            client_order_id.clone(),
            InFlightOrder {
                client_order_id,
                order_id,
                symbol,
                sent_at: Instant::now(),
                timeout_ms: timeout_ms.unwrap_or(self.default_timeout_ms),
            },
        );
        prev.is_none()
    }

    pub fn confirm(&mut self, client_order_id: &str) -> Option<InFlightOrder> {
        self.inner.remove(client_order_id)
    }

    pub fn confirm_by_order_id(&mut self, order_id: i64) -> Option<InFlightOrder> {
        let key = self
            .inner
            .iter()
            .find(|(_, o)| o.order_id == Some(order_id))
            .map(|(k, _)| k.clone());
        key.and_then(|k| self.inner.remove(&k))
    }

    pub fn set_order_id(&mut self, client_order_id: &str, order_id: i64) {
        if let Some(o) = self.inner.get_mut(client_order_id) {
            o.order_id = Some(order_id);
        }
    }

    pub fn get(&self, client_order_id: &str) -> Option<&InFlightOrder> {
        self.inner.get(client_order_id)
    }

    /// Zaman aşımına uğramış emirlerin client order id'leri.
    pub fn expired(&self, now: Instant) -> Vec<(String, Option<i64>, String)> {
        self.inner
            .iter()
            .filter(|(_, o)| o.is_expired(now))
            .map(|(k, o)| (k.clone(), o.order_id, o.symbol.clone()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
