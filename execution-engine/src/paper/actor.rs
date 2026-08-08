#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::should_implement_trait)]

use crate::order::{OrderPositionSide, OrderRequest, OrderSide, OrderType};
use crate::paper::account::AccountState;
use crate::paper::domain_event::DomainEvent;
use crate::paper::config::PaperConfig;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionMode {
    OneWay,
    Hedge,
}

impl PositionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PositionMode::OneWay => "ONE_WAY",
            PositionMode::Hedge => "HEDGE",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ONE_WAY" | "ONE-WAY" | "BOTH" => Some(PositionMode::OneWay),
            "HEDGE" | "DUAL" => Some(PositionMode::Hedge),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarginType {
    Crossed,
    Isolated,
}

impl MarginType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarginType::Crossed => "CROSSED",
            MarginType::Isolated => "ISOLATED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "CROSSED" | "CROSS" => Some(MarginType::Crossed),
            "ISOLATED" | "ISOLATE" => Some(MarginType::Isolated),
            _ => None,
        }
    }
}

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
    MarkPriceUpdate {
        symbol: String,
        mark_price: Decimal,
        funding_rate: Decimal,
        timestamp: u64,
    },
    SetPositionMode {
        mode: PositionMode,
        response_tx: oneshot::Sender<Result<(), String>>,
    },
    SetMarginType {
        symbol: String,
        margin_type: MarginType,
        response_tx: oneshot::Sender<Result<(), String>>,
    },
}

/// Mark price kaynağıyla bekleyen limit emri
#[derive(Debug, Clone)]
pub struct OpenOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub position_side: OrderPositionSide,
    pub quantity: Decimal,
    pub remaining: Decimal,
    pub limit_price: Decimal,
    pub leverage: Decimal,
}

pub struct PaperEngineActor {
    config: PaperConfig,
    account: AccountState,
    positions: PositionManager,
    risk: RiskManager,
    open_orders: Vec<OpenOrder>,
    event_tx: Option<mpsc::UnboundedSender<DomainEvent>>,
    last_funding_ts: u64,
    position_mode: PositionMode,
    default_margin_type: MarginType,
    margin_types: HashMap<String, MarginType>,
    isolated_wallets: HashMap<String, Decimal>,
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
        let risk = RiskManager::new(
            config.initial_usdt,
            config.max_leverage,
            config.max_drawdown_pct,
            config.max_daily_loss,
            config.min_position_notional,
        );

        let position_mode = PositionMode::from_str(&config.position_mode).unwrap_or(PositionMode::OneWay);
        let default_margin_type = MarginType::from_str(&config.margin_type).unwrap_or(MarginType::Crossed);

        let mut actor = Self {
            config,
            account,
            positions: PositionManager::new(),
            risk,
            open_orders: Vec::new(),
            event_tx,
            last_funding_ts: 0,
            position_mode,
            default_margin_type,
            margin_types: HashMap::new(),
            isolated_wallets: HashMap::new(),
            mark_prices: HashMap::new(),
            funding_rates: HashMap::new(),
            recent_trades: Vec::new(),
            snapshot: Arc::new(RwLock::new(PaperSnapshot::build(
                Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
                crate::paper::risk::RiskStatus::Ok, Decimal::ZERO,
                &PositionManager::new(), 0, vec![], &HashMap::new(),
                "ONE_WAY".to_string(), &HashMap::new(),
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
            self.last_price(),
            &self.positions,
            self.open_orders.len(),
            self.recent_trades.clone(),
            &self.mark_prices,
            self.position_mode.as_str().to_string(),
            &self.margin_types,
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
                DomainEvent::OrderFilled { symbol, side, position_side, fill_price, fill_qty, commission, cash_delta, realized_pnl, leverage, .. } => {
                    let signed = if side == "BUY" { *fill_qty } else { -*fill_qty };
                    if self.position_mode == PositionMode::Hedge {
                        let ps = match position_side.as_str() {
                            "LONG" => PositionSide::Long,
                            "SHORT" => PositionSide::Short,
                            _ => PositionSide::Long,
                        };
                        let _ = self.positions.apply_fill_hedge(symbol, ps, signed, *fill_price, *leverage);
                    } else {
                        let _ = self.positions.apply_fill(symbol, signed, *fill_price, *leverage);
                    }
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
        println!("PaperEngineActor: Started | mode={} | margin={} | price=mark",
            self.position_mode.as_str(), self.default_margin_type.as_str());

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                ActorCommand::SubmitOrder { order, response_tx } => {
                    let result = self.process_order(order).await;
                    let _ = response_tx.send(result);
                }
                ActorCommand::MarkPriceUpdate { symbol, mark_price, funding_rate, timestamp } => {
                    self.mark_prices.insert(symbol.clone(), mark_price);
                    self.funding_rates.insert(symbol.clone(), funding_rate);
                    self.on_mark_tick(timestamp);
                    self.check_limit_orders(symbol, mark_price);
                }
                ActorCommand::SetPositionMode { mode, response_tx } => {
                    let res = self.set_position_mode(mode);
                    let _ = response_tx.send(res);
                }
                ActorCommand::SetMarginType { symbol, margin_type, response_tx } => {
                    let res = self.set_margin_type(&symbol, margin_type);
                    let _ = response_tx.send(res);
                }
            }
            self.publish_snapshot();
        }
    }

    pub fn last_price(&self) -> Decimal {
        self.mark_prices.get("BTCUSDT").copied()
            .or_else(|| self.mark_prices.values().next().copied())
            .unwrap_or(Decimal::ZERO)
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

    pub fn position_mode(&self) -> PositionMode {
        self.position_mode
    }

    pub fn equity(&self) -> Decimal {
        self.risk.equity(&self.positions, &self.mark_prices, self.account.get_free("USDT"))
    }

    // ── Mod değişiklikleri ───────────────────────────────────────

    fn set_position_mode(&mut self, mode: PositionMode) -> Result<(), String> {
        if mode == self.position_mode {
            return Ok(());
        }
        if !self.positions.all().is_empty() {
            return Err("Cannot change position mode with open positions".into());
        }
        if !self.open_orders.is_empty() {
            return Err("Cannot change position mode with open orders".into());
        }
        self.position_mode = mode;
        println!("[PAPER] Position mode -> {}", mode.as_str());
        Ok(())
    }

    fn set_margin_type(&mut self, symbol: &str, margin_type: MarginType) -> Result<(), String> {
        if self.positions.total_abs_qty(symbol) > Decimal::ZERO {
            return Err("Cannot change margin type with open position".into());
        }
        self.margin_types.insert(symbol.to_string(), margin_type);
        println!("[PAPER] {} margin -> {}", symbol, margin_type.as_str());
        Ok(())
    }

    fn margin_type_of(&self, symbol: &str) -> MarginType {
        self.margin_types.get(symbol).copied().unwrap_or(self.default_margin_type)
    }

    // ── Marj kilitleme (cross vs isolated) ───────────────────────

    fn lock_margin(&mut self, symbol: &str, amount: Decimal) {
        if amount <= Decimal::ZERO {
            return;
        }
        match self.margin_type_of(symbol) {
            MarginType::Crossed => {
                let _ = self.account.lock_funds("USDT", amount);
            }
            MarginType::Isolated => {
                let _ = self.account.deduct_free_funds("USDT", amount);
                *self.isolated_wallets.entry(symbol.to_string()).or_default() += amount;
            }
        }
    }

    fn release_margin(&mut self, symbol: &str, amount: Decimal) {
        if amount <= Decimal::ZERO {
            return;
        }
        match self.margin_type_of(symbol) {
            MarginType::Crossed => self.account.unlock_funds("USDT", amount),
            MarginType::Isolated => {
                let w = self.isolated_wallets.entry(symbol.to_string()).or_default();
                let rel = amount.min(*w);
                *w -= rel;
                self.account.add_free_funds("USDT", rel);
            }
        }
    }

    fn apply_fill_dispatch(
        &mut self,
        symbol: &str,
        target_side: Option<PositionSide>,
        signed_qty: Decimal,
        price: Decimal,
        leverage: Decimal,
    ) -> (Decimal, Decimal) {
        match self.position_mode {
            PositionMode::OneWay => self.positions.apply_fill(symbol, signed_qty, price, leverage),
            PositionMode::Hedge => {
                self.positions.apply_fill_hedge(symbol, target_side.unwrap_or(PositionSide::Long), signed_qty, price, leverage)
            }
        }
    }

    async fn process_order(&mut self, order: OrderRequest) -> Result<OrderAck, OrderRejectReason> {
        // Latency & Jitter simülasyonu
        let delay = self.config.base_latency_ms
            + (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_millis() as u64 % (self.config.latency_jitter_ms + 1));
        sleep(Duration::from_millis(delay)).await;

        self.process_price_only(order)
    }

    // ─────────────────────────────────────────────────────────────
    // PRICE_ONLY: mark price ile (order book'suz) dolum
    // ─────────────────────────────────────────────────────────────
    fn process_price_only(&mut self, order: OrderRequest) -> Result<OrderAck, OrderRejectReason> {
        // Fiyat kaynağı: mark price. Yoksa emir reddedilir.
        let mark = *self.mark_prices.get(&order.symbol).unwrap_or(&Decimal::ZERO);
        if mark <= Decimal::ZERO {
            return Err(OrderRejectReason::MarketUnavailable);
        }

        // Hedge modda LONG/SHORT zorunlu; one-way'de BOTH beklenir.
        let target_side = match (self.position_mode, order.position_side) {
            (PositionMode::Hedge, OrderPositionSide::Long) => Some(PositionSide::Long),
            (PositionMode::Hedge, OrderPositionSide::Short) => Some(PositionSide::Short),
            (PositionMode::Hedge, OrderPositionSide::Both) => {
                return Err(OrderRejectReason::RiskRejected("position_side required in HEDGE mode".into()));
            }
            (PositionMode::OneWay, _) => None,
        };

        let leverage = self.config.max_leverage.min(Decimal::ONE.max(self.config.max_leverage));
        let order_id = format!("PAPER_{}", now_ms());
        let signed = if order.side == OrderSide::Buy { order.quantity } else { -order.quantity };

        match order.order_type {
            OrderType::Market => {
                if let Err(msg) = self.risk.check_order(
                    order.quantity,
                    leverage,
                    self.account.get_free("USDT"),
                ) {
                    return Err(OrderRejectReason::RiskRejected(msg.to_string()));
                }

                // Marj değişimi (pozisyon tarafı bazında, USDT notional)
                let before = match self.position_mode {
                    PositionMode::OneWay => self.positions.get(&order.symbol).map(|p| p.quantity).unwrap_or(Decimal::ZERO),
                    PositionMode::Hedge => self.positions
                        .get_hedge(&order.symbol, target_side.unwrap())
                        .map(|p| p.quantity)
                        .unwrap_or(Decimal::ZERO),
                };
                let after = before + signed;
                let margin_delta = after.abs() - before.abs();
                let margin_locked = if margin_delta > Decimal::ZERO { margin_delta / leverage } else { Decimal::ZERO };
                let margin_released = if margin_delta < Decimal::ZERO { -margin_delta / leverage } else { Decimal::ZERO };

                self.emit(DomainEvent::OrderCreated {
                    order_id: order_id.clone(),
                    client_oid: order_id.clone(),
                    symbol: order.symbol.clone(),
                    side: format!("{:?}", order.side).to_uppercase(),
                    order_type: "MARKET".to_string(),
                    qty: order.quantity,
                    price: Some(mark),
                });

                let fee = order.quantity * self.config.taker_fee;
                if self.account.get_free("USDT") < (margin_locked + fee) {
                    return Err(OrderRejectReason::InsufficientFunds);
                }

                self.lock_margin(&order.symbol, margin_locked);
                self.release_margin(&order.symbol, margin_released);
                let _ = self.account.deduct_free_funds("USDT", fee);

                let (realized, _) = self.apply_fill_dispatch(&order.symbol, target_side, signed, mark, leverage);
                self.risk.record_realized(realized);
                self.account.add_free_funds("USDT", realized);

                let side_str = match order.side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };
                let pos_side_str = self.position_side_str(target_side);
                self.emit_fill(&order_id, &order.symbol, side_str, &pos_side_str, mark, order.quantity, fee, realized, leverage, margin_released, margin_locked);
                self.persist_trade(&order.symbol, side_str, mark, order.quantity, fee);
                Ok(OrderAck { order_id, avg_price: mark, executed_qty: order.quantity })
            }
            OrderType::Limit => {
                let limit_price = order.price.unwrap_or(mark);
                if let Err(msg) = self.risk.check_order(
                    order.quantity,
                    leverage,
                    self.account.get_free("USDT"),
                ) {
                    return Err(OrderRejectReason::RiskRejected(msg.to_string()));
                }

                // Marj için fonları kilitle (USDT notional / leverage)
                let margin = order.quantity / leverage;
                if self.account.get_free("USDT") < margin {
                    return Err(OrderRejectReason::InsufficientFunds);
                }
                self.lock_margin(&order.symbol, margin);

                self.emit(DomainEvent::OrderCreated {
                    order_id: order_id.clone(),
                    client_oid: order_id.clone(),
                    symbol: order.symbol.clone(),
                    side: format!("{:?}", order.side).to_uppercase(),
                    order_type: "LIMIT".to_string(),
                    qty: order.quantity,
                    price: Some(limit_price),
                });

                // Fiyat zaten seviyeyi geçtiyse anında doldur (mark price ile)
                let crossed = match order.side {
                    OrderSide::Buy => mark <= limit_price,
                    OrderSide::Sell => mark >= limit_price,
                };

                if crossed {
                    self.fill_limit(&order_id, &order.symbol, order.side, target_side, order.quantity, limit_price, leverage, margin);
                    return Ok(OrderAck { order_id, avg_price: limit_price, executed_qty: order.quantity });
                }

                self.open_orders.push(OpenOrder {
                    order_id,
                    symbol: order.symbol.clone(),
                    side: order.side,
                    position_side: order.position_side,
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

    fn position_side_str(&self, target_side: Option<PositionSide>) -> String {
        match self.position_mode {
            PositionMode::OneWay => "BOTH".to_string(),
            PositionMode::Hedge => match target_side {
                Some(PositionSide::Long) => "LONG".to_string(),
                Some(PositionSide::Short) => "SHORT".to_string(),
                None => "BOTH".to_string(),
            },
        }
    }

    fn emit_fill(
        &self,
        order_id: &str,
        symbol: &str,
        side: &str,
        position_side: &str,
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
            position_side: position_side.to_string(),
            fill_price: price,
            fill_qty: qty,
            commission: fee,
            cash_delta,
            realized_pnl: realized,
            leverage,
        });
    }

    fn check_limit_orders(&mut self, symbol: String, price: Decimal) {
        let mut filled: Vec<usize> = Vec::new();
        let mut fill_data: Vec<(String, String, OrderSide, Option<PositionSide>, Decimal, Decimal, Decimal, Decimal)> = Vec::new();
        for (i, o) in self.open_orders.iter().enumerate() {
            if o.symbol != symbol {
                continue;
            }
            let crossed = match o.side {
                OrderSide::Buy => price <= o.limit_price,
                OrderSide::Sell => price >= o.limit_price,
            };
            if crossed {
                let target = match o.position_side {
                    OrderPositionSide::Long => Some(PositionSide::Long),
                    OrderPositionSide::Short => Some(PositionSide::Short),
                    OrderPositionSide::Both => None,
                };
                fill_data.push((o.order_id.clone(), o.symbol.clone(), o.side, target, o.remaining, o.limit_price, o.leverage, o.quantity / o.leverage));
                filled.push(i);
            }
        }
        for (order_id, symbol, side, target, qty, limit_price, leverage, margin) in fill_data {
            self.fill_limit(&order_id, &symbol, side, target, qty, limit_price, leverage, margin);
        }
        for i in filled.into_iter().rev() {
            self.open_orders.remove(i);
        }
    }

    fn fill_limit(
        &mut self,
        order_id: &str,
        symbol: &str,
        side: OrderSide,
        target_side: Option<PositionSide>,
        qty: Decimal,
        price: Decimal,
        leverage: Decimal,
        margin_locked: Decimal,
    ) {
        let fee = qty * self.config.maker_fee;
        let signed = if side == OrderSide::Buy { qty } else { -qty };

        let before = match self.position_mode {
            PositionMode::OneWay => self.positions.get(symbol).map(|p| p.quantity).unwrap_or(Decimal::ZERO),
            PositionMode::Hedge => self.positions
                .get_hedge(symbol, target_side.unwrap_or(PositionSide::Long))
                .map(|p| p.quantity)
                .unwrap_or(Decimal::ZERO),
        };
        let after = before + signed;
        let margin_delta = after.abs() - before.abs();
        let margin_net_locked = if margin_delta > Decimal::ZERO { margin_delta / leverage } else { Decimal::ZERO };
        let margin_released = if margin_delta < Decimal::ZERO { -margin_delta / leverage } else { Decimal::ZERO };

        // Bekleyen emrin kilitlediği marjı serbest bırak, net artışı tekrar kilitle
        self.release_margin(symbol, margin_locked);
        self.lock_margin(symbol, margin_net_locked);
        let _ = self.account.deduct_free_funds("USDT", fee);

        let (realized, _) = self.apply_fill_dispatch(symbol, target_side, signed, price, leverage);
        self.risk.record_realized(realized);
        self.account.add_free_funds("USDT", realized);

        match side {
            OrderSide::Buy => {
                self.account.add_free_funds("BTC", qty / price);
            }
            OrderSide::Sell => {
                self.account.subtract_free_funds_unchecked("BTC", qty / price);
            }
        }
        let side_str = match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };
        let pos_side_str = self.position_side_str(target_side);
        self.emit_fill(order_id, symbol, side_str, &pos_side_str, price, qty, fee, realized, leverage, margin_released, margin_net_locked);
        self.persist_trade(symbol, side_str, price, qty, fee);
        println!("[PAPER] LIMIT {} {} Filled: {} @ {}. Fee: {} USDT", pos_side_str, side_str, qty, price, fee);
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
            let targets: Vec<(String, PositionSide, Decimal, Decimal)> = self.positions.all()
                .iter()
                .filter(|p| p.symbol == sym)
                .map(|p| (p.symbol.clone(), p.side, p.quantity, p.leverage))
                .collect();
            for (symbol, side, pos_qty, leverage) in targets {
                let mark = *self.mark_prices.get(&symbol).unwrap_or(&self.positions.all().iter().find(|p| p.symbol == symbol).map(|p| p.avg_entry_price).unwrap_or(Decimal::ZERO));
                let closing_side = match side { PositionSide::Long => "SELL", PositionSide::Short => "BUY" };
                let side_label = match side { PositionSide::Long => "LONG", PositionSide::Short => "SHORT" };
                let signed = match side { PositionSide::Long => -pos_qty.abs(), PositionSide::Short => pos_qty.abs() };
                let (realized, _) = self.apply_fill_dispatch(&symbol, Some(side), signed, mark, leverage);

                self.risk.record_realized(realized);
                // Marjı serbest bırak (USDT notional / leverage); izole wallet'tan düşülür
                let margin = pos_qty.abs() / leverage;
                self.release_margin(&symbol, margin);
                self.account.add_free_funds("USDT", realized);
                let order_id = format!("PAPER_LIQ_{}", now_ms());
                self.emit_fill(&order_id, &symbol, closing_side, side_label, mark, pos_qty.abs(), Decimal::ZERO, realized, leverage, margin, Decimal::ZERO);
                self.emit(DomainEvent::Liquidation {
                    symbol: symbol.clone(),
                    side: side_label.to_string(),
                    price: mark,
                    qty: pos_qty.abs(),
                });
                self.persist_trade(&symbol, side_label, mark, pos_qty.abs(), Decimal::ZERO);
                println!("[PAPER] ⚠️ LIQUIDATION: {} {} @ {}", symbol, side_label, mark);
            }
        }
    }

    fn apply_funding(&mut self) {
        let funding_data: Vec<(String, Decimal)> = self.positions.all()
            .iter()
            .map(|p| {
                let notional = p.notional(*self.mark_prices.get(&p.symbol).unwrap_or(&p.avg_entry_price));
                (p.symbol.clone(), notional)
            })
            .collect();
        for (sym, notional) in funding_data {
            let rate = *self.funding_rates.get(&sym).unwrap_or(&Decimal::ZERO);
            // Binance funding_rate, 8 saatlik periyot başına verilir (per-interval)
            let payment = notional * rate;
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
        // Tek olay kanalı: kalıcılık (SQLite/PG) katman artık yalnızca
        // DomainEvent akışından beslenir (paper-service projection).
        let timestamp = now_ms();
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
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
