//! force-orders client katmanı.
//!
//! Tüketici servisler bu crate'i kullanır:
//!   1. `client::read` — `/dev/shm/cycle_finance_force_orders` ring'inden binary
//!      kodlu likidasyon kayıtlarını okuyup `ForceOrder`'a çözer.
//!   2. HTTP API (`:3012`) ile son likidasyonları / özet istatistikleri çeker.
//!
//! Ring, üretici (force-orders servisi) tarafından yayınlanır; bu katman sadece okur.

use serde::{Deserialize, Serialize};

pub const DEFAULT_ADDR: &str = "http://127.0.0.1:3012";
/// Servise özel ring — `flow-liquidation`'ın yazdığı
/// `/cycle_finance_liquidations` (kanonik Liquidation event) ile karışmaz.
pub const RING_NAME: &str = "/cycle_finance_force_orders";
/// Ring'de tutulan maksimum likidasyon kaydı sayısı (dairesel — eskiler üzerine yazılır).
pub const RING_CAPACITY: usize = 10_000;

/// Binance Futures `forceOrder` event'i — servis tarafından parse edilen model.
///
/// ```json
/// { "e": "forceOrder", "E": 1568014460893, "o": { "s": "BTCUSDT", "S": "SELL",
///   "o": "LIMIT", "f": "IOC", "q": "0.014", "p": "9910", "ap": "9900",
///   "X": "FILLED", "l": "0.014", "z": "0.014", "T": 1568014460893 } }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceOrder {
    pub symbol: String,
    /// BUY (SHORT likidasyon) veya SELL (LONG likidasyon).
    pub side: String,
    pub order_type: String,
    /// Emir fiyatı (`o.p`).
    pub price: f64,
    /// Ortalama işlem fiyatı (`o.ap`).
    pub avg_price: f64,
    /// Orijinal emir miktarı (`o.q`).
    pub qty: f64,
    /// Doldurulan toplam miktar (`o.z`).
    pub filled: f64,
    /// `avg_price * qty` — USDT karşılığı.
    pub notional: f64,
    /// Event zamanı (`E`, Unix ms).
    pub event_ts: u64,
    /// Emir işlem zamanı (`o.T`, Unix ms).
    pub trade_ts: u64,
}

/// Side string'ini compact u8 koda çevirir (0 = BUY, 1 = SELL).
pub fn side_code(side: &str) -> u8 {
    if side.eq_ignore_ascii_case("BUY") {
        0
    } else {
        1
    }
}

/// Compact u8 kodunu side string'ine çevirir.
pub fn side_str(code: u8) -> &'static str {
    if code == 0 {
        "BUY"
    } else {
        "SELL"
    }
}

/// Unix ms → `HH:MM:SS.mmm` (UTC).
pub fn fmt_time_ms(ms: u64) -> String {
    chrono::DateTime::from_timestamp_millis(ms as i64)
        .map(|dt| dt.format("%H:%M:%S%.3f").to_string())
        .unwrap_or_else(|| ms.to_string())
}

pub mod codec {
    //! Binary encode/decode — ring slot'larına compact binary likidasyon kaydı.
    //!
    //! Düzen (little-endian):
    //! ```text
    //! [0..8)   event_ts   (u64)
    //! [8..16)  trade_ts   (u64)
    //! [16]     side       (u8: 0=BUY, 1=SELL)
    //! [17..25) price      (f64)
    //! [25..33) qty        (f64)
    //! [33..41) avg_price  (f64)
    //! [41..49) notional   (f64)
    //! [49]     sym_len    (u8)
    //! [50..]   symbol     (utf-8)
    //! ```

    use super::ForceOrder;

    pub fn encode(o: &ForceOrder) -> Vec<u8> {
        let sym = o.symbol.as_bytes();
        let mut buf = Vec::with_capacity(50 + sym.len());
        buf.extend_from_slice(&o.event_ts.to_le_bytes());
        buf.extend_from_slice(&o.trade_ts.to_le_bytes());
        buf.push(super::side_code(&o.side));
        buf.extend_from_slice(&o.price.to_le_bytes());
        buf.extend_from_slice(&o.qty.to_le_bytes());
        buf.extend_from_slice(&o.avg_price.to_le_bytes());
        buf.extend_from_slice(&o.notional.to_le_bytes());
        buf.push(sym.len().min(255) as u8);
        buf.extend_from_slice(&sym[..sym.len().min(255)]);
        buf
    }

    pub fn decode(bytes: &[u8]) -> Option<ForceOrder> {
        if bytes.len() < 50 {
            return None;
        }
        let u64_at = |off: usize| -> Option<u64> {
            let a: [u8; 8] = bytes.get(off..off + 8)?.try_into().ok()?;
            Some(u64::from_le_bytes(a))
        };
        let f64_at = |off: usize| -> Option<f64> {
            let a: [u8; 8] = bytes.get(off..off + 8)?.try_into().ok()?;
            Some(f64::from_le_bytes(a))
        };
        let event_ts = u64_at(0)?;
        let trade_ts = u64_at(8)?;
        let side = super::side_str(bytes[16]).to_string();
        let price = f64_at(17)?;
        let qty = f64_at(25)?;
        let avg_price = f64_at(33)?;
        let notional = f64_at(41)?;
        let sym_len = bytes[49] as usize;
        if 50 + sym_len > bytes.len() {
            return None;
        }
        let symbol = String::from_utf8_lossy(&bytes[50..50 + sym_len]).to_string();
        Some(ForceOrder {
            symbol,
            side,
            order_type: String::new(),
            price,
            avg_price,
            qty,
            filled: qty,
            notional,
            event_ts,
            trade_ts,
        })
    }
}

pub mod client {
    //! Ring okuma + HTTP istek.

    use super::{ForceOrder, RING_CAPACITY, RING_NAME};
    use transport::stream_ring::StreamRingBuffer;

    /// Ring'den `cursor`'dan itibaren tüm likidasyon kayıtlarını okur.
    ///
    /// Dönüş: (yeni cursor, kayıtlar). Tüketici cursor'ı ilerletip tekrar çağırır.
    pub fn read(cursor: u64, retries: u32, sleep_ms: u64) -> (u64, Vec<ForceOrder>) {
        use std::thread::sleep;
        use std::time::Duration;

        let ring = StreamRingBuffer::with_name(RING_NAME, RING_CAPACITY);
        let mut out = Vec::new();
        let mut next = cursor;
        for _ in 0..retries.max(1) {
            let head = ring.get_head();
            if head > next {
                for seq in next..head {
                    if let Some(slot) = ring.read_slot(seq) {
                        let bytes = &slot.data[..slot.len as usize];
                        if let Some(o) = super::codec::decode(bytes) {
                            out.push(o);
                        }
                    }
                }
                next = head;
                return (next, out);
            }
            sleep(Duration::from_millis(sleep_ms));
        }
        (next, out)
    }

    /// Son `limit` likidasyonu servis HTTP API'sinden çeker.
    pub async fn recent(
        addr: &str,
        limit: usize,
    ) -> Result<Vec<ForceOrder>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/liquidations?limit={}", addr, limit);
        let resp = reqwest::Client::new().get(&url).send().await?;
        let v: serde_json::Value = resp.json().await?;
        let items = v
            .get("liquidations")
            .ok_or("yanıtta liquidations alanı yok")?;
        serde_json::from_value(items.clone()).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_roundtrip() {
        let o = ForceOrder {
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "LIMIT".to_string(),
            price: 9910.0,
            avg_price: 9900.0,
            qty: 0.014,
            filled: 0.014,
            notional: 138.6,
            event_ts: 1786192080000,
            trade_ts: 1786192080000,
        };
        let bytes = codec::encode(&o);
        let dec = codec::decode(&bytes).expect("decode");
        assert_eq!(dec.symbol, o.symbol);
        assert_eq!(dec.side, o.side);
        assert_eq!(dec.event_ts, o.event_ts);
        assert_eq!(dec.trade_ts, o.trade_ts);
        assert!((dec.price - o.price).abs() < 1e-9);
        assert!((dec.avg_price - o.avg_price).abs() < 1e-9);
        assert!((dec.qty - o.qty).abs() < 1e-9);
        assert!((dec.notional - o.notional).abs() < 1e-9);
    }

    #[test]
    fn side_codes() {
        assert_eq!(side_code("BUY"), 0);
        assert_eq!(side_code("SELL"), 1);
        assert_eq!(side_str(0), "BUY");
        assert_eq!(side_str(1), "SELL");
    }
}
