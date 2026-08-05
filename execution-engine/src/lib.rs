pub mod order;
pub mod signer;

use flume::Receiver;
use order::OrderRequest;
use signer::BinanceSigner;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;

pub async fn start_execution_engine(rx: Receiver<OrderRequest>, api_key: String, secret_key: String) {
    let ws_url = "wss://ws-api.binance.com:443/ws-api/v3";
    let signer = Arc::new(BinanceSigner::new(api_key, secret_key));

    loop {
        println!("ExecutionEngine: Connecting to Binance WS Order API...");
        match connect_async(ws_url).await {
            Ok((mut ws_stream, _)) => {
                println!("ExecutionEngine: Successfully connected to Order API.");

                while let Ok(order_req) = rx.recv_async().await {
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();

                    // Parametreleri Query String formatında hazırla (İmza için zorunlu)
                    let mut query_params = format!(
                        "apiKey={}&quantity={}&side={:?}&symbol={}&timestamp={}&type={:?}",
                        signer.api_key(),
                        order_req.quantity,
                        order_req.side,
                        order_req.symbol,
                        timestamp,
                        order_req.order_type
                    );

                    if let Some(price) = order_req.price {
                        query_params.push_str(&format!("&price={}", price));
                    }
                    if let Some(tif) = &order_req.time_in_force {
                        query_params.push_str(&format!("&timeInForce={:?}", tif));
                    }

                    // HMAC-SHA256 ile imzala
                    let signature = signer.sign(&query_params);

                    // WebSoket JSON Payload'ını hazırla
                    let mut params_json = json!({
                        "apiKey": signer.api_key(),
                        "symbol": order_req.symbol,
                        "side": order_req.side,
                        "type": order_req.order_type,
                        "quantity": order_req.quantity,
                        "timestamp": timestamp,
                        "signature": signature
                    });

                    if let Some(price) = order_req.price {
                        params_json["price"] = json!(price);
                    }
                    if let Some(tif) = &order_req.time_in_force {
                        params_json["timeInForce"] = json!(tif);
                    }

                    let ws_payload = json!({
                        "id": timestamp,
                        "method": "order.place",
                        "params": params_json
                    });

                    // Borsaya fırlat
                    let payload_str = ws_payload.to_string();
                    if let Err(e) = ws_stream.send(Message::Text(payload_str)).await {
                        println!("ExecutionEngine Error: Failed to send order: {}", e);
                        break; // Reconnect
                    }

                    // (Opsiyonel) Cevabı bekle
                    // if let Some(Ok(response)) = ws_stream.next().await {
                    //     println!("Order Response: {:?}", response);
                    // }
                }
            }
            Err(e) => {
                println!("ExecutionEngine Error: Connection failed: {}. Retrying in 3s...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }
    }
}
