// ============================================================================
// detect-trb — FAZ UZAYI GRİDİ (PhaseSpace)
// ============================================================================
// 3D grid: (Nx=fiyat, Ny=derinlik, Nz=zaman) — ndarray ile.
// Fiyat ekseni logaritmiktir: P_log = ln(P/P_ref).
// Aktif çözüm 2D dilim üzerinde: mevcut zaman adımı (Nx×Ny).
//
// SIMD: divergence() → wide::f64x4 (AVX2, stable Rust)
// Paralel: rayon ile satır bazlı işlem
// ============================================================================

use ndarray::{Array2, Array3};
use wide::f64x4;
use tracing::error;

use crate::types::{FluidError, FluidResult, InflowData};

/// Grid boyutu sabitleri
pub const NX: usize = 64; // Fiyat ekseni (logaritmik dilimler)
pub const NY: usize = 16; // Derinlik ekseni (normalize 0–1)

// ================================================================
// FAZ UZAYI
// ================================================================

/// Navier-Stokes çözücüsünün 2D + zaman-tarih grid yapısı.
///
/// density  : Yoğunluk alanı ρ(x,y)  — (NX, NY)
/// vel_x    : x-yönü hız u(x,y)      — (NX, NY)
/// vel_y    : y-yönü hız v(x,y)      — (NX, NY)
/// pressure : Basınç alanı p(x,y)    — (NX, NY)
/// history  : Son `nz` adımın yoğunluk geçmişi — (NX, NY, NZ)
/// viscous  : ν — anlık kinematik viskozite
/// dx, dy   : Grid aralıkları
pub struct PhaseSpace {
    pub density:  Array2<f64>,
    pub vel_x:    Array2<f64>,
    pub vel_y:    Array2<f64>,
    pub pressure: Array2<f64>,
    pub history:  Array3<f64>,
    pub viscous:  f64,
    pub dx:       f64,
    pub dy:       f64,
    pub nz:       usize,
    /// Log-fiyat ekseni alt sınırı
    pub log_p_min: f64,
    /// Log-fiyat ekseni üst sınırı
    pub log_p_max: f64,
}

impl PhaseSpace {
    /// InflowData dizisinden PhaseSpace grid'i başlatır.
    ///
    /// Fiyat ekseni: ln(P_min) .. ln(P_max) → NX eşit dilim.
    /// Derinlik ekseni: hacim yoğunluğuna göre normalize.
    /// Zaman ekseni: her InflowData bir adım.
    pub fn from_inflows(inflows: &[InflowData]) -> FluidResult<Self> {
        if inflows.is_empty() {
            return Err(FluidError::DataStall);
        }

        let prices: Vec<f64> = inflows.iter().filter(|i| i.price > 0.0).map(|i| i.price).collect();
        if prices.is_empty() {
            return Err(FluidError::DataStall);
        }

        let p_min = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let p_max = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if p_min <= 0.0 || p_max <= p_min {
            return Err(FluidError::InvalidGridDimension);
        }

        let log_p_min = p_min.ln();
        let log_p_max = p_max.ln();
        let dx = (log_p_max - log_p_min) / (NX as f64);
        let dy = 1.0 / (NY as f64);

        let nz = inflows.len();

        let mut density  = Array2::<f64>::zeros((NX, NY));
        let mut vel_x    = Array2::<f64>::zeros((NX, NY));
        let mut vel_y    = Array2::<f64>::zeros((NX, NY));
        let mut pressure = Array2::<f64>::zeros((NX, NY));
        let mut history  = Array3::<f64>::zeros((NX, NY, nz));

        // Toplam hacim (normalize için)
        let total_vol: f64 = inflows.iter().map(|i| i.volume).sum::<f64>().max(1.0);

        // Her inflow adımını grid'e yansıt
        for (t, inflow) in inflows.iter().enumerate() {
            if inflow.price <= 0.0 {
                continue;
            }

            // Fiyatın log-uzaydaki bin indeksi
            let log_p = inflow.price.ln();
            let ix = ((log_p - log_p_min) / dx).floor() as usize;
            let ix = ix.min(NX - 1);

            // Derinlik: hacmin toplama oranı (0–NY)
            let depth_frac = (inflow.volume / total_vol * inflows.len() as f64).min(1.0);
            let iy = (depth_frac * NY as f64).floor() as usize;
            let iy = iy.min(NY - 1);

            // Yoğunluk: hacim ağırlıklı
            density[[ix, iy]] += inflow.volume / total_vol;

            // Hız: buy_sell_ratio → x-yönü akış
            // 0.5'in üstü alış baskısı → pozitif akış (yukarı hareket)
            vel_x[[ix, iy]] += (inflow.buy_sell_ratio - 0.5) * 2.0;

            // y-yönü hız: tasfiye baskısı → aşağı çeken kuvvet
            vel_y[[ix, iy]] -= inflow.liquidation_volume / total_vol.max(1.0);

            // Basınç: funding rate + OI delta (Coriolis)
            pressure[[ix, iy]] += inflow.funding_rate * 1000.0 + inflow.oi_delta * 0.001;

            // Tarih kaydı
            history[[ix, iy, t]] = inflow.volume / total_vol;
        }

        // NaN/Inf kontrolü
        if density.iter().any(|v| v.is_nan() || v.is_infinite()) {
            error!("Grid başlatmada NaN/Inf tespit edildi");
            return Err(FluidError::DivergenceExplosion);
        }

        Ok(PhaseSpace {
            density,
            vel_x,
            vel_y,
            pressure,
            history,
            viscous: 0.1, // Başlangıç viskozitesi (kalibratör güncelleyecek)
            dx,
            dy,
            nz,
            log_p_min,
            log_p_max,
        })
    }

    // ================================================================
    // DİVERJANS HESAPLAMA — SIMD (wide::f64x4)
    // ================================================================

    /// ∇·u = ∂u/∂x + ∂v/∂y — Merkezi fark (2. derece)
    ///
    /// x-yönü türev: SIMD ile 4'lü bloklarda işlenir (f64x4).
    /// y-yönü türev: satır bazlı, skaler.
    pub fn divergence(&self) -> FluidResult<Array2<f64>> {
        let mut div = Array2::<f64>::zeros((NX, NY));

        // ── x-yönü türev: ∂u/∂x — SIMD bloklarla ────────────────────────
        // Her y dilimi için x yönünde 4'lü SIMD bloklarla işle
        div.axis_iter_mut(ndarray::Axis(1))
            .enumerate()
            .for_each(|(iy, mut div_col)| {
                let vel_col: Vec<f64> = (0..NX).map(|ix| self.vel_x[[ix, iy]]).collect();

                // Merkezi fark: (u[i+1] - u[i-1]) / (2 * dx)
                // SIMD ile 4'lü bloklar (iç noktalar)
                let mut ix = 1usize;
                while ix + 4 < NX {
                    // u[i-1..i+3] ve u[i+1..i+5] vektörleri
                    let v_left  = f64x4::new([vel_col[ix-1], vel_col[ix],   vel_col[ix+1], vel_col[ix+2]]);
                    let v_right = f64x4::new([vel_col[ix+1], vel_col[ix+2], vel_col[ix+3], vel_col[ix+4]]);
                    let two_dx  = f64x4::splat(2.0 * self.dx);
                    let result  = (v_right - v_left) / two_dx;
                    let arr = result.to_array();
                    for k in 0..4 {
                        div_col[ix + k] += arr[k];
                    }
                    ix += 4;
                }
                // Kalan noktalar (skaler)
                while ix < NX - 1 {
                    div_col[ix] += (vel_col[ix + 1] - vel_col[ix - 1]) / (2.0 * self.dx);
                    ix += 1;
                }
                // Sınır noktaları (tek taraflı fark)
                div_col[0]      += (vel_col[1] - vel_col[0]) / self.dx;
                div_col[NX - 1] += (vel_col[NX-1] - vel_col[NX-2]) / self.dx;
            });

        // ── y-yönü türev: ∂v/∂y — skaler (NY küçük: 16) ─────────────────
        for ix in 0..NX {
            for iy in 1..NY - 1 {
                div[[ix, iy]] +=
                    (self.vel_y[[ix, iy + 1]] - self.vel_y[[ix, iy - 1]]) / (2.0 * self.dy);
            }
            // Sınır
            div[[ix, 0]]      += (self.vel_y[[ix, 1]] - self.vel_y[[ix, 0]]) / self.dy;
            div[[ix, NY - 1]] += (self.vel_y[[ix, NY-1]] - self.vel_y[[ix, NY-2]]) / self.dy;
        }

        // NaN kontrolü
        if div.iter().any(|v| v.is_nan() || v.is_infinite()) {
            return Err(FluidError::DivergenceExplosion);
        }

        Ok(div)
    }

    /// Divergence normu ‖∇·u‖₂ — kararlılık göstergesi
    pub fn divergence_norm(&self) -> FluidResult<f64> {
        let div = self.divergence()?;
        let norm = div.iter().map(|v| v * v).sum::<f64>().sqrt();
        Ok(norm)
    }

    /// Grid'i sıfırla (DivergenceExplosion recovery)
    pub fn reset(&mut self) {
        self.density.fill(0.0);
        self.vel_x.fill(0.0);
        self.vel_y.fill(0.0);
        self.pressure.fill(0.0);
        self.viscous = 0.1;
    }
}
