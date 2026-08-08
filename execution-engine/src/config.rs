//! Execution servisi konfigürasyonu (env tabanlı, `EXEC_` öneki).
//!
//! Canlı mod varsayılan olarak DRY_RUN'dur: emirler doğrulanır, imzalanır,
//! loglanır ama borsaya gönderilmez. Canlı emir gönderimi `EXEC_DRY_RUN=false`
//! ile **açıkça** etkinleştirilir.

use rust_decimal::Decimal;
use std::collections::HashSet;
use std::env;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingMode {
    Live,
    Paper,
}

impl TradingMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradingMode::Live => "LIVE",
            TradingMode::Paper => "PAPER",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecConfig {
    pub mode: TradingMode,
    /// DRY_RUN açıkken hiçbir emir borsaya gitmez (güvenlik önlemi).
    pub dry_run: bool,
    pub api_key: String,
    pub secret_key: String,
    /// REST taban URL. Testnet: https://testnet.binancefuture.com
    pub base_url: String,
    /// User-data WS taban URL.
    pub ws_url: String,
    /// İmzalı isteklerin geçerlilik penceresi (ms).
    pub recv_window_ms: u64,
    /// HTTP istek zaman aşımı (ms).
    pub request_timeout_ms: u64,
    /// Tek emir için üst USDT notional limiti (0 = sınırsız).
    pub max_notional_usdt: Decimal,
    /// Dakikada gönderilebilecek maksimum emir (0 = sınırsız).
    pub max_orders_per_min: u32,
    /// Emir gönderimi tamamen engellenen semboller.
    pub symbol_blocklist: HashSet<String>,
    /// Kill switch dosya yolu (varsa yazma reddedilir).
    pub kill_switch_path: String,
    /// listenKey keepalive aralığı (sn). Binance 60dk'da süreyi düşürür.
    pub listen_key_keepalive_sec: u64,
    /// WS yeniden bağlantı sonrası tam yeniden eşitleme zorunlu.
    pub resync_on_reconnect: bool,
    /// Periyodik uzlaştırma aralığı (sn): pozisyon/açık emirler REST ile karşılaştırılır.
    pub reconcile_interval_sec: u64,
    /// İmzalı isteklerde sunucu saati senkronizasyonu (mutlak drift eşiği, ms).
    pub server_time_sync_ms: i64,
    /// Aynı anda havada (in-flight) olabilecek maksimum emir.
    pub max_in_flight: usize,
    /// İlk eşitleme (initial sync) zaman aşımı (sn).
    pub initial_sync_timeout_sec: u64,
    /// REST API auth için JWT secret.
    pub jwt_secret: String,
    /// REST API bind adresi.
    pub api_addr: String,
}

impl ExecConfig {
    pub fn load_from_env() -> Self {
        let mode = match env::var("EXEC_MODE").unwrap_or_else(|_| "LIVE".into()).to_uppercase().as_str() {
            "PAPER" => TradingMode::Paper,
            _ => TradingMode::Live,
        };
        // Canlı modda bile varsayılan DRY_RUN güvenliğidir.
        let dry_run = env::var("EXEC_DRY_RUN")
            .unwrap_or_else(|_| "true".into())
            .parse()
            .unwrap_or(true);

        let base_url = env::var("EXEC_BASE_URL").unwrap_or_else(|_| "https://fapi.binance.com".into());
        let ws_url = env::var("EXEC_WS_URL").unwrap_or_else(|_| "wss://fstream.binance.com".into());

        let mut symbol_blocklist = HashSet::new();
        if let Ok(list) = env::var("EXEC_SYMBOL_BLOCKLIST") {
            for s in list.split(',') {
                let s = s.trim().to_uppercase();
                if !s.is_empty() {
                    symbol_blocklist.insert(s);
                }
            }
        }

        Self {
            mode,
            dry_run,
            api_key: env::var("BINANCE_API_KEY").unwrap_or_default(),
            secret_key: env::var("BINANCE_SECRET_KEY").unwrap_or_default(),
            base_url,
            ws_url,
            recv_window_ms: env::var("EXEC_RECV_WINDOW_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(1000),
            request_timeout_ms: env::var("EXEC_REQUEST_TIMEOUT_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(5_000),
            max_notional_usdt: env::var("EXEC_MAX_NOTIONAL")
                .ok().and_then(|v| Decimal::from_str(&v).ok())
                .unwrap_or(Decimal::from(1_000)),
            max_orders_per_min: env::var("EXEC_MAX_ORDERS_PER_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(60),
            symbol_blocklist,
            kill_switch_path: env::var("EXEC_KILL_SWITCH_PATH").unwrap_or_else(|_| "/tmp/exec_kill_switch".into()),
            listen_key_keepalive_sec: env::var("EXEC_LISTEN_KEY_KEEPALIVE_SEC").ok().and_then(|v| v.parse().ok()).unwrap_or(3_540),
            resync_on_reconnect: env::var("EXEC_RESYNC_ON_RECONNECT")
                .unwrap_or_else(|_| "true".into())
                .parse()
                .unwrap_or(true),
            reconcile_interval_sec: env::var("EXEC_RECONCILE_INTERVAL_SEC").ok().and_then(|v| v.parse().ok()).unwrap_or(300),
            server_time_sync_ms: env::var("EXEC_SERVER_TIME_SYNC_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(300),
            max_in_flight: env::var("EXEC_MAX_IN_FLIGHT").ok().and_then(|v| v.parse().ok()).unwrap_or(64),
            initial_sync_timeout_sec: env::var("EXEC_INITIAL_SYNC_TIMEOUT_SEC").ok().and_then(|v| v.parse().ok()).unwrap_or(60),
            jwt_secret: env::var("EXEC_JWT_SECRET").unwrap_or_else(|_| "exec-dev-secret-change-me".into()),
            api_addr: env::var("EXEC_API_ADDR").unwrap_or_else(|_| "127.0.0.1:3010".into()),
        }
    }
}
