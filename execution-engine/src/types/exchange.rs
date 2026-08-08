//! Exchange bilgi modeli — `/fapi/v1/exchangeInfo` yanıtı.
//!
//! Pre-trade doğrulamanın temeli: sembol filtreleri (fiyat adımı, lot adımı,
//! min notional, pozisyon limiti, emir adet limiti) ve precizyon bilgisi.

use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitType {
    RequestWeight,
    Orders,
    RawRequests,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: u32,
    pub limit: u32,
}

impl<'de> Deserialize<'de> for RateLimit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "rateLimitType")]
            rate_limit_type: String,
            interval: String,
            #[serde(rename = "intervalNum")]
            interval_num: u32,
            limit: u32,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(RateLimit {
            rate_limit_type: r.rate_limit_type,
            interval: r.interval,
            interval_num: r.interval_num,
            limit: r.limit,
        })
    }
}

/// Sembol filtresi — her filtre tipi farklı alanlara sahiptir.
#[derive(Debug, Clone, serde::Serialize)]
pub enum SymbolFilter {
    PriceFilter { min_price: Decimal, max_price: Decimal, tick_size: Decimal },
    LotSize { min_qty: Decimal, max_qty: Decimal, step_size: Decimal },
    MinNotional { notional: Decimal, apply_to_market: bool },
    MaxNumOrders { limit: u32 },
    MaxNumAlgoOrders { limit: u32 },
    MaxPosition { max_position: Decimal },
    PercentPrice { mult_up: Decimal, mult_down: Decimal },
    MarketLotSize { min_qty: Decimal, max_qty: Decimal, step_size: Decimal },
    Other(String),
}

impl SymbolFilter {
    pub fn name(&self) -> &'static str {
        match self {
            SymbolFilter::PriceFilter { .. } => "PRICE_FILTER",
            SymbolFilter::LotSize { .. } => "LOT_SIZE",
            SymbolFilter::MinNotional { .. } => "MIN_NOTIONAL",
            SymbolFilter::MaxNumOrders { .. } => "MAX_NUM_ORDERS",
            SymbolFilter::MaxNumAlgoOrders { .. } => "MAX_NUM_ALGO_ORDERS",
            SymbolFilter::MaxPosition { .. } => "MAX_POSITION",
            SymbolFilter::PercentPrice { .. } => "PERCENT_PRICE",
            SymbolFilter::MarketLotSize { .. } => "MARKET_LOT_SIZE",
            SymbolFilter::Other(_) => "OTHER",
        }
    }
}

impl<'de> Deserialize<'de> for SymbolFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn dec(s: Option<&str>) -> Decimal {
            s.and_then(|v| v.parse().ok()).unwrap_or(Decimal::ZERO)
        }
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "filterType")]
            filter_type: String,
            #[serde(rename = "minPrice")]
            min_price: Option<String>,
            #[serde(rename = "maxPrice")]
            max_price: Option<String>,
            #[serde(rename = "tickSize")]
            tick_size: Option<String>,
            #[serde(rename = "minQty")]
            min_qty: Option<String>,
            #[serde(rename = "maxQty")]
            max_qty: Option<String>,
            #[serde(rename = "stepSize")]
            step_size: Option<String>,
            #[serde(rename = "notional")]
            notional: Option<String>,
            #[serde(rename = "applyToMarket")]
            apply_to_market: Option<bool>,
            #[serde(rename = "maxNumOrders")]
            max_num_orders: Option<u32>,
            #[serde(rename = "maxNumAlgoOrders")]
            max_num_algo_orders: Option<u32>,
            #[serde(rename = "maxPosition")]
            max_position: Option<String>,
            #[serde(rename = "multiplierUp")]
            multiplier_up: Option<String>,
            #[serde(rename = "multiplierDown")]
            multiplier_down: Option<String>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(match r.filter_type.as_str() {
            "PRICE_FILTER" => SymbolFilter::PriceFilter {
                min_price: dec(r.min_price.as_deref()),
                max_price: dec(r.max_price.as_deref()),
                tick_size: dec(r.tick_size.as_deref()),
            },
            "LOT_SIZE" => SymbolFilter::LotSize {
                min_qty: dec(r.min_qty.as_deref()),
                max_qty: dec(r.max_qty.as_deref()),
                step_size: dec(r.step_size.as_deref()),
            },
            "MIN_NOTIONAL" => SymbolFilter::MinNotional {
                notional: dec(r.notional.as_deref()),
                apply_to_market: r.apply_to_market.unwrap_or(true),
            },
            "MAX_NUM_ORDERS" => SymbolFilter::MaxNumOrders {
                limit: r.max_num_orders.unwrap_or(0),
            },
            "MAX_NUM_ALGO_ORDERS" => SymbolFilter::MaxNumAlgoOrders {
                limit: r.max_num_algo_orders.unwrap_or(0),
            },
            "MAX_POSITION" => SymbolFilter::MaxPosition {
                max_position: dec(r.max_position.as_deref()),
            },
            "PERCENT_PRICE" => SymbolFilter::PercentPrice {
                mult_up: dec(r.multiplier_up.as_deref()),
                mult_down: dec(r.multiplier_down.as_deref()),
            },
            "MARKET_LOT_SIZE" => SymbolFilter::MarketLotSize {
                min_qty: dec(r.min_qty.as_deref()),
                max_qty: dec(r.max_qty.as_deref()),
                step_size: dec(r.step_size.as_deref()),
            },
            other => SymbolFilter::Other(other.to_string()),
        })
    }
}

/// Tek sembolün kuralları.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub pair: String,
    pub status: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub base_asset_precision: u32,
    pub quote_asset_precision: u32,
    pub contract_type: String,
    pub quantity_precision: u32,
    pub price_precision: u32,
    pub margin_trading_supported: bool,
    pub order_types: Vec<String>,
    pub time_in_force: Vec<String>,
    pub filters: Vec<SymbolFilter>,
    pub trigger_protect: bool,
    pub maintenance_margin_percent: Decimal,
    pub required_margin_percent: Decimal,
}

impl SymbolInfo {
    pub fn filter(&self, name: &'static str) -> Option<&SymbolFilter> {
        self.filters.iter().find(|f| f.name() == name)
    }
}

impl<'de> Deserialize<'de> for SymbolInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn dec(s: Option<&str>) -> Decimal {
            s.and_then(|v| v.parse().ok()).unwrap_or(Decimal::ZERO)
        }
        #[derive(Deserialize)]
        struct Raw {
            symbol: String,
            pair: Option<String>,
            status: Option<String>,
            #[serde(rename = "baseAsset")]
            base_asset: Option<String>,
            #[serde(rename = "quoteAsset")]
            quote_asset: Option<String>,
            #[serde(rename = "baseAssetPrecision")]
            base_asset_precision: Option<u32>,
            #[serde(rename = "quoteAssetPrecision")]
            quote_asset_precision: Option<u32>,
            #[serde(rename = "contractType")]
            contract_type: Option<String>,
            #[serde(rename = "quantityPrecision")]
            quantity_precision: Option<u32>,
            #[serde(rename = "pricePrecision")]
            price_precision: Option<u32>,
            #[serde(rename = "marginTradingSupported")]
            margin_trading_supported: Option<bool>,
            #[serde(rename = "orderTypes")]
            order_types: Option<Vec<String>>,
            #[serde(rename = "timeInForce")]
            time_in_force: Option<Vec<String>>,
            #[serde(rename = "filters")]
            filters: Vec<SymbolFilter>,
            #[serde(rename = "triggerProtect")]
            trigger_protect: Option<bool>,
            #[serde(rename = "maintenanceMarginPercent")]
            maintenance_margin_percent: Option<String>,
            #[serde(rename = "requiredMarginPercent")]
            required_margin_percent: Option<String>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(SymbolInfo {
            symbol: r.symbol,
            pair: r.pair.unwrap_or_default(),
            status: r.status.unwrap_or_default(),
            base_asset: r.base_asset.unwrap_or_default(),
            quote_asset: r.quote_asset.unwrap_or_default(),
            base_asset_precision: r.base_asset_precision.unwrap_or(0),
            quote_asset_precision: r.quote_asset_precision.unwrap_or(0),
            contract_type: r.contract_type.unwrap_or_default(),
            quantity_precision: r.quantity_precision.unwrap_or(0),
            price_precision: r.price_precision.unwrap_or(0),
            margin_trading_supported: r.margin_trading_supported.unwrap_or(false),
            order_types: r.order_types.unwrap_or_default(),
            time_in_force: r.time_in_force.unwrap_or_default(),
            filters: r.filters,
            trigger_protect: r.trigger_protect.unwrap_or(false),
            maintenance_margin_percent: dec(r.maintenance_margin_percent.as_deref()),
            required_margin_percent: dec(r.required_margin_percent.as_deref()),
        })
    }
}

/// `/fapi/v1/exchangeInfo` yanıtı.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ExchangeInfo {
    pub timezone: String,
    pub server_time: u64,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<SymbolInfo>,
}

impl ExchangeInfo {
    pub fn symbol(&self, symbol: &str) -> Option<&SymbolInfo> {
        self.symbols.iter().find(|s| s.symbol == symbol)
    }
}

impl<'de> Deserialize<'de> for ExchangeInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            timezone: Option<String>,
            #[serde(rename = "serverTime")]
            server_time: Option<u64>,
            #[serde(rename = "rateLimits")]
            rate_limits: Vec<RateLimit>,
            #[serde(rename = "exchangeFilters")]
            _exchange_filters: Vec<serde_json::Value>,
            symbols: Vec<SymbolInfo>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(ExchangeInfo {
            timezone: r.timezone.unwrap_or_default(),
            server_time: r.server_time.unwrap_or(0),
            rate_limits: r.rate_limits,
            symbols: r.symbols,
        })
    }
}
