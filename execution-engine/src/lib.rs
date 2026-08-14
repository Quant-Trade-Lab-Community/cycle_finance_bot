//! Execution Engine — Binance USDT-M Futures kurumsal emir yürütme katmanı.
//!
//! # Akış
//! ```text
//! strateji / REST API
//!      │ Command (mpsc, tek-yazıcı)
//!      ▼
//! ExecutionActor ──► BinanceClient (REST: emir/iptal/kontrol)
//!      │ ▲
//!      │ │ UserDataEvent (gzip WS)               periyodik uzlaştırma
//!      ▼ └── UserDataStream ── listenKey ──────── Binance user-data WS
//! AccountSnapshot (Arc<RwLock>) ◄── projector
//!      │
//!      ▼
//! REST API (axum) / stratejiler (okuma)
//! ```
//!
//! Güvenlik varsayılanları: `EXEC_DRY_RUN=true` (emir borsaya gitmez),
//! kill switch, max notional, sembol blocklist, idempotency (`newClientOrderId`),
//! ilk eşitleme tamamlanmadan emir kabul edilmez.

pub mod client;
pub mod config;
pub mod error;
pub mod execution;
pub mod gateway;
pub mod metrics;
pub mod order;
pub mod risk;
pub mod service;
pub mod signer;
pub mod state;
pub mod types;
pub mod user_data;

pub use config::{ExecConfig, TradingMode};
pub use error::{ExecError, Result};
pub use gateway::{EngineHandle, Gateway, LiveGateway};

use crate::client::BinanceClient;
use crate::execution::actor::ExecutionActor;
use crate::metrics::Metrics;
use crate::order::OrderRequest;
use crate::risk::checks::RiskChecks;
use crate::risk::kill_switch::KillSwitch;
use crate::state::exchange_cache::ExchangeCache;
use crate::state::snapshot::AccountSnapshot;
use crate::user_data::stream::UserDataStream;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

/// Canlı Binance Futures execution motoru.
pub struct ExecutionEngine {
    pub handle: EngineHandle,
    pub client: Arc<BinanceClient>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl ExecutionEngine {
    /// Motoru başlat: ilk saat senkronu, actor, user-data stream.
    pub async fn start(config: ExecConfig) -> Result<Arc<Self>> {
        let client = BinanceClient::new(&config)?;
        client.http.sync_server_time().await?;
        info!("Sunucu saati senkronize edildi");

        let metrics = Metrics::new();
        let kill_switch = Arc::new(KillSwitch::new(config.kill_switch_path.clone()));
        let snapshot = Arc::new(RwLock::new(AccountSnapshot::default()));
        let exchange = ExchangeCache::new(300);
        let risk = RiskChecks::with_kill_switch(&config, kill_switch.clone());

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<execution::actor::Command>();
        let (user_tx, user_rx) = mpsc::unbounded_channel::<execution::actor::UserEvent>();

        let actor = ExecutionActor::new(
            client.clone(),
            exchange,
            risk,
            kill_switch.clone(),
            snapshot.clone(),
            metrics.clone(),
            config.clone(),
            cmd_rx,
            user_rx,
        );
        let actor_task = tokio::spawn(actor.run());

        let stream = UserDataStream::new(client.clone(), config.clone(), user_tx);
        let stream_task = tokio::spawn(stream.run());

        let handle = EngineHandle {
            cmd_tx,
            snapshot,
            metrics: metrics.clone(),
            kill_switch,
            config: Arc::new(config.clone()),
        };

        Ok(Arc::new(Self {
            handle,
            client,
            tasks: vec![actor_task, stream_task],
        }))
    }

    /// REST API servisini ayrı görevde başlat.
    pub fn spawn_rest(self: &Arc<Self>, addr: &str) {
        let handle = self.handle.clone();
        let metrics = self.handle.metrics.clone();
        let client = Some(self.client.clone());
        let addr = addr.to_string();
        tokio::spawn(async move {
            service::serve(&addr, handle, metrics, client).await;
        });
    }

    pub async fn shutdown(&self) {
        for t in &self.tasks {
            t.abort();
        }
    }
}

/// Eski API uyumu: flume emir akışını EngineHandle'a köprüler.
pub async fn start_execution_engine(
    rx: flume::Receiver<OrderRequest>,
    api_key: String,
    secret_key: String,
) {
    use std::sync::OnceLock;
    static CONFIG: OnceLock<ExecConfig> = OnceLock::new();
    let _ = CONFIG.set({
        let mut c = ExecConfig::load_from_env();
        c.api_key = api_key;
        c.secret_key = secret_key;
        c
    });

    let engine = match ExecutionEngine::start(CONFIG.get().expect("config").clone()).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ExecutionEngine başlatılamadı: {e}");
            return;
        }
    };

    while let Ok(order) = rx.recv_async().await {
        match engine.handle.submit_order(order).await {
            Ok(ack) => println!("ExecutionEngine: emir kabul → {:?}", ack.status),
            Err(e) => println!("ExecutionEngine: emir reddedildi → {e}"),
        }
    }
}
