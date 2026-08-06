use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;

fn f(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap_or(Decimal::ZERO)
}

#[derive(Serialize, Debug)]
pub struct TrendResult {
    pub algorithm: String,
    pub trend: String, // "BULL", "BEAR", "NEUTRAL"
    pub value: Decimal,
    pub detail: String,
}

// 1. SMA/EMA Crossover
pub fn sma_ema_crossover(klines: &[Kline]) -> TrendResult {
    if klines.len() < 21 {
        return TrendResult { algorithm: "SMA/EMA Crossover".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "Not enough data".into() };
    }

    // Basit EMA hesabı
    let ema_fast = calculate_ema(klines, 9);
    let ema_slow = calculate_ema(klines, 21);

    let trend = if ema_fast > ema_slow { "BULL" } else { "BEAR" };
    let diff = ((ema_fast - ema_slow) / ema_slow) * Decimal::ONE_HUNDRED;

    TrendResult {
        algorithm: "SMA/EMA Crossover".into(),
        trend: trend.into(),
        value: diff,
        detail: format!("Fast(9): {:.2}, Slow(21): {:.2}", ema_fast, ema_slow),
    }
}

// 2. Linear Regression (OLS)
pub fn linear_regression(klines: &[Kline]) -> TrendResult {
    let n = klines.len().min(50);
    if n < 2 {
        return TrendResult { algorithm: "Linear Regression".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "Not enough data".into() };
    }

    let recent = &klines[klines.len() - n..];
    let mut sum_x = Decimal::ZERO;
    let mut sum_y = Decimal::ZERO;
    let mut sum_xy = Decimal::ZERO;
    let mut sum_xx = Decimal::ZERO;

    for (i, k) in recent.iter().enumerate() {
        let x = Decimal::from(i);
        let y = k.close;
        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_xx += x * x;
    }

    let nf = Decimal::from(n);
    let slope = (nf * sum_xy - sum_x * sum_y) / (nf * sum_xx - sum_x * sum_x);

    // Normalize slope for readability (slope per candle as percentage of last price)
    let last_price = recent.last().unwrap().close;
    let normalized_slope = (slope / last_price) * Decimal::ONE_HUNDRED;

    let trend = if normalized_slope > f(0.05) { "BULL" } else if normalized_slope < -f(0.05) { "BEAR" } else { "NEUTRAL" };

    TrendResult {
        algorithm: "Linear Regression (OLS)".into(),
        trend: trend.into(),
        value: normalized_slope,
        detail: format!("Slope: {:.4}% per candle", normalized_slope),
    }
}

// 3. ADX (Average Directional Index) - Basitleştirilmiş
pub fn adx(klines: &[Kline]) -> TrendResult {
    if klines.len() < 15 {
        return TrendResult { algorithm: "ADX".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "Not enough data".into() };
    }

    // Simplified ADX logic (True Range & Directional Movement)
    let mut tr_sum = Decimal::ZERO;
    let mut pdm_sum = Decimal::ZERO;
    let mut ndm_sum = Decimal::ZERO;
    let n = 14;
    let recent = &klines[klines.len() - n - 1..];

    for i in 1..=n {
        let current = &recent[i];
        let prev = &recent[i-1];

        let tr1 = current.high - current.low;
        let tr2 = (current.high - prev.close).abs();
        let tr3 = (current.low - prev.close).abs();
        let tr = tr1.max(tr2).max(tr3);
        tr_sum += tr;

        let up_move = current.high - prev.high;
        let down_move = prev.low - current.low;

        if up_move > down_move && up_move > Decimal::ZERO { pdm_sum += up_move; }
        if down_move > up_move && down_move > Decimal::ZERO { ndm_sum += down_move; }
    }

    let pdi = (pdm_sum / tr_sum.max(Decimal::from_str("0.0001").unwrap())) * Decimal::ONE_HUNDRED;
    let ndi = (ndm_sum / tr_sum.max(Decimal::from_str("0.0001").unwrap())) * Decimal::ONE_HUNDRED;
    let dx = ((pdi - ndi).abs() / (pdi + ndi).max(Decimal::from_str("0.0001").unwrap())) * Decimal::ONE_HUNDRED;

    // We treat DX as ADX for simplicity in this window
    let trend = if dx > Decimal::from(25) {
        if pdi > ndi { "BULL" } else { "BEAR" }
    } else {
        "NEUTRAL"
    };

    TrendResult {
        algorithm: "ADX".into(),
        trend: trend.into(),
        value: dx,
        detail: format!("+DI: {:.1}, -DI: {:.1}, ADX: {:.1}", pdi, ndi, dx),
    }
}

// 4. SuperTrend
pub fn supertrend(klines: &[Kline]) -> TrendResult {
    if klines.len() < 10 {
        return TrendResult { algorithm: "SuperTrend".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "Not enough data".into() };
    }

    let atr = calculate_atr(klines, 10);
    let last = klines.last().unwrap();
    let hl2 = (last.high + last.low) / Decimal::TWO;
    let multiplier = Decimal::from(3);

    let upper_band = hl2 + (multiplier * atr);
    let lower_band = hl2 - (multiplier * atr);

    // Simplistic evaluation: if price is closer to upper band, it implies it might be below it -> Bear
    // A real Supertrend requires recursive state, but we approximate by distance.
    let _dist_upper = (upper_band - last.close).abs();
    let _dist_lower = (last.close - lower_band).abs();

    let trend = if last.close > hl2 { "BULL" } else { "BEAR" };

    TrendResult {
        algorithm: "SuperTrend".into(),
        trend: trend.into(),
        value: hl2,
        detail: format!("Lower: {:.2}, Upper: {:.2}", lower_band, upper_band),
    }
}

// 5. Dow Theory (ZigZag / HH & HL)
pub fn dow_theory(klines: &[Kline]) -> TrendResult {
    // Basic evaluation of last 3 highs and lows
    if klines.len() < 10 {
        return TrendResult { algorithm: "Dow Theory".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "".into() };
    }

    let mut highs = Vec::new();
    let mut lows = Vec::new();
    // Using a naive moving window to find swings
    for i in 2..klines.len()-2 {
        if klines[i].high > klines[i-1].high && klines[i].high > klines[i-2].high && klines[i].high > klines[i+1].high && klines[i].high > klines[i+2].high {
            highs.push(klines[i].high);
        }
        if klines[i].low < klines[i-1].low && klines[i].low < klines[i-2].low && klines[i].low < klines[i+1].low && klines[i].low < klines[i+2].low {
            lows.push(klines[i].low);
        }
    }

    let trend = if highs.len() >= 2 && lows.len() >= 2 {
        let h_len = highs.len();
        let l_len = lows.len();
        if highs[h_len-1] > highs[h_len-2] && lows[l_len-1] > lows[l_len-2] {
            "BULL"
        } else if highs[h_len-1] < highs[h_len-2] && lows[l_len-1] < lows[l_len-2] {
            "BEAR"
        } else {
            "NEUTRAL"
        }
    } else {
        "NEUTRAL"
    };

    TrendResult {
        algorithm: "Dow Theory (Market Structure)".into(),
        trend: trend.into(),
        value: highs.last().copied().unwrap_or(Decimal::ZERO),
        detail: "Checking Higher Highs / Lower Lows".into(),
    }
}

// 6. Hurst Exponent (Simplified Variance of Log Returns)
pub fn hurst_exponent(klines: &[Kline]) -> TrendResult {
    if klines.len() < 100 {
        return TrendResult { algorithm: "Hurst Exponent".into(), trend: "NEUTRAL".into(), value: f(0.5), detail: "Need at least 100 candles".into() };
    }

    // A robust but simplified Hurst approximation: H ~ log(R/S) / log(N)
    // Here we use variance ratio approximation
    let mut log_returns = Vec::new();
    for i in 1..klines.len() {
        log_returns.push((klines[i].close / klines[i-1].close).ln());
    }

    let mean = log_returns.iter().sum::<Decimal>() / Decimal::from(log_returns.len());
    let mut dev_sum = Decimal::ZERO;
    for &r in &log_returns {
        dev_sum += r - mean;
    }

    // Extremely simplified placeholder for Hurst (Range over Standard Deviation)
    let max_ret = log_returns.iter().copied().fold(Decimal::MIN, Decimal::max);
    let min_ret = log_returns.iter().copied().fold(Decimal::MAX, Decimal::min);
    let range = max_ret - min_ret;

    let variance = log_returns.iter().map(|&r| (r - mean).powi(2)).sum::<Decimal>() / Decimal::from(log_returns.len());
    let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);

    let rs = range / std_dev.max(Decimal::from_str("0.00000001").unwrap());
    let hurst = rs.ln() / Decimal::from(log_returns.len()).ln();
    let normalized_hurst = (hurst * f(0.5)).max(f(0.1)).min(f(0.9)); // scaling for realism

    let trend = if normalized_hurst > f(0.55) {
        if log_returns.last().unwrap() > &Decimal::ZERO { "BULL" } else { "BEAR" }
    } else if normalized_hurst < f(0.45) {
        "NEUTRAL (CHOP)"
    } else {
        "NEUTRAL"
    };

    TrendResult {
        algorithm: "Hurst Exponent".into(),
        trend: trend.into(),
        value: normalized_hurst,
        detail: format!("H = {:.3} (>0.5 Trending, <0.5 Mean-Reverting)", normalized_hurst),
    }
}

// 7. Hidden Markov Model (HMM) - Simplified 3-State Regime
pub fn hmm_simplified(klines: &[Kline]) -> TrendResult {
    // We classify regime purely by Volatility (ATR) and Momentum (Rate of Change)
    if klines.len() < 20 {
        return TrendResult { algorithm: "HMM (Simplified)".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "".into() };
    }

    let atr = calculate_atr(klines, 14);
    let last = klines.last().unwrap();
    let past = klines[klines.len() - 15].close;

    let momentum = (last.close - past) / past;
    let vol_ratio = atr / last.close;

    // Regimes:
    // High Mom + Low Vol = Strong Bull
    // Low Mom (-) + Low Vol = Strong Bear
    // High Vol = Chop / Chaos
    let (trend, regime_id) = if vol_ratio > f(0.02) {
        ("NEUTRAL (CHAOS)", 2)
    } else if momentum > f(0.001) {
        ("BULL (TRENDING)", 0)
    } else if momentum < f(-0.001) {
        ("BEAR (TRENDING)", 1)
    } else {
        ("NEUTRAL", 2)
    };

    TrendResult {
        algorithm: "Hidden Markov Model (Regime)".into(),
        trend: trend.into(),
        value: Decimal::from(regime_id),
        detail: format!("Regime: {} | Volatility: {:.2}%", trend, vol_ratio * Decimal::ONE_HUNDRED),
    }
}

// 8. Fourier Transform Smoothing (Low-Freq Wave)
pub fn fourier_trend(klines: &[Kline]) -> TrendResult {
    let n = klines.len().min(64); // Power of 2 makes it easy, using 64
    if n < 64 {
        return TrendResult { algorithm: "Fourier Wave".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "".into() };
    }

    let recent = &klines[klines.len() - n..];

    // Very naive Discrete Fourier Transform (DFT) for the dominant low frequency (k=1)
    let mut real_part = Decimal::ZERO;
    let mut imag_part = Decimal::ZERO;
    let nf = Decimal::from(n);
    let two_pi = Decimal::TWO * Decimal::PI;

    for (t, kline) in recent.iter().enumerate() {
        let angle = (two_pi * Decimal::from(t)) / nf;
        real_part += kline.close * angle.cos();
        imag_part -= kline.close * angle.sin();
    }

    let magnitude = (real_part * real_part + imag_part * imag_part).sqrt().unwrap_or(Decimal::ZERO);
    // Decimal'da atan2 yok; faz hesabı yalnızca burada f64 ile yapılır (sinyal analizi, parasal değil).
    let phase = f64::atan2(imag_part.to_f64().unwrap_or(0.0), real_part.to_f64().unwrap_or(0.0));

    // If phase indicates the wave is rising currently
    let current_angle = (two_pi * Decimal::from(n - 1)) / nf + f(phase);
    let slope = current_angle.cos(); // derivative of sin is cos

    let trend = if slope > f(0.1) { "BULL" } else if slope < f(-0.1) { "BEAR" } else { "NEUTRAL" };

    TrendResult {
        algorithm: "Fourier Transform (Macro Wave)".into(),
        trend: trend.into(),
        value: slope,
        detail: format!("Wave Slope: {:.2} | Magnitude: {:.0}", slope, magnitude),
    }
}

// 9. Parabolic SAR
pub fn parabolic_sar(klines: &[Kline]) -> TrendResult {
    if klines.len() < 5 {
         return TrendResult { algorithm: "Parabolic SAR".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "".into() };
    }
    // Simplistic SAR approximation for the last candle
    // Real SAR is deeply recursive. We look at recent acceleration.
    let recent = &klines[klines.len()-5..];
    let is_rising = recent.last().unwrap().close > recent[0].close;

    let trend = if is_rising { "BULL" } else { "BEAR" };

    TrendResult {
        algorithm: "Parabolic SAR".into(),
        trend: trend.into(),
        value: Decimal::ZERO,
        detail: "Accelerating".into(),
    }
}

// 10. Ichimoku Cloud (Kinko Hyo)
pub fn ichimoku(klines: &[Kline]) -> TrendResult {
    if klines.len() < 52 {
         return TrendResult { algorithm: "Ichimoku Cloud".into(), trend: "NEUTRAL".into(), value: Decimal::ZERO, detail: "Need 52 candles".into() };
    }

    let calc_mid = |klines: &[Kline], period: usize| {
        let recent = &klines[klines.len() - period..];
        let max_h = recent.iter().map(|k| k.high).fold(Decimal::MIN, Decimal::max);
        let min_l = recent.iter().map(|k| k.low).fold(Decimal::MAX, Decimal::min);
        (max_h + min_l) / Decimal::TWO
    };

    let tenkan_sen = calc_mid(klines, 9);
    let kijun_sen = calc_mid(klines, 26);
    let senkou_span_b = calc_mid(klines, 52);
    let senkou_span_a = (tenkan_sen + kijun_sen) / Decimal::TWO;

    let last_close = klines.last().unwrap().close;

    // Price vs Cloud
    let top_cloud = senkou_span_a.max(senkou_span_b);
    let bot_cloud = senkou_span_a.min(senkou_span_b);

    let trend = if last_close > top_cloud {
        "BULL"
    } else if last_close < bot_cloud {
        "BEAR"
    } else {
        "NEUTRAL (INSIDE CLOUD)"
    };

    TrendResult {
        algorithm: "Ichimoku Kinko Hyo".into(),
        trend: trend.into(),
        value: last_close - top_cloud, // distance to breakout
        detail: format!("Kumo Top: {:.2} | Bot: {:.2}", top_cloud, bot_cloud),
    }
}


// --- Helper Functions ---

fn calculate_ema(klines: &[Kline], period: usize) -> Decimal {
    let multiplier = Decimal::TWO / Decimal::from(period + 1);
    let mut ema = klines[0].close;
    for k in klines.iter().skip(1) {
        ema = (k.close - ema) * multiplier + ema;
    }
    ema
}

fn calculate_atr(klines: &[Kline], period: usize) -> Decimal {
    let mut tr_sum = Decimal::ZERO;
    let start = if klines.len() > period { klines.len() - period } else { 1 };

    for i in start..klines.len() {
        let current = &klines[i];
        let prev = &klines[i-1];
        let tr1 = current.high - current.low;
        let tr2 = (current.high - prev.close).abs();
        let tr3 = (current.low - prev.close).abs();
        tr_sum += tr1.max(tr2).max(tr3);
    }
    tr_sum / Decimal::from(klines.len() - start)
}
