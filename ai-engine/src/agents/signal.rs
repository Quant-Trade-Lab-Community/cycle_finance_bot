//! Strateji/sinyal agent'ı — fiyat, indikatör ve yapı bağlamından alım/satım kararı.

use super::{Agent, AgentOutput, AgentRole, clamp_confidence, parse_action};
use crate::llm::LlmProvider;
use crate::{Action, MarketContext, SignalOutput};
use async_trait::async_trait;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;

const SYSTEM_PROMPT: &str = r#"Sen Cycle Finance'in strateji/sinyal analistisin.
Verilen piyasa bağlamına göre BTC/ETH/SOL/HEIUSDT vadeli işlem için yön kararı ver.
Sadece yüksek güvenli fırsatlarda BUY/SELL ver; belirsizlikte HOLD.
Kural: yapı (detect-ms ats/trend) ile indikatörler (rsi/macd/bbands/vwap/atr) çelişiyorsa HOLD.
Şu JSON şemasına BİREBİR uy, başka hiçbir şey yazma:
{"action":"BUY|SELL|HOLD","confidence":0.0..1.0,"quantity":sayı_pozitif,"target_price":sayı_veya_null,"stop_loss":sayı_veya_null,"rationale":"kısa Türkçe gerekçe"}"#;

pub struct SignalAgent {
    provider: Option<Arc<dyn LlmProvider>>,
    symbol: String,
}

impl SignalAgent {
    pub fn new(provider: Option<Arc<dyn LlmProvider>>, symbol: &str) -> Self {
        Self {
            provider,
            symbol: symbol.to_string(),
        }
    }
}

#[async_trait]
impl Agent for SignalAgent {
    fn id(&self) -> u32 {
        1
    }

    fn role(&self) -> AgentRole {
        AgentRole::Signal
    }

    async fn run(&self, ctx: &MarketContext) -> AgentOutput {
        let mut out = SignalOutput::default();
        out.symbol = self.symbol.clone();
        let out = match &self.provider {
            Some(p) => self.llm_run(p, ctx).await,
            None => out,
        };
        AgentOutput::Signal(out)
    }
}

impl SignalAgent {
    async fn llm_run(&self, provider: &Arc<dyn LlmProvider>, ctx: &MarketContext) -> SignalOutput {
        let user = format!(
            "SEMBOL: {}\nBAĞLAM (JSON):\n{}\n\nKARAR (JSON şemasına uy):",
            self.symbol,
            ctx.to_compact_json()
        );
        match provider.complete_json(SYSTEM_PROMPT, &user).await {
            Ok(v) => parse_signal(&v, &self.symbol),
            Err(e) => {
                eprintln!("⚠️  [signal] LLM hatası: {e}");
                let mut s = SignalOutput::default();
                s.symbol = self.symbol.clone();
                s
            }
        }
    }
}

#[derive(Default, Deserialize)]
struct RawSignal {
    action: Option<String>,
    confidence: Option<f64>,
    quantity: Option<f64>,
    target_price: Option<f64>,
    stop_loss: Option<f64>,
    rationale: Option<String>,
}

fn parse_signal(v: &serde_json::Value, symbol: &str) -> SignalOutput {
    let raw: RawSignal = serde_json::from_value(v.clone()).unwrap_or_default();
    let action = raw
        .action
        .as_deref()
        .map(parse_action)
        .unwrap_or(Action::Hold);
    let quantity = raw
        .quantity
        .and_then(Decimal::from_f64)
        .unwrap_or(Decimal::ZERO)
        .abs();

    // Miktar 0 ise emir göndermeyi anlamsız kıl → HOLD.
    let action = if quantity.is_zero() { Action::Hold } else { action };

    SignalOutput {
        symbol: symbol.to_string(),
        action,
        confidence: clamp_confidence(raw.confidence),
        quantity,
        target_price: raw.target_price.and_then(Decimal::from_f64),
        stop_loss: raw.stop_loss.and_then(Decimal::from_f64),
        rationale: raw.rationale.unwrap_or_else(|| "—".into()),
    }
}
