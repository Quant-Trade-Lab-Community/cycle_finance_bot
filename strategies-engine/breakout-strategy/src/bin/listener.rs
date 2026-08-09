//! LISTENER — DATA MERKEZİ mikro-yapı metrikleri + korelasyon tabloları (Rust).
//!
//! Veri kaynakları:
//!   - DATA MERKEZİ (core RUN_MODE=DATA → `/dev/shm/cycle_finance_ring`): trade/depth + hacim
//!   - PRICE-FEED (:3004): lastprice (fiyat korelasyonu için)
//!
//! Ekran:
//!   1. Mikro-yapı metrik tablosu (TPS, WLOBI, EffΔ, aVPIN, Hasbrouck, EfP, sinyal)
//!   2. Fiyat korelasyon tablosu (price-feed lastprice, N sn pencere, normalize 0-1)
//!   3. Hacim korelasyon tablosu (DATA trade hacmi, N sn pencere, normalize 0-1)
//!
//! Pencere süreleri shell'den ayarlanabilir (listenconfig-set):
//!   corr_price_window_sec, corr_vol_window_sec
//!
//! Çıktılar: konsol + /tmp/listener_metrics.json

use breakout_strategy::metrics::{normalized_corr, CorrSeries, DepthLevel, SymbolMetrics};
use rust_decimal::prelude::ToPrimitive;

use transport::ring_buffer::GenerationalRingBuffer;
use transport::events::EventType;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const OUT_FILE: &str = "/tmp/listener_metrics.json";
const REFRESH_MS: u64 = 2000;
const PRICE_FEED_URL: &str = "http://127.0.0.1:3004";

fn decode_symbol(buf: &[u8; 16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(16);
    String::from_utf8_lossy(&buf[..len]).to_string().to_uppercase()
}

/// price-feed'ten periyodik lastprice çeker ve CorrSeries'e yazar.
fn spawn_price_corr_thread(symbols: Vec<String>, series: Arc<Mutex<HashMap<String, CorrSeries>>>) {
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .expect("reqwest client");
        loop {
            let url = format!("{PRICE_FEED_URL}/api/lastprice");
            if let Ok(resp) = client.get(&url).send() {
                if let Ok(v) = resp.json::<serde_json::Value>() {
                    if let Some(prices) = v.get("prices").and_then(|p| p.as_object()) {
                        let now = now_ms();
                        let mut s = series.lock().unwrap();
                        for sym in &symbols {
                            if let Some(p) = prices.get(sym).and_then(|x| x.get("last")).and_then(|x| x.as_f64()) {
                                let e = s.entry(sym.clone()).or_insert_with(|| CorrSeries::new(5));
                                e.push(now, p);
                            }
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    });
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn main() {
    println!("{}", "═".repeat(96));
    println!("  🛰️  LISTENER — MİKRO-YAPI METRİKLERİ + KORELASYON");
    println!("  Kaynak: DATA (/dev/shm/cycle_finance_ring) + PRICE-FEED (:3004)");
    println!("{}", "═".repeat(96));

    let ring = Arc::new(GenerationalRingBuffer::new(160_000));
    let mut cursor = ring.get_head();
    let mut symbols: HashMap<String, SymbolMetrics> = HashMap::new();

    let known: Vec<String> = load_symbols();

    // Fiyat korelasyon serileri (price-feed)
    let price_series: Arc<Mutex<HashMap<String, CorrSeries>>> = Arc::new(Mutex::new(HashMap::new()));
    spawn_price_corr_thread(known.clone(), price_series.clone());

    // Hacim korelasyon serileri (DATA trade) — sembol → (pencere, değer)
    let vol_series: Arc<Mutex<HashMap<String, CorrSeries>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut last_render = std::time::Instant::now();
    let mut tick_count: u64 = 0;
    let mut depth_count: u64 = 0;

    loop {
        if let Some(slot) = ring.read_slot(cursor) {
            if let Some(event) = transport::wire::decode(&slot.data[..slot.len as usize]) {
                let sym = decode_symbol(&event.symbol);
                if !known.iter().any(|k| k == &sym) {
                    cursor += 1;
                    continue;
                }
                let m = symbols.entry(sym.clone()).or_default();

                match event.payload {
                    EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
                        let p = price.to_f64().unwrap_or(0.0);
                        let q = quantity.to_f64().unwrap_or(0.0);
                        m.process_tick(p, q, is_buyer_maker, timestamp);
                        // Hacim korelasyonu: trade hacmini pencereye ekle (biriken değer)
                        {
                            let mut vs = vol_series.lock().unwrap();
                            let e = vs.entry(sym.clone()).or_insert_with(|| CorrSeries::new(5));
                            e.push(now_ms(), q);
                        }
                        tick_count += 1;
                    }
                    EventType::Orderbook { bids, asks } => {
                        let bids_l: Vec<DepthLevel> = bids.iter().take(5)
                            .map(|(p, q)| DepthLevel { price: p.to_f64().unwrap_or(0.0), qty: q.to_f64().unwrap_or(0.0) })
                            .collect();
                        let asks_l: Vec<DepthLevel> = asks.iter().take(5)
                            .map(|(p, q)| DepthLevel { price: p.to_f64().unwrap_or(0.0), qty: q.to_f64().unwrap_or(0.0) })
                            .collect();
                        depth_count += 1;
                        m.update_depth(&bids_l, &asks_l);
                        m.refresh();
                    }
                    _ => {}
                }
            }
            cursor += 1;
        } else {
            std::thread::sleep(Duration::from_micros(50));
        }

        if last_render.elapsed().as_millis() as u64 >= REFRESH_MS {
            for m in symbols.values_mut() {
                m.reload_config();
                // korelasyon pencere sürelerini uygula
                let (pw, vw) = (m.cfg.corr_price_window_sec, m.cfg.corr_vol_window_sec);
                {
                    let mut ps = price_series.lock().unwrap();
                    for e in ps.values_mut() {
                        e.set_window(pw);
                    }
                }
                {
                    let mut vs = vol_series.lock().unwrap();
                    for e in vs.values_mut() {
                        e.set_window(vw);
                    }
                }
            }
            render(&symbols, &price_series, &vol_series, tick_count, depth_count);
            tick_count = 0;
            depth_count = 0;
            last_render = std::time::Instant::now();
        }
    }
}

fn load_symbols() -> Vec<String> {
    let mut syms: Vec<String> = Vec::new();
    if let Ok(content) = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../alerts.toml")) {
        for line in content.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("symbol") {
                if let Some(eq) = rest.find('=') {
                    let s = rest[eq + 1..].trim().trim_matches('"').trim_matches('\'').trim().to_string();
                    if !s.is_empty() && !syms.contains(&s) {
                        syms.push(s);
                    }
                }
            }
        }
    }
    if !syms.contains(&"HEIUSDT".to_string()) {
        syms.push("HEIUSDT".to_string());
    }
    syms
}

/// Fiyat/hacim korelasyon matrisini çizer (normalize 0-1).
fn render_corr(title: &str, symbols: &[String], series: &Arc<Mutex<HashMap<String, CorrSeries>>>) {
    let s = series.lock().unwrap();
    println!("  {title}");
    println!("  {:<9}", "");
    for sym in symbols {
        print!("{:>10}", short(sym));
    }
    println!();
    for a in symbols {
        print!("  {:<9}", short(a));
        let av = s.get(a).map(|x| x.values()).unwrap_or_default();
        for b in symbols {
            let bv = s.get(b).map(|x| x.values()).unwrap_or_default();
            let c = normalized_corr(&av, &bv);
            print!("{:>10.2}", c);
        }
        println!();
    }
    println!();
}

fn short(s: &str) -> String {
    s.trim_end_matches("USDT").to_string()
}

fn render(symbols: &HashMap<String, SymbolMetrics>,
          price_series: &Arc<Mutex<HashMap<String, CorrSeries>>>,
          vol_series: &Arc<Mutex<HashMap<String, CorrSeries>>>,
          ticks: u64, depth: u64) {
    print!("\x1b[2J\x1b[H");
    println!("{}", "═".repeat(96));
    println!("  🛰️  LISTENER — MİKRO-YAPI METRİKLERİ + KORELASYON");
    println!("  DATA tick/s: {ticks} | depth/s: {depth} | price-feed: :3004");
    println!("{}", "═".repeat(96));

    if symbols.is_empty() {
        println!("  📭 VERİ BEKLENİYOR — DATA terminali çalışıyor mu?");
        return;
    }

    // ── Mikro-yapı metrik tablosu ──
    println!("  {:<9}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>7}{:>8}{:>8}{:>8}",
        "SEMBOL", "TPS", "WLOBI", "SLP", "EFFΔ", "ΔV", "ABS", "aVPIN", "PERM", "EfP", "P(LONG)", "SİNYAL");
    println!("  {}", "-".repeat(96));
    let mut rows: Vec<(&String, &SymbolMetrics)> = symbols.iter().collect();
    rows.sort_by_key(|(k, _)| k.clone());
    for (sym, m) in rows {
        let signal = match m.signal {
            1 => "▲ LONG",
            -1 => "▼ SHORT",
            _ => "· NÖTR",
        };
        println!(
            "  {:<9}{:>8.1}{:>8.3}{:>8.2}{:>8.2}{:>8.2}{:>8.2}{:>8.3}{:>8.1e}{:>7.3}{:>8.3}{:>8}",
            sym, m.tps, m.wlobi, m.slope_ask, m.eff_delta, m.delta_velocity,
            m.absorption, m.avpin, m.permanent_impact, m.efp, m.p_long, signal
        );
    }
    println!();

    // ── Fiyat korelasyonu (price-feed lastprice) ──
    let sym_list: Vec<String> = {
        let mut v: Vec<String> = symbols.keys().cloned().collect();
        v.sort();
        v
    };
    render_corr(&format!("📈 FİYAT KORELASYONU (price-feed lastprice)"),
                &sym_list, price_series);
    render_corr(&format!("📊 HACİM KORELASYONU (DATA trade hacmi)"),
                &sym_list, vol_series);

    println!("{}", "-".repeat(96));
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    println!("  Son güncelleme: {now} | listenconfig-set corr_price_window_sec / corr_vol_window_sec ile pencere değiştir");

    // ── JSON çıktısı ──
    let mut out = serde_json::Map::new();
    for (sym, m) in &*symbols {
        out.insert(sym.clone(), json!({
            "tps": m.tps,
            "wlobi": m.wlobi,
            "slope_ask": m.slope_ask,
            "slope_bid": m.slope_bid,
            "eff_delta": m.eff_delta,
            "delta_velocity": m.delta_velocity,
            "absorption": m.absorption,
            "idm": m.idm,
            "avpin": m.avpin,
            "permanent_impact": m.permanent_impact,
            "temporary_impact": m.temporary_impact,
            "efp": m.efp,
            "alpha_score": m.alpha_score,
            "p_long": m.p_long,
            "signal": m.signal,
        }));
    }
    let doc = json!({ "timestamp": now, "metrics": out });
    let _ = std::fs::write(OUT_FILE, serde_json::to_string_pretty(&doc).unwrap_or_default());
}
