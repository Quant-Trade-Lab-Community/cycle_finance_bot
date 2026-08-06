//! Actor end-to-end: fiyat besleme + emir dolumu + event kalıcılığı.

use execution_engine::order::{OrderRequest, OrderSide, OrderType};
use execution_engine::paper::actor::{ActorCommand, PaperEngineActor};
use execution_engine::paper::config::PaperConfig;
use execution_engine::paper::domain_event::DomainEvent;
use rust_decimal::Decimal;
use std::str::FromStr;
use tokio::sync::mpsc;

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

#[tokio::test]
async fn test_market_buy_fills_and_emits_event() {
    std::env::set_var("PAPER_INITIAL_USDT", "10000");
    std::env::set_var("PAPER_INITIAL_BTC", "0");
    std::env::set_var("PAPER_MATCHING_MODE", "PRICE_ONLY");
    std::env::set_var("PAPER_DB_PATH", "/tmp/paper_e2e_test.db");

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<DomainEvent>();
    let config = PaperConfig::load_from_env();
    let actor = PaperEngineActor::new_with_events(config, Some(event_tx), &[]);
    let snapshot = actor.snapshot_handle();
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move { actor.run(cmd_rx).await; });

    // Fiyat besle (DATA terminalden gelirmiş gibi)
    cmd_tx.send(ActorCommand::PriceUpdate { symbol: "BTCUSDT".into(), price: dec("50000") }).unwrap();

    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let _ = cmd_tx.send(ActorCommand::SubmitOrder {
        order: OrderRequest {
            symbol: "BTCUSDT".into(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: dec("0.1"),
            price: None,
            time_in_force: None,
        },
        response_tx: resp_tx,
    });

    let ack = resp_rx.await.unwrap().expect("order should fill");
    assert_eq!(ack.executed_qty, dec("0.1"));
    assert_eq!(ack.avg_price, dec("50000"));

    // Snapshot: pozisyon + bakiyeler
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let snap = snapshot.read().clone();
    assert_eq!(snap.positions.len(), 1);
    let pos = &snap.positions[0];
    assert_eq!(pos.symbol, "BTCUSDT");
    assert_eq!(pos.side, "LONG");
    assert_eq!(pos.quantity, dec("0.1"));

    // Event kalıcılığı: OrderCreated + OrderFilled üretildi
    let mut created = false;
    let mut filled = false;
    let mut count = 0;
    while let Ok(ev) = event_rx.try_recv() {
        count += 1;
        if matches!(ev, DomainEvent::OrderCreated { .. }) { created = true; }
        if matches!(ev, DomainEvent::OrderFilled { .. }) { filled = true; }
    }
    assert!(created, "OrderCreated event bekleniyor");
    assert!(filled, "OrderFilled event bekleniyor");
    assert!(count >= 2, "en az 2 event bekleniyor, {} üretildi", count);
}

#[tokio::test]
async fn test_limit_order_fills_on_price_cross() {
    std::env::set_var("PAPER_INITIAL_USDT", "10000");
    std::env::set_var("PAPER_MATCHING_MODE", "PRICE_ONLY");
    std::env::set_var("PAPER_DB_PATH", "/tmp/paper_e2e_lim_test.db");

    let config = PaperConfig::load_from_env();
    let actor = PaperEngineActor::new_with_events(config, None, &[]);
    let snapshot = actor.snapshot_handle();
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move { actor.run(cmd_rx).await; });

    // Fiyat 51000; LIMIT BUY 50000 bekler
    cmd_tx.send(ActorCommand::PriceUpdate { symbol: "BTCUSDT".into(), price: dec("51000") }).unwrap();

    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let _ = cmd_tx.send(ActorCommand::SubmitOrder {
        order: OrderRequest {
            symbol: "BTCUSDT".into(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: dec("0.5"),
            price: Some(dec("50000")),
            time_in_force: None,
        },
        response_tx: resp_tx,
    });

    // Bekleyen (PENDING) dönmeli
    let ack = resp_rx.await.unwrap().expect("order accepted");
    assert_eq!(ack.order_id, "PENDING");

    // Fiyat 50000'e düşünce dolar
    cmd_tx.send(ActorCommand::PriceUpdate { symbol: "BTCUSDT".into(), price: dec("50000") }).unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let snap = snapshot.read().clone();
    assert_eq!(snap.positions.len(), 1);
    assert_eq!(snap.positions[0].quantity, dec("0.5"));
    assert_eq!(snap.open_orders, 0);
}
