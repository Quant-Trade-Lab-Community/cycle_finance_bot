//! Prob: flow'ların kullandığı birebir WS desenini (bare connect + SUBSCRIBE) dener.
//! cargo run -p flows --example probe2 -- <stream...>

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let streams: Vec<String> = std::env::args().skip(1).collect();
    let url = "wss://fstream.binance.com/stream";
    println!("connect {url} streams={streams:?}");
    let (ws, _) = connect_async(url).await.expect("connect");
    let (mut write, mut read) = ws.split();
    let sub = json!({"method": "SUBSCRIBE", "params": streams, "id": 1});
    write.send(Message::Text(sub.to_string())).await.unwrap();

    let mut per_stream: HashMap<String, u64> = Default::default();
    let mut total = 0u64;
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < 10 {
        match tokio::time::timeout(std::time::Duration::from_secs(3), read.next()).await {
            Ok(Some(Ok(Message::Text(t)))) => {
                total += 1;
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) {
                    let s = v.get("stream").and_then(|x| x.as_str()).unwrap_or("(ack/other)").to_string();
                    *per_stream.entry(s).or_insert(0) += 1;
                }
            }
            Ok(Some(Ok(Message::Close(_)))) => { println!("CLOSE"); break; }
            Ok(Some(Err(e))) => { eprintln!("err: {e}"); break; }
            _ => { eprintln!("timeout/closed"); break; }
        }
    }
    println!("toplam mesaj: {total}");
    for (k, v) in &per_stream { println!("  {k}: {v}"); }
}
