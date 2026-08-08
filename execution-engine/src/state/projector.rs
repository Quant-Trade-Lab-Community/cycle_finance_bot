//! User-data stream olaylarını paylaşılan snapshot'a uygular.
//!
//! Actor tek yazıcıdır; bu modül saf fonksiyonlar sunar (test edilebilir).
//! Deltalar borsanın gerçeğidir; periyodik uzlaştırma tam doğruluğu garantiler.

use crate::state::snapshot::AccountSnapshot;
use crate::types::account::{AccountPosition, AssetBalance};
use crate::types::position::PositionRisk;
use crate::types::user_event::{AccountUpdatePosition, OrderUpdate, UserDataEvent};
use rust_decimal::Decimal;

pub fn apply(snap: &mut AccountSnapshot, event: &UserDataEvent) {
    match event {
        UserDataEvent::AccountUpdate {
            balances,
            positions,
            update_time,
            ..
        } => {
            for b in balances {
                upsert_balance(snap, b.asset.clone(), b.wallet_balance, b.cross_wallet_balance);
            }
            for p in positions {
                upsert_position(snap, p);
            }
            if *update_time > snap.last_update_time {
                snap.last_update_time = *update_time;
            }
            snap.sequence += 1;
        }
        UserDataEvent::OrderTradeUpdate { order, transaction_time, .. } => {
            sync_open_orders(snap, order);
            let is_trade = order.execution_type == "TRADE" && order.last_filled_qty != Decimal::ZERO;
            if is_trade {
                apply_fill(snap, order);
            }
            if *transaction_time > snap.last_update_time {
                snap.last_update_time = *transaction_time;
            }
            snap.sequence += 1;
        }
        UserDataEvent::AccountConfigUpdate {
            symbol,
            leverage,
            margin_type,
            dual_side_position,
            ..
        } => {
            if let Some(dual) = dual_side_position {
                snap.position_mode = Some(*dual);
            }
            if let (Some(s), Some(lev)) = (symbol, leverage) {
                for p in snap.positions.iter_mut() {
                    if &p.symbol == s {
                        p.leverage = Decimal::from(*lev);
                    }
                }
            }
            if let (Some(s), Some(mt)) = (symbol, margin_type) {
                for p in snap.positions.iter_mut() {
                    if &p.symbol == s {
                        p.margin_type = mt.clone();
                    }
                }
            }
            snap.sequence += 1;
        }
        UserDataEvent::MarginCall { .. }
        | UserDataEvent::ListenKeyExpired { .. }
        | UserDataEvent::Unknown { .. } => {}
    }
}

fn upsert_balance(snap: &mut AccountSnapshot, asset: String, wallet: Decimal, cross_wallet: Decimal) {
    if let Some(b) = snap.account.assets.iter_mut().find(|b| b.asset == asset) {
        b.wallet_balance = wallet;
        b.cross_wallet_balance = cross_wallet;
    } else {
        snap.account.assets.push(AssetBalance {
            asset,
            wallet_balance: wallet,
            cross_wallet_balance: cross_wallet,
            ..Default::default()
        });
    }
    snap.account.total_wallet_balance = snap.account.assets.iter().map(|a| a.wallet_balance).sum();
}

fn upsert_position(snap: &mut AccountSnapshot, p: &AccountUpdatePosition) {
    let idx = snap
        .positions
        .iter()
        .position(|x| x.symbol == p.symbol && x.position_side == p.position_side);

    match idx {
        Some(i) => {
            snap.positions[i].position_amt = p.position_amt;
            snap.positions[i].entry_price = p.entry_price;
            snap.positions[i].un_realized_profit = p.un_realized_profit;
            snap.positions[i].isolated_wallet = p.isolated_wallet;
            snap.positions[i].margin_type = margin_type_str(&p.margin_type);
            snap.positions[i].isolated_margin = p.isolated_wallet;
            snap.positions[i].notional = p.position_amt * p.entry_price;
        }
        None => snap.positions.push(PositionRisk {
            symbol: p.symbol.clone(),
            position_side: p.position_side.clone(),
            position_amt: p.position_amt,
            entry_price: p.entry_price,
            mark_price: p.entry_price,
            un_realized_profit: p.un_realized_profit,
            margin_type: margin_type_str(&p.margin_type),
            isolated_margin: p.isolated_wallet,
            isolated_wallet: p.isolated_wallet,
            notional: p.position_amt * p.entry_price,
            ..Default::default()
        }),
    }

    // account.positions aynasını eşitle.
    let mirror = snap.account.positions.iter_mut().find(|a| a.symbol == p.symbol && a.position_side == p.position_side);
    match mirror {
        Some(a) => {
            a.position_amt = p.position_amt;
            a.unrealized_profit = p.un_realized_profit;
            a.isolated_wallet = p.isolated_wallet;
        }
        None => snap.account.positions.push(AccountPosition {
            symbol: p.symbol.clone(),
            position_side: p.position_side.clone(),
            position_amt: p.position_amt,
            unrealized_profit: p.un_realized_profit,
            isolated_margin: p.isolated_wallet,
            isolated_wallet: p.isolated_wallet,
            notional: p.position_amt * p.entry_price,
            ..Default::default()
        }),
    }
}

fn margin_type_str(s: &str) -> String {
    if s == "isolated" { "ISOLATED".into() } else { "CROSSED".into() }
}

/// Açık emir listesini emir durum olayıyla eşitler.
pub fn sync_open_orders(snap: &mut AccountSnapshot, order: &OrderUpdate) {
    let status_open = matches!(order.status.as_str(), "NEW" | "PARTIALLY_FILLED");
    if status_open {
        if let Some(o) = snap
            .open_orders
            .iter_mut()
            .find(|o| o.order_id == order.order_id)
        {
            o.status = order.status.clone();
            o.executed_qty = Some(order.cumulative_filled_qty.to_string());
            o.avg_price = Some(order.avg_price.to_string());
            o.cum_quote = Some((order.cumulative_filled_qty * order.avg_price).to_string());
        } else {
            snap.open_orders.push(crate::order::BinanceOrderResponse {
                order_id: order.order_id,
                symbol: order.symbol.clone(),
                status: order.status.clone(),
                client_order_id: order.client_order_id.clone(),
                price: Some(order.price.to_string()),
                avg_price: Some(order.avg_price.to_string()),
                orig_qty: Some(order.orig_qty.to_string()),
                executed_qty: Some(order.cumulative_filled_qty.to_string()),
                cum_quote: Some((order.cumulative_filled_qty * order.avg_price).to_string()),
                time_in_force: Some(order.time_in_force.clone()),
                order_type: Some(order.order_type.clone()),
                reduce_only: Some(order.reduce_only),
                close_position: Some(order.close_position),
                side: Some(order.side.clone()),
                position_side: Some(order.position_side.clone()),
                stop_price: Some(order.stop_price.to_string()),
                working_type: Some(order.working_type.clone()),
                price_protect: Some(order.price_protect),
                orig_type: Some(order.orig_type.clone()),
                update_time: Some(order.transaction_time as i64),
                activation_price: Some(order.activation_price.to_string()),
                callback_rate: Some(order.callback_rate.to_string()),
                time: Some(order.transaction_time as i64),
            });
        }
    } else {
        snap.open_orders.retain(|o| o.order_id != order.order_id);
    }
}

/// Kısmi dolumu pozisyon durumuna işler (hedge ve one-way semantiği).
pub fn apply_fill(snap: &mut AccountSnapshot, order: &OrderUpdate) {
    let signed = signed_fill(order);
    if signed == Decimal::ZERO {
        return;
    }
    if let Some(p) = snap
        .positions
        .iter_mut()
        .find(|p| p.symbol == order.symbol && p.position_side == order.position_side)
    {
        let old_amt = p.position_amt;
        let new_amt = old_amt + signed;
        // Aynı yönde büyümede ağırlıklı ortalama giriş fiyatı.
        let same_dir = (old_amt == Decimal::ZERO) || (old_amt * signed > Decimal::ZERO);
        if same_dir {
            let qty = order.last_filled_qty;
            let cost = old_amt.abs() * p.entry_price + qty * order.last_filled_price;
            let total = old_amt.abs() + qty;
            if total > Decimal::ZERO {
                p.entry_price = cost / total;
            }
        }
        p.position_amt = new_amt;
        p.notional = new_amt * p.entry_price;
    } else {
        let pr = PositionRisk {
            symbol: order.symbol.clone(),
            position_side: order.position_side.clone(),
            position_amt: signed,
            entry_price: order.last_filled_price,
            mark_price: order.last_filled_price,
            notional: signed * order.last_filled_price,
            margin_type: "CROSSED".into(),
            ..Default::default()
        };
        snap.positions.push(pr);
    }

    // account.positions aynası.
    if let Some(a) = snap
        .account
        .positions
        .iter_mut()
        .find(|a| a.symbol == order.symbol && a.position_side == order.position_side)
    {
        a.position_amt += signed;
    }
}

/// Emirin pozisyon büyüklüğüne işaretli etkisi.
pub fn signed_fill(order: &OrderUpdate) -> Decimal {
    let qty = order.last_filled_qty;
    if order.position_side == "SHORT" {
        // SHORT tarafında SELL pozisyonu büyütür.
        if order.side == "SELL" {
            -qty
        } else {
            qty
        }
    } else if order.side == "BUY" {
        qty
    } else {
        -qty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::BinanceOrderResponse;
    use crate::state::snapshot::AccountSnapshot;
    use crate::types::user_event::{AccountUpdateBalance, AccountUpdatePosition, OrderUpdate};
    use rust_decimal::Decimal;
    use rust_decimal::prelude::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn account_update_applies_balances_and_positions() {
        let mut snap = AccountSnapshot::default();
        let ev = UserDataEvent::AccountUpdate {
            event_time: 1,
            transaction_time: 1,
            update_time: 100,
            reason: "ORDER".into(),
            balances: vec![AccountUpdateBalance {
                asset: "USDT".into(),
                wallet_balance: d("1000"),
                cross_wallet_balance: d("900"),
            }],
            positions: vec![AccountUpdatePosition {
                symbol: "BTCUSDT".into(),
                position_side: "BOTH".into(),
                position_amt: d("0.01"),
                entry_price: d("50000"),
                un_realized_profit: d("10"),
                margin_type: "cross".into(),
                isolated_wallet: d("0"),
            }],
        };
        apply(&mut snap, &ev);
        assert_eq!(snap.account.assets.len(), 1);
        assert_eq!(snap.account.assets[0].wallet_balance, d("1000"));
        assert_eq!(snap.positions.len(), 1);
        assert_eq!(snap.positions[0].position_amt, d("0.01"));
        assert!(snap.positions[0].is_open());
    }

    #[test]
    fn order_update_trade_fills_long() {
        let mut snap = AccountSnapshot::default();
        let order = OrderUpdate {
            symbol: "BTCUSDT".into(),
            client_order_id: "c1".into(),
            side: "BUY".into(),
            order_type: "MARKET".into(),
            status: "FILLED".into(),
            execution_type: "TRADE".into(),
            order_id: 42,
            last_filled_qty: d("0.01"),
            last_filled_price: d("50000"),
            cumulative_filled_qty: d("0.01"),
            avg_price: d("50000"),
            position_side: "BOTH".into(),
            ..Default::default()
        };
        apply(&mut snap, &UserDataEvent::OrderTradeUpdate {
            event_time: 1,
            transaction_time: 1,
            order,
        });
        assert!(snap.open_orders.is_empty(), "FILLED emir listeye eklenmez");
        let p = &snap.positions[0];
        assert_eq!(p.position_amt, d("0.01"));
        assert_eq!(p.entry_price, d("50000"));
    }

    #[test]
    fn order_update_open_order_tracked_then_removed() {
        let mut snap = AccountSnapshot::default();
        let make = |status: &str, x: &str| UserDataEvent::OrderTradeUpdate {
            event_time: 1,
            transaction_time: 1,
            order: OrderUpdate {
                symbol: "BTCUSDT".into(),
                client_order_id: "c1".into(),
                side: "SELL".into(),
                order_type: "LIMIT".into(),
                status: status.into(),
                execution_type: x.into(),
                order_id: 7,
                last_filled_qty: Decimal::ZERO,
                last_filled_price: Decimal::ZERO,
                cumulative_filled_qty: Decimal::ZERO,
                avg_price: Decimal::ZERO,
                position_side: "BOTH".into(),
                ..Default::default()
            },
        };
        apply(&mut snap, &make("NEW", "NEW"));
        assert_eq!(snap.open_orders.len(), 1);
        apply(&mut snap, &make("CANCELED", "CANCELED"));
        assert!(snap.open_orders.is_empty());
    }

    #[test]
    fn hedge_short_side_sell_increases_position() {
        let mut snap = AccountSnapshot::default();
        let order = OrderUpdate {
            symbol: "ETHUSDT".into(),
            client_order_id: "c2".into(),
            side: "SELL".into(),
            order_type: "MARKET".into(),
            status: "FILLED".into(),
            execution_type: "TRADE".into(),
            order_id: 43,
            last_filled_qty: d("0.5"),
            last_filled_price: d("3000"),
            cumulative_filled_qty: d("0.5"),
            avg_price: d("3000"),
            position_side: "SHORT".into(),
            ..Default::default()
        };
        apply(&mut snap, &UserDataEvent::OrderTradeUpdate {
            event_time: 1,
            transaction_time: 1,
            order,
        });
        assert_eq!(snap.positions[0].position_amt, d("-0.5"));
    }

    #[test]
    fn sync_open_order_upsert() {
        let mut snap = AccountSnapshot::default();
        let resp = BinanceOrderResponse {
            order_id: 9,
            symbol: "BTCUSDT".into(),
            status: "NEW".into(),
            client_order_id: "c9".into(),
            order_type: Some("LIMIT".into()),
            ..Default::default()
        };
        snap.open_orders.push(resp);
        let updated = BinanceOrderResponse {
            order_id: 9,
            symbol: "BTCUSDT".into(),
            status: "PARTIALLY_FILLED".into(),
            client_order_id: "c9".into(),
            order_type: Some("LIMIT".into()),
            ..Default::default()
        };
        sync_open_orders(&mut snap, &order_from_response(&updated));
        assert_eq!(snap.open_orders.len(), 1);
        assert_eq!(snap.open_orders[0].status, "PARTIALLY_FILLED");
    }

    fn order_from_response(r: &BinanceOrderResponse) -> OrderUpdate {
        OrderUpdate {
            symbol: r.symbol.clone(),
            client_order_id: r.client_order_id.clone(),
            side: r.side.clone().unwrap_or_default(),
            order_type: r.order_type.clone().unwrap_or_default(),
            status: r.status.clone(),
            order_id: r.order_id,
            position_side: r.position_side.clone().unwrap_or("BOTH".into()),
            ..Default::default()
        }
    }
}
