//! LISTENER — açık pozisyonlar için anlık metrik analizi (Rust).
//! Python karşılığı: scripts/listener.py
//!
//! Paper-service'ten pozisyonları + health çeker, tablo çizer ve
//! /tmp/listener_metrics.json'a yazar. Metrikler şu an placeholder.

use serde_json::Value;
use std::env;
use std::time::Duration;

const PAPER_API: &str = "http://127.0.0.1:8080";
const OUT_FILE: &str = "/tmp/listener_metrics.json";

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

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

async fn http_post_json(client: &reqwest::Client, url: &str, body: &Value) -> Value {
    match client.post(url).json(body).send().await {
        Ok(r) => r.json::<Value>().await.unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

async fn login(client: &reqwest::Client, user: &str, pass: &str) -> Option<String> {
    let body = serde_json::json!({"username": user, "password": pass});
    let v = http_post_json(client, &format!("{PAPER_API}/api/v1/auth/login"), &body).await;
    v.get("access_token").and_then(|t| t.as_str()).map(|s| s.to_string())
}

fn fmt_val(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => "—".to_string(),
    }
}

#[tokio::main]
async fn main() {
    let user = env_or("PAPER_ADMIN_USER", "admin");
    let pass = env_or("PAPER_ADMIN_PASS", "changeme123");
    let refresh: u64 = env_or("LISTENER_REFRESH_SEC", "2").parse().unwrap_or(2);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("reqwest client");

    println!("{}", "═".repeat(60));
    println!("  🛰️  LISTENER KATMANI — ANLIK POZİSYON METRİKLERİ");
    println!("{}", "═".repeat(60));

    loop {
        let token = match login(&client, &user, &pass).await {
            Some(t) => t,
            None => {
                println!("⚠️  Paper giriş başarısız — yeniden deneniyor...");
                tokio::time::sleep(Duration::from_secs(refresh)).await;
                continue;
            }
        };

        let health = http_json(&client, &format!("{PAPER_API}/api/v1/system/health"), Some(&token)).await;
        let pos = http_json(&client, &format!("{PAPER_API}/api/v1/account/positions"), Some(&token)).await;

        // ── Ekranı temizle ve çiz ──
        print!("\x1b[2J\x1b[H");
        println!("{}", "═".repeat(60));
        println!("  🛰️  LISTENER — ANLIK POZİSYON METRİKLERİ");
        println!("  Paper: {PAPER_API}  |  Yenileme: {refresh}s");
        println!("{}", "═".repeat(60));

        if !health.is_null() {
            let last = fmt_val(health.get("last_price").unwrap_or(&Value::Null));
            let status = fmt_val(health.get("status").unwrap_or(&Value::Null));
            println!("  Veri Merkezi: {status}  |  Son Fiyat: {last}");
        } else {
            println!("  ⚠️  Veri Merkezi: {health}");
        }

        println!("{}", "-".repeat(60));
        let positions = pos.get("positions").and_then(|p| p.as_array()).cloned().unwrap_or_default();
        if positions.is_empty() {
            println!("  📭 AÇIK POZİSYON YOK");
        } else {
            println!("  {:<12}{:<8}{:<10}{:<14}{:<14}{}", "SEMBOL", "YÖN", "MİKTAR", "GİRİŞ", "MARK", "METRİK");
            println!("  {}", "-".repeat(56));
            for p in &positions {
                let sym = fmt_val(p.get("symbol").unwrap_or(&Value::Null));
                let side = fmt_val(p.get("side").unwrap_or(&Value::Null));
                let qty = fmt_val(p.get("quantity").unwrap_or(&Value::Null));
                let entry = fmt_val(p.get("avg_entry_price").unwrap_or(&Value::Null));
                let mark = fmt_val(p.get("mark_price").unwrap_or(&Value::Null));
                println!("  {:<12}{:<8}{:<10}{:<14}{:<14}⏳ analiz bekliyor", sym, side, qty, entry, mark);
            }
        }
        println!("{}", "-".repeat(60));
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("  Son güncelleme: {now}  (Ctrl+C ile çık)");

        // ── JSON çıktısı ──
        let doc = serde_json::json!({
            "timestamp": now,
            "positions": positions,
            "metrics": {},
        });
        let _ = std::fs::write(OUT_FILE, serde_json::to_string_pretty(&doc).unwrap_or_default());

        tokio::time::sleep(Duration::from_secs(refresh)).await;
    }
}
