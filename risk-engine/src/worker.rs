//! RiskWorker — soğuk yol döngüsü: korelasyon → Tikhonov → EWMA vol → VaR →
//! konsantrasyon → önerilen limitler. 60s'de `run_cycle` çağrılır ve sonuç
//! `RiskCache`'e yazılır. Asla sıcak tick yolunda çalışmaz.

use crate::cache::{RiskCache, RiskParameters};
use crate::correlation;
use crate::policy::RiskPolicy;
use crate::var;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;

/// Worker davranış ayarları.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Sembol başına tutulan maksimum fiyat örneği.
    pub max_samples: usize,
    /// EWMA lambda.
    pub lambda: f64,
    /// Korelasyon matrisi hedef koşul sayısı.
    pub target_condition: f64,
    /// VaR güven seviyesi (parametrik model %99 sabit — `var.rs`).
    pub var_confidence: f64,
    /// Günlük kayıp bütçesi (USDT) — öneri üretiminde.
    pub daily_loss_budget: Decimal,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            max_samples: 120,
            lambda: 0.94,
            target_condition: 50.0,
            var_confidence: 0.99,
            daily_loss_budget: Decimal::from(50),
        }
    }
}

/// Sembol bazlı fiyat geçmişi (mark fiyatları).
#[derive(Debug, Clone, Default)]
pub struct PriceHistory {
    max_samples: usize,
    samples: HashMap<String, Vec<f64>>,
}

impl PriceHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            max_samples: max_samples.max(2),
            samples: HashMap::new(),
        }
    }

    pub fn ingest(&mut self, symbol: &str, price: f64) {
        let v = self.samples.entry(symbol.to_string()).or_default();
        v.push(price);
        if v.len() > self.max_samples {
            let excess = v.len() - self.max_samples;
            v.drain(0..excess);
        }
    }

    /// Sembolün log getiri serisi.
    pub fn log_returns(&self, symbol: &str) -> Vec<f64> {
        let Some(v) = self.samples.get(symbol) else {
            return Vec::new();
        };
        v.windows(2)
            .map(|w| (w[1] / w[0].max(1e-12)).ln())
            .collect()
    }

    pub fn symbols(&self) -> Vec<String> {
        self.samples.keys().cloned().collect()
    }

    pub fn sample_count(&self, symbol: &str) -> usize {
        self.samples.get(symbol).map(|v| v.len()).unwrap_or(0)
    }
}

/// Soğuk yol işlemci.
pub struct RiskWorker {
    pub config: WorkerConfig,
    pub history: PriceHistory,
    pub cache: Arc<RiskCache>,
    last_params: RiskParameters,
}

impl RiskWorker {
    pub fn new(config: WorkerConfig, cache: Arc<RiskCache>) -> Self {
        Self {
            history: PriceHistory::new(config.max_samples),
            config,
            cache,
            last_params: RiskParameters::unavailable(),
        }
    }

    /// Mark fiyatını geçmişe ekler.
    pub fn ingest_mark(&mut self, symbol: &str, price: f64) {
        self.history.ingest(symbol, price);
    }

    /// Tek çevrim: model parametrelerini üretir ve cache'e yazar.
    pub fn run_cycle(&mut self, ts_ms: u64) -> RiskParameters {
        let params = self.compute_params(ts_ms);
        self.last_params = params;
        self.cache.write(params);
        params
    }

    pub fn last_params(&self) -> RiskParameters {
        self.last_params
    }

    fn compute_params(&self, ts_ms: u64) -> RiskParameters {
        let symbols = self.history.symbols();
        let n = symbols.len();
        if n == 0 {
            return RiskParameters::unavailable();
        }

        // Log getirileri.
        let returns: Vec<Vec<f64>> = symbols
            .iter()
            .map(|s| self.history.log_returns(s))
            .collect();
        // Her sembolün en az 2 getirisi olmalı.
        if returns.iter().any(|r| r.len() < 2) {
            return RiskParameters {
                n_symbols: n,
                available: false,
                ..Default::default()
            };
        }

        let corr = correlation::correlation_matrix(&returns);
        let reg = correlation::regularize_correlation_matrix(&corr, self.config.target_condition);

        let vols: Vec<f64> = returns
            .iter()
            .map(|r| correlation::ewma_volatility(r, self.config.lambda).unwrap_or(0.0))
            .collect();

        // Eşit ağırlık varsayımı (brüt exposure payları dışarıdan geçilebilir).
        let weights: Vec<f64> = vec![1.0 / n as f64; n];

        let var_pct = match &reg {
            Some(reg) => var::parametric_var_99_1d(reg, &vols, &weights),
            None => None,
        };
        let var_pct = var_pct.unwrap_or(0.0);

        let hhi = weights.iter().map(|w| w * w).sum();
        let cond = reg
            .as_ref()
            .and_then(|r| correlation::condition_number(r))
            .unwrap_or(f64::NAN);

        let portfolio_vol = {
            let mut v = 0.0f64;
            let n = vols.len();
            for i in 0..n {
                for j in 0..n {
                    let c = reg
                        .as_ref()
                        .map(|r| r[i][j])
                        .unwrap_or(if i == j { 1.0 } else { 0.0 });
                    v += weights[i] * weights[j] * vols[i] * vols[j] * c;
                }
            }
            v.sqrt()
        };

        // Volatilite verisi yokken (var≈0) konservatif: günlük bütçe kadar.
        let suggested_max_position = if var_pct > 0.0 {
            var::safe_notional(self.config.daily_loss_budget, var_pct)
        } else {
            self.config.daily_loss_budget
        };
        // Önerilen kaldıraç: günlük bütçenin var'a oranıyla 1..=3 aralığında.
        let suggested_leverage = {
            let ratio = if var_pct > 0.0 {
                (self.config.daily_loss_budget.to_f64().unwrap_or(0.0) / var_pct / 1000.0).clamp(1.0, 3.0)
            } else {
                1.0
            };
            Decimal::from_f64_retain(ratio).unwrap_or(Decimal::ONE)
        };

        RiskParameters {
            n_symbols: n,
            portfolio_volatility: portfolio_vol,
            var_99_1d_pct: var_pct,
            correlation_condition: cond,
            hhi,
            suggested_max_position_usdt: suggested_max_position,
            suggested_max_leverage: suggested_leverage,
            computed_at_ms: ts_ms,
            available: true,
            gate_ready: true,
        }
    }
}

use rust_decimal::prelude::ToPrimitive;

/// Önerilen parametreleri politikaya yansıtır (örnek kullanım için yardımcı).
pub fn apply_suggestions(policy: &mut RiskPolicy, params: &RiskParameters) {
    if params.available
        && params.suggested_max_position_usdt > Decimal::ZERO
        && (policy.max_position_usdt.is_zero() || params.suggested_max_position_usdt < policy.max_position_usdt)
    {
        policy.max_position_usdt = params.suggested_max_position_usdt;
    }
    if params.available
        && params.suggested_max_leverage > Decimal::ZERO
        && (policy.max_leverage.is_zero() || params.suggested_max_leverage < policy.max_leverage)
    {
        policy.max_leverage = params.suggested_max_leverage;
    }
}
