//! calc-ind client katmanı.
//!
//! Tüketici servisler bu crate'i kullanır:
//!   1. `client::request(...)` — HTTP ile calc-ind servisine istek atar, `request_id` alır.
//!   2. `client::read_result(request_id)` — sonucu `/dev/shm/cycle_finance_calc`
//!      ring'inden binary olarak okuyup `CalcResult`'a çözer.
//!
//! Ring, üretici (calc-ind servisi) tarafından yayınlanır; bu katman sadece okur.

pub mod indicators;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// İndikatör hesaplama isteği (HTTP gövdesi).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndRequest {
    pub symbol: String,
    pub interval: String,
    /// Unix ms — opsiyonel
    pub start_ms: Option<u64>,
    /// Unix ms — opsiyonel
    pub end_ms: Option<u64>,
    pub indicator: String,
    pub params: HashMap<String, f64>,
}

impl IndRequest {
    pub fn new(
        symbol: &str,
        interval: &str,
        start_ms: Option<u64>,
        end_ms: Option<u64>,
        indicator: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            start_ms,
            end_ms,
            indicator: indicator.to_string(),
            params: HashMap::new(),
        }
    }

    pub fn with_params(mut self, params: HashMap<String, f64>) -> Self {
        self.params = params;
        self
    }
}

/// Tek bir kline'ın hafifletilmiş temsili (binary iletim için).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcKline {
    pub open_time: u64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

/// Hesaplanmış sonuç — isteğin kimliği + kline'lar + indikatör serileri.
/// `Option<f64>`: warm-up dönemindeki NaN'lar `null` olarak serileştirilir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcResult {
    pub request_id: u64,
    pub symbol: String,
    pub interval: String,
    pub indicator: String,
    pub klines: Vec<CalcKline>,
    pub series: HashMap<String, Vec<Option<f64>>>,
}

pub mod codec {
    //! Binary encode/decode — ring slot'a sığacak şekilde compact JSON (binary blob).

    use super::CalcResult;

    pub fn encode(result: &CalcResult) -> Vec<u8> {
        serde_json::to_vec(result).unwrap_or_default()
    }

    pub fn decode(bytes: &[u8]) -> Option<CalcResult> {
        serde_json::from_slice(bytes).ok()
    }
}

pub mod client {
    //! HTTP istek + ring okuma.

    use super::{CalcResult, IndRequest};
    use crate::codec;

    const DEFAULT_ADDR: &str = "http://127.0.0.1:3007";
    const RING_NAME: &str = "/cycle_finance_calc";

    /// calc-ind servisine istek atar, `request_id` döndürür.
    pub async fn request(
        addr: &str,
        req: &IndRequest,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let url = format!("{}/api/calc", addr);
        let resp = reqwest::Client::new()
            .post(&url)
            .json(req)
            .send()
            .await?;
        let v: serde_json::Value = resp.json().await?;
        if let Some(id) = v.get("request_id").and_then(|x| x.as_u64()) {
            Ok(id)
        } else {
            let msg = v.get("error").map(|e| e.to_string()).unwrap_or_else(|| "bilinmeyen hata".into());
            Err(msg.into())
        }
    }

    /// Sonucu ring'den okuyup çözer. `retries` kadar bekler (üretici henüz yazmamış olabilir).
    pub fn read_result(
        request_id: u64,
        retries: u32,
        sleep_ms: u64,
    ) -> Option<CalcResult> {
        use std::thread::sleep;
        use std::time::Duration;

        let ring = transport::calc_ring::CalcRingBuffer::with_name(RING_NAME, 64);
        let head = ring.get_head();

        for _ in 0..retries.max(1) {
            // En güncel slotlardan geriye doğru tara (head-1, head-2, ...)
            let start = head.saturating_sub(1);
            for back in 0..16u64 {
                let seq = start.saturating_sub(back);
                if let Some(slot) = ring.read_slot(seq) {
                    let bytes = &slot.data[..slot.len as usize];
                    if let Some(res) = codec::decode(bytes) {
                        if res.request_id == request_id {
                            return Some(res);
                        }
                    }
                }
            }
            sleep(Duration::from_millis(sleep_ms));
        }
        None
    }

    /// Varsayılan adresle istek atar.
    pub async fn request_default(req: &IndRequest) -> Result<u64, Box<dyn std::error::Error>> {
        request(DEFAULT_ADDR, req).await
    }
}
