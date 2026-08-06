use std::cmp;

// Fixed-point integers for Risk Management (Zero float usage)
// Price is multiplied by 100,000, Quantity is multiplied by 1,000.

pub struct LobSimulator {
    // Array of (price, quantity) representing the book
    bids: [(u64, u64); 10],
    asks: [(u64, u64); 10],
    bid_count: usize,
    ask_count: usize,
}

impl LobSimulator {
    pub fn new() -> Self {
        Self {
            bids: [(0, 0); 10],
            asks: [(0, 0); 10],
            bid_count: 0,
            ask_count: 0,
        }
    }

    pub fn update_bids(&mut self, new_bids: &[(u64, u64)]) {
        let count = cmp::min(10, new_bids.len());
        for i in 0..count {
            self.bids[i] = new_bids[i];
        }
        self.bid_count = count;
    }

    pub fn update_asks(&mut self, new_asks: &[(u64, u64)]) {
        let count = cmp::min(10, new_asks.len());
        for i in 0..count {
            self.asks[i] = new_asks[i];
        }
        self.ask_count = count;
    }

    // Returns (Average Price * 100_000, Filled Quantity * 1000)
    pub fn simulate_buy(&self, mut qty: u64) -> (u64, u64) {
        let mut total_cost = 0;
        let mut filled_qty = 0;

        for i in 0..self.ask_count {
            if qty == 0 {
                break;
            }

            let (level_price, level_qty) = self.asks[i];
            let fill = cmp::min(qty, level_qty);
            
            total_cost += level_price * fill;
            filled_qty += fill;
            qty -= fill;
        }

        if filled_qty == 0 {
            (0, 0)
        } else {
            let avg_price = total_cost / filled_qty;
            (avg_price, filled_qty)
        }
    }

    pub fn simulate_sell(&self, mut qty: u64) -> (u64, u64) {
        let mut total_revenue = 0;
        let mut filled_qty = 0;

        for i in 0..self.bid_count {
            if qty == 0 {
                break;
            }

            let (level_price, level_qty) = self.bids[i];
            let fill = cmp::min(qty, level_qty);
            
            total_revenue += level_price * fill;
            filled_qty += fill;
            qty -= fill;
        }

        if filled_qty == 0 {
            (0, 0)
        } else {
            let avg_price = total_revenue / filled_qty;
            (avg_price, filled_qty)
        }
    }
}
