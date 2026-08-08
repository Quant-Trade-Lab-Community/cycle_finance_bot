//! Pozisyon yönetimi (margin/likidasyon için).
//!
//! **Boyut birimi USDT (notional)'dır.** Emirler USDT cinsinden verilir;
//! `quantity` bir pozisyonun USDT değeridir (Long pozitif, Short negatif).
//! PnL yüzde bazlıdır: `(mark/entry - 1) * |quantity|`.
//!
//! Two model destekler:
//! - **ONE_WAY**: sembol başına tek net pozisyon (Long/Short). `apply_fill` ile
//!   netleştirme ve yön değişimi yapılır.
//! - **HEDGE**: sembol başına LONG ve SHORT ayrı ayrı izlenir. `apply_fill_hedge`
//!   ile her taraf kendi içinde artar/azalır, netleştirme yapılmaz.

use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: PositionSide,
    /// USDT notional (Long pozitif, Short negatif).
    pub quantity: Decimal,
    pub avg_entry_price: Decimal,
    pub leverage: Decimal,
}

impl Position {
    /// Gerçekleşmemiş PnL (USDT): `(mark/entry - 1) * |quantity|`
    pub fn unrealized_pnl(&self, mark_price: Decimal) -> Decimal {
        let entry = self.avg_entry_price.max(Decimal::ONE);
        match self.side {
            PositionSide::Long => (mark_price - entry) / entry * self.quantity.abs(),
            PositionSide::Short => (entry - mark_price) / entry * self.quantity.abs(),
        }
    }

    /// Cari piyasa değeri (USDT): `|notional| * mark / entry`
    pub fn notional(&self, mark_price: Decimal) -> Decimal {
        self.quantity.abs() * mark_price / self.avg_entry_price.max(Decimal::ONE)
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
}

#[derive(Debug, Default)]
pub struct PositionManager {
    /// ONE_WAY: sembol → net pozisyon
    positions: HashMap<String, Position>,
    /// HEDGE: (sembol, taraf) → pozisyon
    hedge_positions: HashMap<(String, PositionSide), Position>,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            hedge_positions: HashMap::new(),
        }
    }

    /// ONE_WAY net pozisyonu.
    pub fn get(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// HEDGE taraf pozisyonu.
    pub fn get_hedge(&self, symbol: &str, side: PositionSide) -> Option<&Position> {
        self.hedge_positions.get(&(symbol.to_string(), side))
    }

    /// Moddan bağımsız: semboldeki toplam pozisyon büyüklüğü (abs).
    pub fn total_abs_qty(&self, symbol: &str) -> Decimal {
        let one_way = self.positions.get(symbol).map(|p| p.quantity.abs()).unwrap_or(Decimal::ZERO);
        let hedge: Decimal = self
            .hedge_positions
            .iter()
            .filter(|((sym, _), _)| sym == symbol)
            .map(|(_, p)| p.quantity.abs())
            .sum();
        one_way + hedge
    }

    /// Tüm açık pozisyonlar (mod fark etmeksizin).
    pub fn all(&self) -> Vec<&Position> {
        let mut out: Vec<&Position> = self.positions.values().collect();
        out.extend(self.hedge_positions.values());
        out
    }

    pub fn total_notional(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.all()
            .iter()
            .map(|pos| pos.notional(*mark_prices.get(&pos.symbol).unwrap_or(&pos.avg_entry_price)))
            .sum()
    }

    pub fn total_unrealized_pnl(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.all()
            .iter()
            .map(|pos| pos.unrealized_pnl(*mark_prices.get(&pos.symbol).unwrap_or(&pos.avg_entry_price)))
            .sum()
    }

    /// ONE_WAY emir bazında pozisyon güncelleme.
    /// `fill_qty` USDT notional'dır: `> 0` alım, `< 0` satım.
    /// Long/Short netleşmelerde kapanma yapılır.
    pub fn apply_fill(
        &mut self,
        symbol: &str,
        fill_qty: Decimal,
        fill_price: Decimal,
        leverage: Decimal,
    ) -> (Decimal, Decimal) {
        // (realized_pnl, closed_notional) döndürür
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
                // Kapatma / azaltma: realized = (fill - entry)/entry * closed_notional
                let close_qty = fill_qty.abs().min(pos.quantity.abs());
                let entry = pos.avg_entry_price.max(Decimal::ONE);
                realized = match pos.side {
                    PositionSide::Long => (fill_price - entry) / entry * close_qty,
                    PositionSide::Short => (entry - fill_price) / entry * close_qty,
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

        // Aynı yön: pozisyon büyütme (veya yeni açılış).
        // Ortalama giriş = toplam notional / toplam coin (coin = notional/entry).
        if pos.quantity == Decimal::ZERO {
            pos.side = if fill_qty > Decimal::ZERO { PositionSide::Long } else { PositionSide::Short };
            pos.quantity = fill_qty;
            pos.avg_entry_price = fill_price;
            pos.leverage = leverage;
        } else {
            let old_entry = pos.avg_entry_price.max(Decimal::ONE);
            let coins = pos.quantity.abs() / old_entry + fill_qty.abs() / fill_price.max(Decimal::ONE);
            pos.quantity += fill_qty;
            pos.avg_entry_price = pos.quantity.abs() / coins.max(Decimal::ONE);
            pos.leverage = leverage;
        }
        (realized, closed_qty)
    }

    /// HEDGE emir bazında pozisyon güncelleme.
    /// `side` emirin hedef tarafı (LONG/SHORT), `fill_qty` USDT notional'dır:
    /// - LONG taraf: alım +, satım -
    /// - SHORT taraf: satım -, alım +
    pub fn apply_fill_hedge(
        &mut self,
        symbol: &str,
        side: PositionSide,
        fill_qty: Decimal,
        fill_price: Decimal,
        leverage: Decimal,
    ) -> (Decimal, Decimal) {
        let pos = self.hedge_positions.entry((symbol.to_string(), side)).or_insert(Position {
            symbol: symbol.to_string(),
            side,
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
                let close_qty = fill_qty.abs().min(pos.quantity.abs());
                let entry = pos.avg_entry_price.max(Decimal::ONE);
                realized = match side {
                    PositionSide::Long => (fill_price - entry) / entry * close_qty,
                    PositionSide::Short => (entry - fill_price) / entry * close_qty,
                };
                closed_qty = close_qty;
                pos.quantity += fill_qty;
                if pos.quantity == Decimal::ZERO {
                    self.hedge_positions.remove(&(symbol.to_string(), side));
                    return (realized, closed_qty);
                }
                // Hedge'te ters yöne geçilmez; kapatılandan fazla emir sıfırda durdurulur.
                if (pos.quantity > Decimal::ZERO && pos.side == PositionSide::Short)
                    || (pos.quantity < Decimal::ZERO && pos.side == PositionSide::Long)
                {
                    pos.quantity = Decimal::ZERO;
                    self.hedge_positions.remove(&(symbol.to_string(), side));
                }
                return (realized, closed_qty);
            }
        }

        if pos.quantity == Decimal::ZERO {
            pos.side = side;
            pos.quantity = fill_qty;
            pos.avg_entry_price = fill_price;
            pos.leverage = leverage;
        } else {
            let old_entry = pos.avg_entry_price.max(Decimal::ONE);
            let coins = pos.quantity.abs() / old_entry + fill_qty.abs() / fill_price.max(Decimal::ONE);
            pos.quantity += fill_qty;
            pos.avg_entry_price = pos.quantity.abs() / coins.max(Decimal::ONE);
            pos.leverage = leverage;
        }
        (realized, closed_qty)
    }
}
