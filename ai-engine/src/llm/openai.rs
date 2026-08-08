//! OpenAI Chat Completions istemcisi (JSON object structured output).

use super::{LlmError, LlmProvider};
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    temperature: f64,
    max_tokens: u32,
    timeout: Duration,
    client: reqwest::Client,
}

impl OpenAiProvider {
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
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    async fn complete_json(&self, system: &str, user: &str) -> Result<serde_json::Value, LlmError> {
        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ],
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
            "response_format": { "type": "json_object" }
        });

        let fut = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
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
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::Parse("choices[0].message.content eksik".into()))?;
        serde_json::from_str(content).map_err(|e| LlmError::Parse(e.to_string()))
    }
}
