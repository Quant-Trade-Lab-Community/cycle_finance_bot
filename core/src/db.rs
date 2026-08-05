use rusqlite::{Connection, params};
use flume::Receiver;
use std::time::{Instant, Duration};
use crate::ring_buffer::{OwnedEvent, EventType};

pub fn start_db_writer(rx: Receiver<OwnedEvent>) {
    // Open or create SQLite DB
    let mut conn = Connection::open("market_data.db").expect("Failed to open SQLite database");
    
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

    let mut batch_count = 0;
    let mut last_commit = Instant::now();
    let batch_size_limit = 10_000;
    let commit_interval = Duration::from_millis(1000);

    let mut tx = conn.transaction().expect("Failed to begin transaction");

    while let Ok(event) = rx.recv() {
        let symbol_len = event.symbol.iter().position(|&c| c == 0).unwrap_or(16);
        let symbol_str = std::str::from_utf8(&event.symbol[..symbol_len]).unwrap_or("UNKNOWN");

        match &event.payload {
            EventType::Trade { price, quantity, timestamp } => {
                tx.execute(
                    "INSERT INTO trades (symbol, price, quantity, timestamp) VALUES (?1, ?2, ?3, ?4)",
                    params![symbol_str, price, quantity, timestamp],
                ).expect("Failed to insert trade");
            },
            EventType::Orderbook { bids, asks } => {
                // For orderbooks, serialize the 20 levels into a compact String format
                use std::fmt::Write;
                let mut bids_str = String::with_capacity(512);
                for (p, q) in bids.iter() {
                    if *p == 0.0 && *q == 0.0 { continue; }
                    let _ = write!(&mut bids_str, "{},{}|", p, q);
                }
                
                let mut asks_str = String::with_capacity(512);
                for (p, q) in asks.iter() {
                    if *p == 0.0 && *q == 0.0 { continue; }
                    let _ = write!(&mut asks_str, "{},{}|", p, q);
                }

                tx.execute(
                    "INSERT INTO orderbooks (symbol, bids, asks) VALUES (?1, ?2, ?3)",
                    params![symbol_str, bids_str, asks_str],
                ).expect("Failed to insert orderbook");
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
