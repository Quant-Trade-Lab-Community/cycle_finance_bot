use std::collections::VecDeque;
use chrono::{Local, TimeZone};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use crate::memory::ring_buffer::GenerationalRingBuffer;
use crate::tick::EventParser;
use crate::ring_buffer::{OwnedEvent, EventType};

struct TradeRecord {
    timestamp: u64,
    price: Decimal,
    qty: Decimal,
}

struct ActiveAnomaly {
    id: u64,
    anomaly_type: u8,
    expected_outcome: u8, // 1: Breakout, 2: Drop, 3: Rise
    start_ts: u64,
    end_ts: u64,
    start_price: Decimal,
}

pub fn start_correlation_cli() {
    let window_sec: u64 = std::env::var("WINDOW_SEC")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);
    
    // YENİ: Takip süresi (Opsiyonel olarak dışarıdan alınabilir, varsayılan window_sec kadar)
    let track_sec: u64 = std::env::var("TRACK_SEC")
        .unwrap_or_else(|_| window_sec.to_string())
        .parse()
        .unwrap_or(window_sec);

    let window_ms = window_sec * 1000;
    let track_ms = track_sec * 1000;
    let flat_threshold = Decimal::from_str("0.001").unwrap();
    let breakout_threshold = Decimal::from_str("0.005").unwrap();

    println!("========================================");
    println!("📈 KORELASYON TERMINALİ v5.0 (ASENKRON KUYRUK)");
    println!("Hedef Parite: HEIUSDT");
    println!("Analiz Penceresi: {} sn | Takip Penceresi: {} sn", window_sec, track_sec);
    println!("Kümeleme (Clustering) & Kendi Kendini Doğrulama Aktif!");
    println!("========================================");

    let gen_ring = GenerationalRingBuffer::new(160_000);
    let mut read_cursor = gen_ring.get_head();
    
    let history_limit = window_ms * 2;
    let mut history: VecDeque<TradeRecord> = VecDeque::new();
    
    let mut active_anomalies: Vec<ActiveAnomaly> = Vec::new();
    let mut failed_history: VecDeque<u8> = VecDeque::new();
    
    let mut next_anomaly_id = 1;
    let mut last_anomaly_trigger_ts = 0;

    loop {
        if let Some(slot) = gen_ring.read_slot(read_cursor) {
            let mut data = slot.data[..slot.len as usize].to_vec();
            if let Some(owned_event) = EventParser::parse(&mut data) {
                if owned_event.symbol.starts_with(b"HEIUSDT") {
                    if let EventType::Trade { price, quantity: qty, timestamp, is_buyer_maker: _ } = owned_event.payload {
                        let record = TradeRecord {
                            timestamp,
                            price,
                            qty,
                        };
                        history.push_back(record);

                        let current_ts = timestamp;
                        let current_price = price;
                        
                        // Eski verileri temizle
                        while let Some(front) = history.front() {
                            if current_ts > front.timestamp && current_ts - front.timestamp > history_limit {
                                history.pop_front();
                            } else {
                                break;
                            }
                        }

                        // 1. Yeni Anomali Tespiti
                        if let Some(first) = history.front() {
                            if current_ts - first.timestamp >= window_ms {
                                let split_ts = current_ts - window_ms;
                                
                                let mut prev_total_vol = Decimal::ZERO;
                                let mut curr_total_vol = Decimal::ZERO;
                                
                                let mut prev_prices = Vec::new();
                                let mut curr_prices = Vec::new();

                                for r in &history {
                                    if r.timestamp < split_ts {
                                        prev_total_vol += r.qty;
                                        prev_prices.push(r.price);
                                    } else {
                                        curr_total_vol += r.qty;
                                        curr_prices.push(r.price);
                                    }
                                }

                                if !prev_prices.is_empty() && !curr_prices.is_empty() {
                                    let prev_price_delta = prev_prices.last().unwrap() - prev_prices.first().unwrap();
                                    let curr_price_delta = curr_prices.last().unwrap() - curr_prices.first().unwrap();

                                    let vol_increased = curr_total_vol > prev_total_vol;
                                    let price_increased = curr_price_delta > flat_threshold;
                                    let price_decreased = curr_price_delta < -flat_threshold;
                                    let price_flat = curr_price_delta.abs() <= flat_threshold;
                                    
                                    let mut anomaly_detected = 0;
                                    let mut expected_outcome = 0;

                                    if vol_increased && price_flat {
                                        anomaly_detected = 1; expected_outcome = 1;
                                    } else if !vol_increased && price_increased {
                                        anomaly_detected = 2; expected_outcome = 2;
                                    } else if !vol_increased && price_decreased {
                                        anomaly_detected = 3; expected_outcome = 3;
                                    }

                                    // Spam koruması: Aynı saniye içinde tekrar tetiklenme
                                    if anomaly_detected > 0 && (current_ts - last_anomaly_trigger_ts > 1000) {
                                        let msg = match anomaly_detected {
                                            1 => "🚨 ANORMAL 1 (EMİLİM): Hacim devasa artıyor ama Fiyat YATAY. (Patlama Bekleniyor)",
                                            2 => "⚠️ ANORMAL 2 (SIĞ TAHTA PUMP): Hacim yok ama fiyat artıyor. (Çakılma Bekleniyor)",
                                            3 => "⚠️ ANORMAL 3 (AYI TUZAĞI): Hacim yok ama fiyat düşüyor. (Fırlama Bekleniyor)",
                                            _ => "",
                                        };

                                        let dt = Local.timestamp_millis_opt(current_ts as i64).unwrap();
                                        let time_str = dt.format("%Y-%m-%d %H:%M:%S.%3f").to_string();

                                        println!("\n[{}] [Yeni Sinyal #{}] {}", time_str, next_anomaly_id, msg);
                                        
                                        active_anomalies.push(ActiveAnomaly {
                                            id: next_anomaly_id,
                                            anomaly_type: anomaly_detected,
                                            expected_outcome,
                                            start_ts: current_ts,
                                            end_ts: current_ts + track_ms,
                                            start_price: current_price,
                                        });
                                        
                                        next_anomaly_id += 1;
                                        last_anomaly_trigger_ts = current_ts;
                                    }
                                }
                            }
                        }

                        // 2. Kuyruktaki Anomalilerin Anlık Takibi (Continuous Monitoring)
                        let mut i = 0;
                        while i < active_anomalies.len() {
                            let anomaly = &active_anomalies[i];
                            let price_change = current_price - anomaly.start_price;
                            
                            // Take-Profit kontrolü
                            let success = match anomaly.expected_outcome {
                                1 => price_change.abs() >= breakout_threshold,
                                2 => price_change <= -breakout_threshold,
                                3 => price_change >= breakout_threshold,
                                _ => false,
                            };

                            let dt = Local.timestamp_millis_opt(current_ts as i64).unwrap();
                            let time_str = dt.format("%Y-%m-%d %H:%M:%S.%3f").to_string();

                            if success {
                                let elapsed_ms = current_ts - anomaly.start_ts;
                                println!("🎯 [{}] [BAŞARILI] Sinyal #{} hedefine sadece {} ms içinde ulaştı!", time_str, anomaly.id, elapsed_ms);
                                
                                // Başarılı olunca başarısızlık zincirini kır (İsteğe bağlı, şimdilik temizliyoruz)
                                failed_history.clear();
                                
                                active_anomalies.remove(i);
                            } else if current_ts >= anomaly.end_ts {
                                // Süre doldu ve hedefe ulaşamadı (Time-Out)
                                println!("❌ [{}] [BAŞARISIZ] Sinyal #{} verilen {} sn sürede hedefine ulaşamadı.", time_str, anomaly.id, track_sec);
                                
                                failed_history.push_back(anomaly.anomaly_type);
                                if failed_history.len() > 3 {
                                    failed_history.pop_front();
                                }
                                
                                // Clustering Analizi (Son 3 sinyal)
                                if failed_history.len() == 3 {
                                    let a1 = failed_history[0];
                                    let a2 = failed_history[1];
                                    let a3 = failed_history[2];
                                    
                                    if a1 == a2 && a2 == a3 {
                                        println!("\n🌋 [KÜMELEME UYARISI] BÜYÜK PATLAMA İHTİMALİ ARTIYOR!");
                                        println!(">> Üst üste 3 kez gerçekleşmeyen Anormal {} sinyali birikti. Dev baskı var!\n", a1);
                                        failed_history.clear(); // Uyarıyı verdik, sıfırla
                                    } else if a1 != a2 && a2 != a3 && a1 != a3 {
                                        println!("\n🌪️ [KÜMELEME UYARISI] KARARSIZLIK / ALGORİTMİK SAVAŞ (TESTERE)!");
                                        println!(">> 3 farklı sinyal (Türü: {}, {}, {}) üretildi ama hiçbiri çalışmadı. Piyasa yatay.\n", a1, a2, a3);
                                        failed_history.clear(); // Uyarıyı verdik, sıfırla
                                    }
                                }
                                
                                active_anomalies.remove(i);
                            } else {
                                // Hala izleniyor
                                i += 1;
                            }
                        }
                    }
                }
            }
            read_cursor += 1;
        } else {
            std::hint::spin_loop();
        }
    }
}
