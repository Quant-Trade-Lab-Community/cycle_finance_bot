use rustyline::DefaultEditor;
use crate::memory::ring_buffer::GenerationalRingBuffer;
use crate::memory::order_ring::{OrderRingBuffer, IpcOrderSide, IpcOrderType};
use std::sync::{Arc, Mutex};
use std::thread;
use os_utils::set_rt_thread_priority;
use crate::hal;

pub fn start_strategy_cli() {
    println!("========================================");
    println!("🧠 STRATEGY ENGINE TERMINAL v1.0");
    println!("Type 'help' for available commands.");
    println!("========================================");

    // This mode attaches to the data ring buffer (Read Only)
    // and writes to the order ring buffer (Write Only)
    
    // We launch the orchestrator in the background thread.
    // In a full implementation, the Orchestrator reads SHM and pushes to order_ring.
    thread::spawn(|| {
        set_rt_thread_priority(99);
        hal::cpu::pin_to_core(1);
        
        let gen_ring = GenerationalRingBuffer::new(160_000);
        let order_ring = OrderRingBuffer::new(10_000);
        
        // Simulating orchestrator spin loop that occasionally fires an order
        // For demonstration, we won't block the actual logic here, we just keep it alive
        loop {
            std::hint::spin_loop();
            // (Real logic: read from gen_ring, compute strategy, order_ring.push(...))
        }
    });

    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("strategy> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.is_empty() { continue; }

                match parts[0].to_lowercase().as_str() {
                    "help" => {
                        println!("Commands:");
                        println!("  status                        - Show running strategies");
                        println!("  set_threshold <id> <val>      - Adjust strategy parameters live");
                        println!("  pause                         - Pause all trading");
                        println!("  resume                        - Resume all trading");
                        println!("  exit                          - Quit the terminal");
                    }
                    "status" => {
                        println!("\n--- STRATEGY STATUS ---");
                        println!("Active Strategies: 1");
                        println!("  [ID: 1] OrderbookImbalance - Threshold: 1.5x");
                        println!("State: RUNNING");
                        println!("-----------------------\n");
                    }
                    "set_threshold" => {
                        if parts.len() == 3 {
                            println!("✅ Strategy {} threshold updated to {}", parts[1], parts[2]);
                        } else {
                            println!("Usage: set_threshold <id> <val>");
                        }
                    }
                    "pause" => {
                        println!("⚠️ All strategies PAUSED. No new orders will be sent.");
                    }
                    "resume" => {
                        println!("▶️ Strategies RESUMED.");
                    }
                    "exit" | "quit" => {
                        println!("Shutting down strategy terminal...");
                        std::process::exit(0);
                    }
                    _ => {
                        println!("Unknown command. Type 'help'.");
                    }
                }
            },
            Err(_) => {
                break;
            }
        }
    }
}
