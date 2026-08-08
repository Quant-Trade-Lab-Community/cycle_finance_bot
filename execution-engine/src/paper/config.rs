use rust_decimal::Decimal;
use std::env;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct PaperConfig {
    pub initial_usdt: Decimal,
    pub initial_btc: Decimal,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub base_latency_ms: u64,
    pub latency_jitter_ms: u64,
    pub fee_deduction_asset: String,
    pub db_path: String,
    pub batch_write_interval_ms: u64,
    pub recover_state_on_startup: bool,
    pub wal_enabled: bool,
    /// Başlangıç pozisyon modu: "ONE_WAY" veya "HEDGE" (API ile değiştirilebilir).
    pub position_mode: String,
    /// Varsayılan marj tipi: "CROSSED" veya "ISOLATED" (sembol bazında API ile değiştirilebilir).
    pub margin_type: String,
    /// Risk parametreleri
    pub min_position_notional: Decimal,
    pub max_leverage: Decimal,
    pub max_drawdown_pct: Decimal,
    pub max_daily_loss: Decimal,
}

impl PaperConfig {
    pub fn load_from_env() -> Self {
        Self {
            initial_usdt: env::var("PAPER_INITIAL_USDT")
                .unwrap_or_else(|_| "500.0".to_string())
                .parse()
                .unwrap_or(Decimal::from(500)),
            initial_btc: env::var("PAPER_INITIAL_BTC")
                .unwrap_or_else(|_| "0.0".to_string())
                .parse()
                .unwrap_or(Decimal::ZERO),
            maker_fee: env::var("PAPER_MAKER_FEE")
                .unwrap_or_else(|_| "0.0002".to_string())
                .parse()
                .unwrap_or(Decimal::from_str("0.0002").unwrap()),
            taker_fee: env::var("PAPER_TAKER_FEE")
                .unwrap_or_else(|_| "0.0005".to_string())
                .parse()
                .unwrap_or(Decimal::from_str("0.0005").unwrap()),
            base_latency_ms: env::var("PAPER_BASE_LATENCY_MS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            latency_jitter_ms: env::var("PAPER_LATENCY_JITTER_MS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap_or(2),
            fee_deduction_asset: env::var("PAPER_FEE_DEDUCTION_ASSET")
                .unwrap_or_else(|_| "QUOTE".to_string()),
            db_path: env::var("PAPER_DB_PATH")
                .unwrap_or_else(|_| "./data-engine/data/market_data.db".to_string()),
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
            position_mode: env::var("PAPER_POSITION_MODE")
                .unwrap_or_else(|_| "HEDGE".to_string()),
            margin_type: env::var("PAPER_MARGIN_TYPE")
                .unwrap_or_else(|_| "CROSSED".to_string()),
            min_position_notional: env::var("PAPER_MIN_POSITION_NOTIONAL")
                .unwrap_or_else(|_| "6.0".to_string())
                .parse()
                .unwrap_or(Decimal::from(6)),
            max_leverage: env::var("PAPER_MAX_LEVERAGE")
                .unwrap_or_else(|_| "20.0".to_string())
                .parse()
                .unwrap_or(Decimal::from(20)),
            max_drawdown_pct: env::var("PAPER_MAX_DRAWDOWN_PCT")
                .unwrap_or_else(|_| "0.05".to_string())
                .parse()
                .unwrap_or(Decimal::from_str("0.05").unwrap()),
            max_daily_loss: env::var("PAPER_MAX_DAILY_LOSS")
                .unwrap_or_else(|_| "1000.0".to_string())
                .parse()
                .unwrap_or(Decimal::from(1000)),
        }
    }
}
