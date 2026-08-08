//! Anthropic Messages API istemcisi (JSON structured output).

use super::{LlmError, LlmProvider};
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    temperature: f64,
    max_tokens: u32,
    timeout: Duration,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(
        api_key: String,
        model: String,
        temperature: f64,
        max_tokens: u32,
        timeout: Duration,
    ) -> Self {
        Self {
            api_key,
            model,
            temperature,
            max_tokens,
            timeout,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    async fn complete_json(&self, system: &str, user: &str) -> Result<serde_json::Value, LlmError> {
        let body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": system,
            "temperature": self.temperature,
            "messages": [
                { "role": "user", "content": format!("{user}\n\nTek JSON nesnesiyle yanıtla.") }
            ]
        });

        let fut = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send();

        let resp = tokio::time::timeout(self.timeout, fut)
            .await
            .map_err(|_| LlmError::Timeout)?
            .map_err(|e| LlmError::Http(e.to_string()))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| LlmError::Http(e.to_string()))?;
        if !status.is_success() {
            return Err(LlmError::Status { status, body: text });
        }

        let v: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| LlmError::Parse(e.to_string()))?;
        let content = v["content"][0]["text"]
            .as_str()
            .ok_or_else(|| LlmError::Parse("content[0].text eksik".into()))?;
        serde_json::from_str(content).map_err(|e| LlmError::Parse(e.to_string()))
    }
}
