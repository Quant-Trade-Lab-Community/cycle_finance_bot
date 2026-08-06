use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
}

impl Position {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            quantity: 0.0,
            avg_entry_price: 0.0,
        }
    }

    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        if self.quantity == 0.0 {
            return 0.0;
        }
        (current_price - self.avg_entry_price) * self.quantity
    }
}

pub struct Portfolio {
    pub cash_balance: f64,
    pub realized_pnl: f64,
    pub total_commission: f64,
    pub positions: HashMap<String, Position>,
    pub max_drawdown_limit: f64,
    pub starting_balance: f64,
}

impl Portfolio {
    pub fn new(initial_balance: f64, max_drawdown: f64) -> Self {
        Self {
            cash_balance: initial_balance,
            starting_balance: initial_balance,
            realized_pnl: 0.0,
            total_commission: 0.0,
            positions: HashMap::new(),
            max_drawdown_limit: max_drawdown,
        }
    }

    pub fn process_fill(&mut self, symbol: &str, fill_qty: f64, fill_price: f64, commission: f64) {
        self.cash_balance -= commission;
        self.total_commission += commission;

        let pos = self.positions.entry(symbol.to_string()).or_insert_with(|| Position::new(symbol.to_string()));

        // Check if we are closing a position (signs are opposite)
        if (pos.quantity > 0.0 && fill_qty < 0.0) || (pos.quantity < 0.0 && fill_qty > 0.0) {
            let close_qty = fill_qty.abs().min(pos.quantity.abs());
            let realized = (fill_price - pos.avg_entry_price) * close_qty * pos.quantity.signum();
            self.realized_pnl += realized;
            self.cash_balance += realized; // Add realized to cash
            
            // Adjust position
            pos.quantity += fill_qty;
            if pos.quantity == 0.0 {
                pos.avg_entry_price = 0.0;
            }
        } else {
            // Opening or adding to position
            let total_value = (pos.quantity.abs() * pos.avg_entry_price) + (fill_qty.abs() * fill_price);
            pos.quantity += fill_qty;
            if pos.quantity != 0.0 {
                pos.avg_entry_price = total_value / pos.quantity.abs();
            }
        }
    }

    pub fn get_total_equity(&self, current_prices: &HashMap<String, f64>) -> f64 {
        let mut un_pnl = 0.0;
        for (sym, pos) in &self.positions {
            if let Some(price) = current_prices.get(sym) {
                un_pnl += pos.unrealized_pnl(*price);
            }
        }
        self.cash_balance + un_pnl
    }

    pub fn is_drawdown_exceeded(&self, current_prices: &HashMap<String, f64>) -> bool {
        let equity = self.get_total_equity(current_prices);
        let drawdown = (self.starting_balance - equity) / self.starting_balance;
        drawdown > self.max_drawdown_limit
    }
}
