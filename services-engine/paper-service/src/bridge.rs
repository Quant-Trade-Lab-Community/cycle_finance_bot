//! PAPER sistemini DATA/STRATEGY terminallerine bağlayan köprü.
//!
//! - Price-feed ring (`/demir_yumruk_pricefeed`) → `ActorCommand::MarkPriceUpdate`
//!   (tek fiyat kaynağı: mark price; dolum/likidasyon bunun üzerinden yapılır)
//! - Order ring (`/demir_yumruk_orders`) → `ActorCommand::SubmitOrder`
//!
//! Her iki okuyucu da ayrı thread'de spin-loop ile çalışır (zero-copy).

use transport::order_ring::{IpcOrderSide, IpcOrderType, OrderRingBuffer};
use transport::ring_buffer::GenerationalRingBuffer;
use contracts::events::EventType;
use execution_engine::order::{OrderPositionSide, OrderRequest, OrderSide, OrderType};
use execution_engine::paper::actor::ActorCommand;
use rust_decimal::Decimal;
use tokio::sync::mpsc::UnboundedSender;

const ORDER_RING_CAPACITY: usize = 10_000;

/// Ring buffer'lardan actor'e veri taşıyan okuyucuları başlatır.
pub fn spawn_ring_bridge(actor_tx: UnboundedSender<ActorCommand>) {
    spawn_pricefeed_reader(actor_tx.clone());
    spawn_order_reader(actor_tx);
}

/// Price-feed servisinin yazdığı ring'i (`/demir_yumruk_pricefeed`) okuyup
/// actor'e mark price güncellemesi olarak iletir. Tek veri kaynağı budur;
/// dolum ve likidasyon yalnızca mark price ile yapılır.
fn spawn_pricefeed_reader(actor_tx: UnboundedSender<ActorCommand>) {
    std::thread::spawn(move || {
        let gen_ring = GenerationalRingBuffer::with_name("/demir_yumruk_pricefeed", 20_000);
        let mut cursor = gen_ring.get_head();

        loop {
            if let Some(slot) = gen_ring.read_slot(cursor) {
                if let Some(event) = contracts::wire::decode(&slot.data[..slot.len as usize]) {
                    let symbol = decode_symbol(&event.symbol);
                    match event.payload {
                        EventType::Trade { price, .. } => {
                            let _ = actor_tx.send(ActorCommand::MarkPriceUpdate {
                                symbol,
                                mark_price: price,
                                funding_rate: Decimal::ZERO,
                                timestamp: now_ms(),
                            });
                        }
                        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
                            // Best ask öncelikli; yoksa best bid
                            let price = if best_ask_price > Decimal::ZERO {
                                best_ask_price
                            } else {
                                best_bid_price
                            };
                            if price > Decimal::ZERO {
                                let _ = actor_tx.send(ActorCommand::MarkPriceUpdate {
                                    symbol,
                                    mark_price: price,
                                    funding_rate: Decimal::ZERO,
                                    timestamp: now_ms(),
                                });
                            }
                        }
                        EventType::FundingRate { mark_price, funding_rate, next_funding_time, .. } => {
                            let _ = actor_tx.send(ActorCommand::MarkPriceUpdate {
                                symbol,
                                mark_price,
                                funding_rate,
                                timestamp: next_funding_time.max(now_ms()),
                            });
                        }
                        _ => {}
                    }
                }
                cursor += 1;
            } else {
                // Slot overwrite olmuş olabilir (üretici hızlı) — cursor'ı taşı.
                let head = gen_ring.get_head();
                if head > cursor {
                    cursor = head;
                } else {
                    std::thread::sleep(std::time::Duration::from_micros(500));
                }
            }
        }
    });
}

/// STRATEGY terminalinin yazdığı order ring'i okuyup actor'e emir olarak iletir.
fn spawn_order_reader(actor_tx: UnboundedSender<ActorCommand>) {
    std::thread::spawn(move || {
        let order_ring = OrderRingBuffer::new(ORDER_RING_CAPACITY);
        let mut cursor = order_ring.get_head();

        loop {
            if let Some(slot) = order_ring.read_slot(cursor) {
                let symbol = decode_symbol(&slot.symbol);
                // HEDGE modda BUY → LONG, SELL → SHORT kabul edilir; one-way'de yok sayılır.
                let position_side = match slot.side {
                    IpcOrderSide::Buy => OrderPositionSide::Long,
                    IpcOrderSide::Sell => OrderPositionSide::Short,
                };
                let order = OrderRequest {
                    symbol,
                    side: match slot.side {
                        IpcOrderSide::Buy => OrderSide::Buy,
                        IpcOrderSide::Sell => OrderSide::Sell,
                    },
                    order_type: match slot.order_type {
                        IpcOrderType::Limit => OrderType::Limit,
                        IpcOrderType::Market => OrderType::Market,
                    },
                    quantity: slot.quantity,
                    price: if slot.price > Decimal::ZERO { Some(slot.price) } else { None },
                    time_in_force: None,
                    position_side,
                };

                let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                let _ = actor_tx.send(ActorCommand::SubmitOrder { order, response_tx: resp_tx });

                // Yanıtı bekle (std thread, reactor yok → blocking_recv)
                if let Ok(res) = resp_rx.blocking_recv() {
                    tracing::debug!("Paper order response: {:?}", res);
                }

                cursor += 1;
            } else {
                std::thread::sleep(std::time::Duration::from_micros(500));
            }
        }
    });
}

fn decode_symbol(buf: &[u8; 16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(16);
    String::from_utf8_lossy(&buf[..len]).to_string()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
