// ============================================================================
// detect-trb — NAVIER-STOKES ÇÖZÜCÜsü
// ============================================================================
// Her adım:
//   1. Adveksiyon: (u·∇)u — Upwind differencing
//   2. Difüzyon:   ν∇²u   — Thomas Algorithm (implicit tridiagonal)
//   3. Dış kuvvet: OI delta (basınç) + funding (Coriolis)
//   4. Basınç Poisson: Jacobi iterasyonu
//   5. Hız düzeltmesi: u ← u - ∇p
//
// rayon ile satır/sütun bazlı paralelizasyon.
// ============================================================================

use rayon::prelude::*;
use tracing::{error, warn};

use crate::grid::{NX, NY, PhaseSpace};
use crate::types::{FluidError, FluidResult, InflowData, SolverState};

/// Zaman adımı sabiti (μs cinsinden, normalize edilmiş)
const DT: f64 = 1e-3;
/// Poisson Jacobi iterasyon sayısı
const POISSON_ITER: usize = 20;
/// Iraksama eşiği — norm bu değeri geçerse reset
const DIVERGENCE_THRESHOLD: f64 = 1e6;

// ================================================================
// NS ÇÖZÜCÜsü
// ================================================================

pub struct NSSolver {
    pub grid: PhaseSpace,
    /// Tamamlanan adım sayısı
    pub steps: usize,
}

impl NSSolver {
    pub fn new(grid: PhaseSpace) -> Self {
        Self { grid, steps: 0 }
    }

    /// Tek bir zaman adımı — tüm NS pipeline'ı
    pub fn step(&mut self, inflow: &InflowData) -> FluidResult<()> {
        // 1. Adveksiyon
        self.advect()?;

        // 2. Difüzyon (Thomas Algorithm — implicit)
        self.diffuse()?;

        // 3. Dış kuvvetler (OI + Coriolis/Funding)
        self.force_apply(inflow.oi_delta, inflow.funding_rate);

        // 4. Basınç Poisson
        self.pressure_poisson()?;

        // 5. Hız düzeltmesi
        self.velocity_correction()?;

        self.steps += 1;
        Ok(())
    }

    // ================================================================
    // 1. ADVEKSİYON: (u·∇)u — Upwind Differencing
    // ================================================================

    fn advect(&mut self) -> FluidResult<()> {
        let dx = self.grid.dx;
        let dy = self.grid.dy;

        // Paralel satır işlemi — iç noktalar
        let new_vx: Result<Vec<Vec<f64>>, FluidError> = (1..NX - 1)
            .into_par_iter()
            .map(|ix| {
                let mut row = vec![0.0f64; NY];
                for iy in 1..NY - 1 {
                    let u = self.grid.vel_x[[ix, iy]];
                    let v = self.grid.vel_y[[ix, iy]];

                    // Upwind: akış yönüne göre taraflı türev
                    let du_dx = if u >= 0.0 {
                        (self.grid.vel_x[[ix, iy]] - self.grid.vel_x[[ix - 1, iy]]) / dx
                    } else {
                        (self.grid.vel_x[[ix + 1, iy]] - self.grid.vel_x[[ix, iy]]) / dx
                    };
                    let du_dy = if v >= 0.0 {
                        (self.grid.vel_x[[ix, iy]] - self.grid.vel_x[[ix, iy - 1]]) / dy
                    } else {
                        (self.grid.vel_x[[ix, iy + 1]] - self.grid.vel_x[[ix, iy]]) / dy
                    };

                    let new_u = u - DT * (u * du_dx + v * du_dy);
                    if new_u.is_nan() || new_u.is_infinite() {
                        return Err(FluidError::DivergenceExplosion);
                    }
                    row[iy] = new_u;
                }
                Ok(row)
            })
            .collect();

        let new_vx = new_vx?;
        for (i, row) in new_vx.into_iter().enumerate() {
            let ix = i + 1;
            for iy in 1..NY - 1 {
                self.grid.vel_x[[ix, iy]] = row[iy];
            }
        }

        Ok(())
    }

    // ================================================================
    // 2. DİFÜZYON: ν∇²u — Thomas Algorithm (Tridiagonal Implicit)
    // ================================================================
    // Her sütun için 1D tridiagonal sistem çözeriz (x-yönü).
    // Thomas Algorithm: O(N) — doğrudan, iterasyon yok.

    fn diffuse(&mut self) -> FluidResult<()> {
        let nu = self.grid.viscous;
        let dx = self.grid.dx;
        let r = nu * DT / (dx * dx);

        // Her y dilimi için x-yönünde Thomas solve
        for iy in 0..NY {
            let mut vel_col: Vec<f64> = (0..NX).map(|ix| self.grid.vel_x[[ix, iy]]).collect();
            thomas_solve(&mut vel_col, r)?;
            for ix in 0..NX {
                self.grid.vel_x[[ix, iy]] = vel_col[ix];
            }
        }

        // y-yönü difüzyon
        let r_y = nu * DT / (self.grid.dy * self.grid.dy);
        for ix in 0..NX {
            let mut vel_row: Vec<f64> = (0..NY).map(|iy| self.grid.vel_y[[ix, iy]]).collect();
            thomas_solve(&mut vel_row, r_y)?;
            for iy in 0..NY {
                self.grid.vel_y[[ix, iy]] = vel_row[iy];
            }
        }

        Ok(())
    }

    // ================================================================
    // 3. DIŞ KUVVETLER: OI Delta + Coriolis (Funding)
    // ================================================================

    fn force_apply(&mut self, oi_delta: f64, funding_rate: f64) {
        // OI delta → x-yönü itme (açık pozisyon artışı yukarı ivme)
        let oi_force = oi_delta * 1e-6 * DT;

        // Funding rate → Coriolis benzeri döndürücü kuvvet
        // Pozitif funding → long pahalı → satış baskısı (aşağı)
        let coriolis = -funding_rate * 100.0 * DT;

        // Grid genelinde uygula (rayon parallel)
        self.grid.vel_x.par_mapv_inplace(|v| v + oi_force);
        self.grid.vel_y.par_mapv_inplace(|v| v + coriolis);

        // Density güncelle: yoğunluk OI ile büyür
        self.grid.density.par_mapv_inplace(|d| (d + oi_delta.abs() * 1e-8).min(10.0));
    }

    // ================================================================
    // 4. BASINÇ POISSON: ∇²p = (1/Δt)∇·u — Jacobi İterasyonu
    // ================================================================

    fn pressure_poisson(&mut self) -> FluidResult<()> {
        let div = self.grid.divergence()?;
        let dx2 = self.grid.dx * self.grid.dx;
        let dy2 = self.grid.dy * self.grid.dy;

        let mut p = self.grid.pressure.clone();

        for _ in 0..POISSON_ITER {
            let p_old = p.clone();
            // İç noktalar: Jacobi adımı
            for ix in 1..NX - 1 {
                for iy in 1..NY - 1 {
                    let rhs = -div[[ix, iy]] / DT;
                    let p_new = (
                        (p_old[[ix + 1, iy]] + p_old[[ix - 1, iy]]) / dx2
                      + (p_old[[ix, iy + 1]] + p_old[[ix, iy - 1]]) / dy2
                      - rhs
                    ) / (2.0 / dx2 + 2.0 / dy2);

                    if p_new.is_nan() || p_new.is_infinite() {
                        error!(ix, iy, "Poisson NaN tespit edildi");
                        return Err(FluidError::DivergenceExplosion);
                    }
                    p[[ix, iy]] = p_new;
                }
            }
            // Neumann sınır koşulları: ∂p/∂n = 0
            for ix in 0..NX {
                p[[ix, 0]]      = p[[ix, 1]];
                p[[ix, NY - 1]] = p[[ix, NY - 2]];
            }
            for iy in 0..NY {
                p[[0, iy]]      = p[[1, iy]];
                p[[NX - 1, iy]] = p[[NX - 2, iy]];
            }
        }

        self.grid.pressure = p;
        Ok(())
    }

    // ================================================================
    // 5. HIZ DÜZELTMESİ: u ← u - Δt·∇p
    // ================================================================

    fn velocity_correction(&mut self) -> FluidResult<()> {
        let dx = self.grid.dx;
        let dy = self.grid.dy;

        for ix in 1..NX - 1 {
            for iy in 1..NY - 1 {
                let dp_dx = (self.grid.pressure[[ix + 1, iy]] - self.grid.pressure[[ix - 1, iy]])
                    / (2.0 * dx);
                let dp_dy = (self.grid.pressure[[ix, iy + 1]] - self.grid.pressure[[ix, iy - 1]])
                    / (2.0 * dy);

                self.grid.vel_x[[ix, iy]] -= DT * dp_dx;
                self.grid.vel_y[[ix, iy]] -= DT * dp_dy;
            }
        }

        // NaN kontrolü
        if self.grid.vel_x.iter().any(|v| v.is_nan()) {
            return Err(FluidError::DivergenceExplosion);
        }
        Ok(())
    }

    // ================================================================
    // SOLVER DURUMU
    // ================================================================

    pub fn state(&self) -> FluidResult<SolverState> {
        let mean_density = self.grid.density.mean().unwrap_or(0.0);
        let max_vx = self.grid.vel_x.iter().cloned().fold(0.0f64, f64::max);
        let max_vy = self.grid.vel_y.iter().cloned().fold(0.0f64, f64::max);
        let max_velocity = (max_vx * max_vx + max_vy * max_vy).sqrt();
        let mean_pressure = self.grid.pressure.mean().unwrap_or(0.0);
        let div_norm = self.grid.divergence_norm().unwrap_or(f64::INFINITY);

        let is_stable = div_norm < DIVERGENCE_THRESHOLD
            && !mean_density.is_nan()
            && !max_velocity.is_nan();

        Ok(SolverState {
            mean_density,
            max_velocity,
            mean_pressure,
            viscous: self.grid.viscous,
            divergence_norm: div_norm,
            is_stable,
            steps_completed: self.steps,
        })
    }

    /// Basınç gradyanı ∂p/∂x ortalaması — execution için
    pub fn mean_pressure_gradient(&self) -> f64 {
        let dx = self.grid.dx;
        let mut total = 0.0;
        let mut count = 0;
        for ix in 1..NX - 1 {
            for iy in 0..NY {
                total += (self.grid.pressure[[ix + 1, iy]] - self.grid.pressure[[ix - 1, iy]])
                    / (2.0 * dx);
                count += 1;
            }
        }
        if count > 0 { total / count as f64 } else { 0.0 }
    }
}

// ================================================================
// THOMAS ALGORITHM — Tridiagonal System Solver
// ================================================================
// Sistem: a·u[i-1] - (2a+1)·u[i] + a·u[i+1] = -b[i]
// a = r = ν·Δt/Δx²
// Kaynak: Numerical Recipes, bölüm 2.4

fn thomas_solve(b: &mut Vec<f64>, r: f64) -> FluidResult<()> {
    let n = b.len();
    if n < 2 {
        return Ok(());
    }

    let alpha = r;           // Alt köşegen katsayısı
    let beta  = -(1.0 + 2.0 * r); // Ana köşegen
    let gamma = r;           // Üst köşegen

    let mut c_prime = vec![0.0f64; n];
    let mut d_prime = vec![0.0f64; n];

    // İleri tarama
    c_prime[0] = gamma / beta;
    d_prime[0] = -b[0] / beta;

    for i in 1..n {
        let m = beta - alpha * c_prime[i - 1];
        if m.abs() < 1e-14 {
            warn!("Thomas: singular matrix at i={}", i);
            return Err(FluidError::DivergenceExplosion);
        }
        c_prime[i] = gamma / m;
        d_prime[i] = (-b[i] - alpha * d_prime[i - 1]) / m;
    }

    // Geri yerine koyma
    b[n - 1] = d_prime[n - 1];
    for i in (0..n - 1).rev() {
        b[i] = d_prime[i] - c_prime[i] * b[i + 1];
        if b[i].is_nan() || b[i].is_infinite() {
            return Err(FluidError::DivergenceExplosion);
        }
    }

    Ok(())
}
