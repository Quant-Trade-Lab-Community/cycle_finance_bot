// ============================================================================
// 5. RİSK "KATİL DÜĞME" — AdaptiveRiskEngine
// ar_low, avg_volume ile UT onayı bekleme mekanizması. max_risk_bp stop-loss.
// ============================================================================

use serde::{Deserialize, Serialize};

use crate::models::{Bar, Tick};
use crate::state::ProbabilisticState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskAction {
    Idle,
    TightenStop,
    HedgeAndReverse,
}

impl RiskAction {
    pub fn label(&self) -> &'static str {
        match self {
            RiskAction::Idle => "Idle",
            RiskAction::TightenStop => "TightenStop",
            RiskAction::HedgeAndReverse => "HedgeAndReverse",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskRecord {
    pub action: RiskAction,
    pub action_label: &'static str,
    pub max_risk_bp: i64,
    pub stop_price: f64,
    pub stop_bp: i64,
    pub ut_confirmation_pending: bool,
}

pub struct AdaptiveRiskEngine {
    pub max_risk_bp: i64, // 200 bps = %2
    pub current_stop: Tick,
    pub ar_low: Tick,
    pub avg_volume: u64,
    pub ut_confirmation_pending: bool,
}

impl AdaptiveRiskEngine {
    pub fn new(max_risk_bp: i64, ar_low: Tick, avg_volume: u64, entry: Tick) -> Self {
        let stop = Tick(entry.0.saturating_sub((entry.0 * max_risk_bp) / 10_000));
        Self {
            max_risk_bp,
            current_stop: stop,
            ar_low,
            avg_volume,
            ut_confirmation_pending: false,
        }
    }

    /// Her bar için risk aksiyonu.
    pub fn evaluate(&mut self, bar: &Bar, phase: &ProbabilisticState) -> RiskAction {
        if phase.distribution_weight > 0.8 && self.ut_confirmation_pending {
            if bar.close.0 < self.ar_low.0
                && bar.volume.0 > (self.avg_volume as f64 * 1.3) as u64
            {
                self.ut_confirmation_pending = false;
                return RiskAction::HedgeAndReverse;
            }
            return RiskAction::TightenStop;
        }
        RiskAction::Idle
    }

    pub fn record(&self, action: RiskAction, tick_size: f64) -> RiskRecord {
        let stop_bp = if self.current_stop.0 > 0 {
            10_000 * (self.current_stop.0 - self.ar_low.0).abs() / self.current_stop.0
        } else {
            0
        };
        RiskRecord {
            action,
            action_label: action.label(),
            max_risk_bp: self.max_risk_bp,
            stop_price: self.current_stop.0 as f64 * tick_size,
            stop_bp,
            ut_confirmation_pending: self.ut_confirmation_pending,
        }
    }
}