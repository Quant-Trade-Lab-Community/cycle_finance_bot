//! TimescaleDB batch writer — SQLite yerine zaman-serisi kalıcılığı.
//!
//! Veri akışı kuralı aynıdır: her akış kendi hypertable'ına yazar
//! (`trades`, `orderbooks`, `liquidations`, `open_interests`,
//! `funding_rates`, `markprices`, `lastprices`, `indexprices`, ...).
//!
//! Batch commit: 1000 kayıt veya 1 sn. Bağlantı yoksa üstel bekleme ile
//! yeniden dener — veri akışını asla durdurmaz (ring hep beslenir).

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use flume::Receiver;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, Transaction};
use transport::events::{EventType, OwnedEvent};
use transport::flow::FlowKind;

/// TimescaleDB bağlantı URL'si (`TIMESCALEDB_URL`).
pub fn default_db_url() -> String {
    std::env::var("TIMESCALEDB_URL")
        .unwrap_or_else(|_| "postgres://cycle:cycle@localhost:5432/market_data".into())
}

/// Bağımsız bir thread'de TimescaleDB yazıcısını başlatır.
/// Bağlantı kurulana kadar (ve sonraki kopmalarda) bekleme ile dener.
pub fn start_tsdb_writer(rx: Receiver<OwnedEvent>, kind: FlowKind) {
    let url = default_db_url();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tsdb tokio runtime");

    let pool = loop {
        match runtime.block_on(
            PgPoolOptions::new().max_connections(2).connect(&url),
        ) {
            Ok(p) => {
                println!("[TSDB] bağlandı: {} ({})", url, kind.as_str());
                break p;
            }
            Err(e) => {
                eprintln!("[TSDB] bağlantı hatası: {e} — 2 sn sonra yeniden deneniyor");
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    };

    if let Err(e) = runtime.block_on(ensure_schema(&pool, kind)) {
        eprintln!("[TSDB] şema oluşturma hatası: {e}");
    }

    runtime.block_on(run_writer(&pool, rx, kind));
}

/// Her hypertable için DDL. Tüm tablolar `timestamp` (ms) üzerine hypertable'dır.
const SCHEMAS: &[(&str, &str)] = &[
    ("trades", "CREATE TABLE IF NOT EXISTS trades (symbol TEXT NOT NULL, price DOUBLE PRECISION NOT NULL, quantity DOUBLE PRECISION NOT NULL, is_buyer_maker BOOLEAN NOT NULL, timestamp BIGINT NOT NULL)"),
    ("orderbooks", "CREATE TABLE IF NOT EXISTS orderbooks (symbol TEXT NOT NULL, bids JSONB NOT NULL, asks JSONB NOT NULL, timestamp BIGINT NOT NULL)"),
    ("liquidations", "CREATE TABLE IF NOT EXISTS liquidations (symbol TEXT NOT NULL, side SMALLINT NOT NULL, price DOUBLE PRECISION NOT NULL, quantity DOUBLE PRECISION NOT NULL, timestamp BIGINT NOT NULL)"),
    ("funding_rates", "CREATE TABLE IF NOT EXISTS funding_rates (symbol TEXT NOT NULL, mark_price DOUBLE PRECISION NOT NULL, index_price DOUBLE PRECISION NOT NULL, funding_rate DOUBLE PRECISION NOT NULL, next_funding_time BIGINT NOT NULL, timestamp BIGINT NOT NULL)"),
    ("open_interests", "CREATE TABLE IF NOT EXISTS open_interests (symbol TEXT NOT NULL, open_interest DOUBLE PRECISION NOT NULL, timestamp BIGINT NOT NULL)"),
    ("markprices", "CREATE TABLE IF NOT EXISTS markprices (symbol TEXT NOT NULL, price DOUBLE PRECISION NOT NULL, timestamp BIGINT NOT NULL)"),
    ("indexprices", "CREATE TABLE IF NOT EXISTS indexprices (symbol TEXT NOT NULL, price DOUBLE PRECISION NOT NULL, timestamp BIGINT NOT NULL)"),
    ("lastprices", "CREATE TABLE IF NOT EXISTS lastprices (symbol TEXT NOT NULL, price DOUBLE PRECISION NOT NULL, timestamp BIGINT NOT NULL)"),
];

fn tables_for(kind: FlowKind) -> &'static [&'static str] {
    match kind {
        FlowKind::Trade => &["trades"],
        FlowKind::Depth => &["orderbooks"],
        FlowKind::Liquidation => &["liquidations"],
        FlowKind::OpenInterest => &["open_interests"],
        FlowKind::Funding => &["funding_rates"],
        FlowKind::MarkPrice => &["markprices"],
        FlowKind::LastPrice => &["lastprices"],
        FlowKind::IndexPrice => &["indexprices"],
    }
}

async fn ensure_schema(pool: &PgPool, kind: FlowKind) -> Result<(), sqlx::Error> {
    for table in tables_for(kind) {
        if let Some((_, ddl)) = SCHEMAS.iter().find(|(t, _)| *t == *table) {
            sqlx::query(ddl).execute(pool).await?;
        }
        sqlx::query(&format!(
            "SELECT create_hypertable('{}', 'timestamp', if_not_exists => TRUE)",
            table
        ))
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn run_writer(pool: &PgPool, rx: Receiver<OwnedEvent>, kind: FlowKind) {
    let mut batch: Vec<OwnedEvent> = Vec::with_capacity(1000);
    let mut last_flush = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(ev) => batch.push(ev),
            Err(flume::RecvTimeoutError::Timeout) => {}
            Err(flume::RecvTimeoutError::Disconnected) => break,
        }

        let due = batch.len() >= 1000 || (!batch.is_empty() && last_flush.elapsed().as_secs() >= 1);
        if due {
            if let Err(e) = flush(pool, kind, &batch).await {
                eprintln!("[TSDB] yazma hatası: {e}");
            }
            batch.clear();
            last_flush = Instant::now();
        }
    }

    if !batch.is_empty() {
        let _ = flush(pool, kind, &batch).await;
    }
}

async fn flush(pool: &PgPool, kind: FlowKind, batch: &[OwnedEvent]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for ev in batch {
        insert_event(&mut tx, kind, ev).await?;
    }
    tx.commit().await
}

fn symbol_str(ev: &OwnedEvent) -> String {
    let len = ev.symbol.iter().position(|&c| c == 0).unwrap_or(16);
    String::from_utf8_lossy(&ev.symbol[..len]).to_string()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

async fn insert_event(
    tx: &mut Transaction<'_, Postgres>,
    kind: FlowKind,
    ev: &OwnedEvent,
) -> Result<(), sqlx::Error> {
    let sym = symbol_str(ev);
    let now = now_ms();

    match (kind, ev.payload) {
        (FlowKind::Trade, EventType::Trade { price, quantity, timestamp, is_buyer_maker }) => {
            sqlx::query("INSERT INTO trades (symbol, price, quantity, is_buyer_maker, timestamp) VALUES ($1, $2, $3, $4, $5)")
                .bind(&sym)
                .bind(f64(price))
                .bind(f64(quantity))
                .bind(is_buyer_maker)
                .bind(ts_or(now, timestamp))
                .execute(&mut **tx)
                .await?;
        }
        (FlowKind::Depth, EventType::Orderbook { bids, asks }) => {
            let bids_json = serde_json::to_string(
                &bids
                    .iter()
                    .filter(|(p, q)| !p.is_zero() || !q.is_zero())
                    .map(|(p, q)| [f64(*p), f64(*q)])
                    .collect::<Vec<[f64; 2]>>(),
            )
            .unwrap_or_else(|_| "[]".into());
            let asks_json = serde_json::to_string(
                &asks
                    .iter()
                    .filter(|(p, q)| !p.is_zero() || !q.is_zero())
                    .map(|(p, q)| [f64(*p), f64(*q)])
                    .collect::<Vec<[f64; 2]>>(),
            )
            .unwrap_or_else(|_| "[]".into());
            sqlx::query("INSERT INTO orderbooks (symbol, bids, asks, timestamp) VALUES ($1, $2::jsonb, $3::jsonb, $4)")
                .bind(&sym)
                .bind(&bids_json)
                .bind(&asks_json)
                .bind(now)
                .execute(&mut **tx)
                .await?;
        }
        (FlowKind::Liquidation, EventType::Liquidation { side, price, quantity, timestamp }) => {
            sqlx::query("INSERT INTO liquidations (symbol, side, price, quantity, timestamp) VALUES ($1, $2, $3, $4, $5)")
                .bind(&sym)
                .bind(i16::from(side))
                .bind(f64(price))
                .bind(f64(quantity))
                .bind(ts_or(now, timestamp))
                .execute(&mut **tx)
                .await?;
        }
        (FlowKind::OpenInterest, EventType::OpenInterest { open_interest, timestamp }) => {
            sqlx::query("INSERT INTO open_interests (symbol, open_interest, timestamp) VALUES ($1, $2, $3)")
                .bind(&sym)
                .bind(f64(open_interest))
                .bind(ts_or(now, timestamp))
                .execute(&mut **tx)
                .await?;
        }
        (FlowKind::Funding, EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time }) => {
            sqlx::query("INSERT INTO funding_rates (symbol, mark_price, index_price, funding_rate, next_funding_time, timestamp) VALUES ($1, $2, $3, $4, $5, $6)")
                .bind(&sym)
                .bind(f64(mark_price))
                .bind(f64(index_price))
                .bind(f64(funding_rate))
                .bind(next_funding_time as i64)
                .bind(now)
                .execute(&mut **tx)
                .await?;
        }
        (FlowKind::MarkPrice, EventType::FundingRate { mark_price, .. }) => {
            sqlx::query("INSERT INTO markprices (symbol, price, timestamp) VALUES ($1, $2, $3)")
                .bind(&sym)
                .bind(f64(mark_price))
                .bind(now)
                .execute(&mut **tx)
                .await?;
        }
        (FlowKind::IndexPrice, EventType::FundingRate { index_price, .. }) => {
            sqlx::query("INSERT INTO indexprices (symbol, price, timestamp) VALUES ($1, $2, $3)")
                .bind(&sym)
                .bind(f64(index_price))
                .bind(now)
                .execute(&mut **tx)
                .await?;
        }
        (FlowKind::LastPrice, EventType::FundingRate { mark_price, .. }) => {
            sqlx::query("INSERT INTO lastprices (symbol, price, timestamp) VALUES ($1, $2, $3)")
                .bind(&sym)
                .bind(f64(mark_price))
                .bind(now)
                .execute(&mut **tx)
                .await?;
        }
        _ => {}
    }

    Ok(())
}

fn ts_or(now: i64, ts: u64) -> i64 {
    if ts == 0 {
        now
    } else {
        ts as i64
    }
}
