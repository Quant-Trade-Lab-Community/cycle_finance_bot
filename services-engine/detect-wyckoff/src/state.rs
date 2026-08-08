// ============================================================================
// 4. DURUM MATRİSİ — Wyckoff State Machine
// detect_all + update_weights gerçek implementasyon. Softmax normalize.
// ============================================================================

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::models::{Bar, Tick};
use crate::scorer::ContextualScorer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WyckoffEvent {
    Spring,
    SignOfStrength,
    UpThrust,
    SellingClimax,
}

impl WyckoffEvent {
    pub fn label(&self) -> &'static str {
        match self {
            WyckoffEvent::Spring => "Spring",
            WyckoffEvent::SignOfStrength => "SOS",
            WyckoffEvent::UpThrust => "UT",
            WyckoffEvent::SellingClimax => "SC",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeightedEvent {
    pub raw: WyckoffEvent,
    pub price: Tick,
    pub strength: f64, // Hacim oranına göre 0-1
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProbabilisticState {
    pub accumulation_weight: f64,
    pub distribution_weight: f64,
    pub trend_strength: f64,
}

impl Default for ProbabilisticState {
    fn default() -> Self {
        Self {
            accumulation_weight: 0.5,
            distribution_weight: 0.5,
            trend_strength: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Signal {
    Long { entry: Tick, confidence: f64 },
    Short { entry: Tick, confidence: f64 },
}

impl Signal {
    pub fn label(&self) -> &'static str {
        match self {
            Signal::Long { .. } => "LONG",
            Signal::Short { .. } => "SHORT",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalStats {
    pub springs: u64,
    pub sos: u64,
    pub upthrust: u64,
    pub selling_climax: u64,
    pub long_signals: u64,
    pub short_signals: u64,
    pub fake_springs: u64, // Düşüş trendinde üretilen Spring-yanlışları
}

pub struct WyckoffStateMachine {
    pub state: ProbabilisticState,
    pub stats: SignalStats,
    history: VecDeque<Bar>,
    pub scored_events: Vec<(WyckoffEvent, f64)>,
}

const HISTORY_LEN: usize = 40;

impl WyckoffStateMachine {
    pub fn new() -> Self {
        Self {
            state: ProbabilisticState::default(),
            stats: SignalStats::default(),
            history: VecDeque::with_capacity(HISTORY_LEN),
            scored_events: Vec::new(),
        }
    }

    pub fn observe(&mut self, bar: &Bar) {
        self.history.push_back(bar.clone());
        while self.history.len() > HISTORY_LEN {
            self.history.pop_front();
        }
    }

    fn window_extremes(&self) -> Option<(f64, f64)> {
        if self.history.is_empty() {
            return None;
        }
        let rng_high = self.history.iter().map(|b| b.high.0 as f64).fold(f64::NEG_INFINITY, f64::max);
        let rng_low = self.history.iter().map(|b| b.low.0 as f64).fold(f64::INFINITY, f64::min);
        Some((rng_high, rng_low))
    }

    fn window_avg_volume(&self) -> Option<f64> {
        if self.history.is_empty() {
            return None;
        }
        Some(self.history.iter().map(|b| b.volume.0 as f64).sum::<f64>() / self.history.len() as f64)
    }

    /// Gerçek tespit mantığı — Wyckoff v4 kuralları.
    pub fn detect_all(&self, bar: &Bar) -> Vec<WeightedEvent> {
        let mut events = Vec::new();
        let Some((rng_high, rng_low)) = self.window_extremes() else {
            return events;
        };
        let Some(prev) = self.history.back() else {
            return events;
        };
        let avg_vol = self.window_avg_volume().unwrap_or(0.0);

        let low = bar.low.0 as f64;
        let high = bar.high.0 as f64;
        let close = bar.close.0 as f64;
        let open = bar.open.0 as f64;
        let prev_close = prev.close.0 as f64;
        let volume = bar.volume.0 as f64;

        // Spring: Range dibini testi + güçlü toparlanma kapanışı
        if low <= rng_low * 1.002 && close > prev_close {
            events.push(WeightedEvent {
                raw: WyckoffEvent::Spring,
                price: bar.low,
                strength: (0.5 + (volume / (avg_vol.max(1.0))).clamp(0.0, 0.5)).min(1.0),
            });
        }
        // SOS: Yüksek hacimli yukarı kırılım
        if close > prev.high.0 as f64 && volume > avg_vol * 1.5 {
            events.push(WeightedEvent {
                raw: WyckoffEvent::SignOfStrength,
                price: bar.close,
                strength: (0.5 + (volume / (avg_vol * 1.5).max(1.0)).clamp(0.0, 0.5)).min(1.0),
            });
        }
        // UT: Üst bandı test edip geri çekilme (red mum)
        if high >= rng_high * 0.98 && close < open {
            events.push(WeightedEvent {
                raw: WyckoffEvent::UpThrust,
                price: bar.high,
                strength: (0.5 + (volume / (avg_vol * 1.0).max(1.0)).clamp(0.0, 0.5)).min(1.0),
            });
        }
        // SC (SellingClimax): Kapitülasyon — dip + 2.5x hacim + red mum
        if low <= rng_low * 1.001 && volume > avg_vol * 2.5 && close < open {
            events.push(WeightedEvent {
                raw: WyckoffEvent::SellingClimax,
                price: bar.low,
                strength: (0.6 + (volume / (avg_vol * 2.5).max(1.0)).clamp(0.0, 0.4)).min(1.0),
            });
        }

        events
    }

    /// Bayes güncellemesi + softmax normalizasyonu.
    pub fn update_weights(&mut self, event: &WyckoffEvent) {
        match event {
            WyckoffEvent::Spring | WyckoffEvent::SignOfStrength => {
                self.state.accumulation_weight =
                    (self.state.accumulation_weight + 0.1).min(1.0);
                self.state.distribution_weight =
                    (self.state.distribution_weight - 0.05).max(0.0);
            }
            WyckoffEvent::UpThrust | WyckoffEvent::SellingClimax => {
                self.state.distribution_weight =
                    (self.state.distribution_weight + 0.1).min(1.0);
                self.state.accumulation_weight =
                    (self.state.accumulation_weight - 0.05).max(0.0);
            }
        }
        let sum = self.state.accumulation_weight + self.state.distribution_weight;
        if sum > 0.0 {
            self.state.accumulation_weight /= sum;
            self.state.distribution_weight /= sum;
        }
    }

    /// Bar'ı işler: olay tespiti + bağlamsal skor + sinyal üretimi.
    ///
    /// Önemli: `observe` tespit SONRASI çağrılır — pencere mevcut barı içermez.
    pub fn ingest(&mut self, bar: &Bar, scorer: &ContextualScorer) -> Option<Signal> {
        let events = self.detect_all(bar);
        if events.is_empty() {
            self.observe(bar);
            return None;
        }

        let mut scored: Vec<(WeightedEvent, f64)> = Vec::new();
        for ev in &events {
            let s = scorer.evaluate(ev);
            scored.push((ev.clone(), s));
            self.stats_inc(ev.raw);
        }
        self.scored_events = scored
            .iter()
            .map(|(e, s)| (e.raw, *s))
            .collect::<Vec<_>>()
            .into_iter()
            .rev() // en güncel önce
            .take(8)
            .collect();

        let best = scored
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        let (event, score) = match best {
            Some((e, s)) => (e, *s),
            None => return None,
        };

        if score > 0.82 {
            self.update_weights(&event.raw);
            // Sahte Spring muhasebesi: düşüş trendinde Spring skorlanamaz — ama ölçülür
            if matches!(event.raw, WyckoffEvent::Spring) && scorer.trend_angle < -0.3 {
                self.stats.fake_springs += 1;
            }
            if matches!(event.raw, WyckoffEvent::SignOfStrength)
                && self.state.accumulation_weight > 0.75
            {
                self.stats.long_signals += 1;
                self.observe(bar);
                return Some(Signal::Long { entry: bar.close, confidence: score });
            }
            if matches!(event.raw, WyckoffEvent::UpThrust)
                && self.state.distribution_weight > 0.75
            {
                self.stats.short_signals += 1;
                self.observe(bar);
                return Some(Signal::Short { entry: bar.close, confidence: score });
            }
        }
        self.observe(bar);
        None
    }

    fn stats_inc(&mut self, ev: WyckoffEvent) {
        match ev {
            WyckoffEvent::Spring => self.stats.springs += 1,
            WyckoffEvent::SignOfStrength => self.stats.sos += 1,
            WyckoffEvent::UpThrust => self.stats.upthrust += 1,
            WyckoffEvent::SellingClimax => self.stats.selling_climax += 1,
        }
    }
}

impl Default for WyckoffStateMachine {
    fn default() -> Self {
        Self::new()
    }
}