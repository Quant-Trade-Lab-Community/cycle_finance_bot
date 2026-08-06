//! Pozisyon yönetimi (margin/likidasyon için).
//!
//! Her pozisyon bir sembol için Long/Short olarak izlenir. Gerçekleşen PnL
//! kapatma sırasında, gerçekleşmemiş PnL mark price üzerinden hesaplanır.

use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: PositionSide,
    pub quantity: Decimal,
    pub avg_entry_price: Decimal,
    pub leverage: Decimal,
}

impl Position {
    pub fn unrealized_pnl(&self, mark_price: Decimal) -> Decimal {
        match self.side {
            PositionSide::Long => (mark_price - self.avg_entry_price) * self.quantity,
            PositionSide::Short => (self.avg_entry_price - mark_price) * self.quantity,
        }
    }

    /// Likidasyon fiyatı (basitleştirilmiş cross-margin yaklaşımı).
    /// long:  entry * (1 - 1/lev + maintenance)
    /// short: entry * (1 + 1/lev - maintenance)
    pub fn liquidation_price(&self, maintenance_margin_rate: Decimal) -> Decimal {
        let inv_lev = Decimal::ONE / self.leverage;
        match self.side {
            PositionSide::Long => {
                self.avg_entry_price * (Decimal::ONE - inv_lev + maintenance_margin_rate)
            }
            PositionSide::Short => {
                self.avg_entry_price * (Decimal::ONE + inv_lev - maintenance_margin_rate)
            }
        }
    }

    pub fn notional(&self, mark_price: Decimal) -> Decimal {
        self.quantity * mark_price
    }
}

#[derive(Debug, Default)]
pub struct PositionManager {
    positions: HashMap<String, Position>,
}

impl PositionManager {
    pub fn new() -> Self {
        Self { positions: HashMap::new() }
    }

    pub fn get(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    pub fn all(&self) -> &HashMap<String, Position> {
        &self.positions
    }

    pub fn total_notional(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.positions
            .iter()
            .map(|(sym, pos)| pos.notional(*mark_prices.get(sym).unwrap_or(&pos.avg_entry_price)))
            .sum()
    }

    pub fn total_unrealized_pnl(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.positions
            .iter()
            .map(|(sym, pos)| pos.unrealized_pnl(*mark_prices.get(sym).unwrap_or(&pos.avg_entry_price)))
            .sum()
    }

    /// Emir bazında pozisyon güncelleme.
    /// `qty > 0` alım, `qty < 0` satım. Long/Short netleşmelerde kapanma yapılır.
    pub fn apply_fill(
        &mut self,
        symbol: &str,
        fill_qty: Decimal,
        fill_price: Decimal,
        leverage: Decimal,
    ) -> (Decimal, Decimal) {
        // (realized_pnl, closed_qty) döndürür
        let pos = self.positions.entry(symbol.to_string()).or_insert(Position {
            symbol: symbol.to_string(),
            side: PositionSide::Long,
            quantity: Decimal::ZERO,
            avg_entry_price: Decimal::ZERO,
            leverage,
        });

        let mut realized = Decimal::ZERO;
        let mut closed_qty = Decimal::ZERO;

        if pos.quantity != Decimal::ZERO {
            let same_direction = (pos.quantity > Decimal::ZERO && fill_qty > Decimal::ZERO)
                || (pos.quantity < Decimal::ZERO && fill_qty < Decimal::ZERO);

            if !same_direction {
                // Kapatma / azaltma
                let close_qty = fill_qty.abs().min(pos.quantity.abs());
                realized = match pos.side {
                    PositionSide::Long => (fill_price - pos.avg_entry_price) * close_qty,
                    PositionSide::Short => (pos.avg_entry_price - fill_price) * close_qty,
                };
                closed_qty = close_qty;
                pos.quantity += fill_qty;
                if pos.quantity == Decimal::ZERO {
                    self.positions.remove(symbol);
                    return (realized, closed_qty);
                }
                // Yön değişimi (netleşme sonrası ters pozisyon): ortalama güncelle
                if (pos.quantity > Decimal::ZERO && pos.side == PositionSide::Short)
                    || (pos.quantity < Decimal::ZERO && pos.side == PositionSide::Long)
                {
                    pos.side = if pos.quantity > Decimal::ZERO { PositionSide::Long } else { PositionSide::Short };
                    pos.avg_entry_price = fill_price;
                    pos.leverage = leverage;
                }
                return (realized, closed_qty);
            }
        }

        // Aynı yön: pozisyon büyütme (veya yeni açılış)
        if pos.quantity == Decimal::ZERO {
            pos.side = if fill_qty > Decimal::ZERO { PositionSide::Long } else { PositionSide::Short };
            pos.quantity = fill_qty;
            pos.avg_entry_price = fill_price;
            pos.leverage = leverage;
        } else {
            let total_value = (pos.quantity.abs() * pos.avg_entry_price) + (fill_qty.abs() * fill_price);
            pos.quantity += fill_qty;
            pos.avg_entry_price = total_value / pos.quantity.abs();
            pos.leverage = leverage;
        }
        (realized, closed_qty)
    }
}
