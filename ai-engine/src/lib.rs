//! AI Agent Engine — Cycle Finance yapay zeka katmanı.
//!
//! LLM agent'ları mevcut altyapıdan (ring'ler + REST servisleri) bağlam toplar,
//! strateji/risk/duygu analizi yapar, koordinatör kararı sentezler ve emri
//! risk kapısından geçirip paper (order ring) veya canlı (executiond) icra eder.

pub mod agents;
pub mod config;
pub mod context;
pub mod executor;
pub mod gates;
pub mod llm;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Sinyal yönü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    Buy,
    Sell,
    Hold,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Buy => "BUY",
            Action::Sell => "SELL",
            Action::Hold => "HOLD",
        }
    }

    pub fn is_trade(&self) -> bool {
        !matches!(self, Action::Hold)
    }
}

/// Anlık fiyat anlık görüntüsü (price-feed kaynağı).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub symbol: String,
    pub last: f64,
    pub mark: f64,
    pub bid: f64,
    pub ask: f64,
    pub ts: u64,
}

/// İndikatör özeti (calc-ind / ferro_ta_core).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndicatorSnapshot {
    pub symbol: String,
    pub rsi: Option<f64>,
    pub macd: Option<f64>,
    pub macd_signal: Option<f64>,
    pub bbands_upper: Option<f64>,
    pub bbands_middle: Option<f64>,
    pub bbands_lower: Option<f64>,
    pub vwap: Option<f64>,
    pub atr: Option<f64>,
    pub sma20: Option<f64>,
    pub ema50: Option<f64>,
}

/// Piyasa yapısı özeti (detect-ms MSMP 2.0).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructureSnapshot {
    pub symbol: String,
    pub ats: Option<f64>,
    pub hurst: Option<f64>,
    pub r_squared: Option<f64>,
    pub trend_label: Option<String>,
    pub confluence_index: Option<f64>,
    pub vwap: Option<f64>,
    pub poc: Option<f64>,
    pub bsl_ssl_ratio: Option<f64>,
    pub atr: Option<f64>,
    pub levels: Vec<String>,
    pub current_price: Option<f64>,
}

/// Açık pozisyon özeti (paper/executiond kaynağı).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionSummary {
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub unrealized_pnl: f64,
}

/// Hesap özeti.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountSnapshot {
    pub equity: Option<f64>,
    pub cash_balance: Option<f64>,
    pub positions: Vec<PositionSummary>,
}

/// Tek sembol için agent'lara verilen birleşik bağlam.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketContext {
    pub generated_at_ms: u64,
    pub price: PriceSnapshot,
    pub indicators: IndicatorSnapshot,
    pub structure: StructureSnapshot,
    pub account: Option<AccountSnapshot>,
    pub recent_news: Vec<String>,
}

impl MarketContext {
    /// Fiyat kaynağı sağlıklı mı? (değilse agent'ları çalıştırma)
    pub fn is_healthy(&self) -> bool {
        self.price.last > 0.0 && self.price.ts > 0
    }

    /// Agent'lara token-verimli tek JSON satırı.
    pub fn to_compact_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".into())
    }
}

/// Strateji/sinyal agent'ı çıktısı.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalOutput {
    pub symbol: String,
    pub action: Action,
    /// 0.0 .. 1.0
    pub confidence: f64,
    /// Baz-coin cinsinden miktar.
    pub quantity: Decimal,
    #[serde(default)]
    pub target_price: Option<Decimal>,
    #[serde(default)]
    pub stop_loss: Option<Decimal>,
    pub rationale: String,
}

impl Default for SignalOutput {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            action: Action::Hold,
            confidence: 0.0,
            quantity: Decimal::ZERO,
            target_price: None,
            stop_loss: None,
            rationale: "LLM kullanılamıyor — beklemede".into(),
        }
    }
}

/// Risk/anomali agent'ı çıktısı.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskOutput {
    /// 0.0 (çok güvenli) .. 1.0 (çok riskli)
    pub risk_score: f64,
    /// true ise koordinatör kararı iptal edilir (fail-safe).
    pub veto: bool,
    /// Maksimum emir boyutu (baz puan; 10000 = %100).
    pub max_size_bps: Option<u32>,
    pub flags: Vec<String>,
}

/// Duygu/sentiment agent'ı çıktısı.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentimentOutput {
    /// -1.0 (çok negatif) .. +1.0 (çok pozitif)
    pub sentiment: f64,
    pub trending_terms: Vec<String>,
    pub bias: String,
}

/// Koordinatörün ürettiği nihai karar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalDecision {
    pub symbol: String,
    pub action: Action,
    pub confidence: f64,
    pub quantity: Decimal,
    #[serde(default)]
    pub target_price: Option<Decimal>,
    #[serde(default)]
    pub stop_loss: Option<Decimal>,
    pub risk_score: f64,
    pub sentiment: f64,
    /// true ise emir gönderilmez.
    pub veto: bool,
    pub rationale: String,
    pub ts_ms: u64,
}

impl FinalDecision {
    pub fn hold(symbol: &str, rationale: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            action: Action::Hold,
            confidence: 0.0,
            quantity: Decimal::ZERO,
            target_price: None,
            stop_loss: None,
            risk_score: 0.5,
            sentiment: 0.0,
            veto: false,
            rationale: rationale.to_string(),
            ts_ms: now_ms(),
        }
    }
}

/// Unix epoch milisaniye.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
