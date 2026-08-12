//! 🤖 telegram-bot — kırılım stratejisi sinyallerini ve alert-service uyarılarını Telegram'a iletir.
//!
//! `strategies-engine` her sinyal değişiminde `/tmp/strategy_signals.jsonl`'e bir JSONL satırı
//! ekler; `alert-service` ise her tetiklenen uyarıyı `/tmp/alert_events.jsonl`'e yazar.
//! Bu servis iki dosyayı da izler ve yeni satırları Telegram Bot API ile ilgili sohbete gönderir.
//! Ayrıca her `DETECT_MS_PERIOD_SEC` (varsayılan 300 sn) aralığında detect-ms'ten MSMP raporu
//! çekip gönderir.
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
//! cycle-engine dayanıklılık deseni: tek örnek koruması uygular.

use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

const SIGNALS_DEFAULT: &str = "/tmp/strategy_signals.jsonl";
const ALERTS_DEFAULT: &str = "/tmp/alert_events.jsonl";
const DETECT_MS_DEFAULT: &str = "http://127.0.0.1:3002";

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

    println!("🤖 Telegram bot başlatıldı — sinyal akışı: {path} | uyarı akışı: {alert_path}");
    println!("📊 Periyodik detect-ms raporu: her {detect_period}sn → {detect_url} ({detect_symbol} {detect_interval})");
    let _ = send(&token, &chat_id, "🤖 Cycle Finance sinyal botu çalışıyor — kırılım sinyallerini ve uyarıları bildirecek.");

    // Dosyaların mevcut sonundan başla (eski satırları yeniden gönderme).
    let mut last_offset = current_len(&path);
    let mut last_alert_offset = current_len(&alert_path);
    let mut last_detect = std::time::Instant::now();

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
