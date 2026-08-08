//! Canlı icra — executiond :3010 REST client (JWT auth + emir gönderimi).

use crate::config::AiConfig;
use crate::{Action, now_ms};
use rust_decimal::Decimal;
use serde_json::json;

pub struct LiveExecutor {
    client: reqwest::Client,
    url: String,
    user: String,
    pass: String,
}

impl LiveExecutor {
    pub fn new(cfg: &AiConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: cfg.execution.execd_url.clone(),
            user: cfg.execution.execd_user.clone(),
            pass: cfg.execution.execd_password.clone(),
        }
    }

    pub async fn execute(
        &self,
        symbol: &str,
        action: Action,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Result<String, String> {
        let token = self.login().await?;
        let body = json!({
            "symbol": symbol.to_uppercase(),
            "side": action.as_str(),
            "type": if price.is_some() { "LIMIT" } else { "MARKET" },
            "quantity": quantity.to_string(),
            "price": price.map(|p| p.to_string()),
            "client_order_id": format!("ai_{}", now_ms()),
            "reduce_only": false,
        });

        let resp = self
            .client
            .post(format!("{}/api/v1/orders", self.url))
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("executiond isteği başarısız: {e}"))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("yanıt okunamadı: {e}"))?;

        if status.is_success() {
            Ok(format!("✅ LIVE executiond: {status} {text}"))
        } else {
            Err(format!("❌ LIVE executiond {status}: {text}"))
        }
    }

    async fn login(&self) -> Result<String, String> {
        let resp = self
            .client
            .post(format!("{}/api/v1/auth/login", self.url))
            .json(&json!({ "username": self.user, "password": self.pass }))
            .send()
            .await
            .map_err(|e| format!("executiond login başarısız: {e}"))?;
        let status = resp.status();
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("login yanıtı ayrıştırılamadı: {e}"))?;
        if !status.is_success() {
            return Err(format!("executiond login {status}: {v}"));
        }
        v["access_token"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "login yanıtında access_token yok".into())
    }
}
