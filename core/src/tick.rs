// core/src/tick.rs

use simd_json;
use simd_json::prelude::*;
use crate::ring_buffer::OwnedEvent;

pub struct EventParser;

impl EventParser {
    #[inline(always)]
    pub fn parse(bytes: &mut [u8]) -> Option<OwnedEvent> {
        let parsed = simd_json::to_borrowed_value(bytes).ok()?;
        
        let stream = parsed.get("stream")?.as_str()?;
        let data = parsed.get("data")?;
        
        if stream.ends_with("@trade") {
            let symbol = data.get("s")?.as_str()?;
            let price_str = data.get("p")?.as_str()?;
            let quantity_str = data.get("q")?.as_str()?;
            let timestamp = data.get("T")?.as_u64()?;
            
            let price = price_str.parse::<f64>().ok()?;
            let quantity = quantity_str.parse::<f64>().ok()?;
            
            Some(OwnedEvent::new_trade(symbol, price, quantity, timestamp))
        } else if stream.contains("@depth") {
            let symbol = stream.split('@').next()?;
            let mut bids = [(0.0, 0.0); 20];
            let mut asks = [(0.0, 0.0); 20];
            
            if let Some(b) = data.get("bids").and_then(|v| v.as_array()) {
                for (i, bid) in b.iter().take(20).enumerate() {
                    if let Some(arr) = bid.as_array() {
                        let p = arr.get(0).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        let q = arr.get(1).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        bids[i] = (p, q);
                    }
                }
            }
            if let Some(a) = data.get("asks").and_then(|v| v.as_array()) {
                for (i, ask) in a.iter().take(20).enumerate() {
                    if let Some(arr) = ask.as_array() {
                        let p = arr.get(0).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        let q = arr.get(1).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        asks[i] = (p, q);
                    }
                }
            }
            
            Some(OwnedEvent::new_orderbook(symbol, bids, asks))
        } else {
            None
        }
    }
}
