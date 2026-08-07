//! STRATEGY terminali — HEIUSDT kırılım stratejisini çalıştırır.
//!
//! Strateji mantığı Python'da (`strategies/heiusdt_breakout.py`) çalışır:
//! detect-ms'ten seviye/yapı analizi alır, kırılım koşullarını kontrol eder,
//! paper-service'e emir açar. Bu modül Python sürecini spawn eder.

use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const HEIUSDT_SCRIPT: &str = "/home/smhvz/Desktop/PROJE/strategies/heiusdt_breakout.py";

struct StrategyChild {
    child: Child,
}

pub fn start_strategy_cli() {
    println!("========================================");
    println!("🎯 STRATEGY ENGINE — HEIUSDT KIRILIM");
    println!("  Script: {}", HEIUSDT_SCRIPT);
    println!("  detect-ms :3002 + paper-service :8080");
    println!("========================================");

    let running = Arc::new(AtomicBool::new(false));
    let mut child: Option<StrategyChild> = spawn_strategy();
    if child.is_none() {
        println!("❌ HEIUSDT stratejisi başlatılamadı.");
    } else {
        running.store(true, Ordering::SeqCst);
        println!("✅ HEIUSDT stratejisi çalışıyor.");
    }

    let mut rl = rustyline::DefaultEditor::new().unwrap();

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
                        println!("  status      - Show strategy status");
                        println!("  restart     - Restart HEIUSDT strategy");
                        println!("  exit        - Quit the terminal");
                    }
                    "status" => {
                        if running.load(Ordering::SeqCst) {
                            println!("  🎯 HEIUSDT Kırılım — RUNNING");
                        } else {
                            println!("  🎯 HEIUSDT Kırılım — DURDU");
                        }
                    }
                    "restart" => {
                        println!("🔄 Strateji yeniden başlatılıyor...");
                        if let Some(mut c) = child.take() {
                            let _ = c.child.kill();
                        }
                        child = spawn_strategy();
                        if child.is_some() {
                            running.store(true, Ordering::SeqCst);
                            println!("✅ HEIUSDT stratejisi yeniden başlatıldı.");
                        } else {
                            running.store(false, Ordering::SeqCst);
                            println!("❌ Yeniden başlatılamadı.");
                        }
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

fn spawn_strategy() -> Option<StrategyChild> {
    match Command::new("python3")
        .arg(HEIUSDT_SCRIPT)
        .current_dir("/home/smhvz/Desktop/PROJE")
        .spawn()
    {
        Ok(child) => Some(StrategyChild { child }),
        Err(e) => {
            eprintln!("❌ Python süreci başlatılamadı: {}", e);
            None
        }
    }
}
