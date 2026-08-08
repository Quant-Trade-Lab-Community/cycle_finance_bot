//! AI Engine konfigürasyonu — `ai.toml` yükleme ve varsayılanlar.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Kök config. `ai.toml` dosyasından yüklenir (yoksa varsayılan).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    pub providers: ProvidersConfig,
    pub schedule: ScheduleConfig,
    pub execution: ExecutionConfig,
    pub risk: RiskGateConfig,
    pub context: ContextConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ProvidersConfig {
    /// openai | anthropic | none
    pub provider: String,
    pub openai_model: String,
    pub anthropic_model: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub timeout_secs: u64,
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            provider: "none".into(),
            openai_model: "gpt-4o-mini".into(),
            anthropic_model: "claude-sonnet-4-20250514".into(),
            temperature: 0.2,
            max_tokens: 2048,
            timeout_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ScheduleConfig {
    pub interval_secs: u64,
    pub symbols: Vec<String>,
    pub approval_wait_secs: u64,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            interval_secs: 60,
            symbols: vec![
                "BTCUSDT".into(),
                "ETHUSDT".into(),
                "SOLUSDT".into(),
                "HEIUSDT".into(),
            ],
            approval_wait_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ExecutionConfig {
    /// paper | live | both | none
    pub mode: String,
    /// auto | human (HITL onayı)
    pub approval: String,
    pub execd_url: String,
    pub execd_user: String,
    pub execd_password: String,
    pub paper_url: String,
    pub paper_admin_user: String,
    pub paper_admin_pass: String,
    /// Deterministik emir boyutu sınırı (USDT).
    pub max_notional_usdt: f64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            mode: "paper".into(),
            approval: "auto".into(),
            execd_url: "http://127.0.0.1:3010".into(),
            execd_user: "admin".into(),
            execd_password: "changeme123".into(),
            paper_url: "http://127.0.0.1:8080".into(),
            paper_admin_user: "admin".into(),
            paper_admin_pass: "changeme123".into(),
            max_notional_usdt: 1_000.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RiskGateConfig {
    pub enable_risk_gate: bool,
    pub anomaly_veto: bool,
    pub risk_config_path: String,
    pub initial_balance_usdt: f64,
}

impl Default for RiskGateConfig {
    fn default() -> Self {
        Self {
            enable_risk_gate: true,
            anomaly_veto: true,
            risk_config_path: "risk.toml".into(),
            initial_balance_usdt: 100_000.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    pub price_feed_url: String,
    pub detect_ms_url: String,
    pub calc_ind_url: String,
    /// İsteğe bağlı haber kaynağı (boş ise duygu agent'ı nötr kalır).
    pub news_feed_url: String,
    pub indicator_interval: String,
    pub structure_interval: String,
    pub structure_limit: u32,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            price_feed_url: "http://127.0.0.1:3004".into(),
            detect_ms_url: "http://127.0.0.1:3002".into(),
            calc_ind_url: "http://127.0.0.1:3007".into(),
            news_feed_url: String::new(),
            indicator_interval: "1m".into(),
            structure_interval: "1m".into(),
            structure_limit: 100,
        }
    }
}

impl AiConfig {
    /// `AI_CONFIG` env'inden veya `./ai.toml`'dan yükler; dosya yoksa varsayılan.
    pub fn load() -> Self {
        let path = Self::resolve_path();
        Self::load_from(&path)
    }

    pub fn resolve_path() -> PathBuf {
        std::env::var("AI_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("ai.toml"))
    }

    pub fn load_from(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str::<AiConfig>(&content)
                .unwrap_or_else(|e| {
                    eprintln!("⚠️  ai.toml parse hatası ({e}) — varsayılan config kullanılıyor");
                    AiConfig::default()
                }),
            Err(_) => AiConfig::default(),
        }
    }

    /// LLM API anahtarlarını env'den okur.
    pub fn openai_api_key() -> Option<String> {
        std::env::var("OPENAI_API_KEY").ok().filter(|k| !k.is_empty())
    }

    pub fn anthropic_api_key() -> Option<String> {
        std::env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.is_empty())
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            providers: ProvidersConfig::default(),
            schedule: ScheduleConfig::default(),
            execution: ExecutionConfig::default(),
            risk: RiskGateConfig::default(),
            context: ContextConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = AiConfig::default();
        assert_eq!(c.providers.provider, "none");
        assert_eq!(c.schedule.interval_secs, 60);
        assert_eq!(c.execution.mode, "paper");
        assert!(!c.schedule.symbols.is_empty());
    }

    #[test]
    fn parses_toml() {
        let toml = r#"
[providers]
provider = "openai"
openai_model = "gpt-4o"

[schedule]
interval_secs = 30
symbols = ["BTCUSDT", "ETHUSDT"]

[execution]
mode = "both"
approval = "human"
max_notional_usdt = 500

[risk]
enable_risk_gate = true
anomaly_veto = true
"#;
        let c: AiConfig = toml::from_str(toml).expect("toml parse");
        assert_eq!(c.providers.provider, "openai");
        assert_eq!(c.schedule.interval_secs, 30);
        assert_eq!(c.execution.mode, "both");
        assert_eq!(c.execution.approval, "human");
        assert_eq!(c.execution.max_notional_usdt, 500.0);
    }
}
