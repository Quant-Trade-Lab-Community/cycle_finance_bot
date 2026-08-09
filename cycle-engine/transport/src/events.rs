//! Market veri modeli — tüm katmanların ortak dili.
//!
//! `OwnedEvent` + `EventType`: veri alım hattının çıktısı, ring buffer'ın
//! veri modeli ve analiz/tüketici katmanlarının girdisi. Bu dosya
//! `contracts` katmanında durduğu için hiçbir katman başka bir katmanın
//! implementasyonundan bu tipleri ithal etmek zorunda değildir.

use rust_decimal::Decimal;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum EventType {
    Trade { price: Decimal, quantity: Decimal, timestamp: u64, is_buyer_maker: bool },
    Orderbook {
        bids: [(Decimal, Decimal); 20],
        asks: [(Decimal, Decimal); 20]
    },
    Liquidation { side: u8, price: Decimal, quantity: Decimal, timestamp: u64 },
    FundingRate { mark_price: Decimal, index_price: Decimal, funding_rate: Decimal, next_funding_time: u64 },
    BookTicker { best_bid_price: Decimal, best_bid_qty: Decimal, best_ask_price: Decimal, best_ask_qty: Decimal },
    OpenInterest { open_interest: Decimal, timestamp: u64 },
    /// Scout fırsat sinyali — mikroyapi analiz sonucu (verdict: 0=GUCLU, 1=IYI, 2=NORMAL, 3=BOT/GURULTU, 4=ZAYIF).
    Opportunity {
        score: Decimal,
        efficiency: Decimal,
        price_bps_per_s: Decimal,
        price_ticks_per_s: Decimal,
        ob_changes_per_s: Decimal,
        spread_bps: Decimal,
        verdict: u8,
    },
    /// Tek sembol canlı mikroyapi metrikleri (scout analizi).
    SymbolMetrics {
        score: Decimal,
        efficiency: Decimal,
        price_bps_per_s: Decimal,
        price_ticks_per_s: Decimal,
        ob_changes_per_s: Decimal,
        spread_bps: Decimal,
    },
}

impl std::fmt::Debug for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
                f.debug_struct("Trade")
                    .field("price", price)
                    .field("quantity", quantity)
                    .field("timestamp", timestamp)
                    .field("is_buyer_maker", is_buyer_maker)
                    .finish()
            }
            EventType::Orderbook { bids, asks } => {
                f.debug_struct("Orderbook").field("bids", bids).field("asks", asks).finish()
            }
            EventType::Liquidation { side, price, quantity, timestamp } => {
                f.debug_struct("Liquidation")
                    .field("side", side)
                    .field("price", price)
                    .field("quantity", quantity)
                    .field("timestamp", timestamp)
                    .finish()
            }
            EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time } => {
                f.debug_struct("FundingRate")
                    .field("mark_price", mark_price)
                    .field("index_price", index_price)
                    .field("funding_rate", funding_rate)
                    .field("next_funding_time", next_funding_time)
                    .finish()
            }
            EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty } => {
                f.debug_struct("BookTicker")
                    .field("best_bid_price", best_bid_price)
                    .field("best_bid_qty", best_bid_qty)
                    .field("best_ask_price", best_ask_price)
                    .field("best_ask_qty", best_ask_qty)
                    .finish()
            }
            EventType::OpenInterest { open_interest, timestamp } => {
                f.debug_struct("OpenInterest")
                    .field("open_interest", open_interest)
                    .field("timestamp", timestamp)
                    .finish()
            }
            EventType::Opportunity { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict } => {
                f.debug_struct("Opportunity")
                    .field("score", score)
                    .field("efficiency", efficiency)
                    .field("price_bps_per_s", price_bps_per_s)
                    .field("price_ticks_per_s", price_ticks_per_s)
                    .field("ob_changes_per_s", ob_changes_per_s)
                    .field("spread_bps", spread_bps)
                    .field("verdict", verdict)
                    .finish()
            }
            EventType::SymbolMetrics { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps } => {
                f.debug_struct("SymbolMetrics")
                    .field("score", score)
                    .field("efficiency", efficiency)
                    .field("price_bps_per_s", price_bps_per_s)
                    .field("price_ticks_per_s", price_ticks_per_s)
                    .field("ob_changes_per_s", ob_changes_per_s)
                    .field("spread_bps", spread_bps)
                    .finish()
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct OwnedEvent {
    pub symbol: [u8; 16],
    pub payload: EventType,
}

impl std::fmt::Debug for OwnedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedEvent")
            .field("symbol", &self.symbol)
            .field("payload", &self.payload)
            .finish()
    }
}

impl OwnedEvent {
    #[inline(always)]
    fn pack_symbol(sym: &str) -> [u8; 16] {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        symbol
    }

    #[inline(always)]
    pub fn new_trade(sym: &str, price: Decimal, quantity: Decimal, timestamp: u64, is_buyer_maker: bool) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::Trade { price, quantity, timestamp, is_buyer_maker },
        }
    }

    #[inline(always)]
    pub fn new_orderbook(sym: &str, bids: [(Decimal, Decimal); 20], asks: [(Decimal, Decimal); 20]) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::Orderbook { bids, asks },
        }
    }

    #[inline(always)]
    pub fn new_liquidation(sym: &str, side: u8, price: Decimal, quantity: Decimal, timestamp: u64) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::Liquidation { side, price, quantity, timestamp },
        }
    }

    #[inline(always)]
    pub fn new_funding_rate(sym: &str, mark_price: Decimal, index_price: Decimal, funding_rate: Decimal, next_funding_time: u64) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time },
        }
    }

    #[inline(always)]
    pub fn new_bookticker(sym: &str, best_bid_price: Decimal, best_bid_qty: Decimal, best_ask_price: Decimal, best_ask_qty: Decimal) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty },
        }
    }

    #[inline(always)]
    pub fn new_open_interest(sym: &str, open_interest: Decimal, timestamp: u64) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::OpenInterest { open_interest, timestamp },
        }
    }

    #[inline(always)]
    pub fn new_opportunity(
        sym: &str,
        score: Decimal,
        efficiency: Decimal,
        price_bps_per_s: Decimal,
        price_ticks_per_s: Decimal,
        ob_changes_per_s: Decimal,
        spread_bps: Decimal,
        verdict: u8,
    ) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::Opportunity {
                score,
                efficiency,
                price_bps_per_s,
                price_ticks_per_s,
                ob_changes_per_s,
                spread_bps,
                verdict,
            },
        }
    }

    #[inline(always)]
    pub fn new_symbol_metrics(
        sym: &str,
        score: Decimal,
        efficiency: Decimal,
        price_bps_per_s: Decimal,
        price_ticks_per_s: Decimal,
        ob_changes_per_s: Decimal,
        spread_bps: Decimal,
    ) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::SymbolMetrics {
                score,
                efficiency,
                price_bps_per_s,
                price_ticks_per_s,
                ob_changes_per_s,
                spread_bps,
            },
        }
    }
}