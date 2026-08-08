//! Denetim izi — her risk kararı (onay/red) nedenleriyle kaydedilir.
//!
//! JSONL dosyasına arka plan iş parçacığıyla (flume) yazılır; sıcak yol asla
//! diske yazım bekletmez. `AuditLog::disabled()` ile devre dışı bırakılabilir.

use crate::types::{OrderIntent, RejectReason};
use rust_decimal::Decimal;
use serde::Serialize;
use std::sync::Arc;

/// Disk üzerinde kalıcı bir karar kaydı.
#[derive(Debug, Clone, Serialize)]
pub struct RiskDecisionEvent {
    pub ts_ms: u64,
    pub strategy_id: u32,
    pub symbol: String,
    pub side: String,
    pub quantity: String,
    pub price: Option<String>,
    pub decision: String, // "approved" | "rejected"
    pub rule: Option<String>,
    pub reason: Option<String>,
}

impl RiskDecisionEvent {
    pub fn approved(intent: &OrderIntent, ts_ms: u64) -> Self {
        Self {
            ts_ms,
            strategy_id: intent.strategy_id,
            symbol: intent.symbol.clone(),
            side: intent.side.as_str().to_string(),
            quantity: intent.quantity.to_string(),
            price: intent.price.map(|p| p.to_string()),
            decision: "approved".into(),
            rule: None,
            reason: None,
        }
    }

    pub fn rejected(intent: &OrderIntent, reason: &RejectReason, ts_ms: u64) -> Self {
        Self {
            ts_ms,
            strategy_id: intent.strategy_id,
            symbol: intent.symbol.clone(),
            side: intent.side.as_str().to_string(),
            quantity: intent.quantity.to_string(),
            price: intent.price.map(|p| p.to_string()),
            decision: "rejected".into(),
            rule: Some(reason.rule_name().to_string()),
            reason: Some(reason.describe()),
        }
    }
}

/// Denetim hedefi.
pub trait AuditSink: Send + Sync {
    fn record(&self, event: RiskDecisionEvent);
}

/// JSONL dosyasına arka plan iş parçacığıyla yazan sink.
pub struct JsonLinesAudit {
    tx: flume::Sender<RiskDecisionEvent>,
    _writer: Option<std::thread::JoinHandle<()>>,
}

impl JsonLinesAudit {
    pub fn open(path: impl Into<String>) -> Self {
        let path = path.into();
        let (tx, rx) = flume::unbounded::<RiskDecisionEvent>();
        let writer = std::thread::spawn(move || {
            let file = match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                Ok(f) => f,
                Err(_) => return,
            };
            use std::io::Write;
            let mut w = std::io::BufWriter::new(file);
            while let Ok(ev) = rx.recv() {
                if let Ok(line) = serde_json::to_string(&ev) {
                    let _ = writeln!(w, "{line}");
                }
            }
        });
        Self {
            tx,
            _writer: Some(writer),
        }
    }

    pub fn disabled() -> Self {
        let (tx, rx) = flume::unbounded::<RiskDecisionEvent>();
        let _ = rx; // tüketici yok → kayıtlar düşer
        Self { tx, _writer: None }
    }
}

impl AuditSink for JsonLinesAudit {
    fn record(&self, event: RiskDecisionEvent) {
        let _ = self.tx.try_send(event);
    }
}

impl AuditSink for Arc<JsonLinesAudit> {
    fn record(&self, event: RiskDecisionEvent) {
        self.tx.try_send(event).ok();
    }
}

/// Audit bağlamını taşıyan kısa yol.
pub struct AuditLog {
    sink: Arc<dyn AuditSink>,
}

impl AuditLog {
    pub fn new(sink: Arc<dyn AuditSink>) -> Self {
        Self { sink }
    }

    pub fn disabled() -> Self {
        Self {
            sink: Arc::new(JsonLinesAudit::disabled()),
        }
    }

    pub fn record_approved(&self, intent: &OrderIntent, ts_ms: u64) {
        self.sink.record(RiskDecisionEvent::approved(intent, ts_ms));
    }

    pub fn record_rejected(&self, intent: &OrderIntent, reason: &RejectReason, ts_ms: u64) {
        self.sink.record(RiskDecisionEvent::rejected(intent, reason, ts_ms));
    }

    pub fn record_fill(&self, symbol: &str, quantity: Decimal, price: Decimal) {
        let ev = RiskDecisionEvent {
            ts_ms: now_ms(),
            strategy_id: 0,
            symbol: symbol.to_string(),
            side: "FILL".into(),
            quantity: quantity.to_string(),
            price: Some(price.to_string()),
            decision: "fill".into(),
            rule: None,
            reason: None,
        };
        self.sink.record(ev);
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
