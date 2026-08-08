// ============================================================================
// 3. HACİM PROFİLİ — "Lazy Decay" ile Amortize O(1)
// update() tüm bucket'ları dolaşmaz; her bucket kendi last_update'ini taşır.
// Okuma anında decay uygulanır. POC: BTreeMap üzerinde O(log n) arama.
// ============================================================================

use std::collections::BTreeMap;

use serde::Serialize;

use crate::models::{Bar, Tick};

#[derive(Debug, Clone, Copy)]
pub struct BucketEntry {
    pub volume: u64,
    pub last_update: i64, // Bar timestamp (ms)
}

const MAX_BUCKETS: usize = 4096;

#[derive(Debug, Clone)]
pub struct IncrementalVolumeProfile {
    buckets: BTreeMap<i64, BucketEntry>,
    total_volume: u128,
    decay_factor: f64, // 0.999 — dakika bazlı bozunma
    current_time: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VolumeProfileSnapshot {
    pub poc_price: f64,
    pub total_volume: u128,
    pub bucket_count: usize,
    pub top_buckets: Vec<(f64, u64)>,
}

impl IncrementalVolumeProfile {
    pub fn new() -> Self {
        Self::with_decay(0.999)
    }

    pub fn with_decay(decay_factor: f64) -> Self {
        Self {
            buckets: BTreeMap::new(),
            total_volume: 0,
            decay_factor,
            current_time: 0,
        }
    }

    /// Sadece ilgili bucket'ı günceller — O(log n).
    pub fn update(&mut self, bar: &Bar) {
        self.current_time = bar.timestamp;
        let mid = bar.mid_tick().0;
        let entry = self
            .buckets
            .entry(mid)
            .or_insert(BucketEntry { volume: 0, last_update: bar.timestamp });
        let age = (bar.timestamp - entry.last_update).max(0) as f64;
        let decayed = (entry.volume as f64) * (self.decay_factor.powf(age / 60_000.0));
        entry.volume = (decayed + bar.volume.0 as f64) as u64;
        entry.last_update = bar.timestamp;
        self.total_volume = self.total_volume.saturating_add(bar.volume.0 as u128);

        if self.buckets.len() > MAX_BUCKETS {
            let drop = self.buckets.len() / 2;
            let keys: Vec<i64> = self.buckets.keys().take(drop).copied().collect();
            for k in keys {
                self.buckets.remove(&k);
            }
        }
    }

    /// Okuma anında decay uygula: bucket'ın güncel hacmini döndürür.
    pub fn live_volume(&self, key: i64) -> f64 {
        match self.buckets.get(&key) {
            Some(e) => {
                let age = (self.current_time - e.last_update).max(0) as f64;
                (e.volume as f64) * (self.decay_factor.powf(age / 60_000.0))
            }
            None => 0.0,
        }
    }

    /// POC: en yüksek hacimli bucket — BTreeMap iter, O(log n) amortized.
    pub fn poc(&self) -> Tick {
        Tick(
            self.buckets
                .iter()
                .max_by_key(|(_, e)| e.volume)
                .map(|(k, _)| *k)
                .unwrap_or(0),
        )
    }

    pub fn snapshot(&self, tick_size: f64, n: usize) -> VolumeProfileSnapshot {
        let mut ranked: Vec<(i64, u64)> = self
            .buckets
            .iter()
            .map(|(k, e)| (*k, e.volume))
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));
        ranked.truncate(n);
        VolumeProfileSnapshot {
            poc_price: self.poc().0 as f64 * tick_size,
            total_volume: self.total_volume,
            bucket_count: self.buckets.len(),
            top_buckets: ranked
                .into_iter()
                .map(|(k, v)| (k as f64 * tick_size, v))
                .collect(),
        }
    }
}

impl Default for IncrementalVolumeProfile {
    fn default() -> Self {
        Self::new()
    }
}