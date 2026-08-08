//! İndikatör hesaplama katmanı — ferro_ta_core üzerinde ince dispatch.
//!
//! Girdi: OHLCV kline'ları + indikatör adı + parametreler (HashMap).
//! Çıktı: adlandırılmış seriler (Vec<f64>), her biri kline sayısı uzunluğunda.
//! Warm-up dönemleri NaN olarak korunur (ferro_ta_core davranışı).

use std::collections::HashMap;
use rust_decimal::prelude::*;
use ohlcv_engine::Kline;

/// Hesaplanmış bir indikatör çıktısı: seri adı → değerler.
/// `Option<f64>`: NaN (warm-up) değerleri `None` olarak taşınır (serde_json `null`).
pub type IndicatorSeries = HashMap<String, Vec<Option<f64>>>;

fn close_f64(klines: &[Kline]) -> Vec<f64> {
    klines.iter().map(|k| k.close.to_f64().unwrap_or(f64::NAN)).collect()
}

fn high_f64(klines: &[Kline]) -> Vec<f64> {
    klines.iter().map(|k| k.high.to_f64().unwrap_or(f64::NAN)).collect()
}

fn low_f64(klines: &[Kline]) -> Vec<f64> {
    klines.iter().map(|k| k.low.to_f64().unwrap_or(f64::NAN)).collect()
}

fn vol_f64(klines: &[Kline]) -> Vec<f64> {
    klines.iter().map(|k| k.volume.to_f64().unwrap_or(f64::NAN)).collect()
}

fn p<'a>(params: &'a HashMap<String, f64>, key: &str, default: f64) -> f64 {
    params.get(key).copied().unwrap_or(default)
}

/// f64 vektörünü Option vektörüne çevirir (NaN → None).
fn opt(v: Vec<f64>) -> Vec<Option<f64>> {
    v.into_iter().map(|x| if x.is_nan() { None } else { Some(x) }).collect()
}

/// Hacim ağırlıklı ortalama fiyat (VWAP) — seri olarak.
fn vwap(klines: &[Kline]) -> Vec<f64> {
    let mut cum_pv = 0.0;
    let mut cum_v = 0.0;
    klines
        .iter()
        .map(|k| {
            let tp = (k.high + k.low + k.close).to_f64().unwrap_or(0.0) / 3.0;
            let v = k.volume.to_f64().unwrap_or(0.0);
            cum_pv += tp * v;
            cum_v += v;
            if cum_v > 0.0 { cum_pv / cum_v } else { f64::NAN }
        })
        .collect()
}

/// İndikatörü hesaplar. Bilinmeyen indikatör adı için Err döner.
pub fn calc_indicator(
    name: &str,
    klines: &[Kline],
    params: &HashMap<String, f64>,
) -> Result<IndicatorSeries, String> {
    let close = close_f64(klines);
    let high = high_f64(klines);
    let low = low_f64(klines);

    let mut out = IndicatorSeries::new();
    match name.to_ascii_lowercase().as_str() {
        "sma" => {
            let period = p(params, "period", 20.0).max(1.0) as usize;
            out.insert("sma".into(), opt(ferro_ta_core::overlap::sma(&close, period)));
        }
        "ema" => {
            let period = p(params, "period", 20.0).max(1.0) as usize;
            out.insert("ema".into(), opt(ferro_ta_core::overlap::ema(&close, period)));
        }
        "wma" => {
            let period = p(params, "period", 20.0).max(1.0) as usize;
            out.insert("wma".into(), opt(ferro_ta_core::overlap::wma(&close, period)));
        }
        "macd" => {
            let fast = p(params, "fast", 12.0).max(1.0) as usize;
            let slow = p(params, "slow", 26.0).max(fast as f64 + 1.0) as usize;
            let signal = p(params, "signal", 9.0).max(1.0) as usize;
            let (m, s, h) = ferro_ta_core::overlap::macd(&close, fast, slow, signal);
            out.insert("macd".into(), opt(m));
            out.insert("signal".into(), opt(s));
            out.insert("histogram".into(), opt(h));
        }
        "bbands" | "bb" => {
            let period = p(params, "period", 20.0).max(1.0) as usize;
            let nbdev = p(params, "nbdev", 2.0);
            let (upper, mid, lower) = ferro_ta_core::overlap::bbands(&close, period, nbdev, nbdev);
            out.insert("upper".into(), opt(upper));
            out.insert("middle".into(), opt(mid));
            out.insert("lower".into(), opt(lower));
        }
        "rsi" => {
            let period = p(params, "period", 14.0).max(1.0) as usize;
            out.insert("rsi".into(), opt(ferro_ta_core::momentum::rsi(&close, period)));
        }
        "stoch" => {
            let fastk = p(params, "fastk", 14.0).max(1.0) as usize;
            let slowk = p(params, "slowk", 3.0).max(1.0) as usize;
            let slowd = p(params, "slowd", 3.0).max(1.0) as usize;
            let (k, d) = ferro_ta_core::momentum::stoch(&high, &low, &close, fastk, slowk, slowd);
            out.insert("stoch_k".into(), opt(k));
            out.insert("stoch_d".into(), opt(d));
        }
        "momentum" | "mom" => {
            let period = p(params, "period", 10.0).max(1.0) as usize;
            out.insert("momentum".into(), opt(ferro_ta_core::momentum::mom(&close, period)));
        }
        "roc" => {
            let period = p(params, "period", 12.0).max(1.0) as usize;
            out.insert("roc".into(), opt(ferro_ta_core::momentum::roc(&close, period)));
        }
        "stddev" => {
            let period = p(params, "period", 20.0).max(1.0) as usize;
            let nbdev = p(params, "nbdev", 1.0);
            out.insert("stddev".into(), opt(ferro_ta_core::statistic::stddev(&close, period, nbdev)));
        }
        "atr" => {
            let period = p(params, "period", 14.0).max(1.0) as usize;
            out.insert("atr".into(), opt(ferro_ta_core::volatility::atr(&high, &low, &close, period)));
        }
        "vwap" => {
            out.insert("vwap".into(), opt(vwap(klines)));
        }
        "volume" => {
            out.insert("volume".into(), opt(vol_f64(klines)));
        }
        _ => return Err(format!("Bilinmeyen indikatör: {name}")),
    }
    Ok(out)
}
