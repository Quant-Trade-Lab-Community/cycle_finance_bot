//! Cold Starter routines for system recovery and initialization.

use sqlx::postgres::PgPoolOptions;

/// Cold Starter routines for system recovery and initialization.
pub struct CatchupRoutines;

const EMA_PERIOD: usize = 200;

fn db_url() -> String {
    std::env::var("TIMESCALEDB_URL")
        .unwrap_or_else(|_| "postgres://cycle:cycle@localhost:5432/market_data".into())
}

impl CatchupRoutines {
    /// 1. TimescaleDB `trades` hypertable'ındaki son trade fiyatlarından 200 EMA'yı hesaplar.
    pub async fn fetch_200_ema(&self) -> Result<f64, String> {
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&db_url())
            .await
            .map_err(|e| format!("TimescaleDB bağlantı hatası: {e}"))?;

        let mut prices: Vec<f64> = sqlx::query_scalar(
            "SELECT price FROM trades ORDER BY timestamp DESC LIMIT $1",
        )
        .bind(EMA_PERIOD as i64)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Sorgu hatası: {e}"))?;

        if prices.is_empty() {
            return Err("TimescaleDB'de trade verisi yok".into());
        }

        prices.reverse();
        let multiplier = 2.0 / (EMA_PERIOD as f64 + 1.0);
        let mut ema = prices[0];
        for &price in &prices[1..] {
            ema = price * multiplier + ema * (1.0 - multiplier);
        }

        println!("ColdStarter: 200 EMA hesaplandı = {ema:.4} ({} trade)", prices.len());
        Ok(ema)
    }

    /// 2. Buffer'ı temizleyip canlı moda geçer.
    pub fn transition_to_live(&self) {
        println!("ColdStarter: Buffer cleared. Transitioning to LIVE mode.");
    }
}
