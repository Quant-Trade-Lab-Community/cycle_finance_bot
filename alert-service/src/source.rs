//! Veri kaynakları: `(symbol, price)` akışı üreten kaynaklar.
//!
//! - **ring**: mevcut DATA terminalinin tick ring'ini okur (`/dev/shm/demir_yumruk_ring`)
//! - **binance**: doğrudan Binance Futures WS'ine abone olur (bağımsız çalışır)

use flume::Sender;
use rust_decimal::Decimal;
use std::sync::Arc;

pub type PriceSink = Sender<(String, Decimal)>;

/// DATA terminalinin tick ring'inden fiyatları okur ve `sink`'e iletir.
pub fn spawn_ring_source(sink: PriceSink) {
    std::thread::spawn(move || {
        let gen_ring = proje_core::memory::ring_buffer::GenerationalRingBuffer::new(160_000);
        let mut cursor = gen_ring.get_head();

        loop {
            if let Some(slot) = gen_ring.read_slot(cursor) {
                let mut data = slot.data[..slot.len as usize].to_vec();
                if let Some(event) = proje_core::tick::EventParser::parse(&mut data) {
                    use proje_core::ring_buffer::EventType;
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
                std::hint::spin_loop();
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
            if let Some(event) = proje_core::tick::EventParser::parse(&mut owned) {
                use proje_core::ring_buffer::EventType;
                if let EventType::Trade { price, .. } = event.payload {
                    let sym = decode_symbol(&event.symbol);
                    let _ = sink.send((sym, price));
                }
            }
        }
    }
}

/// Sembol seti için tick ring'de veri gelip gelmediğini doğrular (debug).
pub fn is_ring_alive() -> bool {
    let ring = proje_core::memory::ring_buffer::GenerationalRingBuffer::new(160_000);
    ring.get_head() > 0
}

/// Price-feed servisinden (:3004) periyodik fiyat çeker ve sink'e iletir.
/// `last` → `mark` → `index` → `ask` önceliğiyle fiyatı kullanır.
pub fn spawn_pricefeed_source(sink: PriceSink, symbols: Vec<String>, refresh_ms: u64) {
    let base = std::env::var("PRICE_FEED_URL").unwrap_or_else(|_| "http://127.0.0.1:3004".to_string());
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .expect("reqwest client");
        loop {
            for sym in &symbols {
                let url = format!("{base}/api/lastprice/{}", sym.to_uppercase());
                match client.get(&url).send() {
                    Ok(resp) => {
                        if let Ok(v) = resp.json::<serde_json::Value>() {
                            if let Some(price) = v.pointer("/price")
                                .and_then(|p| p.get("last").or(p.get("mark")).or(p.get("index")).or(p.get("ask")))
                                .and_then(|x| x.as_f64())
                            {
                                if price > 0.0 {
                                    if let Some(d) = rust_decimal::Decimal::from_f64_retain(price) {
                                        let rounded = d.round_dp(6);
                                        let _ = sink.send((sym.to_uppercase(), rounded));
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(refresh_ms));
        }
    });
}

pub type SharedPriceSink = Arc<PriceSink>;
