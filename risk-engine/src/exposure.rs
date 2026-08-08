//! Exposure ve konsantrasyon hesaplamaları.

use crate::accounting::Position;
use rust_decimal::Decimal;
use rust_decimal::prelude::{Signed, ToPrimitive};
use std::collections::HashMap;

/// Portföy exposure özeti.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExposureSummary {
    /// Brüt exposure: tüm pozisyonların |değer| toplamı.
    pub gross: Decimal,
    /// Net exposure: LONG - SHORT.
    pub net: Decimal,
    /// Herfindahl–Hirschman Index (brüt exposure payları üzerinden, 0..=1).
    pub hhi: f64,
    /// En büyük tek sembolün brüt payı (0..=1).
    pub max_symbol_share: f64,
}

/// Pozisyon değerlerini mark fiyatlarla hesaplar.
pub fn exposure(
    positions: &HashMap<String, Position>,
    mark_prices: &HashMap<String, Decimal>,
) -> ExposureSummary {
    let mut gross = Decimal::ZERO;
    let mut net = Decimal::ZERO;
    let mut notional_per_symbol: HashMap<String, Decimal> = HashMap::new();

    for p in positions.values() {
        let mark = mark_prices.get(&p.symbol).copied().unwrap_or(p.avg_entry_price);
        let val = p.notional(mark);
        gross += val;
        net += val * p.quantity.signum();
        *notional_per_symbol.entry(p.symbol.clone()).or_default() += val;
    }

    let g = gross.to_f64().unwrap_or(0.0);
    let mut hhi: f64 = 0.0;
    let mut max_share: f64 = 0.0;
    if g > 0.0 {
        for v in notional_per_symbol.values() {
            let share = v.to_f64().unwrap_or(0.0) / g;
            hhi += share * share;
            max_share = max_share.max(share);
        }
    }

    ExposureSummary {
        gross,
        net,
        hhi,
        max_symbol_share: max_share,
    }
}

/// Projeksiyon sonrası brüt exposure (sembol başına) — pre-trade kontrolü için.
/// `positions` mevcut pozisyonlar, `symbol_delta` bu emrin işaretli değer katkısıdır (USDT).
pub fn projected_gross_exposure(
    positions: &HashMap<String, Position>,
    mark_prices: &HashMap<String, Decimal>,
    symbol: &str,
    symbol_delta: Decimal,
) -> Decimal {
    let mut gross = Decimal::ZERO;
    for p in positions.values() {
        if p.symbol == symbol {
            continue;
        }
        let mark = mark_prices.get(&p.symbol).copied().unwrap_or(p.avg_entry_price);
        gross += p.notional(mark);
    }
    // Mevcut sembol pozisyonunun değeri + bu emrin USDT katkısı.
    let existing = positions.get(symbol).map(|p| p.quantity).unwrap_or(Decimal::ZERO);
    let mark = mark_prices.get(symbol).copied().unwrap_or(Decimal::ZERO);
    gross += existing.abs() * mark + symbol_delta.abs();
    gross
}

impl ExposureSummary {
    /// HHI sınırı aşıldı mı? `max_hhi == 0` ise kapalı sayılır.
    pub fn concentration_breached(&self, max_hhi: f64) -> bool {
        max_hhi > 0.0 && self.hhi > max_hhi
    }
}
