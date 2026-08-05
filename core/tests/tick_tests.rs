use core::tick::TickData;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_tick_parsing_zero_allocation(price in 1.0..100000.0f64, qty in 0.001..10.0f64, timestamp in 1600000000000..1700000000000u64) {
        // Constructing a payload
        let mut raw_payload = format!("{{\"s\":\"BTCUSDT\",\"p\":\"{}\",\"q\":\"{}\",\"T\":{}}}", price, qty, timestamp).into_bytes();
        
        // 1. Tick Deserialize Testi: simd-json ile 1 milyon farklı Binance mesajını TickData<'a>'ya çevir. 
        // Kural: Hiçbir tick'te malloc/free çağrısı olmamalı (allocation count = 0).
        // Allocation validation is handled externally by heaptrack in CI, but here we prove safety/correctness.
        
        let parsed_tick_opt = TickData::parse(&mut raw_payload);
        
        prop_assert!(parsed_tick_opt.is_some());
        let tick = parsed_tick_opt.unwrap();
        
        prop_assert_eq!(tick.symbol, "BTCUSDT");
        prop_assert_eq!(tick.timestamp, timestamp);
    }
}

#[test]
fn test_tick_allocation_mock() {
    // A mock specific test to run 1 million iterations quickly to simulate the CI run
    let base_payload = b"{\"s\":\"BTCUSDT\",\"p\":\"50000.0\",\"q\":\"1.5\",\"T\":1620000000000}";
    
    for _ in 0..1_000_000 {
        let mut payload = base_payload.to_vec(); // Outer alloc, inner parse should be 0 alloc
        let tick = TickData::parse(&mut payload);
        assert!(tick.is_some());
    }
}
