//! Agent'lar — her biri tek rol üstlenir, `MarketContext` alır, yapılandırılmış
//! çıktı üretir. LLM yoksa fail-safe varsayılana dönerler.

pub mod coordinator;
pub mod risk;
pub mod sentiment;
pub mod signal;

use crate::{Action, MarketContext, RiskOutput, SentimentOutput, SignalOutput};
use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    Signal,
    Risk,
    Sentiment,
    Coordinator,
}

impl AgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentRole::Signal => "SIGNAL",
            AgentRole::Risk => "RISK",
            AgentRole::Sentiment => "SENTIMENT",
            AgentRole::Coordinator => "COORDINATOR",
        }
    }
}

/// Bir agent'ın ürettiği çıktı.
#[derive(Debug)]
pub enum AgentOutput {
    Signal(SignalOutput),
    Risk(RiskOutput),
    Sentiment(SentimentOutput),
}

/// Ortak agent arayüzü.
#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> u32;
    fn role(&self) -> AgentRole;
    async fn run(&self, ctx: &MarketContext) -> AgentOutput;
}

/// JSON'daki yön string'ini `Action`'a çevirir.
pub(crate) fn parse_action(s: &str) -> Action {
    match s.trim().to_uppercase().as_str() {
        "BUY" | "LONG" => Action::Buy,
        "SELL" | "SHORT" => Action::Sell,
        _ => Action::Hold,
    }
}

pub(crate) fn clamp_confidence(x: Option<f64>) -> f64 {
    x.unwrap_or(0.0).clamp(0.0, 1.0)
}

pub(crate) fn clamp_risk(x: Option<f64>) -> f64 {
    x.unwrap_or(0.5).clamp(0.0, 1.0)
}
