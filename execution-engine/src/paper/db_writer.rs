use rusqlite::{Connection, Result};
use tokio::sync::mpsc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub enum PersistEvent {
    Trade {
        order_id: String,
        symbol: String,
        side: String,
        price: f64,
        quantity: f64,
        fee: f64,
        timestamp: u64,
    },
    // We can add OpenOrder events here in the future
}

pub async fn start_db_writer(mut rx: mpsc::UnboundedReceiver<PersistEvent>, db_path: String, batch_interval_ms: u64) {
    let mut conn = Connection::open(&db_path).expect("Failed to open paper db");
    
    // Enable WAL mode
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;"
    ).expect("Failed to configure WAL");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS paper_trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id TEXT NOT NULL,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            fee REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).expect("Failed to create paper_trades table");
    
    // Also paper_open_orders
    conn.execute(
        "CREATE TABLE IF NOT EXISTS paper_open_orders (
            order_id TEXT PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            open_quantity REAL NOT NULL,
            original_quantity REAL NOT NULL,
            locked_balances_json TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create paper_open_orders table");

    let mut batch_count = 0;
    
    println!("PaperEngine: DB Writer started at {}", db_path);

    loop {
        let mut events = Vec::new();
        
        // Wait for first event
        if let Some(ev) = rx.recv().await {
            events.push(ev);
            batch_count += 1;
            
            // Gather remaining events within the timeout window to batch them
            let timeout = sleep(Duration::from_millis(batch_interval_ms));
            tokio::pin!(timeout);
            
            loop {
                tokio::select! {
                    Ok(ev) = tokio::time::timeout(Duration::from_millis(1), rx.recv()) => {
                        if let Some(e) = ev {
                            events.push(e);
                            batch_count += 1;
                            if batch_count > 5000 { break; }
                        } else {
                            break;
                        }
                    }
                    _ = &mut timeout => {
                        break;
                    }
                }
            }
        } else {
            // Channel closed
            break;
        }
        
        // Write batch
        if !events.is_empty() {
            let tx = conn.transaction().expect("Failed to begin transaction");
            {
                let mut stmt_trade = tx.prepare_cached(
                    "INSERT INTO paper_trades (order_id, symbol, side, price, quantity, fee, timestamp)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
                ).unwrap();
                
                for ev in events {
                    match ev {
                        PersistEvent::Trade { order_id, symbol, side, price, quantity, fee, timestamp } => {
                            stmt_trade.execute(rusqlite::params![order_id, symbol, side, price, quantity, fee, timestamp]).ok();
                        }
                    }
                }
            }
            tx.commit().expect("Failed to commit batch");
            batch_count = 0;
        }
    }
}
