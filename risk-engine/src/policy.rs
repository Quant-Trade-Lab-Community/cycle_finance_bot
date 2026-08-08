//! Risk politikası — konfigüre edilebilir limit seti + sembol bazlı override.

use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

/// Sembol bazlı limit override'ları.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct PerSymbolLimits {
    pub max_position_usdt: Option<Decimal>,
    pub max_notional_per_order: Option<Decimal>,
    pub max_leverage: Option<Decimal>,
    pub max_slippage_bps: Option<Decimal>,
}

/// Tüm risk limitleri. `risk.toml` dosyasından yüklenir.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RiskPolicy {
    // ── Genel pozisyon/exposure limitleri ──
    /// Tek sembol için üst net pozisyon değeri (USDT). 0 = sınırsız.
    pub max_position_usdt: Decimal,
    /// Tek emir için üst notional (USDT). 0 = sınırsız.
    pub max_notional_per_order: Decimal,
    /// Portföy toplam brüt exposure (USDT). 0 = sınırsız.
    pub max_gross_exposure_usdt: Decimal,
    /// Portföy konsantrasyonu (Herfindahl–Hirschman Index üst sınırı). 0 = kapalı.
    pub max_hhi: f64,
    /// Maksimum kaldıraç (x).
    pub max_leverage: Decimal,

    // ── Kayıp limitleri ──
    /// Günlük maksimum kayıp (USDT; gerçekleşen + gerçekleşmemiş).
    pub max_daily_loss_usdt: Decimal,
    /// Maksimum drawdown (oransal, 0.20 = %20).
    pub max_drawdown_pct: Decimal,
    /// Bakım marjı oranı (likidasyon fiyatı hesabı için, varsayılan %0.5).
    pub maintenance_margin_rate: Decimal,

    // ── Emir akışı ──
    /// Dakikada maksimum emir. 0 = sınırsız.
    pub max_orders_per_min: u32,
    /// Emir gönderimi tamamen engellenen semboller.
    pub blocklist: HashSet<String>,

    // ── Fail-closed zamanlama ──
    /// Mark fiyatın bayat sayılacağı eşik (ms). Aşılırsa o sembol için red.
    pub stale_mark_ms: u64,

    // ── Parametrik risk kapısı (worker çıktısı) ──
    /// Parametrik risk modeli (VaR) mevcut değilken emir reddedilsin mi?
    /// false ise model kapalı sayılır (hot path bloklanmaz).
    pub gate_on_parametric_risk: bool,

    // ── Likidite kapısı (LOB simülasyonu) ──
    pub enable_liquidity_gate: bool,
    /// Maksimum kabul edilebilir slippage (baz puan). 0 = varsayılan 50.
    pub max_slippage_bps: Decimal,

    // ── Sembol bazlı override ──
    #[serde(rename = "symbol")]
    pub per_symbol: HashMap<String, PerSymbolLimits>,

    // ── Circuit breaker ──
    /// Ardışık red sayısı bu eşiği geçerse kill switch otomatik devreye girer.
    pub consecutive_rejection_auto_stop: u32,
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            max_position_usdt: Decimal::from(1_000),
            max_notional_per_order: Decimal::from(500),
            max_gross_exposure_usdt: Decimal::from(3_000),
            max_hhi: 0.0,
            max_leverage: Decimal::from(3),
            max_daily_loss_usdt: Decimal::from(50),
            max_drawdown_pct: Decimal::from_str("0.20").unwrap(),
            maintenance_margin_rate: Decimal::from_str("0.005").unwrap(),
            max_orders_per_min: 10,
            blocklist: HashSet::new(),
            stale_mark_ms: 200,
            gate_on_parametric_risk: false,
            enable_liquidity_gate: false,
            max_slippage_bps: Decimal::from(50),
            per_symbol: HashMap::new(),
            consecutive_rejection_auto_stop: 3,
        }
    }
}

impl RiskPolicy {
    /// Sembol bazlı override'lar uygulanmış etkin limitler.
    pub fn effective(&self, symbol: &str) -> EffectiveLimits {
        let sym = self.per_symbol.get(&symbol.to_uppercase());
        EffectiveLimits {
            max_position_usdt: sym
                .and_then(|s| s.max_position_usdt)
                .unwrap_or(self.max_position_usdt),
            max_notional_per_order: sym
                .and_then(|s| s.max_notional_per_order)
                .unwrap_or(self.max_notional_per_order),
            max_leverage: sym.and_then(|s| s.max_leverage).unwrap_or(self.max_leverage),
            max_slippage_bps: sym
                .and_then(|s| s.max_slippage_bps)
                .unwrap_or(self.max_slippage_bps),
        }
    }

    pub fn is_blocked(&self, symbol: &str) -> bool {
        self.blocklist.contains(&symbol.to_uppercase())
    }
}

/// Sembol override'ları uygulanmış, `RiskEngine::evaluate` içinde kullanılan limitler.
#[derive(Debug, Clone, Copy)]
pub struct EffectiveLimits {
    pub max_position_usdt: Decimal,
    pub max_notional_per_order: Decimal,
    pub max_leverage: Decimal,
    pub max_slippage_bps: Decimal,
}
