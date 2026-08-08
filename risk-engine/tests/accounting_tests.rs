//! Muhasebe değişmezleri ve PnL doğruluğu testleri.

use risk_engine::accounting::Portfolio;
use risk_engine::types::{Fill, Side};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn prices(pairs: &[(&str, &str)]) -> HashMap<String, Decimal> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), d(v)))
        .collect()
}

#[test]
fn open_long_and_unrealized_pnl() {
    let mut p = Portfolio::new(d("10000"), d("0.20"));
    p.process_fill("BTCUSDT", d("0.1"), d("50000"), d("0"));
    assert_eq!(p.positions["BTCUSDT"].quantity, d("0.1"));
    // 0.1 BTC @ 50000, mark 51000 → +100 USDT.
    let u = p.unrealized_pnl(&prices(&[("BTCUSDT", "51000")]));
    assert_eq!(u, d("100"));
    // Mark 49000 → -100 USDT.
    let u = p.unrealized_pnl(&prices(&[("BTCUSDT", "49000")]));
    assert_eq!(u, d("-100"));
}

#[test]
fn open_short_unrealized_pnl() {
    let mut p = Portfolio::new(d("10000"), d("0.20"));
    p.process_fill("ETHUSDT", d("-2"), d("3000"), d("0"));
    // Short 2 ETH @ 3000, mark 2800 → +400 USDT.
    let u = p.unrealized_pnl(&prices(&[("ETHUSDT", "2800")]));
    assert_eq!(u, d("400"));
}

#[test]
fn weighted_average_entry() {
    let mut p = Portfolio::new(d("10000"), d("0.20"));
    p.process_fill("BTCUSDT", d("1"), d("100"), d("0"));
    p.process_fill("BTCUSDT", d("1"), d("300"), d("0"));
    // (100*1 + 300*1) / 2 = 200.
    assert_eq!(p.positions["BTCUSDT"].avg_entry_price, d("200"));
}

#[test]
fn partial_close_realizes_pnl() {
    let mut p = Portfolio::new(d("10000"), d("0.20"));
    p.process_fill("BTCUSDT", d("1"), d("100"), d("0"));
    // 0.5 BTC'yi 140'tan kapat → +20 USDT.
    let realized = p.process_fill("BTCUSDT", d("-0.5"), d("140"), d("0"));
    assert_eq!(realized, d("20"));
    assert_eq!(p.realized_pnl, d("20"));
    assert_eq!(p.positions["BTCUSDT"].quantity, d("0.5"));
    assert_eq!(p.positions["BTCUSDT"].avg_entry_price, d("100"));
}

#[test]
fn full_close_removes_position() {
    let mut p = Portfolio::new(d("10000"), d("0.20"));
    p.process_fill("BTCUSDT", d("1"), d("100"), d("0"));
    let realized = p.process_fill("BTCUSDT", d("-1"), d("110"), d("0"));
    assert_eq!(realized, d("10"));
    assert!(!p.positions.contains_key("BTCUSDT"));
}

#[test]
fn flip_direction_sets_new_entry() {
    let mut p = Portfolio::new(d("10000"), d("0.20"));
    p.process_fill("BTCUSDT", d("1"), d("100"), d("0"));
    // 1.5 BTC sat → 1'i kapat (-100 → +?): long 100'den, kapanış 80 → -20 USDT realized.
    let realized = p.process_fill("BTCUSDT", d("-1.5"), d("80"), d("0"));
    assert_eq!(realized, d("-20"));
    // Net: -0.5 BTC (short), yeni giriş 80.
    let pos = &p.positions["BTCUSDT"];
    assert_eq!(pos.quantity, d("-0.5"));
    assert_eq!(pos.avg_entry_price, d("80"));
}

#[test]
fn commission_reduces_cash() {
    let mut p = Portfolio::new(d("1000"), d("0.20"));
    p.process_fill("BTCUSDT", d("0.1"), d("50000"), d("2.5"));
    assert_eq!(p.total_commission, d("2.5"));
    assert_eq!(p.cash_balance, d("997.5"));
}

#[test]
fn equity_equals_cash_plus_unrealized() {
    let mut p = Portfolio::new(d("1000"), d("0.20"));
    p.process_fill("BTCUSDT", d("0.1"), d("50000"), d("0"));
    let eq = p.get_total_equity(&prices(&[("BTCUSDT", "50500")]));
    assert_eq!(eq, d("1050"));
}

#[test]
fn drawdown_detection() {
    let mut p = Portfolio::new(d("1000"), d("0.10"));
    // 1000 → peak 1000; 1200'ye çık (peak 1200), sonra 1060'a düş → drawdown %11.67 > %10.
    let mkt = prices(&[("BTCUSDT", "60000")]);
    p.process_fill("BTCUSDT", d("0.02"), d("50000"), d("0")); // equity 1000
    p.update_peak(p.get_total_equity(&mkt)); // 1200
    let mkt2 = prices(&[("BTCUSDT", "53000")]); // equity 1060
    assert!(p.is_drawdown_exceeded(&mkt2));
}

#[test]
fn gross_and_net_exposure() {
    let mut p = Portfolio::new(d("10000"), d("0.20"));
    p.process_fill("BTCUSDT", d("1"), d("100"), d("0")); // long
    p.process_fill("ETHUSDT", d("-2"), d("50"), d("0")); // short
    let m = prices(&[("BTCUSDT", "110"), ("ETHUSDT", "45")]);
    let gross = p.gross_exposure(&m);
    let net = p.net_exposure(&m);
    assert_eq!(gross, d("110") + d("90"));
    assert_eq!(net, d("110") - d("90"));
}

#[test]
fn daily_loss_tracks_realized_today() {
    let mut p = Portfolio::new(d("1000"), d("0.20"));
    p.process_fill("BTCUSDT", d("1"), d("100"), d("0"));
    p.process_fill("BTCUSDT", d("-1"), d("90"), d("0"));
    assert_eq!(p.realized_today, d("-10"));
    assert_eq!(p.daily_loss(&HashMap::new()), d("-10"));
}

#[test]
fn fill_struct_processes_correctly() {
    let mut p = Portfolio::new(d("1000"), d("0.20"));
    let fill = Fill {
        symbol: "SOLUSDT".into(),
        side: Side::Buy,
        quantity: d("10"),
        price: d("30"),
        commission: d("0.15"),
        leverage: d("2"),
        ts_ms: 0,
    };
    let realized = p.apply_fill(&fill);
    assert_eq!(realized, Decimal::ZERO);
    assert_eq!(p.positions["SOLUSDT"].quantity, d("10"));
    assert_eq!(p.total_commission, d("0.15"));
}
