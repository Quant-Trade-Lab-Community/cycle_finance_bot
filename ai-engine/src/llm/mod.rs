//! LLM provider soyutlaması — OpenAI ve Anthropic.
//!
//! Her provider JSON-schema kısıtlı (structured) çıktı üretir; agent'lar
//! `complete_json` sonucunu kendi tiplerine çözer. LLM yoksa (`none`) agent'lar
//! fail-safe varsayılanlara döner — asla kör emir üretilmez.

pub mod anthropic;
pub mod openai;

use crate::config::AiConfig;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Provider hataları.
#[derive(Debug)]
pub enum LlmError {
    NoProvider,
    Http(String),
    Status { status: reqwest::StatusCode, body: String },
    Parse(String),
    Timeout,
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::NoProvider => write!(f, "LLM provider tanımlı değil"),
            LlmError::Http(e) => write!(f, "HTTP hatası: {e}"),
            LlmError::Status { status, body } => write!(f, "LLM {status}: {body}"),
            LlmError::Parse(e) => write!(f, "yanıt ayrıştırılamadı: {e}"),
            LlmError::Timeout => write!(f, "LLM istek zaman aşımı"),
        }
    }
}

impl std::error::Error for LlmError {}

/// Yapılandırılmış JSON döndüren LLM istemcisi.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &'static str;
    /// System + user prompt verilir, tek JSON nesnesi döner.
    async fn complete_json(&self, system: &str, user: &str) -> Result<serde_json::Value, LlmError>;
}

/// Config + env anahtarlarına göre provider üretir.
/// `none` veya anahtar eksikse `None` → agent'lar varsayılana döner.
pub fn make_provider(cfg: &AiConfig) -> Option<Arc<dyn LlmProvider>> {
    let timeout = Duration::from_secs(cfg.providers.timeout_secs);
    match cfg.providers.provider.to_ascii_lowercase().as_str() {
        "openai" => {
            let key = AiConfig::openai_api_key()?;
            Some(Arc::new(openai::OpenAiProvider::new(
                key,
                cfg.providers.openai_model.clone(),
                cfg.providers.temperature,
                cfg.providers.max_tokens,
                timeout,
            )))
        }
        "anthropic" => {
            let key = AiConfig::anthropic_api_key()?;
            Some(Arc::new(anthropic::AnthropicProvider::new(
                key,
                cfg.providers.anthropic_model.clone(),
                cfg.providers.temperature,
                cfg.providers.max_tokens,
                timeout,
            )))
        }
        _ => None,
    }
}
