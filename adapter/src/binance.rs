use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use flume::Sender;
use serde_json::{Value, json};

async fn fetch_usdt_spot_pairs() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    println!("Binance WS: Fetching active USDT Futures pairs from REST API...");
    let url = "https://fapi.binance.com/fapi/v1/exchangeInfo";
    let resp = reqwest::get(url).await?.json::<Value>().await?;
    
    let mut pairs = Vec::new();
    if let Some(symbols) = resp.get("symbols").and_then(|s| s.as_array()) {
        for s in symbols {
            if let (Some(quote), Some(status), Some(symbol)) = (
                s.get("quoteAsset").and_then(|v| v.as_str()),
                s.get("status").and_then(|v| v.as_str()),
                s.get("symbol").and_then(|v| v.as_str()),
            ) {
                if quote == "USDT" && status == "TRADING" {
                    let sym = symbol.to_lowercase();
                    pairs.push(format!("{}@trade", sym));
                    pairs.push(format!("{}@depth20@100ms", sym));
                    pairs.push(format!("{}@forceOrder", sym));
                    pairs.push(format!("{}@markPrice", sym));
                    pairs.push(format!("{}@bookTicker", sym));
                }
            }
        }
    }
    
    println!("Binance WS: Found {} streams for USDT Futures pairs.", pairs.len());
    Ok(pairs)
}

async fn start_ws_chunk(tx: Sender<Vec<u8>>, chunk: Vec<String>, chunk_id: usize) {
    let ws_url = "wss://fstream.binance.com/stream";
    
    println!("Binance WS [Chunk {}]: Connecting ({} streams)...", chunk_id, chunk.len());

    match connect_async(ws_url).await {
        Ok((ws_stream, _)) => {
            println!("Binance WS [Chunk {}]: Successfully connected.", chunk_id);
            let (mut write, mut read) = ws_stream.split();

            let sub_msg = json!({
                "method": "SUBSCRIBE",
                "params": chunk,
                "id": chunk_id
            });
            
            if let Err(e) = write.send(Message::Text(sub_msg.to_string())).await {
                eprintln!("Binance WS [Chunk {}]: Subscribe failed: {}", chunk_id, e);
                return;
            }

            while let Some(msg) = read.next().await {
                if let Ok(message) = msg {
                    if message.is_text() {
                        let text = message.into_text().unwrap();
                        let bytes = text.into_bytes();
                        
                        if tx.send(bytes).is_err() {
                            eprintln!("Binance WS [Chunk {}]: Consumer queue dropped, shutting down.", chunk_id);
                            break;
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
