use crate::memory::ring_buffer::MarketDataSlot;

#[derive(Debug, Clone)]
pub enum Signal {
    None,
    BuyMarket { quantity: f64 },
    SellMarket { quantity: f64 },
    BuyLimit { price: f64, quantity: f64 },
    SellLimit { price: f64, quantity: f64 },
    CancelAll,
}

#[derive(Debug, Clone)]
pub struct FillReport {
    pub order_id: String,
    pub executed_qty: f64,
    pub avg_price: f64,
}

pub trait Strategy: Send + Sync {
    fn id(&self) -> u32;
    fn on_market_data(&mut self, frame_id: u64, data: &MarketDataSlot) -> Signal;
    fn on_timer(&mut self, frame_id: u64, delta_ns: u64) -> Signal;
    fn on_fill(&mut self, report: &FillReport) -> Signal;
    fn reset(&mut self);
}
