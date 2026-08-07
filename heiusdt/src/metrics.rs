//! Microstructure Metrics — kurumsal tick-by-tick metrik çekirdeği.
//!
//! Veri kaynağı: DATA MERKEZİ (`/dev/shm/demir_yumruk_ring`). price-feed KULLANILMAZ.
//!
//! Aşamalar:
//!   0. Lee-Ready Signing (trade yönü)
//!   1. WLOBI + Quote Slope (likidite mimarisi)
//!   2. EffDelta + Delta Velocity (saldırgan akış)
//!   3. Absorption Ratio + Iceberg (pasif emilim)
//!   4. aVPIN (mikro-yapı toksisitesi)
//!   5. Hasbrouck VAR + EfP (kalıcı/geçici etki)
//!   6. Alpha Basket (lojistik sinyal)

use std::collections::VecDeque;

// ── Metrik parametreleri (Θ) — shell'den değiştirilebilir ─────
// /tmp/listener_metrics.conf dosyasından okunur (listenconfig komutu).
pub const CONFIG_FILE: &str = "/tmp/listener_metrics.conf";

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub lambda: f64,           // WLOBI decay
    pub theta_vol: f64,        // Delta velocity eşiği
    pub alpha_bucket: f64,     // aVPIN bucket sabiti
    pub k_abs: usize,          // absorption penceresi (trade)
    pub n_bucket: usize,       // aVPIN bucket sayısı
    pub ice_threshold: f64,    // IDM eşiği
    pub efp_threshold: f64,    // execution footprint eşiği
    pub noise_corr: f64,       // Lee-Ready gürültü filtresi
    pub delta_window_sec: usize, // ΔV penceresi (saniye)
    pub gamma: [f64; 6],       // Alpha Basket ağırlıkları
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            lambda: 0.015,
            theta_vol: 2.5,
            alpha_bucket: 0.75,
            k_abs: 100,
            n_bucket: 50,
            ice_threshold: 1.2,
            efp_threshold: 0.05,
            noise_corr: 0.85,
            delta_window_sec: 60,
            gamma: [0.0, 0.4, -0.3, 0.5, 0.6, -0.35],
        }
    }
}

impl MetricsConfig {
    /// /tmp/listener_metrics.conf dosyasından parametreleri yükler.
    /// Format: key = value  (bir satırda bir parametre)
    pub fn load() -> Self {
        let mut cfg = Self::default();
        let content = match std::fs::read_to_string(CONFIG_FILE) {
            Ok(c) => c,
            Err(_) => return cfg,
        };
        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            let (k, v) = match t.split_once('=') {
                Some(x) => x,
                None => continue,
            };
            let k = k.trim();
            let v = v.trim();
            let f = |d: f64| v.parse::<f64>().unwrap_or(d);
            match k {
                "lambda" => cfg.lambda = f(cfg.lambda),
                "theta_vol" => cfg.theta_vol = f(cfg.theta_vol),
                "alpha_bucket" => cfg.alpha_bucket = f(cfg.alpha_bucket),
                "k_abs" => cfg.k_abs = v.parse::<usize>().unwrap_or(cfg.k_abs),
                "n_bucket" => cfg.n_bucket = v.parse::<usize>().unwrap_or(cfg.n_bucket),
                "ice_threshold" => cfg.ice_threshold = f(cfg.ice_threshold),
                "efp_threshold" => cfg.efp_threshold = f(cfg.efp_threshold),
                "noise_corr" => cfg.noise_corr = f(cfg.noise_corr),
                "delta_window_sec" => cfg.delta_window_sec = v.parse::<usize>().unwrap_or(cfg.delta_window_sec),
                "gamma0" => cfg.gamma[0] = f(cfg.gamma[0]),
                "gamma1" => cfg.gamma[1] = f(cfg.gamma[1]),
                "gamma2" => cfg.gamma[2] = f(cfg.gamma[2]),
                "gamma3" => cfg.gamma[3] = f(cfg.gamma[3]),
                "gamma4" => cfg.gamma[4] = f(cfg.gamma[4]),
                "gamma5" => cfg.gamma[5] = f(cfg.gamma[5]),
                _ => {}
            }
        }
        cfg
    }
}

// ── Derinlik kademesi ────────────────────────────────────────
#[derive(Debug, Clone, Copy, Default)]
pub struct DepthLevel {
    pub price: f64,
    pub qty: f64,
}

// ── Sembol başına metrik durumu ──────────────────────────────
pub struct SymbolMetrics {
    // Lee-Ready
    prev_price: f64,
    prev_prev_price: f64,
    prev_sign: i8,
    prev_delta: f64,
    // mid / spread
    mid: f64,
    avg_spread: f64,
    spread_count: u64,
    // order book (ilk 5 kademe)
    bids: [DepthLevel; 5],
    asks: [DepthLevel; 5],
    // EffDelta
    pub eff_delta: f64,
    eff_delta_hist: VecDeque<f64>, // saniyelik
    last_delta_time: u64,
    // Absorption
    trade_signs: VecDeque<(f64, i8)>, // (qty, sign)
    // aVPIN
    bucket_volume: f64,
    bucket_vbuy: VecDeque<f64>,
    bucket_vsell: VecDeque<f64>,
    last_park_high: f64,
    last_park_low: f64,
    // Hasbrouck VAR (son 200 örnek)
    var_r: VecDeque<f64>,
    var_x: VecDeque<f64>,
    // EfP
    last_depth_total: f64,
    // sonuçlar
    pub cfg: MetricsConfig,
    pub wlobi: f64,
    pub slope_ask: f64,
    pub slope_bid: f64,
    pub delta_velocity: f64,
    pub absorption: f64,
    pub idm: f64,
    pub avpin: f64,
    pub permanent_impact: f64,
    pub temporary_impact: f64,
    pub efp: f64,
    pub alpha_score: f64,
    pub p_long: f64,
    pub signal: i8, // +1 Long, -1 Short, 0 Nötr
}

impl Default for SymbolMetrics {
    fn default() -> Self {
        Self {
            prev_price: 0.0,
            prev_prev_price: 0.0,
            prev_sign: 0,
            prev_delta: 0.0,
            mid: 0.0,
            avg_spread: 0.0,
            spread_count: 0,
            bids: [DepthLevel::default(); 5],
            asks: [DepthLevel::default(); 5],
            eff_delta: 0.0,
            eff_delta_hist: VecDeque::new(),
            last_delta_time: 0,
            trade_signs: VecDeque::new(),
            bucket_volume: 0.0,
            bucket_vbuy: VecDeque::new(),
            bucket_vsell: VecDeque::new(),
            last_park_high: 0.0,
            last_park_low: f64::MAX,
            var_r: VecDeque::new(),
            var_x: VecDeque::new(),
            last_depth_total: 0.0,
            cfg: MetricsConfig::load(),
            wlobi: 0.0,
            slope_ask: 0.0,
            slope_bid: 0.0,
            delta_velocity: 0.0,
            absorption: 0.0,
            idm: 0.0,
            avpin: 0.0,
            permanent_impact: 0.0,
            temporary_impact: 0.0,
            efp: 0.0,
            alpha_score: 0.0,
            p_long: 0.5,
            signal: 0,
        }
    }
}

impl SymbolMetrics {
    /// Config dosyasını yeniden yükler (shell'den değiştirilen parametreleri uygular)
    pub fn reload_config(&mut self) {
        self.cfg = MetricsConfig::load();
        // Pencere sınırlarını yeni değerlere kırp
        while self.eff_delta_hist.len() > self.cfg.delta_window_sec {
            self.eff_delta_hist.pop_front();
        }
        while self.trade_signs.len() > self.cfg.k_abs {
            self.trade_signs.pop_front();
        }
        while self.bucket_vbuy.len() > self.cfg.n_bucket {
            self.bucket_vbuy.pop_front();
        }
        while self.bucket_vsell.len() > self.cfg.n_bucket {
            self.bucket_vsell.pop_front();
        }
    }

    // ══ AŞAMA 0: Lee-Ready Signing ═══════════════════════════
    pub fn lee_ready_sign(&mut self, price: f64) -> i8 {
        let mid = self.mid;
        let sign = if price > mid {
            1
        } else if price < mid {
            -1
        } else if self.prev_delta != 0.0 {
            self.prev_sign
        } else {
            // Tick rule: sign(P_t - P_{t-2})
            if price > self.prev_prev_price { 1 } else if price < self.prev_prev_price { -1 } else { 0 }
        } as i8;

        self.prev_delta = price - self.prev_price;
        self.prev_prev_price = self.prev_price;
        self.prev_price = price;
        self.prev_sign = sign;
        sign
    }

    // ══ Order book güncelleme (ilk 5 kademe) ═════════════════
    pub fn update_depth(&mut self, bids: &[DepthLevel], asks: &[DepthLevel]) {
        for i in 0..5 {
            self.bids[i] = bids.get(i).copied().unwrap_or_default();
            self.asks[i] = asks.get(i).copied().unwrap_or_default();
        }
        // Top of book → mid + spread
        let b0 = self.bids[0].price;
        let a0 = self.asks[0].price;
        if b0 > 0.0 && a0 > 0.0 {
            self.mid = (b0 + a0) / 2.0;
            let spread = a0 - b0;
            self.avg_spread = (self.avg_spread * self.spread_count as f64 + spread) / (self.spread_count + 1) as f64;
            self.spread_count += 1;
        }
        // EfP paydası: ilk 5 kademe toplam derinlik
        self.last_depth_total = self.bids.iter().map(|l| l.qty).sum::<f64>()
            + self.asks.iter().map(|l| l.qty).sum::<f64>();
    }

    // ══ AŞAMA 1: WLOBI ═══════════════════════════════════════
    pub fn compute_wlobi(&mut self) -> f64 {
        // ω_i = e^(-λ·i) — kademe derinliği yaşam süresi vekili
        let mut w_bid = 0.0;
        let mut w_ask = 0.0;
        for i in 0..5 {
            let w = (-self.cfg.lambda * (i as f64 + 1.0)).exp();
            w_bid += w * self.bids[i].qty;
            w_ask += w * self.asks[i].qty;
        }
        let denom = w_ask + w_bid;
        self.wlobi = if denom > 0.0 { (w_ask - w_bid) / denom } else { 0.0 };
        self.wlobi
    }

    // Quote Slope: (ln V1 - ln V5) / (P5 - P1)
    pub fn compute_slopes(&mut self) {
        let (v1a, v5a, p1a, p5a) = (
            self.asks[0].qty.max(1e-12),
            self.asks[4].qty.max(1e-12),
            self.asks[0].price,
            self.asks[4].price,
        );
        let (v1b, v5b, p1b, p5b) = (
            self.bids[0].qty.max(1e-12),
            self.bids[4].qty.max(1e-12),
            self.bids[0].price,
            self.bids[4].price,
        );
        self.slope_ask = if (p5a - p1a).abs() > 1e-12 { (v1a.ln() - v5a.ln()) / (p5a - p1a) } else { 0.0 };
        self.slope_bid = if (p5b - p1b).abs() > 1e-12 { (v1b.ln() - v5b.ln()) / (p5b - p1b) } else { 0.0 };
    }

    // ══ AŞAMA 2: EffDelta + Delta Velocity ═══════════════════
    pub fn update_eff_delta(&mut self, price: f64, qty: f64, sign: i8, ts_ms: u64) {
        let s_eff = 2.0 * (price - self.mid).abs();
        let s_bar = if self.avg_spread > 0.0 { self.avg_spread } else { s_eff.max(1e-12) };
        let delta_contribution = (sign as f64) * qty * (s_eff / s_bar);
        self.eff_delta += delta_contribution;

        // Saniyelik velocity
        let sec = ts_ms / 1000;
        if sec != self.last_delta_time {
            if self.eff_delta_hist.len() >= self.cfg.delta_window_sec {
                self.eff_delta_hist.pop_front();
            }
            self.eff_delta_hist.push_back(self.eff_delta);
            self.last_delta_time = sec;
        }
        if self.eff_delta_hist.len() >= 2 {
            let prev = *self.eff_delta_hist.get(self.eff_delta_hist.len() - 2).unwrap();
            let cur = *self.eff_delta_hist.back().unwrap();
            self.delta_velocity = cur - prev; // Δt = 1 sn
        }
    }

    // ══ AŞAMA 3: Absorption Ratio ════════════════════════════
    pub fn update_absorption(&mut self, qty: f64, sign: i8) {
        self.trade_signs.push_back((qty, sign));
        if self.trade_signs.len() > self.cfg.k_abs {
            self.trade_signs.pop_front();
        }
        let mut buy = 0.0;
        let mut sell = 0.0;
        for &(q, s) in &self.trade_signs {
            if s > 0 { buy += q; } else { sell += q; }
        }
        // Abs = pasif alım hacmi / agresif satış hacmi
        self.absorption = if sell > 0.0 { buy / sell } else { 0.0 };
    }

    // ══ AŞAMA 4: aVPIN ═══════════════════════════════════════
    pub fn update_avpin(&mut self, price: f64, qty: f64, sign: i8, ts_ms: u64) {
        // Parkinson H/L (son saniye içindeki max/min)
        let sec = ts_ms / 1000;
        if self.last_park_high == 0.0 {
            self.last_park_high = price;
            self.last_park_low = price;
        }
        if sec != self.last_delta_time {
            self.last_park_high = price;
            self.last_park_low = price;
        } else {
            self.last_park_high = self.last_park_high.max(price);
            self.last_park_low = self.last_park_low.min(price);
        }

        let h = self.last_park_high.max(price);
        let l = self.last_park_low.min(price);
        // Parkinson volatilitesi: sqrt(1/(4·ln2)) · sqrt(avg ln²(H/L))
        let parkinson = if h > 0.0 && l > 0.0 && h > l {
            let r = (h / l).ln();
            (1.0 / (4.0 * std::f64::consts::LN_2)).sqrt() * r.abs()
        } else {
            0.0
        };

        if sign > 0 {
            self.bucket_vbuy.push_back(qty);
        } else {
            self.bucket_vsell.push_back(qty);
        }
        if self.bucket_vbuy.len() > self.cfg.n_bucket {
            self.bucket_vbuy.pop_front();
        }
        if self.bucket_vsell.len() > self.cfg.n_bucket {
            self.bucket_vsell.pop_front();
        }

        // Ortalama trade hacmi (son 1000 trade, bucket listelerinden)
        let n_trades = (self.bucket_vbuy.len() + self.bucket_vsell.len()).max(1) as f64;
        let total_vol: f64 = self.bucket_vbuy.iter().sum::<f64>() + self.bucket_vsell.iter().sum::<f64>();
        let avg_vol = total_vol / n_trades;

        // Dinamik hacim bucket'ı: B_vol = α · σ_parkinson · V̄
        let b_vol = self.cfg.alpha_bucket * parkinson.max(1e-9) * avg_vol.max(1e-9);

        let sum_buy: f64 = self.bucket_vbuy.iter().sum();
        let sum_sell: f64 = self.bucket_vsell.iter().sum();
        let n = self.bucket_vbuy.len().max(self.bucket_vsell.len()).max(1) as f64;
        self.avpin = (sum_buy - sum_sell).abs() / (n * b_vol.max(1e-9));
    }

    // ══ AŞAMA 5: Hasbrouck VAR ═══════════════════════════════
    pub fn update_hasbrouck(&mut self, price: f64, qty: f64, sign: i8) {
        let r = price.ln() - self.prev_prev_price.ln().max(1e-12).ln();
        // Basitleştirme: r_t = ln(P_t) - ln(P_{t-1}); prev_price saklanır
        let r_prev = self.var_r.back().copied().unwrap_or(0.0);
        let x = (sign as f64) * qty;
        self.var_r.push_back(r);
        self.var_x.push_back(x);
        if self.var_r.len() > 200 {
            self.var_r.pop_front();
            self.var_x.pop_front();
        }

        if self.var_r.len() < 30 {
            return;
        }
        // OLS: r_t = α1·x_t + α2·r_{t-1} + ε
        let n = self.var_r.len();
        let (mut s_xx, mut s_xr, mut s_rr, mut s_yr, mut s_yx, mut s_yy) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for i in 1..n {
            let xi = self.var_x[i];
            let r_prev_i = self.var_r[i - 1];
            let yi = self.var_r[i];
            s_xx += xi * xi;
            s_xr += xi * r_prev_i;
            s_rr += r_prev_i * r_prev_i;
            s_yr += yi * r_prev_i;
            s_yx += yi * xi;
            s_yy += yi * yi;
        }
        let denom = s_xx * s_rr - s_xr * s_xr;
        if denom.abs() < 1e-15 {
            return;
        }
        let alpha1 = (s_yx * s_rr - s_yr * s_xr) / denom;
        let alpha2 = (s_yy * s_xx - s_yx * s_xr) / denom;
        // α2'yi regresyon katsayısı olarak düzelt (proxy)
        let _ = r_prev;
        self.permanent_impact = alpha1 / (1.0 - alpha2.max(-0.99).min(0.99)).max(1e-9);
        self.temporary_impact = self.var_r[n - 1] - alpha1 * self.var_x[n - 1] - alpha2 * self.var_r[n - 2];
    }

    // EfP: agresif trade / toplam L2 derinlik
    pub fn update_efp(&mut self, qty: f64) {
        self.efp = if self.last_depth_total > 0.0 { qty / self.last_depth_total } else { 0.0 };
    }

    // ══ AŞAMA 6: Alpha Basket ════════════════════════════════
    pub fn compute_signal(&mut self) -> i8 {
        // Z-skor standardizasyonu (ham değerler → normalize)
        let z_wlobi = (self.wlobi).tanh();
        let z_avpin = (self.avpin - 0.5) * 2.0;
        let z_abs = (self.absorption - 1.0).tanh();
        let z_effdelta = (self.eff_delta / 1000.0).tanh();
        let z_perm = (self.permanent_impact / 1e-6).tanh();

        // A_t = γ0 + γ1·(Abs-1) + γ2·(-WLOBI) + γ3·(0.7-aVPIN)
        //        + γ4·sign(-EffDelta)·1{|ΔV|<θ} - γ5·Perm
        let not_exhausted = (self.delta_velocity.abs() < self.cfg.theta_vol) as i32 as f64;
        let a = self.cfg.gamma[0]
            + self.cfg.gamma[1] * z_abs
            + self.cfg.gamma[2] * (-z_wlobi)
            + self.cfg.gamma[3] * (0.7 - z_avpin)
            + self.cfg.gamma[4] * (-z_effdelta).signum() * not_exhausted
            - self.cfg.gamma[5] * z_perm;

        self.alpha_score = a;
        self.p_long = 1.0 / (1.0 + (-a).exp());

        // Kesin karar kuralı
        if self.avpin >= 0.6 {
            self.signal = 0; // toksik akışta pasif kal
        } else if self.p_long > 0.65 {
            self.signal = 1;
        } else if self.p_long < 0.35 {
            self.signal = -1;
        } else {
            self.signal = 0;
        }
        self.signal
    }

    // Tüm metrikleri tek adımda tazele
    pub fn refresh(&mut self) {
        self.compute_wlobi();
        self.compute_slopes();
        self.compute_signal();
    }

    pub fn process_tick(&mut self, price: f64, qty: f64, is_buyer_maker: bool, ts_ms: u64) {
        let sign = self.lee_ready_sign(price);
        self.update_eff_delta(price, qty, sign, ts_ms);
        self.update_absorption(qty, sign);
        self.update_avpin(price, qty, sign, ts_ms);
        self.update_hasbrouck(price, qty, sign);
        self.update_efp(qty);
        let _ = is_buyer_maker; // Lee-Ready yönü is_buyer_maker'ı aşar (mid'e göre)
        self.refresh();
    }
}
