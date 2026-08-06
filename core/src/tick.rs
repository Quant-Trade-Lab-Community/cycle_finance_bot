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
            let is_buyer_maker = data.get("m")?.as_bool()?;
            
            Some(OwnedEvent::new_trade(symbol, price, quantity, timestamp, is_buyer_maker))
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
        } else if stream.ends_with("@forceOrder") {
            let o = data.get("o")?;
            let symbol = o.get("s")?.as_str()?;
            let side_str = o.get("S")?.as_str()?;
            let side = if side_str == "BUY" { 0 } else { 1 };
            let price = o.get("p")?.as_str()?.parse::<f64>().ok()?;
            let quantity = o.get("q")?.as_str()?.parse::<f64>().ok()?;
            let timestamp = o.get("T")?.as_u64()?;
            Some(OwnedEvent::new_liquidation(symbol, side, price, quantity, timestamp))
        } else if stream.ends_with("@markPrice") {
            let symbol = data.get("s")?.as_str()?;
            let mark_price = data.get("p")?.as_str()?.parse::<f64>().ok()?;
            let funding_rate = data.get("r")?.as_str()?.parse::<f64>().ok().unwrap_or(0.0);
            let next_funding_time = data.get("T")?.as_u64().unwrap_or(0);
            Some(OwnedEvent::new_funding_rate(symbol, mark_price, funding_rate, next_funding_time))
        } else if stream.ends_with("@bookTicker") {
            let symbol = data.get("s")?.as_str()?;
            let best_bid_price = data.get("b")?.as_str()?.parse::<f64>().ok()?;
            let best_bid_qty = data.get("B")?.as_str()?.parse::<f64>().ok()?;
            let best_ask_price = data.get("a")?.as_str()?.parse::<f64>().ok()?;
            let best_ask_qty = data.get("A")?.as_str()?.parse::<f64>().ok()?;
            Some(OwnedEvent::new_bookticker(symbol, best_bid_price, best_bid_qty, best_ask_price, best_ask_qty))
        } else {
            None
        }
    }
}
