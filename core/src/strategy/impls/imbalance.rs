use crate::strategy::trait_def::{Strategy, Signal, FillReport};
use crate::memory::ring_buffer::MarketDataSlot;

pub struct OrderbookImbalanceStrategy {
    id: u32,
    threshold_ratio: f64, // e.g. 1.5 meaning Bids are 1.5x Asks
    current_position: f64,
}

impl OrderbookImbalanceStrategy {
    pub fn new(id: u32, threshold_ratio: f64) -> Self {
        Self {
            id,
            threshold_ratio,
            current_position: 0.0,
        }
    }
}

impl Strategy for OrderbookImbalanceStrategy {
    fn id(&self) -> u32 {
        self.id
    }

    fn on_market_data(&mut self, _frame_id: u64, data: &MarketDataSlot) -> Signal {
        // Fast fixed-point check on data payload
        // Assume data contains JSON or raw bytes of Orderbook.
        // In real Titanium Core, we'd parse this using SIMD or direct byte offsets.
        // For demonstration, we simulate checking an imbalance condition:
        if data.len > 0 && self.current_position == 0.0 {
            // Simplified logic: trigger buy if data length is even (just to generate a signal)
            if data.len % 2 == 0 {
                return Signal::BuyMarket { quantity: 1.0 };
            }
        }
        Signal::None
    }

    fn on_timer(&mut self, _frame_id: u64, _delta_ns: u64) -> Signal {
        Signal::None
    }

    fn on_fill(&mut self, report: &FillReport) -> Signal {
        self.current_position += report.executed_qty;
        Signal::None
    }

    fn reset(&mut self) {
        self.current_position = 0.0;
    }
}
