//! Ortak risk tipleri: emir niyeti, karar, ret nedeni, durum.

use rust_decimal::Decimal;
use serde::Serialize;

/// Emir yönü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }

    /// Pozisyona etki işareti (alım +, satım -).
    pub fn sign(self) -> i8 {
        match self {
            Side::Buy => 1,
            Side::Sell => -1,
        }
    }
}

/// Emir türü (fiyat gereksinimi için).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderKind {
    Market,
    Limit,
}

/// Risk kapısına giren emir niyeti — strateji katmanından (`Signal`) veya
/// execution katmanından (`OrderRequest`) bu tipe dönüştürülür.
#[derive(Debug, Clone)]
pub struct OrderIntent {
    /// Sinyali üreten strateji (bağımsız sayaç; 0 = dış API/manuel).
    pub strategy_id: u32,
    pub symbol: String,
    pub side: Side,
    /// Baz-coin cinsinden pozitif miktar.
    pub quantity: Decimal,
    /// Limit emirlerde fiyat; market emirlerde `None` → mark fiyat kullanılır.
    pub price: Option<Decimal>,
    pub kind: OrderKind,
    /// Yalnızca azaltma emri (pozisyon büyütme yasak).
    pub reduce_only: bool,
    pub close_position: bool,
    /// Biliniyorsa emir bazında kaldıraç (yoksa politika kullanılır).
    pub leverage: Option<Decimal>,
}

impl OrderIntent {
    /// Mark fiyatı (veya emir fiyatı) üzerinden USDT notional tahmini.
    /// Fiyat yoksa `None` — market emri için mark gerekir (fail-closed).
    pub fn notional(&self, mark_price: Option<Decimal>) -> Option<Decimal> {
        let price = self.price.or(mark_price)?;
        Some(price * self.quantity)
    }

    /// Pozisyon işaretli miktar (alım +, satım -).
    pub fn signed_quantity(&self) -> Decimal {
        match self.side {
            Side::Buy => self.quantity,
            Side::Sell => -self.quantity,
        }
    }
}

/// Risk kapısı kararı.
#[derive(Debug, Clone)]
pub enum RiskDecision {
    Approved {
        intent: OrderIntent,
    },
    Rejected {
        intent: OrderIntent,
        reason: RejectReason,
    },
}

impl RiskDecision {
    pub fn is_approved(&self) -> bool {
        matches!(self, RiskDecision::Approved { .. })
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self, RiskDecision::Rejected { .. })
    }
}

/// Ret nedenleri — her biri tek bir kuralı temsil eder (denetim izi).
#[derive(Debug, Clone, PartialEq)]
pub enum RejectReason {
    KillSwitch,
    CircuitBreaker,
    BlockedSymbol(String),
    RateLimit { limit: u32 },
    LeverageExceeded { max: Decimal },
    NotionalExceeded { notional: Decimal, max: Decimal },
    PositionLimitExceeded { symbol: String, current_notional: Decimal, max: Decimal },
    ExposureLimitExceeded { gross: Decimal, max: Decimal },
    ConcentrationExceeded { hhi: f64, max: f64 },
    InsufficientMargin { required: Decimal, available: Decimal },
    DailyLossExceeded { loss: Decimal, limit: Decimal },
    DrawdownExceeded { drawdown_pct: Decimal, max: Decimal },
    StaleMark { symbol: String, age_ms: u64 },
    ParametricRiskUnavailable,
    LiquidityLimitExceeded { slippage_bps: Decimal, max: Decimal },
    LiquidationProximity { symbol: String },
}

impl RejectReason {
    /// Kural adı — denetim izinde hangi kuralın reddettiğini gösterir.
    pub fn rule_name(&self) -> &'static str {
        match self {
            RejectReason::KillSwitch => "KILL_SWITCH",
            RejectReason::CircuitBreaker => "CIRCUIT_BREAKER",
            RejectReason::BlockedSymbol(_) => "SYMBOL_BLOCKLIST",
            RejectReason::RateLimit { .. } => "RATE_LIMIT",
            RejectReason::LeverageExceeded { .. } => "LEVERAGE_LIMIT",
            RejectReason::NotionalExceeded { .. } => "NOTIONAL_LIMIT",
            RejectReason::PositionLimitExceeded { .. } => "POSITION_LIMIT",
            RejectReason::ExposureLimitExceeded { .. } => "EXPOSURE_LIMIT",
            RejectReason::ConcentrationExceeded { .. } => "CONCENTRATION_LIMIT",
            RejectReason::InsufficientMargin { .. } => "MARGIN_CHECK",
            RejectReason::DailyLossExceeded { .. } => "DAILY_LOSS_LIMIT",
            RejectReason::DrawdownExceeded { .. } => "DRAWDOWN_LIMIT",
            RejectReason::StaleMark { .. } => "STALE_MARK",
            RejectReason::ParametricRiskUnavailable => "PARAMETRIC_RISK_UNAVAILABLE",
            RejectReason::LiquidityLimitExceeded { .. } => "LIQUIDITY_LIMIT",
            RejectReason::LiquidationProximity { .. } => "LIQUIDATION_PROXIMITY",
        }
    }

    /// İnsan okunur açıklama.
    pub fn describe(&self) -> String {
        match self {
            RejectReason::KillSwitch => "kill switch açık".to_string(),
            RejectReason::CircuitBreaker => "circuit breaker tetiklendi".to_string(),
            RejectReason::BlockedSymbol(s) => format!("{s} blocklist'te"),
            RejectReason::RateLimit { limit } => format!("dakikada {limit} emir limiti doldu"),
            RejectReason::LeverageExceeded { max } => format!("kaldıraç üst sınır {max}x aşıldı"),
            RejectReason::NotionalExceeded { notional, max } => {
                format!("notional {notional} USDT, üst sınır {max} USDT aşıldı")
            }
            RejectReason::PositionLimitExceeded { symbol, current_notional, max } => {
                format!("{symbol} pozisyon notional'ı {current_notional} USDT, sınır {max} USDT")
            }
            RejectReason::ExposureLimitExceeded { gross, max } => {
                format!("brüt exposure {gross} USDT, sınır {max} USDT aşıldı")
            }
            RejectReason::ConcentrationExceeded { hhi, max } => {
                format!("konsantrasyon HHI {hhi:.4}, sınır {max:.4}")
            }
            RejectReason::InsufficientMargin { required, available } => {
                format!("marj gerekli {required} USDT, mevcut {available} USDT")
            }
            RejectReason::DailyLossExceeded { loss, limit } => {
                format!("günlük kayıp {loss} USDT, sınır {limit} USDT")
            }
            RejectReason::DrawdownExceeded { drawdown_pct, max } => {
                format!("drawdown %{drawdown_pct:.2}, sınır %{max:.2}")
            }
            RejectReason::StaleMark { symbol, age_ms } => {
                format!("{symbol} mark fiyatı bayat ({age_ms}ms > eşik)")
            }
            RejectReason::ParametricRiskUnavailable => {
                "parametrik risk modeli kullanılamıyor (fail-closed)".to_string()
            }
            RejectReason::LiquidityLimitExceeded { slippage_bps, max } => {
                format!("slippage {slippage_bps} bps, sınır {max} bps")
            }
            RejectReason::LiquidationProximity { symbol } => {
                format!("{symbol} likidasyon fiyatına yakın")
            }
        }
    }
}

/// Portföy risk durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RiskStatus {
    Ok,
    MaxDrawdownBreached,
    MaxDailyLossBreached,
    MaxLeverageBreached,
    Liquidation,
    ParametricRiskUnavailable,
}

impl RiskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskStatus::Ok => "OK",
            RiskStatus::MaxDrawdownBreached => "MAX_DRAWDOWN_BREACHED",
            RiskStatus::MaxDailyLossBreached => "MAX_DAILY_LOSS_BREACHED",
            RiskStatus::MaxLeverageBreached => "MAX_LEVERAGE_BREACHED",
            RiskStatus::Liquidation => "LIQUIDATION",
            RiskStatus::ParametricRiskUnavailable => "PARAMETRIC_RISK_UNAVAILABLE",
        }
    }

    /// Emir girişini engelleyen kalıcı durumlar.
    pub fn halts_trading(&self) -> bool {
        matches!(
            self,
            RiskStatus::MaxDrawdownBreached
                | RiskStatus::MaxDailyLossBreached
                | RiskStatus::MaxLeverageBreached
                | RiskStatus::Liquidation
        )
    }
}

/// Gerçekleşen bir dolum (fill) — pozisyon/PnL muhasebesini günceller.
#[derive(Debug, Clone)]
pub struct Fill {
    pub symbol: String,
    pub side: Side,
    /// Baz-coin cinsinden pozitif dolu miktar.
    pub quantity: Decimal,
    pub price: Decimal,
    pub commission: Decimal,
    pub leverage: Decimal,
    pub ts_ms: u64,
}

/// Mark fiyat güncellemesi — unrealized PnL, drawdown, likidasyon kontrolü.
#[derive(Debug, Clone)]
pub struct MarkPrice {
    pub symbol: String,
    pub price: Decimal,
    pub ts_ms: u64,
}

impl MarkPrice {
    pub fn new(symbol: impl Into<String>, price: Decimal, ts_ms: u64) -> Self {
        Self {
            symbol: symbol.into(),
            price,
            ts_ms,
        }
    }
}
