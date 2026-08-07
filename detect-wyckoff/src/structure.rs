// ============================================================================
// WYCKOFF V4 — KATMAN 2: YAPISAL KONUM
// ============================================================================
// POC (Point of Control): Volume Profile ile hacim yoğunluğu en yüksek fiyat
// Spread durumu, hacim trendi ve fiyat bandı konumu belirlenir.
// İptal seviyeleri (invalidation) hesaplanır.
// ============================================================================

use std::collections::VecDeque;

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::types::StructuralPosition;

fn d(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap_or(Decimal::ZERO)
}

/// Penceredeki bar'lardan yapısal konumu çıkarır.
///
/// - POC: Fiyat aralığını 100 bucket'a böler, en çok hacim toplanan bucket'ın
///   orta noktasını POC olarak döner (detect-ms liquidity.rs ile paralel mantık).
/// - Price Zone: Fiyatın range içindeki yüzdesel konumuna göre etiket üretir.
/// - Spread Status: Son bar spread'ini son 10 bar ortalamasıyla karşılaştırır.
/// - Volume Trend: Son bar hacmini son 5 bar ortalamasıyla karşılaştırır.
/// - Invalidation: Range sınırlarının %1.5 dışındaki seviyeler.
pub fn extract_structure(bar: &Kline, window: &VecDeque<Kline>) -> StructuralPosition {
    let range_high = window.iter().map(|b| b.high).fold(Decimal::MIN, Decimal::max);
    let range_low = window.iter().map(|b| b.low).fold(Decimal::MAX, Decimal::min);

    // ── POC: Volume Profile (100 bucket) ──────────────────────────────────
    let poc_price = calculate_poc(window, 100).unwrap_or(bar.close);

    // POC mesafesi (yüzde)
    let poc_distance = if poc_price > Decimal::ZERO {
        ((bar.close - poc_price) / poc_price) * Decimal::ONE_HUNDRED
    } else {
        Decimal::ZERO
    };

    // ── Price Zone ────────────────────────────────────────────────────────
    let price_zone = if range_high > range_low {
        let price_ratio = (bar.close - range_low) / (range_high - range_low);
        if price_ratio >= d(0.95) {
            "Range'in Üst Bantı (Direnişe Yakın)".to_string()
        } else if price_ratio <= d(0.05) {
            "Range'in Alt Bantı (Desteğe Yakın)".to_string()
        } else {
            "Range'in Orta Bantı (Kararsız)".to_string()
        }
    } else {
        "Range Belirsiz (Yetersiz Veri)".to_string()
    };

    // ── Volume Trend ──────────────────────────────────────────────────────
    let vol_5: Vec<Decimal> = window.iter().rev().take(5).map(|b| b.volume).collect();
    let avg_vol_5 = if vol_5.is_empty() {
        Decimal::ONE
    } else {
        vol_5.iter().sum::<Decimal>() / Decimal::from(vol_5.len())
    };

    let vol_trend = if avg_vol_5 > Decimal::ZERO {
        if bar.volume > avg_vol_5 * d(1.2) {
            "Artan Hacim (Aktif Katılım)".to_string()
        } else if bar.volume < avg_vol_5 * d(0.8) {
            "Azalan Hacim (İlgisizlik / Tuzak)".to_string()
        } else {
            "Yatay Hacim (Normal)".to_string()
        }
    } else {
        "Hacim Verisi Yetersiz".to_string()
    };

    // ── Spread Status ─────────────────────────────────────────────────────
    let spread = bar.high - bar.low;
    let spreads_10: Vec<Decimal> = window
        .iter()
        .rev()
        .take(10)
        .map(|b| b.high - b.low)
        .collect();
    let avg_spread = if spreads_10.is_empty() {
        Decimal::ONE
    } else {
        spreads_10.iter().sum::<Decimal>() / Decimal::from(spreads_10.len())
    };

    let spread_status = if avg_spread > Decimal::ZERO {
        if spread < avg_spread * d(0.8) {
            "Daralıyor (Sıkışma — Kırılım Yakın)".to_string()
        } else if spread > avg_spread * d(1.2) {
            "Genişliyor (Oynaklık Artıyor)".to_string()
        } else {
            "Normal Aralık".to_string()
        }
    } else {
        "Spread Verisi Yetersiz".to_string()
    };

    // ── Invalidation Seviyeleri (%1.5 buffer) ────────────────────────────
    let inv_upper = range_high * d(1.015);
    let inv_lower = range_low * d(0.985);

    StructuralPosition {
        price_zone,
        poc_distance,
        volume_trend: vol_trend,
        spread_status,
        invalidation_upper: inv_upper,
        invalidation_lower: inv_lower,
        poc_price,
        range_high,
        range_low,
    }
}

/// Volume Profile POC hesaplaması.
///
/// Fiyat aralığını `bucket_count` eşit parçaya böler.
/// Her bar'ın hacmini high-low aralığına orantılı dağıtır.
/// En fazla hacim toplanan bucket'ın orta noktasını döner.
fn calculate_poc(window: &VecDeque<Kline>, bucket_count: usize) -> Option<Decimal> {
    if window.is_empty() || bucket_count == 0 {
        return None;
    }

    let price_min = window.iter().map(|b| b.low).fold(Decimal::MAX, Decimal::min);
    let price_max = window.iter().map(|b| b.high).fold(Decimal::MIN, Decimal::max);

    if price_max <= price_min {
        return None;
    }

    let bucket_size = (price_max - price_min) / Decimal::from(bucket_count);
    if bucket_size <= Decimal::ZERO {
        return None;
    }

    let mut buckets = vec![Decimal::ZERO; bucket_count];

    for bar in window {
        let low_idx = ((bar.low - price_min) / bucket_size)
            .floor()
            .to_usize()
            .unwrap_or(0)
            .min(bucket_count - 1);
        let high_idx = ((bar.high - price_min) / bucket_size)
            .floor()
            .to_usize()
            .unwrap_or(0)
            .min(bucket_count - 1);

        let span = Decimal::from(high_idx - low_idx + 1);
        let vol_per_bucket = if span > Decimal::ZERO {
            bar.volume / span
        } else {
            bar.volume
        };

        for b in low_idx..=high_idx {
            buckets[b] += vol_per_bucket;
        }
    }

    // En yüksek hacimli bucket'ın orta noktası
    let (best_idx, _) = buckets
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))?;

    let poc = price_min + Decimal::from(best_idx) * bucket_size + bucket_size / Decimal::TWO;
    Some(poc)
}
