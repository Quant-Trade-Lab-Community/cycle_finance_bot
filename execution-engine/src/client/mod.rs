//! Binance USDT-M Futures REST istemcisi.
//!
//! Emir, hesap, pozisyon ve hesap yapılandırma uçları tek yüzeyde toplanır.
//! Tüm imzalı istekler `HttpClient` üzerinden ağırlık takibi ve retry ile gider.

pub mod http;

use crate::config::ExecConfig;
use crate::error::{ExecError, Result};
use crate::order::{BinanceOrderResponse, OrderRequest, OrderType};
use crate::signer::BinanceSigner;
use crate::types::account::{AccountInfo, Balance, MarginType};
use crate::types::exchange::ExchangeInfo;
use crate::types::income::Income;
use crate::types::position::PositionRisk;
use http::HttpClient;
use reqwest::Method;
use rust_decimal::Decimal;
use serde_json::Value;
use std::sync::Arc;

pub struct BinanceClient {
    pub http: Arc<HttpClient>,
    signer: BinanceSigner,
    recv_window: u64,
}

fn qp(key: &str, value: impl ToString) -> (String, String) {
    (key.to_string(), value.to_string())
}

impl BinanceClient {
    pub fn new(config: &ExecConfig) -> Result<Arc<Self>> {
        if config.api_key.is_empty() || config.secret_key.is_empty() {
            return Err(ExecError::Config(
                "BINANCE_API_KEY / BINANCE_SECRET_KEY env değişkenleri eksik".into(),
            ));
        }
        let http = HttpClient::new(config.base_url.clone(), config.request_timeout_ms)?;
        Ok(Arc::new(Self {
            http,
            signer: BinanceSigner::new(config.api_key.clone(), config.secret_key.clone()),
            recv_window: config.recv_window_ms,
        }))
    }

    /// Test amaçlı: kimlik bilgisi olmadan salt okunur istemci.
    pub fn new_public(config: &ExecConfig) -> Result<Arc<Self>> {
        let http = HttpClient::new(config.base_url.clone(), config.request_timeout_ms)?;
        Ok(Arc::new(Self {
            http,
            signer: BinanceSigner::new(String::new(), String::new()),
            recv_window: 0,
        }))
    }

    pub fn signer(&self) -> &BinanceSigner {
        &self.signer
    }

    pub async fn sync_server_time(&self) -> Result<()> {
        self.http.sync_server_time().await
    }

    // ── Pazar / metadata ─────────────────────────────────────────

    pub async fn ping(&self) -> Result<()> {
        self.http.request(Method::GET, "/fapi/v1/ping", vec![], None, 0).await?;
        Ok(())
    }

    pub async fn server_time(&self) -> Result<u64> {
        let v = self.http.request(Method::GET, "/fapi/v1/time", vec![], None, 0).await?;
        v.get("serverTime").and_then(|x| x.as_u64()).ok_or_else(|| {
            ExecError::InvalidResponse("serverTime missing".into())
        })
    }

    pub async fn exchange_info(&self) -> Result<ExchangeInfo> {
        let v = self.http.request(Method::GET, "/fapi/v1/exchangeInfo", vec![], None, 0).await?;
        serde_json::from_value(v).map_err(ExecError::Json)
    }

    // ── Emir ─────────────────────────────────────────────────────

    pub async fn place_order(&self, order: &OrderRequest) -> Result<BinanceOrderResponse> {
        let params = order_params(order);
        let v = self.http.request(Method::POST, "/fapi/v1/order", params, Some(&self.signer), self.recv_window).await?;
        serde_json::from_value(v).map_err(ExecError::Json)
    }

    /// Toplu emir (≤5). Borsa dizi döndürür; tek tek hata nesneleri olabilir.
    pub async fn batch_orders(&self, orders: &[OrderRequest]) -> Result<Vec<Value>> {
        if orders.is_empty() || orders.len() > 5 {
            return Err(ExecError::Preflight("batchOrders 1..=5 emir alır".into()));
        }
        let items: Vec<String> = orders.iter().map(order_params_json).collect();
        let batch = serde_json::to_string(&items).map_err(ExecError::Json)?;
        let params = vec![("batchOrders".to_string(), batch)];
        let v = self.http.request(Method::POST, "/fapi/v1/batchOrders", params, Some(&self.signer), self.recv_window).await?;
        v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("batchOrders response not array".into()))
    }

    pub async fn query_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
    ) -> Result<BinanceOrderResponse> {
        let mut params = vec![qp("symbol", symbol)];
        if let Some(id) = order_id {
            params.push(qp("orderId", id));
        }
        if let Some(cid) = client_order_id {
            params.push(qp("origClientOrderId", cid));
        }
        let v = self.http.request(Method::GET, "/fapi/v1/order", params, Some(&self.signer), self.recv_window).await?;
        serde_json::from_value(v).map_err(ExecError::Json)
    }

    pub async fn query_open_orders(&self, symbol: Option<&str>) -> Result<Vec<BinanceOrderResponse>> {
        let mut params = vec![];
        if let Some(s) = symbol {
            params.push(qp("symbol", s));
        }
        let v = self.http.request(Method::GET, "/fapi/v1/openOrders", params, Some(&self.signer), self.recv_window).await?;
        let arr = v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("openOrders not array".into()))?;
        arr.into_iter().map(|item| serde_json::from_value(item).map_err(ExecError::Json)).collect()
    }

    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
    ) -> Result<BinanceOrderResponse> {
        let mut params = vec![qp("symbol", symbol)];
        if let Some(id) = order_id {
            params.push(qp("orderId", id));
        }
        if let Some(cid) = client_order_id {
            params.push(qp("origClientOrderId", cid));
        }
        let v = self.http.request(Method::DELETE, "/fapi/v1/order", params, Some(&self.signer), self.recv_window).await?;
        serde_json::from_value(v).map_err(ExecError::Json)
    }

    pub async fn cancel_all_open(&self, symbol: &str) -> Result<Vec<BinanceOrderResponse>> {
        let params = vec![qp("symbol", symbol)];
        let v = self.http.request(Method::DELETE, "/fapi/v1/allOpenOrders", params, Some(&self.signer), self.recv_window).await?;
        let arr = v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("allOpenOrders not array".into()))?;
        arr.into_iter().map(|item| serde_json::from_value(item).map_err(ExecError::Json)).collect()
    }

    /// Emir değiştirme (PUT /fapi/v1/order).
    #[allow(clippy::too_many_arguments)]
    pub async fn modify_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
        quantity: Option<Decimal>,
        price: Option<Decimal>,
        stop_price: Option<Decimal>,
        recv_window: u64,
    ) -> Result<BinanceOrderResponse> {
        let mut params = vec![qp("symbol", symbol)];
        if let Some(id) = order_id {
            params.push(qp("orderId", id));
        }
        if let Some(cid) = client_order_id {
            params.push(qp("origClientOrderId", cid));
        }
        if let Some(q) = quantity {
            params.push(qp("quantity", q));
        }
        if let Some(p) = price {
            params.push(qp("price", p));
        }
        if let Some(sp) = stop_price {
            params.push(qp("stopPrice", sp));
        }
        let rw = if recv_window > 0 { recv_window } else { self.recv_window };
        let v = self.http.request(Method::PUT, "/fapi/v1/order", params, Some(&self.signer), rw).await?;
        serde_json::from_value(v).map_err(ExecError::Json)
    }

    // ── Hesap / pozisyon ─────────────────────────────────────────

    pub async fn account_info(&self) -> Result<AccountInfo> {
        let v = self.http.request(Method::GET, "/fapi/v3/account", vec![], Some(&self.signer), self.recv_window).await?;
        serde_json::from_value(v).map_err(ExecError::Json)
    }

    pub async fn balance(&self) -> Result<Vec<Balance>> {
        let v = self.http.request(Method::GET, "/fapi/v3/balance", vec![], Some(&self.signer), self.recv_window).await?;
        let arr = v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("balance not array".into()))?;
        arr.into_iter().map(|item| serde_json::from_value(item).map_err(ExecError::Json)).collect()
    }

    pub async fn position_risk(&self, symbol: Option<&str>) -> Result<Vec<PositionRisk>> {
        let mut params = vec![];
        if let Some(s) = symbol {
            params.push(qp("symbol", s));
        }
        let v = self.http.request(Method::GET, "/fapi/v2/positionRisk", params, Some(&self.signer), self.recv_window).await?;
        let arr = v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("positionRisk not array".into()))?;
        arr.into_iter().map(|item| serde_json::from_value(item).map_err(ExecError::Json)).collect()
    }

    pub async fn income(
        &self,
        symbol: Option<&str>,
        income_type: Option<&str>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<Income>> {
        let mut params = vec![];
        if let Some(s) = symbol {
            params.push(qp("symbol", s));
        }
        if let Some(t) = income_type {
            params.push(qp("incomeType", t));
        }
        if let Some(t) = start_time {
            params.push(qp("startTime", t));
        }
        if let Some(t) = end_time {
            params.push(qp("endTime", t));
        }
        if let Some(l) = limit {
            params.push(qp("limit", l));
        }
        let v = self.http.request(Method::GET, "/fapi/v1/income", params, Some(&self.signer), self.recv_window).await?;
        let arr = v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("income not array".into()))?;
        arr.into_iter().map(|item| serde_json::from_value(item).map_err(ExecError::Json)).collect()
    }

    /// FUNDING_FEE tipi gelirleri.
    pub async fn funding_payments(&self, symbol: &str, start_time: Option<u64>, end_time: Option<u64>) -> Result<Vec<Income>> {
        self.income(Some(symbol), Some("FUNDING_FEE"), start_time, end_time, None).await
    }

    pub async fn leverage_bracket(&self, symbol: &str) -> Result<Value> {
        let params = vec![qp("symbol", symbol)];
        self.http.request(Method::GET, "/fapi/v1/leverageBracket", params, Some(&self.signer), self.recv_window).await
    }

    pub async fn commission_rate(&self, symbol: &str) -> Result<Value> {
        let params = vec![qp("symbol", symbol)];
        self.http.request(Method::GET, "/fapi/v1/commissionRate", params, Some(&self.signer), self.recv_window).await
    }

    pub async fn api_trading_status(&self) -> Result<Value> {
        self.http.request(Method::GET, "/fapi/v1/apiTradingStatus", vec![], Some(&self.signer), self.recv_window).await
    }

    pub async fn force_orders(&self, symbol: Option<&str>) -> Result<Vec<Value>> {
        let mut params = vec![];
        if let Some(s) = symbol {
            params.push(qp("symbol", s));
        }
        let v = self.http.request(Method::GET, "/fapi/v1/forceOrders", params, Some(&self.signer), self.recv_window).await?;
        v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("forceOrders not array".into()))
    }

    pub async fn rate_limit_order(&self) -> Result<Vec<Value>> {
        let v = self.http.request(Method::GET, "/fapi/v1/rateLimit/order", vec![], Some(&self.signer), self.recv_window).await?;
        v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("rateLimit/order not array".into()))
    }

    pub async fn position_adl_quantile(&self, symbol: Option<&str>) -> Result<Value> {
        let mut params = vec![];
        if let Some(s) = symbol {
            params.push(qp("symbol", s));
        }
        self.http.request(Method::GET, "/fapi/v1/positionADLQuantile", params, Some(&self.signer), self.recv_window).await
    }

    // ── Yapılandırma / kontrol ───────────────────────────────────

    pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<Value> {
        let params = vec![qp("symbol", symbol), qp("leverage", leverage)];
        self.http.request(Method::POST, "/fapi/v1/leverage", params, Some(&self.signer), self.recv_window).await
    }

    pub async fn set_margin_type(&self, symbol: &str, margin_type: MarginType) -> Result<Value> {
        let params = vec![qp("symbol", symbol), qp("marginType", margin_type.binance_str())];
        self.http.request(Method::POST, "/fapi/v1/marginType", params, Some(&self.signer), self.recv_window).await
    }

    /// İzole marj ekle (1) / çek (2).
    pub async fn adjust_position_margin(&self, symbol: &str, amount: Decimal, direction: u8) -> Result<Value> {
        let params = vec![
            qp("symbol", symbol),
            qp("amount", amount),
            qp("type", direction),
        ];
        self.http.request(Method::POST, "/fapi/v1/positionMargin", params, Some(&self.signer), self.recv_window).await
    }

    pub async fn position_margin_history(&self, symbol: &str) -> Result<Vec<Value>> {
        let params = vec![qp("symbol", symbol)];
        let v = self.http.request(Method::GET, "/fapi/v1/positionMargin/history", params, Some(&self.signer), self.recv_window).await?;
        v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("positionMargin/history not array".into()))
    }

    /// Hedge modu aç/kapat (true = hedge / dualSidePosition).
    pub async fn set_position_mode(&self, dual_side_position: bool) -> Result<Value> {
        let params = vec![qp("dualSidePosition", dual_side_position)];
        self.http.request(Method::POST, "/fapi/v1/positionSide/dual", params, Some(&self.signer), self.recv_window).await
    }

    pub async fn get_position_mode(&self) -> Result<bool> {
        let v = self.http.request(Method::GET, "/fapi/v1/positionSide/dual", vec![], Some(&self.signer), self.recv_window).await?;
        v.get("dualSidePosition").and_then(|x| x.as_bool()).ok_or_else(|| {
            ExecError::InvalidResponse("dualSidePosition missing".into())
        })
    }

    pub async fn set_multi_assets(&self, enabled: bool) -> Result<Value> {
        let params = vec![qp("multiAssetsMargin", enabled)];
        self.http.request(Method::POST, "/fapi/v1/multiAssetsMargin", params, Some(&self.signer), self.recv_window).await
    }

    pub async fn get_multi_assets(&self) -> Result<bool> {
        let v = self.http.request(Method::GET, "/fapi/v1/multiAssetsMargin", vec![], Some(&self.signer), self.recv_window).await?;
        v.get("multiAssetsMargin").and_then(|x| x.as_bool()).ok_or_else(|| {
            ExecError::InvalidResponse("multiAssetsMargin missing".into())
        })
    }

    pub async fn premium_index(&self, symbol: &str) -> Result<Value> {
        let params = vec![qp("symbol", symbol)];
        self.http.request(Method::GET, "/fapi/v1/premiumIndex", params, None, 0).await
    }

    pub async fn funding_rate(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<Value>> {
        let mut params = vec![qp("symbol", symbol)];
        if let Some(l) = limit {
            params.push(qp("limit", l));
        }
        let v = self.http.request(Method::GET, "/fapi/v1/fundingRate", params, None, 0).await?;
        v.as_array().cloned().ok_or_else(|| ExecError::InvalidResponse("fundingRate not array".into()))
    }

    // ── User-data stream (listenKey) ─────────────────────────────

    pub async fn create_listen_key(&self) -> Result<String> {
        let v = self.http.request(Method::POST, "/fapi/v1/listenKey", vec![], Some(&self.signer), self.recv_window).await?;
        v.get("listenKey").and_then(|x| x.as_str()).map(|s| s.to_string()).ok_or_else(|| {
            ExecError::InvalidResponse("listenKey missing".into())
        })
    }

    pub async fn refresh_listen_key(&self, listen_key: &str) -> Result<()> {
        let params = vec![qp("listenKey", listen_key)];
        self.http.request(Method::PUT, "/fapi/v1/listenKey", params, Some(&self.signer), self.recv_window).await?;
        Ok(())
    }

    pub async fn delete_listen_key(&self, listen_key: &str) -> Result<()> {
        let params = vec![qp("listenKey", listen_key)];
        self.http.request(Method::DELETE, "/fapi/v1/listenKey", params, Some(&self.signer), self.recv_window).await?;
        Ok(())
    }
}

/// `OrderRequest` → imza parametreleri (canlı borsa formatı).
pub fn order_params(order: &OrderRequest) -> Vec<(String, String)> {
    let mut p = vec![
        qp("symbol", &order.symbol),
        qp("side", order.side.binance_str()),
        qp("type", order.order_type.binance_str()),
        qp("positionSide", order.position_side.binance_str()),
        qp("newOrderRespType", order.new_order_resp_type.unwrap_or(crate::order::NewOrderRespType::Result).binance_str()),
    ];
    // MARKET emirlerde USDT bazlı büyüklük: quantity yerine quoteOrderQty.
    if let Some(qoq) = order.quote_order_qty {
        p.push(qp("quoteOrderQty", qoq));
    } else {
        p.push(qp("quantity", order.quantity));
    }
    if let Some(price) = order.price {
        p.push(qp("price", price));
    }
    if let Some(sp) = order.stop_price {
        p.push(qp("stopPrice", sp));
    }
    if let Some(tif) = order.time_in_force {
        p.push(qp("timeInForce", tif.binance_str()));
    }
    if let Some(cid) = &order.client_order_id {
        p.push(qp("newClientOrderId", cid));
    }
    if let Some(ro) = order.reduce_only {
        p.push(qp("reduceOnly", ro));
    }
    if let Some(cp) = order.close_position {
        p.push(qp("closePosition", cp));
    }
    if let Some(wt) = order.working_type {
        p.push(qp("workingType", wt.binance_str()));
    }
    if let Some(pp) = order.price_protect {
        p.push(qp("priceProtect", pp));
    }
    if let Some(ap) = order.activation_price {
        p.push(qp("activationPrice", ap));
    }
    if let Some(cr) = order.callback_rate {
        p.push(qp("callbackRate", cr));
    }
    if let Some(stp) = order.self_trade_prevention_mode {
        p.push(qp("selfTradePreventionMode", stp.binance_str()));
    }
    p
}

/// `OrderRequest` → batchOrders JSON nesnesi.
pub fn order_params_json(order: &OrderRequest) -> String {
    let p = order_params(order);
    let mut obj = serde_json::Map::new();
    for (k, v) in p {
        obj.insert(k, serde_json::Value::String(v));
    }
    serde_json::to_string(&obj).unwrap_or_else(|_| "{}".into())
}

/// `OrderType` is_stop bilgisi ile fiyat/stop gereksinimi preflight'ta denetlenir.
pub fn needs_price(order_type: &OrderType) -> bool {
    order_type.requires_price()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::{OrderSide, OrderType, TimeInForce};
    use std::str::FromStr;

    #[test]
    fn order_params_canonical_mapping() {
        let order = OrderRequest {
            symbol: "BTCUSDT".into(),
            side: OrderSide::Buy,
            order_type: OrderType::StopLoss,
            quantity: Decimal::from_str("0.01").unwrap(),
            price: Some(Decimal::from_str("45000").unwrap()),
            stop_price: Some(Decimal::from_str("44000").unwrap()),
            time_in_force: Some(TimeInForce::Gtc),
            position_side: crate::order::OrderPositionSide::Long,
            client_order_id: Some("cid-1".into()),
            reduce_only: Some(true),
            ..Default::default()
        };
        let p = order_params(&order);
        let get = |k: &str| p.iter().find(|(x, _)| x == k).map(|(_, v)| v.clone()).unwrap_or_default();
        assert_eq!(get("side"), "BUY");
        assert_eq!(get("type"), "STOP");
        assert_eq!(get("positionSide"), "LONG");
        assert_eq!(get("timeInForce"), "GTC");
        assert_eq!(get("stopPrice"), "44000");
        assert_eq!(get("newClientOrderId"), "cid-1");
        assert_eq!(get("reduceOnly"), "true");
    }

    #[test]
    fn market_type_maps() {
        let order = OrderRequest {
            symbol: "X".into(),
            side: OrderSide::Sell,
            order_type: OrderType::TakeProfitMarket,
            quantity: Decimal::from(1),
            position_side: crate::order::OrderPositionSide::Short,
            ..Default::default()
        };
        let p = order_params(&order);
        let get = |k: &str| p.iter().find(|(x, _)| x == k).map(|(_, v)| v.clone()).unwrap_or_default();
        assert_eq!(get("type"), "TAKE_PROFIT_MARKET");
        assert_eq!(get("positionSide"), "SHORT");
    }
}
