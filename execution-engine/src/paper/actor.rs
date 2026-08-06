use crate::order::OrderRequest;
use crate::paper::account::AccountState;
use crate::paper::hybrid_book::HybridOrderBook;
use crate::paper::config::PaperConfig;
use crate::paper::db_writer::{PersistEvent, start_db_writer};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum OrderRejectReason {
    InsufficientFunds,
    MarketUnavailable,
    InsufficientDepth,
}

#[derive(Debug)]
pub struct OrderAck {
    pub order_id: String,
    pub avg_price: f64,
    pub executed_qty: f64,
}

pub enum ActorCommand {
    SubmitOrder {
        order: OrderRequest,
        response_tx: oneshot::Sender<Result<OrderAck, OrderRejectReason>>,
    },
    PriceUpdate(f64),
}

pub struct PaperEngineActor {
    config: PaperConfig,
    orderbook: HybridOrderBook,
    account: AccountState,
    db_tx: mpsc::UnboundedSender<PersistEvent>,
}

impl PaperEngineActor {
    pub fn new(config: PaperConfig) -> Self {
        let account = AccountState::new(config.initial_usdt, config.initial_btc);
        let orderbook = HybridOrderBook::new(config.slippage_model.clone(), config.market_impact_factor);
        
        let (db_tx, db_rx) = mpsc::unbounded_channel();
        let db_path = config.db_path.clone();
        let batch_interval = config.batch_write_interval_ms;
        
        tokio::spawn(async move {
            start_db_writer(db_rx, db_path, batch_interval).await;
        });

        Self {
            config,
            orderbook,
            account,
            db_tx,
        }
    }

    pub async fn run(mut self, mut cmd_rx: mpsc::UnboundedReceiver<ActorCommand>) {
        println!("PaperEngineActor: Started in {} mode.", self.config.slippage_model);
        
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                ActorCommand::SubmitOrder { order, response_tx } => {
                    let result = self.process_order(order).await;
                    let _ = response_tx.send(result);
                }
                ActorCommand::PriceUpdate(price) => {
                    self.orderbook.apply_price(price);
                }
            }
        }
    }

    async fn process_order(&mut self, order: OrderRequest) -> Result<OrderAck, OrderRejectReason> {
        // Latency & Jitter simülasyonu
        let delay = self.config.base_latency_ms + (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_millis() as u64 % (self.config.latency_jitter_ms + 1));
        sleep(Duration::from_millis(delay)).await;

        match order.order_type {
            crate::order::OrderType::Market => {
                if order.side == crate::order::OrderSide::Buy {
                    // Tahmini maliyet (Bakiye kontrolü için)
                    let estimated_price = if self.orderbook.last_price > 0.0 { self.orderbook.last_price } else { 999_999.0 };
                    let estimated_cost = order.quantity * estimated_price;
                    
                    if self.account.get_free("USDT") < estimated_cost {
                        return Err(OrderRejectReason::InsufficientFunds);
                    }

                    // Sweep Orderbook
                    match self.orderbook.sweep_buy(order.quantity) {
                        Ok(trades) => {
                            let mut total_cost = 0.0;
                            let mut total_qty = 0.0;
                            for t in trades {
                                total_cost += t.price * t.quantity;
                                total_qty += t.quantity;
                            }
                            let avg_price = total_cost / total_qty;
                            
                            // Fee Kesintisi
                            let fee = total_cost * self.config.taker_fee; // QUOTE bazında kesinti
                            
                            if self.account.get_free("USDT") < (total_cost + fee) {
                                return Err(OrderRejectReason::InsufficientFunds); // Rollback
                            }

                            // Commit State (Değişiklikleri onayla)
                            let _ = self.account.deduct_free_funds("USDT", total_cost + fee);
                            self.account.add_free_funds("BTC", total_qty);
                            
                            println!("[PAPER] M-BUY Filled: {} BTC @ {}. Fee: {} USDT", total_qty, avg_price, fee);

                            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                            let _ = self.db_tx.send(PersistEvent::Trade {
                                order_id: "PAPER_OID_MBUY".to_string(),
                                symbol: order.symbol.clone(),
                                side: "BUY".to_string(),
                                price: avg_price,
                                quantity: total_qty,
                                fee,
                                timestamp,
                            });

                            Ok(OrderAck {
                                order_id: "PAPER_OID_MBUY".to_string(),
                                avg_price,
                                executed_qty: total_qty,
                            })
                        }
                        Err(e) if e == "MARKET_UNAVAILABLE" => Err(OrderRejectReason::MarketUnavailable),
                        Err(_) => Err(OrderRejectReason::InsufficientDepth),
                    }
                } else {
                    // SELL
                    if self.account.get_free("BTC") < order.quantity {
                        return Err(OrderRejectReason::InsufficientFunds);
                    }

                    match self.orderbook.sweep_sell(order.quantity) {
                        Ok(trades) => {
                            let mut total_revenue = 0.0;
                            let mut total_qty = 0.0;
                            for t in trades {
                                total_revenue += t.price * t.quantity;
                                total_qty += t.quantity;
                            }
                            let avg_price = total_revenue / total_qty;
                            
                            let fee = total_revenue * self.config.taker_fee; // QUOTE bazında
                            
                            // Commit State
                            let _ = self.account.deduct_free_funds("BTC", total_qty);
                            self.account.add_free_funds("USDT", total_revenue - fee);
                            
                            println!("[PAPER] M-SELL Filled: {} BTC @ {}. Fee: {} USDT", total_qty, avg_price, fee);

                            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                            let _ = self.db_tx.send(PersistEvent::Trade {
                                order_id: "PAPER_OID_MSELL".to_string(),
                                symbol: order.symbol.clone(),
                                side: "SELL".to_string(),
                                price: avg_price,
                                quantity: total_qty,
                                fee,
                                timestamp,
                            });

                            Ok(OrderAck {
                                order_id: "PAPER_OID_MSELL".to_string(),
                                avg_price,
                                executed_qty: total_qty,
                            })
                        }
                        Err(e) if e == "MARKET_UNAVAILABLE" => Err(OrderRejectReason::MarketUnavailable),
                        Err(_) => Err(OrderRejectReason::InsufficientDepth),
                    }
                }
            }
            crate::order::OrderType::Limit => {
                // Şimdilik pasif olarak Limit emirleri eklemeyi atlıyoruz, sadece temel marketi destekliyoruz.
                // İleride hybrid_book'a .add_limit_order eklenebilir.
                println!("[PAPER] Limit Orders not fully implemented in v3.0 sweep logic yet.");
                Err(OrderRejectReason::MarketUnavailable)
            }
            _ => Err(OrderRejectReason::MarketUnavailable),
        }
    }
}
