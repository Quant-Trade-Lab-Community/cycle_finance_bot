//! Pre-trade kural zinciri testleri.

use risk_engine::audit::AuditLog;
use risk_engine::cache::RiskCache;
use risk_engine::engine::RiskEngine;
use risk_engine::kill_switch::KillSwitch;
use risk_engine::policy::RiskPolicy;
use risk_engine::types::{
    MarkPrice, OrderIntent, OrderKind, RejectReason, RiskDecision, RiskStatus, Side,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

static KS_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Her test için benzersiz kill switch yolu (dosya kirliliğini önler).
fn unique_ks_path() -> String {
    let n = KS_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("/tmp/risk_engine_test_ks_{}_{}", std::process::id(), n)
}

fn engine_with(policy: RiskPolicy) -> RiskEngine {
    RiskEngine::with_parts(
        d("10000"),
        policy,
        RiskCache::new(),
        std::sync::Arc::new(KillSwitch::new(unique_ks_path())),
        AuditLog::disabled(),
    )
}

fn market_buy(symbol: &str, qty: &str) -> OrderIntent {
    OrderIntent {
        strategy_id: 1,
        symbol: symbol.into(),
        side: Side::Buy,
        quantity: d(qty),
        price: None,
        kind: OrderKind::Market,
        reduce_only: false,
        close_position: false,
        leverage: None,
    }
}

fn limit_sell(symbol: &str, qty: &str, price: &str) -> OrderIntent {
    OrderIntent {
        strategy_id: 1,
        symbol: symbol.into(),
        side: Side::Sell,
        quantity: d(qty),
        price: Some(d(price)),
        kind: OrderKind::Limit,
        reduce_only: false,
        close_position: false,
        leverage: None,
    }
}

fn fresh_mark(engine: &RiskEngine, symbol: &str, price: &str) {
    engine.on_mark(&MarkPrice::new(symbol, d(price), now_ms()));
}

#[test]
fn market_order_without_mark_is_rejected_fail_closed() {
    let engine = engine_with(RiskPolicy::default());
    let decision = engine.evaluate(market_buy("BTCUSDT", "0.1"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert!(matches!(reason, RejectReason::StaleMark { .. })),
        RiskDecision::Approved { .. } => panic!("mark yokken market emri onaylanmamalı"),
    }
}

#[test]
fn market_order_with_fresh_mark_is_approved() {
    let mut policy = RiskPolicy::default();
    policy.max_position_usdt = d("100000");
    policy.max_notional_per_order = d("100000");
    policy.max_gross_exposure_usdt = d("100000");
    let engine = engine_with(policy);
    fresh_mark(&engine, "BTCUSDT", "50000");
    let decision = engine.evaluate(market_buy("BTCUSDT", "0.1"));
    assert!(decision.is_approved(), "0.1*50000=5000 USDT sınırlar içinde onaylanmalı");
}

#[test]
fn blocklisted_symbol_rejected() {
    let mut policy = RiskPolicy::default();
    policy.blocklist.insert("TRXUSDT".into());
    let engine = engine_with(policy);
    fresh_mark(&engine, "TRXUSDT", "0.2");
    let decision = engine.evaluate(market_buy("TRXUSDT", "100"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert!(matches!(reason, RejectReason::BlockedSymbol(_))),
        _ => panic!("blocklist'teki sembol onaylanmamalı"),
    }
}

#[test]
fn notional_limit_rejected() {
    let mut policy = RiskPolicy::default();
    policy.max_notional_per_order = d("500");
    let engine = engine_with(policy);
    fresh_mark(&engine, "BTCUSDT", "50000");
    // 0.02 * 50000 = 1000 > 500.
    let decision = engine.evaluate(market_buy("BTCUSDT", "0.02"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert!(matches!(reason, RejectReason::NotionalExceeded { .. })),
        _ => panic!("notional limiti aşılmış emir onaylanmamalı"),
    }
}

#[test]
fn position_limit_projects_after_fill() {
    let mut policy = RiskPolicy::default();
    policy.max_position_usdt = d("1000");
    policy.max_notional_per_order = d("1000");
    let engine = engine_with(policy);
    fresh_mark(&engine, "BTCUSDT", "50000");
    // 0.02 BTC = 1000 USDT — onaylanır ve fill uygulanır.
    assert!(engine.evaluate(market_buy("BTCUSDT", "0.02")).is_approved());
    engine.on_fill(&risk_engine::types::Fill {
        symbol: "BTCUSDT".into(),
        side: Side::Buy,
        quantity: d("0.02"),
        price: d("50000"),
        commission: d("0"),
        leverage: d("3"),
        ts_ms: now_ms(),
    });
    // 0.01 daha = 1500 > 1000 — red.
    let decision = engine.evaluate(market_buy("BTCUSDT", "0.01"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert!(matches!(reason, RejectReason::PositionLimitExceeded { .. })),
        _ => panic!("pozisyon limiti projeksiyonu kırılmamalı"),
    }
}

#[test]
fn stale_mark_rejects_market_order() {
    let engine = engine_with(RiskPolicy::default());
    let ts = now_ms();
    // Mark 500 sn önce → stale (eşik 200ms).
    engine.on_mark(&MarkPrice::new("BTCUSDT", d("50000"), ts.saturating_sub(500_000)));
    let decision = engine.evaluate(market_buy("BTCUSDT", "0.1"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert!(matches!(reason, RejectReason::StaleMark { .. })),
        _ => panic!("bayat mark ile market emri reddedilmeli"),
    }
}

#[test]
fn kill_switch_blocks_everything() {
    let engine = engine_with(RiskPolicy::default());
    engine.kill_switch().engage().unwrap();
    fresh_mark(&engine, "BTCUSDT", "50000");
    let decision = engine.evaluate(limit_sell("BTCUSDT", "0.1", "51000"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert_eq!(reason, RejectReason::KillSwitch),
        _ => panic!("kill switch açıkken emir onaylanmamalı"),
    }
}

#[test]
fn daily_loss_engages_kill_switch_automatically() {
    let mut policy = RiskPolicy::default();
    policy.max_daily_loss_usdt = d("50");
    policy.max_notional_per_order = d("100000");
    policy.max_position_usdt = d("100000");
    policy.max_gross_exposure_usdt = d("100000");
    policy.max_leverage = d("10");
    let engine = engine_with(policy);
    fresh_mark(&engine, "BTCUSDT", "50000");

    // Büyük kayıp: alım 1 BTC @ 50000, sonra mark 49000 → -1000 unrealized.
    assert!(engine.evaluate(market_buy("BTCUSDT", "1")).is_approved());
    engine.on_fill(&risk_engine::types::Fill {
        symbol: "BTCUSDT".into(),
        side: Side::Buy,
        quantity: d("1"),
        price: d("50000"),
        commission: d("0"),
        leverage: d("10"),
        ts_ms: now_ms(),
    });
    engine.on_mark(&MarkPrice::new("BTCUSDT", d("49000"), now_ms()));

    let snap = engine.state().snapshot();
    assert_eq!(snap.status, RiskStatus::MaxDailyLossBreached.as_str());
    assert!(snap.kill_switch, "günlük kayıp aşımı kill switch'i otomatik kapatmalı");

    // Artık her emir reddedilir.
    let decision = engine.evaluate(market_buy("BTCUSDT", "0.01"));
    assert!(decision.is_rejected());
}

#[test]
fn three_consecutive_rejections_engage_kill_switch() {
    let mut policy = RiskPolicy::default();
    policy.consecutive_rejection_auto_stop = 3;
    let engine = engine_with(policy);
    fresh_mark(&engine, "BTCUSDT", "50000");
    // Üç red: blok listesiz ama notional çok yüksek.
    policy = engine.policy();
    let mut p = policy;
    p.max_notional_per_order = d("1");
    engine.set_policy(p);

    for _ in 0..3 {
        let _ = engine.evaluate(market_buy("BTCUSDT", "1"));
    }
    assert!(engine.kill_switch().is_open(), "3 ardışık red kill switch'i kapatmalı");
}

#[test]
fn rate_limit_blocks_excess_orders() {
    let mut policy = RiskPolicy::default();
    policy.max_orders_per_min = 2;
    let engine = engine_with(policy);
    fresh_mark(&engine, "BTCUSDT", "50000");
    assert!(engine.evaluate(market_buy("BTCUSDT", "0.001")).is_approved());
    assert!(engine.evaluate(market_buy("BTCUSDT", "0.001")).is_approved());
    let decision = engine.evaluate(market_buy("BTCUSDT", "0.001"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert!(matches!(reason, RejectReason::RateLimit { .. })),
        _ => panic!("rate limit aşılınca emir reddedilmeli"),
    }
}

#[test]
fn close_position_without_mark_is_allowed_for_reduce_only() {
    // Limit emri fiyat içerdiğinden mark olmadan da onaylanır; yeni risk eklemez.
    let mut policy = RiskPolicy::default();
    policy.max_notional_per_order = d("1000");
    let engine = engine_with(policy);
    let decision = engine.evaluate(limit_sell("BTCUSDT", "0.01", "51000"));
    assert!(decision.is_approved(), "limit emri fiyat taşır, mark gerektirmez");
}

#[test]
fn margin_check_rejects_when_cash_insufficient() {
    let mut policy = RiskPolicy::default();
    policy.max_leverage = d("1"); // marj = notional
    policy.max_notional_per_order = d("100000");
    policy.max_position_usdt = d("100000");
    policy.max_gross_exposure_usdt = d("100000");
    let engine = engine_with(policy);
    fresh_mark(&engine, "BTCUSDT", "50000");
    // 0.5 BTC = 25000 USDT, nakit 10000 → yetersiz marj.
    let decision = engine.evaluate(market_buy("BTCUSDT", "0.5"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert!(matches!(reason, RejectReason::InsufficientMargin { .. })),
        _ => panic!("yetersiz marj onaylanmamalı"),
    }
}

#[test]
fn per_symbol_override_tightens_limit() {
    let mut policy = RiskPolicy::default();
    policy.max_position_usdt = d("100000");
    policy.max_notional_per_order = d("100000");
    policy.per_symbol.insert(
        "VELVETUSDT".into(),
        risk_engine::policy::PerSymbolLimits {
            max_position_usdt: Some(d("100")),
            ..Default::default()
        },
    );
    let engine = engine_with(policy);
    fresh_mark(&engine, "VELVETUSDT", "1");
    // 500 HEI = 500 USDT > 100.
    let decision = engine.evaluate(market_buy("VELVETUSDT", "500"));
    match decision {
        RiskDecision::Rejected { reason, .. } => assert!(matches!(reason, RejectReason::PositionLimitExceeded { .. })),
        _ => panic!("per-symbol pozisyon limiti işlemeli"),
    }
}
