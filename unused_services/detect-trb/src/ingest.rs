// ============================================================================
// detect-trb — VERİ KATMANI: CORE DATA MERKEZİ
// ============================================================================
// İki kaynak:
//   1. SQLite (market_data.db) → tarihsel tick'ler → OHLCV gruplandırma
//   2. GenerationalRingBuffer (/dev/shm/cycle_finance_ring) → canlı tick'ler
//
// Her iki kaynaktan elde edilen InflowData dizisi NSSolver'a beslenir.
// ============================================================================

use std::collections::BTreeMap;

use rusqlite::{Connection, params};
use tracing::{warn, info};

use transport::ring_buffer::GenerationalRingBuffer;
use transport::wire;
use transport::events::EventType;

use crate::types::{FluidError, FluidResult, InflowData};

// ================================================================
// BÖLÜM 1: SQLite → Tarihsel InflowData
// ================================================================

/// SQLite'tan son `limit` adet trade tick'ini çeker ve
/// `interval_ms` aralıklarına gruplandırarak `InflowData` dizisi döner.
///
/// Aynı zamanda liquidation, funding_rate ve open_interest tablolarını da okur.
pub fn load_from_sqlite(
    db_path: &str,
    symbol: &str,
    interval_ms: u64,
    limit: usize,
) -> FluidResult<Vec<InflowData>> {
    let conn = Connection::open(db_path)
        .map_err(|e| FluidError::DbError(e.to_string()))?;

    // ── Trade tick'leri ──────────────────────────────────────────────────
    let mut stmt = conn
        .prepare(
            "SELECT price, quantity, timestamp FROM trades \
             WHERE symbol = ?1 \
             ORDER BY timestamp DESC \
             LIMIT ?2",
        )
        .map_err(|e| FluidError::DbError(e.to_string()))?;

    let trades: Vec<(f64, f64, u64)> = stmt
        .query_map(params![symbol, limit as i64 * 10], |row| {
            Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?, row.get::<_, u64>(2)?))
        })
        .map_err(|e| FluidError::DbError(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    if trades.is_empty() {
        warn!(symbol = symbol, "SQLite'ta trade verisi bulunamadı");
        return Err(FluidError::DataStall);
    }

    // ── Liquidation tick'leri ─────────────────────────────────────────────
    let liq_map = load_liquidations(&conn, symbol, limit)?;

    // ── Funding rate ──────────────────────────────────────────────────────
    let funding_rate = load_latest_funding(&conn, symbol)?;

    // ── Open Interest deltas ──────────────────────────────────────────────
    let oi_delta = load_oi_delta(&conn, symbol)?;

    // ── OHLCV Gruplandırma ────────────────────────────────────────────────
    let inflows = aggregate_to_inflows(trades, &liq_map, funding_rate, oi_delta, interval_ms, limit);

    info!(
        symbol = symbol,
        steps = inflows.len(),
        "SQLite'tan inflow adımları yüklendi"
    );

    Ok(inflows)
}

/// Liquidation tablosundan timestamp bazlı hacim haritası oluştur
fn load_liquidations(
    conn: &Connection,
    symbol: &str,
    limit: usize,
) -> FluidResult<BTreeMap<u64, f64>> {
    let mut map = BTreeMap::new();
    let mut stmt = match conn.prepare(
        "SELECT price, quantity, timestamp FROM liquidations \
         WHERE symbol = ?1 ORDER BY timestamp DESC LIMIT ?2",
    ) {
        Ok(s) => s,
        Err(e) => {
            warn!("liquidations sorgusu hazırlanamadı: {}", e);
            return Ok(map);
        }
    };

    let _ = stmt
        .query_map(params![symbol, limit as i64], |row| {
            Ok((
                row.get::<_, f64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, u64>(2)?,
            ))
        })
        .map(|rows| {
            for r in rows.flatten() {
                let (price, qty, ts) = r;
                *map.entry(ts).or_insert(0.0) += price * qty;
            }
        });

    Ok(map)
}

/// En güncel funding rate değerini çek
fn load_latest_funding(conn: &Connection, symbol: &str) -> FluidResult<f64> {
    let rate: f64 = conn
        .query_row(
            "SELECT funding_rate FROM funding_rates WHERE symbol = ?1 ORDER BY id DESC LIMIT 1",
            params![symbol],
            |row| row.get(0),
        )
        .unwrap_or(0.0);
    Ok(rate)
}

/// Open Interest delta: son iki kayıt arasındaki fark
fn load_oi_delta(conn: &Connection, symbol: &str) -> FluidResult<f64> {
    let mut stmt = match conn.prepare(
        "SELECT open_interest FROM open_interests WHERE symbol = ?1 ORDER BY id DESC LIMIT 2",
    ) {
        Ok(s) => s,
        Err(_) => return Ok(0.0),
    };

    let ois: Vec<f64> = stmt
        .query_map(params![symbol], |row| row.get::<_, f64>(0))
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default();

    let delta = match ois.as_slice() {
        [newest, oldest] => newest - oldest,
        _ => 0.0,
    };
    Ok(delta)
}

/// Trade tick'lerini zaman aralıklarına göre grupla → InflowData dizisi üret
fn aggregate_to_inflows(
    mut trades: Vec<(f64, f64, u64)>,
    liq_map: &BTreeMap<u64, f64>,
    funding_rate: f64,
    oi_delta: f64,
    interval_ms: u64,
    limit: usize,
) -> Vec<InflowData> {
    // Eskiden yeniye sırala
    trades.sort_by_key(|(_, _, ts)| *ts);

    if trades.is_empty() {
        return vec![];
    }

    let t_start = trades.first().map(|(_, _, ts)| *ts).unwrap_or(0);
    let t_end   = trades.last().map(|(_, _, ts)| *ts).unwrap_or(0);

    if interval_ms == 0 || t_end <= t_start {
        return vec![];
    }

    // Kaç bucket?
    let n_buckets = ((t_end - t_start) / interval_ms + 1).min(limit as u64) as usize;
    let mut inflows: Vec<InflowData> = Vec::with_capacity(n_buckets);

    for b in 0..n_buckets {
        let bucket_start = t_start + b as u64 * interval_ms;
        let bucket_end   = bucket_start + interval_ms;

        // Bu aralıktaki trade'ler
        let bucket_trades: Vec<_> = trades
            .iter()
            .filter(|(_, _, ts)| *ts >= bucket_start && *ts < bucket_end)
            .collect();

        if bucket_trades.is_empty() {
            continue;
        }

        // Hacim ağırlıklı ortalama fiyat (VWAP)
        let total_vol: f64 = bucket_trades.iter().map(|(_, q, _)| q).sum();
        let vwap = if total_vol > 0.0 {
            bucket_trades.iter().map(|(p, q, _)| p * q).sum::<f64>() / total_vol
        } else {
            bucket_trades.last().map(|(p, _, _)| *p).unwrap_or(0.0)
        };

        // Tasfiye hacmi bu bucket aralığında
        let liq_vol: f64 = liq_map
            .range(bucket_start..bucket_end)
            .map(|(_, v)| v)
            .sum();

        inflows.push(InflowData {
            price: vwap,
            volume: total_vol,
            oi_delta,
            funding_rate,
            buy_sell_ratio: 0.5, // orderbook olmadan varsayılan
            liquidation_volume: liq_vol,
            timestamp_ms: bucket_start,
        });
    }

    inflows
}

// ================================================================
// BÖLÜM 2: GenerationalRingBuffer → Canlı InflowData
// ================================================================

/// Ring buffer'ın son `max_ticks` tick'ini okur ve
/// sembol filtresiyle InflowData üretir.
///
/// Ring buffer /dev/shm/cycle_finance_ring üzerinde yazar.
/// Bu fonksiyon core ring buffer'ı salt okunur şekilde tüketir.
pub fn drain_ring_buffer(symbol: &str, max_ticks: usize) -> Vec<InflowData> {
    // Ring buffer'ı aç (varsa — core çalışmıyorsa graceful döner)
    let ring = match std::panic::catch_unwind(|| {
        GenerationalRingBuffer::new(20_000)
    }) {
        Ok(r) => r,
        Err(_) => {
            warn!("Ring buffer açılamadı — core çalışmıyor olabilir");
            return vec![];
        }
    };

    let head = ring.get_head();
    if head == 0 {
        return vec![];
    }

    let sym_bytes = symbol_to_bytes(symbol);

    let mut inflows = Vec::with_capacity(max_ticks);
    let start_seq = head.saturating_sub(max_ticks as u64);

    for seq in start_seq..head {
        let Some(slot) = ring.read_slot(seq) else { continue };
        let data = &slot.data[..slot.len as usize];
        let Some(event) = wire::decode(data) else { continue };

        // Sembol filtresi
        if event.symbol != sym_bytes {
            continue;
        }

        match event.payload {
            EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
                let p = price.to_string().parse::<f64>().unwrap_or(0.0);
                let q = quantity.to_string().parse::<f64>().unwrap_or(0.0);
                let bsr = if is_buyer_maker { 0.7 } else { 0.3 };
                inflows.push(InflowData {
                    price: p,
                    volume: q,
                    oi_delta: 0.0,
                    funding_rate: 0.0,
                    buy_sell_ratio: bsr,
                    liquidation_volume: 0.0,
                    timestamp_ms: timestamp,
                });
            }
            EventType::Liquidation { side, price, quantity, timestamp } => {
                let p = price.to_string().parse::<f64>().unwrap_or(0.0);
                let q = quantity.to_string().parse::<f64>().unwrap_or(0.0);
                let dir = if side == 0 { 0.0 } else { 1.0 };
                inflows.push(InflowData {
                    price: p,
                    volume: 0.0,
                    oi_delta: 0.0,
                    funding_rate: 0.0,
                    buy_sell_ratio: dir,
                    liquidation_volume: p * q,
                    timestamp_ms: timestamp,
                });
            }
            EventType::FundingRate { funding_rate, mark_price, .. } => {
                let fr = funding_rate.to_string().parse::<f64>().unwrap_or(0.0);
                let mp = mark_price.to_string().parse::<f64>().unwrap_or(0.0);
                inflows.push(InflowData {
                    price: mp,
                    volume: 0.0,
                    oi_delta: 0.0,
                    funding_rate: fr,
                    buy_sell_ratio: 0.5,
                    liquidation_volume: 0.0,
                    timestamp_ms: 0,
                });
            }
            EventType::OpenInterest { open_interest, timestamp } => {
                let oi = open_interest.to_string().parse::<f64>().unwrap_or(0.0);
                inflows.push(InflowData {
                    price: 0.0,
                    volume: 0.0,
                    oi_delta: oi,
                    funding_rate: 0.0,
                    buy_sell_ratio: 0.5,
                    liquidation_volume: 0.0,
                    timestamp_ms: timestamp,
                });
            }
            _ => {}
        }

        if inflows.len() >= max_ticks {
            break;
        }
    }

    inflows
}

/// Sembol string'ini 16 baytlık sabit dizi'ye dönüştürür (core wire formatı)
fn symbol_to_bytes(symbol: &str) -> [u8; 16] {
    let mut arr = [0u8; 16];
    let bytes = symbol.as_bytes();
    let len = bytes.len().min(16);
    arr[..len].copy_from_slice(&bytes[..len]);
    arr
}

/// İki kaynağı birleştirip zaman sırasına göre sıralar.
/// Ring buffer ticks SQLite verisiyle çakışırsa ring buffer önceliklidir
/// (daha güncel — core canlı çalışıyordur).
pub fn merge_sources(
    mut sqlite_inflows: Vec<InflowData>,
    mut ring_inflows: Vec<InflowData>,
) -> Vec<InflowData> {
    // Ring buffer tick'leri SQLite'ın olmadığı zaman aralıklarını doldurur
    let sqlite_max_ts = sqlite_inflows
        .iter()
        .map(|i| i.timestamp_ms)
        .max()
        .unwrap_or(0);

    // SQLite'ın kapsamadığı canlı tick'leri ekle
    ring_inflows.retain(|r| r.timestamp_ms > sqlite_max_ts && r.price > 0.0);
    sqlite_inflows.extend(ring_inflows);
    sqlite_inflows.sort_by_key(|i| i.timestamp_ms);
    sqlite_inflows
}
