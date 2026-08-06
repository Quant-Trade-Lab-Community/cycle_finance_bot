use std::env;

#[derive(Clone, Debug)]
pub struct PaperConfig {
    pub initial_usdt: f64,
    pub initial_btc: f64,
    pub maker_fee: f64,
    pub taker_fee: f64,
    pub base_latency_ms: u64,
    pub latency_jitter_ms: u64,
    pub slippage_model: String,
    pub market_impact_factor: f64,
    pub fee_deduction_asset: String,
    pub db_path: String,
    pub batch_write_interval_ms: u64,
    pub recover_state_on_startup: bool,
    pub wal_enabled: bool,
}

impl PaperConfig {
    pub fn load_from_env() -> Self {
        Self {
            initial_usdt: env::var("PAPER_INITIAL_USDT")
                .unwrap_or_else(|_| "100000.0".to_string())
                .parse()
                .unwrap_or(100000.0),
            initial_btc: env::var("PAPER_INITIAL_BTC")
                .unwrap_or_else(|_| "0.0".to_string())
                .parse()
                .unwrap_or(0.0),
            maker_fee: env::var("PAPER_MAKER_FEE")
                .unwrap_or_else(|_| "0.0002".to_string())
                .parse()
                .unwrap_or(0.0002),
            taker_fee: env::var("PAPER_TAKER_FEE")
                .unwrap_or_else(|_| "0.0005".to_string())
                .parse()
                .unwrap_or(0.0005),
            base_latency_ms: env::var("PAPER_BASE_LATENCY_MS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            latency_jitter_ms: env::var("PAPER_LATENCY_JITTER_MS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap_or(2),
            slippage_model: env::var("PAPER_SLIPPAGE_MODEL")
                .unwrap_or_else(|_| "L2_SWEEP".to_string()),
            market_impact_factor: env::var("PAPER_MARKET_IMPACT_FACTOR")
                .unwrap_or_else(|_| "0.00001".to_string())
                .parse()
                .unwrap_or(0.00001),
            fee_deduction_asset: env::var("PAPER_FEE_DEDUCTION_ASSET")
                .unwrap_or_else(|_| "QUOTE".to_string()),
            db_path: env::var("PAPER_DB_PATH")
                .unwrap_or_else(|_| "./market_data.db".to_string()),
            batch_write_interval_ms: env::var("PAPER_BATCH_WRITE_INTERVAL_MS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            recover_state_on_startup: env::var("PAPER_RECOVER_STATE_ON_STARTUP")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            wal_enabled: env::var("PAPER_WAL_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}
