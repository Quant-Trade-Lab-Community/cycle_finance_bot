//! Korelasyon matrisi, shrinkage, Tikhonov regularizasyonu ve koşul sayısı.
//!
//! Tüm hesaplar `f64` (istatistiksel model — para değil). BLAS bağımlılığı yok;
//! N≤64 matrisler için Jacobi özdeğer çözücü kullanılır. Bu, soğuk yolda
//! (60s risk-worker) çalışır, asla sıcak tick yolunda çağrılmaz.

/// Sembol getirilerinden (satır = sembol, sütun = zaman) Pearson korelasyon
/// matrisi hesaplar. Her sembolün varyansı sıfırsa o sembol korelasyon 1 ile
/// katılır (yalnızca kendisiyle), aksi halde 0.
#[allow(clippy::needless_range_loop)]
pub fn correlation_matrix(returns: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = returns.len();
    if n == 0 {
        return Vec::new();
    }
    let t = returns[0].len();
    let mut means = vec![0.0; n];
    for i in 0..n {
        let sum: f64 = returns[i].iter().sum();
        means[i] = if t > 0 { sum / t as f64 } else { 0.0 };
    }
    // Kovaryans matrisi.
    let mut cov = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let mut s = 0.0;
            for k in 0..t {
                s += (returns[i][k] - means[i]) * (returns[j][k] - means[j]);
            }
            cov[i][j] = if t > 1 { s / (t as f64 - 1.0) } else { 0.0 };
        }
    }
    // Korelasyona çevir.
    let mut corr = vec![vec![0.0; n]; n];
    for i in 0..n {
        let di = cov[i][i].sqrt();
        for j in 0..n {
            let dj = cov[j][j].sqrt();
            if di > 0.0 && dj > 0.0 {
                corr[i][j] = cov[i][j] / (di * dj);
            } else if i == j {
                corr[i][j] = 1.0;
            } else {
                corr[i][j] = 0.0;
            }
        }
    }
    corr
}

/// Korelasyon matrisini shrink eder (Ledoit–Wolf yaklaşımı):
/// `(1-s) * C + s * I`. `s` genelde 0.05..=0.30 — matrisi tekil olmaktan uzaklaştırır.
#[allow(clippy::needless_range_loop)]
pub fn shrink(corr: &[Vec<f64>], s: f64) -> Vec<Vec<f64>> {
    let n = corr.len();
    let s = s.clamp(0.0, 1.0);
    let mut out = corr.to_vec();
    for i in 0..n {
        for j in 0..n {
            out[i][j] = (1.0 - s) * corr[i][j] + if i == j { s } else { 0.0 };
        }
    }
    out
}

/// Tikhonov (ridge) regularizasyonu: `C + alpha * I`.
#[allow(clippy::needless_range_loop)]
pub fn tikhonov(corr: &[Vec<f64>], alpha: f64) -> Vec<Vec<f64>> {
    let n = corr.len();
    let mut out = corr.to_vec();
    for i in 0..n {
        out[i][i] += alpha;
    }
    out
}

/// Koşul sayısı: `|λmax / λmin|` (Jacobi özdeğerlerinden). Hesaplanamazsa `None`.
pub fn condition_number(corr: &[Vec<f64>]) -> Option<f64> {
    let eigen = jacobi_eigenvalues(corr);
    let mut max = 0.0f64;
    let mut min = f64::INFINITY;
    for &v in &eigen {
        max = max.max(v.abs());
        min = min.min(v.abs());
    }
    if min <= f64::EPSILON {
        None
    } else {
        Some(max / min)
    }
}

/// Güvenli (well-conditioned) korelasyon matrisi üretir: hedef koşul sayısına
/// ulaşana kadar Tikhonov alpha'sını artırır. `None` dönerse veri yetersizdir.
pub fn regularize_correlation_matrix(corr: &[Vec<f64>], target_condition: f64) -> Option<Vec<Vec<f64>>> {
    let n = corr.len();
    if n == 0 {
        return None;
    }
    let mut alpha = 0.001;
    for _ in 0..20 {
        let reg = tikhonov(corr, alpha);
        if let Some(cond) = condition_number(&reg) {
            if cond <= target_condition {
                return Some(reg);
            }
        }
        alpha *= 2.0;
    }
    // Hâlâ kötü koşullu: güçlü shrink ile son deneme.
    let heavy = shrink(corr, 0.5);
    Some(tikhonov(&heavy, 0.01))
}

/// EWMA volatilite (yıllıklandırılmamış, periyot başına): `lambda=0.94`.
pub fn ewma_volatility(returns: &[f64], lambda: f64) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }
    let mut var = 0.0;
    let mut seen = false;
    for &r in returns.iter().rev() {
        let r2 = r * r;
        if !seen {
            var = r2;
            seen = true;
        } else {
            var = lambda * var + (1.0 - lambda) * r2;
        }
    }
    Some(var.sqrt())
}

/// Simetrik gerçel matrisin özdeğerleri (Jacobi döndürmeleri, N≤64).
#[allow(clippy::needless_range_loop)]
pub fn jacobi_eigenvalues(m: &[Vec<f64>]) -> Vec<f64> {
    let n = m.len();
    if n == 0 {
        return Vec::new();
    }
    let mut a = m.to_vec();
    let max_iter = 100 * n * n;
    let mut iter = 0;
    loop {
        // En büyük off-diagonal öğeyi bul.
        let mut max_off = 0.0;
        let mut p = 0usize;
        let mut q = 1usize;
        for i in 0..n {
            for j in (i + 1)..n {
                let v = a[i][j].abs();
                if v > max_off {
                    max_off = v;
                    p = i;
                    q = j;
                }
            }
        }
        if max_off < 1e-12 || iter >= max_iter {
            break;
        }
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        let angle = 0.5 * (2.0 * apq) / (app - aqq).max(1e-300);
        let theta = 0.5 * (angle).atan();
        let c = theta.cos();
        let s = theta.sin();

        // Döndürme.
        for k in 0..n {
            let akp = a[k][p];
            let akq = a[k][q];
            a[k][p] = c * akp - s * akq;
            a[p][k] = a[k][p];
            a[k][q] = s * akp + c * akq;
            a[q][k] = a[k][q];
        }
        a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;
        iter += 1;
    }
    (0..n).map(|i| a[i][i]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_well_conditioned() {
        let n = 8;
        let mut m = vec![vec![0.0f64; n]; n];
        for i in 0..n {
            m[i][i] = 1.0;
        }
        let cond = condition_number(&m).unwrap();
        assert!(cond < 2.0);
    }

    #[test]
    fn singular_matrix_regularizes() {
        // Tümü 1 → rank 1 → tekil.
        let n = 12;
        let mut m = vec![vec![1.0f64; n]; n];
        for i in 0..n {
            m[i][i] = 1.0;
        }
        // Tekil olduğu için koşul sayısı yoktur (min eigen 0).
        assert!(condition_number(&m).is_none());
        let reg = regularize_correlation_matrix(&m, 100.0).unwrap();
        assert!(condition_number(&reg).is_some());
    }

    #[test]
    fn correlation_of_identical_returns_is_one() {
        let a: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
        let b: Vec<f64> = vec![2.0, 4.0, 6.0, 8.0];
        let c = correlation_matrix(&[a, b]);
        assert!((c[0][1] - 1.0).abs() < 1e-9);
        assert!((c[1][0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ewma_vol_sane() {
        let r = vec![0.01, -0.01, 0.02, -0.02, 0.005];
        let v = ewma_volatility(&r, 0.94).unwrap();
        assert!(v > 0.0 && v < 0.1);
    }
}
