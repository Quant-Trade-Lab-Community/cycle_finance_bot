// ============================================================================
// 7. GÖZLEMLENEBİLİRLİK — Tüm Kararlar Immutable Log
// Her bar, her skor, her ağırlık güncellemesi JSON audit trail'e yazılır.
// ============================================================================

use serde_json::json;

use crate::models::{Bar, Bias};
use crate::state::{ProbabilisticState, Signal};

pub struct AuditRecord;

impl AuditRecord {
    /// Tek bir kararı JSON nesnesine çevirir (immutable log satırı).
    pub fn decision(
        bar: &Bar,
        score: f64,
        event_label: &str,
        phase: &ProbabilisticState,
        bias: Bias,
        signal: Option<&Signal>,
        tick_size: f64,
    ) -> serde_json::Value {
        json!({
            "timestamp": bar.timestamp,
            "close": bar.close.0 as f64 * tick_size,
            "spread_ticks": bar.spread_ticks(),
            "volume": bar.volume.0,
            "score": (score * 10000.0).round() / 10000.0,
            "event": event_label,
            "acc": (phase.accumulation_weight * 10000.0).round() / 10000.0,
            "dist": (phase.distribution_weight * 10000.0).round() / 10000.0,
            "trend_strength": (phase.trend_strength * 10000.0).round() / 10000.0,
            "bias": bias.label(),
            "signal": signal.map(|s| s.label()),
        })
    }
}