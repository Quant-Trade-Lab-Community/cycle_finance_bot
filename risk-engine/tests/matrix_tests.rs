//! Korelasyon / VaR / worker matematik testleri.

use risk_engine::cache::RiskCache;
use risk_engine::correlation::{condition_number, correlation_matrix, ewma_volatility, jacobi_eigenvalues, regularize_correlation_matrix};
use risk_engine::var::parametric_var_99_1d;
use risk_engine::worker::{RiskWorker, WorkerConfig};
use rust_decimal::Decimal;
use std::sync::Arc;

#[test]
fn correlation_of_identical_returns_is_one() {
    let a: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let b: Vec<f64> = vec![3.0, 6.0, 9.0, 12.0, 15.0];
    let c = correlation_matrix(&[a, b]);
    assert!((c[0][1] - 1.0).abs() < 1e-9);
}

#[test]
fn singular_matrix_regularizes_to_finite_condition() {
    let n = 16;
    let mut m = vec![vec![1.0f64; n]; n];
    for i in 0..n {
        m[i][i] = 1.0;
    }
    assert!(condition_number(&m).is_none(), "tekil matrisin koşul sayısı olmamalı");
    let reg = regularize_correlation_matrix(&m, 100.0).unwrap();
    assert!(condition_number(&reg).is_some(), "regularize sonrası koşul sayısı hesaplanabilmeli");
}

#[test]
fn jacobi_eigenvalue_trace_preserved() {
    let n = 6;
    let mut m = vec![vec![0.0f64; n]; n];
    let mut trace = 0.0;
    for i in 0..n {
        m[i][i] = (i + 1) as f64;
        trace += (i + 1) as f64;
    }
    let eigen = jacobi_eigenvalues(&m);
    let sum: f64 = eigen.iter().sum();
    assert!((sum - trace).abs() < 1e-6, "Jacobi izi korumalı: {sum} vs {trace}");
}

#[test]
fn parametric_var_increases_with_volatility() {
    // Tek varlık: corr=1, vol 0.01 vs 0.02.
    let c1 = vec![vec![1.0]];
    let v1 = parametric_var_99_1d(&c1, &[0.01], &[1.0]).unwrap();
    let v2 = parametric_var_99_1d(&c1, &[0.02], &[1.0]).unwrap();
    assert!(v2 > v1);
    // 2.326 * 0.01 ≈ 0.0233.
    assert!((v1 - 0.0233).abs() < 1e-3);
}

#[test]
fn worker_produces_available_params_after_enough_samples() {
    let cache = Arc::new(RiskCache::new());
    let mut worker = RiskWorker::new(WorkerConfig::default(), cache.clone());
    for i in 0..20 {
        worker.ingest_mark("BTCUSDT", 100.0 + (i as f64 * 0.5));
        worker.ingest_mark("ETHUSDT", 2000.0 + (i as f64 * 10.0));
    }
    let params = worker.run_cycle(1_000_000);
    assert!(params.available, "yeterli örnekle parametre üretilmeli");
    assert!(params.var_99_1d_pct > 0.0);
    assert!(params.suggested_max_position_usdt > Decimal::ZERO);
}

#[test]
fn worker_unavailable_without_samples() {
    let cache = Arc::new(RiskCache::new());
    let mut worker = RiskWorker::new(WorkerConfig::default(), cache.clone());
    let params = worker.run_cycle(1_000_000);
    assert!(!params.available);
    assert!(!cache.read().available);
}

#[test]
fn ewma_vol_is_finite_and_positive() {
    let returns = vec![0.01, -0.02, 0.03, -0.01, 0.005];
    let v = ewma_volatility(&returns, 0.94).unwrap();
    assert!(v > 0.0 && v < 0.1);
}
