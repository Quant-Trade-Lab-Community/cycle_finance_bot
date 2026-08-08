#![allow(clippy::too_many_arguments)]

//! API/CLI okumaları için paylaşılan durum snapshot'ı.
//!
//! Yazma işlemleri actor task'ında sıralıdır; okuma istekleri bu snapshot'ı okur.

use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;

use super::actor::MarginType;
use super::domain_event::DomainEvent;
use super::position::{PositionManager, PositionSide};
use super::risk::RiskStatus;

#[derive(Debug, Clone, Serialize)]
pub struct PositionView {
    pub symbol: String,
    pub side: String,
    pub quantity: Decimal,
    pub avg_entry_price: Decimal,
    pub leverage: Decimal,
    pub liquidation_price: Option<Decimal>,
    pub mark_price: Option<Decimal>,
    /// Gerçekleşmemiş PnL (mark price - entry) * qty
    pub unrealized_pnl: Option<Decimal>,
    /// PnL yüzdesi (girişe göre)
    pub unrealized_pnl_pct: Option<Decimal>,
    pub margin_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeView {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub fee: Decimal,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaperSnapshot {
    pub cash_balance: Decimal,
    pub equity: Decimal,
    pub realized_pnl: Decimal,
    pub total_commission: Decimal,
    pub risk_status: String,
    pub last_price: Decimal,
    pub position_mode: String,
    pub positions: Vec<PositionView>,
    pub open_orders: usize,
    pub recent_trades: Vec<TradeView>,
}

#[derive(Debug, Default)]
pub struct SnapshotBuilder {
    pub recent_trades: Vec<TradeView>,
}

impl PaperSnapshot {
    pub fn build(
        cash: Decimal,
        equity: Decimal,
        realized_pnl: Decimal,
        commission: Decimal,
        risk_status: RiskStatus,
        last_price: Decimal,
        positions: &PositionManager,
        open_orders: usize,
        recent_trades: Vec<TradeView>,
        mark_prices: &std::collections::HashMap<String, Decimal>,
        position_mode: String,
        margin_types: &std::collections::HashMap<String, MarginType>,
    ) -> Self {
        let positions = positions
            .all()
            .into_iter()
            .map(|pos| {
                let mark = mark_prices.get(&pos.symbol).copied();
                let unrealized = mark.map(|m| pos.unrealized_pnl(m));
                let unrealized_pnl_pct = unrealized.map(|up| {
                    let cost = pos.quantity.abs().max(Decimal::ONE);
                    (up / cost) * Decimal::ONE_HUNDRED
                });
                let margin_type = margin_types.get(&pos.symbol)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| "CROSSED".to_string());
                PositionView {
                    symbol: pos.symbol.clone(),
                    side: match pos.side {
                        PositionSide::Long => "LONG".to_string(),
                        PositionSide::Short => "SHORT".to_string(),
                    },
                    quantity: pos.quantity,
                    avg_entry_price: pos.avg_entry_price,
                    leverage: pos.leverage,
                    liquidation_price: Some(pos.liquidation_price(Decimal::from_str("0.005").unwrap_or(Decimal::ZERO))),
                    mark_price: mark,
                    unrealized_pnl: unrealized,
                    unrealized_pnl_pct,
                    margin_type,
                }
            })
            .collect();

        Self {
            cash_balance: cash,
            equity,
            realized_pnl,
            total_commission: commission,
            risk_status: risk_status.as_str().to_string(),
            last_price,
            position_mode,
            positions,
            open_orders,
            recent_trades,
        }
    }
}

impl From<&DomainEvent> for TradeView {
    fn from(ev: &DomainEvent) -> Self {
        match ev {
            DomainEvent::OrderFilled { order_id, symbol, side, fill_price, fill_qty, commission, .. } => TradeView {
                order_id: order_id.clone(),
                symbol: symbol.clone(),
                side: side.clone(),
                price: *fill_price,
                quantity: *fill_qty,
                fee: *commission,
                timestamp: 0,
            },
            DomainEvent::Liquidation { symbol, side, price, qty, .. } => TradeView {
                order_id: format!("LIQ_{symbol}"),
                symbol: symbol.clone(),
                side: side.clone(),
                price: *price,
                quantity: *qty,
                fee: Decimal::ZERO,
                timestamp: 0,
            },
            _ => TradeView {
                order_id: String::new(),
                symbol: String::new(),
                side: String::new(),
                price: Decimal::ZERO,
                quantity: Decimal::ZERO,
                fee: Decimal::ZERO,
                timestamp: 0,
            },
        }
    }
}
