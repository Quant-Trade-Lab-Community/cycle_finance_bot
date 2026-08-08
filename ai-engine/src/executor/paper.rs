//! Paper icra — emri `/cycle_finance_orders` ring'ine yazar; paper-service bridge
//! bunu alıp actor'e iletir (STRATEGY → EXECUTION yolu).

use crate::{Action, now_ms};
use rust_decimal::Decimal;
use transport::order_ring::{IpcOrderSide, IpcOrderType, OrderRingBuffer};

const ORDER_RING_CAPACITY: usize = 10_000;

pub struct PaperExecutor {
    ring: OrderRingBuffer,
}

impl PaperExecutor {
    /// Ring'i açar. shm açılamazsa `None` (panik yerine güvenli düşüş).
    pub fn new() -> Option<Self> {
        match std::panic::catch_unwind(|| OrderRingBuffer::new(ORDER_RING_CAPACITY)) {
            Ok(ring) => Some(Self { ring }),
            Err(_) => {
                eprintln!("⚠️  paper order ring (/cycle_finance_orders) açılamadı");
                None
            }
        }
    }

    pub fn execute(
        &self,
        symbol: &str,
        action: Action,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Result<String, String> {
        let side = match action {
            Action::Buy => IpcOrderSide::Buy,
            Action::Sell => IpcOrderSide::Sell,
            Action::Hold => return Err("HOLD emri gönderilmez".into()),
        };
        let order_type = if price.is_some() {
            IpcOrderType::Limit
        } else {
            IpcOrderType::Market
        };

        self.ring.push(
            symbol.as_bytes(),
            side,
            order_type,
            quantity,
            price.unwrap_or(Decimal::ZERO),
        );

        Ok(format!(
            "✅ PAPER ring: {} {} {} @ {} (ts: {})",
            action.as_str(),
            symbol,
            quantity,
            price.map(|p| p.to_string()).unwrap_or_else(|| "MARKET".into()),
            now_ms()
        ))
    }
}

