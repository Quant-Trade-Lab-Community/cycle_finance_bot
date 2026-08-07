//! PAPER sistemini DATA/STRATEGY terminallerine bağlayan köprü.
//!
//! - Tick ring (`/dev/shm/demir_yumruk_ring`) → `ActorCommand::PriceUpdate`
//!   (gerçek Binance Futures fiyat verisi, order book simülasyonu olmadan)
//! - Order ring (`/dev/shm/demir_yumruk_orders`) → `ActorCommand::SubmitOrder`
//!
//! Her iki okuyucu da ayrı thread'de spin-loop ile çalışır (zero-copy).

use proje_core::memory::order_ring::{IpcOrderSide, IpcOrderType, OrderRingBuffer};
use proje_core::memory::ring_buffer::GenerationalRingBuffer;
use proje_core::ring_buffer::EventType;
use proje_core::tick::EventParser;
use execution_engine::order::{OrderRequest, OrderSide, OrderType};
use execution_engine::paper::actor::ActorCommand;
use rust_decimal::Decimal;
use tokio::sync::mpsc::UnboundedSender;

const TICK_RING_CAPACITY: usize = 160_000;
const ORDER_RING_CAPACITY: usize = 10_000;

/// Ring buffer'lardan actor'e veri taşıyan okuyucuları başlatır.
pub fn spawn_ring_bridge(actor_tx: UnboundedSender<ActorCommand>) {
    spawn_tick_reader(actor_tx.clone());
    spawn_pricefeed_reader(actor_tx.clone());
    spawn_order_reader(actor_tx);
}

/// DATA terminalinin yazdığı tick ring'i okuyup fiyat güncellemesi olarak iletir.
fn spawn_tick_reader(actor_tx: UnboundedSender<ActorCommand>) {
    std::thread::spawn(move || {
        let gen_ring = GenerationalRingBuffer::new(TICK_RING_CAPACITY);
        let mut cursor = gen_ring.get_head();

        loop {
            if let Some(slot) = gen_ring.read_slot(cursor) {
                let mut data = slot.data[..slot.len as usize].to_vec();
                if let Some(event) = EventParser::parse(&mut data) {
                    match event.payload {
                        EventType::Trade { price, .. } => {
                            let symbol = decode_symbol(&event.symbol);
                            let _ = actor_tx.send(ActorCommand::PriceUpdate { symbol, price });
                        }
                        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
                            // Best ask öncelikli; yoksa best bid
                            let price = if best_ask_price > Decimal::ZERO {
                                best_ask_price
                            } else {
                                best_bid_price
                            };
                            if price > Decimal::ZERO {
                                let symbol = decode_symbol(&event.symbol);
                                let _ = actor_tx.send(ActorCommand::PriceUpdate { symbol, price });
                            }
                        }
                        EventType::FundingRate { mark_price, funding_rate, next_funding_time, .. } => {
                            let symbol = decode_symbol(&event.symbol);
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
                std::thread::sleep(std::time::Duration::from_micros(500));
            }
        }
    });
}

/// Price-feed servisinin yazdığı ring'i (`/demir_yumruk_pricefeed`) okuyup
/// actor'e fiyat güncellemesi olarak iletir. DATA terminali kapalı olsa bile
/// fiyat beslemesini sağlar.
fn spawn_pricefeed_reader(actor_tx: UnboundedSender<ActorCommand>) {
    std::thread::spawn(move || {
        let gen_ring = GenerationalRingBuffer::with_name("/demir_yumruk_pricefeed", 20_000);
        let mut cursor = gen_ring.get_head();

        loop {
            if let Some(slot) = gen_ring.read_slot(cursor) {
                let mut data = slot.data[..slot.len as usize].to_vec();
                if let Some(event) = EventParser::parse(&mut data) {
                    match event.payload {
                        EventType::Trade { price, .. } => {
                            let symbol = decode_symbol(&event.symbol);
                            let _ = actor_tx.send(ActorCommand::PriceUpdate { symbol, price });
                        }
                        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
                            let price = if best_ask_price > Decimal::ZERO {
                                best_ask_price
                            } else {
                                best_bid_price
                            };
                            if price > Decimal::ZERO {
                                let symbol = decode_symbol(&event.symbol);
                                let _ = actor_tx.send(ActorCommand::PriceUpdate { symbol, price });
                            }
                        }
                        EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time } => {
                            let symbol = decode_symbol(&event.symbol);
                            let _ = actor_tx.send(ActorCommand::MarkPriceUpdate {
                                symbol: symbol.clone(),
                                mark_price,
                                funding_rate,
                                timestamp: next_funding_time.max(now_ms()),
                            });
                            // index price'ı da ayrı fiyat güncellemesi olarak ilet
                            if index_price > Decimal::ZERO {
                                let _ = actor_tx.send(ActorCommand::PriceUpdate { symbol, price: index_price });
                            }
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
