//! PostgreSQL Event Store (tam set: `--features full`).
//!
//! `domain_events` tablosuna her event'i yazar ve replay için okur.
//! Ayrıca `account_snapshots` için şema hazırlığı yapar.

use crate::events::DomainEvent;
use rust_decimal::Decimal;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;

pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS domain_events (
                id BIGSERIAL PRIMARY KEY,
                event_type TEXT NOT NULL,
                payload JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS account_snapshots (
                id BIGSERIAL PRIMARY KEY,
                event_count BIGINT NOT NULL,
                snapshot JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn append(&self, event: &DomainEvent) -> Result<(), sqlx::Error> {
        let payload = serde_json::to_value(event).unwrap_or_default();
        let event_type = match event {
            DomainEvent::OrderCreated { .. } => "order_created",
            DomainEvent::OrderFilled { .. } => "order_filled",
            DomainEvent::OrderCancelled { .. } => "order_cancelled",
            DomainEvent::PositionOpened { .. } => "position_opened",
            DomainEvent::PositionClosed { .. } => "position_closed",
            DomainEvent::Liquidation { .. } => "liquidation",
            DomainEvent::FundingRateApplied { .. } => "funding_rate_applied",
        };
        sqlx::query("INSERT INTO domain_events (event_type, payload) VALUES ($1, $2)")
            .bind(event_type)
            .bind(payload)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn replay(&self, limit: i64) -> Result<Vec<DomainEvent>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT payload FROM domain_events ORDER BY id ASC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let payload: serde_json::Value = row.try_get("payload")?;
            if let Ok(ev) = serde_json::from_value(payload) {
                events.push(ev);
            }
        }
        Ok(events)
    }

    /// Her 1000 event'te bir çağrılır; son durumu snapshot olarak saklar.
    pub async fn save_snapshot(&self, event_count: i64, snapshot: &serde_json::Value) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO account_snapshots (event_count, snapshot) VALUES ($1, $2)")
            .bind(event_count)
            .bind(snapshot)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// Decimal'in Postgres'e NUMERIC olarak güvenle gitmesi için yardımcı.
pub fn decimal_to_str(d: &Decimal) -> String {
    d.to_string()
}
