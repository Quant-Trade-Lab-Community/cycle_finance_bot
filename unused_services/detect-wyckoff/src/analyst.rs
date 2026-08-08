// ============================================================================
// WyckoffAnalyst — v4.1.4 (EWMA Faz Motoru + Yapısal + Olasılık + Naratif)
// detect-wyckoff REST servisinin tek çağrılık analiz boru hattı.
// ============================================================================

use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;

use crate::audit::AuditRecord;
use crate::execution::{ExecutionBroker, ExecutionPlan};
use crate::models::{AssetDefinition, Bar, Bias, Tick, Volume};
use crate::profile::{IncrementalVolumeProfile, VolumeProfileSnapshot};
use crate::risk::{AdaptiveRiskEngine, RiskAction, RiskRecord};
use crate::scorer::ContextualScorer;
use crate::state::{ProbabilisticState, Signal, SignalStats, WyckoffStateMachine};
use ohlcv_engine::Kline;

pub const CALIBRATION_VERSION: &str = "v4.1.4";

#[derive(Debug, Clone, Serialize)]
pub struct PhaseWeights {
    pub accumulation: f64,
    pub markup: f64,
    pub distribution: f64,
    pub markdown: f64,
    pub phase_label: String,
    pub decay_factor: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructuralPosition {
    pub price_zone: String,
    pub poc_distance_pct: f64,
    pub volume_trend: String,
    pub spread_status: String,
    pub invalidation_upper: f64,
    pub invalidation_lower: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbabilityForecast {
    pub breakout_upper: f64,
    pub breakdown_lower: f64,
    pub range_continuation: f64,
    pub volatility_risk_pct: f64,
    pub fake_break_risk: f64,
    pub momentum_risk: f64,
    pub suggested_position_size_factor: f64,
    pub confidence_interval: f64,
    pub brier_score_reference: f64,
    pub model_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NarrativeInsight {
    pub summary: String,
    pub wyckoff_event_detected: String,
    pub risk_warning: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventRecord {
    pub event_type: &'static str,
    pub price: f64,
    pub score: f64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalRecord {
    pub side: &'static str,
    pub entry: f64,
    pub confidence: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Insight {
    pub calibration_version: String,
    pub phase_distribution: PhaseWeights,
    pub structural_position: StructuralPosition,
    pub probability_forecast: ProbabilityForecast,
    pub wyckoff_events: Vec<EventRecord>,
    pub signals: Vec<SignalRecord>,
    pub state: ProbabilisticState,
    pub stats: SignalStats,
    pub volume_profile: VolumeProfileSnapshot,
    pub risk: RiskRecord,
    pub narrative: NarrativeInsight,
    pub suggested_bias: Bias,
    pub execution_plan: Option<ExecutionPlan>,
    pub audit_trail: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub window: usize,
    pub max_risk_bp: i64,
    pub tick_size: f64,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            window: 144,
            max_risk_bp: 200,
            tick_size: 1e-6,
        }
    }
}

/// Kline → Tick tabanlı Bar (taşma kontrollü, tick_size = 1e-6).
fn tick(v: f64) -> Tick {
    Tick((v / 1e-6).round() as i64)
}

fn tick_price(t: Tick, tick_size: f64) -> f64 {
    t.0 as f64 * tick_size
}

impl From<&Kline> for Bar {
    fn from(k: &Kline) -> Self {
        let f = |v: rust_decimal::Decimal| v.to_f64().unwrap_or(0.0);
        Bar {
            timestamp: k.open_time as i64,
            high: tick(f(k.high)),
            low: tick(f(k.low)),
            open: tick(f(k.open)),
            close: tick(f(k.close)),
            volume: Volume(f(k.volume).max(0.0) as u64),
        }
    }
}

fn signal_entry_price(s: &Signal, tick_size: f64) -> f64 {
    match s {
        Signal::Long { entry, .. } | Signal::Short { entry, .. } => entry.0 as f64 * tick_size,
    }
}

fn signal_confidence(s: &Signal) -> f64 {
    match s {
        Signal::Long { confidence, .. } | Signal::Short { confidence, .. } => *confidence,
    }
}

/// Ana giriş: kline listesi → Insight.
pub fn analyze(klines: &[Kline], cfg: &AnalysisConfig) -> Result<Insight, String> {
    if klines.is_empty() {
        return Err("Veri yok".into());
    }

    let asset = AssetDefinition::default_asset();
    let bars: Vec<Bar> = klines
        .iter()
        .map(Bar::from)
        .filter(|b| b.spread_ticks() >= asset.min_move)
        .collect();
    if bars.is_empty() {
        return Err("min_move filtrelemesinden sonra bar kalmadı".into());
    }

    let tick_size = cfg.tick_size;
    let current_price = tick_price(bars.last().unwrap().close, tick_size);

    // ── Bağlam (tüm pencere) ─────────────────────────────────────────────
    let scorer = ContextualScorer::build(&bars);
    let range_low = bars.iter().map(|b| b.low.0).min().unwrap_or(0);
    let avg_volume = bars.iter().map(|b| b.volume.0).sum::<u64>() / bars.len() as u64;

    // ── Boru hattı: profil + durum makinesi + risk ───────────────────────
    let mut machine = WyckoffStateMachine::new();
    let mut profile = IncrementalVolumeProfile::with_decay(0.999);
    let mut risk = AdaptiveRiskEngine::new(
        cfg.max_risk_bp,
        Tick(range_low),
        avg_volume,
        bars.last().unwrap().close,
    );

    let mut signals: Vec<SignalRecord> = Vec::new();
    let mut events: HashMap<&'static str, EventRecord> = HashMap::new();
    let mut audit: Vec<serde_json::Value> = Vec::new();
    let mut last_action = RiskAction::Idle;

    for bar in &bars {
        profile.update(bar);
        let sig = machine.ingest(bar, &scorer);

        if let Some(s) = sig {
            signals.push(SignalRecord {
                side: s.label(),
                entry: signal_entry_price(&s, tick_size),
                confidence: (signal_confidence(&s) * 10000.0).round() / 10000.0,
                timestamp: bar.timestamp,
            });
            while signals.len() > 20 {
                signals.remove(0);
            }
        }

        for (ev, score) in &machine.scored_events {
            let e = events.entry(ev.label()).or_insert(EventRecord {
                event_type: ev.label(),
                price: current_bar_price(bar, tick_size),
                score: 0.0,
                count: 0,
            });
            e.count += 1;
            e.price = tick_price(bar.close, tick_size);
            e.score = (*score * 10000.0).round() / 10000.0;
        }

        last_action = risk.evaluate(bar, &machine.state);

        let top = machine.scored_events.first().cloned();
        audit.push(AuditRecord::decision(
            bar,
            top.as_ref().map(|(_, s)| *s).unwrap_or(0.0),
            top.as_ref().map(|(e, _)| e.label()).unwrap_or("NONE"),
            &machine.state,
            Bias::Neutral,
            sig.as_ref(),
            tick_size,
        ));
        while audit.len() > 16 {
            audit.remove(0);
        }
    }

    machine.state.trend_strength = scorer.trend_angle;
    let structure = structural_position(&bars, &profile, current_price, tick_size, &scorer);
    let probs = probability_forecast(&bars, &structure, current_price, tick_size);
    let bias = suggested_bias(&machine, &scorer, &probs);

    audit.push(AuditRecord::decision(
        bars.last().unwrap(),
        1.0,
        "FINAL",
        &machine.state,
        bias,
        None,
        tick_size,
    ));

    // ── v4 Fazcı: EWMA faz ağırlıkları ───────────────────────────────────
    let phases = ewma_phase_weights(&bars, cfg.window);

    let mut wyckoff_events: Vec<EventRecord> = events.into_values().collect();
    wyckoff_events.sort_by_key(|b| std::cmp::Reverse(b.count));

    let last_event_label = machine
        .scored_events
        .first()
        .map(|(e, _)| e.label())
        .unwrap_or("Nötr range");

    let risk_record = risk.record(last_action, tick_size);

    let narrative = NarrativeInsight {
        summary: format!(
            "📊 Piyasa Durumu: {}. Fiyat {} konumunda. {} Yukarı kırılma %{:.0}, aşağı kırılma %{:.0}.",
            phases.phase_label,
            structure.price_zone,
            structure.volume_trend,
            probs.breakout_upper * 100.0,
            probs.breakdown_lower * 100.0
        ),
        wyckoff_event_detected: format!(
            "🔍 Tespit Edilen Wyckoff Olayı: {}",
            last_event_label
        ),
        risk_warning: format!(
            "⚠️ Sahte kırılma riski %{:.0}. Volatilite riski %{:.0}. İptal (Stop): Üst {} / Alt {}",
            probs.fake_break_risk * 100.0,
            probs.volatility_risk_pct,
            structure.invalidation_upper,
            structure.invalidation_lower
        ),
    };

    // ── Yürütme planı (varsa) ─────────────────────────────────────────────
    let execution_plan = signals.last().map(|s| {
        let broker = ExecutionBroker::new();
        let sig = if s.side == "LONG" {
            Signal::Long { entry: tick(s.entry), confidence: s.confidence }
        } else {
            Signal::Short { entry: tick(s.entry), confidence: s.confidence }
        };
        let orders = broker.execute(&sig, 100_000, tick_size);
        broker.plan(&orders, 100_000)
    });

    Ok(Insight {
        calibration_version: CALIBRATION_VERSION.into(),
        phase_distribution: phases,
        structural_position: structure,
        probability_forecast: probs,
        wyckoff_events,
        signals,
        state: machine.state,
        stats: machine.stats,
        volume_profile: profile.snapshot(tick_size, 5),
        risk: risk_record,
        narrative,
        suggested_bias: bias,
        execution_plan,
        audit_trail: audit,
    })
}

fn current_bar_price(bar: &Bar, tick_size: f64) -> f64 {
    tick_price(bar.close, tick_size)
}

#[derive(Debug, Clone, Copy)]
struct InstantScores {
    acc: f64,
    markup: f64,
    dist: f64,
    markdown: f64,
}

impl InstantScores {
    fn neutral() -> Self {
        Self { acc: 0.25, markup: 0.25, dist: 0.25, markdown: 0.25 }
    }
}

/// EWMA faz ağırlıkları — v4 algoritması (decay 0.85).
///
/// Kural tabanı: price_ratio, hacim yüksekliği, mum rengi.
fn ewma_phase_weights(bars: &[Bar], window: usize) -> PhaseWeights {
    let decay = 0.85;
    let mut acc = 0.0;
    let mut markup = 0.0;
    let mut dist = 0.0;
    let mut markdown = 0.0;

    for (i, bar) in bars.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let lo = i.saturating_sub(window).max(1);
        let win = &bars[lo..=i];
        let inst = instant_scores(win, bar);
        acc = acc * decay + inst.acc * (1.0 - decay);
        markup = markup * decay + inst.markup * (1.0 - decay);
        dist = dist * decay + inst.dist * (1.0 - decay);
        markdown = markdown * decay + inst.markdown * (1.0 - decay);
    }

    PhaseWeights {
        accumulation: acc,
        markup,
        distribution: dist,
        markdown,
        phase_label: phase_label(acc, markup, dist, markdown),
        decay_factor: decay,
    }
}

/// v4 kural tabanı: fiyat oranı + hacim + mum rengi → anlık faz skorları.
fn instant_scores(win: &[Bar], bar: &Bar) -> InstantScores {
    if win.len() < 10 {
        return InstantScores::neutral();
    }
    let rng_high = win.iter().map(|b| b.high.0 as f64).fold(f64::NEG_INFINITY, f64::max);
    let rng_low = win.iter().map(|b| b.low.0 as f64).fold(f64::INFINITY, f64::min);
    let ratio = if rng_high > rng_low {
        ((bar.close.0 as f64 - rng_low) / (rng_high - rng_low)).clamp(0.0, 1.0)
    } else {
        0.5
    };

    let cnt = 5.min(win.len());
    let avg_vol = win[win.len() - cnt..].iter().map(|b| b.volume.0).sum::<u64>() as f64 / cnt as f64;
    let vol_high = bar.volume.0 as f64 > avg_vol;

    if ratio < 0.3 && vol_high {
        InstantScores { acc: 0.8, markup: 0.1, dist: 0.05, markdown: 0.05 }
    } else if ratio > 0.6 && vol_high && bar.close.0 > bar.open.0 {
        InstantScores { acc: 0.1, markup: 0.75, dist: 0.1, markdown: 0.05 }
    } else if ratio > 0.7 && !vol_high {
        InstantScores { acc: 0.1, markup: 0.1, dist: 0.7, markdown: 0.1 }
    } else if ratio < 0.4 && bar.close.0 < bar.open.0 {
        InstantScores { acc: 0.1, markup: 0.1, dist: 0.1, markdown: 0.7 }
    } else {
        InstantScores { acc: 0.4, markup: 0.2, dist: 0.2, markdown: 0.2 }
    }
}

fn phase_label(acc: f64, markup: f64, dist: f64, markdown: f64) -> String {
    let max = acc.max(markup).max(dist).max(markdown);
    if acc == max {
        if acc > 0.7 {
            "Güçlü Birikim (Accumulation) - Kış Sonu / Bahar".into()
        } else {
            "Erken Birikim (Accumulation)".into()
        }
    } else if markup == max {
        "Yükseliş Trendi (Markup) - Yaz Mevsimi".into()
    } else if dist == max {
        "Dağıtım (Distribution) - Sonbahar".into()
    } else {
        "Düşüş Trendi (Markdown) - Kış".into()
    }
}

/// Yapısal konum: POC mesafesi, hacim trendi, spread durumu, iptal seviyeleri.
fn structural_position(
    bars: &[Bar],
    profile: &IncrementalVolumeProfile,
    current_price: f64,
    tick_size: f64,
    scorer: &ContextualScorer,
) -> StructuralPosition {
    let rng_high = scorer.range_high.0 as f64;
    let rng_low = scorer.range_low.0 as f64;
    let poc = profile.poc().0 as f64 * tick_size;
    let poc_distance_pct = if poc > 0.0 { ((current_price - poc) / poc) * 100.0 } else { 0.0 };

    let n = 5.min(bars.len());
    let avg_vol: f64 = bars[bars.len() - n..].iter().map(|b| b.volume.0 as f64).sum::<f64>() / n as f64;
    let last_bar = bars.last().unwrap();
    let vol_trend = if last_bar.volume.0 as f64 > avg_vol * 1.2 {
        "Artan Hacim (Aktif Katılım)".to_string()
    } else if (last_bar.volume.0 as f64) < avg_vol * 0.8 {
        "Azalan Hacim (İlgisizlik / Tuzak)".to_string()
    } else {
        "Yatay Hacim (Normal)".to_string()
    };

    let m = 10.min(bars.len());
    let avg_spread: f64 = bars[bars.len() - m..].iter().map(|b| (b.high.0 - b.low.0) as f64).sum::<f64>() / m as f64;
    let spread = (last_bar.close.0 - last_bar.open.0).abs() as f64;
    let spread_status = if spread < avg_spread * 0.8 {
        "Daralıyor (Sıkışma - Kırılım Yakın)".to_string()
    } else if spread > avg_spread * 1.2 {
        "Genişliyor (Oynaklık Artıyor)".to_string()
    } else {
        "Normal Aralık".to_string()
    };

    let price_zone = if current_price > rng_high * 0.95 {
        "Range'in Üst Bantı (Direnişe Yakın)".to_string()
    } else if current_price < rng_low * 1.05 {
        "Range'in Alt Bantı (Desteğe Yakın)".to_string()
    } else {
        "Range'in Orta Bantı (Kararsız)".to_string()
    };

    StructuralPosition {
        price_zone,
        poc_distance_pct,
        volume_trend: vol_trend,
        spread_status,
        invalidation_upper: rng_high * 1.015 * tick_size,
        invalidation_lower: rng_low * 0.985 * tick_size,
    }
}

/// Olasılık tahmini — v4 formüllerinin tam karşılığı.
#[allow(clippy::too_many_arguments)]
fn probability_forecast(
    bars: &[Bar],
    structure: &StructuralPosition,
    current_price: f64,
    tick_size: f64,
) -> ProbabilityForecast {
    let poc_factor = (structure.poc_distance_pct / 100.0 + 1.0).clamp(0.0, 1.0);
    let mut breakout_upper = 0.50 + poc_factor * 0.40;
    if structure.spread_status.contains("Daralıyor") {
        breakout_upper += 0.10;
    }
    breakout_upper = breakout_upper.clamp(0.0, 0.98);

    let mut breakdown_lower = 0.10 + (1.0 - poc_factor) * 0.30;
    if structure.volume_trend.contains("Azalan") && structure.price_zone.contains("Üst Bant") {
        breakdown_lower += 0.15; // sahte yukarı hareket riski
    }
    breakdown_lower = breakdown_lower.clamp(0.0, 0.98);

    let range_continuation = (1.0 - breakout_upper - breakdown_lower).max(0.05);

    let mut atr_sum = 0.0;
    let atr_n = 14.min(bars.len().saturating_sub(1));
    for b in bars.iter().skip(bars.len().saturating_sub(atr_n)) {
        atr_sum += (b.spread_ticks() as f64).max(1.0);
    }
    let atr_ticks = atr_sum / (atr_n.max(1)) as f64;
    let volatility_risk_pct = atr_ticks * tick_size / current_price.max(1e-12) * 100.0;

    let mut fake_break_risk: f64 = 0.20;
    if structure.volume_trend.contains("Azalan") && structure.price_zone.contains("Üst Bant") {
        fake_break_risk += 0.30;
    }
    if structure.spread_status.contains("Genişliyor") {
        fake_break_risk += 0.15;
    }
    fake_break_risk = fake_break_risk.clamp(0.05, 0.80);

    let last = bars.last().unwrap();
    let momentum_risk: f64 = if last.close.0 > last.open.0 && last.volume.0 < 1000 {
        0.30 // Hacimsiz yükseliş zayıf
    } else {
        0.10
    };

    let size_factor = (1.0 - (volatility_risk_pct / 100.0).clamp(0.0, 0.5))
        * (1.0 - fake_break_risk.clamp(0.0, 0.9))
        * (1.0 - momentum_risk.clamp(0.0, 0.9));
    let size_factor = size_factor.clamp(0.1, 1.0);

    ProbabilityForecast {
        breakout_upper: (breakout_upper * 10000.0).round() / 10000.0,
        breakdown_lower: (breakdown_lower * 10000.0).round() / 10000.0,
        range_continuation: (range_continuation * 10000.0).round() / 10000.0,
        volatility_risk_pct: (volatility_risk_pct * 100.0).round() / 100.0,
        fake_break_risk: (fake_break_risk * 100.0).round() / 100.0,
        momentum_risk,
        suggested_position_size_factor: (size_factor * 100.0).round() / 100.0,
        confidence_interval: 0.025,
        brier_score_reference: 0.04,
        model_features: vec![
            "POC_Mesafe".into(),
            "Spread_Delta".into(),
            "Volume_Delta".into(),
            "RSI_Divergence".into(),
            "Bar_Count_Since_Spring".into(),
        ],
    }
}

/// Bias önerisi: v4 olasılık kuralları + durum makinesi ağırlıkları.
fn suggested_bias(machine: &WyckoffStateMachine, scorer: &ContextualScorer, probs: &ProbabilityForecast) -> Bias {
    if probs.breakout_upper > 0.65 && probs.fake_break_risk < 0.35 {
        Bias::Bullish
    } else if probs.breakdown_lower > 0.55 && probs.fake_break_risk < 0.30 {
        Bias::Bearish
    } else if machine.state.accumulation_weight > 0.6 && scorer.trend_angle > 0.0 {
        Bias::Bullish
    } else if machine.state.distribution_weight > 0.6 && scorer.trend_angle < 0.0 {
        Bias::Bearish
    } else {
        Bias::Neutral
    }
}