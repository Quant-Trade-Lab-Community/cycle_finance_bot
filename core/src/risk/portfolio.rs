use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: Decimal,
    pub avg_entry_price: Decimal,
}

impl Position {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            quantity: Decimal::ZERO,
            avg_entry_price: Decimal::ZERO,
        }
    }

    pub fn unrealized_pnl(&self, current_price: Decimal) -> Decimal {
        if self.quantity == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (current_price - self.avg_entry_price) * self.quantity
    }
}

pub struct Portfolio {
    pub cash_balance: Decimal,
    pub realized_pnl: Decimal,
    pub total_commission: Decimal,
    pub positions: HashMap<String, Position>,
    pub max_drawdown_limit: Decimal,
    pub starting_balance: Decimal,
}

impl Portfolio {
    pub fn new(initial_balance: Decimal, max_drawdown: Decimal) -> Self {
        Self {
            cash_balance: initial_balance,
            starting_balance: initial_balance,
            realized_pnl: Decimal::ZERO,
            total_commission: Decimal::ZERO,
            positions: HashMap::new(),
            max_drawdown_limit: max_drawdown,
        }
    }

    pub fn process_fill(&mut self, symbol: &str, fill_qty: Decimal, fill_price: Decimal, commission: Decimal) {
        self.cash_balance -= commission;
        self.total_commission += commission;

        let pos = self.positions.entry(symbol.to_string()).or_insert_with(|| Position::new(symbol.to_string()));

        // Check if we are closing a position (signs are opposite)
        if (pos.quantity > Decimal::ZERO && fill_qty < Decimal::ZERO) || (pos.quantity < Decimal::ZERO && fill_qty > Decimal::ZERO) {
            let close_qty = fill_qty.abs().min(pos.quantity.abs());
            let realized = (fill_price - pos.avg_entry_price) * close_qty * pos.quantity.signum();
            self.realized_pnl += realized;
            self.cash_balance += realized; // Add realized to cash

            // Adjust position
            pos.quantity += fill_qty;
            if pos.quantity == Decimal::ZERO {
                pos.avg_entry_price = Decimal::ZERO;
            }
        } else {
            // Opening or adding to position
            let total_value = (pos.quantity.abs() * pos.avg_entry_price) + (fill_qty.abs() * fill_price);
            pos.quantity += fill_qty;
            if pos.quantity != Decimal::ZERO {
                pos.avg_entry_price = total_value / pos.quantity.abs();
            }
        }
    }

    pub fn get_total_equity(&self, current_prices: &HashMap<String, Decimal>) -> Decimal {
        let mut un_pnl = Decimal::ZERO;
        for (sym, pos) in &self.positions {
            if let Some(price) = current_prices.get(sym) {
                un_pnl += pos.unrealized_pnl(*price);
            }
        }
        self.cash_balance + un_pnl
    }

    pub fn is_drawdown_exceeded(&self, current_prices: &HashMap<String, Decimal>) -> bool {
        let equity = self.get_total_equity(current_prices);
        let drawdown = (self.starting_balance - equity) / self.starting_balance;
        drawdown > self.max_drawdown_limit
    }
}
