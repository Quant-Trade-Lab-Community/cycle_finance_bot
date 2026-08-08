// ============================================================================
// detect-trb — KALİBRASYON (Nelder-Mead)
// ============================================================================
// Akışkan parametreleri (ν viskozite, Cs Smagorinsky) Nelder-Mead simplex
// optimizasyonu ile aranır:
//   maliyet = |KE(ν_eff) − KE_hedef| / (KE_hedef + ε) + 1e-9·‖∇·u‖
// Simülasyon: ilk `MAX_SIM_STEPS` inflow üzerinde NSSolver çalıştırılır.
// unwrap() yok — tüm yollar FluidResult.
// ============================================================================

use std::f64::INFINITY;

use crate::grid::PhaseSpace;
use crate::solver::NSSolver;
use crate::types::{CalibrationResult, FluidError, FluidResult, InflowData};

/// Maliyet hesabında çalıştırılan simülasyon adım sayısı
const MAX_SIM_STEPS: usize = 8;
/// Nelder-Mead maksimum iterasyon
const MAX_NM_ITER: usize = 60;
/// İlk simpleks genişliği (x ekseni birim oranı)
const NM_LAMBDA: f64 = 0.1;
/// Simplex yayılım toleransı — altı inen iterasyon durur
const NM_TOL: f64 = 1e-10;

/// ν (kinematik viskozite) sınırları
pub const VISCOSITY_MIN: f64 = 1e-4;
pub const VISCOSITY_MAX: f64 = 1.0;
/// Cs (Smagorinsky) sınırları
pub const CS_MIN: f64 = 0.01;
pub const CS_MAX: f64 = 0.3;

/// Simülasyon özeti
struct SimMetric {
    /// Kinetik enerji ortalama kökü √(Σ|u|²/N)
    ke: f64,
    /// Diverjans normu
    div: f64,
}

/// Grid üzerinde `viscosity` ile `MAX_SIM_STEPS` adım simüle et.
fn simulate(inflows: &[InflowData], viscosity: f64) -> FluidResult<SimMetric> {
    let n = inflows.len().min(MAX_SIM_STEPS);
    if n == 0 {
        return Err(FluidError::DataStall);
    }

    let grid = PhaseSpace::from_inflows(inflows)?;
    let n_eff = if viscosity.is_finite() && viscosity > 0.0 {
        viscosity
    } else {
        grid.viscous
    };
    let mut solver = NSSolver::new(grid);
    solver.grid.viscous = n_eff;

    for inf in &inflows[..n] {
        solver.step(inf)?;
    }

    // Kinetik enerji yoğunluğu
    let mut ke_sum = 0.0;
    let mut count = 0usize;
    for v in solver.grid.vel_x.iter() {
        ke_sum += v * v;
        count += 1;
    }
    for v in solver.grid.vel_y.iter() {
        ke_sum += v * v;
    }
    let ke = (ke_sum / (count.max(1) as f64)).sqrt();
    if ke.is_nan() || ke.is_infinite() {
        return Err(FluidError::DivergenceExplosion);
    }

    let div = solver.state()?.divergence_norm;
    Ok(SimMetric { ke, div })
}

/// Hedef kinetik enerji — inflow dengesizlikleri (buy/sell + tasfiye)
fn target_energy(inflows: &[InflowData]) -> f64 {
    let n = inflows.len().max(1) as f64;
    let total: f64 = inflows
        .iter()
        .map(|i| {
            let bsr = (i.buy_sell_ratio - 0.5).powi(2) * 4.0;
            let liq = (i.liquidation_volume / (i.volume.abs() + 1.0)).min(4.0);
            bsr + liq
        })
        .sum();
    (total / n).max(1e-6)
}

/// Inflow verisiyle ν ve Cs kalibre et (Nelder-Mead).
///
/// Hata durumunda `FluidError` döner — çağıran `grid.viscous` varsayılanına
/// düşer (graceful degradation).
pub fn calibrate(inflows: &[InflowData]) -> FluidResult<CalibrationResult> {
    if inflows.is_empty() {
        return Err(FluidError::DataStall);
    }
    let target = target_energy(inflows);

    // ν_eff = ν·(1 + 0.5·Cs) — Cs Smagorinsky türbülans ek difüzyonu
    let mut cost = |x: [f64; 2]| -> f64 {
        let nu_eff = x[0] * (1.0 + 0.5 * x[1]);
        match simulate(inflows, nu_eff) {
            Ok(m) => {
                let ke_err = (m.ke - target).abs() / target;
                ke_err + 1e-9 * m.div.min(1e3)
            }
            Err(_) => INFINITY,
        }
    };

    let (best, best_cost, iters) = nelder_mead(
        &mut cost,
        [0.1, 0.05],
        [VISCOSITY_MIN, CS_MIN],
        [VISCOSITY_MAX, CS_MAX],
    );

    Ok(CalibrationResult {
        viscosity: best[0],
        smagorinsky_cs: best[1],
        cost: if best_cost.is_finite() { best_cost } else { 0.0 },
        iterations: iters as u32,
    })
}

/// Nelder-Mead simplex optimizasyonu (2 boyut).
///
/// `cost` her nokta clamp edilmiş parametrelerle çağrılır;
/// NaN/Inf maliyet sonsuz sayılır (soft güvenlik).
fn nelder_mead<F>(
    cost: &mut F,
    x0: [f64; 2],
    lo: [f64; 2],
    hi: [f64; 2],
) -> ([f64; 2], f64, usize)
where
    F: FnMut([f64; 2]) -> f64,
{
    let clamp = |mut v: [f64; 2]| -> [f64; 2] {
        for i in 0..2 {
            if !v[i].is_finite() {
                v[i] = lo[i];
            }
            v[i] = v[i].clamp(lo[i], hi[i]);
        }
        v
    };

    let mut pts: Vec<([f64; 2], f64)> = Vec::with_capacity(3);
    pts.push((clamp(x0), 0.0));
    for i in 0..2 {
        let mut p = x0;
        p[i] *= 1.0 + NM_LAMBDA;
        pts.push((clamp(p), 0.0));
    }

    let mut iters = 0usize;
    loop {
        for (p, v) in pts.iter_mut() {
            let c = cost(*p);
            *v = if c.is_finite() { c } else { INFINITY };
        }
        pts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        iters += 1;
        let spread = (pts[2].1 - pts[0].1).abs();
        if iters >= MAX_NM_ITER || spread <= NM_TOL || !pts[0].1.is_finite() {
            if !pts[0].1.is_finite() || pts[0].1.is_nan() {
                // Degrade: x0 + varsayılan iterasyon
                return ([x0[0], x0[1]], 0.0, iters);
            }
            return (pts[0].0, pts[0].1, iters);
        }

        // Centroid: en iyi iki nokta (en kötü hariç)
        let centroid = [
            (pts[0].0[0] + pts[1].0[0]) / 2.0,
            (pts[0].0[1] + pts[1].0[1]) / 2.0,
        ];
        let worst = pts[2];

        // Reflection
        let mut xr = [
            centroid[0] + (centroid[0] - worst.0[0]),
            centroid[1] + (centroid[1] - worst.0[1]),
        ];
        xr = clamp(xr);
        let fr = {
            let c = cost(xr);
            if c.is_finite() { c } else { INFINITY }
        };

        if fr < pts[1].1 && fr >= pts[0].1 {
            pts[2] = (xr, fr);
            continue;
        }

        // Expansion
        if fr < pts[0].1 {
            let mut xe = [
                centroid[0] + 2.0 * (xr[0] - centroid[0]),
                centroid[1] + 2.0 * (xr[1] - centroid[1]),
            ];
            xe = clamp(xe);
            let fe = {
                let c = cost(xe);
                if c.is_finite() { c } else { INFINITY }
            };
            pts[2] = if fe < fr { (xe, fe) } else { (xr, fr) };
            continue;
        }

        // (Dış) contraction
        let mut xc = [
            centroid[0] + 0.5 * (worst.0[0] - centroid[0]),
            centroid[1] + 0.5 * (worst.0[1] - centroid[1]),
        ];
        xc = clamp(xc);
        let fc = {
            let c = cost(xc);
            if c.is_finite() { c } else { INFINITY }
        };
        if fc < worst.1 {
            pts[2] = (xc, fc);
            continue;
        }

        // Shrink: en iyi noktaya doğru çek
        for k in 1..3 {
            let p = pts[0].0;
            let ns = [
                p[0] + 0.5 * (pts[k].0[0] - p[0]),
                p[1] + 0.5 * (pts[k].0[1] - p[1]),
            ];
            pts[k] = (clamp(ns), 0.0);
        }
    }
}
