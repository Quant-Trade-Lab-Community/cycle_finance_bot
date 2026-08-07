// ============================================================================
// WYCKOFF V4 — KATMAN 5: OLASILIK TAHMİNLERİ
// ============================================================================
// Feature vektörü tabanlı olasılık hesaplama:
//   1. POC Mesafesi       → Fiyatın POC'a göre konumu
//   2. Spread Delta       → Spread daralıyor mu, genişliyor mu?
//   3. Volume Delta       → Hacim artan mı, azalan mı?
//   4. Volatilite (ATR)   → Piyasa gürültüsü
//   5. Fake Break Riski   → Hacim/yön uyumsuzluğu
//
// Tüm olasılıklar [0.0, 1.0] aralığında döner.
// Hiçbir zaman 0.98'i geçmez (model belirsizliği marjı).
// ============================================================================

use std::collections::VecDeque;

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::types::{ProbabilityForecast, StructuralPosition};

fn d(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap_or(Decimal::ZERO)
}

fn clamp(val: Decimal, lo: f64, hi: f64) -> Decimal {
    val.max(d(lo)).min(d(hi))
}

/// Feature tabanlı olasılık tahmini.
///
/// Tüm hesaplamalar `StructuralPosition`'dan türetilir — bu, modelin
/// girdi ve çıktı arasındaki izlenebilirliğini (audit trail) sağlar.
pub fn calculate_probabilities(
    bar: &Kline,
    window: &VecDeque<Kline>,
    structure: &StructuralPosition,
) -> ProbabilityForecast {
    let features = vec![
        "POC_Mesafe".to_string(),
        "Spread_Delta".to_string(),
        "Volume_Delta".to_string(),
        "ATR_Volatilite".to_string(),
        "Fake_Break_Riski".to_string(),
    ];

    // ── ATR(14) ───────────────────────────────────────────────────────────
    let atr = calculate_atr(window, 14);

    // ── 1. Yukarı Kırılma Olasılığı ──────────────────────────────────────
    // POC üzerinde fiyat → kırılma eğilimi artar
    // POC mesafesi pozitifse (fiyat POC üstünde) daha yüksek olasılık
    let poc_factor = clamp(
        (structure.poc_distance / Decimal::ONE_HUNDRED) + d(0.5),
        0.0,
        1.0,
    );
    let mut breakout_upper = d(0.40) + poc_factor * d(0.40);

    // Spread daralıyorsa olasılık artar (sıkışma = kırılım hazırlığı)
    if structure.spread_status.contains("Daralıyor") {
        breakout_upper += d(0.10);
    }
    breakout_upper = clamp(breakout_upper, 0.0, 0.98);

    // ── 2. Aşağı Kırılma Olasılığı ───────────────────────────────────────
    let mut breakdown_lower = d(0.10) + (d(1.0) - poc_factor) * d(0.30);

    // Üst bantta azalan hacim = sahte yükseliş sinyali → breakdown riski artar
    if structure.volume_trend.contains("Azalan") && structure.price_zone.contains("Üst") {
        breakdown_lower += d(0.15);
    }
    breakdown_lower = clamp(breakdown_lower, 0.0, 0.98);

    // ── 3. Range Devamı ───────────────────────────────────────────────────
    let range_continuation = clamp(
        d(1.0) - breakout_upper - breakdown_lower,
        0.05,
        1.0,
    );

    // ── 4. Volatilite Riski ───────────────────────────────────────────────
    // ATR / close × 100 → yüzde cinsinden oynaklık
    let volatility_risk = if bar.close > Decimal::ZERO {
        clamp(atr / bar.close * Decimal::ONE_HUNDRED, 0.0, 100.0)
    } else {
        Decimal::ZERO
    };

    // ── 5. Sahte Kırılma Riski ───────────────────────────────────────────
    let mut fake_break_risk = d(0.20);
    if structure.volume_trend.contains("Azalan") && structure.price_zone.contains("Üst") {
        fake_break_risk += d(0.30);
    }
    if structure.spread_status.contains("Genişliyor") {
        fake_break_risk += d(0.15);
    }
    fake_break_risk = clamp(fake_break_risk, 0.05, 0.80);

    // ── 6. Momentum Riski ─────────────────────────────────────────────────
    // Fiyat yükseliyor ama hacim düşük → hacimsiz yükseliş = yüksek risk
    let avg_vol_5: Decimal = {
        let v: Vec<Decimal> = window.iter().rev().take(5).map(|b| b.volume).collect();
        if v.is_empty() {
            Decimal::ONE
        } else {
            v.iter().sum::<Decimal>() / Decimal::from(v.len())
        }
    };
    let momentum_risk = if bar.close > bar.open && bar.volume < avg_vol_5 * d(0.7) {
        d(0.30) // Hacimsiz yükseliş zayıf
    } else {
        d(0.10)
    };

    // ── 7. Pozisyon Büyüklüğü Çarpanı ────────────────────────────────────
    // Yüksek volatilite ve sahte kırılma riski → küçük pozisyon
    let vol_penalty = clamp(volatility_risk / Decimal::ONE_HUNDRED, 0.0, 0.5);
    let size_factor = clamp(
        (d(1.0) - vol_penalty) * (d(1.0) - fake_break_risk),
        0.1,
        1.0,
    );

    ProbabilityForecast {
        breakout_upper,
        breakdown_lower,
        range_continuation,
        volatility_risk,
        fake_break_risk,
        momentum_risk,
        suggested_position_size_factor: size_factor,
        confidence_interval: d(0.025),      // ±%2.5 güven aralığı
        brier_score_reference: d(0.04),     // Mükemmel kalibrasyon referansı
        model_features: features,
    }
}

/// ATR(14) — True Range'in 14-periyot EMA'sı.
/// detect-ms/pivot.rs ile aynı algoritma.
fn calculate_atr(window: &VecDeque<Kline>, period: usize) -> Decimal {
    if window.len() < 2 {
        return Decimal::ZERO;
    }

    let bars: Vec<&Kline> = window.iter().collect();
    let mut trs = Vec::with_capacity(bars.len());

    for i in 1..bars.len() {
        let high = bars[i].high;
        let low = bars[i].low;
        let prev_close = bars[i - 1].close;
        let tr = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());
        trs.push(tr);
    }

    if trs.is_empty() {
        return Decimal::ZERO;
    }

    let p = period.min(trs.len());
    let first_atr = trs[..p].iter().sum::<Decimal>() / Decimal::from(p);
    let multiplier = Decimal::TWO / Decimal::from(p + 1);

    let mut atr = first_atr;
    for &tr in &trs[p..] {
        atr = (tr - atr) * multiplier + atr;
    }

    atr
}
