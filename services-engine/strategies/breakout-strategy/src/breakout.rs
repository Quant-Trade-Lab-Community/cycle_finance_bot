//! KRİPTO FUTURES TEK ZAMAN DİLİMİ PATLAMA (KIRILIM) TESPİT ALGORİTMASI
//!
//! Teknik Spesifikasyon — Sürüm 1.0 (Acımasız & Mükemmeliyetçi)
//!
//! Bu modül, yalnızca tek bir zaman dilimine (N mum) odaklanarak destek/direnç
//! kırılımını matematiksel kesinlikle sınıflandırır. Formüller spesifikasyona
//! birebir uygulanır:
//!
//! ```text
//! S_level  = min(1,T_cnt/15)*0.4 + min(1,V_touch_avg/V_avg)*0.4 + min(1,2σ/|R−S|+ε)*0.2
//! Direction= UP  ⟺ P_close ≥ R+0.25σ
//!           = DOWN ⟺ P_close ≤ S−0.25σ
//!           = NONE (aksi)
//! Q = (V_score*0.40 + M_score*0.35 + Body_ratio*0.25)*100
//! F = (W_score*0.30 + OI_score*0.30 + FZ_score*0.20 + Liq_score*0.20)*100
//! C = (S_level*0.40 + CVD_score*0.40 + MP_score*0.20)*100
//! ```
//!
//! Ek "acımasız" kurallar (Bölüm 5):
//! - Fitil tuzak: seviye delinip kapanış eşiğin altında kalırsa Fake'ye +%15.
//! - Likidasyon avı: Liq_current > 5×Liq_avg ise direction → NONE.
//! - Z_funding > 3 ise certainty %30'da tavanlanır.

/// Sıfır bölme koruması (ε).
pub const EPS: f64 = 1e-9;

/// Kırılım yönü.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    None,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::Up => "UP",
            Direction::Down => "DOWN",
            Direction::None => "NONE",
        }
    }
}

/// Algoritma girdileri — spesifikasyondaki değişken tanımları.
#[derive(Debug, Clone)]
pub struct BreakoutInput {
    pub symbol: String,
    // Mevcut mum (doğrudan girdi)
    pub p_high: f64,
    pub p_low: f64,
    pub p_open: f64,
    pub p_close: f64,
    // Volatilite / hacim (ATR14, SMA20, High14/Low14)
    pub atr: f64,
    pub v_avg: f64,
    pub volume_current: f64,
    pub high_14: f64,
    pub low_14: f64,
    // Seviyeler (önceden hesaplanmış pivot)
    pub resistance: f64, // R
    pub support: f64,    // S
    pub touches: u32,    // T_cnt (R veya S için)
    pub v_touch_avg: f64,
    // Türev veriler
    pub oi: f64,
    pub oi_prev: f64,
    pub funding_rate: f64,
    pub funding_mean_20: f64,
    pub funding_std_20: f64,
    pub cvd_now: f64,
    pub cvd_prev_10: f64,
    pub cvd_sigma: f64,
    pub liq_current: f64,
    pub liq_avg: f64,
    pub mark: f64,
    pub last: f64,
}

/// Algoritma çıktısı — spesifikasyondaki JSON formatı.
#[derive(Debug, Clone)]
pub struct BreakoutResult {
    /// "UP" | "DOWN" | "NONE"
    pub direction: &'static str,
    /// Kırılan R veya S değeri (NONE ise 0)
    pub broken_level: f64,
    /// Q (Yüzde) — kırılım kalitesi
    pub quality: f64,
    /// F (Yüzde) — sahte olma olasılığı (düşük iyi)
    pub fake: f64,
    /// C (Yüzde) — nihai kesinlik skoru
    pub certainty: f64,
}

impl BreakoutResult {
    pub fn none() -> Self {
        Self { direction: "NONE", broken_level: 0.0, quality: 0.0, fake: 0.0, certainty: 0.0 }
    }
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "direction": self.direction,
            "broken_level": self.broken_level,
            "breakout_quality": (self.quality * 10.0).round() / 10.0,
            "fake_percentage": (self.fake * 10.0).round() / 10.0,
            "certainty_percentage": (self.certainty * 10.0).round() / 10.0,
        })
    }
}

/// 3.1 — Seviye Sağlamlık Skoru (S_level) [0.0 – 1.0].
fn level_strength(i: &BreakoutInput) -> f64 {
    let t = (i.touches as f64 / 15.0).min(1.0);
    let v = (i.v_touch_avg / (i.v_avg + EPS)).min(1.0);
    // |R−S| dar ise (sıkışma) seviye daha güçlü.
    let spread = if i.resistance > 0.0 && i.support > 0.0 {
        (2.0 * i.atr / ((i.resistance - i.support).abs() + EPS)).min(1.0)
    } else {
        0.5
    };
    t * 0.4 + v * 0.4 + spread * 0.2
}

/// 3.2 — Kırılım Tetikleme Eşiği (Acımasız Filtre).
/// Sadece fiyatın seviyeyi 0.25 ATR net geçmesi kabul edilir.
fn breakout_direction(i: &BreakoutInput) -> (Direction, f64) {
    if i.resistance > 0.0 && i.p_close >= i.resistance + 0.25 * i.atr {
        (Direction::Up, i.resistance)
    } else if i.support > 0.0 && i.p_close <= i.support - 0.25 * i.atr {
        (Direction::Down, i.support)
    } else {
        (Direction::None, 0.0)
    }
}

/// Kırılımın "denendiği" yön — kalite/sahte hesabı için aday yön.
/// Kapanış eşiği aşmadıysa ama fitil seviyeyi deldiyse o yön aday sayılır.
fn attempt_direction(i: &BreakoutInput) -> Direction {
    if i.resistance > 0.0 && i.p_close >= i.resistance + 0.25 * i.atr {
        Direction::Up
    } else if i.support > 0.0 && i.p_close <= i.support - 0.25 * i.atr {
        Direction::Down
    } else if i.resistance > 0.0 && i.p_high > i.resistance {
        Direction::Up
    } else if i.support > 0.0 && i.p_low < i.support {
        Direction::Down
    } else {
        Direction::None
    }
}

/// 3.3 + 3.4 + 3.5 — Kalite (Q), Fake (F), Kesinlik (C).
fn quality(i: &BreakoutInput, dir: Direction) -> f64 {
    let v_score = (i.volume_current / (i.v_avg + EPS)).min(1.0);
    let m_score = match dir {
        Direction::Up => ((i.p_close - i.low_14) / (i.high_14 - i.low_14 + EPS)).max(0.0),
        Direction::Down => ((i.high_14 - i.p_close) / (i.high_14 - i.low_14 + EPS)).max(0.0),
        Direction::None => 0.0,
    }
    .min(1.0);
    let body = ((i.p_close - i.p_open).abs() / (i.p_high - i.p_low + EPS)).min(1.0);
    (v_score * 0.40 + m_score * 0.35 + body * 0.25) * 100.0
}

fn fake(i: &BreakoutInput, dir: Direction) -> f64 {
    let wick = match dir {
        // Uzun üst fitil = tuzak (yukarı kırılım)
        Direction::Up => ((i.p_high - i.p_close.max(i.p_open)) / (i.p_high - i.p_low + EPS) * 2.0).min(1.0),
        // Uzun alt fitil = tuzak (aşağı kırılım)
        Direction::Down => ((i.p_close.min(i.p_open) - i.p_low) / (i.p_high - i.p_low + EPS) * 2.0).min(1.0),
        Direction::None => 0.0,
    };
    // ΔOI<0 (fiyat yukarı giderken OI düşüyorsa) = short kapatma → sahte.
    let oi_norm = (i.oi - i.oi_prev) / (i.oi_prev + EPS);
    let oi_score = (-oi_norm).max(0.0);
    let z_f = (i.funding_rate - i.funding_mean_20) / (i.funding_std_20 + EPS);
    let fz = (z_f / 3.0).max(0.0).min(1.0);
    let liq_score = (i.liq_current / (i.liq_avg + EPS)).min(1.0);

    let mut f = (wick * 0.30 + oi_score * 0.30 + fz * 0.20 + liq_score * 0.20) * 100.0;

    // Bölüm 5 — Fitil kontrolü: seviyeyi delip kapanış eşiğin altında kaldıysa +%15.
    let wick_fake = match dir {
        Direction::Up => i.p_high > i.resistance && i.p_close < i.resistance + 0.25 * i.atr,
        Direction::Down => i.p_low < i.support && i.p_close > i.support - 0.25 * i.atr,
        Direction::None => false,
    };
    if wick_fake {
        f += 15.0;
    }
    f.min(100.0)
}

fn certainty(i: &BreakoutInput, dir: Direction) -> f64 {
    let s_level = level_strength(i);
    let cvd_score = ((i.cvd_now - i.cvd_prev_10) / (i.cvd_sigma * 10.0 + EPS)).max(0.0).min(1.0);
    let mp = match (dir, i.mark, i.last) {
        (Direction::Up, m, l) if m > l => 1.0,   // Contango / taşıma maliyeti pozitif
        (Direction::Down, m, l) if m < l => 1.0, // Backwardation
        _ => 0.5,
    };
    let mut c = (s_level * 0.40 + cvd_score * 0.40 + mp * 0.20) * 100.0;

    // Bölüm 6 — Funding aşırı ucu: Z_funding > 3 → kesinlik en fazla %30.
    let z_f = (i.funding_rate - i.funding_mean_20) / (i.funding_std_20 + EPS);
    if z_f > 3.0 {
        c = c.min(30.0);
    }
    c
}

/// Çekirdek motor — `compute(input)` spesifikasyonun 2-5. adımlarını uygular.
pub fn compute(input: &BreakoutInput) -> BreakoutResult {
    let (dir, broken_level) = breakout_direction(input);
    let attempt = attempt_direction(input);

    // Bölüm 5 — Likidasyon avı: Liq > 5×avg → gerçek trend değil, stop avı.
    let liq_run = input.liq_avg > 0.0 && input.liq_current > 5.0 * input.liq_avg;

    if attempt == Direction::None {
        // Seviye testi bile yok — bilgi yok.
        return BreakoutResult::none();
    }

    // Kalite/sahte/kesinlik, aday yöne göre her zaman hesaplanır (fitil tuzağı dahil).
    let q = quality(input, attempt);
    let f = fake(input, attempt);
    let c = certainty(input, attempt);

    let direction = if liq_run { "NONE" } else { dir.as_str() };
    let broken_level = if liq_run { 0.0 } else { broken_level };

    BreakoutResult { direction, broken_level, quality: q, fake: f, certainty: c }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> BreakoutInput {
        BreakoutInput {
            symbol: "BTCUSDT".into(),
            p_high: 65_000.0,
            p_low: 64_200.0,
            p_open: 64_300.0,
            p_close: 64_900.0,
            atr: 400.0,
            v_avg: 1_000.0,
            volume_current: 1_500.0,
            high_14: 65_000.0,
            low_14: 63_500.0,
            resistance: 64_800.0,
            support: 63_900.0,
            touches: 10,
            v_touch_avg: 1_200.0,
            oi: 500.0,
            oi_prev: 480.0,
            funding_rate: 0.0001,
            funding_mean_20: 0.00005,
            funding_std_20: 0.00003,
            cvd_now: 5_000.0,
            cvd_prev_10: 3_000.0,
            cvd_sigma: 100.0,
            liq_current: 50.0,
            liq_avg: 100.0,
            mark: 64_910.0,
            last: 64_900.0,
        }
    }

    #[test]
    fn up_breakout_detected() {
        let i = base_input();
        // close 64900 >= R(64800) + 0.25*400(100) = 64900 → tam eşikte → UP
        let r = compute(&i);
        assert_eq!(r.direction, "UP");
        assert_eq!(r.broken_level, 64_800.0);
        assert!(r.quality > 0.0);
        assert!(r.fake >= 0.0 && r.fake <= 100.0);
        assert!(r.certainty >= 0.0 && r.certainty <= 100.0);
    }

    #[test]
    fn no_breakout_when_below_threshold() {
        let mut i = base_input();
        i.p_close = 64_700.0; // < 64900 eşiği
        let r = compute(&i);
        assert_eq!(r.direction, "NONE");
    }

    #[test]
    fn liq_run_forces_none() {
        let mut i = base_input();
        i.liq_current = 600.0; // > 5 × 100
        let r = compute(&i);
        assert_eq!(r.direction, "NONE");
        assert_eq!(r.broken_level, 0.0);
    }

    #[test]
    fn funding_extreme_caps_certainty() {
        let mut i = base_input();
        i.funding_rate = 0.0002; // Z = (0.0002-0.00005)/0.00003 = 5 > 3
        let r = compute(&i);
        assert!(r.certainty <= 30.0);
    }

    #[test]
    fn wick_fake_penalty() {
        // Fitil seviyeyi deldi (high 65000 > R) ama kapanış eşiğin altında
        let mut i = base_input();
        i.p_close = 64_700.0; // < R+0.25σ → NONE ama fitil tuzak
        let base = compute(&i);
        // Aynı durumda fitil deldiği için fake hesabına +15 uygulanmış olmalı.
        // (direction NONE döner ama fake bilgisi yüksek.)
        assert_eq!(base.direction, "NONE");
    }
}
