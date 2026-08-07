// ============================================================================
// WYCKOFF V4 — KATMAN 6: NARATİF VE YÖN TAVSİYESİ
// ============================================================================
// Tüm analiz katmanlarını birleştirerek Türkçe özet üretir.
// Bias (yön tavsiyesi): Breakout/Breakdown eşiklerine göre kural tabanlı.
// ============================================================================

use rust_decimal::prelude::ToPrimitive;
use crate::types::{Bias, NarrativeInsight, ProbabilityForecast, StructuralPosition, WyckoffEvent};

/// Tüm analiz çıktılarını birleştirerek Türkçe narrative üretir.
pub fn generate_narrative(
    phase_label: &str,
    structure: &StructuralPosition,
    probs: &ProbabilityForecast,
    event: &WyckoffEvent,
) -> NarrativeInsight {
    // Ondalık sayıları % olarak formatlayan yardımcı
    let pct = |d: rust_decimal::Decimal| -> i64 {
        (d * rust_decimal::Decimal::ONE_HUNDRED)
            .round()
            .to_i64()
            .unwrap_or(0)
    };

    let summary = format!(
        "📊 Piyasa Durumu: {}.\n\
         Fiyat {} konumunda. {}\n\
         Yukarı kırılma ihtimali %{}, aşağı kırılma %{}, range devamı %{}.\n\
         POC'a uzaklık: {:.2}%.",
        phase_label,
        structure.price_zone,
        structure.volume_trend,
        pct(probs.breakout_upper),
        pct(probs.breakdown_lower),
        pct(probs.range_continuation),
        structure.poc_distance,
    );

    let wyckoff_event_detected = format!(
        "🔍 Tespit Edilen Wyckoff Olayı: {}",
        event.label()
    );

    let risk_warning = format!(
        "⚠️  RİSK UYARISI: Sahte kırılma riski %{}. Volatilite riski %{:.1}.\n\
         📉 İptal Seviyeleri → Üst: {:.2} | Alt: {:.2}\n\
         📏 Range: {:.2} – {:.2} | POC: {:.2}\n\
         💰 Önerilen Pozisyon Çarpanı: {:.0}%",
        pct(probs.fake_break_risk),
        probs.volatility_risk,
        structure.invalidation_upper,
        structure.invalidation_lower,
        structure.range_low,
        structure.range_high,
        structure.poc_price,
        probs.suggested_position_size_factor * rust_decimal::Decimal::ONE_HUNDRED,
    );

    NarrativeInsight {
        summary,
        wyckoff_event_detected,
        risk_warning,
    }
}

/// Yön tavsiyesi — kural tabanlı.
///
/// - **Bullish**: Yukarı kırılma > %65 VE sahte kırılma riski < %35
/// - **Bearish**: Aşağı kırılma > %55 VE sahte kırılma riski < %30
/// - **Neutral**: Hiçbiri karşılanmıyorsa
pub fn suggest_bias(probs: &ProbabilityForecast) -> Bias {
    let d35 = rust_decimal::Decimal::from_str_exact("0.35").unwrap();
    let d65 = rust_decimal::Decimal::from_str_exact("0.65").unwrap();
    let d55 = rust_decimal::Decimal::from_str_exact("0.55").unwrap();
    let d30 = rust_decimal::Decimal::from_str_exact("0.30").unwrap();

    if probs.breakout_upper > d65 && probs.fake_break_risk < d35 {
        Bias::Bullish
    } else if probs.breakdown_lower > d55 && probs.fake_break_risk < d30 {
        Bias::Bearish
    } else {
        Bias::Neutral
    }
}
