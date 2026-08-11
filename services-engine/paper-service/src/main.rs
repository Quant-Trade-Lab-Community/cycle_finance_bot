use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use execution_engine::paper::actor::PaperEngineActor;
use execution_engine::paper::config::PaperConfig;
use paper_service::api::{AppState, AuthState, EngineHandle};
use paper_service::bridge;
use paper_service::events::{self, DomainEvent};
use paper_service::idempotency::{IdempotencyCache, InMemoryIdempotencyCache};
use rand::rngs::OsRng;
use std::sync::Arc;
use tokio::sync::mpsc;

#[cfg(feature = "full")]
use paper_service::postgres_store::PostgresEventStore;

#[tokio::main]
async fn main() {
    let _ = infra::util::single_instance("paper-service");
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "paper_service=info,execution_engine=info".into()))
        .init();

    println!("========================================");
    println!("🛡️ PAPER SERVICE v2.0 (Event Sourcing + Actor Model)");
    println!("========================================");

    let config = PaperConfig::load_from_env();

    // ── Event Store (Sled WAL) + Replay ──
    let store = events::open_wal_store();
    let replay_events: Vec<DomainEvent> = {
        let guard = store.lock().unwrap();
        guard.replay()
    };
    if !replay_events.is_empty() {
        println!("[RECOVERY] {} event bulundu; state replay ediliyor...", replay_events.len());
    }

    #[cfg(feature = "full")]
    let postgres = match std::env::var("DATABASE_URL") {
        Ok(url) => match PostgresEventStore::connect(&url).await {
            Ok(pg) => {
                println!("[PG] PostgreSQL event store bağlandı: {}",
                         url.split('@').next().unwrap_or(url.as_str()));
                Some(pg)
            }
            Err(e) => {
                tracing::warn!("[PG] PostgreSQL bağlanamadı, Sled WAL yedekli: {}", e);
                None
            }
        },
        Err(_) => {
            println!("[PG] DATABASE_URL yok — PostgreSQL kapalı (Sled WAL aktif).");
            None
        }
    };

    // ── TEK olay kanalı: actor → Sled WAL + PostgreSQL + SQLite projection ──
    // Ayrı "persist" kanalı kaldırıldı: actor yalnızca DomainEvent üretir;
    // tüm tüketiciler (event store, PG, SQLite) bu tek akıştan beslenir.
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<DomainEvent>();
    let (sqlite_path, sqlite_batch_ms) = (config.db_path.clone(), config.batch_write_interval_ms);
    tokio::spawn(async move {
        let mut sqlite_conn = match paper_service::sqlite_projection::open_connection(&sqlite_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[SQLITE] Bağlantı başarısız ({}): {}", sqlite_path, e);
                paper_service::sqlite_projection::open_connection("/dev/null").expect("fallback sqlite open")
            }
        };
        let mut projection = paper_service::sqlite_projection::SqliteProjection::new();
        let mut count: i64 = 0;
        let mut flush_interval = tokio::time::interval(std::time::Duration::from_millis(sqlite_batch_ms));

        loop {
            tokio::select! {
                _ = flush_interval.tick() => {
                    if let Err(e) = projection.flush(&mut sqlite_conn) {
                        eprintln!("[SQLITE] Flush hatası: {}", e);
                    }
                }
                Some(ev) = event_rx.recv() => {
                    {
                        let mut guard = store.lock().unwrap();
                        guard.append(&ev);
                    }
                    count += 1;
                    #[cfg(feature = "full")]
                    if let Some(pg) = &postgres {
                        let _ = pg.append(&ev).await;
                    }
                    projection.apply(&ev);
                    if count % 1000 == 0 {
                        tracing::info!("[WAL] Toplam {} event yazıldı.", count);
                    }
                }
                else => {
                    let _ = projection.flush(&mut sqlite_conn);
                    break;
                }
            }
        }
    });

    // ── Actor + engine handle ──
    let actor = PaperEngineActor::new_with_events(config, Some(event_tx), &replay_events);
    let snapshot = actor.snapshot_handle();

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        actor.run(cmd_rx).await;
    });

    // ── Auth (env'den kullanıcı, argon2 hash'li) ──
    let admin_user = std::env::var("PAPER_ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
    let admin_pass = std::env::var("PAPER_ADMIN_PASS").unwrap_or_else(|_| "changeme123".to_string());
    let salt = SaltString::generate(&mut OsRng);
    let pass_hash = Argon2::default()
        .hash_password(admin_pass.as_bytes(), &salt)
        .expect("hash admin password")
        .to_string();
    let auth = Arc::new(AuthState {
        secret: std::env::var("PAPER_JWT_SECRET").unwrap_or_else(|_| "paper-dev-secret-change-me".to_string()),
        admin_user,
        admin_pass_hash: pass_hash,
    });

    // ── REST API + idempotency ──
    let idempotency: Arc<dyn IdempotencyCache> = Arc::new(InMemoryIdempotencyCache::new());
    let engine_handle = EngineHandle {
        cmd_tx: cmd_tx.clone(),
        snapshot,
        idempotency,
    };
    let metrics = paper_service::metrics::Metrics::new();
    let app_state = Arc::new(AppState { engine: engine_handle, auth, metrics });
    let api_addr = std::env::var("PAPER_API_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let api_state = app_state.clone();
    let api_addr_clone = api_addr.clone();
    tokio::spawn(async move {
        paper_service::api::serve(&api_addr_clone, api_state).await;
    });

    // ── DATA (tick ring) ve STRATEGY (order ring) terminallerine bağlan ──
    bridge::spawn_ring_bridge(cmd_tx);

    println!("Paper service running.");
    println!("  REST API : http://{api_addr}/api/v1/system/health");
    println!("  Login    : POST /api/v1/auth/login (user: {})", app_state.auth.admin_user);

    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
    println!("Shutting down paper service...");
}
