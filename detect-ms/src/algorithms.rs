use ohlcv_engine::Kline;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct MSResult {
    pub trend: String,          // BULLISH, BEARISH
    pub last_bos: Option<f64>,  // Fiyat
    pub last_choch: Option<f64>,// Fiyat
    pub strong_low: Option<f64>,// Protected
    pub weak_high: Option<f64>, // Targeted
    pub strong_high: Option<f64>, // Protected (Bearish trend)
    pub weak_low: Option<f64>,  // Targeted (Bearish trend)
}

#[derive(Clone, Debug, PartialEq)]
enum PivotType {
    High,
    Low,
}

#[derive(Clone, Debug)]
struct Pivot {
    price: f64,
    index: usize,
    p_type: PivotType,
}

pub fn analyze_market_structure(klines: &[Kline]) -> MSResult {
    if klines.len() < 20 {
        return MSResult {
            trend: "UNKNOWN".into(), last_bos: None, last_choch: None,
            strong_low: None, weak_high: None, strong_high: None, weak_low: None,
        };
    }

    let pivots = find_pivots(klines, 5);
    
    if pivots.len() < 4 {
        return MSResult {
            trend: "UNKNOWN".into(), last_bos: None, last_choch: None,
            strong_low: None, weak_high: None, strong_high: None, weak_low: None,
        };
    }

    // SMC Analizi (Geçmişten günümüze tarama)
    let mut trend = "UNKNOWN";
    let mut last_bos = None;
    let mut last_choch = None;
    let mut strong_low = None;
    let mut weak_high = None;
    let mut strong_high = None;
    let mut weak_low = None;

    let highs: Vec<&Pivot> = pivots.iter().filter(|p| p.p_type == PivotType::High).collect();
    let lows: Vec<&Pivot> = pivots.iter().filter(|p| p.p_type == PivotType::Low).collect();

    if highs.len() >= 2 && lows.len() >= 2 {
        let mut t_state = "UNKNOWN";
        
        for i in 1..highs.len().min(lows.len()) {
            let h1 = highs[i-1].price;
            let h2 = highs[i].price;
            let l1 = lows[i-1].price;
            let l2 = lows[i].price;

            if h2 > h1 && l2 > l1 {
                t_state = "BULLISH";
                last_bos = Some(h1); // Kırılan eski tepe (BOS)
                strong_low = Some(l1); // Trendi koruyan güçlü dip
                weak_high = Some(h2); // Hedef tepe
                last_choch = Some(l1); // Trendi bozacak seviye
            } else if h2 < h1 && l2 < l1 {
                t_state = "BEARISH";
                last_bos = Some(l1); // Kırılan eski dip (BOS)
                strong_high = Some(h1); // Trendi koruyan güçlü tepe
                weak_low = Some(l2); // Hedef dip
                last_choch = Some(h1); // Trendi bozacak seviye
            } else {
                // Sıkışma (Consolidation) - Eski yapıyı koru ama CHoCH kontrolü yap
                if t_state == "BULLISH" {
                    if l2 < strong_low.unwrap_or(0.0) {
                        t_state = "BEARISH (CHoCH)";
                        last_choch = strong_low;
                        strong_high = Some(h2);
                        weak_low = Some(l2);
                    } else {
                        t_state = "CONSOLIDATION (BULLISH BIAS)";
                    }
                } else if t_state == "BEARISH" {
                    if h2 > strong_high.unwrap_or(f64::MAX) {
                        t_state = "BULLISH (CHoCH)";
                        last_choch = strong_high;
                        strong_low = Some(l2);
                        weak_high = Some(h2);
                    } else {
                        t_state = "CONSOLIDATION (BEARISH BIAS)";
                    }
                }
            }
        }
        trend = t_state;
    }

    MSResult {
        trend: trend.into(),
        last_bos,
        last_choch,
        strong_low,
        weak_high,
        strong_high,
        weak_low,
    }
}

fn find_pivots(klines: &[Kline], window: usize) -> Vec<Pivot> {
    let mut pivots = Vec::new();
    let n = klines.len();
    if n < window * 2 + 1 { return pivots; }

    for i in window..(n - window) {
        let current_high = klines[i].high;
        let current_low = klines[i].low;

        let mut is_high = true;
        let mut is_low = true;

        for j in 1..=window {
            if klines[i - j].high > current_high || klines[i + j].high > current_high { is_high = false; }
            if klines[i - j].low < current_low || klines[i + j].low < current_low { is_low = false; }
        }

        if is_high { pivots.push(Pivot { price: current_high, index: i, p_type: PivotType::High }); }
        if is_low { pivots.push(Pivot { price: current_low, index: i, p_type: PivotType::Low }); }
    }
    pivots
}
