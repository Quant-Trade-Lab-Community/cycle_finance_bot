use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use crate::strategy::trait_def::Signal;

pub struct RiskEngine {
    max_position: i64,
    current_position: AtomicI64,
    daily_loss_limit: i64,
    current_pnl: AtomicI64,
    trades_per_second: AtomicU64,
}

impl RiskEngine {
    pub fn new(max_position: i64, daily_loss_limit: i64) -> Self {
        Self {
            max_position,
            current_position: AtomicI64::new(0),
            daily_loss_limit,
            current_pnl: AtomicI64::new(0),
            trades_per_second: AtomicU64::new(0),
        }
    }

    pub fn process_signal(&self, signal: Signal, _strategy_id: u32) -> Option<Signal> {
        match signal {
            Signal::BuyMarket { quantity } | Signal::BuyLimit { quantity, .. } => {
                let pos = self.current_position.load(Ordering::Acquire);
                if pos + (quantity as i64) > self.max_position {
                    None // Reject
                } else {
                    Some(signal) // Pass to gateway (the orchestrator will dispatch it)
                }
            }
            Signal::SellMarket { quantity } | Signal::SellLimit { quantity, .. } => {
                let pos = self.current_position.load(Ordering::Acquire);
                if pos - (quantity as i64) < -self.max_position {
                    None
                } else {
                    Some(signal)
                }
            }
            Signal::None | Signal::CancelAll => Some(signal),
        }
    }

    pub fn update_position(&self, delta: i64) {
        self.current_position.fetch_add(delta, Ordering::Release);
    }
}
