// ============================================================================
// WYCKOFF V4 — TİP SİSTEMİ (Tüm Output Struct'ları)
// ============================================================================
// Bu modül sadece veri yapılarını tanımlar. İş mantığı ayrı modüllerdedir.
// Sistemin standardına uygun: rust_decimal::Decimal kullanılır (f64 değil).
// ============================================================================

use rust_decimal::Decimal;
use serde::Serialize;

// ================================================================
// ANA ÇIKTI: WyckoffInsight
// ================================================================

/// Wyckoff analizinin tam çıktısı — tüm katmanları birleştirir
#[derive(Debug, Clone, Serialize)]
pub struct WyckoffInsight {
    /// Wyckoff faz dağılımı (EWMA ile yumuşatılmış)
    pub phase_distribution: PhaseWeights,
    /// Yapısal konum (POC, bant, spread)
    pub structural_position: StructuralPosition,
    /// Olasılık tahminleri (breakout/breakdown/range)
    pub probability_forecast: ProbabilityForecast,
    /// Doğal dil özeti ve risk uyarısı
    pub narrative: NarrativeInsight,
    /// Tavsiye edilen yön
    pub suggested_bias: Bias,
    /// Denetim kaydı (meta veri)
    pub audit_trail: AuditTrail,
}

// ================================================================
// FAZ AĞIRLIKLARI (EWMA Tabanlı)
// ================================================================

/// Wyckoff faz ağırlıkları — EWMA ile güncellenir
#[derive(Debug, Clone, Serialize)]
pub struct PhaseWeights {
    /// Birikim fazı ağırlığı (0.0 – 1.0)
    pub accumulation: Decimal,
    /// Yükseliş trendi fazı ağırlığı (0.0 – 1.0)
    pub markup: Decimal,
    /// Dağıtım fazı ağırlığı (0.0 – 1.0)
    pub distribution: Decimal,
    /// Düşüş trendi fazı ağırlığı (0.0 – 1.0)
    pub markdown: Decimal,
    /// İnsan okunabilir faz etiketi (Türkçe)
    pub phase_label: String,
    /// EWMA çarpanı (sabit: 0.85)
    pub decay_factor: Decimal,
}

// ================================================================
// YAPISAL KONUM
// ================================================================

/// Fiyatın range içindeki yapısal konumu
#[derive(Debug, Clone, Serialize)]
pub struct StructuralPosition {
    /// Fiyatın bant konumu (Üst/Orta/Alt)
    pub price_zone: String,
    /// POC'a uzaklık (yüzde)
    pub poc_distance: Decimal,
    /// Volume trendi açıklaması
    pub volume_trend: String,
    /// Spread durumu (daralıyor/genişliyor/normal)
    pub spread_status: String,
    /// Boğa tarafında iptal fiyatı (range_high × 1.015)
    pub invalidation_upper: Decimal,
    /// Ayı tarafında iptal fiyatı (range_low × 0.985)
    pub invalidation_lower: Decimal,
    /// Point of Control fiyatı (en yüksek hacimli seviye)
    pub poc_price: Decimal,
    /// Range yüksek seviyesi
    pub range_high: Decimal,
    /// Range düşük seviyesi
    pub range_low: Decimal,
}

// ================================================================
// OLASILIK TAHMİNLERİ
// ================================================================

/// Feature tabanlı olasılık tahminleri
#[derive(Debug, Clone, Serialize)]
pub struct ProbabilityForecast {
    /// Yukarı kırılma olasılığı (0.0 – 1.0)
    pub breakout_upper: Decimal,
    /// Aşağı kırılma olasılığı (0.0 – 1.0)
    pub breakdown_lower: Decimal,
    /// Range devamı olasılığı (0.0 – 1.0)
    pub range_continuation: Decimal,
    /// Volatilite riski (ATR / Fiyat × 100)
    pub volatility_risk: Decimal,
    /// Sahte kırılma riski (0.0 – 1.0)
    pub fake_break_risk: Decimal,
    /// Momentum riski (0.0 – 1.0)
    pub momentum_risk: Decimal,
    /// Önerilen pozisyon büyüklüğü çarpanı (0.1 – 1.0)
    pub suggested_position_size_factor: Decimal,
    /// Güven aralığı (±%)
    pub confidence_interval: Decimal,
    /// Brier Score referansı (kalibrasyon ölçütü)
    pub brier_score_reference: Decimal,
    /// Modelin kullandığı özellikler
    pub model_features: Vec<String>,
}

// ================================================================
// WYCKOFF OLAYLARI
// ================================================================

/// Wyckoff teorisindeki olaylar
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum WyckoffEvent {
    /// Tuzak kırılması — Range dibini test edip güçlü kapanış
    Spring,
    /// Sign of Strength — Yüksek hacimle yukarı kırılım
    SignOfStrength,
    /// Upthrust — Üst bantı test edip geri çekilme (sahte yükseliş)
    Upthrust,
    /// Sinyal yok / Nötr range
    Neutral,
}

impl WyckoffEvent {
    pub fn label(&self) -> &'static str {
        match self {
            WyckoffEvent::Spring => "Spring (Bahar) — Tuzak kırılması, güçlü alım",
            WyckoffEvent::SignOfStrength => "SOS (Güç Gösterisi) — Yüksek hacimli yükseliş",
            WyckoffEvent::Upthrust => "UT (Upthrust) — Sahte yükseliş, satış baskısı",
            WyckoffEvent::Neutral => "Sinyal yok (Nötr range)",
        }
    }
}

// ================================================================
// NARATİF ÇIKTI
// ================================================================

/// Doğal dil özeti ve risk açıklamaları
#[derive(Debug, Clone, Serialize)]
pub struct NarrativeInsight {
    /// Türkçe piyasa özeti
    pub summary: String,
    /// Tespit edilen Wyckoff olayı açıklaması
    pub wyckoff_event_detected: String,
    /// Risk uyarısı ve iptal seviyeleri
    pub risk_warning: String,
}

// ================================================================
// YÖN TAVSİYESİ
// ================================================================

/// Analizin önerdiği işlem yönü
#[derive(Debug, Clone, Serialize)]
pub enum Bias {
    /// Boğa eğilimi (yükseliş beklentisi)
    Bullish,
    /// Ayı eğilimi (düşüş beklentisi)
    Bearish,
    /// Nötr (yön belirsiz)
    Neutral,
}

// ================================================================
// DENETİM KAYDI
// ================================================================

/// Analiz meta verisi — loglama ve kalibrasyon için
#[derive(Debug, Clone, Serialize)]
pub struct AuditTrail {
    /// Analiz zamanı (ISO 8601 string)
    pub analysis_time: String,
    /// Penceredeki bar sayısı
    pub window_bars: usize,
    /// Kullanılan penceredeki gerçek bar sayısı
    pub actual_bars_used: usize,
    /// Kalibrasyon versiyonu
    pub calibration_version: String,
}
