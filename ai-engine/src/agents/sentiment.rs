//! Duygu/sentiment agent'ı — dış haber kaynağından piyasa duyarlılığını çıkarır.
//! Haber yoksa veya LLM kapalıysa nötr (0.0) döner.

use super::{Agent, AgentOutput, AgentRole};
use crate::llm::LlmProvider;
use crate::{MarketContext, SentimentOutput};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

const SYSTEM_PROMPT: &str = r#"Sen kripto haber sentiment analistisin.
Verilen haber başlıklarından piyasa duyarlılığını -1.0..+1.0 arası ölçekte ver.
(+1 çok pozitif, -1 çok negatif). Şu JSON şemasına BİREBİR uy:
{"sentiment":-1.0..1.0,"trending_terms":["anahtar kelimeler"],"bias":"bulut|boğa|nötr|ayı"}"#;

pub struct SentimentAgent {
    provider: Option<Arc<dyn LlmProvider>>,
}

impl SentimentAgent {
    pub fn new(provider: Option<Arc<dyn LlmProvider>>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Agent for SentimentAgent {
    fn id(&self) -> u32 {
        3
    }

    fn role(&self) -> AgentRole {
        AgentRole::Sentiment
    }

    async fn run(&self, ctx: &MarketContext) -> AgentOutput {
        let out = match &self.provider {
            Some(p) if !ctx.recent_news.is_empty() => {
                let news = ctx.recent_news.join("\n- ");
                match p
                    .complete_json(SYSTEM_PROMPT, &format!("HABERLER:\n- {news}\n\nSENTIMENT (JSON):"))
                    .await
                {
                    Ok(v) => parse_sentiment(&v),
                    Err(e) => {
                        eprintln!("⚠️  [sentiment] LLM hatası: {e}");
                        SentimentOutput::default()
                    }
                }
            }
            _ => SentimentOutput::default(),
        };
        AgentOutput::Sentiment(out)
    }
}

#[derive(Default, Deserialize)]
struct RawSentiment {
    sentiment: Option<f64>,
    trending_terms: Option<Vec<String>>,
    bias: Option<String>,
}

fn parse_sentiment(v: &serde_json::Value) -> SentimentOutput {
    let raw: RawSentiment = serde_json::from_value(v.clone()).unwrap_or_default();
    SentimentOutput {
        sentiment: raw.sentiment.unwrap_or(0.0).clamp(-1.0, 1.0),
        trending_terms: raw.trending_terms.unwrap_or_default(),
        bias: raw.bias.unwrap_or_else(|| "nötr".into()),
    }
}
