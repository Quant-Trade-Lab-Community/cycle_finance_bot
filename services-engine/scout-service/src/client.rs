use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde_json::Value;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::models::{now_ts, BINANCE_REST, BINANCE_WS, WS_BACKOFF_BASE_SECS, WS_BACKOFF_CAP_SECS, WS_HEARTBEAT_SECS};

pub type Handler = Box<dyn FnMut(Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

pub struct BinanceClient {
    http: reqwest::Client,
}

impl BinanceClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .pool_max_idle_per_host(8)
            .build()
            .expect("reqwest client build failed");
        Self { http }
    }

    pub async fn fetch_symbols(&self) -> Result<Vec<String>, reqwest::Error> {
        let url = format!("{}/fapi/v1/exchangeInfo", BINANCE_REST);
        let data: Value = self.http.get(&url).send().await?.json().await?;

        let mut symbols: Vec<String> = data["symbols"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|sym| {
                        sym["symbol"].as_str().map_or(false, |s| s.ends_with("USDT"))
                            && sym["status"].as_str() == Some("TRADING")
                            && sym["contractType"].as_str() == Some("PERPETUAL")
                    })
                    .filter_map(|sym| sym["symbol"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        symbols.sort();
        Ok(symbols)
    }

    pub async fn stream_book_tickers(&self, symbols: &[String], handler: Handler) {
        let streams: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}@bookTicker", s.to_lowercase()))
            .collect();
        self.stream_loop(streams, "bookTicker", handler).await;
    }

    pub async fn stream_partial_depths(&self, symbols: &[String], handler: Handler) {
        let suffix = format!("depth{}@{}", crate::models::DEPTH_LEVELS, crate::models::DEPTH_UPDATE_SPEED);
        let streams: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}@{}", s.to_lowercase(), suffix))
            .collect();
        self.stream_loop(streams, "partialDepth", handler).await;
    }

    async fn stream_loop(&self, streams: Vec<String>, stream_name: &'static str, mut handler: Handler) {
        let mut backoff = WS_BACKOFF_BASE_SECS;

        loop {
            match connect_async(BINANCE_WS).await {
                Ok((ws, _)) => {
                    backoff = WS_BACKOFF_BASE_SECS;
                    let (mut write, mut read) = ws.split();
                    let sub = serde_json::json!({
                        "method": "SUBSCRIBE",
                        "params": streams,
                        "id": 1
                    });
                    if write
                        .send(Message::Text(sub.to_string()))
                        .await
                        .is_err()
                    {
                        eprintln!("{} abonelik gonderilemedi", stream_name);
                        backoff = WS_BACKOFF_BASE_SECS;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }

                    let mut heartbeat =
                        tokio::time::interval(Duration::from_secs(WS_HEARTBEAT_SECS));
                    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

                    loop {
                        tokio::select! {
                            _ = heartbeat.tick() => {
                                if write.send(Message::Ping(Vec::new().into())).await.is_err() {
                                    break;
                                }
                            }
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        let payload: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                                        if payload.is_null() {
                                            continue;
                                        }
                                        let data = payload.get("data").cloned().unwrap_or(payload);
                                        handler(data).await;
                                    }
                                    Some(Ok(Message::Ping(p))) => {
                                        if write.send(Message::Pong(p)).await.is_err() {
                                            break;
                                        }
                                    }
                                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{} stream hatasi: {}", stream_name, err);
                }
            }

            let jitter: f64 = rand::thread_rng().gen_range(0.0..0.5);
            let sleep_for = (backoff + jitter).min(WS_BACKOFF_CAP_SECS);
            tokio::time::sleep(Duration::from_secs_f64(sleep_for)).await;
            backoff = (backoff * 2.0).min(WS_BACKOFF_CAP_SECS);
        }
    }
}

pub fn event_ts(data: &Value) -> f64 {
    let raw = data["T"].as_u64().or_else(|| data["E"].as_u64());
    match raw {
        Some(ts) => ts as f64 / 1000.0,
        None => now_ts(),
    }
}

pub fn chunked(items: &[String], size: usize) -> Vec<Vec<String>> {
    items.chunks(size).map(|c| c.to_vec()).collect()
}
