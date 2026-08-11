//! Veri akışı (flow) tipleri.
//!
//! Her akış bağımsız bir OS sürecidir ve aynı hattı izler:
//!
//! ```text
//! WS → parse → validate → ring buffer → TimescaleDB
//! ```
//!
//! Bellek bütçeleri ring buffer boyutunu belirler; her akışın kendi
//! paylaşımlı bellek ring'i ve TimescaleDB hypertable'ı vardır. Akışlar
//! birbirinden bağımsızdır; yalnızca Binance API limitlerine takılmamak
//! için ortak rate kapısından (`gateway::rate_gate`) geçer.

use crate::ring_buffer::MarketDataSlot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowKind {
    /// 1. akış — trade data (bellek: 50 MB)
    Trade,
    /// 2. akış — orderbook depth 20 (bellek: 100 MB)
    Depth,
    /// 3. akış — likidasyon data (bellek: 20 MB)
    Liquidation,
    /// 4. akış — open interest (bellek: 20 MB)
    OpenInterest,
    /// 5. akış — funding rate (bellek: 10 MB)
    Funding,
    /// 6. akış — mark price (bellek: 50 MB)
    MarkPrice,
    /// 7. akış — last price (bellek: 50 MB)
    LastPrice,
    /// 8. akış — index price (bellek: 50 MB)
    IndexPrice,
}

impl FlowKind {
    /// Bağımsız akışlar (All dahil değil).
    pub const ALL: &'static [FlowKind] = &[
        FlowKind::Trade,
        FlowKind::Depth,
        FlowKind::Liquidation,
        FlowKind::OpenInterest,
        FlowKind::Funding,
        FlowKind::MarkPrice,
        FlowKind::LastPrice,
        FlowKind::IndexPrice,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            FlowKind::Trade => "trade",
            FlowKind::Depth => "depth",
            FlowKind::Liquidation => "liquidation",
            FlowKind::OpenInterest => "open-interest",
            FlowKind::Funding => "funding",
            FlowKind::MarkPrice => "mark-price",
            FlowKind::LastPrice => "last-price",
            FlowKind::IndexPrice => "index-price",
        }
    }

    /// Bu akışın POSIX shm ring adı (akış başına ayrı ring).
    pub fn ring_name(self) -> &'static str {
        match self {
            FlowKind::Trade => "/cycle_finance_trades",
            FlowKind::Depth => "/cycle_finance_depth",
            FlowKind::Liquidation => "/cycle_finance_liquidations",
            FlowKind::OpenInterest => "/cycle_finance_open_interest",
            FlowKind::Funding => "/cycle_finance_funding",
            FlowKind::MarkPrice => "/cycle_finance_markprice",
            FlowKind::LastPrice => "/cycle_finance_lastprice",
            FlowKind::IndexPrice => "/cycle_finance_indexprice",
        }
    }

    /// Ring buffer bellek bütçesi (bayt) — paylaşımlı bellekte bu kadar yer ayrılır.
    pub fn memory_budget_bytes(self) -> usize {
        const MB: usize = 1024 * 1024;
        match self {
            FlowKind::Trade => 50 * MB,
            FlowKind::Depth => 100 * MB,
            FlowKind::Liquidation => 20 * MB,
            FlowKind::OpenInterest => 20 * MB,
            FlowKind::Funding => 10 * MB,
            FlowKind::MarkPrice => 50 * MB,
            FlowKind::LastPrice => 50 * MB,
            FlowKind::IndexPrice => 50 * MB,
        }
    }

    /// Bellek bütçesine göre ring slot kapasitesi.
    pub fn ring_capacity(self) -> usize {
        (self.memory_budget_bytes() / std::mem::size_of::<MarketDataSlot>()).max(1)
    }

    /// Bu akışın TimescaleDB hypertable'ı.
    pub fn table(self) -> &'static str {
        match self {
            FlowKind::Trade => "trades",
            FlowKind::Depth => "orderbooks",
            FlowKind::Liquidation => "liquidations",
            FlowKind::OpenInterest => "open_interests",
            FlowKind::Funding => "funding_rates",
            FlowKind::MarkPrice => "markprices",
            FlowKind::LastPrice => "lastprices",
            FlowKind::IndexPrice => "indexprices",
        }
    }
}
