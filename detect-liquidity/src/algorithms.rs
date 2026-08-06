use ohlcv_engine::Kline;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct LiquidityResult {
    pub eqh: Vec<f64>,
    pub eql: Vec<f64>,
    pub bullish_fvg: Vec<FVG>,
    pub bearish_fvg: Vec<FVG>,
    pub sweeps: Vec<Sweep>,
}

#[derive(Serialize, Debug)]
pub struct FVG {
    pub top: f64,
    pub bottom: f64,
}

#[derive(Serialize, Debug)]
pub struct Sweep {
    pub side: String, // "BUY_SIDE" or "SELL_SIDE"
    pub price_level: f64,
    pub index: usize,
}

pub fn analyze_liquidity(klines: &[Kline]) -> LiquidityResult {
    if klines.len() < 5 {
        return LiquidityResult { eqh: vec![], eql: vec![], bullish_fvg: vec![], bearish_fvg: vec![], sweeps: vec![] };
    }

    let eqh = find_equal_levels(klines, true, 0.0005); // %0.05
    let eql = find_equal_levels(klines, false, 0.0005);
    
    let (bullish_fvg, bearish_fvg) = find_fvgs(klines);
    let sweeps = find_sweeps(klines);

    LiquidityResult {
        eqh, eql, bullish_fvg, bearish_fvg, sweeps
    }
}

fn find_equal_levels(klines: &[Kline], is_high: bool, threshold_pct: f64) -> Vec<f64> {
    let mut levels = Vec::new();
    let n = klines.len();
    
    for i in 0..n {
        for j in (i+5)..n { // En az 5 mum arayla
            let p1 = if is_high { klines[i].high } else { klines[i].low };
            let p2 = if is_high { klines[j].high } else { klines[j].low };
            
            if (p1 - p2).abs() / p1 <= threshold_pct {
                levels.push((p1 + p2) / 2.0);
            }
        }
    }
    // Remove duplicates
    levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
    levels.dedup_by(|a, b| (*a - *b).abs() / *a < threshold_pct);
    levels
}

fn find_fvgs(klines: &[Kline]) -> (Vec<FVG>, Vec<FVG>) {
    let mut bull = Vec::new();
    let mut bear = Vec::new();

    for i in 2..klines.len() {
        let k1 = &klines[i-2];
        let k3 = &klines[i];

        // Bullish FVG: K1 High < K3 Low
        if k1.high < k3.low {
            bull.push(FVG { top: k3.low, bottom: k1.high });
        }
        // Bearish FVG: K1 Low > K3 High
        if k1.low > k3.high {
            bear.push(FVG { top: k1.low, bottom: k3.high });
        }
    }
    (bull, bear)
}

fn find_sweeps(klines: &[Kline]) -> Vec<Sweep> {
    let mut sweeps = Vec::new();
    // Basit bir Sweep analizi: Mum çok uzun iğne atmış ama gövdesi küçük kapanmış ve önceki mumları yutmuş
    for i in 5..klines.len() {
        let k = &klines[i];
        let body_top = k.open.max(k.close);
        let body_bot = k.open.min(k.close);
        
        let upper_wick = k.high - body_top;
        let lower_wick = body_bot - k.low;
        let body = body_top - body_bot;

        // Buy Side Sweep (Yukarı iğne atıp avlamış)
        if upper_wick > body * 3.0 {
            // Önceki mumların high'ını geçmiş mi?
            let prev_max = klines[i-5..i].iter().map(|x| x.high).fold(f64::MIN, f64::max);
            if k.high > prev_max && k.close < prev_max { // Body close below
                sweeps.push(Sweep { side: "BUY_SIDE_SWEEP".into(), price_level: k.high, index: i });
            }
        }

        // Sell Side Sweep (Aşağı iğne atıp avlamış)
        if lower_wick > body * 3.0 {
            let prev_min = klines[i-5..i].iter().map(|x| x.low).fold(f64::MAX, f64::min);
            if k.low < prev_min && k.close > prev_min { // Body close above
                sweeps.push(Sweep { side: "SELL_SIDE_SWEEP".into(), price_level: k.low, index: i });
            }
        }
    }
    sweeps
}
