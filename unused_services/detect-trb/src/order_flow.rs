// ============================================================================
// detect-trb — EMİR AKIŞI / YÜRÜTME (Pontryagin Minimum Prensibi)
// ============================================================================
// Basınç gradyanı ∂p/∂x → zaman-emir eğrisi (TWAP).
//   - Erken dilimler daha agresif (sermaye maliyeti cezası modeli)
//   - Dilim toplamı 1.0'e normalize
//   - Yön: BurstSignal (kavitasyon) varsa onun yönü, yoksa gradyan işareti
// ============================================================================

use crate::types::{BurstSignal, FluidError, FluidResult, OrderSlice};

/// Varsayılan dilim sayısı
pub const DEFAULT_SLICES: usize = 8;
/// Risk kaçınma katsayısı (0–1): yüksek → erken dilimler daha büyük
pub const DEFAULT_RISK_AVERSION: f64 = 0.8;
/// Fiyat kayması katsayı (gradyan → price_offset ölçekleme)
pub const PRICE_IMPACT: f64 = 1e-4;

/// Basınç gradyanından TWAP emir eğrisi üretir (Pontryagin yaklaşımı).
///
/// `pressure_gradient`: ∂p/∂x ortalaması (solver'dan)
/// `direction`: +1.0 yukarı (long), −1.0 aşağı (short)
/// `slices`: dilim sayısı (None → varsayılan 8)
/// `risk_aversion`: 0–1 arası erken dilim ağırlığı
pub fn build_twap_curve(
    pressure_gradient: f64,
    direction: f64,
    slices: Option<usize>,
    risk_aversion: Option<f64>,
) -> FluidResult<Vec<OrderSlice>> {
    if !pressure_gradient.is_finite() {
        return Err(FluidError::DivergenceExplosion);
    }

    let n = slices.unwrap_or(DEFAULT_SLICES).max(1);
    let r = risk_aversion.unwrap_or(DEFAULT_RISK_AVERSION).clamp(0.0, 1.0);

    // Ağırlıklar: w_i = r^i → geometrik azalma (erken dilimler ağırlıklı)
    let weights: Vec<f64> = (0..n).map(|i| r.powi(i as i32)).collect();
    let sum: f64 = weights.iter().sum();
    if sum <= 0.0 || !sum.is_finite() {
        return Err(FluidError::DivergenceExplosion);
    }

    let g = pressure_gradient.abs().min(10.0);
    let dir = if direction >= 0.0 { 1.0 } else { -1.0 };

    let mut curve: Vec<OrderSlice> = Vec::with_capacity(n);
    for (i, w) in weights.into_iter().enumerate() {
        let size = (w / sum).clamp(0.0, 1.0);
        // Fiyat ofseti: gradyan yönünde kademeli — erken piyasa etkisi küçük
        let offset = dir * g * ((i + 1) as f64 / n as f64) * PRICE_IMPACT;
        curve.push(OrderSlice {
            size,
            price_offset: if offset.is_finite() { offset } else { 0.0 },
            index: i,
        });
    }

    // Toplam 1.0 kontrolü (kayan nokta hassasiyeti)
    let total: f64 = curve.iter().map(|s| s.size).sum();
    if (total - 1.0).abs() > 1e-9 {
        if let Some(last) = curve.last_mut() {
            let fix = 1.0 - total;
            if (last.size + fix).is_finite() {
                last.size = (last.size + fix).clamp(0.0, 1.0);
            }
        }
    }

    Ok(curve)
}

/// Burst sinyalinden yön işareti: LONG → 1.0, SHORT → −1.0, yoksa 0.0
pub fn direction_from_burst(burst: Option<&BurstSignal>) -> f64 {
    match burst {
        Some(b) if b.direction == "LONG" => 1.0,
        Some(_) => -1.0,
        None => 0.0,
    }
}

/// Kavitasyon yönü varsa onu, yoksa gradyan işaretini kullan
pub fn net_direction(pressure_gradient: f64, burst: Option<&BurstSignal>) -> f64 {
    let dir = direction_from_burst(burst);
    if dir != 0.0 {
        dir
    } else if pressure_gradient > 1e-12 {
        1.0
    } else if pressure_gradient < -1e-12 {
        -1.0
    } else {
        0.0
    }
}
