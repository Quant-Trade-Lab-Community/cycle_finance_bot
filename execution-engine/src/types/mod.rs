//! Execution servisi veri modeli.
//!
//! Binance USDT-M Futures yanıtlarının tipli görünümleri. Tüm alanlar
//! `camelCase` JSON ile eşlenir; sayısal değerler string olarak gelir ve
//! `rust_decimal`'e çevrilir.

pub mod account;
pub mod exchange;
pub mod income;
pub mod position;
pub mod user_event;

pub use account::{AccountInfo, AccountPosition, AssetBalance, Balance, MarginType};
pub use exchange::{ExchangeInfo, RateLimit, SymbolFilter, SymbolInfo};
pub use income::{Income, IncomeType};
pub use position::{PositionRisk, PositionSide};
pub use user_event::UserDataEvent;
