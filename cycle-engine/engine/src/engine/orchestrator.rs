use std::panic::{catch_unwind, AssertUnwindSafe};
use transport::ring_buffer::GenerationalRingBuffer;
use strategies_engine::trait_def::{Strategy, Signal};
use risk_engine::engine::RiskEngine;
use risk_engine::types::{OrderIntent, OrderKind, RiskDecision, Side};
use infra::timer::tsc::TscTimer;
use crossbeam_channel::Sender;
use rust_decimal::Decimal;

#[derive(PartialEq)]
enum StrategyState {
    Active,
    Draining,
    Poisoned,
}

struct ShardedStrategy {
    strategy: Box<dyn Strategy>,
    state: StrategyState,
    /// Sinyal→OrderIntent eşlemesinde kullanılan sembol.
    symbol: String,
}

pub struct TitaniumOrchestrator {
    strategies: Vec<ShardedStrategy>,
    risk_manager: RiskEngine,
    gateway_tx: Sender<Signal>,
}

/// `Signal`'ı risk kapısına girecek `OrderIntent`'e çevirir.
/// Sembol, stratejinin işlem yaptığı piyasadır (headless modda tek sembol).
fn signal_to_intent(signal: Signal, symbol: &str, strategy_id: u32) -> Option<OrderIntent> {
    let (side, quantity, price, kind) = match signal {
        Signal::BuyMarket { quantity } => (Side::Buy, quantity, None, OrderKind::Market),
        Signal::SellMarket { quantity } => (Side::Sell, quantity, None, OrderKind::Market),
        Signal::BuyLimit { price, quantity } => (Side::Buy, quantity, Some(price), OrderKind::Limit),
        Signal::SellLimit { price, quantity } => (Side::Sell, quantity, Some(price), OrderKind::Limit),
        Signal::None | Signal::CancelAll => return None,
    };
    Some(OrderIntent {
        strategy_id,
        symbol: symbol.to_string(),
        side,
        quantity,
        price,
        kind,
        reduce_only: false,
        close_position: false,
        leverage: None,
    })
}

impl TitaniumOrchestrator {
    pub fn new(
        strategies: Vec<(Box<dyn Strategy>, String)>,
        _initial_balance: Decimal,
        max_position_usdt: Decimal,
        daily_loss_usdt: Decimal,
        gateway_tx: Sender<Signal>,
    ) -> Self {
        let sharded = strategies
            .into_iter()
            .map(|(strategy, symbol)| ShardedStrategy {
                strategy,
                state: StrategyState::Active,
                symbol,
            })
            .collect();

        Self {
            strategies: sharded,
            risk_manager: RiskEngine::with_limits(max_position_usdt, daily_loss_usdt),
            gateway_tx,
        }
    }

    pub fn run_spin_loop(&mut self, ring_buffer: &GenerationalRingBuffer) {
        println!("TitaniumOrchestrator: Entering spin loop (Headless)...");

        let mut head: u64 = 0;
        let timer = TscTimer::new();
        let mut last_timer_tick = timer.elapsed_ns();

        loop {
            let current_seq = ring_buffer.get_head(); // Acquire

            while head < current_seq {
                if let Some(slot) = ring_buffer.read_slot(head) {
                    let frame_id = slot.seq;

                    let risk = &self.risk_manager;
                    let gateway = &self.gateway_tx;
                    for shard in &mut self.strategies {
                        if shard.state == StrategyState::Active {
                            // Protect against panics in strategy code (Catch-Unwind)
                            let result = catch_unwind(AssertUnwindSafe(|| {
                                shard.strategy.on_market_data(frame_id, &slot)
                            }));

                            match result {
                                Ok(sig) => {
                                    gate_and_dispatch(risk, gateway, shard, sig);
                                }
                                Err(_) => {
                                    eprintln!("STRATEGY PANIC CAUGHT! Poisoning strategy ID: {}", shard.strategy.id());
                                    shard.state = StrategyState::Poisoned;
                                }
                            }
                        }
                    }
                }
                head += 1;
            }

            // Timer tick (e.g. 1ms = 1_000_000 ns)
            let current_time = timer.elapsed_ns();
            if current_time - last_timer_tick > 1_000_000 {
                let frame_id = current_time; // Simplified
                let delta = current_time - last_timer_tick;

                let risk = &self.risk_manager;
                let gateway = &self.gateway_tx;
                for shard in &mut self.strategies {
                    if shard.state == StrategyState::Active {
                        let result = catch_unwind(AssertUnwindSafe(|| {
                            shard.strategy.on_timer(frame_id, delta)
                        }));

                        match result {
                            Ok(sig) => {
                                gate_and_dispatch(risk, gateway, shard, sig);
                            }
                            Err(_) => {
                                shard.state = StrategyState::Poisoned;
                            }
                        }
                    }
                }
                last_timer_tick = current_time;
            }

            // Spin-wait optimization (CPU Pause)
            std::hint::spin_loop();
        }
    }
}

/// Sinyali risk kapısından geçirir; onaylanırsa gateway'e gönderir.
fn gate_and_dispatch(
    risk: &RiskEngine,
    gateway: &Sender<Signal>,
    shard: &mut ShardedStrategy,
    signal: Signal,
) {
    let Some(intent) = signal_to_intent(signal, &shard.symbol, shard.strategy.id()) else {
        return;
    };
    match risk.evaluate(intent) {
        RiskDecision::Approved { intent } => {
            let signal = match (intent.kind, intent.side) {
                (OrderKind::Market, Side::Buy) => Signal::BuyMarket { quantity: intent.quantity },
                (OrderKind::Market, Side::Sell) => Signal::SellMarket { quantity: intent.quantity },
                (OrderKind::Limit, Side::Buy) => Signal::BuyLimit {
                    price: intent.price.unwrap_or_default(),
                    quantity: intent.quantity,
                },
                (OrderKind::Limit, Side::Sell) => Signal::SellLimit {
                    price: intent.price.unwrap_or_default(),
                    quantity: intent.quantity,
                },
            };
            let _ = gateway.send(signal);
        }
        RiskDecision::Rejected { reason, .. } => {
            eprintln!(
                "RISK REJECTED [{}] {}: {}",
                reason.rule_name(),
                shard.strategy.id(),
                reason.describe()
            );
        }
    }
}
