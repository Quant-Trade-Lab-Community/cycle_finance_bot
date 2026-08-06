use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub price: Decimal,
    pub quantity: Decimal,
}

pub struct HybridOrderBook {
    pub external_bids: BTreeMap<Decimal, Decimal>,
    pub external_asks: BTreeMap<Decimal, Decimal>,
    pub user_bids: BTreeMap<Decimal, Decimal>,
    pub user_asks: BTreeMap<Decimal, Decimal>,
    pub slippage_model: String,
    pub market_impact_factor: Decimal,
    pub last_price: Decimal,
}

impl HybridOrderBook {
    pub fn new(slippage_model: String, market_impact_factor: Decimal) -> Self {
        Self {
            external_bids: BTreeMap::new(),
            external_asks: BTreeMap::new(),
            user_bids: BTreeMap::new(),
            user_asks: BTreeMap::new(),
            slippage_model,
            market_impact_factor,
            last_price: Decimal::ZERO,
        }
    }

    pub fn apply_price(&mut self, price: Decimal) {
        self.last_price = price;
    }

    pub fn sweep_buy(&mut self, mut quantity: Decimal) -> Result<Vec<TradeResult>, String> {
        let mut trades = Vec::new();

        if self.slippage_model == "LINEAR_IMPACT" {
            // Basit doğrusal kayma (Slippage)
            if self.last_price == Decimal::ZERO {
                return Err("MARKET_UNAVAILABLE".to_string());
            }
            let avg_price = self.last_price * (Decimal::ONE + (quantity * self.market_impact_factor));
            trades.push(TradeResult {
                price: avg_price,
                quantity,
            });
            return Ok(trades);
        }

        // L2_SWEEP: Gerçekçi Orderbook süpürmesi
        // Asks are sorted ascending. BTreeMap iterates ascending automatically.
        let mut to_remove = Vec::new();

        for (&price_key, &mut depth_qty) in self.external_asks.iter_mut() {
            if quantity <= Decimal::ZERO {
                break;
            }

            let fill_qty = if depth_qty > quantity { quantity } else { depth_qty };
            trades.push(TradeResult {
                price: price_key,
                quantity: fill_qty,
            });

            quantity -= fill_qty;
            if depth_qty <= fill_qty {
                to_remove.push(price_key);
            }
        }

        // Remove empty depth levels
        for key in to_remove {
            self.external_asks.remove(&key);
        }

        if quantity > Decimal::from_str("0.0001").unwrap() {
            return Err("INSUFFICIENT_DEPTH".to_string());
        }

        Ok(trades)
    }

    pub fn sweep_sell(&mut self, mut quantity: Decimal) -> Result<Vec<TradeResult>, String> {
        let mut trades = Vec::new();

        if self.slippage_model == "LINEAR_IMPACT" {
            if self.last_price == Decimal::ZERO {
                return Err("MARKET_UNAVAILABLE".to_string());
            }
            let avg_price = self.last_price * (Decimal::ONE - (quantity * self.market_impact_factor));
            trades.push(TradeResult {
                price: avg_price,
                quantity,
            });
            return Ok(trades);
        }

        // L2_SWEEP for Sells. Bids need to be sorted descending (highest bid first).
        // BTreeMap iterates ascending, so we use .rev()
        let mut to_remove = Vec::new();

        for (&price_key, &mut depth_qty) in self.external_bids.iter_mut().rev() {
            if quantity <= Decimal::ZERO {
                break;
            }

            let fill_qty = if depth_qty > quantity { quantity } else { depth_qty };
            trades.push(TradeResult {
                price: price_key,
                quantity: fill_qty,
            });

            quantity -= fill_qty;
            if depth_qty <= fill_qty {
                to_remove.push(price_key);
            }
        }

        for key in to_remove {
            self.external_bids.remove(&key);
        }

        if quantity > Decimal::from_str("0.0001").unwrap() {
            return Err("INSUFFICIENT_DEPTH".to_string());
        }

        Ok(trades)
    }
}
