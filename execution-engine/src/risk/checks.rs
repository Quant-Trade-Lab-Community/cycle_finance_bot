//! Emir öncesi risk kontrolleri — ortak `risk-engine` çekirdeğine ince bağdaştırıcı.
//!
//! Tüm risk kuralları `risk_engine::RiskEngine`'de yaşar (tek doğruluk kaynağı);
//! bu modül yalnızca `OrderRequest` → `OrderIntent` eşlemesi ve borsa snapshot'ı
//! → risk state senkronizasyonunu yapar. API geriye dönük uyumludur.

use crate::config::ExecConfig;
use crate::error::{ExecError, Result};
use crate::order::{OrderRequest, OrderType};
use crate::state::snapshot::AccountSnapshot;
use risk_engine::audit::AuditLog;
use risk_engine::cache::RiskCache;
use risk_engine::engine::RiskEngine;
use risk_engine::kill_switch::KillSwitch;
use risk_engine::policy::{PerSymbolLimits, RiskPolicy};
use risk_engine::types::{MarkPrice, OrderIntent, OrderKind, RiskDecision, Side};
use rust_decimal::Decimal;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RiskChecks {
    engine: Arc<RiskEngine>,
}

impl RiskChecks {
    /// Yapılandırmadan ortak risk çekirdeğini kurar (kendi kill switch'ini üretir).
    pub fn new(config: &ExecConfig) -> Self {
        Self::with_kill_switch(
            config,
            Arc::new(KillSwitch::new(config.kill_switch_path.clone())),
        )
    }

    /// Actor ile AYNI kill switch'i paylaşır — aksi halde actor'den yapılan
    /// release, RiskEngine'in ayrı bayrağını sıfırlamaz ve kill switch açık
    /// kalmaya devam ederdi (her emir reddedilip switch yeniden arm edilirdi).
    pub fn with_kill_switch(config: &ExecConfig, kill_switch: Arc<KillSwitch>) -> Self {
        // Geriye dönük davranış: max_notional aynı zamanda sembol pozisyon tavanıdır.
        let policy = RiskPolicy {
            max_notional_per_order: config.max_notional_usdt,
            max_orders_per_min: config.max_orders_per_min,
            blocklist: config.symbol_blocklist.clone(),
            max_position_usdt: config.max_notional_usdt,
            ..Default::default()
        };

        let engine = RiskEngine::with_parts(
            Decimal::ZERO,
            policy,
            RiskCache::new(),
            kill_switch,
            AuditLog::disabled(),
        );
        Self {
            engine: Arc::new(engine),
        }
    }

    pub fn engine(&self) -> &Arc<RiskEngine> {
        &self.engine
    }

    /// Emir gönderim öncesi tam risk zinciri.
    pub fn check(&self, order: &OrderRequest) -> Result<()> {
        let intent = order_intent(order);
        match self.engine.evaluate(intent) {
            RiskDecision::Approved { .. } => Ok(()),
            RiskDecision::Rejected { reason, .. } => Err(ExecError::Risk(reason.describe())),
        }
    }

    /// Başarılı gönderim sonrası rate-limit penceresine kaydeder.
    pub fn record_order(&self) {
        self.engine.record_approved();
    }

    /// Devre kesici sayacını sıfırlar (kill switch kapatılınca çağrılır).
    pub fn reset_breaker(&self) {
        self.engine.reset_breaker();
    }

    // ── Snapshot senkronizasyonu ──

    /// Resync sonrası borsa gerçeğini risk state'ine yansıtır.
    pub fn sync_from_snapshot(&self, snap: &AccountSnapshot) {
        self.engine.set_cash_balance(snap.available_balance());
        self.engine.set_open_orders_notional(snap.open_orders_notional());
        for p in snap.positions.iter().filter(|p| p.is_open()) {
            self.engine.sync_position(&p.symbol, p.position_amt, p.entry_price, p.leverage);
            self.engine.on_mark(&MarkPrice::new(&p.symbol, p.mark_price, now_ms()));
        }
    }

    /// Harici bir mark fiyatını risk state'ine besler (ör. markprice flow ring).
    pub fn push_mark(&self, symbol: &str, price: Decimal) {
        self.engine.on_mark(&MarkPrice::new(symbol, price, now_ms()));
    }

    /// Gerçekleşen bir fill'i risk muhasebesine işler.
    pub fn on_fill(&self, symbol: &str, side: crate::order::OrderSide, quantity: Decimal, price: Decimal) {
        let fill = risk_engine::types::Fill {
            symbol: symbol.to_uppercase(),
            side: if side == crate::order::OrderSide::Buy { Side::Buy } else { Side::Sell },
            quantity,
            price,
            commission: Decimal::ZERO,
            leverage: Decimal::ONE,
            ts_ms: now_ms(),
        };
        self.engine.on_fill(&fill);
    }

    // ── Geriye dönük API ──

    pub fn max_notional(&self) -> Decimal {
        self.engine.policy().max_notional_per_order
    }

    pub fn set_max_notional(&mut self, v: Decimal) {
        let mut p = self.engine.policy();
        p.max_notional_per_order = v;
        p.max_position_usdt = v;
        self.engine.set_policy(p);
    }

    pub fn set_max_orders_per_min(&mut self, v: u32) {
        let mut p = self.engine.policy();
        p.max_orders_per_min = v;
        self.engine.set_policy(p);
    }

    pub fn set_blocklist(&mut self, list: HashSet<String>) {
        let mut p = self.engine.policy();
        p.blocklist = list;
        self.engine.set_policy(p);
    }

    pub fn orders_in_window(&self) -> usize {
        self.engine.orders_in_window()
    }

    pub fn set_per_symbol_limit(&mut self, symbol: &str, max_position_usdt: Decimal) {
        let mut p = self.engine.policy();
        p.per_symbol.insert(
            symbol.to_uppercase(),
            PerSymbolLimits {
                max_position_usdt: Some(max_position_usdt),
                ..Default::default()
            },
        );
        self.engine.set_policy(p);
    }
}

/// `OrderRequest` → `OrderIntent` eşlemesi.
fn order_intent(order: &OrderRequest) -> OrderIntent {
    let side = match order.side {
        crate::order::OrderSide::Buy => Side::Buy,
        crate::order::OrderSide::Sell => Side::Sell,
    };
    OrderIntent {
        strategy_id: 0,
        symbol: order.symbol.to_uppercase(),
        side,
        quantity: order.quantity.abs(),
        price: order.price,
        kind: if order.order_type == OrderType::Market {
            OrderKind::Market
        } else {
            OrderKind::Limit
        },
        reduce_only: order.reduce_only.unwrap_or(false),
        close_position: order.close_position.unwrap_or(false),
        leverage: None,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
