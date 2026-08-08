//! Gelir/komisyon modeli — `/fapi/v1/income` yanıtı.

use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomeType {
    Transfer,
    WelcomeBonus,
    RealizedPnl,
    FundingFee,
    Commission,
    InsuranceClear,
    ReferralKickback,
    CommissionRebate,
    Dividend,
    LiquidatedAccounts,
    Others,
}

impl IncomeType {
    pub fn from_binance(s: &str) -> Self {
        match s {
            "TRANSFER" => IncomeType::Transfer,
            "WELCOME_BONUS" => IncomeType::WelcomeBonus,
            "REALIZED_PNL" => IncomeType::RealizedPnl,
            "FUNDING_FEE" => IncomeType::FundingFee,
            "COMMISSION" => IncomeType::Commission,
            "INSURANCE_CLEAR" => IncomeType::InsuranceClear,
            "REFERRAL_KICKBACK" => IncomeType::ReferralKickback,
            "COMMISSION_REBATE" => IncomeType::CommissionRebate,
            "DIVIDEND" => IncomeType::Dividend,
            "LIQUIDATION_FEE" | "LIQUIDATED_ACCOUNTS" => IncomeType::LiquidatedAccounts,
            _ => IncomeType::Others,
        }
    }

    pub fn binance_str(&self) -> &'static str {
        match self {
            IncomeType::Transfer => "TRANSFER",
            IncomeType::WelcomeBonus => "WELCOME_BONUS",
            IncomeType::RealizedPnl => "REALIZED_PNL",
            IncomeType::FundingFee => "FUNDING_FEE",
            IncomeType::Commission => "COMMISSION",
            IncomeType::InsuranceClear => "INSURANCE_CLEAR",
            IncomeType::ReferralKickback => "REFERRAL_KICKBACK",
            IncomeType::CommissionRebate => "COMMISSION_REBATE",
            IncomeType::Dividend => "DIVIDEND",
            IncomeType::LiquidatedAccounts => "LIQUIDATED_ACCOUNTS",
            IncomeType::Others => "OTHERS",
        }
    }
}

/// Tek gelir kaydı.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Income {
    pub symbol: String,
    pub income_type: String,
    pub income: Decimal,
    pub asset: String,
    pub time: u64,
    pub info: String,
    pub tran_id: i64,
    pub trade_id: String,
}

impl<'de> Deserialize<'de> for Income {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn dec(s: Option<&str>) -> Decimal {
            s.and_then(|v| v.parse().ok()).unwrap_or(Decimal::ZERO)
        }
        #[derive(Deserialize)]
        struct Raw {
            symbol: Option<String>,
            #[serde(rename = "incomeType")]
            income_type: Option<String>,
            income: Option<String>,
            asset: Option<String>,
            time: Option<u64>,
            info: Option<String>,
            #[serde(rename = "tranId")]
            tran_id: Option<i64>,
            #[serde(rename = "tradeId")]
            trade_id: Option<String>,
        }
        let r = Raw::deserialize(deserializer)?;
        Ok(Income {
            symbol: r.symbol.unwrap_or_default(),
            income_type: r.income_type.unwrap_or_default(),
            income: dec(r.income.as_deref()),
            asset: r.asset.unwrap_or_default(),
            time: r.time.unwrap_or(0),
            info: r.info.unwrap_or_default(),
            tran_id: r.tran_id.unwrap_or(0),
            trade_id: r.trade_id.unwrap_or_default(),
        })
    }
}
