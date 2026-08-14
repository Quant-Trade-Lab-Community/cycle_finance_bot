//! Gateway yüzeyi: stratejiler canlı borsaya emir verir.
//!
//! `LiveGateway` (canlı binance) `Gateway` trait'ini uygular; `EngineHandle`
//! tüm yazma/okuma işlemleri için tek kol.

use crate::config::TradingMode;
use crate::metrics::Metrics;
use crate::order::{
    BinanceOrderResponse, OrderAck, OrderPositionSide, OrderRequest, OrderSide, OrderType,
};
use crate::risk::kill_switch::KillSwitch;
use crate::state::snapshot::AccountSnapshot;
use crate::types::account::MarginType;
use async_trait::async_trait;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

use crate::execution::actor::Command;

/// Actor'a erişim koludur (REST servisi ve stratejiler tarafından kullanılır).
#[derive(Clone)]
pub struct EngineHandle {
    pub cmd_tx: mpsc::UnboundedSender<Command>,
    pub snapshot: Arc<RwLock<AccountSnapshot>>,
    pub metrics: Arc<Metrics>,
    pub kill_switch: Arc<KillSwitch>,
    pub config: Arc<crate::config::ExecConfig>,
}

const CMD_TIMEOUT: Duration = Duration::from_secs(10);

impl EngineHandle {
    pub async fn submit_order(&self, order: OrderRequest) -> Result<OrderAck, String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::SubmitOrder { order, tx })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        tokio::time::timeout(CMD_TIMEOUT, rx)
            .await
            .map_err(|_| "emir yanıtı zaman aşımı".to_string())?
            .map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn submit_batch(&self, orders: Vec<OrderRequest>) -> Result<Vec<OrderAck>, String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::BatchOrders { orders, tx })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        tokio::time::timeout(CMD_TIMEOUT, rx)
            .await
            .map_err(|_| "batch yanıtı zaman aşımı".to_string())?
            .map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
    ) -> Result<BinanceOrderResponse, String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::CancelOrder {
                symbol: symbol.to_string(),
                order_id,
                client_order_id: client_order_id.map(|s| s.to_string()),
                tx,
            })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        tokio::time::timeout(CMD_TIMEOUT, rx)
            .await
            .map_err(|_| "iptal yanıtı zaman aşımı".to_string())?
            .map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn cancel_all(&self, symbol: &str) -> Result<usize, String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::CancelAll {
                symbol: symbol.to_string(),
                tx,
            })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        tokio::time::timeout(CMD_TIMEOUT, rx)
            .await
            .map_err(|_| "iptal yanıtı zaman aşımı".to_string())?
            .map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    /// Sembolün açık pozisyonlarını kapatır (hedge modda LONG+SHORT; `position_side`
    /// verilirse yalnızca o taraf). Dönen değer kapatılan pozisyon sayısıdır.
    pub async fn close_symbol(
        &self,
        symbol: &str,
        position_side: Option<&str>,
    ) -> Result<usize, String> {
        let positions = self.snapshot.read().positions.clone();
        let targets: Vec<_> = positions
            .iter()
            .filter(|p| p.symbol.eq_ignore_ascii_case(symbol) && !p.position_amt.is_zero())
            .filter(|p| match position_side {
                Some(s) => p.position_side.eq_ignore_ascii_case(s),
                None => true,
            })
            .cloned()
            .collect();
        if targets.is_empty() {
            return Ok(0);
        }
        let mut closed = 0usize;
        for p in targets {
            // Pozitif amt = LONG → SELL ile kapat; negatif = SHORT → BUY ile kapat.
            let side = if p.position_amt.is_sign_positive() {
                OrderSide::Sell
            } else {
                OrderSide::Buy
            };
            let order = OrderRequest {
                symbol: p.symbol.clone(),
                side,
                order_type: OrderType::Market,
                quantity: p.position_amt.abs(),
                position_side: match p.position_side.as_str() {
                    "LONG" => OrderPositionSide::Long,
                    "SHORT" => OrderPositionSide::Short,
                    _ => OrderPositionSide::Both,
                },
                client_order_id: Some(format!("close_{}_{}", p.symbol, now_ms())),
                ..Default::default()
            };
            self.submit_order(order).await?;
            closed += 1;
        }
        Ok(closed)
    }

    /// Tüm açık pozisyonları kapatır. Dönen değer kapatılan pozisyon sayısıdır.
    pub async fn close_all(&self) -> Result<usize, String> {
        let positions = self.snapshot.read().positions.clone();
        let symbols: std::collections::HashSet<String> = positions
            .iter()
            .filter(|p| !p.position_amt.is_zero())
            .map(|p| p.symbol.clone())
            .collect();
        if symbols.is_empty() {
            return Ok(0);
        }
        let mut total = 0usize;
        for symbol in symbols {
            total += self.close_symbol(&symbol, None).await?;
        }
        Ok(total)
    }

    pub async fn query_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
    ) -> Result<BinanceOrderResponse, String> {        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::QueryOrder {
                symbol: symbol.to_string(),
                order_id,
                client_order_id: client_order_id.map(|s| s.to_string()),
                tx,
            })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        tokio::time::timeout(CMD_TIMEOUT, rx)
            .await
            .map_err(|_| "sorgu yanıtı zaman aşımı".to_string())?
            .map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn modify_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
        quantity: Option<Decimal>,
        price: Option<Decimal>,
        stop_price: Option<Decimal>,
    ) -> Result<BinanceOrderResponse, String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::ModifyOrder {
                symbol: symbol.to_string(),
                order_id,
                client_order_id: client_order_id.map(|s| s.to_string()),
                quantity,
                price,
                stop_price,
                tx,
            })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        tokio::time::timeout(CMD_TIMEOUT, rx)
            .await
            .map_err(|_| "modify yanıtı zaman aşımı".to_string())?
            .map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::SetLeverage {
                symbol: symbol.to_string(),
                leverage,
                tx,
            })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        rx.await.map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn set_margin_type(&self, symbol: &str, margin_type: MarginType) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::SetMarginType {
                symbol: symbol.to_string(),
                margin_type,
                tx,
            })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        rx.await.map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn adjust_margin(&self, symbol: &str, amount: Decimal, direction: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::AdjustMargin {
                symbol: symbol.to_string(),
                amount,
                direction,
                tx,
            })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        rx.await.map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn set_position_mode(&self, dual: bool) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::SetPositionMode { dual, tx })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        rx.await.map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub async fn set_multi_assets(&self, enabled: bool) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::SetMultiAssets { enabled, tx })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        rx.await.map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    /// Kill switch aç/kapat. Kapatırken devre kesici sıfırlanır.
    pub async fn set_kill_switch(&self, enabled: bool) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::SetKillSwitch { enabled, tx })
            .map_err(|_| "actor kanalı kapalı".to_string())?;
        tokio::time::timeout(CMD_TIMEOUT, rx)
            .await
            .map_err(|_| "kill switch yanıtı zaman aşımı".to_string())?
            .map_err(|_| "yanıt kanalı kapandı".to_string())?
    }

    pub fn snapshot(&self) -> AccountSnapshot {
        self.snapshot.read().clone()
    }

    pub fn mode(&self) -> TradingMode {
        self.config.mode
    }

    pub fn dry_run(&self) -> bool {
        self.config.dry_run
    }
}

/// Strateji katmanının gördüğü soyut emir yüzeyi.
#[async_trait]
pub trait Gateway: Send + Sync {
    async fn submit_order(&self, order: OrderRequest) -> Result<OrderAck, String>;
    fn snapshot(&self) -> AccountSnapshot;
    fn mode(&self) -> TradingMode;
}

/// Canlı Binance Futures gateway'i.
pub struct LiveGateway {
    handle: EngineHandle,
}

impl LiveGateway {
    pub fn new(handle: EngineHandle) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> &EngineHandle {
        &self.handle
    }
}

#[async_trait]
impl Gateway for LiveGateway {
    async fn submit_order(&self, order: OrderRequest) -> Result<OrderAck, String> {
        self.handle.submit_order(order).await
    }

    fn snapshot(&self) -> AccountSnapshot {
        self.handle.snapshot()
    }

    fn mode(&self) -> TradingMode {
        self.handle.mode()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
