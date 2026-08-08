// ============================================================================
// detect-trb — ANLATI (Türkçe özet + denetim meta)
// ============================================================================
// Solver durumu + kavitasyon + kalibrasyon → insan okur Türkçe özet
// ve TrbReport.audit meta bilgisi üretir.
// ============================================================================

use crate::grid::{NX, NY};
use crate::types::{
    AuditMeta, BurstSignal, CalibrationResult, NarrativeOutput, SolverState,
};

/// Üst-düzey faz etiketi
pub fn phase_label(state: &SolverState, burst: Option<&BurstSignal>) -> String {
    if burst.is_some() {
        return "Kavitasyon Dalgası".to_string();
    }
    if !state.is_stable {
        return "Iraksama / Dengesiz".to_string();
    }
    match state.mean_density {
        d if d > 2.0 => "Yoğunlaşma".to_string(),
        d if d < 0.05 => "Seyreltme".to_string(),
        _ => "Kararlı Akış".to_string(),
    }
}

/// Akış yönü etiketi (mean yönünden)
pub fn flow_direction(state: &SolverState) -> String {
    if state.max_velocity > 1e-9 {
        if state.mean_pressure > 0.0 {
            "Yukarı Akış".to_string()
        } else {
            "Aşağı Akış".to_string()
        }
    } else {
        "Yatay (Durağan)".to_string()
    }
}

/// Türbülans seviyesi — max_velocity eşikleri
pub fn turbulence_level(state: &SolverState) -> String {
    match state.max_velocity {
        v if !v.is_finite() => "Belirsiz".to_string(),
        v if v > 1.0 => "Yüksek".to_string(),
        v if v > 0.1 => "Orta".to_string(),
        _ => "Düşük".to_string(),
    }
}

/// Türkçe naratif + audit meta — TrbReport için
pub fn narrate(
    state: &SolverState,
    calibration: &CalibrationResult,
    burst: Option<&BurstSignal>,
    _symbol: &str,
) -> NarrativeOutput {
    let phase = phase_label(state, burst);
    let flow = flow_direction(state);
    let turb = turbulence_level(state);

    let mut summary = format!(
        "{} altında {} mevcut; ortalama basınç {:.2}, viskozite {:.4}. ",
        phase, flow, state.mean_pressure, calibration.viscosity
    );
    if let Some(b) = &burst {
        summary.push_str(&format!(
            "Tasfiye kavitasyonu tespit edildi ({} yönü, frekans {:.0} Hz, genlik {:.2}).",
            b.direction, b.frequency, b.amplitude
        ));
    } else {
        summary.push_str("Aktif kavitasyon sinyali yok — tasfiye baskısı düşük.");
    }

    let risk_warning = if burst.is_some() {
        "Tasfiye şok dalgası algılandı — pozisyon boyutlamada temkinli olun, likidite riski yüksek.".to_string()
    } else if !state.is_stable {
        "Çözücü kararsız (divergence yüksek) — sinyal güvenilirliği düşük.".to_string()
    } else {
        "Standart risk: NS modeli gerçek piyasa koşullarının yaklaşımıdır, yatırım tavsiyesi değildir.".to_string()
    };

    NarrativeOutput {
        phase_label: phase,
        flow_direction: flow,
        turbulence_level: turb,
        summary,
        risk_warning,
    }
}

/// Audit meta — ne zaman/hangi grid/hangi kaynak
pub fn audit_meta(_symbol: &str, data_source: &str) -> AuditMeta {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "n/a".to_string());

    AuditMeta {
        analysis_time: now_ms,
        grid_nx: NX,
        grid_ny: NY,
        data_source: data_source.to_string(),
        calibration_version: "v1.0.0".to_string(),
    }
}
