// ============================================================================
// WYCKOFF V4 — KATMAN 1+3: EWMA ANALİZ MOTORU
// ============================================================================
// WyckoffAnalyst: Stateful, her yeni bar geldiğinde güncellenir.
// EWMA decay_factor = 0.85 (tek bar piyasayı sarsmaz).
//
// Anlık faz skoru → EWMA güncelleme → Faz etiketi belirleme
// Bu modül tüm katmanları orkestre eder.
// ============================================================================

use std::collections::VecDeque;

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::{
    events::detect_wyckoff_event,
    narrative::{generate_narrative, suggest_bias},
    probability::calculate_probabilities,
    structure::extract_structure,
    types::{AuditTrail, PhaseWeights, WyckoffInsight},
};

fn d(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap_or(Decimal::ZERO)
}

// ================================================================
// EWMA FAZ AĞIRLIKLARI (İç Yardımcı)
// ================================================================

/// Anlık faz skorları — tek bir bar bazında hesaplanır
struct InstantPhaseScores {
    acc: Decimal,
    markup: Decimal,
    dist: Decimal,
    markdown: Decimal,
}

// ================================================================
// ANA MOTOR: WyckoffAnalyst
// ================================================================

/// EWMA tabanlı stateful Wyckoff analiz motoru.
///
/// ## Kullanım
/// ```ignore
/// let mut analyst = WyckoffAnalyst::new(144);
/// for kline in klines {
///     let _ = analyst.feed(&kline);  // Pencereyi doldur
/// }
/// let insight = analyst.feed(&last_kline);  // Son analiz raporunu al
/// ```
pub struct WyckoffAnalyst {
    window_size: usize,
    decay_factor: Decimal,
    window: VecDeque<Kline>,

    // EWMA durumu (stateful faz ağırlıkları)
    ewma_accumulation: Decimal,
    ewma_markup: Decimal,
    ewma_distribution: Decimal,
    ewma_markdown: Decimal,
}

impl WyckoffAnalyst {
    /// Yeni bir WyckoffAnalyst oluşturur.
    /// `window_size`: EWMA hesabında kullanılacak maksimum bar sayısı (önerilen: 144)
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            decay_factor: d(0.85),
            window: VecDeque::with_capacity(window_size),
            ewma_accumulation: d(0.25),
            ewma_markup: d(0.25),
            ewma_distribution: d(0.25),
            ewma_markdown: d(0.25),
        }
    }

    /// Yeni bir bar besler ve tam analiz raporu döner.
    ///
    /// Her çağrıda:
    /// 1. Pencere güncellenir (eski barlar çıkar)
    /// 2. Anlık faz skorları hesaplanır
    /// 3. EWMA güncellenir
    /// 4. Tüm yapısal ve olasılıksal analizler çalıştırılır
    pub fn feed(&mut self, bar: &Kline) -> WyckoffInsight {
        // ── Pencereyi kaydır ─────────────────────────────────────────────
        self.window.push_back(bar.clone());
        while self.window.len() > self.window_size {
            self.window.pop_front();
        }

        // ── Anlık faz skorları ────────────────────────────────────────────
        let instant = self.calculate_instant_phase(bar);

        // ── EWMA güncelleme ───────────────────────────────────────────────
        let alpha = d(1.0) - self.decay_factor;
        self.ewma_accumulation =
            self.ewma_accumulation * self.decay_factor + instant.acc * alpha;
        self.ewma_markup =
            self.ewma_markup * self.decay_factor + instant.markup * alpha;
        self.ewma_distribution =
            self.ewma_distribution * self.decay_factor + instant.dist * alpha;
        self.ewma_markdown =
            self.ewma_markdown * self.decay_factor + instant.markdown * alpha;

        // ── Faz etiketi ───────────────────────────────────────────────────
        let phase_label = self.determine_phase_label();

        // ── Yapısal konum ─────────────────────────────────────────────────
        let structure = extract_structure(bar, &self.window);

        // ── Olasılık tahmini ──────────────────────────────────────────────
        let probs = calculate_probabilities(bar, &self.window, &structure);

        // ── Wyckoff olayı ─────────────────────────────────────────────────
        let event = detect_wyckoff_event(bar, &self.window);

        // ── Narrative ─────────────────────────────────────────────────────
        let narrative = generate_narrative(&phase_label, &structure, &probs, &event);

        // ── Bias tavsiyesi ────────────────────────────────────────────────
        let bias = suggest_bias(&probs);

        // ── Denetim kaydı ─────────────────────────────────────────────────
        let audit = AuditTrail {
            analysis_time: audit_timestamp(bar.open_time),
            window_bars: self.window_size,
            actual_bars_used: self.window.len(),
            calibration_version: "v4.1.0".to_string(),
        };

        // ── Faz ağırlıkları ───────────────────────────────────────────────
        let phase_weights = PhaseWeights {
            accumulation: self.ewma_accumulation,
            markup: self.ewma_markup,
            distribution: self.ewma_distribution,
            markdown: self.ewma_markdown,
            phase_label,
            decay_factor: self.decay_factor,
        };

        WyckoffInsight {
            phase_distribution: phase_weights,
            structural_position: structure,
            probability_forecast: probs,
            narrative,
            suggested_bias: bias,
            audit_trail: audit,
        }
    }

    // ================================================================
    // ANLIK FAZ SKORLARI
    // ================================================================

    /// Tek bir bar'ın faz skorlarını hesaplar.
    ///
    /// Kural tabanlı mantık:
    /// - Alt bant + yüksek hacim → Accumulation
    /// - Orta-üst bant + yüksek hacim + yükseliş kapanışı → Markup
    /// - Üst bant + düşük hacim → Distribution
    /// - Alt bant + düşüş kapanışı → Markdown
    /// - Diğer → Neutral (range)
    fn calculate_instant_phase(&self, bar: &Kline) -> InstantPhaseScores {
        if self.window.len() < 10 {
            // Yetersiz veri: nötr dağılım
            return InstantPhaseScores {
                acc: d(0.25),
                markup: d(0.25),
                dist: d(0.25),
                markdown: d(0.25),
            };
        }

        let range_high = self.window.iter().map(|b| b.high).fold(Decimal::MIN, Decimal::max);
        let range_low = self.window.iter().map(|b| b.low).fold(Decimal::MAX, Decimal::min);

        // Fiyatın range içindeki oranı (0.0 = en alt, 1.0 = en üst)
        let price_ratio = if range_high > range_low {
            (bar.close - range_low) / (range_high - range_low)
        } else {
            d(0.5)
        };

        // Hacim ortalaması (son 5 bar)
        let vol_5: Vec<Decimal> = self.window.iter().rev().take(5).map(|b| b.volume).collect();
        let avg_vol = if vol_5.is_empty() {
            Decimal::ONE
        } else {
            vol_5.iter().sum::<Decimal>() / Decimal::from(vol_5.len())
        };
        let vol_above_avg = avg_vol > Decimal::ZERO && bar.volume > avg_vol;

        // ── Kural matrisi ─────────────────────────────────────────────────
        if price_ratio < d(0.3) && vol_above_avg {
            // Alt bant + yüksek hacim → Birikim
            InstantPhaseScores {
                acc: d(0.80),
                markup: d(0.10),
                dist: d(0.05),
                markdown: d(0.05),
            }
        } else if price_ratio > d(0.6) && vol_above_avg && bar.close > bar.open {
            // Orta-üst bant + yüksek hacim + yükseliş → Yükseliş Trendi
            InstantPhaseScores {
                acc: d(0.10),
                markup: d(0.75),
                dist: d(0.10),
                markdown: d(0.05),
            }
        } else if price_ratio > d(0.7) && !vol_above_avg {
            // Üst bant + düşük hacim → Dağıtım
            InstantPhaseScores {
                acc: d(0.10),
                markup: d(0.10),
                dist: d(0.70),
                markdown: d(0.10),
            }
        } else if price_ratio < d(0.4) && bar.close < bar.open {
            // Alt bant + ayı kapanışı → Düşüş Trendi
            InstantPhaseScores {
                acc: d(0.10),
                markup: d(0.10),
                dist: d(0.10),
                markdown: d(0.70),
            }
        } else {
            // Nötr / Range ortası
            InstantPhaseScores {
                acc: d(0.40),
                markup: d(0.20),
                dist: d(0.20),
                markdown: d(0.20),
            }
        }
    }

    // ================================================================
    // FAZ ETİKETİ
    // ================================================================

    /// En yüksek EWMA ağırlığını baskın faz olarak etiketler.
    fn determine_phase_label(&self) -> String {
        let max = self
            .ewma_accumulation
            .max(self.ewma_markup)
            .max(self.ewma_distribution)
            .max(self.ewma_markdown);

        if self.ewma_accumulation == max {
            if self.ewma_accumulation > d(0.7) {
                "Güçlü Birikim (Accumulation) — Kış Sonu / Bahar".to_string()
            } else {
                "Erken Birikim (Accumulation)".to_string()
            }
        } else if self.ewma_markup == max {
            "Yükseliş Trendi (Markup) — Yaz Mevsimi".to_string()
        } else if self.ewma_distribution == max {
            "Dağıtım (Distribution) — Sonbahar".to_string()
        } else {
            "Düşüş Trendi (Markdown) — Kış".to_string()
        }
    }
}

/// Bar'ın open_time (millisecond timestamp) değerinden ISO 8601 benzeri
/// basit bir tarih string'i üretir (chrono bağımlılığı olmadan).
fn audit_timestamp(open_time_ms: u64) -> String {
    let secs = open_time_ms / 1000;
    // Basit epoch→tarih dönüşümü (1970-01-01 tabanlı)
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;

    // Yıl/ay/gün hesabı (Gregoryen takvim, yaklaşık)
    let mut year = 1970u64;
    let mut day_count = days;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if day_count < days_in_year {
            break;
        }
        day_count -= days_in_year;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_month = [31u64, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for &dim in &days_in_month {
        if day_count < dim {
            break;
        }
        day_count -= dim;
        month += 1;
    }
    let day = day_count + 1;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, h, m, s)
}
