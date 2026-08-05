#![forbid(unsafe_code)]

mod tick;
mod queue;
mod ring_buffer;
pub mod state;
pub mod config;
pub mod pii;
pub mod db;

use tick::EventParser;
use queue::LockFreeDispatcher;
use ring_buffer::{RingBuffer, OwnedEvent};
use std::thread;
use std::time::Instant;
use os_utils::set_rt_thread_priority;
use adapter::binance::start_binance_ws_client;

#[tokio::main]
async fn main() {
    println!("Demir Yumruk 2.0 Core Initialization...");

    let dispatcher = LockFreeDispatcher::new();
    let tx = dispatcher.producer();
    let rx = dispatcher.consumer();

    // Veritabanı Thread'ini başlat (Lock-Free kuyruk ile)
    let (db_tx, db_rx) = flume::bounded(1_000_000); // 1 Milyon kapasiteli yedek kuyruk
    thread::spawn(move || {
        db::start_db_writer(db_rx);
    });

    // Execution (Emir İletim) Thread'ini başlat (Lock-Free kuyruk ile)
    let (order_tx, order_rx) = flume::bounded(10_000);
    thread::spawn(move || {
        // Çevre değişkenlerini yükle
        let _ = dotenvy::dotenv();
        let api_key = std::env::var("BINANCE_API_KEY").unwrap_or_else(|_| "DUMMY_KEY".to_string());
        let secret_key = std::env::var("BINANCE_SECRET_KEY").unwrap_or_else(|_| "DUMMY_SECRET".to_string());
        
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
            
        rt.block_on(async {
            execution_engine::start_execution_engine(order_rx, api_key, secret_key).await;
        });
    });

    thread::spawn(move || {
        set_rt_thread_priority(99);

        let mut tick_count = 0;
        let mut total_parse_time = std::time::Duration::new(0, 0);
        let mut last_report = Instant::now();
        
        // Allocate exactly ~1 GB Ring Buffer for 20-level Orderbook structs (~1.6 Million events)
        let mut ring_buffer = RingBuffer::new(160_000);

        while let Ok(mut bytes) = rx.recv() {
            let start_parse = Instant::now();
            
            if let Some(owned_event) = EventParser::parse(&mut bytes) {
                // SIFIR TAHSİS YAZMA: Önceden tahsis edilmiş devasa Ring Buffer'a kalıcı olarak kaydet
                // Eğer buffer tamamen dolar ve baştan yazmaya başlarsa, ezilen eski veriyi döner
                if let Some(evicted) = ring_buffer.push(owned_event) {
                    // Ezilen veriyi ana döngüyü yavaşlatmadan DB'ye asenkron postala
                    let _ = db_tx.try_send(evicted);
                }

                total_parse_time += start_parse.elapsed();
                tick_count += 1;
            } else {
                if let Ok(s) = std::str::from_utf8(&bytes) {
                    println!("Failed to parse: {}", s);
                }
            }

            if last_report.elapsed().as_secs() >= 1 {
                let avg_parse_time = if tick_count > 0 {
                    total_parse_time.as_nanos() as f64 / tick_count as f64
                } else {
                    0.0
                };
                println!("[METRICS] Ticks/sec: {} | Avg Parse Latency: {:.2} ns | RAM Buffer: {}/{}", tick_count, avg_parse_time, ring_buffer.write_index(), ring_buffer.capacity());
                
                tick_count = 0;
                total_parse_time = std::time::Duration::new(0, 0);
                last_report = Instant::now();
            }
        }
    });

    start_binance_ws_client(tx).await;
}
