//! Risk/anomali agent'ı — piyasa yapısı ve indikatör bağlamından risk postürü üretir.
//! `veto=true` ise koordinatör kararı iptal edilir (fail-safe).

use super::{Agent, AgentOutput, AgentRole, clamp_risk};
use crate::llm::LlmProvider;
use crate::{MarketContext, RiskOutput};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

const SYSTEM_PROMPT: &str = r#"Sen Cycle Finance'in risk analistisin.
Verilen piyasa bağlamına göre risk skoru üret. Aşağıdaki durumlarda VETO=true döndür (fail-safe):
- aşırı volatilite (atr orantısız / fiyat çok hızlı hareketli),
- kritik seviyelere dayanma (fiyat, detect-ms seviyelerine çok yakın),
- anormal indikatör değerleri (rsi aşırı bölgelerde + aşırı geniş bantlar).
Şu JSON şemasına BİREBİR uy, başka hiçbir şey yazma:
{"risk_score":0.0..1.0,"veto":true|false,"max_size_bps":10000_veya_altı,"flags":["kısa etiketler"],"rationale":"kısa Türkçe gerekçe"}"#;

pub struct RiskAgent {
    provider: Option<Arc<dyn LlmProvider>>,
}

impl RiskAgent {
    pub fn new(provider: Option<Arc<dyn LlmProvider>>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Agent for RiskAgent {
    fn id(&self) -> u32 {
        2
    }

    fn role(&self) -> AgentRole {
        AgentRole::Risk
    }

    async fn run(&self, ctx: &MarketContext) -> AgentOutput {
        let out = match &self.provider {
            Some(p) => match p
                .complete_json(SYSTEM_PROMPT, &format!("BAĞLAM (JSON):\n{}", ctx.to_compact_json()))
                .await
            {
                Ok(v) => parse_risk(&v),
                Err(e) => {
                    eprintln!("⚠️  [risk] LLM hatası: {e}");
                    neutral_risk()
                }
            },
            None => neutral_risk(),
        };
        AgentOutput::Risk(out)
    }
}

/// LLM yoksa nötr (0.5) risk postürü — fail-open değil, tarafsız.
fn neutral_risk() -> RiskOutput {
    RiskOutput {
        risk_score: 0.5,
        veto: false,
        max_size_bps: None,
        flags: vec!["llm-off".into()],
    }
}

#[derive(Default, Deserialize)]
struct RawRisk {
    risk_score: Option<f64>,
    veto: Option<bool>,
    max_size_bps: Option<f64>,
    flags: Option<Vec<String>>,
    #[allow(dead_code)]
    rationale: Option<String>,
}

fn parse_risk(v: &serde_json::Value) -> RiskOutput {
    let raw: RawRisk = serde_json::from_value(v.clone()).unwrap_or_default();
    RiskOutput {
        risk_score: clamp_risk(raw.risk_score),
        veto: raw.veto.unwrap_or(false),
        max_size_bps: raw
            .max_size_bps
            .filter(|x| x.is_finite() && *x > 0.0)
            .map(|x| x.min(10_000.0) as u32),
        flags: raw.flags.unwrap_or_default(),
    }
}


