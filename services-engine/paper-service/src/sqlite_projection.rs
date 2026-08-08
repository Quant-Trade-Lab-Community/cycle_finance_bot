//! Tek DomainEvent kanalından beslenen SQLite projection.
//!
//! Actor artık özel bir "persist kanalı" tutmaz; tüm kalıcılık (Sled WAL,
//! PostgreSQL, SQLite) aynı `DomainEvent` akışından beslenir. Bu modül o
//! akıştaki OLASILIK event'lerini SQLite tablolarına (`paper_trades`,
//! `paper_open_orders`) işler.
//!
//! Yazma stratejisi: event'ler hafızada toplanır, `batch_interval_ms`'de (dolaylı
//! flush) veya `flush()` çağrısıyla tek transaction içinde commit edilir — start,
//! 5000 event/sn'ye kadar olan yüklerde disk IO'yu amorti eder.

use execution_engine::paper::domain_event::DomainEvent;
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// SQLite bağlantısını açar ve şemayı (WAL + tablolar) hazırlar.
pub fn open_connection(db_path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;

    let _ = conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE IF NOT EXISTS paper_trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id TEXT NOT NULL,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            fee REAL NOT NULL,
            timestamp INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS paper_open_orders (
            order_id TEXT PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL,
            open_quantity REAL NOT NULL,
            original_quantity REAL NOT NULL,
            locked_balances_json TEXT NOT NULL
         );",
    );
    Ok(conn)
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

/// `paper_open_orders` satırının hafızadaki hali (OrderCreated/Upsert kaynağı).
struct OpenRow {
    symbol: String,
    side: String,
    price: Option<f64>,
    open_qty: Decimal,
    original: Decimal,
}

#[derive(Default)]
pub struct SqliteProjection {
    opens: HashMap<String, OpenRow>,
    pending_trades: Vec<(String, String, String, Decimal, Decimal, Decimal, u64)>,
    /// Aynı loop içinde kaç event işlendi (eşik logu için).
    applied: u64,
}

impl SqliteProjection {
    pub fn new() -> Self {
        Self::default()
    }

    /// `DomainEvent` akışından tek event uygula (memory projection).
    pub fn apply(&mut self, ev: &DomainEvent) {
        match ev {
            DomainEvent::OrderCreated { order_id, symbol, side, qty, price, .. } => {
                self.opens.insert(
                    order_id.clone(),
                    OpenRow {
                        symbol: symbol.clone(),
                        side: side.clone(),
                        price: price.map(|p| p.to_f64().unwrap_or(0.0)),
                        open_qty: *qty,
                        original: *qty,
                    },
                );
            }
            DomainEvent::OrderFilled {
                order_id,
                symbol,
                side,
                fill_price,
                fill_qty,
                commission,
                ..
            } => {
                let ts = now_ms();
                if let Some(row) = self.opens.get_mut(order_id) {
                    row.open_qty -= *fill_qty;
                }
                self.pending_trades.push((
                    order_id.clone(),
                    symbol.clone(),
                    side.clone(),
                    *fill_price,
                    *fill_qty,
                    *commission,
                    ts,
                ));
            }
            DomainEvent::OrderCancelled { order_id, .. } => {
                if let Some(row) = self.opens.get_mut(order_id) {
                    row.open_qty = Decimal::ZERO;
                }
            }
            _ => {}
        }
        self.applied += 1;
    }

    /// Bekleyen trade'leri ve güncel open order setini tek transaction ile yazar.
    pub fn flush(&mut self, conn: &mut Connection) -> rusqlite::Result<()> {
        if self.pending_trades.is_empty() && self.opens.is_empty() {
            return Ok(());
        }

        let tx = conn.transaction()?;
        {
            let mut stmt_trade = tx.prepare_cached(
                "INSERT INTO paper_trades (order_id, symbol, side, price, quantity, fee, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for (order_id, symbol, side, price, quantity, fee, ts) in self.pending_trades.drain(..) {
                stmt_trade.execute(rusqlite::params![
                    order_id, symbol, side,
                    price.to_f64().unwrap_or(0.0),
                    quantity.to_f64().unwrap_or(0.0),
                    fee.to_f64().unwrap_or(0.0),
                    ts
                ])?;
            }

            // Open order'ları tam set olarak yaz (upsert улse REPLACE).
            let mut stmt_open = tx.prepare_cached(
                "INSERT OR REPLACE INTO paper_open_orders
                 (order_id, symbol, side, price, open_quantity, original_quantity, locked_balances_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for (order_id, row) in self.opens.iter() {
                stmt_open.execute(rusqlite::params![
                    order_id,
                    row.symbol,
                    row.side,
                    row.price,
                    row.open_qty.to_f64().unwrap_or(0.0),
                    row.original.to_f64().unwrap_or(0.0),
                    "{}"
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Son flush'tan beri kaç event gözlemlendi (metrik).
    pub fn applied(&self) -> u64 {
        self.applied
    }
}