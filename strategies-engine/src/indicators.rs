//! OHLCV göstergeleri — kırılım algoritmasının ihtiyaç duyduğu türetilmiş değerler.
//!
//! - `atr(14)`: Ortalama Gerçek Aralık (volatilite σ)
//! - `sma(20)`: Ortalama işlem hacmi (V_avg)
//! - `high_14 / low_14`: son 14 barın en yüksek/en düşük fiyatı

use ohlcv_engine::Kline;
use rust_decimal::prelude::ToPrimitive;

fn f(c: &Kline, sel: fn(&Kline) -> rust_decimal::Decimal) -> f64 {
    sel(c).to_f64().unwrap_or(0.0)
}

/// True Range: max(high-low, |high-prev_close|, |low-prev_close|).
fn true_range(c: &Kline, prev_close: f64) -> f64 {
    let high = f(c, |k| k.high);
    let low = f(c, |k| k.low);
    let hl = high - low;
    let hc = (high - prev_close).abs();
    let lc = (low - prev_close).abs();
    hl.max(hc).max(lc)
}

/// ATR(period) — son `period` barın ortalama gerçek aralığı.
pub fn atr(candles: &[Kline], period: usize) -> f64 {
    if candles.len() < period + 1 {
        return 0.0;
    }
    let start = candles.len() - period;
    let mut sum = 0.0;
    for i in start..candles.len() {
        let prev_close = f(&candles[i - 1], |k| k.close);
        sum += true_range(&candles[i], prev_close);
    }
    sum / period as f64
}

/// SMA(values, period) — son `period` değerin ortalaması.
pub fn sma(values: &[f64], period: usize) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let start = values.len().saturating_sub(period);
    let slice = &values[start..];
    slice.iter().sum::<f64>() / slice.len() as f64
}

/// Son `period` barın en yüksek / en düşük fiyatı.
pub fn high_low(candles: &[Kline], period: usize) -> (f64, f64) {
    let start = candles.len().saturating_sub(period);
    let slice = &candles[start..];
    let mut high = f64::NEG_INFINITY;
    let mut low = f64::INFINITY;
    for c in slice {
        let h = f(c, |k| k.high);
        let l = f(c, |k| k.low);
        if h > high {
            high = h;
        }
        if l < low {
            low = l;
        }
    }
    (if high.is_finite() { high } else { 0.0 }, if low.is_finite() { low } else { 0.0 })
}

/// Son mumun OHLCV değerleri.
pub fn last_candle(candles: &[Kline]) -> Option<(f64, f64, f64, f64, f64)> {
    candles.last().map(|c| (f(c, |k| k.open), f(c, |k| k.high), f(c, |k| k.low), f(c, |k| k.close), f(c, |k| k.volume)))
}
