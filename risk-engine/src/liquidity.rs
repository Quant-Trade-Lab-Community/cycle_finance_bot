//! Likidite modeli: order book seviyeleri üzerinden slippage / market impact.
//!
//! Fiyatlar sabit nokta (×100_000) tutulur, taşma riski olmadan tamsayı aritmetiği
//! yapılır (orijinal `lob_simulator.rs` yaklaşımı korunur).

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::cmp;

/// Fiyat ölçeği: 100_000 (1.00000).
const PRICE_SCALE: u64 = 100_000;
/// Miktar ölçeği: 1_000.
const QTY_SCALE: u64 = 1_000;

/// Sabit boyutlu order book (ilk 10 seviye).
#[derive(Debug, Clone)]
pub struct LobSimulator {
    /// (fiyat×100k, miktar×1k) — best bid'den geriye doğru.
    bids: [(u64, u64); 10],
    /// (fiyat×100k, miktar×1k) — best ask'ten geriye doğru.
    asks: [(u64, u64); 10],
    bid_count: usize,
    ask_count: usize,
}

impl Default for LobSimulator {
    fn default() -> Self {
        Self::new()
    }
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

    #[allow(clippy::needless_range_loop)]
    pub fn update_bids(&mut self, levels: &[(Decimal, Decimal)]) {
        let count = cmp::min(10, levels.len());
        for i in 0..count {
            let (p, q) = levels[i];
            self.bids[i] = (to_scale(p, PRICE_SCALE), to_scale(q, QTY_SCALE));
        }
        self.bid_count = count;
    }

    #[allow(clippy::needless_range_loop)]
    pub fn update_asks(&mut self, levels: &[(Decimal, Decimal)]) {
        let count = cmp::min(10, levels.len());
        for i in 0..count {
            let (p, q) = levels[i];
            self.asks[i] = (to_scale(p, PRICE_SCALE), to_scale(q, QTY_SCALE));
        }
        self.ask_count = count;
    }

    /// Piyasa alış simülasyonu: (ortalama fiyat×100k, dolu miktar×1k).
    pub fn simulate_buy(&self, mut qty: u64) -> (u64, u64) {
        let mut total_cost = 0u128;
        let mut filled = 0u64;
        for i in 0..self.ask_count {
            if qty == 0 {
                break;
            }
            let (p, q) = self.asks[i];
            if q == 0 {
                continue;
            }
            let fill = cmp::min(qty, q);
            total_cost += (p as u128) * (fill as u128);
            filled += fill;
            qty -= fill;
        }
        if filled == 0 {
            (0, 0)
        } else {
            ( (total_cost / filled as u128) as u64, filled)
        }
    }

    /// Piyasa satış simülasyonu: (ortalama fiyat×100k, dolu miktar×1k).
    pub fn simulate_sell(&self, mut qty: u64) -> (u64, u64) {
        let mut total_revenue = 0u128;
        let mut filled = 0u64;
        for i in 0..self.bid_count {
            if qty == 0 {
                break;
            }
            let (p, q) = self.bids[i];
            if q == 0 {
                continue;
            }
            let fill = cmp::min(qty, q);
            total_revenue += (p as u128) * (fill as u128);
            filled += fill;
            qty -= fill;
        }
        if filled == 0 {
            (0, 0)
        } else {
            ((total_revenue / filled as u128) as u64, filled)
        }
    }

    pub fn bid_count(&self) -> usize {
        self.bid_count
    }

    pub fn ask_count(&self) -> usize {
        self.ask_count
    }
}

/// Belirli bir emir için tahmini slippage'i baz puan (bps) cinsinden döndürür.
/// Sembol bilgisi için `LiquidityEngine` kullanılır (aşağıda).
pub fn estimate_slippage_bps(book: &LobSimulator, side: Side, qty: Decimal) -> Option<Decimal> {
    let mid = mid_price(book)?;
    if mid <= 0.0 {
        return None;
    }
    let qty_scaled = to_scale(qty, QTY_SCALE);
    let (avg, filled) = match side {
        Side::Buy => book.simulate_buy(qty_scaled),
        Side::Sell => book.simulate_sell(qty_scaled),
    };
    if filled == 0 || avg == 0 {
        return None;
    }
    // Yalnızca tamamen doldurulabildiyse slippage anlamlıdır.
    if filled < qty_scaled {
        return None;
    }
    let avg_f64 = avg as f64 / PRICE_SCALE as f64;
    let slippage = (avg_f64 / mid - 1.0).abs();
    Some(Decimal::from_f64_retain(slippage * 10_000.0).unwrap_or_default())
}

fn mid_price(book: &LobSimulator) -> Option<f64> {
    if book.ask_count == 0 || book.bid_count == 0 {
        return None;
    }
    let best_bid = book.bids[0].0 as f64 / PRICE_SCALE as f64;
    let best_ask = book.asks[0].0 as f64 / PRICE_SCALE as f64;
    if best_bid <= 0.0 || best_ask <= 0.0 {
        return None;
    }
    Some((best_bid + best_ask) / 2.0)
}

fn to_scale(v: Decimal, scale: u64) -> u64 {
    (v * Decimal::from(scale)).round().to_u64().unwrap_or(0)
}

use crate::types::Side;
