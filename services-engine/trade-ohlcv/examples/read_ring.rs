//! Örnek tüketici: trade-ohlcv servisinin yayınladığı 1s OHLCV mumlarını
//! `/cycle_finance_trade_ohlcv` ring'inden okur.

use trade_ohlcv::client;

fn main() {
    println!("Son kapanmış 1s OHLCV mumları ({}) ring'den okunuyor...", trade_ohlcv::RING_NAME);
    let candles = client::read_latest(10);
    if candles.is_empty() {
        println!("Henüz mum yok — trade-ohlcv servisinin çalıştığından emin olun.");
        return;
    }
    for c in &candles {
        println!(
            "[{}] {}  O={}  H={}  L={}  C={}  V={}  TB={}  n={}  closed={}",
            c.open_time,
            c.symbol,
            c.open,
            c.high,
            c.low,
            c.close,
            c.volume,
            c.taker_buy_volume,
            c.trades,
            c.closed
        );
    }
    println!("toplam={}", candles.len());
}
