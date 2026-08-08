use proje_core::tick::EventParser;
use proje_core::queue::LockFreeDispatcher;
use std::thread;
use std::time::Instant;
use os_utils::set_rt_thread_priority;
use adapter::binance::start_binance_ws_client;
use transport::ring_buffer::GenerationalRingBuffer;

#[tokio::main]
async fn main() {
    cycle_splash::show_splash();

    let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "DATA".to_string());
    
    if run_mode == "DATA" {
        println!("🚀 Başlatılıyor: MARKET DATA KONSOLU");
        let gen_ring = std::sync::Arc::new(GenerationalRingBuffer::new(160_000));
        let gen_ring_data = gen_ring.clone();
        
        let (db_tx, db_rx) = flume::bounded(1_000_000); 
        thread::spawn(move || {
            proje_core::db::start_db_writer(db_rx);
        });

        let dispatcher = LockFreeDispatcher::new();
        let tx = dispatcher.producer();
        let rx = dispatcher.consumer();

        thread::spawn(move || {
            set_rt_thread_priority(99);
            let mut tick_count = 0;
            let mut depth_count = 0u64;
            let mut invalid_count = 0u64;
            let mut db_drop_count = 0u64;
            let mut total_parse_time = std::time::Duration::new(0, 0);
            let mut last_report = Instant::now();
            let mut validator = proje_core::validator::DataValidator::new();
            let mut frame_buf = [0u8; contracts::wire::MAX_FRAME_SIZE];
            
            while let Ok(mut bytes) = rx.recv() {
                let start_parse = Instant::now();
                // simd_json sıfır-kopya parse buffer'ı BOZAR (ayırıcıları '\0' yapar).
                // Ring'e artık typed binary (wire::encode) yazılır — kopya yoktur.
                if let Some(owned_event) = EventParser::parse(&mut bytes) {
                    if !validator.is_valid(&owned_event) { invalid_count += 1; continue; }
                    if matches!(owned_event.payload, contracts::events::EventType::Orderbook { .. }) {
                        depth_count += 1;
                    }
                    if let Some(len) = contracts::wire::encode(&owned_event, &mut frame_buf) {
                        gen_ring_data.push(&frame_buf[..len]);
                    }
                    if db_tx.try_send(owned_event).is_err() {
                        db_drop_count += 1;
                    }

                    total_parse_time += start_parse.elapsed();
                    tick_count += 1;
                }

                if last_report.elapsed().as_secs() >= 1 {
                    let avg_parse_time = if tick_count > 0 {
                        total_parse_time.as_nanos() as f64 / tick_count as f64
                    } else { 0.0 };
                    println!("[MARKET DATA] Ticks/sec: {} | depth: {} | invalid: {} | db_drops: {} | Avg Parse: {:.2} ns", tick_count, depth_count, invalid_count, db_drop_count, avg_parse_time);
                    
                    tick_count = 0;
                    depth_count = 0;
                    invalid_count = 0;
                    db_drop_count = 0;
                    total_parse_time = std::time::Duration::new(0, 0);
                    last_report = Instant::now();
                }
            }
        });

        start_binance_ws_client(tx).await;
        return;
    }

    if run_mode == "PAPER" {
        proje_core::cli::paper_cli::start_paper_cli();
        return;
    }

    if run_mode == "STRATEGY" {
        println!("🚀 Başlatılıyor: STRATEJI KONSOLU");
        proje_core::cli::strategy_cli::start_strategy_cli();
        return;
    }

    if run_mode == "BACKTEST" {
        let csv_path = std::env::var("CSV_PATH").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data.csv").to_string());
        proje_core::engine::backtester::start_backtester(&csv_path);
        return;
    }

    if run_mode == "CORRELATION" {
        proje_core::cli::correlation_cli::start_correlation_cli();
        return;
    }

    println!("Lütfen geçerli bir RUN_MODE belirleyin (DATA, PAPER, STRATEGY, BACKTEST, CORRELATION)");
}
