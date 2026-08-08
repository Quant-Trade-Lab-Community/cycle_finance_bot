//! Kill switch — ortak `risk_engine::KillSwitch` üzerinden (tek doğruluk kaynağı).
//!
//! API geriye dönük uyumludur: `new(path)`, `is_open()`, `engage()`, `release()`.

pub use risk_engine::kill_switch::KillSwitch;
