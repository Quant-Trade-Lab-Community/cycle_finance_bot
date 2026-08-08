//! Emir akışı limitleri — kayan pencere hız sınırı (rate limit).

use std::collections::VecDeque;
use std::time::Instant;

/// Kayan pencere (60 sn) emir sayısı sınırı.
#[derive(Debug, Clone)]
pub struct RateLimit {
    max_per_min: u32,
    window: VecDeque<Instant>,
}

impl RateLimit {
    pub fn new(max_per_min: u32) -> Self {
        Self {
            max_per_min,
            window: VecDeque::new(),
        }
    }

    pub fn max_per_min(&self) -> u32 {
        self.max_per_min
    }

    pub fn set_max_per_min(&mut self, v: u32) {
        self.max_per_min = v;
    }

    /// Limit dolduysa `Err(limit)` döner.
    pub fn check(&mut self) -> Result<(), u32> {
        self.prune();
        if self.max_per_min == 0 {
            return Ok(());
        }
        if self.window.len() >= self.max_per_min as usize {
            return Err(self.max_per_min);
        }
        Ok(())
    }

    /// Başarılı gönderim sonrası pencereye kaydet.
    pub fn record(&mut self) {
        self.prune();
        self.window.push_back(Instant::now());
    }

    pub fn count(&mut self) -> usize {
        self.prune();
        self.window.len()
    }

    fn prune(&mut self) {
        let cutoff = Instant::now() - std::time::Duration::from_secs(60);
        while self.window.front().is_some_and(|t| *t < cutoff) {
            self.window.pop_front();
        }
    }
}

/// Circuit breaker — ardışık red sayaçlı otomatik durdurma.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub consecutive_rejections: u32,
    pub max_rejections: u32,
}

impl CircuitBreaker {
    pub fn new(max_rejections: u32) -> Self {
        Self {
            consecutive_rejections: 0,
            max_rejections,
        }
    }

    pub fn record_rejection(&mut self) -> bool {
        self.consecutive_rejections += 1;
        self.max_rejections > 0 && self.consecutive_rejections >= self.max_rejections
    }

    pub fn record_approval(&mut self) {
        self.consecutive_rejections = 0;
    }

    pub fn reset(&mut self) {
        self.consecutive_rejections = 0;
    }
}
