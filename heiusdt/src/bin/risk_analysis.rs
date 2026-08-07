//! Risk analizi (Rust) — Python karşılığı: scripts/risk_analysis.py
//!
//! market_data.db'deki trades tablosunu SQL ile özetler, hacim ve
//! volatilite riski hesaplar.

use rusqlite::Connection;

#[derive(Debug)]
struct Row {
    symbol: String,
    count: i64,
    volume: f64,
    min: f64,
    max: f64,
}

fn main() {
    println!("Veritabanı taranıyor... İstatistiksel risk hesaplanıyor...\n");

    let conn = match Connection::open("market_data.db") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Veritabanı açılamadı: {e}");
            std::process::exit(1);
        }
    };

    let query = "
        SELECT symbol, COUNT(*) as cnt,
               SUM(price * quantity) as volume,
               MIN(price) as min_p,
               MAX(price) as max_p
        FROM trades
        GROUP BY symbol
        HAVING cnt > 50
        ORDER BY volume DESC
    ";

    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(_) => {
            println!("Yeterli veri bulunamadı.");
            return;
        }
    };

    let rows: Vec<Row> = match stmt.query_map([], |r| {
        Ok(Row {
            symbol: r.get(0)?,
            count: r.get(1)?,
            volume: r.get(2)?,
            min: r.get(3)?,
            max: r.get(4)?,
        })
    }) {
        Ok(iter) => iter.filter_map(|x| x.ok()).collect(),
        Err(_) => vec![],
    };

    if rows.is_empty() {
        println!("Yeterli veri bulunamadı.");
        return;
    }

    let rows: Vec<(Row, f64)> = rows
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

    let mut sorted: Vec<&(Row, f64)> = rows.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\n=== ⚠️ EN YÜKSEK RİSK / VOLATİLİTE İÇEREN 10 PARİTE ===");
    println!("  {:<10}{:<12}{:<18}{:<16}", "PARİTE", "İŞLEM", "VOLATİLİTE_%", "HACİM_USDT");
    for (r, vol) in sorted.iter().take(10) {
        println!("  {:<10}{:<12}{:<18.2}{:<16.2}", r.symbol, r.count, vol, r.volume);
    }
}
