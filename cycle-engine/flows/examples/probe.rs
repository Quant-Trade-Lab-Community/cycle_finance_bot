//! Prob: hangi stream'in hangi akış ayrıştırıcısıyla parse edildiğini dener.
//! cargo run -p flows --example probe

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let streams = vec![
        "btcusdt@markPrice@1s".to_string(),
        "btcusdt@indexPrice@1s".to_string(),
        "btcusdt@lastPrice@1s".to_string(),
        "btcusdt@forceOrder".to_string(),
        "!openInterest@arr".to_string(),
        "btcusdt@trade".to_string(),
    ];
    let url = "wss://fstream.binance.com/stream?streams=".to_string() + &streams.join("/");
    println!("connecting {url}");
    let (ws, _) = connect_async(&url).await.expect("connect");
    let (mut write, mut read) = ws.split();
    let sub = json!({"method": "SUBSCRIBE", "params": streams, "id": 1});
    write.send(Message::Text(sub.to_string())).await.unwrap();

    let kinds = [
        (transport::flow::FlowKind::Trade, "trade"),
        (transport::flow::FlowKind::Funding, "funding"),
        (transport::flow::FlowKind::MarkPrice, "markprice"),
        (transport::flow::FlowKind::LastPrice, "lastprice"),
        (transport::flow::FlowKind::IndexPrice, "indexprice"),
        (transport::flow::FlowKind::OpenInterest, "oi"),
        (transport::flow::FlowKind::Liquidation, "liquidation"),
    ];
    let mut parsed: std::collections::HashMap<&str, u64> = Default::default();
    let mut msgs: std::collections::HashMap<String, u64> = Default::default();
    let start = std::time::Instant::now();
    let mut total = 0u64;

    while start.elapsed().as_secs() < 12 {
        match tokio::time::timeout(std::time::Duration::from_secs(2), read.next()).await {
            Ok(Some(Ok(Message::Text(t)))) => {
                total += 1;
                let raw = t.as_bytes();
                let _ = msgs;
                for (kind, label) in &kinds {
                    let mut c = raw.to_vec();
                    let evs = flows::parse::parse_for(*kind, &mut c);
                    *parsed.entry(label).or_insert(0) += evs.len() as u64;
                }
            }
            Ok(Some(Ok(_))) => {}
            Ok(Some(Err(e))) => {
                eprintln!("err: {e}");
                break;
            }
            _ => {
                eprintln!("(timeout/closed)");
                break;
            }
        }
    }

    println!("toplam mesaj: {total}");
    for (k, v) in &parsed {
        println!("  parse → {k}: {v}");
    }
}
