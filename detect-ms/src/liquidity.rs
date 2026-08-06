// ============================================================================
// MSMP 2.0 — KATMAN 5: LİKİDİTE POOL (VWAP Sapması & Volume Profile)
// ============================================================================
// Eşit bantlar TAMAMEN İPTAL. Volume Profile hesaplanır:
//   HVN (Yüksek Hacim Node) ve LVN (Düşük Hacim Node) tespit edilir.
// BSL Yoğunluğu = +1.5σ ile +3σ arası HVN bölgeleri
// SSL Yoğunluğu = -1.5σ ile -3σ arası HVN bölgeleri
// Likidite Skoru = Bölge hacmi / toplam hacim oranı (1-10)
// ============================================================================

use ohlcv_engine::Kline;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum NodeType {
    /// Yüksek Hacim Node — Kurumsal alım-satım yoğunluğu
    HVN,
    /// Düşük Hacim Node — Fiyat hızla geçer
    LVN,
}

#[derive(Debug, Clone, Serialize)]
pub struct VolumeNode {
    pub price_low: f64,
    pub price_high: f64,
    pub price_mid: f64,
    pub volume: f64,
    /// Bu node'un toplam hacme oranı (0.0 - 1.0)
    pub volume_ratio: f64,
    pub node_type: NodeType,
    /// Likidite skoru (1-10)
    pub liquidity_score: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiquidityAnalysis {
    /// Volume-Weighted Average Price
    pub vwap: f64,
    /// VWAP standart sapması (σ)
    pub vwap_std_dev: f64,
    /// Point of Control — en yüksek hacimli fiyat seviyesi
    pub poc: f64,
    /// Buy-Side Liquidity bölgeleri (+1.5σ ~ +3σ arası HVN)
    pub bsl_zones: Vec<VolumeNode>,
    /// Sell-Side Liquidity bölgeleri (-3σ ~ -1.5σ arası HVN)
    pub ssl_zones: Vec<VolumeNode>,
    pub bsl_total_volume: f64,
    pub ssl_total_volume: f64,
    /// BSL/SSL Oranı — Risk asimetrisi
    pub bsl_ssl_ratio: f64,
    /// Aktif Volatilite Bandı alt sınırı: POC - 1.5σ
    pub volatility_band_low: f64,
    /// Aktif Volatilite Bandı üst sınırı: POC + 1.5σ
    pub volatility_band_high: f64,
    /// Tam volume profile
    pub volume_profile: Vec<VolumeNode>,
}

/// VWAP (Volume-Weighted Average Price) hesaplaması
pub fn vwap(klines: &[Kline]) -> f64 {
    let mut cum_tp_vol = 0.0;
    let mut cum_vol = 0.0;

    for k in klines {
        let typical_price = (k.high + k.low + k.close) / 3.0;
        cum_tp_vol += typical_price * k.volume;
        cum_vol += k.volume;
    }

    if cum_vol == 0.0 {
        return 0.0;
    }
    cum_tp_vol / cum_vol
}

/// VWAP Standart Sapması (σ) — Hacim ağırlıklı
pub fn vwap_std_dev(klines: &[Kline], vwap_val: f64) -> f64 {
    if klines.is_empty() {
        return 0.0;
    }

    let mut sum_sq = 0.0;
    let mut cum_vol = 0.0;

    for k in klines {
        let typical_price = (k.high + k.low + k.close) / 3.0;
        sum_sq += k.volume * (typical_price - vwap_val).powi(2);
        cum_vol += k.volume;
    }

    if cum_vol == 0.0 {
        return 0.0;
    }
    (sum_sq / cum_vol).sqrt()
}

/// Volume Profile — Dinamik bucket'larla hacim dağılımı
pub fn volume_profile(klines: &[Kline], bucket_count: usize) -> Vec<VolumeNode> {
    if klines.is_empty() || bucket_count == 0 {
        return vec![];
    }

    let price_min = klines
        .iter()
        .map(|k| k.low)
        .fold(f64::INFINITY, f64::min);
    let price_max = klines
        .iter()
        .map(|k| k.high)
        .fold(f64::NEG_INFINITY, f64::max);

    if price_max <= price_min {
        return vec![];
    }

    let bucket_size = (price_max - price_min) / bucket_count as f64;
    let mut buckets = vec![0.0f64; bucket_count];
    let total_volume: f64 = klines.iter().map(|k| k.volume).sum();

    // Her mumun hacmini fiyat aralığına orantılı dağıt
    for k in klines {
        let low_idx = ((k.low - price_min) / bucket_size).floor() as usize;
        let high_idx = ((k.high - price_min) / bucket_size).floor() as usize;
        let low_idx = low_idx.min(bucket_count - 1);
        let high_idx = high_idx.min(bucket_count - 1);

        let span = (high_idx - low_idx + 1) as f64;
        let vol_per_bucket = k.volume / span;

        for b in low_idx..=high_idx {
            buckets[b] += vol_per_bucket;
        }
    }

    // Medyan hacmi hesapla (HVN/LVN eşiği olarak kullanılır)
    let mut sorted_vols: Vec<f64> = buckets.clone();
    sorted_vols.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_vol = sorted_vols[sorted_vols.len() / 2];

    let mut nodes = Vec::with_capacity(bucket_count);
    for (i, &vol) in buckets.iter().enumerate() {
        let p_low = price_min + i as f64 * bucket_size;
        let p_high = p_low + bucket_size;
        let ratio = if total_volume > 0.0 {
            vol / total_volume
        } else {
            0.0
        };

        let node_type = if vol >= median_vol * 1.5 {
            NodeType::HVN
        } else {
            NodeType::LVN
        };

        // Likidite Skoru: hacim oranının yüzdesel dilimi (1-10)
        let score = ((ratio * 100.0).round() as u8).clamp(1, 10);

        nodes.push(VolumeNode {
            price_low: p_low,
            price_high: p_high,
            price_mid: (p_low + p_high) / 2.0,
            volume: vol,
            volume_ratio: ratio,
            node_type,
            liquidity_score: score,
        });
    }

    nodes
}

/// BSL ve SSL bölgelerini tespit et
/// BSL: current_price + 1.5σ ~ +3σ arası HVN'ler
/// SSL: current_price - 3σ ~ -1.5σ arası HVN'ler
pub fn detect_bsl_ssl(
    nodes: &[VolumeNode],
    current_price: f64,
    sigma: f64,
) -> (Vec<VolumeNode>, Vec<VolumeNode>) {
    let bsl_low = current_price + 1.5 * sigma;
    let bsl_high = current_price + 3.0 * sigma;
    let ssl_low = current_price - 3.0 * sigma;
    let ssl_high = current_price - 1.5 * sigma;

    let bsl: Vec<VolumeNode> = nodes
        .iter()
        .filter(|n| {
            matches!(n.node_type, NodeType::HVN)
                && n.price_mid >= bsl_low
                && n.price_mid <= bsl_high
        })
        .cloned()
        .collect();

    let ssl: Vec<VolumeNode> = nodes
        .iter()
        .filter(|n| {
            matches!(n.node_type, NodeType::HVN)
                && n.price_mid >= ssl_low
                && n.price_mid <= ssl_high
        })
        .cloned()
        .collect();

    (bsl, ssl)
}

/// Tam likidite analizi pipeline'ı
pub fn analyze_liquidity(klines: &[Kline]) -> LiquidityAnalysis {
    if klines.is_empty() {
        return LiquidityAnalysis {
            vwap: 0.0,
            vwap_std_dev: 0.0,
            poc: 0.0,
            bsl_zones: vec![],
            ssl_zones: vec![],
            bsl_total_volume: 0.0,
            ssl_total_volume: 0.0,
            bsl_ssl_ratio: 0.0,
            volatility_band_low: 0.0,
            volatility_band_high: 0.0,
            volume_profile: vec![],
        };
    }

    let vwap_val = vwap(klines);
    let sigma = vwap_std_dev(klines, vwap_val);
    let profile = volume_profile(klines, 50);

    let current_price = klines.last().map(|k| k.close).unwrap_or(0.0);

    // POC: En yüksek hacimli bucket'ın orta noktası
    let poc = profile
        .iter()
        .max_by(|a, b| {
            a.volume
                .partial_cmp(&b.volume)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|n| n.price_mid)
        .unwrap_or(current_price);

    let (bsl, ssl) = detect_bsl_ssl(&profile, current_price, sigma);

    let bsl_total: f64 = bsl.iter().map(|n| n.volume).sum();
    let ssl_total: f64 = ssl.iter().map(|n| n.volume).sum();
    let ratio = if ssl_total > 0.0 {
        bsl_total / ssl_total
    } else if bsl_total > 0.0 {
        f64::INFINITY
    } else {
        1.0
    };

    LiquidityAnalysis {
        vwap: vwap_val,
        vwap_std_dev: sigma,
        poc,
        bsl_zones: bsl,
        ssl_zones: ssl,
        bsl_total_volume: bsl_total,
        ssl_total_volume: ssl_total,
        bsl_ssl_ratio: ratio,
        volatility_band_low: poc - 1.5 * sigma,
        volatility_band_high: poc + 1.5 * sigma,
        volume_profile: profile,
    }
}
