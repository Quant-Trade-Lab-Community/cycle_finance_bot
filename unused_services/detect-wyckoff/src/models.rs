// ============================================================================
// 1. ÇEKİRDEK ONTOLOJİ — "Varlık Bilinci" (Zero-Cost Precision)
// Tick tabanlı taşma kontrollü aritmetik. Tüm tipler deny'den geçer.
// ============================================================================

use serde::{Deserialize, Serialize};

/// Fiyat / TickSize — taşma kontrollü kullanılır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Tick(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Volume(pub u64);

#[derive(Debug, Clone)]
pub struct AssetDefinition {
    pub tick_size: f64,
    pub min_move: i64, // Tick cinsinden minimum adım — filtrelemede kullanılır
}

impl AssetDefinition {
    pub fn default_asset() -> Self {
        Self { tick_size: 1e-6, min_move: 1 }
    }
    pub fn btc() -> Self {
        Self { tick_size: 1e-6, min_move: 50 }
    }
}

#[derive(Debug, Clone)]
pub struct Bar {
    pub timestamp: i64,
    pub high: Tick,
    pub low: Tick,
    pub open: Tick,
    pub close: Tick,
    pub volume: Volume,
}

impl Bar {
    pub fn spread_ticks(&self) -> i64 {
        self.high.0.saturating_sub(self.low.0)
    }
    pub fn mid_tick(&self) -> Tick {
        Tick(self.high.0.saturating_add(self.low.0) / 2)
    }
    pub fn price(&self, tick_size: f64) -> f64 {
        self.close.0 as f64 * tick_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bias {
    Bullish,
    Bearish,
    Neutral,
}

impl Bias {
    pub fn label(&self) -> &'static str {
        match self {
            Bias::Bullish => "Bullish",
            Bias::Bearish => "Bearish",
            Bias::Neutral => "Neutral",
        }
    }
}