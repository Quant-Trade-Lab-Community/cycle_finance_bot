// ============================================================================
// detect-trb — TİP SİSTEMİ
// ============================================================================
// FluidError, FluidResult, tüm çıktı struct'ları.
// unwrap() yasak — tüm iç fonksiyonlar FluidResult<T> döner.
// ============================================================================

use serde::Serialize;

// ================================================================
// HATA YÖNETİMİ
// ================================================================

/// Fluid NS sistemindeki tüm hata türleri.
/// std::panic::catch_unwind yalnızca orchestrator (main.rs) düzeyinde.
#[derive(Debug)]
pub enum FluidError {
    /// Veri kaynağından veri gelmedi (SQLite boş veya ring buffer stale)
    DataStall,
    /// PDE çözücüsü ıraksadı — NaN veya Inf algılandı
    DivergenceExplosion,
    /// SQLite erişim hatası
    DbError(String),
    /// Ring buffer bağlantısı kesildi
    RingBufferDisconnect,
    /// Geçersiz grid boyutu
    InvalidGridDimension,
    /// Sembol bulunamadı
    SymbolNotFound(String),
}

impl std::fmt::Display for FluidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FluidError::DataStall            => write!(f, "DataStall: veri akışı durdu"),
            FluidError::DivergenceExplosion  => write!(f, "DivergenceExplosion: PDE ıraksadı (NaN/Inf)"),
            FluidError::DbError(s)           => write!(f, "DbError: {}", s),
            FluidError::RingBufferDisconnect => write!(f, "RingBufferDisconnect: shm erişim hatası"),
            FluidError::InvalidGridDimension => write!(f, "InvalidGridDimension: geçersiz grid"),
            FluidError::SymbolNotFound(s)    => write!(f, "SymbolNotFound: {}", s),
        }
    }
}

pub type FluidResult<T> = Result<T, FluidError>;

// ================================================================
// VERİ GİRİŞ YAPISI
// ================================================================

/// Core sistemden gelen anlık piyasa akışı.
/// Her zaman adımı için bir `InflowData` üretilir.
#[derive(Debug, Clone)]
pub struct InflowData {
    /// Ağırlıklı ortalama fiyat (trade ağırlıklı)
    pub price: f64,
    /// Toplam işlem hacmi
    pub volume: f64,
    /// Open Interest değişimi (Δ OI)
    pub oi_delta: f64,
    /// Anlık funding oranı (Coriolis kuvveti)
    pub funding_rate: f64,
    /// Alış/satış hacim oranı (taker imbalance)
    pub buy_sell_ratio: f64,
    /// Tasfiye hacmi (kavitasyon girdisi)
    pub liquidation_volume: f64,
    /// Unix timestamp (ms)
    pub timestamp_ms: u64,
}

// ================================================================
// KAVİTASYON — BURST SİNYALİ
// ================================================================

/// Rayleigh-Plesset ODE eşiği aşıldığında üretilen basınç dalgası sinyali.
#[derive(Debug, Clone, Serialize)]
pub struct BurstSignal {
    /// Kabarcık yarıçapının kritik eşiği aşma anı
    pub trigger_price: f64,
    /// Basınç dalgası frekansı (Hz cinsinden normalize)
    pub frequency: f64,
    /// Dalga genliği (0–1 arası normalize)
    pub amplitude: f64,
    /// Tasfiye yönü: "LONG" veya "SHORT"
    pub direction: String,
}

// ================================================================
// KALİBRASYON SONUCU
// ================================================================

/// Nelder-Mead optimizasyonu ile bulunan akışkan parametreleri
#[derive(Debug, Clone, Serialize)]
pub struct CalibrationResult {
    /// Kinematik viskozite ν (optimize edilmiş)
    pub viscosity: f64,
    /// Smagorinsky sabiti Cs (LES türbülans modeli)
    pub smagorinsky_cs: f64,
    /// Kalibrasyon maliyet fonksiyonu değeri
    pub cost: f64,
    /// Optimizasyon iterasyon sayısı
    pub iterations: u32,
}

// ================================================================
// EMİR DİLİMİ (TWAP)
// ================================================================

/// Pontryagin Minimum Prensibi ile üretilen emir dilimi
#[derive(Debug, Clone, Serialize)]
pub struct OrderSlice {
    /// Normalleştirilmiş emir boyutu (0–1)
    pub size: f64,
    /// Referans fiyattan sapma (pozitif = yukarı)
    pub price_offset: f64,
    /// Dilim indeksi (0 = en erken)
    pub index: usize,
}

// ================================================================
// SOLVER DURUMU
// ================================================================

/// NS çözücüsünün mevcut durumu — HTTP yanıtına dahil edilir
#[derive(Debug, Clone, Serialize)]
pub struct SolverState {
    /// Ortalama yoğunluk (fiyat uzayı genelinde)
    pub mean_density: f64,
    /// Maksimum hız büyüklüğü |u|_max
    pub max_velocity: f64,
    /// Ortalama basınç
    pub mean_pressure: f64,
    /// Güncel kinematik viskozite
    pub viscous: f64,
    /// Iraksama kontrolü: divergence normu ∇·u
    pub divergence_norm: f64,
    /// Çözücü kararlı mı?
    pub is_stable: bool,
    /// Tamamlanan solver adım sayısı
    pub steps_completed: usize,
}

// ================================================================
// ANA RAPOR
// ================================================================

/// detect-trb'nin tam çıktısı — tüm katmanları birleştirir
#[derive(Debug, Clone, Serialize)]
pub struct TrbReport {
    /// Sembol
    pub symbol: String,
    /// Zaman aralığı
    pub interval: String,
    /// İşlenen inflow adım sayısı
    pub inflow_steps: usize,

    /// NS çözücü durumu
    pub solver_state: SolverState,

    /// Kavitasyon sinyali (tasfiye şok dalgası)
    pub burst_signal: Option<BurstSignal>,

    /// Kalibrasyon sonuçları
    pub calibration: CalibrationResult,

    /// TWAP emir dilimleri (basınç gradyanından)
    pub twap_curve: Vec<OrderSlice>,

    /// Türkçe naratif özet
    pub narrative: NarrativeOutput,

    /// Analiz meta verisi
    pub audit: AuditMeta,
}

/// Türkçe özet çıktısı
#[derive(Debug, Clone, Serialize)]
pub struct NarrativeOutput {
    pub phase_label: String,
    pub flow_direction: String,
    pub turbulence_level: String,
    pub summary: String,
    pub risk_warning: String,
}

/// Analiz meta verisi
#[derive(Debug, Clone, Serialize)]
pub struct AuditMeta {
    pub analysis_time: String,
    pub grid_nx: usize,
    pub grid_ny: usize,
    pub data_source: String,
    pub calibration_version: String,
}
