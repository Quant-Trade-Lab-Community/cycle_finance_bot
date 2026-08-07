//! LISTENER — data merkezinden gelen verilerle anlık metrik analizi (Rust).
//!
//! Pozisyon izleyici DEĞİLDİR. Data merkezi (price-feed :3004) üzerinden
//! sistemde tanımlı HER sembol için anlık fiyat verilerini (last/mark/index/
//! bid/ask) çeker ve metrik hesaplar.
//!
//! Metrikler ŞU AN BOŞ (placeholder) — gerçek metrikler sonra eklenecek.
//! Çıktılar: konsol tablosu + /tmp/listener_metrics.json

use serde_json::Value;
use std::env;
use std::time::Duration;

const PRICE_FEED_URL: &str = "http://127.0.0.1:3004";
const OUT_FILE: &str = "/tmp/listener_metrics.json";

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

async fn http_json(client: &reqwest::Client, url: &str) -> Value {
    match client.get(url).send().await {
        Ok(r) => r.json::<Value>().await.unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn fmt_val(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Null) | None => "—".to_string(),
        _ => "—".to_string(),
    }
}

/// ANLIK METRİK ANALİZİ — ŞU AN BOŞ, metrikler sonra eklenecek.
/// Her sembol için hesaplanacak örnek metrikler:
///   - spread_pct, momentum, distance_to_vwap, liquidity_score, ...
fn compute_metrics(symbol: &str, price: &Value) -> Value {
    let last = price.get("last").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let mark = price.get("mark").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let index = price.get("index").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let bid = price.get("bid").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let ask = price.get("ask").and_then(|v| v.as_f64()).unwrap_or(0.0);

    // ── METRİK ŞABLONU (doldurulacak) ─────────────────────
    serde_json::json!({
        "symbol": symbol,
        "placeholder": true,
        "spread_pct": if ask > bid && bid > 0.0 { Some((ask - bid) / bid * 100.0) } else { None },
        "last": last,
        "mark": mark,
        "index": index,
        "bid": bid,
        "ask": ask,
    })
}

#[tokio::main]
async fn main() {
    let refresh: u64 = env_or("LISTENER_REFRESH_SEC", "2").parse().unwrap_or(2);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("reqwest client");

    println!("{}", "═".repeat(72));
    println!("  🛰️  LISTENER — ANLIK METRİK ANALİZİ (data merkezi: price-feed)");
    println!("{}", "═".repeat(72));

    loop {
        let data = http_json(&client, &format!("{PRICE_FEED_URL}/api/lastprice")).await;

        print!("\x1b[2J\x1b[H");
        println!("{}", "═".repeat(72));
        println!("  🛰️  LISTENER — DATA MERKEZİ ANLIK METRİKLERİ");
        println!("  Kaynak: {PRICE_FEED_URL}  |  Yenileme: {refresh}s");
        println!("{}", "═".repeat(72));

        if data.get("error").is_some() {
            println!("  ⚠️  Data merkezi erişilemiyor: {}", data.get("error").unwrap());
            tokio::time::sleep(Duration::from_secs(refresh)).await;
            continue;
        }

        let prices = data.get("prices").and_then(|p| p.as_object()).cloned().unwrap_or_default();
        let symbols = data.get("symbols").and_then(|s| s.as_array()).cloned().unwrap_or_default();

        if prices.is_empty() {
            println!("  📭 VERİ YOK — price-feed çalışıyor mu? (pricefeed-start)");
        } else {
            println!("  {:<10}{:<14}{:<14}{:<14}{:<14}{:<14}{:<14}", "SEMBOL", "LAST", "MARK", "INDEX", "BID", "ASK", "METRİK");
            println!("  {}", "-".repeat(72));
            for s in &symbols {
                let sym = fmt_val(Some(s));
                if let Some(p) = prices.get(sym.as_str()) {
                    let last = fmt_val(p.get("last"));
                    let mark = fmt_val(p.get("mark"));
                    let index = fmt_val(p.get("index"));
                    let bid = fmt_val(p.get("bid"));
                    let ask = fmt_val(p.get("ask"));
                    let m = compute_metrics(&sym, p);
                    let mtext = if m.get("placeholder").and_then(|x| x.as_bool()).unwrap_or(false) {
                        "⏳ analiz bekliyor".to_string()
                    } else {
                        "—".to_string()
                    };
                    println!("  {:<10}{:<14}{:<14}{:<14}{:<14}{:<14}{:<14}", sym, last, mark, index, bid, ask, mtext);
                }
            }
        }
        println!("{}", "-".repeat(70));
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("  Son güncelleme: {now}  (Ctrl+C ile çık)");

        // ── Metrik çıktısı (JSON) ──
        let metrics: serde_json::Map<String, Value> = prices
            .iter()
            .map(|(k, v)| (k.clone(), compute_metrics(k, v)))
            .collect();
        let doc = serde_json::json!({
            "timestamp": now,
            "symbols": symbols,
            "metrics": metrics,
        });
        let _ = std::fs::write(OUT_FILE, serde_json::to_string_pretty(&doc).unwrap_or_default());

        tokio::time::sleep(Duration::from_secs(refresh)).await;
    }
}
