//! Compact typed-binary market data frame codec.
//!
//! RAM hot path'teki ring buffer (`/dev/shm`) ham JSON yerine bu compact
//! binary formatı saklar:
//!
//! ```text
//! [0]    tag: u8
//! [1..17] symbol: [u8;16]
//! ... per-tag alanlar (i64 mantissa + u8 scale)
//! ```
//!
//! Ondalıklı değerler `rust_decimal::Decimal`'ın `(mantissa, scale)` ikilisi
//! olarak saklanır; `Decimal::new(mantissa, scale)` ile birebir geri kurulur.
//! Kısıt: |mantissa| <= i64::MAX — kripto fiyat/miktar aralığında imkânsız.
//!
//! Boyutlar: Trade 44B · BookTicker 53B · Funding 52B · Liquidation 44B ·
//! OI 34B · Depth20 659B (JSON ~1100B).
//!
//! ## Tag'ler
//! 0=Trade · 1=Depth · 2=Funding · 3=BookTicker · 4=Liquidation · 5=OpenInterest

use crate::events::{EventType, OwnedEvent};
use rust_decimal::Decimal;

/// Depth20 frame boyutu: tag(1)+sym(16)+p_scale(1)+q_scale(1)+40*(8+8)=659
pub const DEPTH_FRAME_SIZE: usize = 1 + 16 + 1 + 1 + 40 * 16;
/// En büyük frame boyutu (tüm tipler bunun içinde).
pub const MAX_FRAME_SIZE: usize = DEPTH_FRAME_SIZE;

const TAG_TRADE: u8 = 0;
const TAG_DEPTH: u8 = 1;
const TAG_FUNDING: u8 = 2;
const TAG_BOOKTICKER: u8 = 3;
const TAG_LIQUIDATION: u8 = 4;
const TAG_OPEN_INTEREST: u8 = 5;
const TAG_OPPORTUNITY: u8 = 6;
const TAG_SYMBOL_METRICS: u8 = 7;

#[inline(always)]
fn put_u8(buf: &mut [u8], off: usize, v: u8) {
    buf[off] = v;
}

#[inline(always)]
fn put_i64(buf: &mut [u8], off: usize, v: i64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

#[inline(always)]
fn put_u64(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

#[inline(always)]
fn rd_u8(buf: &[u8], off: usize) -> u8 {
    buf[off]
}

#[inline(always)]
fn rd_i64(buf: &[u8], off: usize) -> i64 {
    i64::from_le_bytes(buf[off..off + 8].try_into().unwrap())
}

#[inline(always)]
fn rd_u64(buf: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(buf[off..off + 8].try_into().unwrap())
}

#[inline(always)]
fn write_decimal(buf: &mut [u8], off: usize, d: Decimal) -> Option<usize> {
    let mantissa = d.mantissa();
    let m = i64::try_from(mantissa).ok()?;
    put_i64(buf, off, m);
    put_u8(buf, off + 8, d.scale() as u8);
    Some(off + 9)
}

#[inline(always)]
fn read_decimal(buf: &[u8], off: usize) -> Decimal {
    let m = rd_i64(buf, off);
    let s = rd_u8(buf, off + 8);
    if m == 0 && s == 0 {
        Decimal::ZERO
    } else {
        Decimal::new(m, s as u32)
    }
}

/// `OwnedEvent`'i compact binary frame'e yazar; boyutu döner.
/// Buffer `MAX_FRAME_SIZE`'dan büyük olmalı; mantissa i64 taşarsa `None`.
pub fn encode(ev: &OwnedEvent, buf: &mut [u8]) -> Option<usize> {
    buf[1..17].copy_from_slice(&ev.symbol);

    match &ev.payload {
        EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
            put_u8(buf, 0, TAG_TRADE);
            let mut off = 17;
            off = write_decimal(buf, off, *price)?;
            off = write_decimal(buf, off, *quantity)?;
            put_u64(buf, off, *timestamp);
            put_u8(buf, off + 8, if *is_buyer_maker { 1 } else { 0 });
            Some(off + 9)
        }
        EventType::Orderbook { bids, asks } => {
            put_u8(buf, 0, TAG_DEPTH);
            let mut off = 17;
            // Ortak scale'ler — rescale kayıpsız (değer korunur).
            let p_scale = bids.iter().chain(asks.iter())
                .filter(|(p, _)| !p.is_zero())
                .map(|(p, _)| p.scale())
                .max()
                .unwrap_or(0);
            let q_scale = bids.iter().chain(asks.iter())
                .filter(|(_, q)| !q.is_zero())
                .map(|(_, q)| q.scale())
                .max()
                .unwrap_or(0);
            put_u8(buf, off, p_scale as u8);
            put_u8(buf, off + 1, q_scale as u8);
            off += 2;
            for (p, q) in bids.iter().chain(asks.iter()) {
                let pm = if p.is_zero() {
                    0i64
                } else {
                    let mut d = *p;
                    d.rescale(p_scale);
                    i64::try_from(d.mantissa()).ok()?
                };
                let qm = if q.is_zero() {
                    0i64
                } else {
                    let mut d = *q;
                    d.rescale(q_scale);
                    i64::try_from(d.mantissa()).ok()?
                };
                put_i64(buf, off, pm);
                put_i64(buf, off + 8, qm);
                off += 16;
            }
            Some(off)
        }
        EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time } => {
            put_u8(buf, 0, TAG_FUNDING);
            let mut off = 17;
            off = write_decimal(buf, off, *mark_price)?;
            off = write_decimal(buf, off, *index_price)?;
            off = write_decimal(buf, off, *funding_rate)?;
            put_u64(buf, off, *next_funding_time);
            Some(off + 8)
        }
        EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty } => {
            put_u8(buf, 0, TAG_BOOKTICKER);
            let mut off = 17;
            off = write_decimal(buf, off, *best_bid_price)?;
            off = write_decimal(buf, off, *best_bid_qty)?;
            off = write_decimal(buf, off, *best_ask_price)?;
            off = write_decimal(buf, off, *best_ask_qty)?;
            Some(off)
        }
        EventType::Liquidation { side, price, quantity, timestamp } => {
            put_u8(buf, 0, TAG_LIQUIDATION);
            put_u8(buf, 17, *side);
            let mut off = 18;
            off = write_decimal(buf, off, *price)?;
            off = write_decimal(buf, off, *quantity)?;
            put_u64(buf, off, *timestamp);
            Some(off + 8)
        }
        EventType::OpenInterest { open_interest, timestamp } => {
            put_u8(buf, 0, TAG_OPEN_INTEREST);
            let mut off = 17;
            off = write_decimal(buf, off, *open_interest)?;
            put_u64(buf, off, *timestamp);
            Some(off + 8)
        }
        EventType::Opportunity { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict } => {
            put_u8(buf, 0, TAG_OPPORTUNITY);
            let mut off = 17;
            off = write_decimal(buf, off, *score)?;
            off = write_decimal(buf, off, *efficiency)?;
            off = write_decimal(buf, off, *price_bps_per_s)?;
            off = write_decimal(buf, off, *price_ticks_per_s)?;
            off = write_decimal(buf, off, *ob_changes_per_s)?;
            off = write_decimal(buf, off, *spread_bps)?;
            put_u8(buf, off, *verdict);
            Some(off + 1)
        }
        EventType::SymbolMetrics { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps } => {
            put_u8(buf, 0, TAG_SYMBOL_METRICS);
            let mut off = 17;
            off = write_decimal(buf, off, *score)?;
            off = write_decimal(buf, off, *efficiency)?;
            off = write_decimal(buf, off, *price_bps_per_s)?;
            off = write_decimal(buf, off, *price_ticks_per_s)?;
            off = write_decimal(buf, off, *ob_changes_per_s)?;
            off = write_decimal(buf, off, *spread_bps)?;
            Some(off)
        }
    }
}

/// Compact binary frame'i `OwnedEvent`'e geri kurar. Bozuk/güdük frame'de `None`.
pub fn decode(buf: &[u8]) -> Option<OwnedEvent> {
    if buf.len() < 17 {
        return None;
    }
    let tag = buf[0];
    let symbol = buf[1..17].try_into().ok()?;

    match tag {
        TAG_TRADE => {
            if buf.len() < 44 {
                return None;
            }
            let price = read_decimal(buf, 17);
            let quantity = read_decimal(buf, 26);
            let timestamp = rd_u64(buf, 35);
            let is_buyer_maker = rd_u8(buf, 43) != 0;
            Some(OwnedEvent {
                symbol,
                payload: EventType::Trade { price, quantity, timestamp, is_buyer_maker },
            })
        }
        TAG_DEPTH => {
            if buf.len() < DEPTH_FRAME_SIZE {
                return None;
            }
            let p_scale = rd_u8(buf, 17);
            let q_scale = rd_u8(buf, 18);
            let mut bids = [(Decimal::ZERO, Decimal::ZERO); 20];
            let mut asks = [(Decimal::ZERO, Decimal::ZERO); 20];
            let mut off = 19;
            for i in 0..20 {
                let pm = rd_i64(buf, off);
                let qm = rd_i64(buf, off + 8);
                if pm != 0 {
                    bids[i].0 = Decimal::new(pm, p_scale as u32);
                }
                if qm != 0 {
                    bids[i].1 = Decimal::new(qm, q_scale as u32);
                }
                off += 16;
            }
            for i in 0..20 {
                let pm = rd_i64(buf, off);
                let qm = rd_i64(buf, off + 8);
                if pm != 0 {
                    asks[i].0 = Decimal::new(pm, p_scale as u32);
                }
                if qm != 0 {
                    asks[i].1 = Decimal::new(qm, q_scale as u32);
                }
                off += 16;
            }
            Some(OwnedEvent {
                symbol,
                payload: EventType::Orderbook { bids, asks },
            })
        }
        TAG_FUNDING => {
            if buf.len() < 52 {
                return None;
            }
            let mark_price = read_decimal(buf, 17);
            let index_price = read_decimal(buf, 26);
            let funding_rate = read_decimal(buf, 35);
            let next_funding_time = rd_u64(buf, 44);
            Some(OwnedEvent {
                symbol,
                payload: EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time },
            })
        }
        TAG_BOOKTICKER => {
            if buf.len() < 53 {
                return None;
            }
            let best_bid_price = read_decimal(buf, 17);
            let best_bid_qty = read_decimal(buf, 26);
            let best_ask_price = read_decimal(buf, 35);
            let best_ask_qty = read_decimal(buf, 44);
            Some(OwnedEvent {
                symbol,
                payload: EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty },
            })
        }
        TAG_LIQUIDATION => {
            if buf.len() < 44 {
                return None;
            }
            let side = rd_u8(buf, 17);
            let price = read_decimal(buf, 18);
            let quantity = read_decimal(buf, 27);
            let timestamp = rd_u64(buf, 36);
            Some(OwnedEvent {
                symbol,
                payload: EventType::Liquidation { side, price, quantity, timestamp },
            })
        }
        TAG_OPEN_INTEREST => {
            if buf.len() < 34 {
                return None;
            }
            let open_interest = read_decimal(buf, 17);
            let timestamp = rd_u64(buf, 26);
            Some(OwnedEvent {
                symbol,
                payload: EventType::OpenInterest { open_interest, timestamp },
            })
        }
        TAG_OPPORTUNITY => {
            if buf.len() < 72 {
                return None;
            }
            let score = read_decimal(buf, 17);
            let efficiency = read_decimal(buf, 26);
            let price_bps_per_s = read_decimal(buf, 35);
            let price_ticks_per_s = read_decimal(buf, 44);
            let ob_changes_per_s = read_decimal(buf, 53);
            let spread_bps = read_decimal(buf, 62);
            let verdict = rd_u8(buf, 71);
            Some(OwnedEvent {
                symbol,
                payload: EventType::Opportunity {
                    score,
                    efficiency,
                    price_bps_per_s,
                    price_ticks_per_s,
                    ob_changes_per_s,
                    spread_bps,
                    verdict,
                },
            })
        }
        TAG_SYMBOL_METRICS => {
            if buf.len() < 71 {
                return None;
            }
            let score = read_decimal(buf, 17);
            let efficiency = read_decimal(buf, 26);
            let price_bps_per_s = read_decimal(buf, 35);
            let price_ticks_per_s = read_decimal(buf, 44);
            let ob_changes_per_s = read_decimal(buf, 53);
            let spread_bps = read_decimal(buf, 62);
            Some(OwnedEvent {
                symbol,
                payload: EventType::SymbolMetrics {
                    score,
                    efficiency,
                    price_bps_per_s,
                    price_ticks_per_s,
                    ob_changes_per_s,
                    spread_bps,
                },
            })
        }
        _ => None,
    }
}

