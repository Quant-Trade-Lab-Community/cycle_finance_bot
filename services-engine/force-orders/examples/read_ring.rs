//! force-orders ring tüketici örneği.
//!
//! `/dev/shm/cycle_finance_force_orders` ring'inden binary likidasyon kayıtlarını
//! okur ve ekrana basar. Servis çalışırken başka bir terminalde koşun:
//!
//! ```bash
//! cargo run -p force-orders --example read_ring -- --count 10
//! ```

use force_orders::client;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: u64 = args
        .iter()
        .position(|a| a == "--count")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    let mut cursor = 0u64;
    let mut printed = 0u64;
    loop {
        let (next, orders) = client::read(cursor, 3, 500);
        for o in orders {
            println!(
                "{}  {:<14} {:<5} {:>12.2} {:>12.4} {:>14.2}",
                force_orders::fmt_time_ms(o.event_ts),
                o.symbol,
                o.side,
                o.price,
                o.qty,
                o.notional
            );
            printed += 1;
            if printed >= count {
                std::process::exit(0);
            }
        }
        cursor = next;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
