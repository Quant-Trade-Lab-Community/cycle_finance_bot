// ============================================================================
// detect-trb — ANALİZ BORU HATTI
// ============================================================================
// Tüm katmanları birleştirir: ingest → grid → solver → kavitasyon →
// kalibrasyon → TWAP → naratif → TrbReport.
// ============================================================================

use crate::calibration;
use crate::cavitation;
use crate::grid::PhaseSpace;
use crate::ingest;
use crate::narrative;
use crate::order_flow;
use crate::solver::NSSolver;
use crate::types::{
    CalibrationResult, FluidError, FluidResult, InflowData, TrbReport,
};

/// Veri kaynağı etiketi (audit için)
pub const DATA_SOURCE: &str = "sqlite+ringbuffer";

/// Tam boru hattı: sadece inflow dizisiyle (test/replay için).
pub fn analyze_inflows(inflows: &[InflowData]) -> FluidResult<TrbReport> {
    if inflows.is_empty() {
        return Err(FluidError::DataStall);
    }

    // 1 — Grid kur
    let mut solver = NSSolver::new(PhaseSpace::from_inflows(inflows)?);

    // 2 — NS adımları (tüm inflow sırası)
    for inf in inflows {
        solver.step(inf)?;
    }
    let state = solver.state()?;

    // 3 — Kavitasyon (tasfiye şok dalgası)
    let total_liq: f64 = inflows.iter().map(|i| i.liquidation_volume).sum();
    let ob_depth: f64 = inflows.iter().map(|i| i.volume).sum::<f64>()
        / inflows.len().max(1) as f64;
    let last_price = inflows
        .iter()
        .rev()
        .find(|i| i.price > 0.0)
        .map(|i| i.price)
        .unwrap_or(0.0);
    let burst = cavitation::analyze_cavitation(total_liq, state.mean_pressure, last_price, ob_depth)?;

    // 4 — Kalibrasyon (başarısızsa varsayılan)
    let calibration = match calibration::calibrate(inflows) {
        Ok(c) => c,
        Err(_) => CalibrationResult {
            viscosity: solver.grid.viscous,
            smagorinsky_cs: 0.05,
            cost: 0.0,
            iterations: 0,
        },
    };

    // 5 — TWAP eğrisi (Pontryagin)
    let grad = solver.mean_pressure_gradient();
    let dir = order_flow::net_direction(grad, burst.as_ref());
    let twap_curve = order_flow::build_twap_curve(grad, dir, None, None)?;

    // 6 — Narativ + audit
    let narrative_output = narrative::narrate(&state, &calibration, burst.as_ref(), "report");
    let audit = narrative::audit_meta("report", DATA_SOURCE);

    Ok(TrbReport {
        symbol: "report".to_string(),
        interval: "replay".to_string(),
        inflow_steps: inflows.len(),
        solver_state: state,
        burst_signal: burst,
        calibration,
        twap_curve,
        narrative: narrative_output,
        audit,
    })
}

/// Canlı boru hattı: SQLite + ring buffer canlı veri.
///
/// `extra_live`: rtrb kanalından gelen en son canlı İnflowData (opsiyonel).
pub fn analyze(
    db_path: &str,
    symbol: &str,
    interval_ms: u64,
    limit: usize,
    extra_live: &[InflowData],
) -> FluidResult<TrbReport> {
    let mut inflows = ingest::load_from_sqlite(db_path, symbol, interval_ms, limit)?;
    let live = ingest::drain_ring_buffer(symbol, 8192);
    inflows = ingest::merge_sources(inflows, live);
    if !extra_live.is_empty() {
        inflows = ingest::merge_sources(inflows, extra_live.to_vec());
    }
    if inflows.is_empty() {
        return Err(FluidError::DataStall);
    }

    let mut report = analyze_inflows(&inflows)?;
    report.symbol = symbol.to_string();
    report.interval = format!("{interval_ms}ms");
    report.audit.data_source = DATA_SOURCE.to_string();
    Ok(report)
}
