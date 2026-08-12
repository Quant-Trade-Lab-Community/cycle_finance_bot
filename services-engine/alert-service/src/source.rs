//! Veri kaynakları: `(symbol, price)` akışı üreten kaynaklar.
//!
//! - **ring**: flow ring'lerinden fiyatları okur (`/dev/shm/cycle_finance_trades`)
//! - **binance**: doğrudan Binance Futures WS'ine abone olur (bağımsız çalışır)

use flume::Sender;
use rust_decimal::Decimal;
use std::sync::Arc;

pub type PriceSink = Sender<(String, Decimal)>;

/// Flow ring'indeki trade fiyatlarını okur ve `sink`'e iletir.
pub fn spawn_ring_source(sink: PriceSink) {
    std::thread::spawn(move || {
        let gen_ring = transport::ring_buffer::GenerationalRingBuffer::with_name("/cycle_finance_trades", 160_000);
        let mut cursor = gen_ring.get_head();

        loop {
            if let Some(slot) = gen_ring.read_slot(cursor) {
                if let Some(event) = transport::wire::decode(&slot.data[..slot.len as usize]) {
                    use transport::events::EventType;
                    match event.payload {
                        EventType::Trade { price, .. } => {
                            let sym = decode_symbol(&event.symbol);
                            let _ = sink.send((sym, price));
                        }
                        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
                            let price = if best_ask_price > Decimal::ZERO {
                                best_ask_price
                            } else {
                                best_bid_price
                            };
                            if price > Decimal::ZERO {
                                let sym = decode_symbol(&event.symbol);
                                let _ = sink.send((sym, price));
                            }
                        }
                        _ => {}
                    }
                }
                cursor += 1;
            } else {
                std::thread::sleep(std::time::Duration::from_micros(500));
            }
        }
    });
}

fn decode_symbol(buf: &[u8; 16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(16);
    String::from_utf8_lossy(&buf[..len]).to_string()
}

/// Doğrudan Binance Futures WS'ine abone olur (bağımsız çalışma modu).
pub async fn spawn_binance_source(sink: PriceSink, symbols: Vec<String>) {
    use futures_util::{SinkExt, StreamExt};
    use serde_json::json;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    let streams: Vec<String> = symbols
        .iter()
        .map(|s| format!("{}@trade", s.to_lowercase()))
        .collect();

    let url = format!("wss://fstream.binance.com/stream?streams={}", streams.join("/"));
    println!("[ALERT] Binance WS: {url}");

    let (mut ws, _) = match connect_async(&url).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[ALERT] WS bağlantı hatası: {e}");
            return;
        }
    };
    let (mut write, mut read) = ws.split();

    let sub = json!({"method":"SUBSCRIBE","params":streams,"id":1});
    if let Err(e) = write.send(Message::Text(sub.to_string())).await {
        eprintln!("[ALERT] subscribe hatası: {e}");
        return;
    }

    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            let bytes = text.into_bytes();
            let mut owned = bytes;
            if let Some(event) = pipeline::tick::EventParser::parse(&mut owned) {
                use transport::events::EventType;
                if let EventType::Trade { price, .. } = event.payload {
                    let sym = decode_symbol(&event.symbol);
                    let _ = sink.send((sym, price));
                }
            }
        }
    }
}

/// Sembol seti için trade flow ring'inde veri gelip gelmediğini doğrular (debug).
pub fn is_ring_alive() -> bool {
    let ring = transport::ring_buffer::GenerationalRingBuffer::with_name("/cycle_finance_trades", 160_000);
    ring.get_head() > 0
}

/// Trade flow ring'ini (`/cycle_finance_trades`) SPIN-LOOP ile okur ve sink'e iletir.
/// Poll gecikmesi yoktur — gerçek zamanlı. Kaynak: flow (RAM paylaşımlı bellek).
pub fn spawn_flow_ring_source(sink: PriceSink) {
    std::thread::spawn(move || {
        let gen_ring = transport::ring_buffer::GenerationalRingBuffer::with_name(
            "/cycle_finance_trades", 20_000,
        );
        let mut cursor = gen_ring.get_head();

        loop {
            if ingest_flow_slot(&gen_ring, cursor, &sink) {
                cursor += 1;
                continue;
            }

            // Slot henüz yazılmamış olabilir — körlemesine head'e atlamak yeni
            // event'i kaçırır; önce kısa bir retry penceresiyle tekrar dene.
            let mut advanced = false;
            for _ in 0..40 {
                std::thread::sleep(std::time::Duration::from_micros(100));
                if ingest_flow_slot(&gen_ring, cursor, &sink) {
                    cursor += 1;
                    advanced = true;
                    break;
                }
            }
            if advanced {
                continue;
            }

            // ~4ms boyunca okunamadı ve üretici bizim önümüzdeyse slot
            // overwrite olmuştur — cursor'ı güncel konuma taşı.
            let head = gen_ring.get_head();
            if head > cursor {
                cursor = head;
            } else {
                std::thread::sleep(std::time::Duration::from_micros(500));
            }
        }
    });
}

/// `seq`'deki flow ring slot'unu okur, fiyat event'ini sink'e iletir. Slot yoksa `false`.
fn ingest_flow_slot(ring: &transport::ring_buffer::GenerationalRingBuffer, seq: u64, sink: &PriceSink) -> bool {
    let Some(slot) = ring.read_slot(seq) else {
        return false;
    };
    // Yeni (sıfırlanmış) ring'de seq=0 slot'ları boş görünür ve `read_slot`
    // seq=0 için yanlışlıkla Some döner — içerik doğrulaması şart.
    if slot.len == 0 {
        return false;
    }
    let Some(event) = transport::wire::decode(&slot.data[..slot.len as usize]) else {
        return false;
    };
    use transport::events::EventType;
    let sym = decode_symbol(&event.symbol);
    if sym.is_empty() {
        return true;
    }
    match event.payload {
        EventType::Trade { price, .. } => {
            let _ = sink.send((sym, price));
        }
        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
            let price = if best_ask_price > Decimal::ZERO {
                best_ask_price
            } else {
                best_bid_price
            };
            if price > Decimal::ZERO {
                let _ = sink.send((sym, price));
            }
        }
        EventType::FundingRate { mark_price, index_price, .. } => {
            let _ = sink.send((sym.clone(), mark_price));
            if index_price > Decimal::ZERO {
                let _ = sink.send((sym, index_price));
            }
        }
        _ => {}
    }
    true
}

pub type SharedPriceSink = Arc<PriceSink>;
