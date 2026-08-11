//! Akış çalıştırıcı — her akış bağımsız bir OS sürecidir.
//!
//! Hattın tüm akışlar için değişmez kuralı:
//!
//! ```text
//! WS | REST → parse → validate → ring buffer → TimescaleDB
//! ```
//!
//! - Trade, depth, liquidation → **WS** (Binance bu ağdan iletiyor).
//! - Funding, markprice, indexprice, lastprice, oi → **REST fallback**
//!   (bu ağdan WS ile iletilmiyor; aynı frame → parse → validate → ring → TSDB hattı).
//!
//! 1. Veri kaynağı (WS görevi / REST poller): ham veriyi sınırlı flume
//!    kanalına yazar (geri basınç).
//! 2. Tüketici thread (RT 99): parse → validate → `wire::encode` →
//!    bu akışın `/dev/shm` ring'ine push → DB kanalına ilet.
//! 3. DB yazıcı thread: TimescaleDB hypertable'a batch commit.
//!
//! Bağımsız akışlar yalnızca Binance API rate limitlerine takılmamak için
//! ortak rate kapısından (`RateGate`) geçer.

pub mod parse;
pub mod rest;

use gateway::binance::start_ws_client;
use pipeline::validator::DataValidator;
use persistence::timescaledb::start_tsdb_writer;
use transport::flow::FlowKind;
use transport::ring_buffer::GenerationalRingBuffer;
use transport::wire;

pub const RAW_QUEUE_CAPACITY: usize = 262_144;
pub const DB_QUEUE_CAPACITY: usize = 1_000_000;

/// Bu akışta WS iletilmiyorsa REST fallback kullanılır (aynı hat).
pub fn has_rest_fallback(kind: FlowKind) -> bool {
    matches!(
        kind,
        FlowKind::Funding
            | FlowKind::MarkPrice
            | FlowKind::IndexPrice
            | FlowKind::LastPrice
            | FlowKind::OpenInterest
    )
}

/// Akışın abone olacağı Binance Futures stream'leri (WS kaynağı için).
pub fn streams_for(kind: FlowKind, symbols: &[String]) -> Vec<String> {
    let lower: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
    match kind {
        FlowKind::OpenInterest => vec!["!openInterest@arr".to_string()],
        FlowKind::Trade => lower.iter().map(|s| format!("{s}@trade")).collect(),
        FlowKind::Depth => lower.iter().map(|s| format!("{s}@depth20@100ms")).collect(),
        FlowKind::Liquidation => lower.iter().map(|s| format!("{s}@forceOrder")).collect(),
        FlowKind::Funding | FlowKind::MarkPrice => {
            lower.iter().map(|s| format!("{s}@markPrice@1s")).collect()
        }
        FlowKind::LastPrice => lower.iter().map(|s| format!("{s}@lastPrice@1s")).collect(),
        FlowKind::IndexPrice => lower.iter().map(|s| format!("{s}@indexPrice@1s")).collect(),
    }
}

/// Sembol seti (`CYCLE_FLOW_SYMBOLS`), varsayılan gateway ile aynı.
pub fn load_symbols() -> Vec<String> {
    if let Ok(v) = std::env::var("CYCLE_FLOW_SYMBOLS") {
        let s: Vec<String> = v
            .split(',')
            .map(|x| x.trim().to_uppercase())
            .filter(|x| !x.is_empty())
            .collect();
        if !s.is_empty() {
            return s;
        }
    }
    vec!["BTCUSDT".into(), "ETHUSDT".into(), "SOLUSDT".into(), "HEIUSDT".into()]
}

/// Bir akışı başlatır (bloklar). Her akış kendi ring'ini ve DB tablosunu kullanır.
pub fn run(kind: FlowKind) {
    let budget_mb = kind.memory_budget_bytes() / (1024 * 1024);
    println!("🚀 Akış başlıyor: {} | ring: {} | bellek: {} MB | tablo: {}", kind.as_str(), kind.ring_name(), budget_mb, kind.table());

    let symbols = load_symbols();
    let streams = streams_for(kind, &symbols);
    let source = if has_rest_fallback(kind) { "REST" } else { "WS" };
    println!("  Semboller: {} | kaynak: {source} | stream: {}", symbols.join(", "), streams.join(", "));

    let (raw_tx, raw_rx) = flume::bounded::<Vec<u8>>(RAW_QUEUE_CAPACITY);
    let (db_tx, db_rx) = flume::bounded::<transport::events::OwnedEvent>(DB_QUEUE_CAPACITY);

    // 1) Tüketici thread — parse → validate → ring → DB kanalı (veri akışı kuralı).
    std::thread::spawn(move || {
        os_utils::set_rt_thread_priority(99);
        let ring = GenerationalRingBuffer::with_name(kind.ring_name(), kind.ring_capacity());
        let mut validator = DataValidator::new();
        let mut frame_buf = [0u8; wire::MAX_FRAME_SIZE];

        let mut evt = 0u64;
        let mut invalid = 0u64;
        let mut db_drops = 0u64;
        let mut last_report = std::time::Instant::now();

        while let Ok(mut bytes) = raw_rx.recv() {
            for ev in parse::parse_for(kind, &mut bytes) {
                if !validator.is_valid(&ev) {
                    invalid += 1;
                    continue;
                }
                if let Some(len) = wire::encode(&ev, &mut frame_buf) {
                    ring.push(&frame_buf[..len]);
                }
                if db_tx.try_send(ev).is_err() {
                    db_drops += 1;
                }
                evt += 1;
            }

            if last_report.elapsed().as_secs() >= 1 {
                println!("[{}] evt/s: {} | invalid: {} | db_drops: {}", kind.as_str(), evt, invalid, db_drops);
                evt = 0;
                invalid = 0;
                db_drops = 0;
                last_report = std::time::Instant::now();
            }
        }
    });

    // 2) DB yazıcı thread → TimescaleDB (batch commit).
    std::thread::spawn(move || start_tsdb_writer(db_rx, kind));

    // 3) Veri kaynağı: REST fallback (WS sessiz akışlar) veya WS + rate kapısı.
    if has_rest_fallback(kind) {
        // REST poller sonsuza kadar çalışır → join ana thread'i bloklar (süreç yaşar).
        let _ = rest::spawn(kind, symbols, raw_tx).join();
    } else {
        let runtime = tokio::runtime::Runtime::new().expect("akış tokio runtime");
        runtime.block_on(async move {
            start_ws_client(raw_tx, streams, true).await;
        });
    }
}
