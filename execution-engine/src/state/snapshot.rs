//! Paylaşılan hesap durumu (okuma görünümü).
//!
//! Actor tek yazıcıdır; API/strateji tüketicileri bu snapshot'ı okur.
//! `ready=false` iken emir kabul edilmez (borsa ile ilk eşitleme tamamlanmadan).

use crate::order::BinanceOrderResponse;
use crate::types::account::AccountInfo;
use crate::types::exchange::ExchangeInfo;
use crate::types::position::PositionRisk;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct AccountSnapshot {
    pub account: AccountInfo,
    pub positions: Vec<PositionRisk>,
    pub open_orders: Vec<BinanceOrderResponse>,
    pub exchange: Option<ExchangeInfo>,
    /// İlk eşitleme tamamlandı mı?
    pub ready: bool,
    pub position_mode: Option<bool>,
    pub last_update_time: u64,
    /// Her değişiklikte artar (projection doğrulaması).
    pub sequence: u64,
}

impl AccountSnapshot {
    pub fn open_position_notional(&self) -> Decimal {
        self.positions.iter().map(|p| p.notional.abs()).sum()
    }

    /// Açık emirlerin rezerve ettiği yaklaşık notional (fiyat × miktar).
    pub fn open_orders_notional(&self) -> Decimal {
        use crate::order::OrderStatus;
        self.open_orders
            .iter()
            .filter(|o| OrderStatus::from_binance(&o.status).map(|s| s.is_open()).unwrap_or(false))
            .map(|o| {
                let price = o
                    .price
                    .as_deref()
                    .and_then(|p| p.parse::<Decimal>().ok())
                    .unwrap_or(Decimal::ZERO);
                let qty = o
                    .orig_qty
                    .as_deref()
                    .and_then(|q| q.parse::<Decimal>().ok())
                    .unwrap_or(Decimal::ZERO);
                price * qty
            })
            .sum()
    }

    pub fn open_position_count(&self) -> usize {
        self.positions.iter().filter(|p| p.is_open()).count()
    }

    pub fn total_unrealized_pnl(&self) -> Decimal {
        self.account.total_unrealized_profit
    }

    pub fn available_balance(&self) -> Decimal {
        self.account.available_balance
    }

    pub fn usdt_balance(&self) -> Option<&crate::types::account::AssetBalance> {
        self.account.assets.iter().find(|a| a.asset == "USDT")
    }
}
