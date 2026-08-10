//! BREAKOUT Kırılım Stratejisi (Rust) — Event-Driven Sürüm
//!
//! Mimari (Katman 5: Strateji): **Actor + olay güdümlü**. Eski sürüm 20 dakikada
//! bir REST polling ile uyanıyordu; bu sürüm fiyatı price-feed ring'inden
//! **event-by-event** alır, değerlendirmeyi bekleme aralığında otomatik daya
//! (varsayılan 20 dakika, `/tmp/breakout_wait_sec.txt` ile dinamik).
//!
//! **Sinyal üretici mod**: Emir AÇMAZ. Sadece kırılım algılandığında
//! sembol + yön (BUY/SELL) bilgisini üretir.
//!
//! Akış:
//! ```text
//! price-feed ring (/cycle_finance_pricefeed)
//!   → ring okuyucu std thread (fiyat event'leri)
//!   → mpsc UnboundedChannel → [actor döngüsü]
//!                                ├─ fiyat anlık güncel (bekleme aralığında bile)
//!                                └─ bekleme aralığı dolmuşsa değerlendirme:
//!                                   detect-ms (:3002) → kırılım → sinyal (sembol+yön)
//! ```

use transport::events::{EventType, OwnedEvent};
use transport::wire;
use rust_decimal::prelude::*;
use serde_json::Value;
use std::env;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use transport::ring_buffer::GenerationalRingBuffer;

const DETECT_MS_URL: &str = "http://127.0.0.1:3002";
const PRICE_FEED_URL: &str = "http://127.0.0.1:3004";
const WAIT_FILE: &str = "/tmp/breakout_wait_sec.txt";
/// Ring'de yeni event yoksa uyanma sınırı — döngü asla tamamen uykuda kalmaz.
const WAKE_INTERVAL: Duration = Duration::from_millis(500);

struct Config {
    symbol: String,
    interval: String,
    limit: usize,
    wait_sec: u64,
    once: bool,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn load_config() -> Config {
    let check_every: usize = env_or("BREAKOUT_CHECK_EVERY", "20").parse().unwrap_or(20);
    let wait_sec: u64 = env_or("BREAKOUT_WAIT_SEC", &(check_every * 60).to_string())
        .parse()
        .unwrap_or((check_every * 60) as u64);
    let args: Vec<String> = env::args().collect();
    Config {
        symbol: env_or("BREAKOUT_SYMBOL", "HEIUSDT"),
        interval: env_or("BREAKOUT_INTERVAL", "1m"),
        limit: env_or("BREAKOUT_LIMIT", "100").parse().unwrap_or(100),
        wait_sec,
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

async fn fetch_analysis(client: &reqwest::Client, cfg: &Config) -> Value {
    let url = format!(
        "{DETECT_MS_URL}/api/ms?symbol={}&interval={}&limit={}",
        cfg.symbol, cfg.interval, cfg.limit
    );
    http_get(client, &url).await
}

async fn fetch_price_feed(client: &reqwest::Client, cfg: &Config) -> (Option<f64>, Option<String>) {
    let url = format!("{PRICE_FEED_URL}/api/lastprice/{}", cfg.symbol);
    let v = http_get(client, &url).await;
    if v.get("error").is_some() {
        return (None, v.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()));
    }
    if let Some(p) = v.pointer("/price") {
        for key in ["last", "mark", "index", "ask"] {
            if let Some(f) = p.get(key).and_then(|x| x.as_f64()) {
                if f > 0.0 {
                    return (Some(f), None);
                }
            }
        }
    }
    (None, Some("price-feed'te fiyat yok".to_string()))
}

// ── Seviye seçimi ────────────────────────────────────────────
fn best_level(levels: &[Value], level_type: &str) -> Option<(f64, f64)> {
    levels
        .iter()
        .filter(|l| l.get("level_type").and_then(|x| x.as_str()) == Some(level_type))
        .filter_map(|l| {
            let price = l.get("price").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok())?;
            let score = l.get("priority_score").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            Some((price, score))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}

// ── Kırılım değerlendirme (saf fonksiyon — test edilebilir) ──
fn evaluate(data: &Value, price: f64) -> (Option<String>, String) {
    if data.get("error").is_some() {
        return (None, format!("detect-ms hatası: {}", data.get("error").unwrap()));
    }
    let levels = match data.get("levels").and_then(|l| l.as_array()) {
        Some(l) if !l.is_empty() => l,
        _ => return (None, "Seviye yok".to_string()),
    };

    let ats: f64 = data.get("ats").and_then(|a| a.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let trend = data.get("trend_label").and_then(|t| t.as_str()).unwrap_or("");
    let confluence = data.get("confluence_index").and_then(|c| c.as_str()).unwrap_or("");
    let log = format!("Fiyat={price:.6}  ATS={ats:.4}  Trend={trend}  Confluence=%{confluence}");

    if ats > 0.0 {
        match best_level(levels, "SH") {
            Some((lv, score)) => {
                if price > lv {
                    (Some("BUY".into()), format!("{log} | 🎯 DİRENC KIRILDI SH={lv} (skor:{score}) → BUY"))
                } else {
                    (None, format!("{log} | Direnc yukarı kırılmadı SH={lv}"))
                }
            }
            None => (None, format!("{log} | Direnc yok")),
        }
    } else if ats < 0.0 {
        match best_level(levels, "SL") {
            Some((lv, score)) => {
                if price < lv {
                    (Some("SELL".into()), format!("{log} | 🎯 DESTEK KIRILDI SL={lv} (skor:{score}) → SELL"))
                } else {
                    (None, format!("{log} | Destek aşağı kırılmadı SL={lv}"))
                }
            }
            None => (None, format!("{log} | Destek yok")),
        }
    } else {
        (None, format!("{log} | Nötr trend"))
    }
}

// ── Bekleme süresi (dinamik) ─────────────────────────────────
fn current_wait_sec(default: u64) -> u64 {
    if let Ok(content) = std::fs::read_to_string(WAIT_FILE) {
        if let Ok(v) = content.trim().parse::<u64>() {
            if v > 0 {
                return v;
            }
        }
    }
    default
}

// ── Ring okuyucu (Katman 2 trans sözleşmesi) ─────────────────
/// Price-feed ring'indeki ilgili sembolün fiyat event'lerini kanala basar.
fn spawn_price_reader(symbol: &str, tx: mpsc::UnboundedSender<f64>) {
    let symbol = symbol.to_ascii_uppercase();
    std::thread::spawn(move || {
        let gen_ring = GenerationalRingBuffer::with_name("/cycle_finance_pricefeed", 20_000);
        let mut cursor = gen_ring.get_head();
        let mut symbol_buf = [0u8; 16];
        let bytes = symbol.as_bytes();
        let len = bytes.len().min(16);
        symbol_buf[..len].copy_from_slice(&bytes[..len]);

        loop {
            match gen_ring.read_slot(cursor) {
                Some(slot) => {
                    if let Some(ev) = wire::decode(&slot.data[..slot.len as usize]) {
                        if ev.symbol == symbol_buf {
                            if let Some(price) = event_price(&ev) {
                                if tx.send(price).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    cursor += 1;
                }
                None => {
                    let head = gen_ring.get_head();
                    if head > cursor {
                        cursor = head; // üretici arayı kapattı
                    } else {
                        std::thread::sleep(std::time::Duration::from_micros(500));
                    }
                }
            }
        }
    });
}

/// Event'ten stratejinin kullanacağı tek fiyatı çıkarır (bridge ile aynı öncelik).
fn event_price(ev: &OwnedEvent) -> Option<f64> {
    match &ev.payload {
        EventType::Trade { price, .. } => price.to_f64(),
        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
            let ask = best_ask_price.to_f64()?;
            if ask > 0.0 {
                Some(ask)
            } else {
                let bid = best_bid_price.to_f64()?;
                (bid > 0.0).then_some(bid)
            }
        }
        EventType::FundingRate { mark_price, .. } => mark_price.to_f64(),
        _ => None,
    }
}

// ── Tek değerlendirme ────────────────────────────────────────
struct EvalOutcome {
    ok: bool,
    msg: String,
}

async fn analyze_once(client: &reqwest::Client, cfg: &Config, price_override: Option<f64>) -> EvalOutcome {
    let data = fetch_analysis(client, cfg).await;
    if data.get("error").is_some() {
        let e = data.get("error").unwrap();
        return EvalOutcome { ok: false, msg: format!("⚠️ detect-ms erişilemiyor: {e}") };
    }

    let (pf_price, pf_err) = fetch_price_feed(client, cfg).await;
    let price = price_override
        .filter(|p| *p > 0.0)
        .or(pf_price)
        .unwrap_or_else(|| {
            data.get("current_price").and_then(|c| c.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)
        });
    let (signal, msg) = evaluate(&data, price);
    let feed_tag = if price_override.is_some() { "ring" } else if pf_err.is_none() { "REST" } else { "detect-ms" };

    let Some(side) = signal else {
        return EvalOutcome { ok: true, msg: format!("{msg}") };
    };

    EvalOutcome {
        ok: true,
        msg: format!("📡 SİNYAL → Sembol: {} | Yön: {} (fiyat: {feed_tag}) | {msg}", cfg.symbol, side),
    }
}

#[tokio::main]
async fn main() {
    let cfg = load_config();
    println!("══════════════════════════════════════════════════");
    println!("  🎯 BREAKOUT KIRILIM STRATEJİSİ — EVENT-DRIVEN  ({} {})", cfg.symbol, cfg.interval);
    println!("  Pencere: {} | Bekleme: {} sn | Kaynak: price-feed ring", cfg.limit, cfg.wait_sec);
    println!("  detect-ms: {DETECT_MS_URL}");
    println!("  📡 MOD: Sinyal üretici (sembol + yön, emir AÇILMAZ)");
    println!("══════════════════════════════════════════════════");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client");

    if cfg.once {
        let r = analyze_once(&client, &cfg, None).await;
        println!("[{}] {}", timestamp(), r.msg);
        return;
    }

    // Event-driven döngü: fiyat anlık (ring), değerlendirme bekleme aralığında.
    let (tx, mut rx) = mpsc::unbounded_channel::<f64>();
    spawn_price_reader(&cfg.symbol, tx);

    let mut latest_price: Option<f64> = None;
    let mut last_eval = Instant::now() - Duration::from_secs(cfg.wait_sec);
    let mut startup = true;

    loop {
        let evt = tokio::time::timeout(WAKE_INTERVAL, rx.recv()).await;
        if let Ok(Some(p)) = evt {
            latest_price = Some(p);
        }

        let sec = current_wait_sec(cfg.wait_sec);
        if startup || last_eval.elapsed().as_secs() >= sec {
            last_eval = Instant::now();
            startup = false;

            let r = analyze_once(&client, &cfg, latest_price).await;
            println!("[{}] {}", timestamp(), r.msg);
            if !r.ok {
                println!("  🔄 10 sn sonra yeniden deneniyor...");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
            println!("  😴 {sec} sn ({:.1} dk) bekleniyor... (breakout-wait ile değişir)\n", sec as f64 / 60.0);
        }
    }
}

fn timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}
