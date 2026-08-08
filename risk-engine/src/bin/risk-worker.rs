//! risk-worker — bağımsız risk parametre üretici daemon (cold path).
//!
//! Her çevrimde (varsayılan 60s):
//!   1. `/tmp/price_feed.json` (price-feed çıktısı) veya HTTP'den mark fiyatları okur
//!   2. Sembol getiri serilerini toplar
//!   3. Korelasyon → Tikhonov → EWMA vol → parametrik VaR → konsantrasyon hesaplar
//!   4. Önerilen limitleri `RiskCache` + `/cycle_finance_risk_params` ring + `/tmp/risk_params.json`'a yazar
//!
//! REST:
//!   GET  /healthz             → durum
//!   GET  /api/risk/snapshot   → model parametreleri + politika
//!   PUT  /api/risk/kill-switch {enabled} → acil durdurma

use axum::extract::State;
use axum::routing::{get, put};
use axum::{Json, Router};
use risk_engine::cache::{RiskCache, RiskParameters};
use risk_engine::config::{load_risk_config, resolve_risk_config_path, ConfigWatcher};
use risk_engine::kill_switch::KillSwitch;
use risk_engine::worker::{RiskWorker, WorkerConfig};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const RING_NAME: &str = "/cycle_finance_risk_params";
const RING_CAPACITY: usize = 1_024;
const PRICE_FILE: &str = "/tmp/price_feed.json";
const PARAMS_FILE: &str = "/tmp/risk_params.json";

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// `/tmp/price_feed.json`'dan mark fiyatlarını okur.
fn read_marks() -> HashMap<String, f64> {
    let content = match std::fs::read_to_string(PRICE_FILE) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let doc: serde_json::Value = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => return HashMap::new(),
    };
    let mut out = HashMap::new();
    if let Some(prices) = doc.get("prices").and_then(|p| p.as_object()) {
        for (sym, v) in prices {
            let mark = v.get("mark").and_then(|m| m.as_f64());
            let last = v.get("last").and_then(|m| m.as_f64());
            let price = mark.or(last);
            if let Some(p) = price {
                if p > 0.0 {
                    out.insert(sym.to_uppercase(), p);
                }
            }
        }
    }
    out
}

/// Parametreleri ring'e (compact JSON) ve dosyaya yazar.
fn publish(params: &RiskParameters) {
    let body = serde_json::json!({
        "version": 1,
        "computed_at_ms": params.computed_at_ms,
        "n_symbols": params.n_symbols,
        "portfolio_volatility": params.portfolio_volatility,
        "var_99_1d_pct": params.var_99_1d_pct,
        "correlation_condition": params.correlation_condition,
        "hhi": params.hhi,
        "suggested_max_position_usdt": params.suggested_max_position_usdt.to_string(),
        "suggested_max_leverage": params.suggested_max_leverage.to_string(),
        "available": params.available,
        "gate_ready": params.gate_ready,
    });
    let bytes = body.to_string();
    if bytes.len() <= 700 {
        let ring = transport::ring_buffer::GenerationalRingBuffer::with_name(RING_NAME, RING_CAPACITY);
        ring.push(bytes.as_bytes());
    }
    let _ = std::fs::write(PARAMS_FILE, body.to_string());
}

// ── Paylaşılan durum ──
struct AppState {
    cache: Arc<RiskCache>,
    kill_switch: Arc<KillSwitch>,
    policy: Arc<RwLock<risk_engine::policy::RiskPolicy>>,
    cycle_count: std::sync::atomic::AtomicU64,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    cycle_count: u64,
    last_params: LastParams,
    policy_path: String,
}

#[derive(Serialize, Clone, Default)]
struct LastParams {
    available: bool,
    n_symbols: usize,
    var_99_1d_pct: f64,
    correlation_condition: f64,
    hhi: f64,
    suggested_max_position_usdt: String,
    suggested_max_leverage: String,
    computed_at_ms: u64,
}

impl From<RiskParameters> for LastParams {
    fn from(p: RiskParameters) -> Self {
        Self {
            available: p.available,
            n_symbols: p.n_symbols,
            var_99_1d_pct: p.var_99_1d_pct,
            correlation_condition: p.correlation_condition,
            hhi: p.hhi,
            suggested_max_position_usdt: p.suggested_max_position_usdt.to_string(),
            suggested_max_leverage: p.suggested_max_leverage.to_string(),
            computed_at_ms: p.computed_at_ms,
        }
    }
}

async fn health(State(st): State<Arc<AppState>>) -> Json<Health> {
    Json(Health {
        status: "ok",
        cycle_count: st.cycle_count.load(std::sync::atomic::Ordering::Relaxed),
        last_params: st.cache.read().into(),
        policy_path: resolve_risk_config_path().display().to_string(),
    })
}

#[derive(Serialize)]
struct Snapshot {
    params: LastParams,
    policy: PolicyView,
    kill_switch: bool,
}

#[derive(Serialize)]
struct PolicyView {
    max_position_usdt: String,
    max_notional_per_order: String,
    max_gross_exposure_usdt: String,
    max_leverage: String,
    max_daily_loss_usdt: String,
    max_drawdown_pct: String,
    stale_mark_ms: u64,
    blocklist: Vec<String>,
}

impl From<risk_engine::policy::RiskPolicy> for PolicyView {
    fn from(p: risk_engine::policy::RiskPolicy) -> Self {
        let mut blocklist: Vec<String> = p.blocklist.iter().cloned().collect();
        blocklist.sort();
        Self {
            max_position_usdt: p.max_position_usdt.to_string(),
            max_notional_per_order: p.max_notional_per_order.to_string(),
            max_gross_exposure_usdt: p.max_gross_exposure_usdt.to_string(),
            max_leverage: p.max_leverage.to_string(),
            max_daily_loss_usdt: p.max_daily_loss_usdt.to_string(),
            max_drawdown_pct: p.max_drawdown_pct.to_string(),
            stale_mark_ms: p.stale_mark_ms,
            blocklist,
        }
    }
}

async fn snapshot(State(st): State<Arc<AppState>>) -> Json<Snapshot> {
    let policy = st.policy.read().await.clone();
    Json(Snapshot {
        params: st.cache.read().into(),
        policy: policy.into(),
        kill_switch: st.kill_switch.is_open(),
    })
}

#[derive(serde::Deserialize)]
struct KillSwitchReq {
    enabled: bool,
}

async fn set_kill_switch(
    State(st): State<Arc<AppState>>,
    Json(req): Json<KillSwitchReq>,
) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    let res = if req.enabled {
        st.kill_switch.engage()
    } else {
        st.kill_switch.release()
    };
    match res {
        Ok(()) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({ "kill_switch": req.enabled })),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

fn main() {
    let port: u16 = std::env::var("RISK_WORKER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3011);
    let cycle_sec: u64 = std::env::var("RISK_WORKER_INTERVAL_SEC")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let max_samples: usize = std::env::var("RISK_WORKER_MAX_SAMPLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120);

    let policy_path = resolve_risk_config_path();
    let policy = load_risk_config().unwrap_or_default();
    let mut watcher = ConfigWatcher::new(policy_path.clone());

    let cache = Arc::new(RiskCache::new());
    let kill_switch = Arc::new(KillSwitch::new(
        std::env::var("RISK_KILL_SWITCH_PATH")
            .unwrap_or_else(|_| "/tmp/exec_kill_switch".into()),
    ));
    let worker_cfg = WorkerConfig {
        max_samples,
        daily_loss_budget: policy.max_daily_loss_usdt,
        ..Default::default()
    };

    let shared_policy = Arc::new(tokio::sync::RwLock::new(policy.clone()));
    let app_state = Arc::new(AppState {
        cache: cache.clone(),
        kill_switch: kill_switch.clone(),
        policy: shared_policy.clone(),
        cycle_count: std::sync::atomic::AtomicU64::new(0),
    });

    // ── Döngü iş parçacığı ──
    {
        let cache = cache.clone();
        let shared_policy = shared_policy.clone();
        std::thread::spawn(move || {
            let mut worker = RiskWorker::new(worker_cfg, cache.clone());
            let mut cycle: u64 = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(cycle_sec));

                // Hot-reload: risk.toml değiştiyse politikayı güncelle.
                if let Some(new_policy) = watcher.reload_if_changed() {
                    let mut p = shared_policy.blocking_write();
                    *p = new_policy.clone();
                    // Günlük kayıp bütçesi politikadan izlenir.
                    worker.config.daily_loss_budget = new_policy.max_daily_loss_usdt;
                }

                let marks = read_marks();
                if marks.is_empty() {
                    // Fiyat yoksa parametreler "unavailable" kalır → fail-closed.
                    cache.write(RiskParameters::unavailable());
                    continue;
                }

                let mut syms: Vec<String> = marks.keys().cloned().collect();
                syms.sort();
                for s in &syms {
                    worker.ingest_mark(s, marks[s]);
                }

                let params = worker.run_cycle(now_ms());
                publish(&params);
                cycle += 1;
                tracing::info!(
                    cycle,
                    n = params.n_symbols,
                    available = params.available,
                    var = params.var_99_1d_pct,
                    "risk-worker çevrimi tamamlandı"
                );
            }
        });
    }

    // ── REST ──
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async move {
        let app = Router::new()
            .route("/healthz", get(health))
            .route("/api/risk/snapshot", get(snapshot))
            .route("/api/risk/kill-switch", put(set_kill_switch))
            .with_state(app_state);
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await.expect("port bind");
        println!("risk-worker: http://127.0.0.1:{port}/healthz (cycle={cycle_sec}s)");
        axum::serve(listener, app).await.expect("serve");
    });
}
