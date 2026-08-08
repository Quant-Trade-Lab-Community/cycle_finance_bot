//! Pre-trade doğrulama: sembol kuralları, precizyon, mod, filtreler.
//!
//! Emir borsaya gitmeden önce `OrderRequest` normalleştirilir (fiyat/miktar
//! adıma yuvarlanır) ve tüm sembol filtrelerine karşı doğrulanır. Reddedilen
//! emir borsaya asla ulaşmaz.

use crate::error::{ExecError, Result};
use crate::order::{OrderPositionSide, OrderRequest, OrderType};
use crate::state::exchange_cache::{
    lot_step, round_price_to_tick, round_qty_to_step, round_to_precision, tick_size, ExchangeCache,
};
use crate::types::exchange::SymbolFilter;
use rust_decimal::Decimal;

pub struct Preflight {
    exchange: ExchangeCache,
}

impl Preflight {
    pub fn new(exchange: ExchangeCache) -> Self {
        Self { exchange }
    }

    pub fn exchange(&self) -> &ExchangeCache {
        &self.exchange
    }

    /// Doğrula + normalleştir. `position_mode`: true = hedge (dualSidePosition).
    /// `client_order_id` yoksa otomatik üretilir.
    pub fn normalize_and_check(&self, order: &OrderRequest, position_mode: Option<bool>) -> Result<OrderRequest> {
        let symbol = order.symbol.to_uppercase();
        if symbol.is_empty() {
            return Err(ExecError::Preflight("symbol boş".into()));
        }
        let info = self
            .exchange
            .symbol(&symbol)
            .ok_or_else(|| ExecError::Preflight(format!("{symbol} exchangeInfo'da bulunamadı")))?;

        if info.status != "TRADING" {
            return Err(ExecError::Preflight(format!("{symbol} durumu '{}' — işlem kapalı", info.status)));
        }
        if !info.margin_trading_supported {
            return Err(ExecError::Preflight(format!("{symbol} marjin desteği yok")));
        }

        // Emir tipi izni.
        let type_str = order.order_type.binance_str().to_string();
        if !info.order_types.iter().any(|t| t == &type_str) {
            return Err(ExecError::Preflight(format!(
                "{symbol} emir tipi '{type_str}' desteklenmiyor (izinli: {})",
                info.order_types.join(", ")
            )));
        }

        // Hedge/one-way pozisyon modu tutarlılığı.
        if let Some(mode) = position_mode {
            match (mode, order.position_side) {
                (true, OrderPositionSide::Both) => {
                    return Err(ExecError::Preflight(
                        "HEDGE modda positionSide LONG/SHORT zorunludur (BOTH kabul edilmez)".into(),
                    ));
                }
                (false, OrderPositionSide::Long | OrderPositionSide::Short) => {
                    return Err(ExecError::Preflight(
                        "ONE_WAY modda positionSide BOTH olmalıdır".into(),
                    ));
                }
                _ => {}
            }
        }

        let mut normalized = order.clone();
        normalized.symbol = symbol.clone();

        // Miktar: precizyon + step + min/max.
        let qty = normalize_quantity(&info, normalized.quantity)?;
        normalized.quantity = qty;

        // Fiyat: adım + min/max (fiyat gerektiren tipler).
        if let Some(price) = normalized.price {
            let price = normalize_price(&info, price)?;
            normalized.price = Some(price);
        } else if order.order_type.requires_price() {
            return Err(ExecError::Preflight(format!(
                "{} fiyat gerektirir",
                order.order_type.binance_str()
            )));
        }

        // stopPrice (koşullu emirler) — trailing hariç.
        let needs_stop = matches!(
            order.order_type,
            OrderType::StopLoss
                | OrderType::StopLossLimit
                | OrderType::StopMarket
                | OrderType::TakeProfit
                | OrderType::TakeProfitLimit
                | OrderType::TakeProfitMarket
        );
        if needs_stop && normalized.stop_price.is_none() {
            return Err(ExecError::Preflight("koşullu emirler stop_price ister".into()));
        }
        if let Some(sp) = normalized.stop_price {
            let sp = normalize_price(&info, sp)?;
            normalized.stop_price = Some(sp);
        }

        // TRAILING_STOP_MARKET: activationPrice + callbackRate zorunlu.
        if order.order_type == OrderType::TrailingStopMarket {
            if normalized.activation_price.is_none() || normalized.callback_rate.is_none() {
                return Err(ExecError::Preflight(
                    "TRAILING_STOP_MARKET activation_price ve callback_rate ister".into(),
                ));
            }
            if let Some(ap) = normalized.activation_price {
                normalized.activation_price = Some(normalize_price(&info, ap)?);
            }
        }

        // TIF kontrolü (LIMIT tipi; LIMIT_MAKER POST_ONLY'dir, TIF taşımaz).
        if order.order_type == OrderType::Limit && normalized.time_in_force.is_none() {
            return Err(ExecError::Preflight("LIMIT tipi emirler time_in_force ister (GTC/IOC/FOK)".into()));
        }

        // MIN_NOTIONAL: fiyat belli ise qty*price >= notional (reduceOnly/closePosition hariç).
        let is_reduce = normalized.reduce_only.unwrap_or(false) || normalized.close_position.unwrap_or(false);
        if !is_reduce
            && let Some(price) = normalized.price {
                let notional = qty * price;
                if let Some(f) = info.filter("MIN_NOTIONAL")
                    && let SymbolFilter::MinNotional { notional: min_n, apply_to_market, .. } = f {
                        let _ = apply_to_market;
                        if min_n > &Decimal::ZERO && notional < *min_n {
                            return Err(ExecError::Preflight(format!(
                                "notional {notional} < MIN_NOTIONAL {min_n} ({symbol})"
                            )));
                        }
                    }
            }

        // MAX_NUM_ALGO_ORDERS (koşullu emirler).
        if order.order_type.is_stop()
            && let Some(f) = info.filter("MAX_NUM_ALGO_ORDERS")
                && let SymbolFilter::MaxNumAlgoOrders { limit } = f
                    && *limit == 0 {
                        return Err(ExecError::Preflight(format!("{symbol} koşullu emir yasak (algo limit 0)")));
                    }

        // Client order id uzunluğu (Binance ≤ 36).
        if let Some(cid) = &normalized.client_order_id {
            if cid.len() > 36 {
                return Err(ExecError::Preflight("client_order_id en fazla 36 karakter".into()));
            }
        } else {
            normalized.client_order_id = Some(new_client_order_id());
        }

        Ok(normalized)
    }
}

/// Miktarı sembol kurallarına göre normalleştirir.
/// Yuvarlama yalnızca aşağıya yapılır (stepSize katı + precizyon) — asla
/// yukarı yuvarlayarak geçersiz miktar üretilmez.
pub fn normalize_quantity(info: &crate::types::exchange::SymbolInfo, qty: Decimal) -> Result<Decimal> {
    if qty <= Decimal::ZERO {
        return Err(ExecError::Preflight("quantity > 0 olmalı".into()));
    }
    let mut q = qty;
    if let Some(step) = lot_step(info) {
        q = round_qty_to_step(q, step);
    }
    // Precizyon da tabana: örn. 0.003 → prec 2 ise 0.00 yerine 0.003'ü korumak
    // için adım zaten belirleyici; precizyon yalnızca kısıtlar.
    q = floor_to_precision(q, info.quantity_precision);
    for f in &info.filters {
        match f {
            SymbolFilter::LotSize { min_qty, max_qty, .. } => {
                if q < *min_qty {
                    return Err(ExecError::Preflight(format!(
                        "quantity {q} < LOT_SIZE min {min_qty}"
                    )));
                }
                if *max_qty > Decimal::ZERO && q > *max_qty {
                    return Err(ExecError::Preflight(format!(
                        "quantity {q} > LOT_SIZE max {max_qty}"
                    )));
                }
            }
            SymbolFilter::MaxPosition { max_position }
                if *max_position > Decimal::ZERO && q > *max_position => {
                    return Err(ExecError::Preflight(format!(
                        "quantity {q} > MAX_POSITION {max_position}"
                    )));
                }
            _ => {}
        }
    }
    Ok(q)
}

/// Fiyatı sembol kurallarına göre normalleştirir (tick katına yarım-yukarı).
pub fn normalize_price(info: &crate::types::exchange::SymbolInfo, price: Decimal) -> Result<Decimal> {
    if price <= Decimal::ZERO {
        return Err(ExecError::Preflight("price > 0 olmalı".into()));
    }
    let mut p = round_price_to_tick(price, tick_size(info).unwrap_or(Decimal::ONE));
    p = round_to_precision(p, info.price_precision);
    for f in &info.filters {
        if let SymbolFilter::PriceFilter { min_price, max_price, .. } = f {
            if p < *min_price {
                return Err(ExecError::Preflight(format!("price {p} < PRICE_FILTER min {min_price}")));
            }
            if *max_price > Decimal::ZERO && p > *max_price {
                return Err(ExecError::Preflight(format!("price {p} > PRICE_FILTER max {max_price}")));
            }
        }
    }
    Ok(p)
}

/// Pozitif değerleri ondalık precizyona tabana yuvarlar.
fn floor_to_precision(value: Decimal, precision: u32) -> Decimal {
    let scale = Decimal::from(10u64.pow(precision));
    (value * scale).floor() / scale
}

/// İstemci tarafı emir kimliği (uuid v4, 36 karakter).
pub fn new_client_order_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::exchange_cache::ExchangeCache;
    use crate::types::exchange::{SymbolFilter, SymbolInfo};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn test_symbol() -> SymbolInfo {
        SymbolInfo {
            symbol: "BTCUSDT".into(),
            pair: "BTCUSDT".into(),
            status: "TRADING".into(),
            base_asset: "BTC".into(),
            quote_asset: "USDT".into(),
            base_asset_precision: 3,
            quote_asset_precision: 2,
            contract_type: "PERPETUAL".into(),
            quantity_precision: 3,
            price_precision: 2,
            margin_trading_supported: true,
            order_types: vec![
                "LIMIT".into(),
                "MARKET".into(),
                "STOP".into(),
                "STOP_MARKET".into(),
                "TAKE_PROFIT".into(),
                "TAKE_PROFIT_MARKET".into(),
                "TRAILING_STOP_MARKET".into(),
                "LIMIT_MAKER".into(),
            ],
            time_in_force: vec!["GTC".into(), "IOC".into(), "FOK".into(), "GTX".into()],
            filters: vec![
                SymbolFilter::PriceFilter {
                    min_price: Decimal::from_str("0.01").unwrap(),
                    max_price: Decimal::from_str("1000000").unwrap(),
                    tick_size: Decimal::from_str("0.01").unwrap(),
                },
                SymbolFilter::LotSize {
                    min_qty: Decimal::from_str("0.001").unwrap(),
                    max_qty: Decimal::from_str("1000").unwrap(),
                    step_size: Decimal::from_str("0.001").unwrap(),
                },
                SymbolFilter::MinNotional {
                    notional: Decimal::from_str("100").unwrap(),
                    apply_to_market: true,
                },
            ],
            trigger_protect: Decimal::from_str("0.05").unwrap(),
            maintenance_margin_percent: Decimal::from(1),
            required_margin_percent: Decimal::from(5),
        }
    }

    fn cache_with_symbol() -> ExchangeCache {
        let cache = ExchangeCache::new(3600);
        let info = {
            let mut i = crate::types::exchange::ExchangeInfo::default();
            i.symbols.push(test_symbol());
            i
        };
        *cache.handle().write() = info;
        cache
    }

    #[test]
    fn quantity_floor_to_step() {
        let info = test_symbol();
        let q = normalize_quantity(&info, Decimal::from_str("0.0015").unwrap()).unwrap();
        assert_eq!(q, Decimal::from_str("0.001").unwrap());
        let q = normalize_quantity(&info, Decimal::from_str("0.00101").unwrap()).unwrap();
        assert_eq!(q, Decimal::from_str("0.001").unwrap());
    }

    #[test]
    fn quantity_below_min_rejected() {
        let info = test_symbol();
        assert!(normalize_quantity(&info, Decimal::from_str("0.0001").unwrap()).is_err());
    }

    #[test]
    fn price_rounds_to_tick() {
        let info = test_symbol();
        let p = normalize_price(&info, Decimal::from_str("100.005").unwrap()).unwrap();
        assert_eq!(p, Decimal::from_str("100.01").unwrap());
    }

    #[test]
    fn hedge_mode_requires_side() {
        let cache = cache_with_symbol();
        let pf = Preflight::new(cache);
        let order = OrderRequest {
            symbol: "BTCUSDT".into(),
            side: crate::order::OrderSide::Buy,
            order_type: crate::order::OrderType::Market,
            quantity: Decimal::from_str("0.01").unwrap(),
            position_side: crate::order::OrderPositionSide::Both,
            time_in_force: None,
            ..Default::default()
        };
        let err = pf.normalize_and_check(&order, Some(true)).unwrap_err();
        assert!(err.to_string().contains("positionSide"));
        // one-way modda LONG/SHORT reddedilir
        let order2 = OrderRequest {
            symbol: "BTCUSDT".into(),
            side: crate::order::OrderSide::Buy,
            order_type: crate::order::OrderType::Market,
            quantity: Decimal::from_str("0.01").unwrap(),
            position_side: crate::order::OrderPositionSide::Long,
            ..Default::default()
        };
        let err = pf.normalize_and_check(&order2, Some(false)).unwrap_err();
        assert!(err.to_string().contains("ONE_WAY"));
    }

    #[test]
    fn market_order_passes_and_gets_cid() {
        let cache = cache_with_symbol();
        let pf = Preflight::new(cache);
        let order = OrderRequest {
            symbol: "btcusdt".into(),
            side: crate::order::OrderSide::Buy,
            order_type: crate::order::OrderType::Market,
            quantity: Decimal::from_str("0.01").unwrap(),
            position_side: crate::order::OrderPositionSide::Both,
            ..Default::default()
        };
        let norm = pf.normalize_and_check(&order, Some(false)).unwrap();
        assert_eq!(norm.symbol, "BTCUSDT");
        assert!(norm.client_order_id.is_some());
    }

    #[test]
    fn limit_needs_tif() {
        let cache = cache_with_symbol();
        let pf = Preflight::new(cache);
        let order = OrderRequest {
            symbol: "BTCUSDT".into(),
            side: crate::order::OrderSide::Buy,
            order_type: crate::order::OrderType::Limit,
            quantity: Decimal::from_str("0.01").unwrap(),
            price: Some(Decimal::from_str("50000").unwrap()),
            position_side: crate::order::OrderPositionSide::Both,
            time_in_force: None,
            ..Default::default()
        };
        let err = pf.normalize_and_check(&order, Some(false)).unwrap_err();
        assert!(err.to_string().contains("time_in_force"));
    }

    #[test]
    fn limit_maker_needs_no_tif() {
        let cache = cache_with_symbol();
        let pf = Preflight::new(cache);
        let order = OrderRequest {
            symbol: "BTCUSDT".into(),
            side: crate::order::OrderSide::Sell,
            order_type: crate::order::OrderType::LimitMaker,
            quantity: Decimal::from_str("0.01").unwrap(),
            price: Some(Decimal::from_str("60000").unwrap()),
            position_side: crate::order::OrderPositionSide::Both,
            time_in_force: None,
            ..Default::default()
        };
        // TIF olmadan kabul edilir.
        let norm = pf.normalize_and_check(&order, Some(false)).unwrap();
        assert!(norm.time_in_force.is_none());
    }

    #[test]
    fn trailing_stop_requires_activation_and_callback() {
        let cache = cache_with_symbol();
        let pf = Preflight::new(cache);
        let order = OrderRequest {
            symbol: "BTCUSDT".into(),
            side: crate::order::OrderSide::Sell,
            order_type: crate::order::OrderType::TrailingStopMarket,
            quantity: Decimal::from_str("0.01").unwrap(),
            stop_price: Some(Decimal::from_str("40000").unwrap()),
            position_side: crate::order::OrderPositionSide::Both,
            ..Default::default()
        };
        let err = pf.normalize_and_check(&order, Some(false)).unwrap_err();
        assert!(err.to_string().contains("activation_price"));
    }

    #[test]
    fn stoploss_without_price_is_stop_market() {
        let cache = cache_with_symbol();
        let pf = Preflight::new(cache);
        let order = OrderRequest {
            symbol: "BTCUSDT".into(),
            side: crate::order::OrderSide::Buy,
            order_type: crate::order::OrderType::StopLoss,
            quantity: Decimal::from_str("0.01").unwrap(),
            stop_price: Some(Decimal::from_str("40000").unwrap()),
            position_side: crate::order::OrderPositionSide::Both,
            ..Default::default()
        };
        // StopLoss fiyatsız stop-market olarak kabul edilir (STOP tipi).
        let norm = pf.normalize_and_check(&order, Some(false)).unwrap();
        assert!(norm.price.is_none());
    }

    #[test]
    fn min_notional_enforced() {
        let cache = cache_with_symbol();
        let pf = Preflight::new(cache);
        let order = OrderRequest {
            symbol: "BTCUSDT".into(),
            side: crate::order::OrderSide::Buy,
            order_type: crate::order::OrderType::Limit,
            quantity: Decimal::from_str("0.001").unwrap(),
            price: Some(Decimal::from_str("50000").unwrap()),
            time_in_force: Some(crate::order::TimeInForce::Gtc),
            position_side: crate::order::OrderPositionSide::Both,
            ..Default::default()
        };
        // 0.001 * 50000 = 50 < 100 → reddedilir
        let err = pf.normalize_and_check(&order, Some(false)).unwrap_err();
        assert!(err.to_string().contains("MIN_NOTIONAL"));
    }
}
