//! Koordinatör agent'ı — sinyal/risk/duygu çıktılarını sentezleyip nihai kararı üretir.
//! Risk agent'ının vetosu her zaman önceliklidir (fail-safe).

use super::{clamp_confidence, parse_action};
use crate::llm::LlmProvider;
use crate::{Action, FinalDecision, MarketContext, RiskOutput, SentimentOutput, SignalOutput, now_ms};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;

const SYSTEM_PROMPT: &str = r#"Sen Cycle Finance'in baş koordinatörüsün.
Strateji analisti, risk analisti ve sentiment analistinin çıktılarını tek karara indirge.
Kurallar:
1. risk analisti VETO=true verdiyse karar HOLD olmalı, quantity 0.
2. Strateji BUY/SELL ve güven >= 0.5 ise onaylayabilirsin; altında HOLD.
3. Sentiment strateji yönüyle zıtsa quantity'yi küçült (veya HOLD).
4. Fiyat/yapı riskliyse HOLD.
Şu JSON şemasına BİREBİR uy, başka hiçbir şey yazma:
{"action":"BUY|SELL|HOLD","confidence":0.0..1.0,"quantity":sayı_pozitif,"target_price":sayı_veya_null,"stop_loss":sayı_veya_null,"rationale":"kısa Türkçe gerekçe"}"#;

pub struct Coordinator {
    provider: Option<Arc<dyn LlmProvider>>,
}

impl Coordinator {
    pub fn new(provider: Option<Arc<dyn LlmProvider>>) -> Self {
        Self { provider }
    }

    pub async fn decide(
        &self,
        ctx: &MarketContext,
        signal: &SignalOutput,
        risk: &RiskOutput,
        sentiment: &SentimentOutput,
    ) -> FinalDecision {
        match &self.provider {
            Some(p) => match self.llm_decide(p, ctx, signal, risk, sentiment).await {
                Ok(mut d) => {
                    // Risk vetosu her zaman öncelikli.
                    if risk.veto {
                        d.action = Action::Hold;
                        d.quantity = Decimal::ZERO;
                        d.rationale = format!("RISK VETO — {}", risk.flags.join(", "));
                    }
                    d
                }
                Err(e) => {
                    eprintln!("⚠️  [coordinator] LLM hatası: {e}");
                    fallback(signal, risk, sentiment, &ctx.price.symbol)
                }
            },
            None => fallback(signal, risk, sentiment, &ctx.price.symbol),
        }
    }

    async fn llm_decide(
        &self,
        provider: &Arc<dyn LlmProvider>,
        ctx: &MarketContext,
        signal: &SignalOutput,
        risk: &RiskOutput,
        sentiment: &SentimentOutput,
    ) -> Result<FinalDecision, crate::llm::LlmError> {
        let input = serde_json::json!({
            "baglam": ctx.to_compact_json(),
            "strateji": signal,
            "risk": risk,
            "sentiment": sentiment,
        });
        let user = format!("AGENT ÇIKTILARI (JSON):\n{}\n\nNİHAİ KARAR (JSON):", serde_json::to_string(&input).unwrap_or_default());
        let v = provider.complete_json(SYSTEM_PROMPT, &user).await?;
        Ok(parse_final(&v, &ctx.price.symbol))
    }
}

fn fallback(signal: &SignalOutput, risk: &RiskOutput, sentiment: &SentimentOutput, symbol: &str) -> FinalDecision {
    let veto = risk.veto;
    let trade = signal.action.is_trade() && signal.confidence >= 0.5 && !signal.quantity.is_zero();

    if veto || !trade {
        return FinalDecision::hold(
            symbol,
            if veto {
                "RISK VETO (deterministik) — emir gönderilmedi"
            } else {
                "Düşük güven veya HOLD (deterministik)"
            },
        );
    }

    // Sentiment zıtsa miktarı %50 azalt.
    let mut qty = signal.quantity;
    if sentiment.sentiment < -0.3 {
        qty = qty * Decimal::new(5, 1);
    }

    FinalDecision {
        symbol: symbol.to_string(),
        action: signal.action,
        confidence: signal.confidence,
        quantity: qty,
        target_price: signal.target_price,
        stop_loss: signal.stop_loss,
        risk_score: risk.risk_score,
        sentiment: sentiment.sentiment,
        veto,
        rationale: signal.rationale.clone(),
        ts_ms: now_ms(),
    }
}

#[derive(Default, Deserialize)]
struct RawFinal {
    action: Option<String>,
    confidence: Option<f64>,
    quantity: Option<f64>,
    target_price: Option<f64>,
    stop_loss: Option<f64>,
    rationale: Option<String>,
}

fn parse_final(v: &serde_json::Value, symbol: &str) -> FinalDecision {
    let raw: RawFinal = serde_json::from_value(v.clone()).unwrap_or_default();
    let action = raw.action.as_deref().map(parse_action).unwrap_or(Action::Hold);
    let quantity = raw.quantity.and_then(Decimal::from_f64).unwrap_or(Decimal::ZERO).abs();
    let action = if quantity.is_zero() { Action::Hold } else { action };

    FinalDecision {
        symbol: symbol.to_string(),
        action,
        confidence: clamp_confidence(raw.confidence),
        quantity,
        target_price: raw.target_price.and_then(Decimal::from_f64),
        stop_loss: raw.stop_loss.and_then(Decimal::from_f64),
        risk_score: 0.5,
        sentiment: 0.0,
        veto: false,
        rationale: raw.rationale.unwrap_or_else(|| "—".into()),
        ts_ms: now_ms(),
    }
}
