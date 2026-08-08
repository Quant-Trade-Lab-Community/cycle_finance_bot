use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use flume::Sender;
use serde_json::json;

/// Yeniden bağlanma politikası: 1s ile başla, her başarısız denemede ikiye katla,
/// en fazla 60s (üstel geri çekilme). Başarılı bağlantı geri çekilme seviyesini sıfırlar.
const BASE_RECONNECT_DELAY_MS: u64 = 1_000;
const MAX_RECONNECT_DELAY_MS: u64 = 60_000;

async fn fetch_usdt_spot_pairs() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    println!("Binance WS: Limiting subscriptions to specific symbols...");
    
    let target_symbols = vec!["btcusdt", "ethusdt", "solusdt", "heiusdt"];
    let mut pairs = Vec::new();
    
    for sym in target_symbols {
        pairs.push(format!("{}@trade", sym));
        pairs.push(format!("{}@depth20@100ms", sym));
    }
    
    println!("Binance WS: Found {} streams for targeted Futures pairs.", pairs.len());
    Ok(pairs)
}

async fn start_ws_chunk(tx: Sender<Vec<u8>>, chunk: Vec<String>, chunk_id: usize) {
    let ws_url = "wss://fstream.binance.com/stream";

    println!("Binance WS [Chunk {}]: Connecting ({} streams)...", chunk_id, chunk.len());

    // Üstel geri çekme: her başarısız denemeden sonra ikiye katlan, 60s'de tavanla.
    let mut backoff_ms = BASE_RECONNECT_DELAY_MS;

    loop {
        match connect_async(ws_url).await {
            Ok((ws_stream, _)) => {
                println!("Binance WS [Chunk {}]: Successfully connected.", chunk_id);
                backoff_ms = BASE_RECONNECT_DELAY_MS;

                let (mut write, mut read) = ws_stream.split();

                let sub_msg = json!({
                    "method": "SUBSCRIBE",
                    "params": chunk,
                    "id": chunk_id
                });

                if let Err(e) = write.send(Message::Text(sub_msg.to_string())).await {
                    eprintln!("Binance WS [Chunk {}]: Subscribe failed: {}", chunk_id, e);
                    continue;
                }

                // 30 sn'de bir Ping — Binance sessiz bağlantıları kapatır (idle timeout).
                // Ayrıca ticker, runtuk kapanmadan önce kopuşu yakalamamızı sağlar.
                let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));
                ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

                loop {
                    tokio::select! {
                        _ = ping_interval.tick() => {
                            if write.send(Message::Ping(Vec::new())).await.is_err() {
                                eprintln!("Binance WS [Chunk {}]: Ping failed, reconnecting.", chunk_id);
                                break;
                            }
                        }
                        msg = read.next() => {
                            match msg {
                                Some(Ok(message)) => {
                                    if message.is_text() {
                                        let text = message.into_text().unwrap();
                                        let bytes = text.into_bytes();

                                        // Bounded kuyruk → geri basınç (asla RAM taşmaz).
                                        if tx.send_async(bytes).await.is_err() {
                                            eprintln!("Binance WS [Chunk {}]: Consumer queue dropped, shutting down.", chunk_id);
                                            return;
                                        }
                                    } else if message.is_close() {
                                        eprintln!("Binance WS [Chunk {}]: Server closed connection.", chunk_id);
                                        break;
                                    }
                                }
                                Some(Err(e)) => {
                                    eprintln!("Binance WS [Chunk {}]: Read error: {}", chunk_id, e);
                                    break;
                                }
                                None => {
                                    eprintln!("Binance WS [Chunk {}]: Stream ended.", chunk_id);
                                    break;
                                }
                            }
                        }
                    }
                }

                println!("Binance WS [Chunk {}]: Disconnected.", chunk_id);
            }
            Err(e) => {
                eprintln!("Binance WS [Chunk {}]: Connection failed: {}", chunk_id, e);
            }
        }

        // Bağlantı koptu ya da hiç kurulamadı: geri çekme ile yeniden dene.
        tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
        backoff_ms = (backoff_ms * 2).min(MAX_RECONNECT_DELAY_MS);
        println!("Binance WS [Chunk {}]: Reconnecting in {}s...", chunk_id, backoff_ms / 1000);
    }
}

/// Connects to Binance live WebSocket stream for all USDT trade events.
pub async fn start_binance_ws_client(tx: Sender<Vec<u8>>) {
    match fetch_usdt_spot_pairs().await {
        Ok(pairs) => {
            // Binance allows up to 200 streams per WebSocket connection
            let chunks: Vec<Vec<String>> = pairs.chunks(200).map(|c| c.to_vec()).collect();
            
            let mut handles = Vec::new();
            for (i, chunk) in chunks.into_iter().enumerate() {
                let tx_clone = tx.clone();
                handles.push(tokio::spawn(async move {
                    start_ws_chunk(tx_clone, chunk, i + 1).await;
                }));
                // Binance's DDoS firewall (WAF) blocks the IP if we open too many WS connections simultaneously.
                // Add a small delay between opening chunks.
                tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
            }
            
            for handle in handles {
                let _ = handle.await;
            }
        }
        Err(e) => {
            eprintln!("Binance WS: Failed to fetch pairs: {}", e);
        }
    }
}