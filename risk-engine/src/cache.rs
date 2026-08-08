//! Risk parametre önbelleği — seqlock tabanlı, sıcak yol okumaları lock-free.
//!
//! Üretici (risk-worker daemon, cold path) 60s'de yazar; tüketiciler
//! (hot path) döngüyü bloklamadan okur. Torn-read koruması seqlock ile sağlanır.

use rust_decimal::Decimal;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU64, Ordering};

/// Worker'ın her çevrimde ürettiği model çıktıları.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RiskParameters {
    /// Sembol sayısı (korelasyon matrisi boyutu). 0 = henüz hesaplanmadı.
    pub n_symbols: usize,
    /// Portföy EWMA volatilite (periyot başına).
    pub portfolio_volatility: f64,
    /// Parametrik portföy VaR (%, 1 gün) — ondalık oran olarak.
    pub var_99_1d_pct: f64,
    /// Korelasyon matrisi koşul sayısı (finite ise).
    pub correlation_condition: f64,
    /// Portföy konsantrasyon HHI (0..=1).
    pub hhi: f64,
    /// Önerilen sembol başına üst pozisyon değeri (USDT).
    pub suggested_max_position_usdt: Decimal,
    /// Önerilen üst kaldıraç (x).
    pub suggested_max_leverage: Decimal,
    /// Model hesaplama zamanı (unix ms).
    pub computed_at_ms: u64,
    /// Model kullanılabilir mi? (false → fail-closed davranın)
    pub available: bool,
    /// Model parametrik kapıya uygun mu? (worker çalışmıyorsa false)
    pub gate_ready: bool,
}

impl RiskParameters {
    pub fn unavailable() -> Self {
        Self {
            available: false,
            ..Default::default()
        }
    }
}

/// Minimal seqlock: yazar seq'i tek yapıp veriyi yazar, sonra çift yapar.
/// Okuyucu seq değişmediyse veriyi güvenle kopyalar.
pub struct Seqlock<T: Copy> {
    seq: AtomicU64,
    value: UnsafeCell<T>,
}

unsafe impl<T: Copy + Send> Sync for Seqlock<T> {}

impl<T: Copy> Seqlock<T> {
    pub fn new(value: T) -> Self {
        Self {
            seq: AtomicU64::new(0),
            value: UnsafeCell::new(value),
        }
    }

    #[inline]
    pub fn read(&self) -> T {
        loop {
            let s1 = self.seq.load(Ordering::Acquire);
            if s1 & 1 == 1 {
                std::hint::spin_loop();
                continue;
            }
            let v = unsafe { *self.value.get() };
            let s2 = self.seq.load(Ordering::Acquire);
            if s1 == s2 {
                return v;
            }
        }
    }

    #[inline]
    pub fn write(&self, value: T) {
        let mut s = self.seq.load(Ordering::Relaxed);
        s += 1; // odd: yazım sürüyor
        self.seq.store(s, Ordering::Release);
        unsafe {
            *self.value.get() = value;
        }
        s += 1; // even: yazım tamam
        self.seq.store(s, Ordering::Release);
    }
}

/// Hot path'in okuduğu parametre önbelleği.
#[derive(Clone)]
pub struct RiskCache {
    inner: Arc<Seqlock<RiskParameters>>,
}

use std::sync::Arc;

impl Default for RiskCache {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Seqlock::new(RiskParameters::unavailable())),
        }
    }

    #[inline]
    pub fn read(&self) -> RiskParameters {
        self.inner.read()
    }

    pub fn write(&self, params: RiskParameters) {
        self.inner.write(params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seqlock_roundtrip() {
        let lock = Seqlock::new(42u64);
        assert_eq!(lock.read(), 42);
        lock.write(7);
        assert_eq!(lock.read(), 7);
    }
}
