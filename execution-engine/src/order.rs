//! Emir domain modeli — Binance USDT-M Futures emir türleri ve istek/yanıt.
//!
//! Mevcut varyantlar canlı borsaya gönderimde `binance_str()` ile kanonik emir
//! tipine çevrilir.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn binance_str(&self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }
}

/// Binance USDT-M Futures emir türleri.
///
/// Kanonik borsa tipleri: `LIMIT, MARKET, STOP, STOP_MARKET, TAKE_PROFIT,
/// TAKE_PROFIT_MARKET, TRAILING_STOP_MARKET, LIMIT_MAKER`.
/// Eski `StopLoss*/TakeProfit*` varyantları canlıya `binance_str()` ile kanonik
/// değere çevrilir.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker,
    StopMarket,
    TakeProfitMarket,
    TrailingStopMarket,
}

impl OrderType {
    /// Canlı Binance futures'ın kabul ettiği kanonik emir tipi.
    pub fn binance_str(&self) -> &'static str {
        match self {
            OrderType::Limit => "LIMIT",
            OrderType::Market => "MARKET",
            OrderType::StopLoss => "STOP",
            OrderType::StopLossLimit => "STOP",
            OrderType::TakeProfit => "TAKE_PROFIT",
            OrderType::TakeProfitLimit => "TAKE_PROFIT",
            OrderType::LimitMaker => "LIMIT_MAKER",
            OrderType::StopMarket => "STOP_MARKET",
            OrderType::TakeProfitMarket => "TAKE_PROFIT_MARKET",
            OrderType::TrailingStopMarket => "TRAILING_STOP_MARKET",
        }
    }

    /// Fiyatı zorunlu kılan tipler (limit davranışı).
    pub fn requires_price(&self) -> bool {
        matches!(
            self,
            OrderType::Limit
                | OrderType::StopLossLimit
                | OrderType::TakeProfitLimit
                | OrderType::LimitMaker
        )
    }

    /// STOP/TAKE_PROFIT limitli varyantları TIF ister.
    pub fn requires_time_in_force(&self) -> bool {
        matches!(self, OrderType::Limit)
    }

    pub fn is_stop(&self) -> bool {
        matches!(
            self,
            OrderType::StopLoss
                | OrderType::StopLossLimit
                | OrderType::StopMarket
                | OrderType::TakeProfit
                | OrderType::TakeProfitLimit
                | OrderType::TakeProfitMarket
                | OrderType::TrailingStopMarket
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
    Gtx,
}

impl TimeInForce {
    pub fn binance_str(&self) -> &'static str {
        match self {
            TimeInForce::Gtc => "GTC",
            TimeInForce::Ioc => "IOC",
            TimeInForce::Fok => "FOK",
            TimeInForce::Gtx => "GTX",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderPositionSide {
    Both,
    Long,
    Short,
}

impl OrderPositionSide {
    pub fn binance_str(&self) -> &'static str {
        match self {
            OrderPositionSide::Both => "BOTH",
            OrderPositionSide::Long => "LONG",
            OrderPositionSide::Short => "SHORT",
        }
    }
}

/// Koşullu emirlerde tetikleme fiyatı kaynağı.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum WorkingType {
    MarkPrice,
    #[default]
    ContractPrice,
}

impl WorkingType {
    pub fn binance_str(&self) -> &'static str {
        match self {
            WorkingType::MarkPrice => "MARK_PRICE",
            WorkingType::ContractPrice => "CONTRACT_PRICE",
        }
    }
}


/// Emir cevap biçimi.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum NewOrderRespType {
    Ack,
    #[default]
    Result,
}

impl NewOrderRespType {
    pub fn binance_str(&self) -> &'static str {
        match self {
            NewOrderRespType::Ack => "ACK",
            NewOrderRespType::Result => "RESULT",
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum SelfTradePreventionMode {
    #[default]
    None,
    ExpireTaker,
    ExpireMaker,
    ExpireBoth,
}

impl SelfTradePreventionMode {
    pub fn binance_str(&self) -> &'static str {
        match self {
            SelfTradePreventionMode::None => "NONE",
            SelfTradePreventionMode::ExpireTaker => "EXPIRE_TAKER",
            SelfTradePreventionMode::ExpireMaker => "EXPIRE_MAKER",
            SelfTradePreventionMode::ExpireBoth => "EXPIRE_BOTH",
        }
    }
}


/// Emir durumu (user-data stream + REST ortak değerleri).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    PendingCancel,
    Rejected,
    Expired,
    ExpiredInMatch,
}

impl OrderStatus {
    pub fn binance_str(&self) -> &'static str {
        match self {
            OrderStatus::New => "NEW",
            OrderStatus::PartiallyFilled => "PARTIALLY_FILLED",
            OrderStatus::Filled => "FILLED",
            OrderStatus::Canceled => "CANCELED",
            OrderStatus::PendingCancel => "PENDING_CANCEL",
            OrderStatus::Rejected => "REJECTED",
            OrderStatus::Expired => "EXPIRED",
            OrderStatus::ExpiredInMatch => "EXPIRED_IN_MATCH",
        }
    }

    pub fn from_binance(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "NEW" => Some(OrderStatus::New),
            "PARTIALLY_FILLED" => Some(OrderStatus::PartiallyFilled),
            "FILLED" => Some(OrderStatus::Filled),
            "CANCELED" => Some(OrderStatus::Canceled),
            "PENDING_CANCEL" => Some(OrderStatus::PendingCancel),
            "REJECTED" => Some(OrderStatus::Rejected),
            "EXPIRED" => Some(OrderStatus::Expired),
            "EXPIRED_IN_MATCH" => Some(OrderStatus::ExpiredInMatch),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderStatus::Filled | OrderStatus::Canceled | OrderStatus::Rejected | OrderStatus::Expired
        )
    }

    pub fn is_open(&self) -> bool {
        matches!(self, OrderStatus::New | OrderStatus::PartiallyFilled)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderExecutionType {
    New,
    Trade,
    Expired,
    Canceled,
    Calculated,
    Trading,
    Replaced,
    Restated,
    Rejected,
    Amend,
    PendingCancel,
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    /// MARKET emirlerde USDT bazlı büyüklük (quantity yerine quoteOrderQty).
    /// Set edildiğinde `quantity` yok sayılır (Binance yalnızca MARKET kabul eder).
    pub quote_order_qty: Option<Decimal>,
    pub price: Option<Decimal>,
    pub time_in_force: Option<TimeInForce>,
    /// Hedge modda LONG/SHORT; one-way modda BOTH.
    pub position_side: OrderPositionSide,
    /// Idempotency anahtarı: aynı değer iki kez borsaya gönderilmez.
    pub client_order_id: Option<String>,
    pub reduce_only: Option<bool>,
    pub close_position: Option<bool>,
    /// Koşullu emirler (STOP*/TAKE_PROFIT*) için stopPrice.
    pub stop_price: Option<Decimal>,
    pub working_type: Option<WorkingType>,
    pub price_protect: Option<bool>,
    pub new_order_resp_type: Option<NewOrderRespType>,
    /// TRAILING_STOP_MARKET: tetikleme fiyatı (aktivasyon).
    pub activation_price: Option<Decimal>,
    /// TRAILING_STOP_MARKET: geri çekilme oranı (%).
    pub callback_rate: Option<Decimal>,
    pub self_trade_prevention_mode: Option<SelfTradePreventionMode>,
    pub recv_window: Option<u64>,
}

impl Default for OrderRequest {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: Decimal::ZERO,
            quote_order_qty: None,
            price: None,
            time_in_force: None,
            position_side: OrderPositionSide::Both,
            client_order_id: None,
            reduce_only: None,
            close_position: None,
            stop_price: None,
            working_type: None,
            price_protect: None,
            new_order_resp_type: None,
            activation_price: None,
            callback_rate: None,
            self_trade_prevention_mode: None,
            recv_window: None,
        }
    }
}

impl OrderRequest {
    /// Emrin USDT notional tahmini (fiyat yoksa 0).
    pub fn estimated_notional(&self) -> Decimal {
        match self.price {
            Some(p) => self.quantity * p,
            None => Decimal::ZERO,
        }
    }
}

/// Binance `/fapi/v1/order` ve `/fapi/v1/batchOrders` yanıtı.
/// ACK tipinde birçok alan eksiktir; bu yüzden hepsi `Option`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BinanceOrderResponse {
    #[serde(rename = "orderId")]
    pub order_id: i64,
    pub symbol: String,
    pub status: String,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub price: Option<String>,
    #[serde(rename = "avgPrice")]
    pub avg_price: Option<String>,
    #[serde(rename = "origQty")]
    pub orig_qty: Option<String>,
    #[serde(rename = "executedQty")]
    pub executed_qty: Option<String>,
    #[serde(rename = "cumQuote")]
    pub cum_quote: Option<String>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<String>,
    #[serde(rename = "type")]
    pub order_type: Option<String>,
    #[serde(rename = "reduceOnly")]
    pub reduce_only: Option<bool>,
    #[serde(rename = "closePosition")]
    pub close_position: Option<bool>,
    pub side: Option<String>,
    #[serde(rename = "positionSide")]
    pub position_side: Option<String>,
    #[serde(rename = "stopPrice")]
    pub stop_price: Option<String>,
    #[serde(rename = "workingType")]
    pub working_type: Option<String>,
    #[serde(rename = "priceProtect")]
    pub price_protect: Option<bool>,
    #[serde(rename = "origType")]
    pub orig_type: Option<String>,
    #[serde(rename = "updateTime")]
    pub update_time: Option<i64>,
    #[serde(rename = "activationPrice")]
    pub activation_price: Option<String>,
    #[serde(rename = "callbackRate")]
    pub callback_rate: Option<String>,
    #[serde(rename = "time")]
    pub time: Option<i64>,
}

impl BinanceOrderResponse {
    pub fn status_enum(&self) -> Option<OrderStatus> {
        OrderStatus::from_binance(&self.status)
    }
}

/// Kurumsal kullanıcıya (strateji/API) dönen işlenmiş emir sonucu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAck {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub status: String,
    pub avg_price: Decimal,
    pub executed_qty: Decimal,
    pub cum_quote: Decimal,
    pub reduce_only: bool,
}

impl From<BinanceOrderResponse> for OrderAck {
    fn from(r: BinanceOrderResponse) -> Self {
        Self {
            order_id: r.order_id.to_string(),
            client_order_id: r.client_order_id,
            symbol: r.symbol,
            status: r.status,
            avg_price: r.avg_price.as_deref().and_then(|s| s.parse().ok()).unwrap_or(Decimal::ZERO),
            executed_qty: r.executed_qty.as_deref().and_then(|s| s.parse().ok()).unwrap_or(Decimal::ZERO),
            cum_quote: r.cum_quote.as_deref().and_then(|s| s.parse().ok()).unwrap_or(Decimal::ZERO),
            reduce_only: r.reduce_only.unwrap_or(false),
        }
    }
}
