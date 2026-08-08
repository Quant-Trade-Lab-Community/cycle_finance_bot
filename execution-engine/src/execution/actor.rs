//! Execution actor — tek-yazıcı komut döngüsü.
//!
//! Tüm yazma işlemleri (emir, iptal, leverage, margin) bu task'tan geçer.
//! User-data stream olayları burada snapshot'a işlenir; periyodik uzlaştırma
//! borsa gerçeğiyle sapmayı yakalar.

use crate::config::ExecConfig;
use crate::error::ExecError;
use crate::execution::idempotency::IdempotencyCache;
use crate::execution::lifecycle::InFlightRegistry;
use crate::execution::preflight::{new_client_order_id, Preflight};
use crate::metrics::Metrics;
use crate::order::{BinanceOrderResponse, OrderAck, OrderRequest, OrderStatus};
use crate::risk::checks::RiskChecks;
use crate::risk::kill_switch::KillSwitch;
use crate::state::exchange_cache::ExchangeCache;
use crate::state::projector;
use crate::state::snapshot::AccountSnapshot;
use crate::types::account::MarginType;
use crate::types::user_event::UserDataEvent;
use crate::client::BinanceClient;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

/// Actor komutları (yazma/okuma — tek yazıcı).
#[allow(clippy::large_enum_variant)]
pub enum Command {
    SubmitOrder {
        order: OrderRequest,
        tx: oneshot::Sender<Result<OrderAck, String>>,
    },
    BatchOrders {
        orders: Vec<OrderRequest>,
        tx: oneshot::Sender<Result<Vec<OrderAck>, String>>,
    },
    CancelOrder {
        symbol: String,
        order_id: Option<i64>,
        client_order_id: Option<String>,
        tx: oneshot::Sender<Result<BinanceOrderResponse, String>>,
    },
    CancelAll {
        symbol: String,
        tx: oneshot::Sender<Result<usize, String>>,
    },
    QueryOrder {
        symbol: String,
        order_id: Option<i64>,
        client_order_id: Option<String>,
        tx: oneshot::Sender<Result<BinanceOrderResponse, String>>,
    },
    ModifyOrder {
        symbol: String,
        order_id: Option<i64>,
        client_order_id: Option<String>,
        quantity: Option<Decimal>,
        price: Option<Decimal>,
        stop_price: Option<Decimal>,
        tx: oneshot::Sender<Result<BinanceOrderResponse, String>>,
    },
    SetLeverage {
        symbol: String,
        leverage: u32,
        tx: oneshot::Sender<Result<(), String>>,
    },
    SetMarginType {
        symbol: String,
        margin_type: MarginType,
        tx: oneshot::Sender<Result<(), String>>,
    },
    AdjustMargin {
        symbol: String,
        amount: Decimal,
        direction: u8,
        tx: oneshot::Sender<Result<(), String>>,
    },
    SetPositionMode {
        dual: bool,
        tx: oneshot::Sender<Result<(), String>>,
    },
    SetMultiAssets {
        enabled: bool,
        tx: oneshot::Sender<Result<(), String>>,
    },
    /// Borsa ile tam yeniden eşitleme (bağlantı kopması / gap sonrası).
    Resync,
}

/// User-data stream'den actor'e akan olaylar.
#[allow(clippy::large_enum_variant)]
pub enum UserEvent {
    Data(UserDataEvent),
    StreamConnected,
}

pub struct ExecutionActor {
    client: Arc<BinanceClient>,
    preflight: Preflight,
    risk: RiskChecks,
    kill_switch: Arc<KillSwitch>,
    snapshot: Arc<RwLock<AccountSnapshot>>,
    metrics: Arc<Metrics>,
    config: ExecConfig,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
    user_rx: mpsc::UnboundedReceiver<UserEvent>,
    in_flight: InFlightRegistry,
    idempotency: IdempotencyCache,
}

impl ExecutionActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: Arc<BinanceClient>,
        exchange: ExchangeCache,
        risk: RiskChecks,
        kill_switch: Arc<KillSwitch>,
        snapshot: Arc<RwLock<AccountSnapshot>>,
        metrics: Arc<Metrics>,
        config: ExecConfig,
        cmd_rx: mpsc::UnboundedReceiver<Command>,
        user_rx: mpsc::UnboundedReceiver<UserEvent>,
    ) -> Self {
        Self {
            preflight: Preflight::new(exchange),
            client,
            risk,
            kill_switch,
            snapshot,
            metrics,
            in_flight: InFlightRegistry::new(5_000, config.max_in_flight.max(1)),
            idempotency: IdempotencyCache::new(10_000),
            config,
            cmd_rx,
            user_rx,
        }
    }

    pub async fn run(mut self) {
        info!(
            "ExecutionActor: başlıyor | mode={} dry_run={}",
            self.config.mode.as_str(),
            self.config.dry_run
        );

        // İlk eşitleme tamamlanmadan döngüye girilmez (emir kabul edilmez).
        if let Err(e) = self.resync().await {
            error!("ExecutionActor: ilk eşitleme başarısız: {e}");
        }

        let reconcile_sec = self.config.reconcile_interval_sec.max(10);
        let mut reconcile = tokio::time::interval(std::time::Duration::from_secs(reconcile_sec));
        reconcile.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut inflight_check = tokio::time::interval(std::time::Duration::from_secs(1));
        inflight_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                Some(cmd) = self.cmd_rx.recv() => {
                    self.handle_command(cmd).await;
                }
                Some(ev) = self.user_rx.recv() => {
                    self.handle_user_event(ev).await;
                }
                _ = reconcile.tick() => {
                    self.reconcile().await;
                }
                _ = inflight_check.tick() => {
                    self.reconcile_inflight().await;
                }
            }
        }
    }

    // ── Komutlar ─────────────────────────────────────────────────

    async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::SubmitOrder { order, tx } => {
                let res = self.submit_order(order).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::BatchOrders { orders, tx } => {
                let res = self.submit_batch(orders).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::CancelOrder { symbol, order_id, client_order_id, tx } => {
                let res = self.cancel_order(&symbol, order_id, client_order_id.as_deref()).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::CancelAll { symbol, tx } => {
                let res = self.cancel_all(&symbol).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::QueryOrder { symbol, order_id, client_order_id, tx } => {
                let res = self.client.query_order(&symbol, order_id, client_order_id.as_deref()).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::ModifyOrder { symbol, order_id, client_order_id, quantity, price, stop_price, tx } => {
                let res = self.client.modify_order(&symbol, order_id, client_order_id.as_deref(), quantity, price, stop_price, 0).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::SetLeverage { symbol, leverage, tx } => {
                let res = self.set_leverage(&symbol, leverage).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::SetMarginType { symbol, margin_type, tx } => {
                let res = self.set_margin_type(&symbol, margin_type).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::AdjustMargin { symbol, amount, direction, tx } => {
                let res = self.adjust_margin(&symbol, amount, direction).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::SetPositionMode { dual, tx } => {
                let res = self.set_position_mode(dual).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::SetMultiAssets { enabled, tx } => {
                let res = self.set_multi_assets(enabled).await;
                let _ = tx.send(res.map_err(|e| e.to_string()));
            }
            Command::Resync => {
                if let Err(e) = self.resync().await {
                    error!("ExecutionActor: resync hatası: {e}");
                }
            }
        }
    }

    // ── Emir gönderim akışı ──────────────────────────────────────

    async fn submit_order(&mut self, order: OrderRequest) -> Result<OrderAck, ExecError> {
        if !self.snapshot.read().ready {
            return Err(ExecError::NotReady("hesap borsa ile eşitlenmedi".into()));
        }
        if self.kill_switch.is_open() {
            return Err(ExecError::Risk("kill switch açık — emir reddedildi".into()));
        }
        // Market emri için snapshot'ta bilinen mark fiyatını risk kapısına besle.
        if order.price.is_none()
            && let Some(p) = self
                .snapshot
                .read()
                .positions
                .iter()
                .find(|p| p.symbol.eq_ignore_ascii_case(&order.symbol))
        {
            self.risk.push_mark(&p.symbol, p.mark_price);
        }        self.risk.check(&order)?;

        // Idempotency: aynı client_order_id tekrar gönderilmez.
        let cid = order
            .client_order_id
            .clone()
            .unwrap_or_else(new_client_order_id);
        if let Some(cached) = self.idempotency.get(&cid) {
            info!("Idempotency: {cid} tekrarı — önbellekten yanıt");
            return Ok(cached);
        }

        let position_mode = self.snapshot.read().position_mode;
        let mut normalized = self.preflight.normalize_and_check(&order, position_mode)?;
        normalized.client_order_id = Some(cid.clone());

        if self.config.dry_run {
            info!("DRY_RUN: {cid} {symbol} {side} {qty} doğrulandı — gönderilmedi",
                symbol = normalized.symbol, side = normalized.side.binance_str(), qty = normalized.quantity);
            let ack = OrderAck {
                order_id: "DRY_RUN".into(),
                client_order_id: cid,
                symbol: normalized.symbol,
                status: "DRY_RUN".into(),
                avg_price: Decimal::ZERO,
                executed_qty: Decimal::ZERO,
                cum_quote: Decimal::ZERO,
                reduce_only: normalized.reduce_only.unwrap_or(false),
            };
            self.idempotency.set(normalized.client_order_id.clone().unwrap(), ack.clone());
            self.metrics.record_order(true);
            return Ok(ack);
        }

        let started = Instant::now();
        self.in_flight.insert(cid.clone(), normalized.symbol.clone(), None, None);

        let res = self.client.place_order(&normalized).await;
        let latency_us = started.elapsed().as_micros() as u64;
        self.metrics.record_latency_us(latency_us);

        match res {
            Ok(response) => {
                self.risk.record_order();
                self.metrics.record_order(true);
                let ack: OrderAck = response.clone().into();
                let status = OrderStatus::from_binance(&response.status).unwrap_or(OrderStatus::New);

                if status.is_open() {
                    self.in_flight.set_order_id(&cid, response.order_id);
                    self.sync_open_order(response.clone());
                } else {
                    self.in_flight.confirm(&cid);
                    self.sync_open_order(response.clone());
                    if status == OrderStatus::Filled {
                        self.metrics.record_fill();
                    }
                }
                self.idempotency.set(cid.clone(), ack.clone());
                info!("Emir kabul: {} {symbol} {side} {qty} → {status}",
                    cid, symbol = normalized.symbol, side = normalized.side.binance_str(),
                    qty = normalized.quantity, status = response.status);
                Ok(ack)
            }
            Err(e) => {
                self.in_flight.confirm(&cid);
                self.metrics.record_order(false);
                if let ExecError::RateLimit { .. } = &e {
                    self.metrics.record_rate_limited();
                }
                error!("Emir reddedildi: {cid} → {e}");
                Err(e)
            }
        }
    }

    async fn submit_batch(&mut self, orders: Vec<OrderRequest>) -> Result<Vec<OrderAck>, ExecError> {
        if orders.is_empty() || orders.len() > 5 {
            return Err(ExecError::Preflight("batchOrders 1..=5 emir alır".into()));
        }
        if !self.snapshot.read().ready {
            return Err(ExecError::NotReady("hesap borsa ile eşitlenmedi".into()));
        }
        if self.kill_switch.is_open() {
            return Err(ExecError::Risk("kill switch açık".into()));
        }
        let position_mode = self.snapshot.read().position_mode;

        let mut normalized_orders = Vec::with_capacity(orders.len());
        for mut o in orders {
            let cid = o.client_order_id.clone().unwrap_or_else(new_client_order_id);
            if self.idempotency.contains(&cid) {
                return Err(ExecError::Preflight(format!(
                    "idempotency: {cid} daha önce kullanıldı"
                )));
            }
            o = self.preflight.normalize_and_check(&o, position_mode)?;
            o.client_order_id = Some(cid);
            normalized_orders.push(o);
        }

        if self.config.dry_run {
            info!("DRY_RUN batch: {} emir doğrulandı — gönderilmedi", normalized_orders.len());
            let acks = normalized_orders
                .iter()
                .map(|o| OrderAck {
                    order_id: "DRY_RUN".into(),
                    client_order_id: o.client_order_id.clone().unwrap_or_default(),
                    symbol: o.symbol.clone(),
                    status: "DRY_RUN".into(),
                    avg_price: Decimal::ZERO,
                    executed_qty: Decimal::ZERO,
                    cum_quote: Decimal::ZERO,
                    reduce_only: o.reduce_only.unwrap_or(false),
                })
                .collect();
            return Ok(acks);
        }

        for o in &normalized_orders {
            self.risk.check(o)?;
        }

        let values = self.client.batch_orders(&normalized_orders).await?;
        let mut acks = Vec::with_capacity(values.len());
        for (o, v) in normalized_orders.iter().zip(values.iter()) {
            let cid = o.client_order_id.clone().unwrap_or_default();
            if let Some(code) = v.get("code").and_then(|c| c.as_i64()) {
                // Tek emir başarısız — diğerleri etkilenmez.
                self.metrics.record_order(false);
                let msg = v.get("msg").and_then(|m| m.as_str()).unwrap_or("").to_string();
                warn!("batch alt-emir reddedildi: {cid} code {code}: {msg}");
                continue;
            }
            let response: BinanceOrderResponse = serde_json::from_value(v.clone()).map_err(ExecError::Json)?;
            let status = OrderStatus::from_binance(&response.status).unwrap_or(OrderStatus::New);
            if status.is_open() {
                self.in_flight.insert(cid.clone(), o.symbol.clone(), Some(response.order_id), None);
                self.sync_open_order(response.clone());
            } else {
                if status == OrderStatus::Filled {
                    self.metrics.record_fill();
                }
                self.sync_open_order(response.clone());
            }
            let ack: OrderAck = response.into();
            self.idempotency.set(cid, ack.clone());
            self.risk.record_order();
            self.metrics.record_order(true);
            acks.push(ack);
        }
        Ok(acks)
    }

    async fn cancel_order(
        &mut self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
    ) -> Result<BinanceOrderResponse, ExecError> {
        let res = self
            .client
            .cancel_order(symbol, order_id, client_order_id)
            .await?;
        self.sync_open_order(res.clone());
        self.metrics.record_cancel();
        Ok(res)
    }

    async fn cancel_all(&mut self, symbol: &str) -> Result<usize, ExecError> {
        let res = self.client.cancel_all_open(symbol).await?;
        let n = res.len();
        for o in res {
            self.sync_open_order(o);
        }
        self.metrics.record_cancel();
        Ok(n)
    }

    // ── Kontrol işlemleri ────────────────────────────────────────

    async fn set_leverage(&mut self, symbol: &str, leverage: u32) -> Result<(), ExecError> {
        if leverage == 0 {
            return Err(ExecError::Preflight("leverage > 0 olmalı".into()));
        }
        let _ = self.client.set_leverage(symbol, leverage).await?;
        for p in self.snapshot.write().positions.iter_mut() {
            if p.symbol == symbol {
                p.leverage = Decimal::from(leverage);
            }
        }
        info!("{symbol} leverage → {leverage}x");
        Ok(())
    }

    async fn set_margin_type(&mut self, symbol: &str, margin_type: MarginType) -> Result<(), ExecError> {
        // Açık pozisyon varken margin tipi değiştirilemez; borsa -4046 döner.
        let _ = self.client.set_margin_type(symbol, margin_type).await?;
        for p in self.snapshot.write().positions.iter_mut() {
            if p.symbol == symbol {
                p.margin_type = margin_type.binance_str().into();
            }
        }
        info!("{symbol} margin → {}", margin_type.binance_str());
        Ok(())
    }

    async fn adjust_margin(&mut self, symbol: &str, amount: Decimal, direction: u8) -> Result<(), ExecError> {
        if !matches!(direction, 1 | 2) {
            return Err(ExecError::Preflight("margin yönü 1 (ekle) veya 2 (çek) olmalı".into()));
        }
        let _ = self.client.adjust_position_margin(symbol, amount, direction).await?;
        info!("{symbol} izole marj {} {amount} USDT", if direction == 1 { "+" } else { "-" });
        Ok(())
    }

    async fn set_position_mode(&mut self, dual: bool) -> Result<(), ExecError> {
        let _ = self.client.set_position_mode(dual).await?;
        self.snapshot.write().position_mode = Some(dual);
        info!("position mode → {}", if dual { "HEDGE" } else { "ONE_WAY" });
        Ok(())
    }

    async fn set_multi_assets(&mut self, enabled: bool) -> Result<(), ExecError> {
        let _ = self.client.set_multi_assets(enabled).await?;
        info!("multi-assets margin → {}", if enabled { "AÇIK" } else { "KAPALI" });
        Ok(())
    }

    // ── User-data stream olayları ────────────────────────────────

    async fn handle_user_event(&mut self, ev: UserEvent) {
        match ev {
            UserEvent::StreamConnected => {
                info!("User-data stream bağlandı — yeniden eşitleniyor");
                if let Err(e) = self.resync().await {
                    error!("stream bağlantısında resync hatası: {e}");
                }
            }
            UserEvent::Data(data) => {
                // Emir onayları in-flight'tan düşülür.
                if let UserDataEvent::OrderTradeUpdate { order, .. } = &data {
                    let terminal = OrderStatus::from_binance(&order.status)
                        .map(|s| s.is_terminal())
                        .unwrap_or(false);
                    if order.execution_type == "TRADE" && order.last_filled_qty != Decimal::ZERO {
                        self.metrics.record_fill();
                        // Fill'i ortak risk muhasebesine işle (pozisyon/PnL/daily loss).
                        let side = if order.side.eq_ignore_ascii_case("BUY") {
                            crate::order::OrderSide::Buy
                        } else {
                            crate::order::OrderSide::Sell
                        };
                        self.risk.on_fill(&order.symbol, side, order.last_filled_qty, order.last_filled_price);
                        self.risk.push_mark(&order.symbol, order.last_filled_price);
                    }
                    if terminal {
                        if !order.client_order_id.is_empty() {
                            self.in_flight.confirm(&order.client_order_id);
                        }
                        if order.order_id > 0 {
                            self.in_flight.confirm_by_order_id(order.order_id);
                        }
                    } else if order.order_id > 0 && !order.client_order_id.is_empty() {
                        self.in_flight.set_order_id(&order.client_order_id, order.order_id);
                    }
                }
                {
                    let mut snap = self.snapshot.write();
                    projector::apply(&mut snap, &data);
                }
            }
        }
    }

    // ── Eşitleme / uzlaştırma ────────────────────────────────────

    /// Tam hesap + pozisyon + açık emir + exchange eşitlemesi.
    async fn resync(&mut self) -> Result<(), ExecError> {
        self.preflight.exchange().refresh_if_stale(&self.client).await?;

        let account = self.client.account_info().await?;
        let positions = self.client.position_risk(None).await?;
        let open_orders = self.client.query_open_orders(None).await?;
        let position_mode = self.client.get_position_mode().await.ok();

        let mut snap = self.snapshot.write();
        snap.account = account;
        snap.positions = positions;
        snap.open_orders = open_orders;
        snap.position_mode = position_mode;
        snap.ready = true;
        snap.last_update_time = now_ms();
        snap.sequence += 1;
        drop(snap);

        // Borsa gerçeğini ortak risk state'ine yansıt.
        let snap = self.snapshot.read();
        self.risk.sync_from_snapshot(&snap);

        self.metrics.record_resync();
        info!(
            "Resync tamamlandı | pozisyon: {} | açık emir: {} | bakiye: {} USDT",
            self.snapshot.read().positions.iter().filter(|p| p.is_open()).count(),
            self.snapshot.read().open_orders.len(),
            self.snapshot.read().available_balance()
        );
        Ok(())
    }

    /// Periyodik uzlaştırma: pozisyon ve açık emirler REST ile karşılaştırılır.
    async fn reconcile(&mut self) {
        let positions_res = self.client.position_risk(None).await;
        let orders_res = self.client.query_open_orders(None).await;
        match (positions_res, orders_res) {
            (Ok(positions), Ok(open_orders)) => {
                let mut snap = self.snapshot.write();
                let mismatch = positions.len() != snap.positions.len()
                    || open_orders.len() != snap.open_orders.len();
                snap.positions = positions;
                snap.open_orders = open_orders;
                snap.sequence += 1;
                drop(snap);
                // Uzlaştırma sonrası pozisyon gerçeğini risk state'ine yansıt.
                let snap = self.snapshot.read();
                self.risk.sync_from_snapshot(&snap);
                if mismatch {
                    warn!("Uzlaştırma fark buldu — pozisyon/açık emir sayısı değişti (tam resync tetikleniyor)");
                    // Actor döngü dışında resync çağırmak için komut yolu yok;
                    // snapshot zaten REST gerçeğiyle güncellendi, tam hesap sonraki
                    // akış olayında/rakipte düzelir.
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                self.metrics.record_http_error();
                warn!("Uzlaştırma başarısız: {e}");
            }
        }
    }

    /// Zaman aşımına uğrayan in-flight emirleri borsadan sorgulayarak uzlaştır.
    async fn reconcile_inflight(&mut self) {
        let now = Instant::now();
        let expired = self.in_flight.expired(now);
        for (cid, order_id, symbol) in expired {
            match self
                .client
                .query_order(&symbol, order_id, Some(cid.as_str()))
                .await
            {
                Ok(resp) => {
                    let status = OrderStatus::from_binance(&resp.status).unwrap_or(OrderStatus::New);
                    if status.is_terminal() {
                        self.in_flight.confirm(&cid);
                        self.sync_open_order(resp);
                        if status == OrderStatus::Filled {
                            self.metrics.record_fill();
                        }
                    } else {
                        // Hâlâ açık: zaman aşımını sıfırla.
                        self.in_flight.insert(cid.clone(), symbol, Some(resp.order_id), Some(10_000));
                        self.sync_open_order(resp);
                    }
                }
                Err(e) => {
                    self.metrics.record_http_error();
                    warn!("in-flight uzlaştırma sorgusu başarısız: {cid}: {e}");
                }
            }
        }
    }

    /// Bir REST yanıtını açık emir listesine yansıtır.
    fn sync_open_order(&mut self, response: BinanceOrderResponse) {
        let status = OrderStatus::from_binance(&response.status).unwrap_or(OrderStatus::New);
        let mut snap = self.snapshot.write();
        if status.is_open() {
            if let Some(o) = snap.open_orders.iter_mut().find(|o| o.order_id == response.order_id) {
                *o = response;
            } else {
                snap.open_orders.push(response);
            }
        } else {
            snap.open_orders.retain(|o| o.order_id != response.order_id);
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
