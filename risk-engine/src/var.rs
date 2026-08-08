//! Value-at-Risk: parametrik (varyans-kovaryans) ve tarihsel yöntemler.
//!
//! Para değildir — `f64` model çıktısıdır. `None` dönerse fail-closed davranın.

use rust_decimal::Decimal;

/// Standart normal z-değeri (tek kuyruk).
fn z_score(confidence: f64) -> f64 {
    // (confidence * 1000).round() → 950/970/980/990/995 olarak eşlenir.
    match (confidence * 1000.0).round() as i32 {
        950 => 1.6449,
        970 => 1.8808,
        980 => 2.0537,
        990 => 2.3263,
        995 => 2.5758,
        _ => 2.3263, // varsayılan %99
    }
}

/// Parametrik portföy VaR (periyot başına):
/// `sigma_p^2 = w' * Sigma * w`, `VaR = z * sigma_p`.
///
/// - `corr`: korelasyon matrisi (N×N, f64)
/// - `vols`: sembol başına periyot volatilitesi (f64)
/// - `weights`: sembol başına portföy ağırlığı (değer payı)
pub fn parametric_var_99_1d(corr: &[Vec<f64>], vols: &[f64], weights: &[f64]) -> Option<f64> {
    if corr.is_empty() || corr.len() != vols.len() || vols.len() != weights.len() {
        return None;
    }
    let n = corr.len();
    let mut var = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            var += weights[i] * weights[j] * vols[i] * vols[j] * corr[i][j];
        }
    }
    if var <= 0.0 {
        return None;
    }
    Some(z_score(0.99) * var.sqrt())
}

/// Tek sembol parametrik VaR: `z * vol * |weight|`.
pub fn single_asset_var_99_1d(vol: f64, weight: f64) -> f64 {
    z_score(0.99) * vol * weight.abs()
}

/// Tarihsel VaR: portföy getiri serisinin `confidence` yüzdelik dilimi.
pub fn historical_var(portfolio_returns: &[f64], confidence: f64) -> Option<f64> {
    if portfolio_returns.is_empty() {
        return None;
    }
    let mut sorted = portfolio_returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((1.0 - confidence) * sorted.len() as f64) as usize;
    let idx = idx.min(sorted.len() - 1);
    Some(-sorted[idx])
}

/// Portföy ağırlıklarını HHI hedefiyle sınırlar (worker'da öneri üretimi için).
/// Brüt exposure paylarının karesi toplamı `max_hhi`'yı geçmeyecek şekilde
/// tek bir sembole düşen ağırlık üst sınırı döndürür.
pub fn max_weight_for_hhi(max_hhi: f64) -> f64 {
    // HHI = w^2 + (1-w)^2/n... üst sınır yaklaşımı: tek sembol payı <= sqrt(max_hhi).
    if max_hhi <= 0.0 {
        1.0
    } else {
        max_hhi.sqrt()
    }
}

/// Decimal tabanlı öneri: `loss_budget / var` ile sembol başına güvenli notional.
pub fn safe_notional(loss_budget_usdt: Decimal, var_99_1d_pct: f64) -> Decimal {
    if var_99_1d_pct <= 0.0 {
        return loss_budget_usdt;
    }
    let budget = loss_budget_usdt.to_f64().unwrap_or(0.0);
    let n = budget / var_99_1d_pct;
    Decimal::from_f64_retain(n).unwrap_or_default()
}

// to_f64 yardımcı erişimi.
use rust_decimal::prelude::ToPrimitive;
