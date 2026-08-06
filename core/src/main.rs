pub mod state;
pub mod config;
pub mod pii;
pub mod db;
pub mod validator;
pub mod cli;

pub mod hal;
pub mod memory;
pub mod timer;
pub mod strategy;
pub mod risk;
pub mod engine;

mod tick;
mod queue;
mod ring_buffer;

use tick::EventParser;
use queue::LockFreeDispatcher;
use std::thread;
use std::time::Instant;
use os_utils::set_rt_thread_priority;
use adapter::binance::start_binance_ws_client;
use crate::memory::ring_buffer::GenerationalRingBuffer;
use crate::memory::order_ring::OrderRingBuffer;

#[tokio::main]
async fn main() {
    let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "DATA".to_string());
    
    if run_mode == "DATA" {
        println!("🚀 Başlatılıyor: MARKET DATA KONSOLU");
        let gen_ring = std::sync::Arc::new(GenerationalRingBuffer::new(160_000));
        let gen_ring_data = gen_ring.clone();
        
        let (db_tx, db_rx) = flume::bounded(1_000_000); 
        thread::spawn(move || {
            db::start_db_writer(db_rx);
        });

        let dispatcher = LockFreeDispatcher::new();
        let tx = dispatcher.producer();
        let rx = dispatcher.consumer();

        thread::spawn(move || {
            set_rt_thread_priority(99);
            let mut tick_count = 0;
            let mut total_parse_time = std::time::Duration::new(0, 0);
            let mut last_report = Instant::now();
            let mut validator = validator::DataValidator::new();
            
            while let Ok(mut bytes) = rx.recv() {
                let start_parse = Instant::now();
                if let Some(owned_event) = EventParser::parse(&mut bytes) {
                    if !validator.is_valid(&owned_event) { continue; }

                    gen_ring_data.push(&bytes);
                    let _ = db_tx.try_send(owned_event); 

                    total_parse_time += start_parse.elapsed();
                    tick_count += 1;
                }

                if last_report.elapsed().as_secs() >= 1 {
                    let avg_parse_time = if tick_count > 0 {
                        total_parse_time.as_nanos() as f64 / tick_count as f64
                    } else { 0.0 };
                    println!("[MARKET DATA] Ticks/sec: {} | Avg Parse Latency: {:.2} ns", tick_count, avg_parse_time);
                    
                    tick_count = 0;
                    total_parse_time = std::time::Duration::new(0, 0);
                    last_report = Instant::now();
                }
            }
        });

        start_binance_ws_client(tx).await;
        return;
    }

    if run_mode == "PAPER" {
        cli::paper_cli::start_paper_cli();
        return;
    }

    if run_mode == "STRATEGY" {
        cli::strategy_cli::start_strategy_cli();
        return;
    }

    if run_mode == "BACKTEST" {
        let csv_path = std::env::var("CSV_PATH").unwrap_or_else(|_| "/home/smhvz/Desktop/PROJE/test_data.csv".to_string());
        engine::backtester::start_backtester(&csv_path);
        return;
    }

    println!("Lütfen geçerli bir RUN_MODE belirleyin (DATA, PAPER, STRATEGY, BACKTEST)");
}
