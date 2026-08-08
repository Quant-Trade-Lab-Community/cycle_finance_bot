// ============================================================================
// detect-trb — ORKESTRATÖR + REST API Servisi (:3006)
// ============================================================================
// İş parçacıkları:
//   1. http (tokio) → axum teşhis + status
//   2. canlı-akış (tokio) → rtrb producer (ring buffer → InflowData)
//   3. solver-oracle (std::thread, core-affinity) → NS + kavitasyon + TWAP
//
// Güvenlik: solver thread içindeki her analiz `catch_unwind` ile zırhlı —
// panik servisi durdurmaz, hata raporlanır ve bir sonraki çevrimde yeniden
// denenir (graceful degradation).
// ============================================================================

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use tracing_subscriber::EnvFilter;

use detect_trb::analyzer;
use detect_trb::ingest;
use detect_trb::types::{InflowData, TrbReport};

const DEFAULT_DB: &str = "data-engine/data/market_data.db";
const DEFAULT_SYMBOL: &str = "BTCUSDT";
const DEFAULT_INTERVAL_MS: u64 = 30_000;
const DEFAULT_LIMIT: usize = 500;
const DEFAULT_PORT: u16 = 3006;
const DEFAULT_REFRESH_SECS: u64 = 10;
const RING_CAPACITY: usize = 65_536;

// ================================================================
// CLI
// ================================================================

struct Cli {
    db: String,
    symbol: String,
    interval_ms: u64,
    limit: usize,
    port: u16,
    refresh_secs: u64,
}

fn parse_args() -> Cli {
    let mut cli = Cli {
        db: DEFAULT_DB.to_string(),
        symbol: DEFAULT_SYMBOL.to_string(),
        interval_ms: DEFAULT_INTERVAL_MS,
        limit: DEFAULT_LIMIT,
        port: DEFAULT_PORT,
        refresh_secs: DEFAULT_REFRESH_SECS,
    };

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--symbol" => {
                if let Some(v) = args.next() {
                    cli.symbol = v;
                }
            }
            "--interval-ms" => {
                if let Some(v) = args.next() {
                    cli.interval_ms = v.parse().unwrap_or(DEFAULT_INTERVAL_MS);
                }
            }
            "--limit" => {
                if let Some(v) = args.next() {
                    cli.limit = v.parse().unwrap_or(DEFAULT_LIMIT);
                }
            }
            "--db" => {
                if let Some(v) = args.next() {
                    cli.db = v;
                }
            }
            "--port" => {
                if let Some(v) = args.next() {
                    cli.port = v.parse().unwrap_or(DEFAULT_PORT);
                }
            }
            "--refresh" => {
                if let Some(v) = args.next() {
                    cli.refresh_secs = v.parse().unwrap_or(DEFAULT_REFRESH_SECS);
                }
            }
            "--help" | "-h" => {
                println!(
                    "Kullanım: detect-trb [--symbol BTCUSDT] [--interval-ms 30000] \
                     [--limit 500] [--db data-engine/data/market_data.db] [--port 3006] [--refresh 10]"
                );
                std::process::exit(0);
            }
            _ => {}
        }
    }
    cli
}

// ================================================================
// PAYLAŞILAN DURUM
// ================================================================

/// HTTP tarafının okuduğu son durum (snapshot)
#[derive(Clone, Serialize)]
struct Snapshot {
    last_updated_ms: Option<u128>,
    report: Option<TrbReport>,
    last_error: Option<String>,
    total_cycles: u64,
}

struct AppState {
    snapshot: Arc<Mutex<Snapshot>>,
}

// ================================================================
// ANA
// ================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = parse_args();

    println!("══════════════════════════════════════════════════════");
    println!("  🌊 TÜRBÜLANS / NAVIER-STOKES ANALİZ MOTORU");
    println!("      detect-trb v1.0 | PhaseSpace 64×16 | Thomas + Jacobi");
    println!("══════════════════════════════════════════════════════");
    println!(
        "  Sembol: {}  |  Aralık: {} ms  |  Limit: {}  |  Db: {}",
        cli.symbol, cli.interval_ms, cli.limit, cli.db
    );
    println!(
        "  API: http://127.0.0.1:{}/api/trb   (+ /api/trb/status)",
        cli.port
    );
    println!("══════════════════════════════════════════════════════");

    // rtrb: canlı akış kanalı (producer → consumer)
    let (mut producer, mut consumer) = rtrb::RingBuffer::<InflowData>::new(RING_CAPACITY);

    let snapshot = Arc::new(Mutex::new(Snapshot {
        last_updated_ms: None,
        report: None,
        last_error: None,
        total_cycles: 0,
    }));
    let app_state = Arc::new(AppState {
        snapshot: snapshot.clone(),
    });

    // ── 1. Canlı akış üreticisi (tokio task) ─────────────────────────
    let symbol_prod = cli.symbol.clone();
    tokio::spawn(async move {
        loop {
            let live = ingest::drain_ring_buffer(&symbol_prod, 4096);
            for d in live {
                let _ = producer.push(d); // doluysa sessizce bırak
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    // ── 2. Solver orkestratörü (core-pinned std::thread) ──────────────
    {
        let db = cli.db.clone();
        let symbol_cli = cli.symbol.clone();
        let interval_ms = cli.interval_ms;
        let limit = cli.limit;
        let refresh = cli.refresh_secs;
        let stats = snapshot.clone();

        std::thread::Builder::new()
            .name("trb-solver".to_string())
            .spawn(move || {
                // Core sabitleme (varsa — iyi niyetli, hata yutulur)
                if let Some(core) =
                    core_affinity::get_core_ids().and_then(|ids| ids.first().copied())
                {
                    let _ = core_affinity::set_for_current(core);
                }

                loop {
                    // rtrb'den biriken canlı veri (bloklayıcı pop — ring kuralı)
                    let mut live: Vec<InflowData> = Vec::new();
                    loop {
                        match consumer.pop() {
                            Ok(d) => live.push(d),
                            Err(_) => break, // geçici boş — devam et
                        }
                    }

                    let started = std::time::Instant::now();
                    let result = std::panic::catch_unwind(|| {
                        analyzer::analyze(&db, &symbol_cli, interval_ms, limit, &live)
                    });

                    let mut snap = stats.lock().unwrap_or_else(|p| p.into_inner());
                    snap.total_cycles += 1;
                    match result {
                        Ok(Ok(report)) => {
                            let steps = report.inflow_steps;
                            let div = report.solver_state.divergence_norm;
                            let burst_dir = report
                                .burst_signal
                                .as_ref()
                                .map(|b| b.direction.as_str())
                                .unwrap_or("akış normal")
                                .to_string();
                            snap.report = Some(report);
                            snap.last_error = None;
                            println!(
                                "✔ analiz {:.0}ms — {steps} adım, {burst}, divergence {div:.4}",
                                started.elapsed().as_millis(),
                                burst = burst_dir,
                                div = div,
                            );
                        }
                        Ok(Err(e)) => {
                            snap.last_error = Some(e.to_string());
                            eprintln!("✘ analiz hatası: {e}");
                        }
                        Err(p) => {
                            let msg = p
                                .downcast_ref::<&str>()
                                .map(|s| s.to_string())
                                .or_else(|| p.downcast_ref::<String>().map(|s| s.clone()))
                                .unwrap_or_else(|| "bilinmeyen panik".to_string());
                            snap.last_error = Some(format!("panik: {}", &msg));
                            eprintln!("✘ panik yakalandı: {msg} — servis ayakta, devam ediliyor");
                        }
                    }
                    snap.last_updated_ms = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis())
                            .unwrap_or(0),
                    );

                    std::thread::sleep(Duration::from_secs(refresh));
                }
            })
            .expect("trb-solver thread başlatılamadı");
    }

    // ── 3. HTTP daemon ─────────────────────────────────────────────────
    let app = Router::new()
        .route("/api/trb", get(get_report))
        .route("/api/trb/status", get(get_status))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| {
        eprintln!("Port {} bağlanamıyor: {e}", cli.port);
        std::process::exit(1);
    });
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| {
            eprintln!("HTTP sunucu hatası: {e}");
            std::process::exit(1);
        });
}

// ================================================================
// HANDLERLAR
// ================================================================

async fn get_report(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let snap = state.snapshot.lock().unwrap_or_else(|p| p.into_inner());

    if let Some(report) = &snap.report {
        Json(serde_json::json!({
            "status": "success",
            "last_updated": snap.last_updated_ms,
            "total_cycles": snap.total_cycles,
            "report": report,
        }))
    } else if let Some(err) = &snap.last_error {
        Json(serde_json::json!({
            "status": "error",
            "message": err,
        }))
    } else {
        Json(serde_json::json!({
            "status": "warming",
            "message": "İlk analiz henüz tamamlanmadı — birkaç saniye bekleyin.",
        }))
    }
}

async fn get_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let snap = state.snapshot.lock().unwrap_or_else(|p| p.into_inner());
    let (healthy, report): (bool, Option<&TrbReport>) = match (&snap.report, snap.last_updated_ms) {
        (Some(r), Some(ts)) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            (r.solver_state.is_stable && now.saturating_sub(ts) < 60_000, Some(r))
        }
        _ => (false, None),
    };

    Json(serde_json::json!({
        "healthy": healthy,
        "last_updated": snap.last_updated_ms,
        "total_cycles": snap.total_cycles,
        "last_error": snap.last_error,
        "grid": report.map(|r| (r.audit.grid_nx, r.audit.grid_ny))
            .map(|(nx, ny)| serde_json::json!({"nx": nx, "ny": ny})),
    }))
}