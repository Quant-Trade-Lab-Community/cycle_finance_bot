//! Hesap modeli — `/fapi/v3/account` ve `/fapi/v3/balance` yanıtları.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Marjin tipi (sembol bazlı).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarginType {
    Isolated,
    Crossed,
}

impl MarginType {
    pub fn binance_str(&self) -> &'static str {
        match self {
            MarginType::Isolated => "ISOLATED",
            MarginType::Crossed => "CROSSED",
        }
    }

    pub fn from_binance(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ISOLATED" | "ISOLATE" => Some(MarginType::Isolated),
            "CROSSED" | "CROSS" => Some(MarginType::Crossed),
            _ => None,
        }
    }
}

/// Bir varlığın cüzdan bakiyesi.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Balance {
    pub asset: String,
    #[serde(rename = "walletBalance")]
    pub wallet_balance: Decimal,
    #[serde(rename = "unrealizedProfit")]
    pub unrealized_profit: Decimal,
    #[serde(rename = "marginBalance")]
    pub margin_balance: Decimal,
    #[serde(rename = "maintMargin")]
    pub maint_margin: Decimal,
    #[serde(rename = "initialMargin")]
    pub initial_margin: Decimal,
    #[serde(rename = "positionInitialMargin")]
    pub position_initial_margin: Decimal,
    #[serde(rename = "openOrderInitialMargin")]
    pub open_order_initial_margin: Decimal,
    #[serde(rename = "crossWalletBalance")]
    pub cross_wallet_balance: Decimal,
    #[serde(rename = "crossUnPnl")]
    pub cross_un_pnl: Decimal,
    #[serde(rename = "availableBalance")]
    pub available_balance: Decimal,
    #[serde(rename = "maxWithdrawAmount")]
    pub max_withdraw_amount: Decimal,
}

impl Balance {
    fn dec(s: Option<&str>) -> Decimal {
        s.and_then(|v| v.parse().ok()).unwrap_or(Decimal::ZERO)
    }
}

// Özel deserializer: Binance string sayıları döndürür.
impl<'de> Deserialize<'de> for Balance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "asset")]
            asset: Option<String>,
            #[serde(rename = "walletBalance")]
            wallet_balance: Option<String>,
            #[serde(rename = "unrealizedProfit")]
            unrealized_profit: Option<String>,
            #[serde(rename = "marginBalance")]
            margin_balance: Option<String>,
            #[serde(rename = "maintMargin")]
            maint_margin: Option<String>,
            #[serde(rename = "initialMargin")]
            initial_margin: Option<String>,
            #[serde(rename = "positionInitialMargin")]
            position_initial_margin: Option<String>,
            #[serde(rename = "openOrderInitialMargin")]
            open_order_initial_margin: Option<String>,
            #[serde(rename = "crossWalletBalance")]
            cross_wallet_balance: Option<String>,
            #[serde(rename = "crossUnPnl")]
            cross_un_pnl: Option<String>,
            #[serde(rename = "availableBalance")]
            available_balance: Option<String>,
            #[serde(rename = "maxWithdrawAmount")]
            max_withdraw_amount: Option<String>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(Balance {
            asset: r.asset.unwrap_or_default(),
            wallet_balance: Balance::dec(r.wallet_balance.as_deref()),
            unrealized_profit: Balance::dec(r.unrealized_profit.as_deref()),
            margin_balance: Balance::dec(r.margin_balance.as_deref()),
            maint_margin: Balance::dec(r.maint_margin.as_deref()),
            initial_margin: Balance::dec(r.initial_margin.as_deref()),
            position_initial_margin: Balance::dec(r.position_initial_margin.as_deref()),
            open_order_initial_margin: Balance::dec(r.open_order_initial_margin.as_deref()),
            cross_wallet_balance: Balance::dec(r.cross_wallet_balance.as_deref()),
            cross_un_pnl: Balance::dec(r.cross_un_pnl.as_deref()),
            available_balance: Balance::dec(r.available_balance.as_deref()),
            max_withdraw_amount: Balance::dec(r.max_withdraw_amount.as_deref()),
        })
    }
}

/// `/fapi/v3/account` içindeki tek varlık.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AssetBalance {
    pub asset: String,
    pub wallet_balance: Decimal,
    pub unrealized_profit: Decimal,
    pub margin_balance: Decimal,
    pub maint_margin: Decimal,
    pub initial_margin: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub cross_wallet_balance: Decimal,
    pub cross_un_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
}

impl<'de> Deserialize<'de> for AssetBalance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "asset")]
            asset: String,
            #[serde(rename = "walletBalance")]
            wallet_balance: Option<String>,
            #[serde(rename = "unrealizedProfit")]
            unrealized_profit: Option<String>,
            #[serde(rename = "marginBalance")]
            margin_balance: Option<String>,
            #[serde(rename = "maintMargin")]
            maint_margin: Option<String>,
            #[serde(rename = "initialMargin")]
            initial_margin: Option<String>,
            #[serde(rename = "positionInitialMargin")]
            position_initial_margin: Option<String>,
            #[serde(rename = "openOrderInitialMargin")]
            open_order_initial_margin: Option<String>,
            #[serde(rename = "crossWalletBalance")]
            cross_wallet_balance: Option<String>,
            #[serde(rename = "crossUnPnl")]
            cross_un_pnl: Option<String>,
            #[serde(rename = "availableBalance")]
            available_balance: Option<String>,
            #[serde(rename = "maxWithdrawAmount")]
            max_withdraw_amount: Option<String>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(AssetBalance {
            asset: r.asset,
            wallet_balance: Balance::dec(r.wallet_balance.as_deref()),
            unrealized_profit: Balance::dec(r.unrealized_profit.as_deref()),
            margin_balance: Balance::dec(r.margin_balance.as_deref()),
            maint_margin: Balance::dec(r.maint_margin.as_deref()),
            initial_margin: Balance::dec(r.initial_margin.as_deref()),
            position_initial_margin: Balance::dec(r.position_initial_margin.as_deref()),
            open_order_initial_margin: Balance::dec(r.open_order_initial_margin.as_deref()),
            cross_wallet_balance: Balance::dec(r.cross_wallet_balance.as_deref()),
            cross_un_pnl: Balance::dec(r.cross_un_pnl.as_deref()),
            available_balance: Balance::dec(r.available_balance.as_deref()),
            max_withdraw_amount: Balance::dec(r.max_withdraw_amount.as_deref()),
        })
    }
}

/// `/fapi/v3/account` içindeki tek pozisyon.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AccountPosition {
    pub symbol: String,
    pub position_side: String,
    pub position_amt: Decimal,
    pub unrealized_profit: Decimal,
    pub isolated_margin: Decimal,
    pub notional: Decimal,
    pub isolated_wallet: Decimal,
    pub initial_margin: Decimal,
    pub maint_margin: Decimal,
    pub update_time: u64,
}

impl<'de> Deserialize<'de> for AccountPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "symbol")]
            symbol: String,
            #[serde(rename = "positionSide")]
            position_side: String,
            #[serde(rename = "positionAmt")]
            position_amt: Option<String>,
            #[serde(rename = "unrealizedProfit")]
            unrealized_profit: Option<String>,
            #[serde(rename = "isolatedMargin")]
            isolated_margin: Option<String>,
            #[serde(rename = "notional")]
            notional: Option<String>,
            #[serde(rename = "isolatedWallet")]
            isolated_wallet: Option<String>,
            #[serde(rename = "initialMargin")]
            initial_margin: Option<String>,
            #[serde(rename = "maintMargin")]
            maint_margin: Option<String>,
            #[serde(rename = "updateTime")]
            update_time: Option<u64>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(AccountPosition {
            symbol: r.symbol,
            position_side: r.position_side,
            position_amt: Balance::dec(r.position_amt.as_deref()),
            unrealized_profit: Balance::dec(r.unrealized_profit.as_deref()),
            isolated_margin: Balance::dec(r.isolated_margin.as_deref()),
            notional: Balance::dec(r.notional.as_deref()),
            isolated_wallet: Balance::dec(r.isolated_wallet.as_deref()),
            initial_margin: Balance::dec(r.initial_margin.as_deref()),
            maint_margin: Balance::dec(r.maint_margin.as_deref()),
            update_time: r.update_time.unwrap_or(0),
        })
    }
}

/// Tam hesap görünümü (`/fapi/v3/account`).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AccountInfo {
    pub total_wallet_balance: Decimal,
    pub total_unrealized_profit: Decimal,
    pub total_margin_balance: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub total_initial_margin: Decimal,
    pub total_maint_margin: Decimal,
    pub total_cross_wallet_balance: Decimal,
    pub total_cross_un_pnl: Decimal,
    pub assets: Vec<AssetBalance>,
    pub positions: Vec<AccountPosition>,
    /// Dönem başı varlık durumu (v3).
    pub fee_tier: i32,
    pub can_trade: bool,
    pub can_withdraw: bool,
}

impl<'de> Deserialize<'de> for AccountInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "totalWalletBalance")]
            total_wallet_balance: Option<String>,
            #[serde(rename = "totalUnrealizedProfit")]
            total_unrealized_profit: Option<String>,
            #[serde(rename = "totalMarginBalance")]
            total_margin_balance: Option<String>,
            #[serde(rename = "availableBalance")]
            available_balance: Option<String>,
            #[serde(rename = "maxWithdrawAmount")]
            max_withdraw_amount: Option<String>,
            #[serde(rename = "totalInitialMargin")]
            total_initial_margin: Option<String>,
            #[serde(rename = "totalMaintMargin")]
            total_maint_margin: Option<String>,
            #[serde(rename = "totalCrossWalletBalance")]
            total_cross_wallet_balance: Option<String>,
            #[serde(rename = "totalCrossUnPnl")]
            total_cross_un_pnl: Option<String>,
            #[serde(rename = "assets")]
            assets: Vec<AssetBalance>,
            #[serde(rename = "positions")]
            positions: Vec<AccountPosition>,
            #[serde(rename = "feeTier")]
            fee_tier: Option<i32>,
            #[serde(rename = "canTrade")]
            can_trade: Option<bool>,
            #[serde(rename = "canWithdraw")]
            can_withdraw: Option<bool>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(AccountInfo {
            total_wallet_balance: Balance::dec(r.total_wallet_balance.as_deref()),
            total_unrealized_profit: Balance::dec(r.total_unrealized_profit.as_deref()),
            total_margin_balance: Balance::dec(r.total_margin_balance.as_deref()),
            available_balance: Balance::dec(r.available_balance.as_deref()),
            max_withdraw_amount: Balance::dec(r.max_withdraw_amount.as_deref()),
            total_initial_margin: Balance::dec(r.total_initial_margin.as_deref()),
            total_maint_margin: Balance::dec(r.total_maint_margin.as_deref()),
            total_cross_wallet_balance: Balance::dec(r.total_cross_wallet_balance.as_deref()),
            total_cross_un_pnl: Balance::dec(r.total_cross_un_pnl.as_deref()),
            assets: r.assets,
            positions: r.positions,
            fee_tier: r.fee_tier.unwrap_or(0),
            can_trade: r.can_trade.unwrap_or(false),
            can_withdraw: r.can_withdraw.unwrap_or(false),
        })
    }
}
