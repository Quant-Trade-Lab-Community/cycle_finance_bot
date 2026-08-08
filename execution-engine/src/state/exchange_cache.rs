//! ExchangeInfo önbelleği.
//!
//! `/fapi/v1/exchangeInfo` ağır bir yanıttır (~300KB); her emirde çekilmez.
//! Periyodik yenilenir; ilk yükleme zorunludur (preflight onsuz çalışmaz).

use crate::client::BinanceClient;
use crate::error::Result;
use crate::types::exchange::{ExchangeInfo, SymbolFilter, SymbolInfo};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ExchangeCache {
    inner: Arc<RwLock<ExchangeInfo>>,
    last_refresh: Arc<RwLock<u64>>,
    refresh_interval_sec: u64,
}

impl ExchangeCache {
    pub fn new(refresh_interval_sec: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ExchangeInfo::default())),
            last_refresh: Arc::new(RwLock::new(0)),
            refresh_interval_sec,
        }
    }

    pub fn handle(&self) -> Arc<RwLock<ExchangeInfo>> {
        self.inner.clone()
    }

    pub fn get(&self) -> ExchangeInfo {
        self.inner.read().clone()
    }

    pub fn symbol(&self, symbol: &str) -> Option<SymbolInfo> {
        self.inner.read().symbol(symbol).cloned()
    }

    pub fn loaded(&self) -> bool {
        !self.inner.read().symbols.is_empty()
    }

    pub async fn refresh(&self, client: &BinanceClient) -> Result<()> {
        let info = client.exchange_info().await?;
        *self.inner.write() = info;
        *self.last_refresh.write() = now_ms();
        Ok(())
    }

    pub async fn refresh_if_stale(&self, client: &BinanceClient) -> Result<()> {
        let stale = {
            let lr = *self.last_refresh.read();
            now_ms().saturating_sub(lr) > self.refresh_interval_sec * 1000
        };
        if stale || !self.loaded() {
            self.refresh(client).await?;
        }
        Ok(())
    }
}

/// Sembol kurallarına göre fiyat/miktar yuvarlama yardımcıları.
/// Miktarı step_size'ın katına yuvarlar (aşağı).
pub fn round_qty_to_step(qty: rust_decimal::Decimal, step: rust_decimal::Decimal) -> rust_decimal::Decimal {
    if step <= rust_decimal::Decimal::ZERO {
        return qty;
    }
    (qty / step).floor() * step
}

/// Fiyatı tick_size'ın katına yuvarlar (yarım-yukarı — banker's yok).
pub fn round_price_to_tick(price: rust_decimal::Decimal, tick: rust_decimal::Decimal) -> rust_decimal::Decimal {
    if tick <= rust_decimal::Decimal::ZERO {
        return price;
    }
    let div = price / tick;
    // Pozitif değerler için yarım-yukarı: floor(div + 0.5).
    let rounded = (div + rust_decimal::Decimal::from(5) / rust_decimal::Decimal::from(10)).floor();
    rounded * tick
}

/// Onluk kesir hassasiyetine yuvarlar.
pub fn round_to_precision(value: rust_decimal::Decimal, precision: u32) -> rust_decimal::Decimal {
    let scale = rust_decimal::Decimal::from(10u64.pow(precision));
    (value * scale).round() / scale
}

/// Lot step'i ve precizyon bilgisini SymbolInfo'dan çeker.
pub fn lot_step(info: &SymbolInfo) -> Option<rust_decimal::Decimal> {
    info.filters.iter().find_map(|f| match f {
        SymbolFilter::LotSize { step_size, .. } => Some(*step_size),
        _ => None,
    })
}

pub fn tick_size(info: &SymbolInfo) -> Option<rust_decimal::Decimal> {
    info.filters.iter().find_map(|f| match f {
        SymbolFilter::PriceFilter { tick_size, .. } => Some(*tick_size),
        _ => None,
    })
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
