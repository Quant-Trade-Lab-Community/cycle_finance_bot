use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

pub const WINDOW_SECONDS: f64 = 3.0;
pub const MIN_SPREAD_BPS: f64 = 0.25;
pub const MIN_TICKS_PER_SECOND: f64 = 0.20;
pub const STALE_SYMBOL_SECS: f64 = 1.5;
pub const DEPTH_CANDIDATE_COUNT: usize = 60;
pub const DEPTH_STREAM_CHUNK_SIZE: usize = 30;
pub const DEPTH_REBALANCE_SECS: f64 = 2.0;
pub const DEPTH_LEVELS: usize = 10;
pub const DEPTH_UPDATE_SPEED: &str = "100ms";
pub const BOOK_TICKER_CHUNK_SIZE: usize = 180;
pub const ANALYSIS_INTERVAL_SECS: u64 = 1;
pub const WS_HEARTBEAT_SECS: u64 = 20;
pub const WS_BACKOFF_BASE_SECS: f64 = 0.75;
pub const WS_BACKOFF_CAP_SECS: f64 = 10.0;
pub const BINANCE_REST: &str = "https://fapi.binance.com";
pub const BINANCE_WS: &str = "wss://fstream.binance.com/stream";

pub const RING_NAME: &str = "/cycle_finance_scout";
pub const RING_CAPACITY: usize = 20_000;

pub fn now_ts() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_secs_f64()
}

#[derive(Debug)]
pub struct SymbolState {
    pub best_bid: f64,
    pub best_ask: f64,
    pub spread_bps: f64,
    pub mid: f64,
    pub last_book_ts: f64,
    pub last_mid_ts: f64,

    price_moves: VecDeque<(f64, f64, i64)>,
    price_bps_sum: f64,
    price_tick_sum: i64,

    depth_updates: VecDeque<f64>,
    depth_changes: VecDeque<(f64, i64)>,
    depth_change_sum: i64,
    pub last_depth_ts: f64,
    last_depth_bids: Vec<(f64, f64)>,
    last_depth_asks: Vec<(f64, f64)>,
}

impl SymbolState {
    pub fn new() -> Self {
        Self {
            best_bid: 0.0,
            best_ask: 0.0,
            spread_bps: 0.0,
            mid: 0.0,
            last_book_ts: 0.0,
            last_mid_ts: 0.0,
            price_moves: VecDeque::new(),
            price_bps_sum: 0.0,
            price_tick_sum: 0,
            depth_updates: VecDeque::new(),
            depth_changes: VecDeque::new(),
            depth_change_sum: 0,
            last_depth_ts: 0.0,
            last_depth_bids: Vec::new(),
            last_depth_asks: Vec::new(),
        }
    }

    pub fn update_book_ticker(&mut self, event_ts: f64, best_bid: f64, best_ask: f64) {
        self.best_bid = best_bid;
        self.best_ask = best_ask;
        self.last_book_ts = event_ts;

        let mid = if best_bid > 0.0 && best_ask > 0.0 {
            self.spread_bps = (best_ask - best_bid) / best_bid * 10000.0;
            (best_bid + best_ask) / 2.0
        } else {
            self.spread_bps = 0.0;
            0.0
        };

        if mid > 0.0 && self.mid > 0.0 && event_ts > self.last_mid_ts && mid != self.mid {
            let bps = (mid - self.mid).abs() / self.mid * 10000.0;
            self.price_moves.push_back((event_ts, bps, 1));
            self.price_bps_sum += bps;
            self.price_tick_sum += 1;
        }

        self.mid = mid;
        self.last_mid_ts = event_ts;
        self.expire_price_moves(event_ts);
    }

    pub fn update_depth(&mut self, event_ts: f64, bids: &[(f64, f64)], asks: &[(f64, f64)]) -> i64 {
        self.last_depth_ts = event_ts;

        let changes = if self.last_depth_bids.is_empty() && self.last_depth_asks.is_empty() {
            0
        } else {
            Self::count_depth_changes(&self.last_depth_bids, bids)
                + Self::count_depth_changes(&self.last_depth_asks, asks)
        };

        self.last_depth_bids = bids.to_vec();
        self.last_depth_asks = asks.to_vec();
        self.depth_updates.push_back(event_ts);
        self.depth_changes.push_back((event_ts, changes));
        self.depth_change_sum += changes;
        self.expire_depth(event_ts);
        changes
    }

    pub fn refresh(&mut self, now_ts: f64) {
        self.expire_price_moves(now_ts);
        self.expire_depth(now_ts);
    }

    fn expire_price_moves(&mut self, now_ts: f64) {
        let cutoff = now_ts - WINDOW_SECONDS;
        while let Some(&(ts, bps, ticks)) = self.price_moves.front() {
            if ts >= cutoff {
                break;
            }
            self.price_moves.pop_front();
            self.price_bps_sum -= bps;
            self.price_tick_sum -= ticks;
        }
    }

    fn expire_depth(&mut self, now_ts: f64) {
        let cutoff = now_ts - WINDOW_SECONDS;
        while let Some(&ts) = self.depth_updates.front() {
            if ts >= cutoff {
                break;
            }
            self.depth_updates.pop_front();
        }
        while let Some(&(ts, changes)) = self.depth_changes.front() {
            if ts >= cutoff {
                break;
            }
            self.depth_changes.pop_front();
            self.depth_change_sum -= changes;
        }
    }

    fn count_depth_changes(prev: &[(f64, f64)], cur: &[(f64, f64)]) -> i64 {
        let max = prev.len().max(cur.len());
        (0..max).filter(|&i| prev.get(i) != cur.get(i)).count() as i64
    }

    pub fn price_bps_per_s(&self) -> f64 {
        self.price_bps_sum / WINDOW_SECONDS
    }

    pub fn price_ticks_per_s(&self) -> f64 {
        self.price_tick_sum as f64 / WINDOW_SECONDS
    }

    pub fn ob_updates_per_s(&self) -> f64 {
        self.depth_updates.len() as f64 / WINDOW_SECONDS
    }

    pub fn ob_changes_per_s(&self) -> f64 {
        self.depth_change_sum as f64 / WINDOW_SECONDS
    }

    pub fn price_score(&self) -> f64 {
        if self.mid <= 0.0 || self.spread_bps <= 0.0 {
            return 0.0;
        }
        let adjusted_spread = self.spread_bps.max(MIN_SPREAD_BPS);
        (self.price_bps_per_s() * self.price_ticks_per_s()) / adjusted_spread
    }

    pub fn is_recent(&self, now_ts: f64) -> bool {
        self.last_book_ts > 0.0 && (now_ts - self.last_book_ts) <= STALE_SYMBOL_SECS
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Verdict {
    Guclu,
    Iyi,
    Normal,
    BotGurultu,
    Zayif,
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Verdict::Guclu => "GUCLU FIRSAT",
            Verdict::Iyi => "IYI FIRSAT",
            Verdict::Normal => "NORMAL",
            Verdict::BotGurultu => "BOT/GURULTU",
            Verdict::Zayif => "ZAYIF",
        }
    }

    /// Ring wire protokolündeki u8 karşılığı (0=GUCLU, 1=IYI, 2=NORMAL, 3=BOT/GURULTU, 4=ZAYIF).
    pub fn code(&self) -> u8 {
        match self {
            Verdict::Guclu => 0,
            Verdict::Iyi => 1,
            Verdict::Normal => 2,
            Verdict::BotGurultu => 3,
            Verdict::Zayif => 4,
        }
    }
}

pub struct Opportunity {
    pub symbol: String,
    pub score: f64,
    pub verdict: Verdict,
    pub efficiency: f64,
    pub price_bps_per_s: f64,
    pub price_ticks_per_s: f64,
    pub ob_changes_per_s: f64,
    pub spread_bps: f64,
}

pub struct MarketState {
    pub states: HashMap<String, SymbolState>,
    pub depth_symbols: HashSet<String>,
}

impl MarketState {
    pub fn new(symbols: Vec<String>) -> Self {
        let states = symbols
            .into_iter()
            .map(|symbol| (symbol, SymbolState::new()))
            .collect();
        Self {
            states,
            depth_symbols: HashSet::new(),
        }
    }
}
