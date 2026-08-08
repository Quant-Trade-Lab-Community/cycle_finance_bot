//! Risk kapısı — `RiskEngine` (risk.toml politikası) + deterministik boyut kırpma
//! + agent veto kuralları. Onaylanan kararlar executor'a gider.

use crate::config::AiConfig;
use crate::executor::Executor;
use crate::{Action, FinalDecision};
use risk_engine::engine::RiskEngine;
use risk_engine::types::{MarkPrice, OrderIntent, OrderKind, Side};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::path::Path;

/// Gate sonucu — denetim izi/ekran için.
pub enum GateOutcome {
    Executed(String),
    Held(String),
    Rejected(String),
}

impl std::fmt::Display for GateOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateOutcome::Executed(msg) => write!(f, "İCRA EDİLDİ: {msg}"),
            GateOutcome::Held(msg) => write!(f, "BEKLEMEDE: {msg}"),
            GateOutcome::Rejected(msg) => write!(f, "REDDEDİLDİ: {msg}"),
        }
    }
}

pub struct RiskGate {
    engine: Option<RiskEngine>,
    anomaly_veto: bool,
    max_notional: Decimal,
}

impl RiskGate {
    pub fn new(cfg: &AiConfig) -> Self {
        let engine = if cfg.risk.enable_risk_gate {
            let policy = risk_engine::config::load_risk_config_from(Path::new(&cfg.risk.risk_config_path))
                .unwrap_or_default();
            let balance = Decimal::from_f64(cfg.risk.initial_balance_usdt)
                .unwrap_or(Decimal::from(100_000));
            Some(RiskEngine::with_policy(balance, policy))
        } else {
            None
        };
        Self {
            engine,
            anomaly_veto: cfg.risk.anomaly_veto,
            max_notional: Decimal::from_f64(cfg.execution.max_notional_usdt)
                .unwrap_or(Decimal::from(1_000)),
        }
    }

    /// Risk engine'ine canlı mark fiyatı besler (stale-mark reddini önler).
    pub fn on_mark(&self, symbol: &str, price: f64) {
        if let Some(eng) = &self.engine {
            let p = Decimal::from_f64(price).unwrap_or_default();
            if p.is_zero() {
                return;
            }
            eng.on_mark(&MarkPrice {
                symbol: symbol.to_string(),
                price: p,
                ts_ms: crate::now_ms(),
            });
        }
    }

    /// Kararı gate'ten geçirir; onaylanırsa executor'a iletir.
    pub async fn process(
        &self,
        decision: &FinalDecision,
        mark_price: Decimal,
        executor: &Executor,
    ) -> GateOutcome {
        if decision.action == Action::Hold {
            return GateOutcome::Held(format!("HOLD — {}", decision.rationale));
        }
        if decision.veto {
            return GateOutcome::Rejected("agent veto".into());
        }

        // Yüksek risk skoru + anomaly_veto açıksa otomatik red.
        if self.anomaly_veto && decision.risk_score >= 0.8 {
            return GateOutcome::Rejected(format!(
                "risk_score {:.2} >= 0.8 (anomaly_veto)",
                decision.risk_score
            ));
        }

        // Deterministik boyut sınırı: max_notional_usdt / mark.
        let mut quantity = decision.quantity;
        if mark_price.is_sign_positive() {
            let cap = self.max_notional / mark_price;
            quantity = quantity.min(cap);
        }
        if quantity.is_zero() || quantity.is_sign_negative() {
            return GateOutcome::Held("boyut sınırı sonrası miktar 0".into());
        }

        let side = match decision.action {
            Action::Buy => Some(Side::Buy),
            Action::Sell => Some(Side::Sell),
            Action::Hold => None,
        };
        let Some(side) = side else {
            return GateOutcome::Held("HOLD".into());
        };

        let intent = OrderIntent {
            strategy_id: 900,
            symbol: decision.symbol.clone(),
            side,
            quantity,
            price: decision.target_price,
            kind: if decision.target_price.is_some() {
                OrderKind::Limit
            } else {
                OrderKind::Market
            },
            reduce_only: false,
            close_position: false,
            leverage: None,
        };

        // RiskEngine onayı (kapalıysa doğrudan onaylı).
        if let Some(eng) = &self.engine {
            match eng.evaluate(intent) {
                risk_engine::types::RiskDecision::Rejected { reason, .. } => {
                    return GateOutcome::Rejected(format!("risk-gate: {}", reason.describe()));
                }
                risk_engine::types::RiskDecision::Approved { .. } => {}
            }
        }

        match executor
            .execute(&decision.symbol, decision.action, quantity, decision.target_price)
            .await
        {
            Ok(msg) => GateOutcome::Executed(msg),
            Err(e) => GateOutcome::Rejected(e),
        }
    }
}
