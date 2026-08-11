//! BREAKOUT Kırılım Stratejisi (Rust) — Kripto Futures Tek Zaman Dilimi
//!
//! **Sinyal üretici mod**: Emir AÇMAZ. Kırılım algılandığında sembol + yön
//! (BUY/SELL) + kalite/sahte/kesinlik skorları üretir.
//!
//! Akış:
//! ```text
//! ohlcv-engine (Binance klines, N=200 mum)  [10 sn'de bir tazelenir]
//!   + detect-ms (:3002) seviyeleri (SH→R, SL→S)  [10 sn'de bir]
//!   + flow ring'leri (CVD, OI, funding, mark, last, liq)  [her saniye]
//!   → breakout::compute() → {direction, quality, fake, certainty}
//! ```

use breakout_strategy::breakout::{self, BreakoutInput};
use breakout_strategy::feed::Feed;
use breakout_strategy::indicators;
use ohlcv_engine::client::BinanceClient;
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value;
use std::env;
use std::time::Duration;

const DETECT_MS_URL: &str = "http://127.0.0.1:3002";
/// Spesifikasyona göre analiz penceresi (N=200 bar).
const CANDLE_LIMIT: usize = 200;
/// Değerlendirme periyodu (her saniye).
const EVAL_MS: u64 = 1_000;
/// Mum + seviye önbellek tazeleme periyodu (Binance yükünü düşük tutar).
const CACHE_REFRESH_SEC: u64 = 10;

struct Config {
    symbol: String,
    interval: String,
    once: bool,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn load_config() -> Config {
    let args: Vec<String> = env::args().collect();
    Config {
        symbol: env_or("BREAKOUT_SYMBOL", "VELVETUSDT"),
        interval: env_or("BREAKOUT_INTERVAL", "1m"),
        once: args.iter().any(|a| a == "--once"),
    }
}

// ── HTTP yardımcıları ────────────────────────────────────────
async fn http_get(client: &reqwest::Client, url: &str) -> Value {
    match client.get(url).send().await {
        Ok(r) => r.json::<Value>().await.unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

/// detect-ms'ten seviye raporunu çeker; en iyi DİRENÇ (SH) ve DESTEK (SL) fiyatını döner.
async fn fetch_levels(client: &reqwest::Client, cfg: &Config) -> (f64, f64) {
    let url = format!(
        "{DETECT_MS_URL}/api/ms?symbol={}&interval={}&limit={}",
        cfg.symbol, cfg.interval, CANDLE_LIMIT
    );
    let data = http_get(client, &url).await;
    let levels = match data.get("levels").and_then(|l| l.as_array()) {
        Some(l) => l,
        None => return (0.0, 0.0),
    };

    let mut best_sh = 0.0;
    let mut best_sh_score = f64::NEG_INFINITY;
    let mut best_sl = 0.0;
    let mut best_sl_score = f64::NEG_INFINITY;

    for l in levels {
        let Some(lt) = l.get("level_type").and_then(|x| x.as_str()) else { continue };
        let Some(price) = l.get("price").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok()) else { continue };
        let score = l.get("priority_score").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        match lt {
            "SH" if score > best_sh_score => { best_sh = price; best_sh_score = score; }
            "SL" if score > best_sl_score => { best_sl = price; best_sl_score = score; }
            _ => {}
        }
    }
    (best_sh, best_sl)
}

/// Seviyeye dokunuş sayısı (T_cnt) ve dokunuş anlarındaki ortalama hacim (V_touch_avg).
/// Dokunuş: mum seviyeyi 0.5·ATR bant içinde test ettiyse sayılır.
fn compute_touches(candles: &[ohlcv_engine::Kline], resistance: f64, support: f64, atr: f64) -> (u32, f64) {
    let band = 0.5 * atr;
    let mut count = 0u32;
    let mut vol_sum = 0.0;
    for c in candles {
        let high = c.high.to_f64().unwrap_or(0.0);
        let low = c.low.to_f64().unwrap_or(0.0);
        let vol = c.volume.to_f64().unwrap_or(0.0);
        let hit_r = resistance > 0.0 && (high - resistance).abs() < band;
        let hit_s = support > 0.0 && (low - support).abs() < band;
        if hit_r || hit_s {
            count += 1;
            vol_sum += vol;
        }
    }
    let v_avg = if count > 0 { vol_sum / count as f64 } else { 0.0 };
    (count, v_avg)
}

fn timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

#[tokio::main]
async fn main() {
    let cfg = load_config();
    println!("══════════════════════════════════════════════════");
    println!("  🎯 BREAKOUT — TEK ZAMAN DİLİMİ KIRILIM TESPİTİ  ({} {})", cfg.symbol, cfg.interval);
    println!("  Pencere: {CANDLE_LIMIT} mum | Güncelleme: her {} sn | Veri: klines + detect-ms + flow ring'leri (RAM)", EVAL_MS / 1000);
    println!("  📡 MOD: Sinyal üretici (sembol + yön + Q/F/C, emir AÇILMAZ)");
    println!("══════════════════════════════════════════════════");

    let http = reqwest::Client::new();
    let klines = BinanceClient::new();
    let mut feed = Feed::new(&cfg.symbol);

    // Önbellek: mumlar + seviyeler (CACHE_REFRESH_SEC'te bir tazelenir).
    let mut cached_candles: Vec<ohlcv_engine::Kline> = Vec::new();
    let mut cached_r = 0.0;
    let mut cached_s = 0.0;
    let mut last_refresh = std::time::Instant::now() - Duration::from_secs(CACHE_REFRESH_SEC + 1);
    let mut last_cache_err = std::time::Instant::now() - Duration::from_secs(60);

    loop {
        // 1) Önbellek tazele (10 sn): mumlar + detect-ms seviyeleri.
        if last_refresh.elapsed().as_secs() >= CACHE_REFRESH_SEC {
            match klines.fetch_klines(&cfg.symbol, &cfg.interval, CANDLE_LIMIT).await {
                Ok(c) if !c.is_empty() => {
                    cached_candles = c;
                    if cached_candles.is_empty() {
                        eprintln!("[{}] ⚠️ klines boş ({} {})", timestamp(), cfg.symbol, cfg.interval);
                    }
                }
                Ok(_) => {
                    if last_cache_err.elapsed().as_secs() >= 30 {
                        eprintln!("[{}] ⚠️ klines boş — yeniden deneniyor ({} {})", timestamp(), cfg.symbol, cfg.interval);
                        last_cache_err = std::time::Instant::now();
                    }
                }
                Err(e) => {
                    if last_cache_err.elapsed().as_secs() >= 30 {
                        eprintln!("[{}] ⚠️ klines hatası: {e} — eski mumlarla devam ({} {})", timestamp(), cfg.symbol, cfg.interval);
                        last_cache_err = std::time::Instant::now();
                    }
                }
            }
            let (r, s) = fetch_levels(&http, &cfg).await;
            cached_r = r;
            cached_s = s;
            last_refresh = std::time::Instant::now();
        }

        if cached_candles.is_empty() {
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        }

        // 2) Göstergeler (önbellekli mumlar).
        let atr = indicators::atr(&cached_candles, 14);
        let vols: Vec<f64> = cached_candles.iter().map(|c| c.volume.to_f64().unwrap_or(0.0)).collect();
        let v_avg = indicators::sma(&vols, 20);
        let (high_14, low_14) = indicators::high_low(&cached_candles, 14);
        let Some((p_open, p_high, p_low, p_close, vol_current)) = indicators::last_candle(&cached_candles) else {
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        };
        let (touches, v_touch_avg) = compute_touches(&cached_candles, cached_r, cached_s, atr);

        // 3) Flow ring'leri (her saniye taze).
        let snap = feed.poll();

        // 4) Algoritma.
        let input = BreakoutInput {
            symbol: cfg.symbol.clone(),
            p_high,
            p_low,
            p_open,
            p_close,
            atr,
            v_avg,
            volume_current: vol_current,
            high_14,
            low_14,
            resistance: cached_r,
            support: cached_s,
            touches,
            v_touch_avg,
            oi: snap.oi,
            oi_prev: snap.oi_prev,
            funding_rate: snap.funding_rate,
            funding_mean_20: snap.funding_mean_20,
            funding_std_20: snap.funding_std_20,
            cvd_now: snap.cvd_now,
            cvd_prev_10: snap.cvd_prev_10,
            cvd_sigma: snap.cvd_sigma,
            liq_current: snap.liq_current,
            liq_avg: snap.liq_avg,
            mark: snap.mark,
            last: snap.last,
        };
        let r = breakout::compute(&input);

        println!("[{}] {}", timestamp(), serde_json::to_string(&r.to_json()).unwrap_or_default());
        let signal = match r.direction {
            "UP" => "📈 BUY",
            "DOWN" => "📉 SELL",
            _ => "· NÖTR",
        };
        println!(
            "  {signal} | seviye: {} | Q=%{:.1} F=%{:.1} C=%{:.1} | ATR={:.4} T_cnt={} R={} S={}",
            r.broken_level, r.quality, r.fake, r.certainty, atr, touches, cached_r, cached_s
        );

        if cfg.once {
            return;
        }
        tokio::time::sleep(Duration::from_millis(EVAL_MS)).await;
    }
}
