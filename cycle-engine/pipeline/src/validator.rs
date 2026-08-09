use transport::events::{OwnedEvent, EventType};
use rust_decimal::Decimal;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct DataValidator {
    pub circuit_breaker: Arc<AtomicBool>,
    pub bad_tick_count: Arc<AtomicUsize>,
    max_latency_ms: u64,
    last_reset_time: u64,
}

impl DataValidator {
    pub fn new() -> Self {
        Self {
            circuit_breaker: Arc::new(AtomicBool::new(false)),
            bad_tick_count: Arc::new(AtomicUsize::new(0)),
            max_latency_ms: 200, // 200 ms gecikme toleransı
            last_reset_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        }
    }

    #[inline(always)]
    pub fn is_valid(&mut self, event: &OwnedEvent) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        
        // Şalter sıfırlama mantığı (1 saniyede bir hata sayacını sıfırla)
        if now - self.last_reset_time > 1000 {
            self.bad_tick_count.store(0, Ordering::Relaxed);
            self.last_reset_time = now;
            
            // Eğer şalter daha önce attıysa ama sular durulduysa şalteri kaldır
            if self.circuit_breaker.load(Ordering::Relaxed) {
                println!("CIRCUIT BREAKER RECOVERED. Safe to trade.");
                self.circuit_breaker.store(false, Ordering::Release);
            }
        }

        match &event.payload {
            EventType::Trade { price, quantity, timestamp, is_buyer_maker: _ } => {
                if *price <= Decimal::ZERO || *quantity <= Decimal::ZERO {
                    return self.flag_invalid("Trade price/qty <= 0");
                }
                if now > *timestamp && (now - *timestamp) > self.max_latency_ms {
                    return self.flag_invalid("Trade Stale Data (Latency)");
                }
                if *timestamp > now && (*timestamp - now) > 5000 {
                    return self.flag_invalid("Trade Future Timestamp (NTP Drift)");
                }
            },
            EventType::Orderbook { bids, asks } => {
                if bids[0].0 > Decimal::ZERO && asks[0].0 > Decimal::ZERO {
                    if bids[0].0 >= asks[0].0 {
                        return self.flag_invalid("Crossed Orderbook (Bid >= Ask)");
                    }
                }
            },
            EventType::Liquidation { price, quantity, timestamp, .. } => {
                if *price <= Decimal::ZERO || *quantity <= Decimal::ZERO {
                    return self.flag_invalid("Liquidation price/qty <= 0");
                }
                if now > *timestamp && (now - *timestamp) > self.max_latency_ms {
                    return self.flag_invalid("Liquidation Stale Data");
                }
            },
            EventType::BookTicker { best_bid_price, best_ask_price, .. } => {
                if *best_bid_price > Decimal::ZERO && *best_ask_price > Decimal::ZERO {
                    if *best_bid_price >= *best_ask_price {
                        return self.flag_invalid("Crossed BookTicker (Bid >= Ask)");
                    }
                }
            },
            _ => {}
        }
        
        true
    }
    
    #[inline(always)]
    fn flag_invalid(&self, _reason: &str) -> bool {
        let count = self.bad_tick_count.fetch_add(1, Ordering::Relaxed);
        
        // Eğer 1 saniyede 100'den fazla bozuk veri gelirse ŞALTER ATAR
        if count > 100 {
            if !self.circuit_breaker.load(Ordering::Relaxed) {
                println!("[!] ⚠️ CIRCUIT BREAKER TRIGGERED! HFT Trading Paused. Reason: {}", _reason);
                self.circuit_breaker.store(true, Ordering::Release);
            }
        }
        false
    }
}
