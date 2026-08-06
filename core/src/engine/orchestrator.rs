use std::panic::{catch_unwind, AssertUnwindSafe};
use crate::memory::ring_buffer::GenerationalRingBuffer;
use crate::strategy::trait_def::{Strategy, Signal};
use crate::risk::engine::RiskEngine;
use crate::timer::tsc::TscTimer;
use crossbeam_channel::Sender;

#[derive(PartialEq)]
enum StrategyState {
    Active,
    Draining,
    Poisoned,
}

struct ShardedStrategy {
    strategy: Box<dyn Strategy>,
    state: StrategyState,
}

pub struct TitaniumOrchestrator {
    strategies: Vec<ShardedStrategy>,
    risk_manager: RiskEngine,
    gateway_tx: Sender<Signal>,
}

impl TitaniumOrchestrator {
    pub fn new(
        strategies: Vec<Box<dyn Strategy>>,
        risk_manager: RiskEngine,
        gateway_tx: Sender<Signal>,
    ) -> Self {
        let sharded = strategies.into_iter().map(|s| ShardedStrategy {
            strategy: s,
            state: StrategyState::Active,
        }).collect();

        Self {
            strategies: sharded,
            risk_manager,
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
                    
                    for shard in &mut self.strategies {
                        if shard.state == StrategyState::Active {
                            // Protect against panics in strategy code (Catch-Unwind)
                            let result = catch_unwind(AssertUnwindSafe(|| {
                                shard.strategy.on_market_data(frame_id, &slot)
                            }));

                            match result {
                                Ok(sig) => {
                                    if let Some(valid_sig) = self.risk_manager.process_signal(sig.clone(), shard.strategy.id()) {
                                        match valid_sig {
                                            Signal::None => {},
                                            _ => {
                                                // Dispatch to gateway (Execution Engine)
                                                let _ = self.gateway_tx.send(valid_sig);
                                            }
                                        }
                                    }
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
                
                for shard in &mut self.strategies {
                    if shard.state == StrategyState::Active {
                        let result = catch_unwind(AssertUnwindSafe(|| {
                            shard.strategy.on_timer(frame_id, delta)
                        }));

                        match result {
                            Ok(sig) => {
                                if let Some(valid_sig) = self.risk_manager.process_signal(sig.clone(), shard.strategy.id()) {
                                    match valid_sig {
                                        Signal::None => {},
                                        _ => {
                                            let _ = self.gateway_tx.send(valid_sig);
                                        }
                                    }
                                }
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
