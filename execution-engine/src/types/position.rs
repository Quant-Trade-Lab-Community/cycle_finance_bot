//! Pozisyon risk modeli — `/fapi/v2/positionRisk` yanıtı.

use rust_decimal::Decimal;
use serde::Deserialize;

/// Hedge mod tarafı (LONG/SHORT).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionSide {
    Long,
    Short,
    Both,
}

impl PositionSide {
    pub fn from_binance(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "LONG" => PositionSide::Long,
            "SHORT" => PositionSide::Short,
            _ => PositionSide::Both,
        }
    }
}

/// Sembol bazlı pozisyon risk bilgisi.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PositionRisk {
    pub symbol: String,
    pub position_side: String,
    pub position_amt: Decimal,
    pub entry_price: Decimal,
    pub mark_price: Decimal,
    pub un_realized_profit: Decimal,
    pub liquidation_price: Decimal,
    pub leverage: Decimal,
    pub max_notional: Decimal,
    pub margin_type: String,
    pub isolated_margin: Decimal,
    pub is_auto_add_margin: bool,
    pub position_initial_margin: Decimal,
    pub maint_margin: Decimal,
    pub isolated_wallet: Decimal,
    pub notional: Decimal,
    pub update_time: u64,
}

impl PositionRisk {
    pub fn is_open(&self) -> bool {
        self.position_amt != Decimal::ZERO
    }
}

impl<'de> Deserialize<'de> for PositionRisk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn dec(s: Option<&str>) -> Decimal {
            s.and_then(|v| v.parse().ok()).unwrap_or(Decimal::ZERO)
        }
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "symbol")]
            symbol: String,
            #[serde(rename = "positionSide")]
            position_side: Option<String>,
            #[serde(rename = "positionAmt")]
            position_amt: Option<String>,
            #[serde(rename = "entryPrice")]
            entry_price: Option<String>,
            #[serde(rename = "markPrice")]
            mark_price: Option<String>,
            #[serde(rename = "unRealizedProfit")]
            un_realized_profit: Option<String>,
            #[serde(rename = "liquidationPrice")]
            liquidation_price: Option<String>,
            #[serde(rename = "leverage")]
            leverage: Option<String>,
            #[serde(rename = "maxNotionalValue")]
            max_notional: Option<String>,
            #[serde(rename = "marginType")]
            margin_type: Option<String>,
            #[serde(rename = "isolatedMargin")]
            isolated_margin: Option<String>,
            #[serde(rename = "isAutoAddMargin")]
            is_auto_add_margin: Option<bool>,
            #[serde(rename = "positionInitialMargin")]
            position_initial_margin: Option<String>,
            #[serde(rename = "maintMargin")]
            maint_margin: Option<String>,
            #[serde(rename = "isolatedWallet")]
            isolated_wallet: Option<String>,
            #[serde(rename = "notional")]
            notional: Option<String>,
            #[serde(rename = "updateTime")]
            update_time: Option<u64>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(PositionRisk {
            symbol: r.symbol,
            position_side: r.position_side.unwrap_or_else(|| "BOTH".into()),
            position_amt: dec(r.position_amt.as_deref()),
            entry_price: dec(r.entry_price.as_deref()),
            mark_price: dec(r.mark_price.as_deref()),
            un_realized_profit: dec(r.un_realized_profit.as_deref()),
            liquidation_price: dec(r.liquidation_price.as_deref()),
            leverage: dec(r.leverage.as_deref()),
            max_notional: dec(r.max_notional.as_deref()),
            margin_type: r.margin_type.unwrap_or_else(|| "CROSSED".into()),
            isolated_margin: dec(r.isolated_margin.as_deref()),
            is_auto_add_margin: r.is_auto_add_margin.unwrap_or(false),
            position_initial_margin: dec(r.position_initial_margin.as_deref()),
            maint_margin: dec(r.maint_margin.as_deref()),
            isolated_wallet: dec(r.isolated_wallet.as_deref()),
            notional: dec(r.notional.as_deref()),
            update_time: r.update_time.unwrap_or(0),
        })
    }
}
