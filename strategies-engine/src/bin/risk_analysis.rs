//! Risk analizi (Rust) — TimescaleDB'deki trades tablosunu SQL ile özetler.
//!
//! --watch  : sabit ekranda her N sn'de yenilenir (tmux RISK paneli için).
//!           clear YAPILMAZ; imleç başa alınıp üzerine yazılır (titreşimsiz).
//! WATCH_SEC: yenileme süresi (varsayılan 5 sn).
//!
//! Bağlantı: `TIMESCALEDB_URL` (varsayılan postgres://cycle:cycle@localhost:5432/market_data)

use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::time::Duration;

#[derive(Debug)]
struct SymbolRow {
    symbol: String,
    count: i64,
    volume: f64,
    min: f64,
    max: f64,
}

fn db_url() -> String {
    std::env::var("TIMESCALEDB_URL")
        .unwrap_or_else(|_| "postgres://cycle:cycle@localhost:5432/market_data".into())
}

async fn render() {
    let pool = match PgPoolOptions::new().max_connections(2).connect(&db_url()).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Veritabanı açılamadı: {e}");
            return;
        }
    };

    let query = "
        SELECT symbol, COUNT(*) as cnt,
               SUM(price * quantity) as volume,
               MIN(price) as min_p,
               MAX(price) as max_p
        FROM trades
        GROUP BY symbol
        HAVING COUNT(*) > 50
        ORDER BY volume DESC
    ";

    let rows: Vec<SymbolRow> = match sqlx::query(query)
        .fetch_all(&pool)
        .await
    {
        Ok(rows) => rows
            .iter()
            .map(|r| SymbolRow {
                symbol: r.get("symbol"),
                count: r.get("cnt"),
                volume: r.get("volume"),
                min: r.get("min_p"),
                max: r.get("max_p"),
            })
            .collect(),
        Err(_) => {
            println!("Yeterli veri bulunamadı.");
            return;
        }
    };

    if rows.is_empty() {
        println!("Yeterli veri bulunamadı.");
        return;
    }

    let rows: Vec<(SymbolRow, f64)> = rows
        .into_iter()
        .map(|r| {
            let vol = if r.min > 0.0 { ((r.max - r.min) / r.min) * 100.0 } else { 0.0 };
            (r, vol)
        })
        .collect();

    println!("=== 📊 PİYASA HACİM VE RİSK DAĞILIMI (EN ÇOK İŞLEM GÖREN 15 PARİTE) ===");
    println!("  {:<10}{:<12}{:<16}{:<14}{:<14}{:<18}", "PARİTE", "İŞLEM", "HACİM_USDT", "MİN", "MAKS", "VOLATİLİTE_%");
    for (r, vol) in rows.iter().take(15) {
        println!(
            "  {:<10}{:<12}{:<16.2}{:<14.2}{:<14.2}{:<18.2}",
            r.symbol, r.count, r.volume, r.min, r.max, vol
        );
    }

    let mut sorted: Vec<&(SymbolRow, f64)> = rows.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\n=== ⚠️ EN YÜKSEK RİSK / VOLATİLİTE İÇEREN 10 PARİTE ===");
    println!("  {:<10}{:<12}{:<18}{:<16}", "PARİTE", "İŞLEM", "VOLATİLİTE_%", "HACİM_USDT");
    for (r, vol) in sorted.iter().take(10) {
        println!("  {:<10}{:<12}{:<18.2}{:<16.2}", r.symbol, r.count, vol, r.volume);
    }
}

#[tokio::main]
async fn main() {
    let watch = std::env::args().any(|a| a == "--watch");
    let watch_sec: u64 = std::env::var("WATCH_SEC")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    if !watch {
        render().await;
        return;
    }

    // Sabit ekran: ilk render tam boyutla çizilir; sonrakiler imleç başa alınır.
    print!("\x1b[2J\x1b[H"); // başta bir kez temizle
    render().await;
    loop {
        tokio::time::sleep(Duration::from_secs(watch_sec)).await;
        print!("\x1b[H"); // imleç en üste
        render().await;
    }
}
