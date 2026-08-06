use crate::order::{OrderRequest, OrderSide, OrderType};
use crate::paper::account::AccountState;
use crate::paper::domain_event::DomainEvent;
use crate::paper::hybrid_book::HybridOrderBook;
use crate::paper::config::PaperConfig;
use crate::paper::db_writer::{PersistEvent, start_db_writer};
use crate::paper::position::{PositionManager, PositionSide};
use crate::paper::risk::RiskManager;
use crate::paper::snapshot::{PaperSnapshot, TradeView};
use parking_lot::RwLock;
use rust_decimal::Decimal;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum OrderRejectReason {
    InsufficientFunds,
    MarketUnavailable,
    InsufficientDepth,
    RiskRejected(String),
}

#[derive(Debug)]
pub struct OrderAck {
    pub order_id: String,
    pub avg_price: Decimal,
    pub executed_qty: Decimal,
}

pub enum ActorCommand {
    SubmitOrder {
        order: OrderRequest,
        response_tx: oneshot::Sender<Result<OrderAck, OrderRejectReason>>,
    },
    PriceUpdate(Decimal),
    MarkPriceUpdate {
        mark_price: Decimal,
        funding_rate: Decimal,
        timestamp: u64,
    },
}

/// PRICE_ONLY modunda bekleyen limit emri
#[derive(Debug, Clone)]
pub struct OpenOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub remaining: Decimal,
    pub limit_price: Decimal,
    pub leverage: Decimal,
}

pub struct PaperEngineActor {
    config: PaperConfig,
    orderbook: HybridOrderBook,
    account: AccountState,
    positions: PositionManager,
    risk: RiskManager,
    open_orders: Vec<OpenOrder>,
    db_tx: mpsc::UnboundedSender<PersistEvent>,
    event_tx: Option<mpsc::UnboundedSender<DomainEvent>>,
    last_funding_ts: u64,
    mark_prices: HashMap<String, Decimal>,
    funding_rates: HashMap<String, Decimal>,
    recent_trades: Vec<TradeView>,
    snapshot: Arc<RwLock<PaperSnapshot>>,
}

impl PaperEngineActor {
    pub fn new(config: PaperConfig) -> Self {
        Self::new_with_events(config, None, &[])
    }

    /// Event sink'i ve başlangıç event'leri (replay) ile yeni actor.
    pub fn new_with_events(
        config: PaperConfig,
        event_tx: Option<mpsc::UnboundedSender<DomainEvent>>,
        replay_events: &[DomainEvent],
    ) -> Self {
        let account = AccountState::new(config.initial_usdt, config.initial_btc);
        let orderbook = HybridOrderBook::new(config.slippage_model.clone(), config.market_impact_factor);
        let risk = RiskManager::new(
            config.initial_usdt,
            config.max_position_qty,
            config.max_leverage,
            config.max_drawdown_pct,
            config.max_daily_loss,
        );

        let (db_tx, db_rx) = mpsc::unbounded_channel();
        let db_path = config.db_path.clone();
        let batch_interval = config.batch_write_interval_ms;

        tokio::spawn(async move {
            start_db_writer(db_rx, db_path, batch_interval).await;
        });

        let mut actor = Self {
            config,
            orderbook,
            account,
            positions: PositionManager::new(),
            risk,
            open_orders: Vec::new(),
            db_tx,
            event_tx,
            last_funding_ts: 0,
            mark_prices: HashMap::new(),
            funding_rates: HashMap::new(),
            recent_trades: Vec::new(),
            snapshot: Arc::new(RwLock::new(PaperSnapshot::build(
                Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
                crate::paper::risk::RiskStatus::Ok, Decimal::ZERO,
                &PositionManager::new(), 0, vec![],
            ))),
        };

        if !replay_events.is_empty() {
            actor.rebuild_from_events(replay_events);
        }

        actor.publish_snapshot();
        actor
    }

    /// API/CLI okumaları için paylaşılan snapshot'ı günceller.
    pub fn publish_snapshot(&mut self) {
        let snap = PaperSnapshot::build(
            self.account.get_free("USDT"),
            self.equity(),
            self.risk.realized_pnl,
            self.account.get_locked("USDT"),
            self.risk.status,
            self.orderbook.last_price,
            &self.positions,
            self.open_orders.len(),
            self.recent_trades.clone(),
        );
        *self.snapshot.write() = snap;
    }

    pub fn snapshot_handle(&self) -> Arc<RwLock<PaperSnapshot>> {
        self.snapshot.clone()
    }

    /// Event replay'i ile state'i yeniden inşa eder.
    pub fn rebuild_from_events(&mut self, events: &[DomainEvent]) {
        let mut replayed_fills = 0usize;
        for ev in events {
            match ev {
                DomainEvent::OrderFilled { symbol, side, fill_price, fill_qty, commission, cash_delta, realized_pnl, leverage, .. } => {
                    let signed = if side == "BUY" { *fill_qty } else { -*fill_qty };
                    let _ = self.positions.apply_fill(symbol, signed, *fill_price, *leverage);
                    self.account.add_free_funds("USDT", *cash_delta);
                    self.risk.record_realized(*realized_pnl);
                    let _ = commission;
                    replayed_fills += 1;
                }
                DomainEvent::FundingRateApplied { payment, .. } => {
                    self.account.add_free_funds("USDT", *payment);
                }
                _ => {}
            }
        }
        if replayed_fills > 0 {
            println!("[PAPER] Replayed {} fill events for state recovery.", replayed_fills);
        }
    }

    #[inline]
    fn emit(&self, event: DomainEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }

    pub async fn run(mut self, mut cmd_rx: mpsc::UnboundedReceiver<ActorCommand>) {
        println!("PaperEngineActor: Started in {} mode.", self.config.slippage_model);
        println!("PaperEngineActor: Matching mode = {}", self.config.matching_mode);

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                ActorCommand::SubmitOrder { order, response_tx } => {
                    let result = self.process_order(order).await;
                    let _ = response_tx.send(result);
                }
                ActorCommand::PriceUpdate(price) => {
                    self.orderbook.apply_price(price);
                    self.check_limit_orders(price);
                }
                ActorCommand::MarkPriceUpdate { mark_price, funding_rate, timestamp } => {
                    self.mark_prices.insert("BTCUSDT".to_string(), mark_price);
                    self.funding_rates.insert("BTCUSDT".to_string(), funding_rate);
                    self.on_mark_tick(timestamp);
                }
            }
            self.publish_snapshot();
        }
    }

    pub fn last_price(&self) -> Decimal {
        self.orderbook.last_price
    }

    pub fn account(&self) -> &AccountState {
        &self.account
    }

    pub fn positions(&self) -> &PositionManager {
        &self.positions
    }

    pub fn risk(&self) -> &RiskManager {
        &self.risk
    }

    pub fn open_orders(&self) -> &[OpenOrder] {
        &self.open_orders
    }

    pub fn equity(&self) -> Decimal {
        self.risk.equity(&self.positions, &self.mark_prices, self.account.get_free("USDT"))
    }

    async fn process_order(&mut self, order: OrderRequest) -> Result<OrderAck, OrderRejectReason> {
        // Latency & Jitter simülasyonu
        let delay = self.config.base_latency_ms
            + (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_millis() as u64 % (self.config.latency_jitter_ms + 1));
        sleep(Duration::from_millis(delay)).await;

        if self.config.matching_mode == "PRICE_ONLY" {
            return self.process_price_only(order);
        }

        self.process_sweep(order).await
    }

    // ─────────────────────────────────────────────────────────────
    // PRICE_ONLY: gerçek fiyat verisiyle (order book'suz) dolum
    // ─────────────────────────────────────────────────────────────
    fn process_price_only(&mut self, order: OrderRequest) -> Result<OrderAck, OrderRejectReason> {
        let last = self.orderbook.last_price;
        if last <= Decimal::ZERO {
            return Err(OrderRejectReason::MarketUnavailable);
        }

        let leverage = self.config.max_leverage.min(Decimal::ONE.max(self.config.max_leverage));
        let order_id = format!("PAPER_{}", now_ms());

        match order.order_type {
            OrderType::Market => {
                // Risk kontrolü (marj/pozisyon)
                if let Err(msg) = self.risk.check_order(
                    &self.positions, &order.symbol, order.quantity, last, leverage,
                    self.account.get_free("USDT"),
                ) {
                    return Err(OrderRejectReason::RiskRejected(msg.to_string()));
                }

                self.emit(DomainEvent::OrderCreated {
                    order_id: order_id.clone(),
                    client_oid: order_id.clone(),
                    symbol: order.symbol.clone(),
                    side: format!("{:?}", order.side).to_uppercase(),
                    order_type: "MARKET".to_string(),
                    qty: order.quantity,
                    price: Some(last),
                });

                // Net marj: pozisyon büyüklüğündeki değişime göre
                let before_qty = self.positions.get(&order.symbol).map(|p| p.quantity.abs()).unwrap_or(Decimal::ZERO);
                let after_qty = (before_qty - order.quantity).abs();
                let margin_released = if before_qty > after_qty { (before_qty - after_qty) * last / leverage } else { Decimal::ZERO };
                let margin_locked = if after_qty > before_qty { (after_qty - before_qty) * last / leverage } else { Decimal::ZERO };

                match order.side {
                    OrderSide::Buy => {
                        let fee = (order.quantity * last) * self.config.taker_fee;
                        if self.account.get_free("USDT") < (margin_locked + fee) {
                            return Err(OrderRejectReason::InsufficientFunds);
                        }
                        if margin_locked > Decimal::ZERO {
                            let _ = self.account.lock_funds("USDT", margin_locked);
                        }
                        if margin_released > Decimal::ZERO {
                            self.account.unlock_funds("USDT", margin_released);
                        }
                        let _ = self.account.deduct_free_funds("USDT", fee);
                        self.account.add_free_funds("BTC", order.quantity);
                        let (realized, _) = self.positions.apply_fill(&order.symbol, order.quantity, last, leverage);
                        self.risk.record_realized(realized);
                        self.account.add_free_funds("USDT", realized);
                        self.emit_fill(&order_id, &order.symbol, "BUY", last, order.quantity, fee, realized, leverage, margin_released, margin_locked);
                        self.persist_trade(&order.symbol, "BUY", last, order.quantity, fee);
                        Ok(OrderAck { order_id, avg_price: last, executed_qty: order.quantity })
                    }
                    OrderSide::Sell => {
                        let fee = (order.quantity * last) * self.config.taker_fee;
                        if self.account.get_free("USDT") < (margin_locked + fee) {
                            return Err(OrderRejectReason::InsufficientFunds);
                        }
                        if margin_locked > Decimal::ZERO {
                            let _ = self.account.lock_funds("USDT", margin_locked);
                        }
                        if margin_released > Decimal::ZERO {
                            self.account.unlock_funds("USDT", margin_released);
                        }
                        let _ = self.account.deduct_free_funds("USDT", fee);
                        self.account.subtract_free_funds_unchecked("BTC", order.quantity); // short: borçlan
                        let (realized, _) = self.positions.apply_fill(&order.symbol, -order.quantity, last, leverage);
                        self.risk.record_realized(realized);
                        self.account.add_free_funds("USDT", realized);
                        self.emit_fill(&order_id, &order.symbol, "SELL", last, order.quantity, fee, realized, leverage, margin_released, margin_locked);
                        self.persist_trade(&order.symbol, "SELL", last, order.quantity, fee);
                        Ok(OrderAck { order_id, avg_price: last, executed_qty: order.quantity })
                    }
                }
            }
            OrderType::Limit => {
                let limit_price = order.price.unwrap_or(last);
                if let Err(msg) = self.risk.check_order(
                    &self.positions, &order.symbol, order.quantity, limit_price, leverage,
                    self.account.get_free("USDT"),
                ) {
                    return Err(OrderRejectReason::RiskRejected(msg.to_string()));
                }

                // Marj için fonları kilitle
                let margin = (order.quantity * limit_price) / leverage;
                if self.account.get_free("USDT") < margin {
                    return Err(OrderRejectReason::InsufficientFunds);
                }
                let _ = self.account.lock_funds("USDT", margin);

                self.emit(DomainEvent::OrderCreated {
                    order_id: order_id.clone(),
                    client_oid: order_id.clone(),
                    symbol: order.symbol.clone(),
                    side: format!("{:?}", order.side).to_uppercase(),
                    order_type: "LIMIT".to_string(),
                    qty: order.quantity,
                    price: Some(limit_price),
                });

                // Fiyat zaten seviyeyi geçtiyse anında doldur
                let crossed = match order.side {
                    OrderSide::Buy => last <= limit_price,
                    OrderSide::Sell => last >= limit_price,
                };

                if crossed {
                    self.fill_limit(&order_id, &order.symbol, order.side, order.quantity, limit_price, leverage, margin);
                    return Ok(OrderAck { order_id, avg_price: limit_price, executed_qty: order.quantity });
                }

                self.open_orders.push(OpenOrder {
                    order_id,
                    symbol: order.symbol.clone(),
                    side: order.side,
                    quantity: order.quantity,
                    remaining: order.quantity,
                    limit_price,
                    leverage,
                });
                Ok(OrderAck { order_id: "PENDING".to_string(), avg_price: Decimal::ZERO, executed_qty: Decimal::ZERO })
            }
            _ => Err(OrderRejectReason::MarketUnavailable),
        }
    }

    fn emit_fill(
        &self,
        order_id: &str,
        symbol: &str,
        side: &str,
        price: Decimal,
        qty: Decimal,
        fee: Decimal,
        realized: Decimal,
        leverage: Decimal,
        margin_released: Decimal,
        margin_locked: Decimal,
    ) {
        let cash_delta = margin_released - margin_locked + realized - fee;
        self.emit(DomainEvent::OrderFilled {
            order_id: order_id.to_string(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            fill_price: price,
            fill_qty: qty,
            commission: fee,
            cash_delta,
            realized_pnl: realized,
            leverage,
        });
    }

    fn check_limit_orders(&mut self, price: Decimal) {
        if self.config.matching_mode != "PRICE_ONLY" {
            return;
        }
        let mut filled: Vec<usize> = Vec::new();
        let mut fill_data: Vec<(String, String, OrderSide, Decimal, Decimal, Decimal, Decimal)> = Vec::new();
        for (i, o) in self.open_orders.iter().enumerate() {
            let crossed = match o.side {
                OrderSide::Buy => price <= o.limit_price,
                OrderSide::Sell => price >= o.limit_price,
            };
            if crossed {
                fill_data.push((o.order_id.clone(), o.symbol.clone(), o.side, o.remaining, o.limit_price, o.leverage, o.quantity * o.limit_price / o.leverage));
                filled.push(i);
            }
        }
        for (order_id, symbol, side, qty, limit_price, leverage, margin) in fill_data {
            self.fill_limit(&order_id, &symbol, side, qty, limit_price, leverage, margin);
        }
        for i in filled.into_iter().rev() {
            self.open_orders.remove(i);
        }
    }

    fn fill_limit(&mut self, order_id: &str, symbol: &str, side: OrderSide, qty: Decimal, price: Decimal, leverage: Decimal, margin_locked: Decimal) {
        let fee = (qty * price) * self.config.maker_fee;
        let before_qty = self.positions.get(symbol).map(|p| p.quantity.abs()).unwrap_or(Decimal::ZERO);
        let after_qty = (before_qty - qty).abs();
        let margin_released = if before_qty > after_qty { (before_qty - after_qty) * price / leverage } else { Decimal::ZERO };
        let margin_net_locked = if after_qty > before_qty { (after_qty - before_qty) * price / leverage } else { Decimal::ZERO };

        // Bekleyen emrin kilitlediği marjı serbest bırak, net artışı tekrar kilitle
        self.account.unlock_funds("USDT", margin_locked);
        if margin_net_locked > Decimal::ZERO {
            let _ = self.account.lock_funds("USDT", margin_net_locked);
        }
        let _ = self.account.deduct_free_funds("USDT", fee);

        match side {
            OrderSide::Buy => {
                self.account.add_free_funds("BTC", qty);
                let (realized, _) = self.positions.apply_fill(symbol, qty, price, leverage);
                self.risk.record_realized(realized);
                self.account.add_free_funds("USDT", realized);
                self.emit_fill(order_id, symbol, "BUY", price, qty, fee, realized, leverage, margin_released, margin_net_locked);
            }
            OrderSide::Sell => {
                self.account.subtract_free_funds_unchecked("BTC", qty);
                let (realized, _) = self.positions.apply_fill(symbol, -qty, price, leverage);
                self.risk.record_realized(realized);
                self.account.add_free_funds("USDT", realized);
                self.emit_fill(order_id, symbol, "SELL", price, qty, fee, realized, leverage, margin_released, margin_net_locked);
            }
        }
        let side_str = match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };
        self.persist_trade(symbol, side_str, price, qty, fee);
        println!("[PAPER] LIMIT {} Filled: {} @ {}. Fee: {} USDT", side_str, qty, price, fee);
    }

    // ─────────────────────────────────────────────────────────────
    // MARK PRICE TICK: likidasyon, drawdown, funding
    // ─────────────────────────────────────────────────────────────
    fn on_mark_tick(&mut self, timestamp: u64) {
        let cash = self.account.get_free("USDT");
        let liquidated = self.risk.on_mark_tick(&self.positions, &self.mark_prices, cash);

        // Funding: her 8 saatte bir (28_800_000 ms)
        let funding_interval_ms = 28_800_000u64;
        if self.last_funding_ts == 0 {
            self.last_funding_ts = timestamp;
        } else if timestamp.saturating_sub(self.last_funding_ts) >= funding_interval_ms {
            self.apply_funding();
            self.last_funding_ts = timestamp;
        }

        // Likidasyon: pozisyonları mark fiyatından kapat
        for sym in liquidated {
            if let Some(pos) = self.positions.get(&sym).cloned() {
                let mark = *self.mark_prices.get(&sym).unwrap_or(&pos.avg_entry_price);
                let side_str = match pos.side { PositionSide::Long => "BUY", PositionSide::Short => "SELL" };
                let closing_side = match pos.side { PositionSide::Long => "SELL", PositionSide::Short => "BUY" };
                let (realized, _) = self.positions.apply_fill(&sym, -pos.quantity, mark, pos.leverage);
                self.risk.record_realized(realized);
                // Marjı serbest bırak ve realizasyonu cash'e ekle
                let margin = (pos.quantity.abs() * pos.avg_entry_price) / pos.leverage;
                self.account.unlock_funds("USDT", margin);
                self.account.add_free_funds("USDT", realized);
                let order_id = format!("PAPER_LIQ_{}", now_ms());
                self.emit_fill(&order_id, &sym, closing_side, mark, pos.quantity.abs(), Decimal::ZERO, realized, pos.leverage, margin, Decimal::ZERO);
                self.emit(DomainEvent::Liquidation {
                    symbol: sym.clone(),
                    side: side_str.to_string(),
                    price: mark,
                    qty: pos.quantity.abs(),
                });
                self.persist_trade(&sym, side_str, mark, pos.quantity.abs(), Decimal::ZERO);
                println!("[PAPER] ⚠️ LIQUIDATION: {} @ {}", sym, mark);
            }
        }
    }

    fn apply_funding(&mut self) {
        for (sym, pos) in self.positions.all().clone() {
            let rate = *self.funding_rates.get(&sym).unwrap_or(&Decimal::ZERO);
            // Binance funding_rate, 8 saatlik periyot başına verilir (per-interval)
            let payment = pos.notional(*self.mark_prices.get(&sym).unwrap_or(&pos.avg_entry_price)) * rate;
            self.account.add_free_funds("USDT", -payment);
            self.emit(DomainEvent::FundingRateApplied {
                symbol: sym.clone(),
                rate,
                payment: -payment,
            });
            println!("[PAPER] Funding applied: {} payment {} USDT", sym, payment);
        }
    }

    fn persist_trade(&mut self, symbol: &str, side: &str, price: Decimal, quantity: Decimal, fee: Decimal) {
        let timestamp = now_ms();
        let _ = self.db_tx.send(PersistEvent::Trade {
            order_id: format!("PAPER_{}", timestamp),
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
            fee,
            timestamp,
        });
        self.recent_trades.push(TradeView {
            order_id: format!("PAPER_{}", timestamp),
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
            fee,
            timestamp,
        });
        if self.recent_trades.len() > 200 {
            let excess = self.recent_trades.len() - 200;
            self.recent_trades.drain(..excess);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // LEGACY: L2_SWEEP / LINEAR_IMPACT (order book tabanlı)
    // ─────────────────────────────────────────────────────────────
    async fn process_sweep(&mut self, order: OrderRequest) -> Result<OrderAck, OrderRejectReason> {
        match order.order_type {
            crate::order::OrderType::Market => {
                if order.side == crate::order::OrderSide::Buy {
                    let estimated_price = if self.orderbook.last_price > Decimal::ZERO { self.orderbook.last_price } else { Decimal::from(999_999) };
                    let estimated_cost = order.quantity * estimated_price;

                    if self.account.get_free("USDT") < estimated_cost {
                        return Err(OrderRejectReason::InsufficientFunds);
                    }

                    match self.orderbook.sweep_buy(order.quantity) {
                        Ok(trades) => {
                            let mut total_cost = Decimal::ZERO;
                            let mut total_qty = Decimal::ZERO;
                            for t in trades {
                                total_cost += t.price * t.quantity;
                                total_qty += t.quantity;
                            }
                            let avg_price = total_cost / total_qty;
                            let fee = total_cost * self.config.taker_fee;

                            if self.account.get_free("USDT") < (total_cost + fee) {
                                return Err(OrderRejectReason::InsufficientFunds);
                            }

                            let _ = self.account.deduct_free_funds("USDT", total_cost + fee);
                            self.account.add_free_funds("BTC", total_qty);
                            self.persist_trade(&order.symbol, "BUY", avg_price, total_qty, fee);
                            Ok(OrderAck { order_id: format!("PAPER_{}", now_ms()), avg_price, executed_qty: total_qty })
                        }
                        Err(e) if e == "MARKET_UNAVAILABLE" => Err(OrderRejectReason::MarketUnavailable),
                        Err(_) => Err(OrderRejectReason::InsufficientDepth),
                    }
                } else {
                    if self.account.get_free("BTC") < order.quantity {
                        return Err(OrderRejectReason::InsufficientFunds);
                    }
                    match self.orderbook.sweep_sell(order.quantity) {
                        Ok(trades) => {
                            let mut total_revenue = Decimal::ZERO;
                            let mut total_qty = Decimal::ZERO;
                            for t in trades {
                                total_revenue += t.price * t.quantity;
                                total_qty += t.quantity;
                            }
                            let avg_price = total_revenue / total_qty;
                            let fee = total_revenue * self.config.taker_fee;

                            let _ = self.account.deduct_free_funds("BTC", total_qty);
                            self.account.add_free_funds("USDT", total_revenue - fee);
                            self.persist_trade(&order.symbol, "SELL", avg_price, total_qty, fee);
                            Ok(OrderAck { order_id: format!("PAPER_{}", now_ms()), avg_price, executed_qty: total_qty })
                        }
                        Err(e) if e == "MARKET_UNAVAILABLE" => Err(OrderRejectReason::MarketUnavailable),
                        Err(_) => Err(OrderRejectReason::InsufficientDepth),
                    }
                }
            }
            _ => {
                println!("[PAPER] Legacy mode does not support this order type yet.");
                Err(OrderRejectReason::MarketUnavailable)
            }
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
