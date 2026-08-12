//! trade-ohlcv client katmanı.
//!
//! Trade data'dan üretilen **1 saniyelik OHLCV** mumlarının üç parçası:
//!
//!   1. `SecondAggregator` — trade akışını 1s barına çeviren saf toplayıcı
//!      (I/O'suz, birim test edilebilir).
//!   2. `codec` — mumu `/cycle_finance_trade_ohlcv` ring slot'una binary kodlar.
//!   3. `client` — ring'den binary mumları okuyup `TradeCandle`'a çözer.
//!
//! Veri akışı:
//!
//! ```text
//! /dev/shm/cycle_finance_trades (flow ring, Trade event'leri)
//!   └── trade-ohlcv daemon → SecondAggregator → 1s mum
//!        └── /dev/shm/cycle_finance_trade_ohlcv (stream ring, binary mum)
//!             └── tüketici: trade_ohlcv::client (ring okuma)
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_ADDR: &str = "http://127.0.0.1:3009";
/// Trade data'dan üretilen 1s mumların yayınlandığı ring.
pub const RING_NAME: &str = "/cycle_finance_trade_ohlcv";
/// Ring'de tutulan maksimum mum sayısı (dairesel — eskiler üzerine yazılır).
pub const RING_CAPACITY: usize = 8192;

/// 1 saniyelik OHLCV mumu — trade data'dan üretilir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCandle {
    pub symbol: String,
    /// Her zaman 1 (saniyelik bar).
    pub interval_secs: u64,
    pub open_time: u64,
    pub close_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    /// Base asset hacmi (trade quantity toplamı).
    pub volume: f64,
    /// Taker alım base hacmi (agresif alıcı trade'lerinin quantity toplamı).
    pub taker_buy_volume: f64,
    /// Barı oluşturan trade sayısı.
    pub trades: u64,
    /// 1 = mum kapanmış (yayınlandı), 0 = oluşan (canlı güncellenen) mum.
    pub closed: u8,
}

impl TradeCandle {
    pub fn closed(&self) -> bool {
        self.closed != 0
    }
}

/// Trade data'yı sembol başına 1 saniyelik OHLCV barına çeviren saf toplayıcı.
///
/// Her sembol için oluşan bir bar tutar; trade'in timestamp'inden 1s dilimi
/// (bucket) hesaplanır. Dilim değişince mevcut bar kapanır (döndürülür) ve
/// yeni bar başlar. Gecikmiş trade (geçmiş dilime ait) sessizce atılır.
pub struct SecondAggregator {
    forming: HashMap<String, FormingCandle>,
}

struct FormingCandle {
    open_time: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    taker_buy_volume: f64,
    trades: u64,
}

impl Default for SecondAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl SecondAggregator {
    pub fn new() -> Self {
        Self {
            forming: HashMap::new(),
        }
    }

    /// Trade'i işler. Sembolün 1s dilimi kapanırsa kapalı `TradeCandle` döndürür
    /// (tüketici bunu ring'e yayınlar), aksi halde `None`.
    ///
    /// `price` ve `quantity` zaten f64'e çevrilmiş sayılardır; `ts_ms` trade
    /// zamanı (Unix ms), `is_buyer_maker` true ise alıcı maker'dır (taker satıcı).
    pub fn on_trade(
        &mut self,
        symbol: &str,
        price: f64,
        quantity: f64,
        ts_ms: u64,
        is_buyer_maker: bool,
    ) -> Option<TradeCandle> {
        let sym = symbol.to_ascii_uppercase();
        let bucket = ts_ms - (ts_ms % 1000);
        let mut closed = None;

        match self.forming.get_mut(&sym) {
            Some(c) if c.open_time == bucket => {
                c.high = c.high.max(price);
                c.low = c.low.min(price);
                c.close = price;
                c.volume += quantity;
                if !is_buyer_maker {
                    c.taker_buy_volume += quantity;
                }
                c.trades += 1;
            }
            Some(c) if c.open_time < bucket => {
                // Dilim değişti — eski barı kapat, yenisi için formi bırak.
                let old = std::mem::replace(
                    c,
                    FormingCandle {
                        open_time: bucket,
                        open: price,
                        high: price,
                        low: price,
                        close: price,
                        volume: quantity,
                        taker_buy_volume: if is_buyer_maker { 0.0 } else { quantity },
                        trades: 1,
                    },
                );
                closed = Some(to_candle(&sym, &old, 1));
            }
            Some(_) => {
                // Gecikmiş trade (geçmiş dilim) — sessizce atla.
            }
            None => {
                self.forming.insert(
                    sym.clone(),
                    FormingCandle {
                        open_time: bucket,
                        open: price,
                        high: price,
                        low: price,
                        close: price,
                        volume: quantity,
                        taker_buy_volume: if is_buyer_maker { 0.0 } else { quantity },
                        trades: 1,
                    },
                );
            }
        }

        closed
    }

    /// Şu an oluşan mumu (kapalı değil) verir.
    pub fn forming(&self, symbol: &str) -> Option<TradeCandle> {
        self.forming
            .get(&symbol.to_ascii_uppercase())
            .map(|f| to_candle(&symbol.to_ascii_uppercase(), f, 0))
    }

    /// Tüm oluşan mumlar (kapalı değil) — API/status için.
    pub fn all_forming(&self) -> Vec<TradeCandle> {
        let mut out: Vec<TradeCandle> = self
            .forming
            .iter()
            .map(|(sym, f)| to_candle(sym, f, 0))
            .collect();
        out.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        out
    }

    /// Şu an izlenen sembol sayısı.
    pub fn symbol_count(&self) -> usize {
        self.forming.len()
    }
}

fn to_candle(symbol: &str, f: &FormingCandle, closed: u8) -> TradeCandle {
    TradeCandle {
        symbol: symbol.to_string(),
        interval_secs: 1,
        open_time: f.open_time,
        close_time: f.open_time + 999,
        open: f.open,
        high: f.high,
        low: f.low,
        close: f.close,
        volume: f.volume,
        taker_buy_volume: f.taker_buy_volume,
        trades: f.trades,
        closed,
    }
}

pub mod codec {
    //! Binary encode/decode — trade-ohlcv ring slot'larına compact binary mum.

    use super::TradeCandle;

    /// Sabit başlık + değişken sembol: 74 + sym_len bayt.
    pub fn encode(c: &TradeCandle) -> Vec<u8> {
        let sym = c.symbol.as_bytes();
        let mut buf = Vec::with_capacity(74 + sym.len());
        buf.extend_from_slice(&c.open_time.to_le_bytes());
        buf.extend_from_slice(&c.close_time.to_le_bytes());
        buf.extend_from_slice(&c.open.to_le_bytes());
        buf.extend_from_slice(&c.high.to_le_bytes());
        buf.extend_from_slice(&c.low.to_le_bytes());
        buf.extend_from_slice(&c.close.to_le_bytes());
        buf.extend_from_slice(&c.volume.to_le_bytes());
        buf.extend_from_slice(&c.taker_buy_volume.to_le_bytes());
        buf.extend_from_slice(&c.trades.to_le_bytes());
        buf.push(c.closed);
        buf.push(sym.len().min(255) as u8);
        buf.extend_from_slice(&sym[..sym.len().min(255)]);
        buf
    }

    pub fn decode(bytes: &[u8]) -> Option<TradeCandle> {
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
        let open_time = u64_at(0)?;
        let close_time = u64_at(8)?;
        let open = f64_at(16)?;
        let high = f64_at(24)?;
        let low = f64_at(32)?;
        let close = f64_at(40)?;
        let volume = f64_at(48)?;
        let taker_buy_volume = f64_at(56)?;
        let trades = u64_at(64)?;
        let closed = bytes[72];
        let sym_len = bytes[73] as usize;
        if 74 + sym_len > bytes.len() {
            return None;
        }
        let symbol = String::from_utf8_lossy(&bytes[74..74 + sym_len]).to_string();
        Some(TradeCandle {
            symbol,
            interval_secs: 1,
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume,
            taker_buy_volume,
            trades,
            closed,
        })
    }
}

pub mod client {
    //! Ring okuma — `/cycle_finance_trade_ohlcv`'den binary mumları çözer.

    use super::{RING_CAPACITY, RING_NAME, TradeCandle};

    /// Ring'den `cursor`'dan itibaren tüm mumları okur.
    ///
    /// Dönüş: (yeni cursor, mumlar). Tüketici cursor'ı ilerletip tekrar çağırır.
    pub fn read_candles(
        cursor: u64,
        retries: u32,
        sleep_ms: u64,
    ) -> (u64, Vec<TradeCandle>) {
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
                            out.push(c);
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
                        out.push(c);
                    }
                }
            }
            next = head_now;
        }
        (next, out)
    }

    /// Son kapanmış mumları döndürür (cursor atanır, eski mumların üzerinden geçilir).
    pub fn read_latest(limit: usize) -> Vec<TradeCandle> {
        let ring = transport::stream_ring::StreamRingBuffer::with_name(RING_NAME, RING_CAPACITY);
        let head = ring.get_head();
        let start = head.saturating_sub(limit as u64);
        let mut out = Vec::new();
        for seq in start..head {
            if let Some(slot) = ring.read_slot(seq) {
                let bytes = &slot.data[..slot.len as usize];
                if let Some(c) = super::codec::decode(bytes) {
                    out.push(c);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_roundtrip() {
        let c = TradeCandle {
            symbol: "BTCUSDT".to_string(),
            interval_secs: 1,
            open_time: 1786192080000,
            close_time: 1786192080999,
            open: 67123.5,
            high: 67130.25,
            low: 67120.1,
            close: 67125.4,
            volume: 12.345,
            taker_buy_volume: 7.2,
            trades: 42,
            closed: 1,
        };
        let bytes = codec::encode(&c);
        let dec = codec::decode(&bytes).expect("decode");
        assert_eq!(dec.symbol, c.symbol);
        assert_eq!(dec.open_time, c.open_time);
        assert_eq!(dec.close_time, c.close_time);
        assert_eq!(dec.trades, c.trades);
        assert_eq!(dec.closed, 1);
        assert!((dec.open - c.open).abs() < 1e-9);
        assert!((dec.high - c.high).abs() < 1e-9);
        assert!((dec.low - c.low).abs() < 1e-9);
        assert!((dec.close - c.close).abs() < 1e-9);
        assert!((dec.volume - c.volume).abs() < 1e-9);
        assert!((dec.taker_buy_volume - c.taker_buy_volume).abs() < 1e-9);
    }

    #[test]
    fn single_second_bar() {
        let mut agg = SecondAggregator::new();
        // Aynı 1s dilimi (12:00:01.000 .. 12:00:01.999)
        assert!(agg.on_trade("BTCUSDT", 100.0, 1.0, 1786192080000, false).is_none());
        assert!(agg.on_trade("BTCUSDT", 102.5, 2.0, 1786192080500, false).is_none());
        assert!(agg.on_trade("BTCUSDT", 99.0, 0.5, 1786192080900, true).is_none());

        let forming = agg.forming("BTCUSDT").expect("forming mum var");
        assert_eq!(forming.closed, 0);
        assert!((forming.open - 100.0).abs() < 1e-9);
        assert!((forming.high - 102.5).abs() < 1e-9);
        assert!((forming.low - 99.0).abs() < 1e-9);
        assert!((forming.close - 99.0).abs() < 1e-9);
        assert!((forming.volume - 3.5).abs() < 1e-9);
        // taker alım: ilk iki trade alıcı taker (is_buyer_maker=false)
        assert!((forming.taker_buy_volume - 3.0).abs() < 1e-9);
        assert_eq!(forming.trades, 3);
    }

    #[test]
    fn bucket_rollover_closes_bar() {
        let mut agg = SecondAggregator::new();
        assert!(agg.on_trade("ETHUSDT", 3000.0, 1.0, 1000, false).is_none());
        // Bir sonraki saniye dilimi → kapalı mum döner
        let closed = agg.on_trade("ETHUSDT", 3010.0, 2.0, 2000, true).expect("kapalı mum");
        assert_eq!(closed.closed, 1);
        assert_eq!(closed.open_time, 1000);
        assert_eq!(closed.close_time, 1999);
        assert!((closed.open - 3000.0).abs() < 1e-9);
        assert!((closed.close - 3000.0).abs() < 1e-9);
        assert_eq!(closed.trades, 1);

        let forming = agg.forming("ETHUSDT").expect("yeni formi var");
        assert_eq!(forming.open_time, 2000);
        assert!((forming.open - 3010.0).abs() < 1e-9);
    }

    #[test]
    fn late_trade_is_ignored() {
        let mut agg = SecondAggregator::new();
        assert!(agg.on_trade("X", 10.0, 1.0, 5000, false).is_none());
        // Geçmiş dilime ait gecikmiş trade → atlanır
        assert!(agg.on_trade("X", 11.0, 1.0, 4000, false).is_none());
        let forming = agg.forming("X").expect("formi var");
        assert_eq!(forming.open_time, 5000);
        assert_eq!(forming.trades, 1);
    }

    #[test]
    fn per_symbol_isolation() {
        let mut agg = SecondAggregator::new();
        assert!(agg.on_trade("BTCUSDT", 100.0, 1.0, 1000, false).is_none());
        assert!(agg.on_trade("ETHUSDT", 3000.0, 1.0, 1000, false).is_none());
        assert_eq!(agg.symbol_count(), 2);
        // BTC dilimi değişince yalnızca BTC kapanır
        let closed = agg.on_trade("BTCUSDT", 101.0, 1.0, 2000, false).expect("kapalı");
        assert_eq!(closed.symbol, "BTCUSDT");
        assert!(agg.forming("ETHUSDT").is_some());
    }
}
