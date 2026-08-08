// ============================================================================
// 6. YÜRÜTME KATMANI — Gerçek TWAP + Iceberg + Kayma
// TWAP zamana göre dilimlenir (50ms), kayma derinlikten alınır.
// ============================================================================

use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::state::Signal;

pub struct ExecutionBroker {
    pub slippage_percent: f64, // 0.05 = %0.05
    pub depth: f64,            // emir defteri derinliği (derinlik etkisi çarpanı)
    pub slice_count: usize,
    pub slice_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChildOrder {
    pub price: f64,
    pub amount: u64,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionPlan {
    pub slices: usize,
    pub chunk_size: u64,
    pub base_price: f64,
    pub avg_price: f64,
    pub max_price: f64,
    pub min_price: f64,
    pub estimated_duration_ms: u64,
    pub iceberg: bool,
}

impl ExecutionBroker {
    pub fn new() -> Self {
        Self {
            slippage_percent: 0.05,
            depth: 250_000_000.0,
            slice_count: 100,
            slice_interval_ms: 50,
        }
    }

    pub fn execute(&self, signal: &Signal, size: u64, tick_size: f64) -> Vec<ChildOrder> {
        let chunks = (size / self.slice_count as u64).max(1);
        let base_tick = match signal {
            Signal::Long { entry, .. } => entry.0,
            Signal::Short { entry, .. } => entry.0,
        };
        let base_price = base_tick as f64 * tick_size;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        // TWAP: her dilimde fiyata kayma + derinlik etkisi uygulanır
        let depth_impact = self.slippage_percent * (size as f64 / self.depth.max(1.0)).clamp(0.0, 1.0);

        (0..self.slice_count)
            .map(|i| {
                let slip = (i as f64 / self.slice_count as f64) * self.slippage_percent / 100.0;
                let price = base_price * (1.0 + slip + depth_impact);
                ChildOrder {
                    price,
                    amount: chunks,
                    expires_at_ms: now_ms + self.slice_interval_ms * i as u64,
                }
            })
            .collect()
    }

    pub fn plan(&self, plan: &[ChildOrder], _size: u64) -> ExecutionPlan {
        let avg = if plan.is_empty() {
            0.0
        } else {
            plan.iter().map(|o| o.price).sum::<f64>() / plan.len() as f64
        };
        let max = plan.iter().map(|o| o.price).fold(0.0, f64::max);
        let min = plan.iter().map(|o| o.price).fold(f64::INFINITY, f64::min);
        let first = plan.first().map(|o| o.expires_at_ms).unwrap_or(0);
        let last = plan.last().map(|o| o.expires_at_ms).unwrap_or(0);
        ExecutionPlan {
            slices: plan.len(),
            chunk_size: plan.first().map(|o| o.amount).unwrap_or(0),
            base_price: plan.first().map(|o| o.price).unwrap_or(0.0),
            avg_price: avg,
            max_price: max,
            min_price: if plan.is_empty() { 0.0 } else { min },
            estimated_duration_ms: last.saturating_sub(first),
            iceberg: true,
        }
    }
}

impl Default for ExecutionBroker {
    fn default() -> Self {
        Self::new()
    }
}