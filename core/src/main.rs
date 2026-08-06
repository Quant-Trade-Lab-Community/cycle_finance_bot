pub mod state;
pub mod config;
pub mod pii;
pub mod db;
pub mod validator;

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

    let db_tx_exec = db_tx.clone();
    
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
            
        let db_tx_clone = db_tx_exec;
        
        rt.block_on(async {
            // Arka planda periyodik olarak Open Interest (Açık Pozisyon) yoklayıcısı başlat
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT"];
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    for sym in &symbols {
                        if let Ok(resp) = client.get(&format!("https://fapi.binance.com/fapi/v1/openInterest?symbol={}", sym)).send().await {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(oi_str) = json.get("openInterest").and_then(|v| v.as_str()) {
                                    if let Ok(oi) = oi_str.parse::<f64>() {
                                        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
                                        let event = ring_buffer::OwnedEvent::new_open_interest(sym, oi, timestamp);
                                        let _ = db_tx_clone.try_send(event);
                                    }
                                }
                            }
                        }
                    }
                }
            });
            
            execution_engine::start_execution_engine(order_rx, api_key, secret_key).await;
        });
    });

    // TITANIUM CORE ORCHESTRATOR
    // Yüksek Performanslı Ring Buffer (Sıfır Kopya, Generational Index)
    let gen_ring = std::sync::Arc::new(memory::ring_buffer::GenerationalRingBuffer::new(100_000));
    let gen_ring_clone = gen_ring.clone();
    
    // Lock-Free Gateway Kanalı (Strateji -> Execution)
    // Execution module is using flume, so we bridge crossbeam to flume
    let (gw_tx, gw_rx) = crossbeam_channel::bounded(1024);
    let order_tx_titanium = order_tx.clone();
    
    thread::spawn(move || {
        while let Ok(sig) = gw_rx.recv() {
            // Signal to OrderRequest bridge
            match sig {
                strategy::trait_def::Signal::BuyMarket { quantity } => {
                    let _ = order_tx_titanium.send(execution_engine::order::OrderRequest {
                        symbol: "BTCUSDT".to_string(),
                        side: execution_engine::order::OrderSide::Buy,
                        order_type: execution_engine::order::OrderType::Market,
                        quantity,
                        price: None,
                        time_in_force: None,
                    });
                },
                strategy::trait_def::Signal::SellMarket { quantity } => {
                    let _ = order_tx_titanium.send(execution_engine::order::OrderRequest {
                        symbol: "BTCUSDT".to_string(),
                        side: execution_engine::order::OrderSide::Sell,
                        order_type: execution_engine::order::OrderType::Market,
                        quantity,
                        price: None,
                        time_in_force: None,
                    });
                },
                _ => {}
            }
        }
    });

    thread::spawn(move || {
        hal::cpu::pin_to_core(1); // Pin Orchestrator to CPU Core 1
        
        let risk_engine = risk::engine::RiskEngine::new(10, -5000); // Max 10 BTC, 5000 USDT max daily loss
        
        let strat1 = Box::new(strategy::impls::imbalance::OrderbookImbalanceStrategy::new(1, 1.5));
        
        let strategies: Vec<Box<dyn strategy::trait_def::Strategy>> = vec![strat1];
        
        let mut orchestrator = engine::orchestrator::TitaniumOrchestrator::new(strategies, risk_engine, gw_tx);
        
        orchestrator.run_spin_loop(&gen_ring_clone);
    });

    thread::spawn(move || {
        set_rt_thread_priority(99);

        let mut tick_count = 0;
        let mut total_parse_time = std::time::Duration::new(0, 0);
        let mut last_report = Instant::now();
        
        let mut validator = validator::DataValidator::new();
        let circuit_breaker = validator.circuit_breaker.clone();
        
        // Allocate exactly ~1 GB Ring Buffer for 20-level Orderbook structs (~1.6 Million events)
        let mut ring_buffer = RingBuffer::new(160_000);

        while let Ok(mut bytes) = rx.recv() {
            let start_parse = Instant::now();
            
            if let Some(owned_event) = EventParser::parse(&mut bytes) {
                // VERİ DOĞRULAMA (DATA VALIDATION)
                if !validator.is_valid(&owned_event) {
                    // Bozuk veri, çöpe at. Şalter (Circuit Breaker) atarsa sistem durur.
                    continue;
                }

                // SIFIR TAHSİS YAZMA: Önceden tahsis edilmiş devasa Ring Buffer'a kalıcı olarak kaydet
                // Eğer buffer tamamen dolar ve baştan yazmaya başlarsa, ezilen eski veriyi döner
                if let Some(evicted) = ring_buffer.push(owned_event) {
                    // Ezilen veriyi ana döngüyü yavaşlatmadan DB'ye asenkron postala
                    let _ = db_tx.try_send(evicted);
                }
                
                // TITANIUM CORE: Push raw bytes to the new Generational Ring Buffer
                // The Orchestrator thread will pick this up instantly via spin-loop.
                gen_ring.push(&bytes);

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
