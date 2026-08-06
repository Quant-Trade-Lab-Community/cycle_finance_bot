use std::collections::BTreeMap;
use crate::order::OrderRequest;

// We use u64 representing Price * 100,000 to allow BTreeMap ordering without f64 issues.
pub type PriceStr = u64; 

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub price: f64,
    pub quantity: f64,
}

pub struct HybridOrderBook {
    pub external_bids: BTreeMap<PriceStr, f64>,
    pub external_asks: BTreeMap<PriceStr, f64>,
    pub user_bids: BTreeMap<PriceStr, f64>,
    pub user_asks: BTreeMap<PriceStr, f64>,
    pub slippage_model: String,
    pub market_impact_factor: f64,
    pub last_price: f64,
}

impl HybridOrderBook {
    pub fn new(slippage_model: String, market_impact_factor: f64) -> Self {
        Self {
            external_bids: BTreeMap::new(),
            external_asks: BTreeMap::new(),
            user_bids: BTreeMap::new(),
            user_asks: BTreeMap::new(),
            slippage_model,
            market_impact_factor,
            last_price: 0.0,
        }
    }

    pub fn to_price_key(price: f64) -> PriceStr {
        (price * 100_000.0).round() as u64
    }
    
    pub fn from_price_key(key: PriceStr) -> f64 {
        key as f64 / 100_000.0
    }

    pub fn apply_price(&mut self, price: f64) {
        self.last_price = price;
    }

    pub fn sweep_buy(&mut self, mut quantity: f64) -> Result<Vec<TradeResult>, String> {
        let mut trades = Vec::new();

        if self.slippage_model == "LINEAR_IMPACT" {
            // Basit doğrusal kayma (Slippage)
            if self.last_price == 0.0 {
                return Err("MARKET_UNAVAILABLE".to_string());
            }
            let avg_price = self.last_price * (1.0 + (quantity * self.market_impact_factor));
            trades.push(TradeResult {
                price: avg_price,
                quantity,
            });
            return Ok(trades);
        }

        // L2_SWEEP: Gerçekçi Orderbook süpürmesi
        // external_asks'den süpür
        // Note: Asks are sorted ascending. BTreeMap iterates ascending automatically.
        let mut to_remove = Vec::new();
        
        for (&price_key, &mut depth_qty) in self.external_asks.iter_mut() {
            if quantity <= 0.0 {
                break;
            }
            
            let fill_qty = if depth_qty > quantity { quantity } else { depth_qty };
            trades.push(TradeResult {
                price: Self::from_price_key(price_key),
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

        if quantity > 0.0001 { // Still has remainder
            return Err("INSUFFICIENT_DEPTH".to_string());
        }

        Ok(trades)
    }

    pub fn sweep_sell(&mut self, mut quantity: f64) -> Result<Vec<TradeResult>, String> {
        let mut trades = Vec::new();

        if self.slippage_model == "LINEAR_IMPACT" {
            if self.last_price == 0.0 {
                return Err("MARKET_UNAVAILABLE".to_string());
            }
            let avg_price = self.last_price * (1.0 - (quantity * self.market_impact_factor));
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
            if quantity <= 0.0 {
                break;
            }
            
            let fill_qty = if depth_qty > quantity { quantity } else { depth_qty };
            trades.push(TradeResult {
                price: Self::from_price_key(price_key),
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

        if quantity > 0.0001 {
            return Err("INSUFFICIENT_DEPTH".to_string());
        }

        Ok(trades)
    }
}
