//! Sözleşme katmanı (Layer 0 — Contracts).
//!
//! Katmanlar arası sabit sözleşmeler burada yaşar:
//! - `events`: Tüm katmanların üzerinde anlaştığı market veri modeli
//!   (`OwnedEvent` / `EventType` — ring buffer üzerinden taşınan veri).
//! - `wire`: Ownership → compact binary frame codec (ring üzerindeki format).

pub mod events;
pub mod wire;