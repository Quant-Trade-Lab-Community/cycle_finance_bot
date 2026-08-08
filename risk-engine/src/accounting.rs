//! Muhasebe: pozisyon yönetimi, fill işleme, gerçekleşen/gerçekleşmemiş PnL.
//!
//! **Birimler:** miktarlar baz-coin, fiyatlar USDT, değerler USDT'dir.
//! Pozisyon `quantity`'si işaretlidir: `>0` LONG, `<0` SHORT.

use crate::types::{Fill, Side};
use rust_decimal::Decimal;
use rust_decimal::prelude::Signed;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    /// İşaretli net miktar (coin): `>0` LONG, `<0` SHORT.
    pub quantity: Decimal,
    pub avg_entry_price: Decimal,
    pub leverage: Decimal,
}

impl Position {
    pub fn is_open(&self) -> bool {
        !self.quantity.is_zero()
    }

    pub fn is_long(&self) -> bool {
        self.quantity > Decimal::ZERO
    }

    /// Gerçekleşmemiş PnL (USDT).
    pub fn unrealized_pnl(&self, mark_price: Decimal) -> Decimal {
        let entry = self.avg_entry_price.max(Decimal::ONE);
        let qty = self.quantity;
        if self.is_long() {
            (mark_price - entry) * qty
        } else {
            (entry - mark_price) * qty.abs()
        }
    }

    /// Cari pozisyon değeri (USDT): `|qty| * mark`.
    pub fn notional(&self, mark_price: Decimal) -> Decimal {
        self.quantity.abs() * mark_price
    }

    /// Likidasyon fiyatı (basitleştirilmiş cross-margin yaklaşımı).
    /// long:  entry * (1 - 1/lev + maintenance)
    /// short: entry * (1 + 1/lev - maintenance)
    pub fn liquidation_price(&self, maintenance_margin_rate: Decimal) -> Decimal {
        let inv_lev = Decimal::ONE / self.leverage.max(Decimal::ONE);
        if self.is_long() {
            self.avg_entry_price * (Decimal::ONE - inv_lev + maintenance_margin_rate)
        } else {
            self.avg_entry_price * (Decimal::ONE + inv_lev - maintenance_margin_rate)
        }
    }

    /// Mark fiyat likidasyon çizgisini geçti mi?
    pub fn liquidation_breached(&self, mark_price: Decimal, maintenance_margin_rate: Decimal) -> bool {
        let liq = self.liquidation_price(maintenance_margin_rate);
        if self.is_long() {
            mark_price <= liq
        } else {
            mark_price >= liq
        }
    }
}

/// Portföy muhasebesi — tek doğruluk kaynağı.
#[derive(Debug, Clone)]
pub struct Portfolio {
    pub cash_balance: Decimal,
    pub starting_balance: Decimal,
    pub realized_pnl: Decimal,
    pub total_commission: Decimal,
    pub positions: HashMap<String, Position>,
    pub max_drawdown_limit: Decimal,
    pub peak_equity: Decimal,
    /// Gün içinde gerçekleşen PnL (yeni UTC gününde sıfırlanır).
    pub realized_today: Decimal,
    /// Gün sınırı takibi için son UTC gün numarası (Unix gün sayacı).
    pub day_index: i64,
    pub maintenance_margin_rate: Decimal,
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
            peak_equity: initial_balance,
            realized_today: Decimal::ZERO,
            day_index: Self::utc_day_index(0),
            maintenance_margin_rate: Decimal::from_str("0.005").unwrap(),
        }
    }

    /// Eski `RiskEngine` sınır değerleriyle uyumlu kurucu.
    pub fn new_with_margin(initial_balance: Decimal, max_drawdown: Decimal, maintenance_margin_rate: Decimal) -> Self {
        let mut p = Self::new(initial_balance, max_drawdown);
        p.maintenance_margin_rate = maintenance_margin_rate;
        p
    }

    /// Unix ts'den UTC gün sayacı (0 döner → bilinmiyor).
    fn utc_day_index(ts_ms: u64) -> i64 {
        let secs = ts_ms / 1000;
        (secs / 86_400) as i64
    }

    /// Yeni UTC günü başladıysa günlük PnL sayacını sıfırlar.
    pub fn roll_day(&mut self, ts_ms: u64) {
        let idx = Self::utc_day_index(ts_ms);
        if idx > 0 && idx != self.day_index {
            self.day_index = idx;
            self.realized_today = Decimal::ZERO;
        }
    }

    /// İşaretli fill (pozitif alım, negatif satım) — ONE_WAY netleştirme.
    /// `commission` USDT'dir. Gerçekleşen PnL (komisyonsuz) döner.
    pub fn process_fill(&mut self, symbol: &str, fill_qty: Decimal, fill_price: Decimal, commission: Decimal) -> Decimal {
        let leverage = self
            .positions
            .get(symbol)
            .map(|p| p.leverage)
            .unwrap_or(Decimal::ONE);

        let fill = Fill {
            symbol: symbol.to_string(),
            side: if fill_qty >= Decimal::ZERO { Side::Buy } else { Side::Sell },
            quantity: fill_qty.abs(),
            price: fill_price,
            commission,
            leverage,
            ts_ms: 0,
        };
        self.apply_fill(&fill)
    }

    /// Yapılandırılmış fill işleme (komisyon + gerçekleşen PnL + pozisyon).
    /// Gerçekleşen PnL (komisyonsuz) döndürür.
    pub fn apply_fill(&mut self, fill: &Fill) -> Decimal {
        self.cash_balance -= fill.commission;
        self.total_commission += fill.commission;

        let signed = match fill.side {
            Side::Buy => fill.quantity,
            Side::Sell => -fill.quantity,
        };
        self.roll_day(fill.ts_ms);

        let realized = self.apply_signed(symbol_key(&fill.symbol), signed, fill.price, fill.leverage);
        self.realized_pnl += realized;
        self.realized_today += realized;
        realized
    }

    fn apply_signed(&mut self, symbol: String, signed: Decimal, fill_price: Decimal, leverage: Decimal) -> Decimal {
        let mut realized = Decimal::ZERO;
        let mut closed = false;
        let mut zeroed = false;

        {
            let pos = self
                .positions
                .entry(symbol.clone())
                .or_insert(Position {
                    symbol: symbol.clone(),
                    quantity: Decimal::ZERO,
                    avg_entry_price: Decimal::ZERO,
                    leverage,
                });

            if !pos.quantity.is_zero() {
                let same_direction = (pos.quantity > Decimal::ZERO && signed > Decimal::ZERO)
                    || (pos.quantity < Decimal::ZERO && signed < Decimal::ZERO);

                if !same_direction {
                    // Kapatma / azaltma.
                    let was_long = pos.is_long();
                    let close_qty = signed.abs().min(pos.quantity.abs());
                    let entry = pos.avg_entry_price.max(Decimal::ONE);
                    realized = if was_long {
                        (fill_price - entry) * close_qty
                    } else {
                        (entry - fill_price) * close_qty
                    };

                    pos.quantity += signed;
                    closed = true;
                    if pos.quantity.is_zero() {
                        zeroed = true;
                    } else if pos.is_long() != was_long {
                        // Yön değişimi: ters pozisyona döndü → yeni giriş.
                        pos.avg_entry_price = fill_price;
                        pos.leverage = leverage;
                    }
                }
            }

            if !closed {
                // Aynı yön (veya yeni açılış): ağırlıklı ortalama giriş.
                if pos.quantity.is_zero() {
                    pos.quantity = signed;
                    pos.avg_entry_price = fill_price;
                    pos.leverage = leverage;
                } else {
                    let old_entry = pos.avg_entry_price.max(Decimal::ONE);
                    let total_cost = pos.quantity.abs() * old_entry + signed.abs() * fill_price;
                    let total_qty = pos.quantity.abs() + signed.abs();
                    pos.quantity += signed;
                    pos.avg_entry_price = total_cost / total_qty.max(Decimal::ONE);
                    pos.leverage = leverage;
                }
            }
        }

        if zeroed {
            self.positions.remove(&symbol);
        }
        realized
    }

    // ── Değerleme ──

    pub fn unrealized_pnl(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.positions
            .values()
            .map(|p| {
                let mark = mark_prices.get(&p.symbol).copied().unwrap_or(p.avg_entry_price);
                p.unrealized_pnl(mark)
            })
            .sum()
    }

    pub fn total_notional(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.positions
            .values()
            .map(|p| {
                let mark = mark_prices.get(&p.symbol).copied().unwrap_or(p.avg_entry_price);
                p.notional(mark)
            })
            .sum()
    }

    /// Eşitlik = nakit + gerçekleşmemiş PnL.
    pub fn get_total_equity(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.cash_balance + self.unrealized_pnl(mark_prices)
    }

    /// Drawdown oranı (0.10 = %10). `peak_equity` güncellenmez (salt okuma).
    pub fn drawdown_pct(&self, equity: Decimal) -> Decimal {
        let peak = self.peak_equity.max(Decimal::ONE);
        (peak - equity) / peak
    }

    /// Günlük kayıp: bugün gerçekleşen + tüm gerçekleşmemiş.
    pub fn daily_loss(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.realized_today + self.unrealized_pnl(mark_prices)
    }

    pub fn is_drawdown_exceeded(&self, mark_prices: &HashMap<String, Decimal>) -> bool {
        let equity = self.get_total_equity(mark_prices);
        self.drawdown_pct(equity) > self.max_drawdown_limit
    }

    pub fn gross_exposure(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.total_notional(mark_prices)
    }

    /// Net exposure = LONG - SHORT değerleri toplamı.
    pub fn net_exposure(&self, mark_prices: &HashMap<String, Decimal>) -> Decimal {
        self.positions
            .values()
            .map(|p| {
                let mark = mark_prices.get(&p.symbol).copied().unwrap_or(p.avg_entry_price);
                p.notional(mark) * p.quantity.signum()
            })
            .sum()
    }

    /// Likidasyon yakınlığı: likidasyon çizgisine yaklaşan semboller.
    pub fn near_liquidation(&self, mark_prices: &HashMap<String, Decimal>, proximity_pct: Decimal) -> Vec<String> {
        let mut out = Vec::new();
        for p in self.positions.values() {
            let mark = mark_prices.get(&p.symbol).copied().unwrap_or(p.avg_entry_price);
            let liq = p.liquidation_price(self.maintenance_margin_rate);
            if p.is_long() {
                if mark - liq <= liq * proximity_pct {
                    out.push(p.symbol.clone());
                }
            } else if liq - mark <= liq * proximity_pct {
                out.push(p.symbol.clone());
            }
        }
        out
    }

    /// Equity'yi peak olarak işaretler; yeni peak gördüyse günceller.
    pub fn update_peak(&mut self, equity: Decimal) {
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }
    }
}

fn symbol_key(s: &str) -> String {
    s.to_uppercase()
}
