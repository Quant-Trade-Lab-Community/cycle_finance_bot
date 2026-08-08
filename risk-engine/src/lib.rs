//! # Risk-Engine — Cycle Finance ortak risk çekirdeği
//!
//! Tek doğruluk kaynağı (single source of truth): tüm risk kuralları burada yaşar.
//! `execution-engine` (hot path, pre-trade) ve `risk-worker` daemon (cold path,
//! korelasyon/VaR parametre üretimi) aynı kodu kullanır.
//!
//! ## İlkeler
//! - **Fail-closed**: durum bilinmiyorsa emir reddedilir (mark stale → red).
//! - **Para `Decimal`'dir, asla `f64`**: PnL/limit/pozisyon/marj `rust_decimal`.
//!   `f64` yalnızca istatistiksel modellerde (korelasyon, VaR).
//! - **Hot path allocation-free**: `RiskEngine::evaluate` sıralı kural zinciri.
//! - **Her karar denetlenebilir**: `AuditLog` tüm onay/redleri kaydeder.
//! - **Kill switch otomatik + manuel**: günlük kayıp/drawdown aşımı veya 3+
//!   ardışık red → otomatik kapan. Sadece manuel açılır.

pub mod accounting;
pub mod audit;
pub mod cache;
pub mod config;
pub mod correlation;
pub mod engine;
pub mod exposure;
pub mod kill_switch;
pub mod limits;
pub mod liquidity;
pub mod policy;
pub mod state;
pub mod types;
pub mod var;
pub mod worker;

pub use accounting::{Portfolio, Position};
pub use audit::{AuditLog, AuditSink, RiskDecisionEvent};
pub use cache::{RiskCache, RiskParameters};
pub use config::{load_risk_config, load_risk_config_from};
pub use engine::RiskEngine;
pub use kill_switch::KillSwitch;
pub use policy::{PerSymbolLimits, RiskPolicy};
pub use state::{RiskSnapshot, RiskState, RiskStateInner};
pub use types::{
    Fill, MarkPrice, OrderIntent, OrderKind, RejectReason, RiskDecision, RiskStatus, Side,
};
