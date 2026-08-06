use std::sync::atomic::{AtomicI64, AtomicU64, AtomicU8, Ordering};
use serde::{Serialize, Deserialize};

// We will use standard atomics for lock-free observation.
// The orchestrator writes, the RPC thread reads.

pub struct SharedMetrics {
    pub pnl: AtomicI64,
    pub free_balance: AtomicU64,
    pub ring_buffer_usage: AtomicU8,
    // Using AtomicU64 for latency since Epoch GC on HDR Histogram requires extra crates
    // and manual lifecycle management which could be overkill here.
    pub p99_latency_ns: AtomicU64,
}

impl SharedMetrics {
    pub fn new() -> Self {
        Self {
            pnl: AtomicI64::new(0),
            free_balance: AtomicU64::new(0),
            ring_buffer_usage: AtomicU8::new(0),
            p99_latency_ns: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            p99_latency_ns: self.p99_latency_ns.load(Ordering::Acquire),
            pnl: self.pnl.load(Ordering::Acquire),
            free_balance: self.free_balance.load(Ordering::Acquire),
            ring_buffer_usage: self.ring_buffer_usage.load(Ordering::Acquire),
        }
    }
}

// We derive Serialize so postcard can convert it to raw bytes for WebSocket
#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsSnapshot {
    pub p99_latency_ns: u64,
    pub pnl: i64,
    pub free_balance: u64,
    pub ring_buffer_usage: u8,
}
