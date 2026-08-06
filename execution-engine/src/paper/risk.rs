//! Risk yönetimi: marj, drawdown, günlük kayıp, kaldıraç ve likidasyon.

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

use super::position::{PositionManager, PositionSide};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskStatus {
    Ok,
    MaxDrawdownBreached,
    MaxDailyLossBreached,
    MaxLeverageBreached,
    Liquidation,
}

impl RiskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskStatus::Ok => "OK",
            RiskStatus::MaxDrawdownBreached => "MAX_DRAWDOWN_BREACHED",
            RiskStatus::MaxDailyLossBreached => "MAX_DAILY_LOSS_BREACHED",
            RiskStatus::MaxLeverageBreached => "MAX_LEVERAGE_BREACHED",
            RiskStatus::Liquidation => "LIQUIDATION",
        }
    }
}

#[derive(Debug)]
pub struct RiskManager {
    pub max_position_qty: Decimal,
    pub max_leverage: Decimal,
    pub max_drawdown_pct: Decimal,
    pub max_daily_loss: Decimal,
    pub maintenance_margin_rate: Decimal,
    pub starting_equity: Decimal,
    pub peak_equity: Decimal,
    pub realized_pnl: Decimal,
    pub status: RiskStatus,
}

impl RiskManager {
    pub fn new(
        starting_equity: Decimal,
        max_position_qty: Decimal,
        max_leverage: Decimal,
        max_drawdown_pct: Decimal,
        max_daily_loss: Decimal,
    ) -> Self {
        Self {
            max_position_qty,
            max_leverage,
            max_drawdown_pct,
            max_daily_loss,
            maintenance_margin_rate: Decimal::from_str("0.005").unwrap(), // %0.5 bakım marjı
            starting_equity,
            peak_equity: starting_equity,
            realized_pnl: Decimal::ZERO,
            status: RiskStatus::Ok,
        }
    }

    /// Emir girişi öncesi pozisyon/marj/kaldıraç kontrolü.
    pub fn check_order(
        &self,
        positions: &PositionManager,
        symbol: &str,
        requested_qty: Decimal,
        price: Decimal,
        leverage: Decimal,
        cash: Decimal,
    ) -> Result<(), &'static str> {
        if self.status == RiskStatus::MaxDrawdownBreached || self.status == RiskStatus::MaxDailyLossBreached {
            return Err("Trading halted by risk status");
        }

        // Maks. pozisyon
        let existing = positions.get(symbol).map(|p| p.quantity.abs()).unwrap_or(Decimal::ZERO);
        if existing + requested_qty.abs() > self.max_position_qty {
            return Err("Max position size exceeded");
        }

        // Kaldıraç: yeni pozisyonun marj ihtiyacı
        let margin_required = (requested_qty.abs() * price) / leverage;
        if margin_required > cash {
            return Err("Insufficient margin for leverage");
        }
        if leverage > self.max_leverage {
            return Err("Leverage exceeds max");
        }

        Ok(())
    }

    /// Mark price tick'i üzerinden equity, drawdown ve likidasyon kontrolü.
    /// Likidasyon tetiklenirse ilgili sembol listesi döner.
    pub fn on_mark_tick(
        &mut self,
        positions: &PositionManager,
        mark_prices: &HashMap<String, Decimal>,
        cash: Decimal,
    ) -> Vec<String> {
        let unrealized = positions.total_unrealized_pnl(mark_prices);
        let equity = cash + unrealized;

        if equity > self.peak_equity {
            self.peak_equity = equity;
        }

        let drawdown = (self.peak_equity - equity) / self.peak_equity.max(Decimal::ONE);
        let daily_loss = self.realized_pnl + unrealized;

        if drawdown > self.max_drawdown_pct {
            self.status = RiskStatus::MaxDrawdownBreached;
        } else if daily_loss <= -self.max_daily_loss {
            self.status = RiskStatus::MaxDailyLossBreached;
        } else {
            self.status = RiskStatus::Ok;
        }

        // Per-pozisyon likidasyon kontrolü
        let mut liquidated = Vec::new();
        for (sym, pos) in positions.all() {
            let mark = *mark_prices.get(sym).unwrap_or(&pos.avg_entry_price);
            let liq_price = pos.liquidation_price(self.maintenance_margin_rate);
            let breached = match pos.side {
                PositionSide::Long => mark <= liq_price,
                PositionSide::Short => mark >= liq_price,
            };
            if breached {
                self.status = RiskStatus::Liquidation;
                liquidated.push(sym.clone());
            }
        }
        liquidated
    }

    pub fn record_realized(&mut self, pnl: Decimal) {
        self.realized_pnl += pnl;
    }

    pub fn equity(&self, positions: &PositionManager, mark_prices: &HashMap<String, Decimal>, cash: Decimal) -> Decimal {
        cash + positions.total_unrealized_pnl(mark_prices)
    }

    pub fn liquidation_price(&self, symbol: &str, positions: &PositionManager) -> Option<Decimal> {
        positions
            .get(symbol)
            .map(|p| p.liquidation_price(self.maintenance_margin_rate))
    }
}

impl std::str::FromStr for RiskStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OK" => Ok(RiskStatus::Ok),
            _ => Err(()),
        }
    }
}
