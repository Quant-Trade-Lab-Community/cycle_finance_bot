//! 🤖 telegram-bot — kırılım stratejisi sinyallerini, alert-service uyarılarını ve
//! canlı Binance hesap/pozisyon bilgisini Telegram'a iletir.
//!
//! `strategies-engine` her sinyal değişiminde `/tmp/strategy_signals.jsonl`'e bir JSONL satırı
//! ekler; `alert-service` ise her tetiklenen uyarıyı `/tmp/alert_events.jsonl`'e yazar.
//! Bu servis iki dosyayı da izler ve yeni satırları Telegram Bot API ile ilgili sohbete gönderir.
//! Ayrıca her `DETECT_MS_PERIOD_SEC` (varsayılan 300 sn) aralığında detect-ms'ten MSMP raporu,
//! her `EXEC_PERIOD_SEC` (varsayılan 300 sn) aralığında executiond'den canlı Binance
//! hesap (balance) ve pozisyon bilgisini çekip gönderir.
//!
//! Gereksinimler:
//!   TELEGRAM_BOT_TOKEN  — @BotFather'dan alınır
//!   TELEGRAM_CHAT_ID    — sinyallerin gideceği sohbet/kanal ID'si
//!   STRATEGY_SIGNALS_FILE (opsiyonel) — sinyal dosyası (varsayılan /tmp/strategy_signals.jsonl)
//!   ALERT_EVENTS_FILE    (opsiyonel) — uyarı dosyası (varsayılan /tmp/alert_events.jsonl)
//!
//! Periyodik detect-ms raporu:
//!   DETECT_MS_URL       (opsiyonel) — varsayılan http://127.0.0.1:3002
//!   DETECT_MS_SYMBOL    (opsiyonel) — varsayılan VELVETUSDT
//!   DETECT_MS_INTERVAL  (opsiyonel) — varsayılan 1m
//!   DETECT_MS_PERIOD_SEC (opsiyonel) — rapor aralığı (sn), varsayılan 300
//!
//! Periyodik executiond (canlı Binance) raporu:
//!   EXEC_API_URL        (opsiyonel) — varsayılan http://127.0.0.1:3010
//!   EXEC_ADMIN_USER     (opsiyonel) — varsayılan admin
//!   EXEC_ADMIN_PASS     (opsiyonel) — varsayılan changeme123
//!   EXEC_PERIOD_SEC     (opsiyonel) — rapor aralığı (sn), varsayılan 300
//!
//! cycle-engine dayanıklılık deseni: tek örnek koruması uygular.

use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

const SIGNALS_DEFAULT: &str = "/tmp/strategy_signals.jsonl";
const ALERTS_DEFAULT: &str = "/tmp/alert_events.jsonl";
const DETECT_MS_DEFAULT: &str = "http://127.0.0.1:3002";
const EXEC_API_DEFAULT: &str = "http://127.0.0.1:3010";

fn main() {
    let _ = infra::util::single_instance("telegram-bot");

    let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
    let path = std::env::var("STRATEGY_SIGNALS_FILE").unwrap_or_else(|_| SIGNALS_DEFAULT.to_string());
    let alert_path =
        std::env::var("ALERT_EVENTS_FILE").unwrap_or_else(|_| ALERTS_DEFAULT.to_string());

    if token.is_empty() || chat_id.is_empty() {
        eprintln!("❌ TELEGRAM_BOT_TOKEN ve TELEGRAM_CHAT_ID gerekli (.env'e ekle)");
        std::process::exit(1);
    }

    let detect_url = std::env::var("DETECT_MS_URL").unwrap_or_else(|_| DETECT_MS_DEFAULT.to_string());
    let detect_symbol = std::env::var("DETECT_MS_SYMBOL").unwrap_or_else(|_| "VELVETUSDT".to_string());
    let detect_interval = std::env::var("DETECT_MS_INTERVAL").unwrap_or_else(|_| "1m".to_string());
    let detect_period: u64 = std::env::var("DETECT_MS_PERIOD_SEC")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    let exec_url = std::env::var("EXEC_API_URL").unwrap_or_else(|_| EXEC_API_DEFAULT.to_string());
    let exec_user = std::env::var("EXEC_ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
    let exec_pass = std::env::var("EXEC_ADMIN_PASS").unwrap_or_else(|_| "changeme123".to_string());
    let exec_period: u64 = std::env::var("EXEC_PERIOD_SEC")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    println!("🤖 Telegram bot başlatıldı — sinyal akışı: {path} | uyarı akışı: {alert_path}");
    println!("📊 Periyodik detect-ms raporu: her {detect_period}sn → {detect_url} ({detect_symbol} {detect_interval})");
    println!("💰 Periyodik exec (canlı Binance) raporu: her {exec_period}sn → {exec_url}");
    let _ = send(&token, &chat_id, "🤖 Cycle Finance sinyal botu çalışıyor — sinyalleri, uyarıları ve hesap bilgilerini bildirecek.");

    // Dosyaların mevcut sonundan başla (eski satırları yeniden gönderme).
    let mut last_offset = current_len(&path);
    let mut last_alert_offset = current_len(&alert_path);
    let mut last_detect = std::time::Instant::now();
    let mut last_exec = std::time::Instant::now();

    loop {
        if let Some(new_lines) = read_new_lines(&path, &mut last_offset) {
            for line in new_lines {
                let text = format_signal(&line);
                println!("📨 {text}");
                let _ = send(&token, &chat_id, &text);
            }
        }

        if let Some(new_lines) = read_new_lines(&alert_path, &mut last_alert_offset) {
            for line in new_lines {
                let text = format_alert(&line);
                println!("🔔 {text}");
                let _ = send(&token, &chat_id, &text);
            }
        }

        if last_detect.elapsed().as_secs() >= detect_period {
            let text = fetch_detect_ms(&detect_url, &detect_symbol, &detect_interval);
            println!("📊 {text}");
            let _ = send(&token, &chat_id, &text);
            last_detect = std::time::Instant::now();
        }

        if last_exec.elapsed().as_secs() >= exec_period {
            let text = fetch_exec_report(&exec_url, &exec_user, &exec_pass);
            println!("💰 {text}");
            let _ = send(&token, &chat_id, &text);
            last_exec = std::time::Instant::now();
        }

        std::thread::sleep(Duration::from_millis(1000));
    }
}

fn current_len(path: &str) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Dosyanın `last_offset` sonrasındaki yeni satırlarını okur; offset'i günceller.
fn read_new_lines(path: &str, last_offset: &mut u64) -> Option<Vec<String>> {
    let mut file = std::fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len <= *last_offset {
        return Some(Vec::new());
    }
    file.seek(SeekFrom::Start(*last_offset)).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    *last_offset = len;
    Some(buf.lines().map(str::to_string).collect())
}

/// Sinyal JSON'undan okunabilir bir Telegram mesajı üretir.
fn format_signal(line: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(line.trim())
        .unwrap_or_else(|_| serde_json::json!({ "raw": line.trim() }));
    let sym = v["symbol"].as_str().unwrap_or("?");
    let dir = v["direction"].as_str().unwrap_or("NONE");
    let level = v["broken_level"].as_f64().unwrap_or(0.0);
    let q = v["quality"].as_f64().unwrap_or(0.0);
    let f = v["fake"].as_f64().unwrap_or(0.0);
    let c = v["certainty"].as_f64().unwrap_or(0.0);
    let ts = v["ts"].as_str().unwrap_or("");
    let arrow = match dir {
        "UP" => "📈 BUY",
        "DOWN" => "📉 SELL",
        _ => "·",
    };
    format!(
        "🚀 BREAKOUT SİNYALİ — {sym}\n{arrow} ({dir})\nSeviye: {level}\nKalite: %{q:.1} | Sahte: %{f:.1} | Kesinlik: %{c:.1}\n⏰ {ts}"
    )
}

/// alert-service uyarı JSON satırından okunabilir bir Telegram mesajı üretir.
fn format_alert(line: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(line.trim())
        .unwrap_or_else(|_| serde_json::json!({ "raw": line.trim() }));
    let sym = v["symbol"].as_str().unwrap_or("?");
    let cond = v["condition"].as_str().unwrap_or("?");
    let price = v["price"].as_f64()
        .or_else(|| v["price"].as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0);
    let voice = v["voice"].as_str().unwrap_or("").to_string();
    let ts = fmt_time_ms(v["ts"].as_u64().unwrap_or(0));

    let cond_label = match cond {
        "above" => "ÜSTÜNE ÇIKTI ⬆️",
        "below" => "ALTINA İNDİ ⬇️",
        "cross" => "GEÇTİ ↔️",
        "touch" => "DEĞDİ 👆",
        other => other,
    };

    let mut out = format!("🔔 ALERT — {sym}\nFiyat {cond_label}: {price}\n⏰ {ts}");
    if !voice.is_empty() {
        out.push_str(&format!("\n🗣️ {voice}"));
    }
    out
}

/// Unix ms'yi (UTC) HH:MM:SS'ye çevirir (chrono bağımlılığı olmadan).
fn fmt_time_ms(ms: u64) -> String {
    let secs = ms / 1000;
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

/// detect-ms API'sinden MSMP raporunu çeker ve Telegram mesajı formatında döndürür.
fn fetch_detect_ms(url: &str, symbol: &str, interval: &str) -> String {
    let api = format!("{url}/api/ms?symbol={symbol}&interval={interval}&limit=200");
    let client = reqwest::blocking::Client::new();
    let resp = match client.get(&api).send() {
        Ok(r) => r,
        Err(e) => return format!("⚠️ detect-ms erişilemedi: {e}"),
    };
    let mut v: serde_json::Value = match resp.json() {
        Ok(v) => v,
        Err(_) => return "⚠️ detect-ms yanıtı ayrıştırılamadı".to_string(),
    };
    if let Some(err) = v.get("error") {
        return format!("⚠️ detect-ms: {err}");
    }
    // Yanıt sembol/interval içermez — mesaja yazmak için enjekte et.
    v["symbol"] = serde_json::Value::String(symbol.to_uppercase());
    v["interval"] = serde_json::Value::String(interval.to_string());
    format_detect_ms(&v)
}

/// JSON değerini f64'e çevirir (rust_decimal string olarak serileşir).
fn as_f64(v: &serde_json::Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

/// 8 ondalığa yuvarlayarak kayan nokta gürültüsünü temizler.
fn fmt(v: &serde_json::Value) -> String {
    let f = as_f64(v);
    let r = (f * 1e8).round() / 1e8;
    format!("{r}")
}

/// detect-ms `/api/ms` yanıtından okunabilir bir Telegram raporu üretir.
fn format_detect_ms(v: &serde_json::Value) -> String {
    let sym = v["symbol"].as_str().unwrap_or("?");
    let interval = v["interval"].as_str().unwrap_or("?");
    let price = fmt(&v["current_price"]);
    let ats = as_f64(&v["ats"]);
    let trend = v["trend_label"].as_str().unwrap_or("?");
    let hurst = fmt(&v["hurst"]);
    let rsq = fmt(&v["r_squared"]);
    let conf = as_f64(&v["confluence_index"]);
    let vwap = fmt(&v["vwap"]);
    let poc = fmt(&v["poc"]);
    let bsl = fmt(&v["bsl_ssl_ratio"]);
    let atr = fmt(&v["atr"]);
    let fvg = v["fvg_count"].as_u64().unwrap_or(0);
    let absorber = v["active_absorber_count"].as_u64().unwrap_or(0);
    let liq = v["liquidity_zones_count"].as_u64().unwrap_or(0);

    let vac = v.get("vacuum_zone");
    let vac_str = match vac {
        Some(z) if !z.is_null() => format!(
            "{} — {}..{} (skor {})",
            z["label"].as_str().unwrap_or("?"),
            fmt(&z["price_low"]),
            fmt(&z["price_high"]),
            fmt(&z["magnetic_score"]),
        ),
        _ => "yok".to_string(),
    };

    let levels = match v["levels"].as_array() {
        Some(arr) => arr
            .iter()
            .take(5)
            .map(|l| {
                format!(
                    "  • {} @ {} (savunma {})",
                    l["level_type"].as_str().unwrap_or("?"),
                    fmt(&l["price"]),
                    l["defense_count"].as_u64().unwrap_or(0),
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        None => String::new(),
    };

    let mut out = format!(
        "📊 DETECT-MS RAPORU — {sym} ({interval})\n\
         Fiyat: {price}\n\
         Trend: {trend} (ATS {ats:+.2}) · Confluence %{conf:.0}\n\
         Hurst: {hurst} · R²: {rsq} · ATR: {atr}\n\
         VWAP: {vwap} · POC: {poc} · BSL/SSL: {bsl}\n\
         Vakum: {vac_str}\n\
         FVG: {fvg} · Absorber: {absorber} · Likidite bölgesi: {liq}"
    );
    if !levels.is_empty() {
        out.push_str("\nSeviyeler:\n");
        out.push_str(&levels);
    }
    out
}

/// executiond'ye giriş yapar, access_token döndürür (başarısızsa None).
fn exec_login(base: &str, user: &str, pass: &str) -> Option<String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{base}/api/v1/auth/login"))
        .json(&serde_json::json!({ "username": user, "password": pass }))
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().ok()?;
    v["access_token"].as_str().map(|s| s.to_string())
}

/// JWT ile korumalı executiond endpoint'inden JSON döndürür.
fn exec_get(base: &str, token: &str, path: &str) -> Option<serde_json::Value> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{base}{path}"))
        .bearer_auth(token)
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json().ok()
}

/// executiond `/api/v1/account` yanıtından hesap (balance) raporu üretir.
fn format_exec_account(v: &serde_json::Value) -> String {
    let acc = &v["account"];
    let wallet = fmt(&acc["total_wallet_balance"]);
    let avail = fmt(&acc["available_balance"]);
    let margin = fmt(&acc["total_margin_balance"]);
    let upnl = fmt(&acc["total_unrealized_profit"]);
    let withdraw = fmt(&acc["max_withdraw_amount"]);
    let can_trade = acc["can_trade"].as_bool().unwrap_or(false);
    format!(
        "💰 BİNANCE HESAP (canlı)\nBakiye: {wallet} USDT\nSerbest: {avail}\nMarjin Bakiye: {margin}\nGerçekleşmemiş PnL: {upnl}\nÇekilebilir: {withdraw}\nTicaret: {}",
        if can_trade { "AÇIK" } else { "KAPALI" }
    )
}

/// executiond `/api/v1/positions` yanıtından (açık pozisyon) raporu üretir.
fn format_exec_positions(v: &serde_json::Value) -> String {
    let arr = match v.as_array() {
        Some(a) => a,
        None => return "📍 POZİSYONLAR\n(Açık pozisyon yok)".to_string(),
    };
    let open: Vec<_> = arr
        .iter()
        .filter(|p| as_f64(&p["position_amt"]) != 0.0)
        .collect();
    if open.is_empty() {
        return "📍 POZİSYONLAR\n(Açık pozisyon yok)".to_string();
    }
    let mut out = format!("📍 AÇIK POZİSYONLAR ({})", open.len());
    for p in open.iter().take(10) {
        let sym = p["symbol"].as_str().unwrap_or("?");
        let side = p["position_side"].as_str().unwrap_or("?");
        let amt = fmt(&p["position_amt"]);
        let entry = fmt(&p["entry_price"]);
        let mark = fmt(&p["mark_price"]);
        let pnl = fmt(&p["un_realized_profit"]);
        let lev = fmt(&p["leverage"]);
        out.push_str(&format!("\n{sym} {side} {amt} @ {entry} | Mark {mark} | PnL {pnl} | {lev}x"));
    }
    out
}

/// executiond'den canlı hesap + pozisyon raporunu çekip birleşik mesaj üretir.
fn fetch_exec_report(base: &str, user: &str, pass: &str) -> String {
    let Some(token) = exec_login(base, user, pass) else {
        return "⚠️ executiond erişilemedi (login başarısız) — executiond çalışıyor mu?".to_string();
    };
    let mut parts = Vec::new();
    match exec_get(base, &token, "/api/v1/account") {
        Some(v) => parts.push(format_exec_account(&v)),
        None => parts.push("⚠️ Hesap bilgisi alınamadı".to_string()),
    }
    match exec_get(base, &token, "/api/v1/positions") {
        Some(v) => parts.push(format_exec_positions(&v)),
        None => parts.push("⚠️ Pozisyon bilgisi alınamadı".to_string()),
    }
    parts.join("\n\n")
}

/// Telegram Bot API'ye mesaj gönderir.
fn send(token: &str, chat_id: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let client = reqwest::blocking::Client::new();
    client
        .post(&url)
        .form(&[("chat_id", chat_id), ("text", text)])
        .send()?;
    Ok(())
}
