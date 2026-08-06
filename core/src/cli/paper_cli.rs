use rustyline::DefaultEditor;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::risk::portfolio::Portfolio;

pub struct PaperState {
    pub portfolio: Portfolio,
    pub leverage: HashMap<String, u32>,
    pub margin_mode: String, // "Cross" or "Isolated"
}

pub fn start_paper_cli() {
    println!("========================================");
    println!("🛡️ PAPER TRADING TERMINAL v1.0");
    println!("Type 'help' for available commands.");
    println!("========================================");

    let state = Arc::new(Mutex::new(PaperState {
        portfolio: Portfolio::new(10000.0, 0.20), // 10k USD balance, 20% max drawdown
        leverage: HashMap::new(),
        margin_mode: "Cross".to_string(),
    }));

    // In a real scenario, this thread would read from OrderRingBuffer
    // and execute orders, updating the portfolio state (simulated fills).
    
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("paper> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.is_empty() { continue; }

                match parts[0].to_lowercase().as_str() {
                    "help" => {
                        println!("Commands:");
                        println!("  status                        - Show balance, PnL, positions");
                        println!("  set leverage <symbol> <val>   - Set leverage for a symbol");
                        println!("  set margin <cross|isolated>   - Set margin mode");
                        println!("  exit                          - Quit the terminal");
                    }
                    "status" => {
                        let st = state.lock().unwrap();
                        let dummy_prices = HashMap::new(); // Simulated market prices could go here
                        let equity = st.portfolio.get_total_equity(&dummy_prices);
                        
                        println!("\n--- ACCOUNT STATUS ---");
                        println!("Cash Balance:  ${:.2}", st.portfolio.cash_balance);
                        println!("Realized PnL:  ${:.2}", st.portfolio.realized_pnl);
                        println!("Total Equity:  ${:.2}", equity);
                        println!("Commissions:   ${:.2}", st.portfolio.total_commission);
                        println!("Margin Mode:   {}", st.margin_mode);
                        println!("Positions:");
                        
                        if st.portfolio.positions.is_empty() {
                            println!("  [None]");
                        } else {
                            for (sym, pos) in &st.portfolio.positions {
                                if pos.quantity != 0.0 {
                                    let lev = st.leverage.get(sym).unwrap_or(&1);
                                    println!("  {} -> Size: {} @ ${:.2} ({}x)", sym, pos.quantity, pos.avg_entry_price, lev);
                                }
                            }
                        }
                        println!("----------------------\n");
                    }
                    "set" => {
                        if parts.len() < 3 {
                            println!("Usage: set leverage <symbol> <val> OR set margin <cross|isolated>");
                            continue;
                        }
                        let mut st = state.lock().unwrap();
                        match parts[1].to_lowercase().as_str() {
                            "leverage" => {
                                if parts.len() == 4 {
                                    let sym = parts[2].to_uppercase();
                                    if let Ok(lev) = parts[3].parse::<u32>() {
                                        st.leverage.insert(sym.clone(), lev);
                                        println!("✅ Leverage for {} set to {}x", sym, lev);
                                    } else {
                                        println!("Invalid leverage value.");
                                    }
                                }
                            }
                            "margin" => {
                                let mode = parts[2].to_lowercase();
                                if mode == "cross" || mode == "isolated" {
                                    st.margin_mode = mode.to_uppercase();
                                    println!("✅ Margin mode set to {}", st.margin_mode);
                                } else {
                                    println!("Mode must be cross or isolated.");
                                }
                            }
                            _ => {
                                println!("Unknown set command.");
                            }
                        }
                    }
                    "exit" | "quit" => {
                        println!("Shutting down paper terminal...");
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
