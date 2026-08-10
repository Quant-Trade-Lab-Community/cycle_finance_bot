use transport::ring_buffer::MarketDataSlot;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub enum Signal {
    None,
    BuyMarket { quantity: Decimal },
    SellMarket { quantity: Decimal },
    BuyLimit { price: Decimal, quantity: Decimal },
    SellLimit { price: Decimal, quantity: Decimal },
    CancelAll,
}

#[derive(Debug, Clone)]
pub struct FillReport {
    pub order_id: String,
    pub executed_qty: Decimal,
    pub avg_price: Decimal,
}

pub trait Strategy: Send + Sync {
    fn id(&self) -> u32;
    fn on_market_data(&mut self, frame_id: u64, data: &MarketDataSlot) -> Signal;
    fn on_timer(&mut self, frame_id: u64, delta_ns: u64) -> Signal;
    fn on_fill(&mut self, report: &FillReport) -> Signal;
    fn reset(&mut self);
}
