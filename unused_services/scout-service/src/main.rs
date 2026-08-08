mod analyzer;
mod client;
mod models;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use transport::ring_buffer::GenerationalRingBuffer;
use contracts::events::OwnedEvent;
use rust_decimal::Decimal;
use serde_json::Value;
use tokio::task::JoinHandle;

use analyzer::OrderbookFluxAnalyzer;
use client::{chunked, event_ts, BinanceClient, Handler};
use models::{
    MarketState, Opportunity, Verdict, ANALYSIS_INTERVAL_SECS, BOOK_TICKER_CHUNK_SIZE,
    DEPTH_REBALANCE_SECS, DEPTH_STREAM_CHUNK_SIZE, RING_CAPACITY, RING_NAME, WINDOW_SECONDS,
    now_ts,
};

fn lock(market: &Arc<Mutex<MarketState>>) -> std::sync::MutexGuard<'_, MarketState> {
    market.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn to_dec(v: f64) -> Decimal {
    let mut d = Decimal::from_f64_retain(v).unwrap_or(Decimal::ZERO);
    d.rescale(6);
    d
}

/// Fırsat + metrikleri compact binary frame olarak ring buffer'a yazar.
struct ScoutRing {
    ring: GenerationalRingBuffer,
    frame_buf: Vec<u8>,
}

impl ScoutRing {
    fn new() -> Self {
        Self {
            ring: GenerationalRingBuffer::with_name(RING_NAME, RING_CAPACITY),
            frame_buf: vec![0u8; contracts::wire::MAX_FRAME_SIZE],
        }
    }

    fn push(&mut self, ev: &OwnedEvent) {
        if let Some(len) = contracts::wire::encode(ev, &mut self.frame_buf) {
            self.ring.push(&self.frame_buf[..len]);
        }
    }

    fn push_opportunity(&mut self, opp: &Opportunity) {
        let ev = OwnedEvent::new_opportunity(
            &opp.symbol,
            to_dec(opp.score),
            to_dec(opp.efficiency),
            to_dec(opp.price_bps_per_s),
            to_dec(opp.price_ticks_per_s),
            to_dec(opp.ob_changes_per_s),
            to_dec(opp.spread_bps),
            opp.verdict.code(),
        );
        self.push(&ev);
    }

    fn push_symbol_metrics(&mut self, opp: &Opportunity) {
        let ev = OwnedEvent::new_symbol_metrics(
            &opp.symbol,
            to_dec(opp.score),
            to_dec(opp.efficiency),
            to_dec(opp.price_bps_per_s),
            to_dec(opp.price_ticks_per_s),
            to_dec(opp.ob_changes_per_s),
            to_dec(opp.spread_bps),
        );
        self.push(&ev);
    }
}

struct OpportunityLogger {
    last_symbol: Option<String>,
    last_verdict: Option<Verdict>,
}

impl OpportunityLogger {
    fn new() -> Self {
        Self {
            last_symbol: None,
            last_verdict: None,
        }
    }

    fn log(&mut self, opp: &Opportunity) {
        if self.last_symbol.as_deref() == Some(opp.symbol.as_str())
            && self.last_verdict == Some(opp.verdict)
        {
            println!(
                "FIRSAT DEVAM: {} | {} | score={:.2} | eff={:.4} | spread={:.2}",
                opp.symbol,
                opp.verdict.as_str(),
                opp.score,
                opp.efficiency,
                opp.spread_bps,
            );
            return;
        }

        self.last_symbol = Some(opp.symbol.clone());
        self.last_verdict = Some(opp.verdict);
        println!(
            "FIRSAT BULUNDU: {} | {} | score={:.2} | eff={:.4} | p_bps={:.2} | ticks={:.2} | ob_changes={:.2} | spread={:.2}",
            opp.symbol,
            opp.verdict.as_str(),
            opp.score,
            opp.efficiency,
            opp.price_bps_per_s,
            opp.price_ticks_per_s,
            opp.ob_changes_per_s,
            opp.spread_bps,
        );
    }
}

struct ScoutService {
    client: Arc<BinanceClient>,
    market: Arc<Mutex<MarketState>>,
    ring: Arc<Mutex<ScoutRing>>,
    book_ticker_tasks: Vec<JoinHandle<()>>,
    depth_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
    depth_manager_task: Option<JoinHandle<()>>,
    analysis_task: Option<JoinHandle<()>>,
}

impl ScoutService {
    fn new() -> Self {
        Self {
            client: Arc::new(BinanceClient::new()),
            market: Arc::new(Mutex::new(MarketState::new(Vec::new()))),
            ring: Arc::new(Mutex::new(ScoutRing::new())),
            book_ticker_tasks: Vec::new(),
            depth_tasks: Arc::new(Mutex::new(Vec::new())),
            depth_manager_task: None,
            analysis_task: None,
        }
    }

    async fn start(&mut self) -> Result<(), reqwest::Error> {
        let symbols = self.client.fetch_symbols().await?;
        self.market = Arc::new(Mutex::new(MarketState::new(symbols.clone())));
        println!("{} sembol taraniyor...", symbols.len());

        for chunk in chunked(&symbols, BOOK_TICKER_CHUNK_SIZE) {
            let client = Arc::clone(&self.client);
            let market = Arc::clone(&self.market);

            let handler: Handler = Box::new(move |data| {
                let market = Arc::clone(&market);
                Box::pin(async move {
                    Self::handle_book_ticker(&market, data).await;
                })
            });

            self.book_ticker_tasks.push(tokio::spawn(async move {
                client.stream_book_tickers(&chunk, handler).await;
            }));
        }

        let market = Arc::clone(&self.market);
        let client = Arc::clone(&self.client);
        let depth_tasks = Arc::clone(&self.depth_tasks);
        self.depth_manager_task = Some(tokio::spawn(async move {
            Self::depth_manager_loop(&market, &client, &depth_tasks).await;
        }));

        let market = Arc::clone(&self.market);
        let ring = Arc::clone(&self.ring);
        self.analysis_task = Some(tokio::spawn(async move {
            Self::analysis_loop(&market, &ring).await;
        }));

        Ok(())
    }

    async fn handle_book_ticker(market: &Arc<Mutex<MarketState>>, data: Value) {
        let Some(symbol) = data["s"].as_str() else { return };
        let ts = event_ts(&data);
        let bid = data["b"].as_str().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let ask = data["a"].as_str().and_then(|v| v.parse().ok()).unwrap_or(0.0);

        let mut m = lock(market);
        if let Some(state) = m.states.get_mut(symbol) {
            state.update_book_ticker(ts, bid, ask);
        }
    }

    async fn handle_depth(market: &Arc<Mutex<MarketState>>, data: Value) {
        let Some(symbol) = data["s"].as_str() else { return };
        let ts = event_ts(&data);

        let bids = parse_levels(&data["b"]);
        let asks = parse_levels(&data["a"]);

        let mut m = lock(market);
        if !m.depth_symbols.contains(symbol) {
            return;
        }
        if let Some(state) = m.states.get_mut(symbol) {
            state.update_depth(ts, &bids, &asks);
        }
    }

    async fn depth_manager_loop(
        market: &Arc<Mutex<MarketState>>,
        client: &Arc<BinanceClient>,
        depth_tasks: &Arc<Mutex<Vec<JoinHandle<()>>>>,
    ) {
        let analyzer = OrderbookFluxAnalyzer::new();
        let mut last_rebalance = 0.0f64;

        loop {
            let candidates: HashSet<String> = {
                let mut m = lock(market);
                analyzer.get_depth_candidates(&mut m).into_iter().collect()
            };

            let depth_empty = lock(market).depth_symbols.is_empty();
            if candidates.is_empty() && !depth_empty {
                tokio::time::sleep(Duration::from_secs(ANALYSIS_INTERVAL_SECS)).await;
                continue;
            }

            let now = now_ts();
            let mut should_rebalance = depth_empty;
            if !should_rebalance && now - last_rebalance >= DEPTH_REBALANCE_SECS {
                let current: HashSet<String> = lock(market).depth_symbols.iter().cloned().collect();
                should_rebalance = candidates != current;
            }

            if should_rebalance {
                {
                    let mut tasks = depth_tasks.lock().unwrap_or_else(|p| p.into_inner());
                    for task in tasks.drain(..) {
                        task.abort();
                    }
                }

                lock(market).depth_symbols = candidates.clone();
                last_rebalance = now;

                if !candidates.is_empty() {
                    let mut sorted: Vec<String> = candidates.into_iter().collect();
                    sorted.sort();
                    println!("Depth izleme guncellendi: {} sembol", sorted.len());

                    for chunk in chunked(&sorted, DEPTH_STREAM_CHUNK_SIZE) {
                        let market = Arc::clone(market);
                        let client = Arc::clone(client);
                        let handler: Handler = Box::new(move |data| {
                            let market = Arc::clone(&market);
                            Box::pin(async move {
                                Self::handle_depth(&market, data).await;
                            })
                        });
                        let handle = tokio::spawn(async move {
                            client.stream_partial_depths(&chunk, handler).await;
                        });
                        depth_tasks
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .push(handle);
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(ANALYSIS_INTERVAL_SECS)).await;
        }
    }

    async fn analysis_loop(market: &Arc<Mutex<MarketState>>, ring: &Arc<Mutex<ScoutRing>>) {
        println!("Isinma suresi ({}s) bekleniyor...", WINDOW_SECONDS as u32);
        tokio::time::sleep(Duration::from_secs_f64(WINDOW_SECONDS)).await;

        let analyzer = OrderbookFluxAnalyzer::new();
        let mut logger = OpportunityLogger::new();

        loop {
            let (best, metrics) = {
                let mut m = lock(market);
                (
                    analyzer.get_best_opportunity(&mut m),
                    analyzer.get_symbol_metrics(&mut m),
                )
            };

            if let Some(opp) = &best {
                logger.log(opp);
            }

            {
                let mut r = ring.lock().unwrap_or_else(|p| p.into_inner());
                if let Some(opp) = &best {
                    r.push_opportunity(opp);
                }
                for m in &metrics {
                    r.push_symbol_metrics(m);
                }
            }

            tokio::time::sleep(Duration::from_secs(ANALYSIS_INTERVAL_SECS)).await;
        }
    }

    async fn stop(&mut self) {
        for task in self.book_ticker_tasks.drain(..) {
            task.abort();
        }
        {
            let mut tasks = self.depth_tasks.lock().unwrap_or_else(|p| p.into_inner());
            for task in tasks.drain(..) {
                task.abort();
            }
        }
        if let Some(task) = self.depth_manager_task.take() {
            task.abort();
        }
        if let Some(task) = self.analysis_task.take() {
            task.abort();
        }
    }
}

fn parse_levels(value: &Value) -> Vec<(f64, f64)> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|level| {
                    let price = level[0].as_str()?.parse().ok()?;
                    let qty = level[1].as_str()?.parse().ok()?;
                    Some((price, qty))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::main]
async fn main() {
    println!("USDT pariteleri icin tarama servisi baslatildi...");

    let mut service = ScoutService::new();
    if let Err(err) = service.start().await {
        eprintln!("Servis baslatilamadi: {}", err);
        std::process::exit(1);
    }

    tokio::signal::ctrl_c().await.ok();
    service.stop().await;
}
