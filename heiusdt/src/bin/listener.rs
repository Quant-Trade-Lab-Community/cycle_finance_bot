//! LISTENER — DATA MERKEZİNDEN anlık mikro-yapı metrik analizi (Rust).
//!
//! Veri kaynağı: DATA MERKEZİ (core RUN_MODE=DATA → `/dev/shm/demir_yumruk_ring`).
//! price-feed KULLANILMAZ.
//!
//! Sistemde tanımlı HER sembol için tick-by-tick mikro-yapı metrikleri:
//!   - Lee-Ready Signing, WLOBI, Quote Slope
//!   - EffDelta, Delta Velocity, Absorption, aVPIN
//!   - Hasbrouck Kalıcı/Geçici Etki, EfP, Alpha Basket sinyali
//!
//! Çıktılar: konsol tablosu + /tmp/listener_metrics.json

use heiusdt::metrics::{DepthLevel, SymbolMetrics};
use rust_decimal::prelude::ToPrimitive;

use proje_core::memory::ring_buffer::GenerationalRingBuffer;
use proje_core::ring_buffer::EventType;
use proje_core::tick::EventParser;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

const OUT_FILE: &str = "/tmp/listener_metrics.json";
const REFRESH_MS: u64 = 2000;

fn decode_symbol(buf: &[u8; 16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(16);
    String::from_utf8_lossy(&buf[..len]).to_string().to_uppercase()
}

fn main() {
    println!("{}", "═".repeat(84));
    println!("  🛰️  LISTENER — DATA MERKEZİ MİKRO-YAPI METRİKLERİ");
    println!("  Kaynak: /dev/shm/demir_yumruk_ring (core RUN_MODE=DATA)");
    println!("{}", "═".repeat(84));

    let ring = Arc::new(GenerationalRingBuffer::new(160_000));
    let mut cursor = ring.get_head();
    let mut symbols: HashMap<String, SymbolMetrics> = HashMap::new();

    // Sembol seti: alerts.toml'dan
    let known: Vec<String> = load_symbols();

    let mut last_render = std::time::Instant::now();
    let mut tick_count: u64 = 0;
    let mut depth_count: u64 = 0;

    loop {
        if let Some(slot) = ring.read_slot(cursor) {
            let mut data = slot.data[..slot.len as usize].to_vec();
            if let Some(event) = EventParser::parse(&mut data) {
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
            render(&symbols, tick_count, depth_count);
            tick_count = 0;
            depth_count = 0;
            last_render = std::time::Instant::now();
        }
    }
}

fn load_symbols() -> Vec<String> {
    let mut syms: Vec<String> = Vec::new();
    if let Ok(content) = std::fs::read_to_string("/home/smhvz/Desktop/PROJE/alerts.toml") {
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

fn render(symbols: &HashMap<String, SymbolMetrics>, ticks: u64, depth: u64) {
    print!("\x1b[2J\x1b[H");
    println!("{}", "═".repeat(84));
    println!("  🛰️  LISTENER — DATA MERKEZİ MİKRO-YAPI METRİKLERİ");
    println!("  Kaynak: /dev/shm/demir_yumruk_ring | tick/s: {ticks} | depth/s: {depth}");
    println!("{}", "═".repeat(84));

    if symbols.is_empty() {
        println!("  📭 VERİ BEKLENİYOR — DATA terminali (RUN_MODE=DATA) çalışıyor mu?");
        return;
    }

    println!("  {:<9}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>7}{:>8}{:>8}",
        "SEMBOL", "WLOBI", "SLP_ASK", "EFFΔ", "ΔV", "ABS", "aVPIN", "PERM", "EfP", "P(LONG)", "SİNYAL");
    println!("  {}", "-".repeat(82));

    let mut rows: Vec<(&String, &SymbolMetrics)> = symbols.iter().collect();
    rows.sort_by_key(|(k, _)| k.clone());

    for (sym, m) in rows {
        let signal = match m.signal {
            1 => "▲ LONG",
            -1 => "▼ SHORT",
            _ => "· NÖTR",
        };
        println!(
            "  {:<9}{:>8.3}{:>8.2}{:>8.2}{:>8.2}{:>8.2}{:>8.3}{:>8.2e}{:>7.3}{:>8.3}{:>8}",
            sym, m.wlobi, m.slope_ask, m.eff_delta, m.delta_velocity,
            m.absorption, m.avpin, m.permanent_impact, m.efp, m.p_long, signal
        );
    }
    println!("{}", "-".repeat(82));
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    println!("  Son güncelleme: {now} | Metrikler: Lee-Ready, WLOBI, EffΔ, aVPIN, Hasbrouck, EfP");

    // ── JSON çıktısı ──
    let mut out = serde_json::Map::new();
    for (sym, m) in &*symbols {
        out.insert(sym.clone(), json!({
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
