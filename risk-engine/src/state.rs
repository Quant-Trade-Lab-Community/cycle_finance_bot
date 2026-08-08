//! Risk state — pozisyon/mark/cash/PnL ve ihlal durumu (tek doğruluk kaynağı).
//!
//! Hot path'te `parking_lot::RwLock` ile paylaşılır. Yazma (fill/mark) kısa ve
//! kilit süresi minimallidir; okuma (pre-trade) allocation-free değildir ama
//! nadirdir ve tek lock'tır.

use crate::accounting::{Portfolio, Position};
use crate::cache::RiskCache;
use crate::kill_switch::KillSwitch;
use crate::policy::RiskPolicy;
use crate::types::{Fill, MarkPrice, RiskStatus};
use parking_lot::RwLock;
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Risk state içi — `RwLock` altında.
#[derive(Debug, Clone)]
pub struct RiskStateInner {
    pub portfolio: Portfolio,
    pub mark_prices: HashMap<String, MarkPrice>,
    pub status: RiskStatus,
    /// Bekleyen emirlerin rezerve ettiği notional (marj kontrolünde kullanılır).
    pub open_orders_notional: Decimal,
}

/// Paylaşılan risk state.
pub struct RiskState {
    inner: RwLock<RiskStateInner>,
    policy: Arc<dyn PolicySource>,
    cache: Arc<RiskCache>,
    kill_switch: Arc<KillSwitch>,
}

/// Politika erişim soyutlaması (sıcak reload için).
pub trait PolicySource: Send + Sync {
    fn policy(&self) -> RiskPolicy;
}

impl PolicySource for RwLock<RiskPolicy> {
    fn policy(&self) -> RiskPolicy {
        self.read().clone()
    }
}

impl RiskState {
    pub fn new(initial_balance: Decimal, max_drawdown: Decimal) -> Self {
        Self::with_parts(
            Portfolio::new(initial_balance, max_drawdown),
            RiskPolicy::default(),
            RiskCache::new(),
            Arc::new(KillSwitch::new("/tmp/exec_kill_switch".into())),
        )
    }

    pub fn with_policy(initial_balance: Decimal, policy: RiskPolicy) -> Self {
        let mut portfolio = Portfolio::new(initial_balance, policy.max_drawdown_pct);
        portfolio.maintenance_margin_rate = policy.maintenance_margin_rate;
        Self::with_parts(portfolio, policy, RiskCache::new(), Arc::new(KillSwitch::new("/tmp/exec_kill_switch".into())))
    }

    pub fn with_parts(
        portfolio: Portfolio,
        policy: RiskPolicy,
        cache: RiskCache,
        kill_switch: Arc<KillSwitch>,
    ) -> Self {
        Self {
            inner: RwLock::new(RiskStateInner {
                portfolio,
                mark_prices: HashMap::new(),
                status: RiskStatus::Ok,
                open_orders_notional: Decimal::ZERO,
            }),
            policy: Arc::new(RwLock::new(policy)),
            cache: Arc::new(cache),
            kill_switch,
        }
    }

    pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, RiskStateInner> {
        self.inner.read()
    }

    pub fn policy(&self) -> RiskPolicy {
        self.policy.policy()
    }

    pub fn cache(&self) -> &RiskCache {
        &self.cache
    }

    pub fn kill_switch(&self) -> &Arc<KillSwitch> {
        &self.kill_switch
    }

    /// Fill uygular; gerçekleşen PnL döner.
    pub fn process_fill(&self, fill: &Fill) -> Decimal {
        let mut g = self.inner.write();
        let realized = g.portfolio.apply_fill(fill);
        self.evaluate_status(&mut g);
        drop(g);
        realized
    }

    /// Mark fiyatı günceller; day roll + status değerlendirmesi yapar.
    pub fn update_mark(&self, mark: &MarkPrice) {
        let mut g = self.inner.write();
        g.portfolio.roll_day(mark.ts_ms);
        g.mark_prices.insert(mark.symbol.clone(), mark.clone());
        self.evaluate_status(&mut g);
        drop(g);
    }

    /// Bekleyen emir notional rezervini ayarlar.
    pub fn set_open_orders_notional(&self, v: Decimal) {
        self.inner.write().open_orders_notional = v;
    }

    /// Nakit bakiyeyi borsa gerçeğiyle senkronize eder (resync).
    pub fn set_cash_balance(&self, v: Decimal) {
        self.inner.write().portfolio.cash_balance = v;
    }

    /// Pozisyonu borsa gerçeğiyle senkronize eder (resync/uzlaştırma).
    pub fn sync_position(&self, symbol: &str, quantity: Decimal, avg_entry: Decimal, leverage: Decimal) {
        let mut g = self.inner.write();
        let key = symbol.to_uppercase();
        if quantity.is_zero() {
            g.portfolio.positions.remove(&key);
            return;
        }
        let pos = g.portfolio.positions.entry(key.clone()).or_insert(Position {
            symbol: key,
            quantity: Decimal::ZERO,
            avg_entry_price: Decimal::ZERO,
            leverage,
        });
        pos.quantity = quantity;
        pos.avg_entry_price = avg_entry;
        pos.leverage = leverage;
    }

    /// Status değerlendirmesi + otomatik kill switch.
    fn evaluate_status(&self, g: &mut RiskStateInner) {
        let policy = self.policy.policy();
        let prices: HashMap<String, Decimal> = g
            .mark_prices
            .iter()
            .map(|(k, v)| (k.clone(), v.price))
            .collect();

        let equity = g.portfolio.get_total_equity(&prices);
        g.portfolio.update_peak(equity);
        let drawdown = g.portfolio.drawdown_pct(equity);
        let daily_loss = g.portfolio.daily_loss(&prices);

        let mut new_status = RiskStatus::Ok;
        if drawdown > policy.max_drawdown_pct {
            new_status = RiskStatus::MaxDrawdownBreached;
        } else if daily_loss <= -policy.max_daily_loss_usdt && policy.max_daily_loss_usdt > Decimal::ZERO {
            new_status = RiskStatus::MaxDailyLossBreached;
        }

        // Likidasyon kontrolü.
        for p in g.portfolio.positions.values() {
            let mark = prices.get(&p.symbol).copied().unwrap_or(p.avg_entry_price);
            if p.liquidation_breached(mark, g.portfolio.maintenance_margin_rate) {
                new_status = RiskStatus::Liquidation;
                break;
            }
        }

        g.status = new_status;
        if new_status.halts_trading() {
            let _ = self.kill_switch.engage();
        }
    }

    /// Salt okunur snapshot (REST/CLI için).
    pub fn snapshot(&self) -> RiskSnapshot {
        let g = self.inner.read();
        let policy = self.policy.policy();
        let prices: HashMap<String, Decimal> = g
            .mark_prices
            .iter()
            .map(|(k, v)| (k.clone(), v.price))
            .collect();
        let equity = g.portfolio.get_total_equity(&prices);
        let gross = g.portfolio.gross_exposure(&prices);
        let net = g.portfolio.net_exposure(&prices);
        let positions = g
            .portfolio
            .positions
            .values()
            .map(|p| PositionView {
                symbol: p.symbol.clone(),
                quantity: p.quantity.to_string(),
                avg_entry_price: p.avg_entry_price.to_string(),
                unrealized_pnl: prices
                    .get(&p.symbol)
                    .map(|m| p.unrealized_pnl(*m).to_string())
                    .unwrap_or_else(|| "0".into()),
                liquidation_price: p
                    .liquidation_price(g.portfolio.maintenance_margin_rate)
                    .to_string(),
            })
            .collect();
        RiskSnapshot {
            cash_balance: g.portfolio.cash_balance.to_string(),
            realized_pnl: g.portfolio.realized_pnl.to_string(),
            unrealized_pnl: g.portfolio.unrealized_pnl(&prices).to_string(),
            equity: equity.to_string(),
            peak_equity: g.portfolio.peak_equity.to_string(),
            drawdown_pct: g.portfolio.drawdown_pct(equity).to_string(),
            daily_loss: g.portfolio.daily_loss(&prices).to_string(),
            gross_exposure: gross.to_string(),
            net_exposure: net.to_string(),
            status: g.status.as_str().to_string(),
            kill_switch: self.kill_switch.is_open(),
            max_drawdown_pct: policy.max_drawdown_pct.to_string(),
            max_daily_loss_usdt: policy.max_daily_loss_usdt.to_string(),
            positions,
            mark_prices: prices
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
        }
    }
}

/// REST/CLI için serileştirilebilir risk görünümü.
#[derive(Debug, Clone, Serialize)]
pub struct RiskSnapshot {
    pub cash_balance: String,
    pub realized_pnl: String,
    pub unrealized_pnl: String,
    pub equity: String,
    pub peak_equity: String,
    pub drawdown_pct: String,
    pub daily_loss: String,
    pub gross_exposure: String,
    pub net_exposure: String,
    pub status: String,
    pub kill_switch: bool,
    pub max_drawdown_pct: String,
    pub max_daily_loss_usdt: String,
    pub positions: Vec<PositionView>,
    pub mark_prices: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionView {
    pub symbol: String,
    pub quantity: String,
    pub avg_entry_price: String,
    pub unrealized_pnl: String,
    pub liquidation_price: String,
}
