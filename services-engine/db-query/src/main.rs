//! db-query — TimescaleDB'ye yazılan verileri sorgulayan terminal servisi.
//!
//! Kullanım:
//!   db-query                     → canlı panel (her 3 sn'de bir yenilenir)
//!   db-query --once              → tek seferlik panel
//!   db-query --tables            → yalnızca tablo satır sayıları
//!   db-query --recent <tablo> <sembol> [limit]   → son kayıtlar
//!   db-query --symbols           → sembolleri listele
//!
//! Bağlantı: `TIMESCALEDB_URL` (varsayılan postgres://cycle:cycle@localhost:5432/market_data)

use rust_decimal::prelude::ToPrimitive;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Column, Row};
use std::env;
use std::time::Duration;

const DEFAULT_URL: &str = "postgres://cycle:cycle@localhost:5432/market_data";
const TABLES: &[&str] = &[
    "trades",
    "orderbooks",
    "liquidations",
    "open_interests",
    "funding_rates",
    "markprices",
    "lastprices",
    "indexprices",
];

fn db_url() -> String {
    env::var("TIMESCALEDB_URL").unwrap_or_else(|_| DEFAULT_URL.to_string())
}

fn symbols() -> Vec<String> {
    if let Ok(v) = env::var("CYCLE_FLOW_SYMBOLS") {
        let s: Vec<String> = v.split(',').map(|x| x.trim().to_uppercase()).filter(|x| !x.is_empty()).collect();
        if !s.is_empty() {
            return s;
        }
    }
    vec!["BTCUSDT".into(), "ETHUSDT".into(), "SOLUSDT".into(), "VELVETUSDT".into()]
}

async fn count_table(pool: &PgPool, table: &str) -> i64 {
    sqlx::query(&format!("SELECT count(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .map(|r| r.get::<i64, _>(0))
        .unwrap_or(-1)
}

async fn latest_f64(pool: &PgPool, table: &str, column: &str, symbol: &str) -> Option<f64> {
    sqlx::query(&format!(
        "SELECT {column} FROM {table} WHERE symbol = $1 ORDER BY timestamp DESC LIMIT 1"
    ))
    .bind(symbol)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .and_then(|r| r.try_get::<f64, _>(0).ok())
}

async fn trade_stats(pool: &PgPool, symbol: &str, window_ms: i64) -> (i64, f64) {
    let since = now_ms() - window_ms;
    sqlx::query(
        "SELECT count(*), COALESCE(sum(quantity),0) FROM trades WHERE symbol=$1 AND timestamp > $2",
    )
    .bind(symbol)
    .bind(since)
    .fetch_one(pool)
    .await
    .map(|r| (r.get::<i64, _>(0), r.get::<f64, _>(1)))
    .unwrap_or((0, 0.0))
}

async fn print_dashboard(pool: &PgPool) {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    print!("\x1b[2J\x1b[H");
    println!("══════════════════════════════════════════════════════════");
    println!("  🛢️  DB-QUERY — TimescaleDB  (market_data)   [{now}]");
    println!("  Veritabanı: {} ({} MB veri)", db_url(), "");
    println!("══════════════════════════════════════════════════════════");

    println!("\n  📊 Tablo Satır Sayıları");
    for t in TABLES {
        let c = count_table(pool, t).await;
        let c_str = if c < 0 { "?".to_string() } else { c.to_string() };
        println!("    {:<16} {:>10} {}", t, c_str, if c < 0 { "ERR" } else { "" });
    }

    println!("\n  📈 Sembol Durumu (son değerler + son 60 sn işlem)");
    println!("    {:<11}{:>14}{:>14}{:>14}{:>12}{:>12}{:>10}", "SEMBOL", "LAST", "MARK", "INDEX", "FUNDING", "OPEN_INT", "TRADE/sn");
    for s in symbols() {
        let last = latest_f64(pool, "lastprices", "price", &s).await.unwrap_or(0.0);
        let mark = latest_f64(pool, "markprices", "price", &s).await.unwrap_or(0.0);
        let index = latest_f64(pool, "indexprices", "price", &s).await.unwrap_or(0.0);
        let funding = latest_f64(pool, "funding_rates", "funding_rate", &s).await.unwrap_or(0.0);
        let oi = latest_f64(pool, "open_interests", "open_interest", &s).await.unwrap_or(0.0);
        let (cnt, _vol) = trade_stats(pool, &s, 60_000).await;
        println!(
            "    {:<11}{:>14.6}{:>14.6}{:>14.6}{:>12.8}{:>12.0}{:>10}",
            s, last, mark, index, funding, oi, cnt
        );
    }

    println!("\n  💥 Son Likidasyonlar (varsa)");
    let liq = latest_liquidations(pool, 5).await;
    if liq.is_empty() {
        println!("    (bu ağda likidasyon WS'i iletilmiyor — tablo boş)");
    } else {
        for l in liq {
            println!("    {l}");
        }
    }

    println!("\n  Komutlar: --recent <tablo> <sembol> [limit]  |  --tables  |  --once");
}

async fn latest_liquidations(pool: &PgPool, limit: i64) -> Vec<String> {
    sqlx::query("SELECT symbol, side, price, quantity, timestamp FROM liquidations ORDER BY timestamp DESC LIMIT $1")
        .bind(limit)
        .fetch_all(pool)
        .await
        .map(|rows| {
            rows.iter()
                .map(|r| {
                    let sym: String = r.get("symbol");
                    let side: i16 = r.get("side");
                    let price: f64 = r.get("price");
                    let qty: f64 = r.get("quantity");
                    let ts: i64 = r.get("timestamp");
                    format!("  {sym} {} @ {price:.6} qty={qty:.4} ts={}", if side == 0 { "BUY " } else { "SELL" }, ts)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `--recent` için tablo → sütun eşlemesi (orderbooks JSONB text'e çevrilir).
fn recent_columns(table: &str) -> &'static str {
    match table {
        "trades" => "symbol, price, quantity, timestamp",
        "orderbooks" => "symbol, bids::text AS bids, asks::text AS asks, timestamp",
        "funding_rates" => "symbol, mark_price, index_price, funding_rate, next_funding_time, timestamp",
        "markprices" | "lastprices" | "indexprices" => "symbol, price, timestamp",
        "open_interests" => "symbol, open_interest, timestamp",
        "liquidations" => "symbol, side, price, quantity, timestamp",
        _ => "symbol, timestamp",
    }
}

async fn recent_rows(pool: &PgPool, table: &str, symbol: &str, limit: i64) -> Result<(), sqlx::Error> {
    let cols = recent_columns(table);
    let sql = format!("SELECT {cols} FROM {table} WHERE symbol=$1 ORDER BY timestamp DESC LIMIT $2");
    let rows = sqlx::query(&sql).bind(symbol).bind(limit).fetch_all(pool).await?;
    println!("{} satır — {table} / {symbol}:", rows.len());
    for row in rows {
        let mut obj = serde_json::Map::new();
        for col in row.columns() {
            let name = col.name().to_string();
            let n = name.as_str();
            let v = if let Ok(x) = row.try_get::<f64, _>(n) {
                serde_json::json!(x)
            } else if let Ok(x) = row.try_get::<i64, _>(n) {
                serde_json::json!(x)
            } else if let Ok(x) = row.try_get::<i16, _>(n) {
                serde_json::json!(x)
            } else if let Ok(x) = row.try_get::<bool, _>(n) {
                serde_json::json!(x)
            } else if let Ok(x) = row.try_get::<rust_decimal::Decimal, _>(n) {
                serde_json::json!(x.to_f64().unwrap_or(0.0))
            } else if let Ok(x) = row.try_get::<String, _>(n) {
                serde_json::json!(x)
            } else {
                serde_json::Value::Null
            };
            obj.insert(name, v);
        }
        println!("  {}", serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default());
    }
    Ok(())
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let url = db_url();

    let pool = loop {
        match PgPoolOptions::new().max_connections(4).connect(&url).await {
            Ok(p) => break p,
            Err(e) => {
                eprintln!("⚠️ DB bağlantı hatası: {e} — 2 sn sonra yeniden deneniyor");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    };
    println!("🛢️ DB-QUERY bağlandı: {url}");

    if let Some(pos) = args.iter().position(|a| a == "--recent") {
        let table = args.get(pos + 1).map(String::as_str).unwrap_or("trades");
        let symbol = args.get(pos + 2).map(|s| s.to_uppercase()).unwrap_or_else(|| "BTCUSDT".into());
        let limit: i64 = args.get(pos + 3).and_then(|s| s.parse().ok()).unwrap_or(10);
        match recent_rows(&pool, table, &symbol, limit).await {
            Ok(()) => return,
            Err(e) => {
                eprintln!("Sorgu hatası: {e}");
                std::process::exit(1);
            }
        }
    }

    if args.iter().any(|a| a == "--tables") {
        println!("Tablo satır sayıları:");
        for t in TABLES {
            println!("  {:<16} {}", t, count_table(&pool, t).await);
        }
        return;
    }

    if args.iter().any(|a| a == "--symbols") {
        println!("Semboller: {}", symbols().join(", "));
        return;
    }

    let once = args.iter().any(|a| a == "--once");
    loop {
        print_dashboard(&pool).await;
        if once {
            break;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
