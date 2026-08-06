//! Event Sourcing replay doğrulaması.

use execution_engine::paper::actor::PaperEngineActor;
use execution_engine::paper::config::PaperConfig;
use execution_engine::paper::domain_event::DomainEvent;
use rust_decimal::Decimal;
use std::str::FromStr;

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

#[tokio::test]
async fn test_replay_rebuilds_positions_and_cash() {
    // Varsayılan config (PRICE_ONLY)
    let config = PaperConfig::load_from_env();

    // 1 BTC @ 50000 long, komisyon 0, marj = 50000/20 = 2500
    let events = vec![
        DomainEvent::OrderFilled {
            order_id: "T1".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            fill_price: dec("50000"),
            fill_qty: dec("1"),
            commission: dec("25"),
            cash_delta: dec("-25"), // marj kilidi + komisyon
            realized_pnl: dec("0"),
            leverage: dec("20"),
        },
    ];

    let actor = PaperEngineActor::new_with_events(config.clone(), None, &events);

    let pos = actor.positions().get("BTCUSDT").expect("pozisyon açık olmalı");
    assert_eq!(pos.quantity, dec("1"));
    assert_eq!(pos.avg_entry_price, dec("50000"));

    // cash = initial - 25
    let expected_cash = config.initial_usdt - dec("25");
    assert_eq!(actor.account().get_free("USDT"), expected_cash);
}

#[tokio::test]
async fn test_replay_close_realizes_pnl() {
    let config = PaperConfig::load_from_env();
    let events = vec![
        DomainEvent::OrderFilled {
            order_id: "T1".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            fill_price: dec("50000"),
            fill_qty: dec("1"),
            commission: dec("25"),
            cash_delta: dec("-25"),
            realized_pnl: dec("0"),
            leverage: dec("20"),
        },
        DomainEvent::OrderFilled {
            order_id: "T2".into(),
            symbol: "BTCUSDT".into(),
            side: "SELL".into(),
            fill_price: dec("51000"),
            fill_qty: dec("1"),
            commission: dec("25.5"),
            cash_delta: dec("2500").checked_add(dec("1000")).unwrap() - dec("25.5"),
            realized_pnl: dec("1000"),
            leverage: dec("20"),
        },
    ];

    let actor = PaperEngineActor::new_with_events(config.clone(), None, &events);
    assert!(actor.positions().get("BTCUSDT").is_none(), "pozisyon kapanmış olmalı");

    let expected_cash = config.initial_usdt
        - dec("25")
        + dec("2500")
        + dec("1000")
        - dec("25.5");
    assert_eq!(actor.account().get_free("USDT"), expected_cash);
}

#[tokio::test]
async fn test_funding_applies_to_cash() {
    let config = PaperConfig::load_from_env();
    let events = vec![
        DomainEvent::OrderFilled {
            order_id: "T1".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            fill_price: dec("50000"),
            fill_qty: dec("1"),
            commission: dec("0"),
            cash_delta: dec("-2500"),
            realized_pnl: dec("0"),
            leverage: dec("20"),
        },
        DomainEvent::FundingRateApplied {
            symbol: "BTCUSDT".into(),
            rate: dec("0.0001"),
            payment: dec("-5"),
        },
    ];

    let actor = PaperEngineActor::new_with_events(config.clone(), None, &events);
    let expected_cash = config.initial_usdt - dec("2500") - dec("5");
    assert_eq!(actor.account().get_free("USDT"), expected_cash);
}
