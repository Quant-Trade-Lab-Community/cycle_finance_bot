use ohlcv_engine::Kline;

// 1. Fractal / Swing Extrema (Yerel Tepeler/Dipler)
pub fn swing_extrema(klines: &[Kline], window: usize) -> Vec<f64> {
    let mut extrema = Vec::new();
    let n = klines.len();

    if n < window * 2 + 1 {
        return extrema;
    }

    for i in window..(n - window) {
        let current_high = klines[i].high;
        let current_low = klines[i].low;

        let mut is_swing_high = true;
        let mut is_swing_low = true;

        for j in 1..=window {
            if klines[i - j].high > current_high || klines[i + j].high > current_high {
                is_swing_high = false;
            }
            if klines[i - j].low < current_low || klines[i + j].low < current_low {
                is_swing_low = false;
            }
        }

        if is_swing_high {
            extrema.push(current_high);
        }
        if is_swing_low {
            extrema.push(current_low);
        }
    }

    cluster_points(extrema, 0.002) // %0.2 tolerans
}

// 2. K-Means 1D Clustering (5 Merkez)
pub fn kmeans_1d(klines: &[Kline], k: usize) -> Vec<f64> {
    let mut data = Vec::new();
    for kline in klines {
        data.push(kline.high);
        data.push(kline.low);
    }

    if data.is_empty() {
        return Vec::new();
    }

    data.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Başlangıç merkezleri (Centroidler) - veriyi eşit aralıklarla böl
    let mut centroids = Vec::new();
    let step = data.len() / k.max(1);
    for i in 0..k {
        let idx = (i * step).min(data.len() - 1);
        centroids.push(data[idx]);
    }

    for _ in 0..100 { // Max iterasyon
        let mut clusters: Vec<Vec<f64>> = vec![Vec::new(); k];

        for &val in &data {
            let mut min_dist = f64::MAX;
            let mut closest = 0;
            for (i, &c) in centroids.iter().enumerate() {
                let dist = (val - c).abs();
                if dist < min_dist {
                    min_dist = dist;
                    closest = i;
                }
            }
            clusters[closest].push(val);
        }

        let mut new_centroids = Vec::new();
        let mut changed = false;

        for (i, cluster) in clusters.iter().enumerate() {
            if cluster.is_empty() {
                new_centroids.push(centroids[i]);
            } else {
                let sum: f64 = cluster.iter().sum();
                let mean = sum / cluster.len() as f64;
                new_centroids.push(mean);
                if (mean - centroids[i]).abs() > 1e-5 {
                    changed = true;
                }
            }
        }

        centroids = new_centroids;
        if !changed {
            break;
        }
    }

    centroids.sort_by(|a, b| b.partial_cmp(a).unwrap()); // Büyükten küçüğe
    centroids
}

// 3. Volume Profile (Hacim Dağılımı ve POC)
pub fn volume_profile(klines: &[Kline], bins: usize) -> Vec<f64> {
    if klines.is_empty() {
        return Vec::new();
    }

    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;

    for k in klines {
        if k.low < min_price { min_price = k.low; }
        if k.high > max_price { max_price = k.high; }
    }

    let bin_size = (max_price - min_price) / (bins as f64).max(1.0);
    let mut profile = vec![0.0; bins];

    for k in klines {
        let typical_price = (k.high + k.low + k.close) / 3.0;
        let mut bin_idx = ((typical_price - min_price) / bin_size).floor() as usize;
        if bin_idx >= bins {
            bin_idx = bins - 1;
        }
        profile[bin_idx] += k.volume;
    }

    // En yüksek hacimli 5 kutuyu bul
    let mut indexed_profile: Vec<(usize, f64)> = profile.into_iter().enumerate().collect();
    indexed_profile.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // Hacme göre büyükten küçüğe

    let mut sr_levels = Vec::new();
    for i in 0..5.min(indexed_profile.len()) {
        let bin_idx = indexed_profile[i].0;
        let price_level = min_price + (bin_idx as f64 * bin_size) + (bin_size / 2.0);
        sr_levels.push(price_level);
    }

    sr_levels.sort_by(|a, b| b.partial_cmp(a).unwrap());
    sr_levels
}

// 4. Kernel Density Estimation (KDE) - Basitleştirilmiş
pub fn kde_peaks(klines: &[Kline]) -> Vec<f64> {
    if klines.is_empty() {
        return Vec::new();
    }

    let bandwidth = 0.005; // Fiyat hassasiyetine göre ayarlanabilir
    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;
    let mut closes = Vec::new();

    for k in klines {
        closes.push(k.close);
        if k.close < min_price { min_price = k.close; }
        if k.close > max_price { max_price = k.close; }
    }

    let steps = 100;
    let step_size = (max_price - min_price) / (steps as f64).max(1.0);
    let mut density = Vec::new();

    for i in 0..=steps {
        let x = min_price + (i as f64 * step_size);
        let mut sum = 0.0;
        for &c in &closes {
            // Basit Gauss Kernel
            let u = (x - c) / bandwidth;
            let val = (-0.5 * u * u).exp() / ((2.0 * std::f64::consts::PI).sqrt());
            sum += val;
        }
        density.push((x, sum));
    }

    // Local Maxima (Peaks) bul
    let mut peaks = Vec::new();
    for i in 1..(density.len() - 1) {
        if density[i].1 > density[i-1].1 && density[i].1 > density[i+1].1 {
            peaks.push(density[i]);
        }
    }

    peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // Yoğunluğa göre sırala
    let mut sr_levels: Vec<f64> = peaks.iter().take(5).map(|p| p.0).collect();
    sr_levels.sort_by(|a, b| b.partial_cmp(a).unwrap());
    
    sr_levels
}


// Yardımcı Fonksiyon: Yakın noktaları (Örn: %0.2) tek bir merkezde kümele
fn cluster_points(points: Vec<f64>, threshold_pct: f64) -> Vec<f64> {
    if points.is_empty() {
        return Vec::new();
    }
    
    let mut sorted = points.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut clusters = Vec::new();
    let mut current_cluster = vec![sorted[0]];

    for i in 1..sorted.len() {
        let prev = current_cluster.last().unwrap();
        let curr = sorted[i];

        if (curr - prev) / prev <= threshold_pct {
            current_cluster.push(curr);
        } else {
            let avg = current_cluster.iter().sum::<f64>() / current_cluster.len() as f64;
            clusters.push(avg);
            current_cluster.clear();
            current_cluster.push(curr);
        }
    }

    if !current_cluster.is_empty() {
        let avg = current_cluster.iter().sum::<f64>() / current_cluster.len() as f64;
        clusters.push(avg);
    }

    clusters.sort_by(|a, b| b.partial_cmp(a).unwrap());
    clusters
}
