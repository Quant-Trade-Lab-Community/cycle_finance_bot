// ============================================================================
// detect-trb — KAVİTASYON MODELİ (Rayleigh-Plesset ODE)
// ============================================================================
// Tasfiyeler = piyasa akışkanındaki kavitasyon kabarcıkları.
// Rayleigh-Plesset ODE:
//   R·R̈ + (3/2)·Ṙ² = (P_v - P_∞) / ρ
//
// Euler-Maruyama ile çözülür (Δt = 1μs).
// Eşik: bubble.radius > 0.7 × OB derinlik oranı → BurstSignal üretilir.
// ============================================================================

use tracing::warn;

use crate::types::{BurstSignal, FluidResult};

// Sıvı yoğunluğu ρ (normalize — gerçek piyasada hacim birimi)
const RHO: f64 = 1000.0;
/// Kritik yarıçap eşiği — bu değeri geçen kabarcık BurstSignal üretir
const CRITICAL_RADIUS: f64 = 0.7;
/// Euler-Maruyama zaman adımı
const DT: f64 = 1e-6;

// ================================================================
// KABARcık YAPISI
// ================================================================

/// Tasfiye kavitasyon kabarcığı
pub struct Bubble {
    /// Normalize kabarcık yarıçapı (OB derinliğine göre)
    pub radius: f64,
    /// Yarıçap değişim hızı dR/dt
    pub wall_velocity: f64,
    /// Marj oranı (yüzey gerilimi surrogat)
    pub surface_tension: f64,
    /// Tasfiye yönü: true = long tasfiye
    pub is_long: bool,
    /// Tetikleme fiyatı
    pub trigger_price: f64,
}

impl Bubble {
    pub fn new(liquidation_volume: f64, ob_depth: f64, price: f64, is_long: bool) -> Self {
        // Başlangıç yarıçapı: tasfiye hacminin OB derinliğine oranı
        let radius = if ob_depth > 0.0 {
            (liquidation_volume / ob_depth).min(1.0)
        } else {
            liquidation_volume * 0.01
        };

        Bubble {
            radius: radius.max(1e-6),
            wall_velocity: 0.0,
            surface_tension: 0.05, // Varsayılan marj oranı
            is_long,
            trigger_price: price,
        }
    }

    /// Rayleigh-Plesset ODE tek adım (Euler-Maruyama)
    ///
    /// R·R̈ + (3/2)·Ṙ² = (P_v - P_∞) / ρ
    ///
    /// P_v: Kabarcık iç basıncı (tasfiye tetikleme fiyatı)
    /// P_∞: Çevre basıncı (güncel piyasa basıncı)
    pub fn step(&mut self, p_inf: f64, p_vapor: f64) -> FluidResult<()> {
        if self.radius <= 1e-9 {
            return Ok(()); // Çökmüş kabarcık
        }

        // R̈ = (P_v - P_∞) / (ρ·R) - (3/2)·Ṙ²/R
        let r_ddot = (p_vapor - p_inf) / (RHO * self.radius)
            - 1.5 * self.wall_velocity.powi(2) / self.radius
            - 2.0 * self.surface_tension / (RHO * self.radius.powi(2));

        if r_ddot.is_nan() || r_ddot.is_infinite() {
            warn!("Rayleigh-Plesset: r_ddot NaN/Inf — kabarcık yeniden başlatılıyor");
            self.radius = 1e-6;
            self.wall_velocity = 0.0;
            return Ok(());
        }

        self.wall_velocity += r_ddot * DT;
        self.radius += self.wall_velocity * DT;

        // Negatif yarıçap fiziksel olarak imkânsız
        if self.radius <= 0.0 {
            self.radius = 1e-9;
            self.wall_velocity = 0.0;
        }

        Ok(())
    }

    /// Kabarcık kritik eşiği geçti mi?
    pub fn is_burst(&self) -> bool {
        self.radius >= CRITICAL_RADIUS
    }

    /// BurstSignal üret — basınç dalgası frekansı ve genliği
    pub fn burst_signal(&self) -> BurstSignal {
        // Frekans: Minnaert formülü yaklaşımı
        // f ≈ (1/2πR)·√(3κP_∞/ρ)  — κ=1.4 (adiabatik), normalize
        let frequency = (1.0 / (2.0 * std::f64::consts::PI * self.radius))
            * (3.0 * 1.4 / RHO).sqrt();

        // Genlik: duvar hızının normalize değeri
        let amplitude = (self.wall_velocity.abs() / (self.wall_velocity.abs() + 1.0)).min(1.0);

        BurstSignal {
            trigger_price: self.trigger_price,
            frequency: frequency.min(1e6), // Cap at 1MHz normalize
            amplitude,
            direction: if self.is_long {
                "LONG".to_string()
            } else {
                "SHORT".to_string()
            },
        }
    }
}

// ================================================================
// KAVİTASYON ANALİZİ
// ================================================================

/// Tüm tasfiye olaylarını değerlendirip en güçlü BurstSignal döner.
///
/// - Her tasfiye olayı için bir Bubble oluşturulur
/// - N Euler-Maruyama adımı çalıştırılır
/// - Eşiği geçen ilk kabarcık BurstSignal üretir
pub fn analyze_cavitation(
    liquidation_volume: f64,
    mean_pressure: f64,
    current_price: f64,
    ob_depth_estimate: f64,
) -> FluidResult<Option<BurstSignal>> {
    if liquidation_volume <= 0.0 {
        return Ok(None);
    }

    // Long ve short tasfiye senaryoları
    let scenarios = [
        (true,  mean_pressure * 1.05), // Long squeeze: p_vapor > p_inf
        (false, mean_pressure * 0.95), // Short squeeze: p_vapor < p_inf
    ];

    let mut strongest: Option<(f64, BurstSignal)> = None;

    for (is_long, p_vapor) in &scenarios {
        let mut bubble = Bubble::new(liquidation_volume, ob_depth_estimate, current_price, *is_long);

        // 1000 Euler-Maruyama adımı (1ms simülasyon)
        for _ in 0..1000 {
            bubble.step(mean_pressure, *p_vapor)?;
            if bubble.is_burst() {
                let sig = bubble.burst_signal();
                let score = sig.amplitude * sig.frequency.log10().max(0.0);
                match &strongest {
                    None => strongest = Some((score, sig)),
                    Some((best_score, _)) if score > *best_score => {
                        strongest = Some((score, sig));
                    }
                    _ => {}
                }
                break;
            }
        }
    }

    Ok(strongest.map(|(_, sig)| sig))
}
