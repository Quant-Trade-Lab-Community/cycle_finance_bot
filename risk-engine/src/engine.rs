use rust_decimal::Decimal;
use strategies_engine::trait_def::Signal;

pub struct RiskEngine {
    max_position: Decimal,
    current_position: Decimal,
    daily_loss_limit: Decimal,
    current_pnl: Decimal,
}

impl RiskEngine {
    pub fn new(max_position: Decimal, daily_loss_limit: Decimal) -> Self {
        Self {
            max_position,
            current_position: Decimal::ZERO,
            daily_loss_limit,
            current_pnl: Decimal::ZERO,
        }
    }

    pub fn process_signal(&self, signal: Signal, _strategy_id: u32) -> Option<Signal> {
        match signal {
            Signal::BuyMarket { quantity } | Signal::BuyLimit { quantity, .. } => {
                if self.current_position + quantity > self.max_position {
                    None // Reject
                } else {
                    Some(signal)
                }
            }
            Signal::SellMarket { quantity } | Signal::SellLimit { quantity, .. } => {
                if self.current_position - quantity < -self.max_position {
                    None
                } else {
                    Some(signal)
                }
            }
            Signal::None | Signal::CancelAll => Some(signal),
        }
    }

    pub fn update_position(&mut self, delta: Decimal) {
        self.current_position += delta;
    }

    pub fn update_pnl(&mut self, delta: Decimal) {
        self.current_pnl += delta;
    }

    pub fn is_daily_loss_exceeded(&self) -> bool {
        self.current_pnl <= -self.daily_loss_limit
    }

    pub fn current_position(&self) -> Decimal {
        self.current_position
    }

    pub fn max_position(&self) -> Decimal {
        self.max_position
    }
}
