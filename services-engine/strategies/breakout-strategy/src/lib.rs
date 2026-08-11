//! velvetusdt — VELVETUSDT stratejisi + mikro-yapı metrik çekirdeği.
//!
//! Modüller:
//! - `metrics`: mikro-yapı metrikleri (TPS, aVPIN, EfP, ...)
//! - `breakout`: Kripto Futures tek zaman dilimi kırılım tespit algoritması (Sürüm 1.0)
//! - `indicators`: ATR(14), SMA(20), High/Low(14) hesaplayıcıları
//! - `feed`: flow ring'lerinden türev veri toplayıcı (CVD, OI, funding, mark, last, liq)

pub mod metrics;
pub mod breakout;
pub mod indicators;
pub mod feed;
