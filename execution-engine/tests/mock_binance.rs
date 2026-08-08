//! Sahte Binance REST sunucusuna karşı entegrasyon testleri.
//!
//! - `BinanceClient` emir/hesap akışı
//! - `-1021` timestamp drift → saat senkronu + yeniden deneme
//! - `ExecutionActor` üzerinden idempotent emir gönderimi
//!
//! Çalıştırma: `cargo test -p execution-engine --test mock_binance`

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use execution_engine::client::BinanceClient;
use execution_engine::config::ExecConfig;
use execution_engine::execution::actor::{Command, ExecutionActor, UserEvent};
use execution_engine::metrics::Metrics;
use execution_engine::order::{OrderPositionSide, OrderRequest, OrderSide, OrderType};
use execution_engine::risk::checks::RiskChecks;
use execution_engine::risk::kill_switch::KillSwitch;
use execution_engine::state::exchange_cache::ExchangeCache;
use execution_engine::state::snapshot::AccountSnapshot;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Sahte borsa durumu.
struct MockBinance {
    order_counter: AtomicUsize,
    /// İlk emir isteğinde -1021 (timestamp drift) döndürsün mü?
    fail_first_order_with_1021: bool,
}

fn exchange_info_body() -> Value {
    json!({
        "timezone": "UTC",
        "serverTime": 0,
        "rateLimits": [],
        "exchangeFilters": [],
        "symbols": [{
            "symbol": "BTCUSDT",
            "pair": "BTCUSDT",
            "contractType": "PERPETUAL",
            "status": "TRADING",
            "baseAsset": "BTC",
            "quoteAsset": "USDT",
            "baseAssetPrecision": 8,
            "quoteAssetPrecision": 8,
            "quantityPrecision": 3,
            "pricePrecision": 2,
            "marginTradingSupported": true,
            "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET", "LIMIT_MAKER"],
            "timeInForce": ["GTC", "IOC", "FOK", "GTX"],
            "filters": [
                {"filterType": "PRICE_FILTER", "minPrice": "0.01", "maxPrice": "1000000", "tickSize": "0.01"},
                {"filterType": "LOT_SIZE", "minQty": "0.001", "maxQty": "1000", "stepSize": "0.001"},
                {"filterType": "MIN_NOTIONAL", "notional": "100", "applyToMarket": true},
                {"filterType": "MAX_NUM_ORDERS", "limit": 200},
                {"filterType": "MAX_POSITION", "maxPosition": "1000"}
            ]
        }]
    })
}

fn account_body() -> Value {
    json!({
        "totalWalletBalance": "5000.00",
        "totalUnrealizedProfit": "0",
        "totalMarginBalance": "5000.00",
        "availableBalance": "5000.00",
        "maxWithdrawAmount": "5000.00",
        "totalInitialMargin": "0",
        "totalMaintMargin": "0",
        "totalCrossWalletBalance": "5000.00",
        "totalCrossUnPnl": "0",
        "assets": [{
            "asset": "USDT",
            "walletBalance": "5000.00",
            "unrealizedProfit": "0",
            "marginBalance": "5000.00",
            "maintMargin": "0",
            "initialMargin": "0",
            "positionInitialMargin": "0",
            "openOrderInitialMargin": "0",
            "crossWalletBalance": "5000.00",
            "crossUnPnl": "0",
            "availableBalance": "5000.00",
            "maxWithdrawAmount": "5000.00"
        }],
        "positions": [],
        "feeTier": 1,
        "canTrade": true,
        "canWithdraw": true
    })
}

async fn place_order(
    State(mock): State<Arc<MockBinance>>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    let n = mock.order_counter.fetch_add(1, Ordering::SeqCst);

    // İlk emir isteğinde timestamp drift simüle et (saat senkronu + retry testi).
    if n == 0 && mock.fail_first_order_with_1021 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": -1021,
                "msg": "Timestamp for this request is outside of the recvWindow."
            })),
        );
    }

    let cid = params.get("newClientOrderId").cloned().unwrap_or_else(|| "mock-cid".into());
    let side = params.get("side").cloned().unwrap_or_else(|| "BUY".into());
    let order_type = params.get("type").cloned().unwrap_or_else(|| "MARKET".into());
    let qty = params.get("quantity").cloned().unwrap_or_else(|| "0.01".into());
    let ps = params.get("positionSide").cloned().unwrap_or_else(|| "BOTH".into());
    let id = n as i64 + 1;

    let resp = json!({
        "orderId": id,
        "symbol": "BTCUSDT",
        "status": "FILLED",
        "clientOrderId": cid,
        "price": "0",
        "avgPrice": "50000",
        "origQty": qty,
        "executedQty": qty,
        "cumQuote": "500.0",
        "timeInForce": "GTC",
        "type": order_type,
        "reduceOnly": false,
        "closePosition": false,
        "side": side,
        "positionSide": ps,
        "stopPrice": "0",
        "workingType": "CONTRACT_PRICE",
        "priceProtect": false,
        "origType": order_type,
        "updateTime": now_ms(),
        "time": now_ms()
    });
    (StatusCode::OK, Json(resp))
}

fn build_router(mock: Arc<MockBinance>) -> Router {
    Router::new()
        .route("/fapi/v1/time", get(|| async { Json(json!({"serverTime": now_ms()})) }))
        .route("/fapi/v1/exchangeInfo", get(|| async { Json(exchange_info_body()) }))
        .route("/fapi/v1/order", axum::routing::post(place_order))
        .route("/fapi/v1/order", get(place_order))
        .route("/fapi/v1/batchOrders", axum::routing::post(place_order))
        .route("/fapi/v1/batchOrders", get(place_order))
        .route("/fapi/v3/account", get(|| async { Json(account_body()) }))
        .route("/fapi/v2/positionRisk", get(|| async { Json(json!([])) }))
        .route("/fapi/v1/openOrders", get(|| async { Json(json!([])) }))
        .route("/fapi/v1/positionSide/dual", get(|| async { Json(json!({"dualSidePosition": false})) }))
        .route("/fapi/v1/listenKey", axum::routing::post(|| async { Json(json!({"listenKey": "mock-key"})) }))
        .route("/fapi/v1/listenKey", axum::routing::put(|| async { Json(json!({})) }))
        .route("/fapi/v1/listenKey", axum::routing::delete(|| async { Json(json!({})) }))
        .with_state(mock)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

async fn start_mock(fail_first_order_with_1021: bool) -> String {
    let mock = Arc::new(MockBinance {
        order_counter: AtomicUsize::new(0),
        fail_first_order_with_1021,
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, build_router(mock)).await.unwrap();
    });
    format!("http://{addr}")
}

fn test_config(base_url: String) -> ExecConfig {
    let mut c = ExecConfig::load_from_env();
    c.base_url = base_url;
    c.api_key = "test-key".into();
    c.secret_key = "test-secret".into();
    c.mode = execution_engine::config::TradingMode::Live;
    c.dry_run = false;
    c.max_notional_usdt = Decimal::from(1_000_000);
    c.max_orders_per_min = 1000;
    c.reconcile_interval_sec = 3600;
    c
}

async fn spawn_actor(config: ExecConfig) -> (mpsc::UnboundedSender<Command>, Arc<RwLock<AccountSnapshot>>) {
    let client = BinanceClient::new(&config).unwrap();
    client.sync_server_time().await.unwrap();

    let metrics = Metrics::new();
    let kill_switch = Arc::new(KillSwitch::new(format!(
        "/tmp/test_exec_ks_{}",
        std::process::id()
    )));
    let snapshot = Arc::new(RwLock::new(AccountSnapshot::default()));
    let exchange = ExchangeCache::new(3600);
    let risk = RiskChecks::new(&config);

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<Command>();
    let (_user_tx, user_rx) = mpsc::unbounded_channel::<UserEvent>();

    let actor = ExecutionActor::new(
        client.clone(),
        exchange,
        risk,
        kill_switch,
        snapshot.clone(),
        metrics,
        config,
        cmd_rx,
        user_rx,
    );
    tokio::spawn(actor.run());
    (cmd_tx, snapshot)
}

async fn wait_ready(snapshot: &Arc<RwLock<AccountSnapshot>>) {
    for _ in 0..100 {
        if snapshot.read().ready {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("actor ilk eşitlemeyi tamamlamadı");
}

#[tokio::test]
async fn client_places_order_and_reads_account() {
    let base = start_mock(true).await;
    let client = BinanceClient::new(&test_config(base.clone())).unwrap();
    client.sync_server_time().await.unwrap();

    let info = client.exchange_info().await.unwrap();
    let sym = info.symbol("BTCUSDT").expect("BTCUSDT");
    assert_eq!(sym.status, "TRADING");

    let order = OrderRequest {
        symbol: "BTCUSDT".into(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: Decimal::from_str("0.01").unwrap(),
        position_side: OrderPositionSide::Both,
        client_order_id: Some("it-client-1".into()),
        ..Default::default()
    };
    // İlk istek -1021 alır, client saat senkronu yapıp yeniden dener → başarılı.
    let resp = client.place_order(&order).await.expect("place_order -1021 retry");
    assert_eq!(resp.status, "FILLED");
    assert_eq!(resp.client_order_id, "it-client-1");
    assert_eq!(resp.avg_price.as_deref(), Some("50000"));

    let acc = client.account_info().await.unwrap();
    assert_eq!(acc.total_wallet_balance, Decimal::from(5000));
    assert_eq!(acc.assets[0].asset, "USDT");
    assert!(acc.can_trade);
}

#[tokio::test]
async fn engine_actor_submits_order_with_mock() {
    let base = start_mock(false).await;
    let (cmd_tx, snapshot) = spawn_actor(test_config(base)).await;
    wait_ready(&snapshot).await;

    let (tx, rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(Command::SubmitOrder {
            order: OrderRequest {
                symbol: "BTCUSDT".into(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                quantity: Decimal::from_str("0.01").unwrap(),
                position_side: OrderPositionSide::Both,
                client_order_id: Some("it-actor-1".into()),
                ..Default::default()
            },
            tx,
        })
        .unwrap();

    let ack = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .expect("ack timeout")
        .unwrap()
        .unwrap();
    assert_eq!(ack.status, "FILLED");
    assert_eq!(ack.client_order_id, "it-actor-1");
    assert_eq!(ack.avg_price, Decimal::from_str("50000").unwrap());
}

#[tokio::test]
async fn idempotency_blocks_duplicate_client_order_id() {
    let base = start_mock(false).await;
    let (cmd_tx, snapshot) = spawn_actor(test_config(base)).await;
    wait_ready(&snapshot).await;

    async fn place(cmd_tx: &mpsc::UnboundedSender<Command>, cid: &str) -> String {
        let (tx, rx) = tokio::sync::oneshot::channel();
        cmd_tx
            .send(Command::SubmitOrder {
                order: OrderRequest {
                    symbol: "BTCUSDT".into(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    quantity: Decimal::from_str("0.01").unwrap(),
                    position_side: OrderPositionSide::Both,
                    client_order_id: Some(cid.to_string()),
                    ..Default::default()
                },
                tx,
            })
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .expect("timeout")
            .unwrap()
            .unwrap()
            .order_id
    }

    let first = place(&cmd_tx, "dup-1").await;
    let second = place(&cmd_tx, "dup-1").await;
    assert_eq!(first, second, "aynı clientOrderId aynı emri döndürmeli");
}
