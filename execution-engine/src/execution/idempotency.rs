//! Idempotency önbelleği.
//!
//! Aynı `client_order_id` ile gelen ikinci istek borsaya gitmez; ilk yanıt
//! yeniden döndürülür. Bu, ağ hatası sonrası yeniden denemede çift emiri önler.

use crate::order::OrderAck;
use std::collections::HashMap;

pub struct IdempotencyCache {
    inner: HashMap<String, OrderAck>,
    max_entries: usize,
}

impl IdempotencyCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            inner: HashMap::new(),
            max_entries,
        }
    }

    pub fn get(&self, client_order_id: &str) -> Option<OrderAck> {
        self.inner.get(client_order_id).cloned()
    }

    pub fn set(&mut self, client_order_id: String, ack: OrderAck) {
        if self.inner.len() >= self.max_entries
            && let Some(k) = self.inner.keys().next().cloned() {
                self.inner.remove(&k);
            }
        self.inner.insert(client_order_id, ack);
    }

    pub fn contains(&self, client_order_id: &str) -> bool {
        self.inner.contains_key(client_order_id)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
