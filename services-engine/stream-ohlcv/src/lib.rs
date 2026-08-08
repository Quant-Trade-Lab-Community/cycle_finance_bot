//! stream-ohlcv client katmanı.
//!
//! Tüketici servisler bu crate'i kullanır:
//!   1. `client::start(...)` — HTTP ile stream-ohlcv servisine istek atar
//!      ({symbol, start_ms, interval_secs}), `stream_id` alır.
//!   2. `client::read_candles(stream_id, cursor)` — `/dev/shm/cycle_finance_stream_ohlcv`
//!      ring'inden binary kodlu mumları okuyup `StreamCandle`'a çözer.
//!
//! Ring, üretici (stream-ohlcv servisi) tarafından yayınlanır; bu katman sadece okur.

use serde::{Deserialize, Serialize};

pub const DEFAULT_ADDR: &str = "http://127.0.0.1:3008";
pub const RING_NAME: &str = "/cycle_finance_stream_ohlcv";
/// Ring'de tutulan maksimum mum sayısı (dairesel — eskiler üzerine yazılır).
pub const RING_CAPACITY: usize = 8192;

/// Saniye cinsinden interval → Binance kline interval string'i.
///
/// >= 1m (60s) için Binance geçmişi çekilebilir; daha küçükler yalnızca
/// canlı price-feed'ten oluşturulur (Binance Futures geçmişi 1s altını desteklemez).
pub fn binance_interval(secs: u64) -> Option<&'static str> {
    match secs {
        1 => Some("1s"),
        5 => Some("5s"),
        15 => Some("15s"),
        30 => Some("30s"),
        60 => Some("1m"),
        120 => Some("2m"),
        180 => Some("3m"),
        300 => Some("5m"),
        900 => Some("15m"),
        1800 => Some("30m"),
        3600 => Some("1h"),
        7200 => Some("2h"),
        10800 => Some("3h"),
        14400 => Some("4h"),
        21600 => Some("6h"),
        28800 => Some("8h"),
        43200 => Some("12h"),
        86400 => Some("1d"),
        _ => None,
    }
}

/// Stream açma isteği (HTTP gövdesi).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRequest {
    pub symbol: String,
    /// Unix ms — geçmişin nereden çekileceği (günümüze kadar).
    pub start_ms: u64,
    /// Mum periyodu (saniye cinsinden), örn. 60, 300, 3600.
    pub interval_secs: u64,
}

impl StreamRequest {
    pub fn new(symbol: &str, start_ms: u64, interval_secs: u64) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            start_ms,
            interval_secs,
        }
    }
}

/// Stream yaşam döngüsü durumu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamStatus {
    Starting,
    Running,
    Stopped,
    Error(String),
}

/// Ring üzerinden yayınlanan mum — fiyatlar binary codec için f64.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamCandle {
    pub stream_id: u64,
    pub symbol: String,
    pub interval_secs: u64,
    pub open_time: u64,
    pub close_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    /// 1 = mum kapanmış (yayınlandı), 0 = oluşan (canlı güncellenen) mum.
    pub closed: u8,
}

/// Stream meta bilgisi — API yanıtı / durum sorgusu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMeta {
    pub stream_id: u64,
    pub symbol: String,
    pub start_ms: u64,
    pub interval_secs: u64,
    pub created: u64,
    pub status: StreamStatus,
    /// Bugüne kadar yayınlanan toplam mum sayısı.
    pub published: u64,
    /// En son görülen fiyat (price-feed lastprice).
    pub last_price: Option<f64>,
    /// Şu an oluşan mum (varsa).
    pub current: Option<StreamCandle>,
}

pub mod codec {
    //! Binary encode/decode — stream_ring slot'larına compact binary mum.

    use super::StreamCandle;

    pub fn encode(c: &StreamCandle) -> Vec<u8> {
        let sym = c.symbol.as_bytes();
        let mut buf = Vec::with_capacity(74 + sym.len());
        buf.extend_from_slice(&c.stream_id.to_le_bytes());
        buf.extend_from_slice(&c.interval_secs.to_le_bytes());
        buf.extend_from_slice(&c.open_time.to_le_bytes());
        buf.extend_from_slice(&c.close_time.to_le_bytes());
        buf.extend_from_slice(&c.open.to_le_bytes());
        buf.extend_from_slice(&c.high.to_le_bytes());
        buf.extend_from_slice(&c.low.to_le_bytes());
        buf.extend_from_slice(&c.close.to_le_bytes());
        buf.extend_from_slice(&c.volume.to_le_bytes());
        buf.push(c.closed);
        buf.push(sym.len().min(255) as u8);
        buf.extend_from_slice(&sym[..sym.len().min(255)]);
        buf
    }

    pub fn decode(bytes: &[u8]) -> Option<StreamCandle> {
        if bytes.len() < 74 {
            return None;
        }
        let u64_at = |off: usize| -> Option<u64> {
            let arr: [u8; 8] = bytes.get(off..off + 8)?.try_into().ok()?;
            Some(u64::from_le_bytes(arr))
        };
        let f64_at = |off: usize| -> Option<f64> {
            let arr: [u8; 8] = bytes.get(off..off + 8)?.try_into().ok()?;
            Some(f64::from_le_bytes(arr))
        };
        let stream_id = u64_at(0)?;
        let interval_secs = u64_at(8)?;
        let open_time = u64_at(16)?;
        let close_time = u64_at(24)?;
        let open = f64_at(32)?;
        let high = f64_at(40)?;
        let low = f64_at(48)?;
        let close = f64_at(56)?;
        let volume = f64_at(64)?;
        let closed = bytes[72];
        let sym_len = bytes[73] as usize;
        if 74 + sym_len > bytes.len() {
            return None;
        }
        let symbol = String::from_utf8_lossy(&bytes[74..74 + sym_len]).to_string();
        Some(StreamCandle {
            stream_id,
            symbol,
            interval_secs,
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume,
            closed,
        })
    }
}

pub mod client {
    //! HTTP istek + ring okuma.

    use super::{RING_NAME, RING_CAPACITY, StreamCandle, StreamMeta, StreamRequest};

    /// stream-ohlcv servisine istek atar, stream meta bilgisi döndürür.
    pub async fn start(
        addr: &str,
        req: &StreamRequest,
    ) -> Result<StreamMeta, Box<dyn std::error::Error>> {
        let url = format!("{}/api/stream", addr);
        let resp = reqwest::Client::new()
            .post(&url)
            .json(req)
            .send()
            .await?;
        let v: serde_json::Value = resp.json().await?;
        if let Ok(meta) = serde_json::from_value::<StreamMeta>(v.clone()) {
            return Ok(meta);
        }
        let msg = v
            .get("error")
            .map(|e| e.to_string())
            .unwrap_or_else(|| "bilinmeyen hata".into());
        Err(msg.into())
    }

    /// Mevcut stream'lerin listesini döndürür.
    pub async fn list(addr: &str) -> Result<Vec<StreamMeta>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/streams", addr);
        let resp = reqwest::Client::new().get(&url).send().await?;
        let v: serde_json::Value = resp.json().await?;
        serde_json::from_value(v).map_err(|_| "yanıt ayrıştırılamadı".into())
    }

    /// Stream'i durdurur.
    pub async fn stop(addr: &str, stream_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/stream/{}", addr, stream_id);
        let resp = reqwest::Client::new().delete(&url).send().await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err("durma isteği başarısız".into())
        }
    }

    /// Ring'den `cursor`'dan itibaren tüm mumları okur; sadece `stream_id`'ye ait olanları döndürür.
    ///
    /// Dönüş: (yeni cursor, mumlar). Tüketici cursor'ı ilerletip tekrar çağırır.
    pub fn read_candles(
        stream_id: u64,
        cursor: u64,
        retries: u32,
        sleep_ms: u64,
    ) -> (u64, Vec<StreamCandle>) {
        use std::thread::sleep;
        use std::time::Duration;

        let ring = transport::stream_ring::StreamRingBuffer::with_name(RING_NAME, RING_CAPACITY);

        let mut out = Vec::new();
        let mut next = cursor;
        for _ in 0..retries.max(1) {
            let head_now = ring.get_head();
            if head_now > next {
                for seq in next..head_now {
                    if let Some(slot) = ring.read_slot(seq) {
                        let bytes = &slot.data[..slot.len as usize];
                        if let Some(c) = super::codec::decode(bytes) {
                            if c.stream_id == stream_id {
                                out.push(c);
                            }
                        }
                    }
                }
                next = head_now;
                return (next, out);
            }
            sleep(Duration::from_millis(sleep_ms));
        }
        // head boşta da olsa son bir kez tara (dairesel taşma durumunda).
        let head_now = ring.get_head();
        if head_now > next {
            for seq in next..head_now {
                if let Some(slot) = ring.read_slot(seq) {
                    let bytes = &slot.data[..slot.len as usize];
                    if let Some(c) = super::codec::decode(bytes) {
                        if c.stream_id == stream_id {
                            out.push(c);
                        }
                    }
                }
            }
            next = head_now;
        }
        (next, out)
    }

    /// Varsayılan adresle stream başlatır.
    pub async fn start_default(req: &StreamRequest) -> Result<StreamMeta, Box<dyn std::error::Error>> {
        start(super::DEFAULT_ADDR, req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_roundtrip() {
        let c = StreamCandle {
            stream_id: 42,
            symbol: "BTCUSDT".to_string(),
            interval_secs: 60,
            open_time: 1786192080000,
            close_time: 1786192139999,
            open: 100.5,
            high: 105.25,
            low: 99.75,
            close: 103.0,
            volume: 1234.567,
            closed: 1,
        };
        let bytes = codec::encode(&c);
        let dec = codec::decode(&bytes).expect("decode");
        assert_eq!(dec.stream_id, c.stream_id);
        assert_eq!(dec.symbol, c.symbol);
        assert_eq!(dec.interval_secs, c.interval_secs);
        assert_eq!(dec.open_time, c.open_time);
        assert_eq!(dec.close_time, c.close_time);
        assert!((dec.open - c.open).abs() < 1e-9);
        assert!((dec.high - c.high).abs() < 1e-9);
        assert!((dec.low - c.low).abs() < 1e-9);
        assert!((dec.close - c.close).abs() < 1e-9);
        assert!((dec.volume - c.volume).abs() < 1e-9);
        assert_eq!(dec.closed, 1);
    }

    #[test]
    fn interval_mapping() {
        assert_eq!(binance_interval(60), Some("1m"));
        assert_eq!(binance_interval(300), Some("5m"));
        assert_eq!(binance_interval(3600), Some("1h"));
        assert_eq!(binance_interval(86400), Some("1d"));
        assert_eq!(binance_interval(45), None);
    }
}
