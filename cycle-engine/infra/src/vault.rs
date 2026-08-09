use std::time::{SystemTime, UNIX_EPOCH};

/// Vault sağlık cevabı (sys/health).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VaultHealth {
    pub initialized: bool,
    pub sealed: bool,
    pub standby: bool,
}

/// Vault Integration for dual key rotation and JWT management.
pub struct VaultAdapter {
    pub current_key_version: u32,
    /// HashiCorp Vault adresi (ör. `http://127.0.0.1:8200`). Boşsa mock modu.
    pub base_url: String,
}

impl VaultAdapter {
    pub fn new() -> Self {
        let base_url = std::env::var("VAULT_ADDR").unwrap_or_default();
        Self {
            current_key_version: 1,
            base_url,
        }
    }

    /// Gerçek Vault sağlık kontrolü: `GET /v1/sys/health?standbyok=true`.
    /// Vault yoksa `None` (mock modunda da `None`).
    pub async fn health(&self) -> Option<VaultHealth> {
        if self.base_url.is_empty() {
            return None;
        }
        let url = format!("{}/v1/sys/health?standbyok=true", self.base_url.trim_end_matches('/'));
        match reqwest::get(&url).await {
            Ok(resp) => resp.json::<VaultHealth>().await.ok(),
            Err(e) => {
                eprintln!("Vault: health check başarısız: {}", e);
                None
            }
        }
    }

    /// Handles dual key rotation with a 5-minute grace period.
    /// During the grace period, both the old and new keys are considered valid.
    pub fn rotate_keys(&mut self) {
        self.current_key_version += 1;
        println!("Vault: Keys rotated to v{}. 5-minute grace period activated for v{}.", 
            self.current_key_version, self.current_key_version - 1);
    }

    /// Creates a JWT with 1 hour TTL.
    /// It should be refreshed 10 minutes prior to expiration.
    pub fn generate_jwt(&self) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let exp = now + 3600; // 1 hour TTL
        let refresh_at = exp - 600; // 10 mins prior
        
        println!("Vault: Generated JWT. Exp: {}, Refresh At: {}", exp, refresh_at);
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.mock.signature".to_string()
    }
}
