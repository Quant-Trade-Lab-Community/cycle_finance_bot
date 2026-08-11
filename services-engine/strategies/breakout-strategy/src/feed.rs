//! Flow ring'lerinden (RAM paylaşımlı bellek) türev veri toplayıcı.
//!
//! Kırılım algoritmasının girdilerini doldurur:
//! - trade ring → CVD (kümülatif hacim deltası, taker alım − satım)
//! - open_interest ring → OI / OI_prev
//! - funding ring → funding rate (+ son N event'ten μ/σ)
//! - markprice / lastprice ring → mark, last
//! - liquidations ring → liq_current (+ son M event'ten ort)
//!
//! CVD, `Feed` örneği yaşadığı sürece trade ring'inden sürekli biriktirilir.

use std::collections::VecDeque;

use rust_decimal::prelude::ToPrimitive;
use transport::events::EventType;
use transport::ring_buffer::GenerationalRingBuffer;
use transport::wire;

const FUNDING_WINDOW: usize = 20;
const LIQ_WINDOW: usize = 10;
const CVD_SAMPLES: usize = 20;

/// Bir poll anında toplanan türev veriler.
#[derive(Debug, Clone, Default)]
pub struct FeedSnapshot {
    pub oi: f64,
    pub oi_prev: f64,
    pub funding_rate: f64,
    pub funding_mean_20: f64,
    pub funding_std_20: f64,
    pub cvd_now: f64,
    pub cvd_prev_10: f64,
    pub cvd_sigma: f64,
    pub liq_current: f64,
    pub liq_avg: f64,
    pub mark: f64,
    pub last: f64,
}

/// Sürekli durum: CVD birikimi + örnek geçmişi.
pub struct Feed {
    trade_cursor: u64,
    cvd_accum: f64,
    cvd_samples: VecDeque<f64>,
    oi_prev_cached: f64,
    symbol: Vec<u8>,
}

fn pack_symbol(sym: &str) -> Vec<u8> {
    let mut b = vec![0u8; 16];
    let bytes = sym.as_bytes();
    let len = bytes.len().min(16);
    b[..len].copy_from_slice(&bytes[..len]);
    b
}

impl Feed {
    pub fn new(symbol: &str) -> Self {
        Self {
            trade_cursor: 0,
            cvd_accum: 0.0,
            cvd_samples: VecDeque::new(),
            oi_prev_cached: 0.0,
            symbol: pack_symbol(&symbol.to_uppercase()),
        }
    }

    /// Ring'leri okuyup `FeedSnapshot` üretir; CVD'yi trade ring'inden biriktirir.
    pub fn poll(&mut self) -> FeedSnapshot {
        self.accumulate_cvd();
        self.cvd_samples.push_back(self.cvd_accum);
        if self.cvd_samples.len() > CVD_SAMPLES {
            self.cvd_samples.pop_front();
        }

        let oi = latest_funding_field("/cycle_finance_open_interest", &self.symbol, false, &["open_interest"]);
        let oi_now = oi.unwrap_or(0.0);
        let oi_prev = if self.oi_prev_cached > 0.0 { self.oi_prev_cached } else { oi_now };
        self.oi_prev_cached = oi_now;

        let funding = latest_funding_field("/cycle_finance_funding", &self.symbol, false, &["funding_rate"]);
        let (funding_mean_20, funding_std_20) = funding_stats("/cycle_finance_funding", &self.symbol);

        let mark = latest_funding_field("/cycle_finance_markprice", &self.symbol, false, &["mark_price"]);
        let last = latest_funding_field("/cycle_finance_lastprice", &self.symbol, false, &["mark_price"]);

        let (liq_current, liq_avg) = liq_stats("/cycle_finance_liquidations", &self.symbol);

        let cvd_now = self.cvd_accum;
        let cvd_prev_10 = self.cvd_samples.iter().rev().nth(10).copied().unwrap_or(0.0);
        let cvd_sigma = std_dev(&self.cvd_samples.iter().copied().collect::<Vec<_>>());

        FeedSnapshot {
            oi: oi_now,
            oi_prev,
            funding_rate: funding.unwrap_or(0.0),
            funding_mean_20,
            funding_std_20,
            cvd_now,
            cvd_prev_10,
            cvd_sigma,
            liq_current,
            liq_avg,
            mark: mark.unwrap_or(0.0),
            last: last.unwrap_or(0.0),
        }
    }

    /// Trade ring'inden yeni event'leri okuyup CVD'yi günceller.
    fn accumulate_cvd(&mut self) {
        let ring = GenerationalRingBuffer::with_name("/cycle_finance_trades", 1);
        if self.trade_cursor == 0 {
            self.trade_cursor = ring.get_head();
            return;
        }
        let head = ring.get_head();
        if head <= self.trade_cursor {
            return;
        }
        // cursor'dan head'e kadar yalnızca kendi sembolümüzün trade'lerini işle.
        let mut cursor = self.trade_cursor;
        let max_scan = 200_000;
        let mut scanned = 0;
        while cursor < head && scanned < max_scan {
            if let Some(slot) = ring.read_slot(cursor) {
                if let Some(ev) = wire::decode(&slot.data[..slot.len as usize]) {
                    if ev.symbol == self.symbol[..] {
                        if let EventType::Trade { quantity, is_buyer_maker, .. } = ev.payload {
                            let q = quantity.to_f64().unwrap_or(0.0);
                            // m=true → alıcı maker (taker satıcı) → CVD −q
                            self.cvd_accum += if is_buyer_maker { -q } else { q };
                        }
                    }
                }
            }
            cursor += 1;
            scanned += 1;
        }
        self.trade_cursor = head;
    }
}

/// Belirtilen ring'deki son `FundingRate`/`OpenInterest` event'inden alanı okur.
/// `fields` ile istenen alan adı seçilir (mark_price, index_price, funding_rate, open_interest).
fn latest_funding_field(name: &str, sym: &[u8], _use_index: bool, fields: &[&str]) -> Option<f64> {
    let ring = GenerationalRingBuffer::with_name(name, 1);
    let head = ring.get_head();
    let start = head.saturating_sub(512);
    for seq in (start..head).rev() {
        if let Some(slot) = ring.read_slot(seq) {
            if let Some(ev) = wire::decode(&slot.data[..slot.len as usize]) {
                if ev.symbol == sym[..] {
                    let v = match ev.payload {
                        EventType::FundingRate { mark_price, index_price, funding_rate, .. } => {
                            if fields.contains(&"funding_rate") { funding_rate }
                            else if fields.contains(&"index_price") { index_price }
                            else { mark_price }
                        }
                        EventType::OpenInterest { open_interest, .. } => open_interest,
                        _ => continue,
                    };
                    let f = v.to_f64()?;
                    if f > 0.0 {
                        return Some(f);
                    }
                }
            }
        }
    }
    None
}

/// Funding ring'indeki son N event'in μ ve σ'sı (Z-skoru için).
fn funding_stats(name: &str, sym: &[u8]) -> (f64, f64) {
    let mut vals: Vec<f64> = Vec::new();
    let ring = GenerationalRingBuffer::with_name(name, 1);
    let head = ring.get_head();
    let start = head.saturating_sub(512);
    for seq in (start..head).rev() {
        if vals.len() >= FUNDING_WINDOW {
            break;
        }
        if let Some(slot) = ring.read_slot(seq) {
            if let Some(ev) = wire::decode(&slot.data[..slot.len as usize]) {
                if ev.symbol == sym[..] {
                    if let EventType::FundingRate { funding_rate, .. } = ev.payload {
                        if let Some(f) = funding_rate.to_f64() {
                            vals.push(f);
                        }
                    }
                }
            }
        }
    }
    if vals.is_empty() {
        return (0.0, 1.0);
    }
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
    (mean, var.sqrt().max(1e-12))
}

/// Likidasyon ring'indeki son M event'in (current, avg).
fn liq_stats(name: &str, sym: &[u8]) -> (f64, f64) {
    let mut vals: Vec<f64> = Vec::new();
    let ring = GenerationalRingBuffer::with_name(name, 1);
    let head = ring.get_head();
    let start = head.saturating_sub(1024);
    for seq in (start..head).rev() {
        if vals.len() >= LIQ_WINDOW {
            break;
        }
        if let Some(slot) = ring.read_slot(seq) {
            if let Some(ev) = wire::decode(&slot.data[..slot.len as usize]) {
                if ev.symbol == sym[..] {
                    if let EventType::Liquidation { quantity, .. } = ev.payload {
                        if let Some(f) = quantity.to_f64() {
                            vals.push(f);
                        }
                    }
                }
            }
        }
    }
    let current = vals.first().copied().unwrap_or(0.0);
    let avg = if vals.is_empty() { 0.0 } else { vals.iter().sum::<f64>() / vals.len() as f64 };
    (current, avg)
}

fn std_dev(vals: &[f64]) -> f64 {
    if vals.len() < 2 {
        return 1.0;
    }
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
    var.sqrt().max(1e-12)
}
