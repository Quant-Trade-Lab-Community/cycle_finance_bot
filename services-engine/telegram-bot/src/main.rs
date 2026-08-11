//! 🤖 telegram-bot — kırılım stratejisi sinyallerini Telegram'a bildirim olarak iletir.
//!
//! `strategies-engine` her sinyal değişiminde `/tmp/strategy_signals.jsonl`'e
//! bir JSONL satırı ekler. Bu servis dosyayı izler ve her yeni sinyali
//! Telegram Bot API ile ilgili sohbete gönderir.
//!
//! Gereksinimler:
//!   TELEGRAM_BOT_TOKEN  — @BotFather'dan alınır
//!   TELEGRAM_CHAT_ID    — sinyallerin gideceği sohbet/kanal ID'si
//!   STRATEGY_SIGNALS_FILE (opsiyonel) — sinyal dosyası (varsayılan /tmp/strategy_signals.jsonl)
//!
//! cycle-engine dayanıklılık deseni: tek örnek koruması uygular.

use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

const SIGNALS_DEFAULT: &str = "/tmp/strategy_signals.jsonl";

fn main() {
    let _ = infra::util::single_instance("telegram-bot");

    let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
    let path = std::env::var("STRATEGY_SIGNALS_FILE").unwrap_or_else(|_| SIGNALS_DEFAULT.to_string());

    if token.is_empty() || chat_id.is_empty() {
        eprintln!("❌ TELEGRAM_BOT_TOKEN ve TELEGRAM_CHAT_ID gerekli (.env'e ekle)");
        std::process::exit(1);
    }

    println!("🤖 Telegram bot başlatıldı — sinyal akışı: {path}");
    let _ = send(&token, &chat_id, "🤖 Cycle Finance sinyal botu çalışıyor — kırılım sinyallerini bildirecek.");

    // Dosyanın mevcut sonundan başla (eski sinyalleri yeniden gönderme).
    let mut last_offset = current_len(&path);

    loop {
        if let Some(new_lines) = read_new_lines(&path, &mut last_offset) {
            for line in new_lines {
                let text = format_signal(&line);
                println!("📨 {text}");
                let _ = send(&token, &chat_id, &text);
            }
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
