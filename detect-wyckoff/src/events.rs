// ============================================================================
// WYCKOFF V4 — KATMAN 4: WYCKOFF OLAY TESPİTİ
// ============================================================================
// Spring: Range dibini test edip güçlü kapanış
// SOS: Yüksek hacimle yukarı kırılım (SMA × 1.5 üzeri hacim)
// UT (Upthrust): Range tepesini test edip düşüş kapanışı
//
// Tüm eşikler Decimal ile hesaplanır; f64 kullanılmaz.
// ============================================================================

use std::collections::VecDeque;

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::types::WyckoffEvent;

fn d(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap_or(Decimal::ZERO)
}

/// Mevcut bar ve pencereye bakarak Wyckoff olayını tespit eder.
///
/// ## Spring
/// Bar'ın dibi, tüm penceredeki en düşük fiyatın %0.2 içindeyse
/// VE bar güçlü bir yükseliş kapanışı yapıyorsa (close > önceki bar close).
///
/// ## SOS (Sign of Strength)
/// Bar'ın kapanışı önceki bar'ın yüksek fiyatını aşıyorsa
/// VE bar hacmi penceredeki ortalama hacmin 1.5 katından fazlaysa.
///
/// ## UT (Upthrust)
/// Bar'ın yüksek fiyatı penceredeki en yüksek fiyatın %2 içindeyse
/// VE bar ayı kapanışı yapıyorsa (close < open).
///
/// Tespit önceliği: Spring > SOS > UT > Neutral
pub fn detect_wyckoff_event(bar: &Kline, window: &VecDeque<Kline>) -> WyckoffEvent {
    if window.len() < 5 {
        return WyckoffEvent::Neutral;
    }

    let range_high = window.iter().map(|b| b.high).fold(Decimal::MIN, Decimal::max);
    let range_low = window.iter().map(|b| b.low).fold(Decimal::MAX, Decimal::min);

    // Ortalama hacim (pencere geneli)
    let avg_vol = if !window.is_empty() {
        window.iter().map(|b| b.volume).sum::<Decimal>() / Decimal::from(window.len())
    } else {
        Decimal::ONE
    };

    // Önceki bar (penceredeki son eleman şu anki bar'dan önce gelir)
    let prev_close = window.back().map(|b| b.close).unwrap_or(bar.open);

    // ── Spring ───────────────────────────────────────────────────────────
    // Bar dibi range dibinin %0.2 içinde + güçlü kapanış
    let spring_threshold = range_low * d(1.002);
    if bar.low <= spring_threshold && bar.close > prev_close {
        return WyckoffEvent::Spring;
    }

    // ── SOS (Sign of Strength) ────────────────────────────────────────────
    // Kapanış önceki bar yüksek'in üstünde + hacim 1.5× ortalama
    let prev_high = window.back().map(|b| b.high).unwrap_or(bar.open);
    if bar.close > prev_high && avg_vol > Decimal::ZERO && bar.volume > avg_vol * d(1.5) {
        return WyckoffEvent::SignOfStrength;
    }

    // ── UT (Upthrust) ────────────────────────────────────────────────────
    // Bar yüksek range tepesinin %2 içinde + ayı kapanışı
    let ut_threshold = range_high * d(0.98);
    if bar.high >= ut_threshold && bar.close < bar.open {
        return WyckoffEvent::Upthrust;
    }

    WyckoffEvent::Neutral
}
