//! HTTP servis katmanı: axum router + dinleme.

pub mod api;

use crate::gateway::EngineHandle;
use crate::metrics::Metrics;
use std::sync::Arc;

/// `EngineHandle` üzerine REST API'yi bind eder (bağımsız görev).
pub async fn serve(
    addr: &str,
    handle: EngineHandle,
    metrics: Arc<Metrics>,
    client: Option<Arc<crate::client::BinanceClient>>,
) {
    // cycle-engine dayanıklılık deseni: ikiz süreç → çift emir riskini önler.
    let _ = infra::util::single_instance("executiond");
    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};
    use rand::rngs::OsRng;

    let admin_user = std::env::var("EXEC_ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
    let admin_pass = std::env::var("EXEC_ADMIN_PASS").unwrap_or_else(|_| "changeme123".to_string());
    let salt = SaltString::generate(&mut OsRng);
    let pass_hash = Argon2::default()
        .hash_password(admin_pass.as_bytes(), &salt)
        .expect("hash admin password")
        .to_string();

    let auth = Arc::new(api::AuthState {
        secret: handle.config.jwt_secret.clone(),
        admin_user,
        admin_pass_hash: pass_hash,
    });

    let app_state = Arc::new(api::AppState {
        engine: handle,
        auth,
        metrics,
        client,
    });

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("REST API bind hatası {addr}: {e}");
            return;
        }
    };
    tracing::info!("Execution REST API dinliyor: http://{addr}");
    let app = api::router(app_state);
    axum::serve(listener, app).await.expect("axum serve");
}
