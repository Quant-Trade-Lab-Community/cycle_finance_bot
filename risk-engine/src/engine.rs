//! RiskEngine — pre-trade kural zinciri (hot path).
//!
//! Kurallar maliyet sırasına göre, fail-fast çalışır. Her reddin nedeni
//! `RejectReason` ile yapılandırılır ve denetim izine yazılır. Ardışık red
//! eşiği aşılırsa kill switch otomatik devreye girer.

use crate::audit::AuditLog;
use crate::cache::RiskCache;
use crate::exposure;
use crate::kill_switch::KillSwitch;
use crate::limits::{CircuitBreaker, RateLimit};
use crate::policy::RiskPolicy;
use crate::types::{OrderIntent, RejectReason, RiskDecision, RiskStatus};
use parking_lot::{Mutex, RwLock};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RiskEngine {
    policy: RwLock<RiskPolicy>,
    state: Arc<crate::state::RiskState>,
    kill_switch: Arc<KillSwitch>,
    cache: Arc<RiskCache>,
    rate_limit: Mutex<RateLimit>,
    breaker: Mutex<CircuitBreaker>,
    audit: AuditLog,
}

impl RiskEngine {
    /// Varsayılan politikayla kurar.
    pub fn new(initial_balance: Decimal) -> Self {
        Self::with_policy(initial_balance, RiskPolicy::default())
    }

    /// Belirtilen politikayla kurar.
    pub fn with_policy(initial_balance: Decimal, policy: RiskPolicy) -> Self {
        Self::with_parts(initial_balance, policy, RiskCache::new(), KillSwitch::new("/tmp/exec_kill_switch".into()), AuditLog::disabled())
    }

    /// Tam kurucu (test / embed için).
    #[allow(clippy::too_many_arguments)]
    pub fn with_parts(
        initial_balance: Decimal,
        policy: RiskPolicy,
        cache: RiskCache,
        kill_switch: KillSwitch,
        audit: AuditLog,
    ) -> Self {
        let kill_switch = Arc::new(kill_switch);
        let state = crate::state::RiskState::with_parts(
            crate::accounting::Portfolio::new_with_margin(
                initial_balance,
                policy.max_drawdown_pct,
                policy.maintenance_margin_rate,
            ),
            policy.clone(),
            cache.clone(),
            kill_switch.clone(),
        );
        let max_orders = policy.max_orders_per_min;
        let breaker_max = policy.consecutive_rejection_auto_stop;
        Self {
            policy: RwLock::new(policy),
            state: Arc::new(state),
            kill_switch,
            cache: Arc::new(cache),
            rate_limit: Mutex::new(RateLimit::new(max_orders)),
            breaker: Mutex::new(CircuitBreaker::new(breaker_max)),
            audit,
        }
    }

    pub fn state(&self) -> &Arc<crate::state::RiskState> {
        &self.state
    }

    pub fn policy(&self) -> RiskPolicy {
        self.policy.read().clone()
    }

    pub fn set_policy(&self, policy: RiskPolicy) {
        *self.policy.write() = policy;
    }

    pub fn kill_switch(&self) -> &Arc<KillSwitch> {
        &self.kill_switch
    }

    /// Pre-trade kural zinciri.
    pub fn evaluate(&self, intent: OrderIntent) -> RiskDecision {
        let ts = now_ms();

        // 1. Kill switch.
        if self.kill_switch.is_open() {
            return self.reject(&intent, RejectReason::KillSwitch, ts);
        }

        let policy = self.policy.read().clone();
        let g = self.state.read();
        let limits = policy.effective(&intent.symbol);
        let mark_prices: HashMap<String, Decimal> = g
            .mark_prices
            .iter()
            .map(|(k, v)| (k.clone(), v.price))
            .collect();

        // 2. Circuit breaker durumu.
        if self.breaker.lock().consecutive_rejections >= policy.consecutive_rejection_auto_stop
            && policy.consecutive_rejection_auto_stop > 0
        {
            return self.reject(&intent, RejectReason::CircuitBreaker, ts);
        }

        // 3. Kalıcı durum ihlali.
        if g.status.halts_trading() {
            let reason = match g.status {
                RiskStatus::MaxDailyLossBreached => {
                    let loss = g.portfolio.daily_loss(&mark_prices);
                    RejectReason::DailyLossExceeded { loss, limit: policy.max_daily_loss_usdt }
                }
                RiskStatus::MaxDrawdownBreached => {
                    let equity = g.portfolio.get_total_equity(&mark_prices);
                    RejectReason::DrawdownExceeded {
                        drawdown_pct: g.portfolio.drawdown_pct(equity),
                        max: policy.max_drawdown_pct,
                    }
                }
                RiskStatus::Liquidation => RejectReason::LiquidationProximity { symbol: intent.symbol.clone() },
                RiskStatus::MaxLeverageBreached => RejectReason::LeverageExceeded { max: limits.max_leverage },
                _ => RejectReason::CircuitBreaker,
            };
            return self.reject(&intent, reason, ts);
        }

        // 4. Blocklist.
        if policy.is_blocked(&intent.symbol) {
            return self.reject(&intent, RejectReason::BlockedSymbol(intent.symbol.clone()), ts);
        }

        // 5. Rate limit.
        if let Err(limit) = self.rate_limit.lock().check() {
            return self.reject(&intent, RejectReason::RateLimit { limit }, ts);
        }

        // 6. Fiyat kaynağı: limit emrinde emir fiyatı, market emrinde mark (fail-closed).
        let mark = g.mark_prices.get(&intent.symbol);
        let mark_stale = match mark {
            Some(m) => ts.saturating_sub(m.ts_ms) > policy.stale_mark_ms,
            None => true,
        };
        if intent.price.is_none() && mark_stale {
            let age_ms = mark.map(|m| ts.saturating_sub(m.ts_ms)).unwrap_or(u64::MAX);
            return self.reject(&intent, RejectReason::StaleMark { symbol: intent.symbol.clone(), age_ms }, ts);
        }
        let price = intent.price.or(mark.map(|m| m.price));
        let notional = match intent.notional(price) {
            Some(n) => n,
            None => {
                return self.reject(
                    &intent,
                    RejectReason::StaleMark { symbol: intent.symbol.clone(), age_ms: u64::MAX },
                    ts,
                )
            }
        };

        // 7. Notional limit.
        if limits.max_notional_per_order > Decimal::ZERO && notional > limits.max_notional_per_order {
            return self.reject(&intent, RejectReason::NotionalExceeded { notional, max: limits.max_notional_per_order }, ts);
        }

        // 8. Kaldıraç limiti.
        let eff_leverage = intent.leverage.unwrap_or(limits.max_leverage);
        if eff_leverage > limits.max_leverage {
            return self.reject(&intent, RejectReason::LeverageExceeded { max: limits.max_leverage }, ts);
        }

        // 9. Pozisyon limiti (projeksiyon).
        if limits.max_position_usdt > Decimal::ZERO {
            let existing = g
                .portfolio
                .positions
                .get(&intent.symbol)
                .map(|p| p.quantity)
                .unwrap_or(Decimal::ZERO);
            let projected = (existing + intent.signed_quantity()).abs() * price.unwrap_or(Decimal::ZERO);
            if projected > limits.max_position_usdt {
                return self.reject(
                    &intent,
                    RejectReason::PositionLimitExceeded {
                        symbol: intent.symbol.clone(),
                        current_notional: projected,
                        max: limits.max_position_usdt,
                    },
                    ts,
                );
            }
        }

        let signed_delta = intent.signed_quantity() * price.unwrap_or(Decimal::ZERO);

        // 10. Brüt exposure limiti.
        if policy.max_gross_exposure_usdt > Decimal::ZERO {
            let projected_gross =
                exposure::projected_gross_exposure(&g.portfolio.positions, &mark_prices, &intent.symbol, signed_delta);
            if projected_gross > policy.max_gross_exposure_usdt {
                return self.reject(&intent, RejectReason::ExposureLimitExceeded { gross: projected_gross, max: policy.max_gross_exposure_usdt }, ts);
            }
        }

        // 11. Konsantrasyon limiti.
        if policy.max_hhi > 0.0 {
            let sum = exposure::exposure(&g.portfolio.positions, &mark_prices);
            let hhi = sum.hhi;
            if hhi > policy.max_hhi {
                return self.reject(&intent, RejectReason::ConcentrationExceeded { hhi, max: policy.max_hhi }, ts);
            }
        }

        // 12. Marj yeterliliği.
        let available = g.portfolio.cash_balance - g.open_orders_notional;
        let margin_required = notional / eff_leverage;
        if margin_required > available {
            return self.reject(&intent, RejectReason::InsufficientMargin { required: margin_required, available }, ts);
        }

        // 13. Parametrik risk kapısı (worker çıktısına bağlı, opsiyonel).
        if policy.gate_on_parametric_risk {
            let params = self.cache.read();
            if !params.available || !params.gate_ready {
                return self.reject(&intent, RejectReason::ParametricRiskUnavailable, ts);
            }
        }

        drop(g);

        // Onay: rate-limit penceresine kaydet, breaker'ı sıfırla, audit et.
        self.rate_limit.lock().record();
        self.breaker.lock().record_approval();
        self.audit.record_approved(&intent, ts);
        RiskDecision::Approved { intent }
    }

    /// Ardışık red sayısı bu eşiği geçerse kill switch otomatik devreye girer.
    /// (Burada dokümantasyon amacıyla; gerçek kullanım `policy` üzerindendir.)
    pub fn reset_breaker(&self) {
        self.breaker.lock().reset();
    }

    /// Onaylanan emri "fiilen gönderildi" olarak işaretler (rate-limit penceresi).
    /// `evaluate` onay sonrası zaten kaydeder; bu metot dış çağrılar (batch) içindir.
    pub fn record_approved(&self) {
        self.rate_limit.lock().record();
        self.breaker.lock().record_approval();
    }

    /// Fill uygular (gerçekleşen PnL + pozisyon + status).
    pub fn on_fill(&self, fill: &crate::types::Fill) {
        let realized = self.state.process_fill(fill);
        self.audit.record_fill(&fill.symbol, fill.quantity, fill.price);
        // Fill sonrası durum değerlendirmesi zaten `process_fill` içinde yapılır.
        let _ = realized;
    }

    /// Mark fiyat güncellemesi (unrealized PnL / drawdown / likidasyon).
    pub fn on_mark(&self, mark: &crate::types::MarkPrice) {
        self.state.update_mark(mark);
    }

    /// Nakit bakiyeyi dış gerçeklikle senkronize eder (execution resync).
    pub fn set_cash_balance(&self, v: Decimal) {
        self.state.set_cash_balance(v);
    }

    /// Bekleyen emir notional rezervini ayarlar.
    pub fn set_open_orders_notional(&self, v: Decimal) {
        self.state.set_open_orders_notional(v);
    }

    /// Pozisyonu dış gerçeklikle senkronize eder (resync/uzlaştırma).
    pub fn sync_position(&self, symbol: &str, quantity: Decimal, avg_entry: Decimal, leverage: Decimal) {
        self.state.sync_position(symbol, quantity, avg_entry, leverage);
    }

    /// Şu anki kayan-pencere emir sayısı (60 sn).
    pub fn orders_in_window(&self) -> usize {
        self.rate_limit.lock().count()
    }

    /// Politika limitlerini önbellekteki worker çıktısına göre daraltabilir
    /// (opsiyonel — varsayılan olarak politika değişmez).
    pub fn apply_worker_params(&self, params: &crate::cache::RiskParameters) {
        let mut p = self.policy.write();
        if params.available && params.suggested_max_position_usdt > Decimal::ZERO
            && (p.max_position_usdt.is_zero() || params.suggested_max_position_usdt < p.max_position_usdt)
        {
            p.max_position_usdt = params.suggested_max_position_usdt;
        }
        if params.available && params.suggested_max_leverage > Decimal::ZERO
            && (p.max_leverage.is_zero() || params.suggested_max_leverage < p.max_leverage)
        {
            p.max_leverage = params.suggested_max_leverage;
        }
    }

    fn reject(&self, intent: &OrderIntent, reason: RejectReason, ts: u64) -> RiskDecision {
        // Breaker'ı artır; eşik aşılırsa kill switch.
        let trip = self.breaker.lock().record_rejection();
        if trip {
            let _ = self.kill_switch.engage();
        }
        self.audit.record_rejected(intent, &reason, ts);
        RiskDecision::Rejected { intent: intent.clone(), reason }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl RiskEngine {
    /// Eski `engine.rs` API uyumlu kısa yardımcı: yalnızca pozisyon limiti.
    /// `max_position` USDT notional üst sınırı, `daily_loss_limit` günlük kayıp sınırı.
    pub fn with_limits(max_position_usdt: Decimal, daily_loss_usdt: Decimal) -> Self {
        let policy = RiskPolicy {
            max_position_usdt,
            max_notional_per_order: max_position_usdt,
            max_daily_loss_usdt: daily_loss_usdt,
            ..Default::default()
        };
        Self::with_policy(max_position_usdt, policy)
    }

    /// Tekil (scalar) pozisyon modeli yerine tam portföy: mevcut pozisyon toplamı.
    pub fn current_position(&self) -> Decimal {
        let g = self.state.read();
        g.portfolio
            .positions
            .values()
            .map(|p| p.quantity)
            .sum()
    }
}
