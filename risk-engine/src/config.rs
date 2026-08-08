//! Risk konfigürasyonu — `risk.toml` yükleme ve hot-reload.

use crate::policy::RiskPolicy;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Yapılandırma yükleme hataları.
#[derive(Debug)]
pub enum RiskConfigError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for RiskConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskConfigError::Io(e) => write!(f, "risk.toml okuma hatası: {e}"),
            RiskConfigError::Parse(e) => write!(f, "risk.toml parse hatası: {e}"),
        }
    }
}

impl std::error::Error for RiskConfigError {}

/// `risk.toml` konumunu çevreden veya varsayılandan bulur.
pub fn resolve_risk_config_path() -> PathBuf {
    std::env::var("RISK_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("risk.toml"))
}

/// `risk.toml`'u yükler; dosya yoksa varsayılan politikayı döndürür.
pub fn load_risk_config() -> Result<RiskPolicy, RiskConfigError> {
    load_risk_config_from(resolve_risk_config_path().as_path())
}

/// Belirli bir yoldan `risk.toml` yükler (yoksa varsayılan).
pub fn load_risk_config_from(path: &Path) -> Result<RiskPolicy, RiskConfigError> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RiskPolicy::default());
        }
        Err(e) => return Err(RiskConfigError::Io(e)),
    };
    toml::from_str::<RiskPolicy>(&content)
        .map_err(|e| RiskConfigError::Parse(e.to_string()))
}

/// Mtime izleyici — dosya değiştiğinde politika yeniden yüklenir.
#[derive(Debug)]
pub struct ConfigWatcher {
    path: PathBuf,
    last_modified: Option<SystemTime>,
}

impl ConfigWatcher {
    pub fn new(path: PathBuf) -> Self {
        Self {
            last_modified: Self::modified(&path),
            path,
        }
    }

    fn modified(path: &Path) -> Option<SystemTime> {
        std::fs::metadata(path).and_then(|m| m.modified()).ok()
    }

    /// Dosya değiştiyse yeni politikayı döndürür.
    pub fn reload_if_changed(&mut self) -> Option<RiskPolicy> {
        let now = Self::modified(&self.path);
        if now != self.last_modified {
            self.last_modified = now;
            load_risk_config_from(&self.path).ok()
        } else {
            None
        }
    }
}

/// Hot-reload'a hazır politika sarmalayıcı (execution/kore içinde kullanım için).
#[derive(Debug)]
pub struct ReloadablePolicy {
    pub watcher: ConfigWatcher,
    pub policy: RiskPolicy,
}

impl ReloadablePolicy {
    pub fn new(path: PathBuf) -> Self {
        let watcher = ConfigWatcher::new(path.clone());
        let policy = load_risk_config_from(&path).unwrap_or_default();
        Self { watcher, policy }
    }

    pub fn reload_if_changed(&mut self) {
        if let Some(new_policy) = self.watcher.reload_if_changed() {
            self.policy = new_policy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn toml_parses_full_policy() {
        let toml = r#"
max_position_usdt = 1000
max_notional_per_order = 500
max_gross_exposure_usdt = 3000
max_hhi = 0.5
max_leverage = 3
max_daily_loss_usdt = 50
max_drawdown_pct = 0.20
maintenance_margin_rate = 0.005
max_orders_per_min = 10
stale_mark_ms = 200
consecutive_rejection_auto_stop = 3
gate_on_parametric_risk = false
enable_liquidity_gate = false
max_slippage_bps = 50
blocklist = ["TRXUSDT"]

[symbol.HEIUSDT]
max_position_usdt = 500
max_leverage = 5
"#;
        let policy: RiskPolicy = toml::from_str(toml).expect("toml parse");
        assert_eq!(policy.max_leverage, Decimal::from(3));
        assert_eq!(policy.max_drawdown_pct, Decimal::from_str("0.20").unwrap());
        assert!(policy.is_blocked("TRXUSDT"));
        let eff = policy.effective("HEIUSDT");
        assert_eq!(eff.max_position_usdt, Decimal::from(500));
        assert_eq!(eff.max_leverage, Decimal::from(5));
        // Override olmayan sembol genel limiti kullanır.
        let eff2 = policy.effective("BTCUSDT");
        assert_eq!(eff2.max_position_usdt, Decimal::from(1000));
    }

    #[test]
    fn missing_file_falls_back_to_default() {
        let p = load_risk_config_from(Path::new("/nonexistent/risk.toml")).unwrap();
        assert_eq!(p.max_leverage, Decimal::from(3));
    }
}
