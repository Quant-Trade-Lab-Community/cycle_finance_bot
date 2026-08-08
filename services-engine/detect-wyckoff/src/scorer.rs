// ============================================================================
// 2. BAĞLAMSAL PUANLAMA MOTORU — Gerçek Bayesian
// trend_angle (EMA52 eğimi), atr_percent, range konumu, lojistik sigmoid.
// ============================================================================

use serde::Serialize;

use crate::models::{Bar, Tick};
use crate::state::{WeightedEvent, WyckoffEvent};

#[derive(Debug, Clone, Serialize)]
pub struct ContextualScorer {
    pub trend_angle: f64, // EMA50 eğimi (−1 ile +1)
    pub atr_percent: f64, // 0-1 arası (ATR / Fiyat)
    pub range_high: Tick,
    pub range_low: Tick,
}

impl ContextualScorer {
    /// Tüm pencere üzerinden bağlamı inşa eder.
    /// trend_angle: EMA50 son iki değerinin normalize edilmiş eğimi.
    /// atr_percent: ATR(14) / son kapanış.
    pub fn build(bars: &[Bar]) -> Self {
        let closes: Vec<f64> = bars.iter().map(|b| b.close.0 as f64).collect();
        let ema = ema(&closes, 50);
        let slope = if ema.len() >= 2 && ema[ema.len() - 2] != 0.0 {
            let last = ema[ema.len() - 1];
            let prev = ema[ema.len() - 2];
            ((last - prev) / prev.abs()).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        let last_close = closes.last().copied().unwrap_or(1.0);
        let atr = atr(bars, 14);
        let atr_percent = if last_close > 0.0 {
            (atr / last_close).min(1.0)
        } else {
            0.0
        };

        let range_high = Tick(
            bars.iter()
                .map(|b| b.high.0)
                .max()
                .unwrap_or(0),
        );
        let range_low = Tick(
            bars.iter()
                .map(|b| b.low.0)
                .min()
                .unwrap_or(0),
        );

        Self {
            trend_angle: slope,
            atr_percent,
            range_high,
            range_low,
        }
    }

    /// Bayesian bağlamsal skor — sigmoid ile [0,1]'e oturtur.
    pub fn evaluate(&self, event: &WeightedEvent) -> f64 {
        let raw_score = event.strength.clamp(0.0, 1.0);

        // Range içindeki konum (0..1)
        let range_range = (self.range_high.0 - self.range_low.0).max(1);
        let proximity =
            ((event.price.0 - self.range_low.0).clamp(0, range_range) as f64) / range_range as f64;

        let context_modifier = match event.raw {
            // Düşü trendinde Spring'ler %70 tuzağıdır → düşük skor
            WyckoffEvent::Spring => {
                if self.trend_angle < -0.3 {
                    0.2
                } else if self.trend_angle > 0.3 {
                    1.4
                } else {
                    1.0
                }
            }
            WyckoffEvent::SignOfStrength => {
                if proximity > 0.8 {
                    1.5
                } else {
                    0.8
                }
            }
            // Yükseli trendinde UT tuzağıdır
            WyckoffEvent::UpThrust => {
                if self.trend_angle > 0.3 {
                    0.3
                } else {
                    1.2
                }
            }
            WyckoffEvent::SellingClimax => {
                if proximity < 0.2 {
                    1.3
                } else {
                    0.9
                }
            }
        };

        // Volatilite düzeltmesi (ATR çok yüksekse sinyal güvenilirliği düşer)
        let atr_mod = 1.0 - self.atr_percent.min(0.5);

        let raw = raw_score * context_modifier * atr_mod;
        // Lojistik dönüşüm: sertleştirilmiş sigmoid
        1.0 / (1.0 + (-8.0 * (raw - 0.5)).exp())
    }
}

/// EMA — basit üstel hareketli ortalama.
fn ema(values: &[f64], period: usize) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let k = 2.0 / (period as f64 + 1.0);
    let warmup = period.min(values.len());
    let mut prev = values[..warmup].iter().sum::<f64>() / warmup as f64;
    let mut out = vec![prev];
    for v in values.iter().skip(1) {
        prev = (*v - prev) * k + prev;
        out.push(prev);
    }
    out
}

/// ATR(period) — true range ortalaması, tick bazlı.
fn atr(bars: &[Bar], period: usize) -> f64 {
    let n = bars.len();
    if n < 2 {
        return 0.0;
    }
    let window = period.min(n - 1);
    let mut sum = 0.0;
    for i in (n - window)..n {
        let prev_close = bars[i - 1].close.0;
        let tr = (bars[i].high.0 - bars[i].low.0)
            .max((bars[i].high.0 - prev_close).abs())
            .max((bars[i].low.0 - prev_close).abs());
        sum += tr as f64;
    }
    sum / window as f64
}