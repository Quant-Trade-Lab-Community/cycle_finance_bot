use rusqlite::{Connection, params};
use flume::Receiver;
use std::time::{Instant, Duration};
use rust_decimal::prelude::*;
use contracts::events::{OwnedEvent, EventType};

pub fn start_db_writer(rx: Receiver<OwnedEvent>) {
    std::fs::create_dir_all("data-engine/data").ok();
    // Open or create SQLite DB
    let mut conn = Connection::open("data-engine/data/market_data.db").expect("Failed to open SQLite database");
    
    // Optimize SQLite for high throughput (WAL mode, synchronous=NORMAL)
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;"
    ).expect("Failed to set PRAGMAs");
    
    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS orderbooks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            bids TEXT NOT NULL,
            asks TEXT NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS liquidations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side INTEGER NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS funding_rates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            mark_price REAL NOT NULL,
            index_price REAL NOT NULL DEFAULT 0,
            funding_rate REAL NOT NULL,
            next_funding_time INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS booktickers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            best_bid_price REAL NOT NULL,
            best_bid_qty REAL NOT NULL,
            best_ask_price REAL NOT NULL,
            best_ask_qty REAL NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS open_interests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            open_interest REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS opportunities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            score REAL NOT NULL,
            efficiency REAL NOT NULL,
            price_bps_per_s REAL NOT NULL,
            price_ticks_per_s REAL NOT NULL,
            ob_changes_per_s REAL NOT NULL,
            spread_bps REAL NOT NULL,
            verdict INTEGER NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS symbol_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            score REAL NOT NULL,
            efficiency REAL NOT NULL,
            price_bps_per_s REAL NOT NULL,
            price_ticks_per_s REAL NOT NULL,
            ob_changes_per_s REAL NOT NULL,
            spread_bps REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    let mut batch_count = 0;
    let mut last_commit = Instant::now();
    let batch_size_limit = 10_000;
    let commit_interval = Duration::from_millis(1000);

    let mut tx = conn.transaction().expect("Failed to begin transaction");

    while let Ok(event) = rx.recv() {
        let symbol_len = event.symbol.iter().position(|&c| c == 0).unwrap_or(16);
        let symbol_str = std::str::from_utf8(&event.symbol[..symbol_len]).unwrap_or("UNKNOWN");

        match &event.payload {
            EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
                tx.execute(
                    "INSERT INTO trades (symbol, price, quantity, timestamp) VALUES (?1, ?2, ?3, ?4)",
                    params![symbol_str, price.to_f64().unwrap_or(0.0), quantity.to_f64().unwrap_or(0.0), timestamp],
                ).expect("Failed to insert trade");
            },
            EventType::Orderbook { bids, asks } => {
                use std::fmt::Write;
                let mut bids_str = String::with_capacity(512);
                for (p, q) in bids.iter() {
                    if *p == rust_decimal::Decimal::ZERO && *q == rust_decimal::Decimal::ZERO { continue; }
                    let _ = write!(&mut bids_str, "{},{}|", p, q);
                }
                
                let mut asks_str = String::with_capacity(512);
                for (p, q) in asks.iter() {
                    if *p == rust_decimal::Decimal::ZERO && *q == rust_decimal::Decimal::ZERO { continue; }
                    let _ = write!(&mut asks_str, "{},{}|", p, q);
                }

                tx.execute(
                    "INSERT INTO orderbooks (symbol, bids, asks) VALUES (?1, ?2, ?3)",
                    params![symbol_str, bids_str, asks_str],
                ).expect("Failed to insert orderbook");
            },
            EventType::Liquidation { side, price, quantity, timestamp } => {
                tx.execute(
                    "INSERT INTO liquidations (symbol, side, price, quantity, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![symbol_str, side, price.to_f64().unwrap_or(0.0), quantity.to_f64().unwrap_or(0.0), timestamp],
                ).expect("Failed to insert liquidation");
            },
            EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time } => {
                tx.execute(
                    "INSERT INTO funding_rates (symbol, mark_price, index_price, funding_rate, next_funding_time) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![symbol_str, mark_price.to_f64().unwrap_or(0.0), index_price.to_f64().unwrap_or(0.0), funding_rate.to_f64().unwrap_or(0.0), next_funding_time],
                ).expect("Failed to insert funding rate");
            },
            EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty } => {
                tx.execute(
                    "INSERT INTO booktickers (symbol, best_bid_price, best_bid_qty, best_ask_price, best_ask_qty) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![symbol_str, best_bid_price.to_f64().unwrap_or(0.0), best_bid_qty.to_f64().unwrap_or(0.0), best_ask_price.to_f64().unwrap_or(0.0), best_ask_qty.to_f64().unwrap_or(0.0)],
                ).expect("Failed to insert bookticker");
            },
            EventType::OpenInterest { open_interest, timestamp } => {
                tx.execute(
                    "INSERT INTO open_interests (symbol, open_interest, timestamp) VALUES (?1, ?2, ?3)",
                    params![symbol_str, open_interest.to_f64().unwrap_or(0.0), timestamp],
                ).expect("Failed to insert open interest");
            }
            EventType::Opportunity { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict } => {
                tx.execute(
                    "INSERT INTO opportunities (symbol, score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%s','now'))",
                    params![symbol_str, score.to_f64().unwrap_or(0.0), efficiency.to_f64().unwrap_or(0.0), price_bps_per_s.to_f64().unwrap_or(0.0), price_ticks_per_s.to_f64().unwrap_or(0.0), ob_changes_per_s.to_f64().unwrap_or(0.0), spread_bps.to_f64().unwrap_or(0.0), verdict],
                ).expect("Failed to insert opportunity");
            }
            EventType::SymbolMetrics { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps } => {
                tx.execute(
                    "INSERT INTO symbol_metrics (symbol, score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%s','now'))",
                    params![symbol_str, score.to_f64().unwrap_or(0.0), efficiency.to_f64().unwrap_or(0.0), price_bps_per_s.to_f64().unwrap_or(0.0), price_ticks_per_s.to_f64().unwrap_or(0.0), ob_changes_per_s.to_f64().unwrap_or(0.0), spread_bps.to_f64().unwrap_or(0.0)],
                ).expect("Failed to insert symbol metrics");
            }
        }

        batch_count += 1;

        if batch_count >= batch_size_limit || last_commit.elapsed() >= commit_interval {
            tx.commit().expect("Failed to commit transaction");
            tx = conn.transaction().expect("Failed to begin transaction");
            batch_count = 0;
            last_commit = Instant::now();
        }
    }
}
