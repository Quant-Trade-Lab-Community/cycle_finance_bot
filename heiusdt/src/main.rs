//! HEIUSDT Kırılım Stratejisi (Rust) — Python karşılığı strategies/heiusdt_breakout.py
//!
//! 1. detect-ms'ten HEIUSDT 1m, 100 pencere analizi alır.
//! 2. ATS > 0 ise en yüksek skorlu direnç (SH) kırılımı → BUY.
//! 3. ATS < 0 ise en yüksek skorlu destek (SL) kırılımı → SELL.
//! 4. Koşul sağlanırsa paper-service'e market emri açar.
//!
//! Bekleme süresi: /tmp/heiusdt_wait_sec.txt (heiusdt-wait <sn> ile değiştirilir).

use serde_json::Value;
use std::env;
use std::time::Duration;

const DETECT_MS_URL: &str = "http://127.0.0.1:3002";
const PRICE_FEED_URL: &str = "http://127.0.0.1:3004";
const PAPER_API: &str = "http://127.0.0.1:8080";
const WAIT_FILE: &str = "/tmp/heiusdt_wait_sec.txt";

struct Config {
    symbol: String,
    interval: String,
    limit: usize,
    qty: String,
    wait_sec: u64,
    paper_user: String,
    paper_pass: String,
    dry_run: bool,
    once: bool,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn load_config() -> Config {
    let check_every: usize = env_or("HEIUSDT_CHECK_EVERY", "20").parse().unwrap_or(20);
    let wait_sec: u64 = env_or("HEIUSDT_WAIT_SEC", &(check_every * 60).to_string())
        .parse()
        .unwrap_or((check_every * 60) as u64);
    let args: Vec<String> = env::args().collect();
    Config {
        symbol: env_or("HEIUSDT_SYMBOL", "HEIUSDT"),
        interval: env_or("HEIUSDT_INTERVAL", "1m"),
        limit: env_or("HEIUSDT_LIMIT", "100").parse().unwrap_or(100),
        qty: env_or("HEIUSDT_QTY", "1000"),
        wait_sec,
        paper_user: env_or("PAPER_ADMIN_USER", "admin"),
        paper_pass: env_or("PAPER_ADMIN_PASS", "changeme123"),
        dry_run: args.iter().any(|a| a == "--dry-run"),
        once: args.iter().any(|a| a == "--once"),
    }
}

// ── HTTP yardımcıları ────────────────────────────────────────
async fn http_json(client: &reqwest::Client, url: &str, token: Option<&str>) -> Value {
    let mut req = client.get(url);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    match req.send().await {
        Ok(r) => r.json::<Value>().await.unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

async fn http_post_json(client: &reqwest::Client, url: &str, token: Option<&str>, body: &Value) -> Value {
    let mut req = client.post(url).json(body);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    match req.send().await {
        Ok(r) => r.json::<Value>().await.unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

async fn login(client: &reqwest::Client, cfg: &Config) -> Option<String> {
    let body = serde_json::json!({
        "username": cfg.paper_user,
        "password": cfg.paper_pass,
    });
    let v = http_post_json(client, &format!("{PAPER_API}/api/v1/auth/login"), None, &body).await;
    v.get("access_token").and_then(|t| t.as_str()).map(|s| s.to_string())
}

async fn get_positions(client: &reqwest::Client, token: &str) -> Value {
    http_json(client, &format!("{PAPER_API}/api/v1/account/positions"), Some(token)).await
}

async fn place_order(client: &reqwest::Client, cfg: &Config, token: &str, side: &str) -> Value {
    let oid = format!(
        "heiusdt-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let body = serde_json::json!({
        "client_order_id": oid,
        "symbol": cfg.symbol,
        "side": side,
        "order_type": "MARKET",
        "quantity": cfg.qty,
    });
    http_post_json(client, &format!("{PAPER_API}/api/v1/order"), Some(token), &body).await
}

async fn fetch_analysis(client: &reqwest::Client, cfg: &Config) -> Value {
    let url = format!(
        "{DETECT_MS_URL}/api/ms?symbol={}&interval={}&limit={}",
        cfg.symbol, cfg.interval, cfg.limit
    );
    http_json(client, &url, None).await
}

async fn fetch_price_feed(client: &reqwest::Client, cfg: &Config) -> (Option<f64>, Option<String>) {
    let url = format!("{PRICE_FEED_URL}/api/lastprice/{}", cfg.symbol);
    let v = http_json(client, &url, None).await;
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

// ── Kırılım değerlendirme ────────────────────────────────────
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
                    (Some("BUY".into()), format!("{log} | 🎯 DİRENÇ KIRILDI SH={lv} (skor:{score}) → BUY"))
                } else {
                    (None, format!("{log} | Direnç yukarı kırılmadı SH={lv}"))
                }
            }
            None => (None, format!("{log} | Direnç yok")),
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

// ── Bekleme süresi ───────────────────────────────────────────
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

fn timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

// ── Ana akış ─────────────────────────────────────────────────
async fn analyze_once(client: &reqwest::Client, cfg: &Config) -> bool {
    let token = match login(client, cfg).await {
        Some(t) => t,
        None => {
            println!("❌ Paper giriş başarısız");
            return false;
        }
    };

    let data = fetch_analysis(client, cfg).await;
    if data.get("error").is_some() {
        let e = data.get("error").unwrap();
        println!("[{}] ⚠️  detect-ms erişilemiyor: {e}", timestamp());
        println!("   → 10 sn sonra yeniden denenecek...");
        return false;
    }

    let (pf_price, pf_err) = fetch_price_feed(client, cfg).await;
    let price = pf_price.unwrap_or_else(|| {
        data.get("current_price").and_then(|c| c.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)
    });
    let (signal, msg) = evaluate(&data, price);

    println!("[{}] {} {} {} pencere", timestamp(), cfg.symbol, cfg.interval, cfg.limit);
    match pf_price {
        Some(p) => println!("  💹 price-feed: {p}"),
        None => {
            if let Some(e) = pf_err {
                println!("  ⚠️  price-feed: {e} (detect-ms fiyatı kullanıldı)");
            }
        }
    }
    println!("  {msg}");

    let Some(side) = signal else {
        return true;
    };

    // Aynı sembolde pozisyon varsa tekrar açma
    let pos = get_positions(client, &token).await;
    if let Some(list) = pos.get("positions").and_then(|p| p.as_array()) {
        for p in list {
            if p.get("symbol").and_then(|s| s.as_str()) == Some(cfg.symbol.as_str())
                && p.get("quantity").and_then(|q| q.as_f64()).unwrap_or(0.0) != 0.0
            {
                println!("  ⏭️  {} pozisyonu zaten var. Yeni emir açılmadı.", cfg.symbol);
                return true;
            }
        }
    }

    if cfg.dry_run {
        println!("  🧪 [DRY-RUN] {side} emri gönderilmedi (QTY={})", cfg.qty);
        return true;
    }

    let resp = place_order(client, cfg, &token, &side).await;
    if let Some(oid) = resp.get("order_id").and_then(|o| o.as_str()) {
        println!("  ✅ {side} emri açıldı → id={oid} avg={}", resp.get("avg_price").unwrap());
    } else {
        println!("  ❌ Emir reddedildi: {resp}");
    }
    true
}

#[tokio::main]
async fn main() {
    let cfg = load_config();
    println!("══════════════════════════════════════════════════");
    println!("  🎯 HEIUSDT KIRILIM STRATEJİSİ  ({} {})", cfg.symbol, cfg.interval);
    println!("  Pencere: {} | Bekleme: {} sn", cfg.limit, cfg.wait_sec);
    println!("  Paper: {PAPER_API} | detect-ms: {DETECT_MS_URL}");
    if cfg.dry_run {
        println!("  🧪 MOD: DRY-RUN (emir gönderilmez)");
    }
    println!("══════════════════════════════════════════════════");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client");

    if cfg.once {
        let _ = analyze_once(&client, &cfg).await;
        return;
    }

    loop {
        let ok = analyze_once(&client, &cfg).await;
        if !ok {
            println!("  🔄 10 sn sonra yeniden deneniyor... {}\n", timestamp());
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        }
        let sec = current_wait_sec(cfg.wait_sec);
        println!("  😴 {sec} saniye ({:.1} dk) bekleniyor... (heiusdt-wait ile değiştir) {}\n",
                 sec as f64 / 60.0, timestamp());
        tokio::time::sleep(Duration::from_secs(sec)).await;
    }
}
