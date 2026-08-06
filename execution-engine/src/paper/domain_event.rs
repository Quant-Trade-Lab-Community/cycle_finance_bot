//! Paper sisteminin domain event'leri (Event Sourcing).
//!
//! State'i değiştiren her aksiyon bir event olarak üretilir ve event store'a
//! yazılır. Çökme durumunda olaylar replay edilerek son duruma ulaşılır.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    OrderCreated {
        order_id: String,
        client_oid: String,
        symbol: String,
        side: String,
        order_type: String,
        qty: Decimal,
        price: Option<Decimal>,
    },
    OrderFilled {
        order_id: String,
        symbol: String,
        side: String,
        fill_price: Decimal,
        fill_qty: Decimal,
        commission: Decimal,
        /// Net nakit etkisi (marj açılışı/kapanışı + komisyon + realized PnL dahil)
        cash_delta: Decimal,
        realized_pnl: Decimal,
        leverage: Decimal,
    },
    OrderCancelled {
        order_id: String,
        reason: String,
    },
    PositionOpened {
        symbol: String,
        side: String,
        qty: Decimal,
        entry_price: Decimal,
        leverage: Decimal,
    },
    PositionClosed {
        symbol: String,
        realized_pnl: Decimal,
    },
    Liquidation {
        symbol: String,
        side: String,
        price: Decimal,
        qty: Decimal,
    },
    FundingRateApplied {
        symbol: String,
        rate: Decimal,
        payment: Decimal,
    },
}
