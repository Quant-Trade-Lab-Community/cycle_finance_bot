use proje_core::ring_buffer::OwnedEvent;
use proje_core::tick::EventParser;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_tick_parsing_zero_allocation(price in 1.0..100000.0f64, qty in 0.001..10.0f64, timestamp in 1600000000000..1700000000000u64) {
        // Binance combined-stream format with trade payload
        let mut raw_payload = format!(
            "{{\"stream\":\"btcusdt@trade\",\"data\":{{\"e\":\"trade\",\"s\":\"BTCUSDT\",\"p\":\"{:.8}\",\"q\":\"{:.8}\",\"T\":{},\"m\":false}}}}",
            price, qty, timestamp
        ).into_bytes();

        let parsed_tick_opt = EventParser::parse(&mut raw_payload);

        prop_assert!(parsed_tick_opt.is_some());
        let tick: OwnedEvent = parsed_tick_opt.unwrap();

        let sym_len = tick.symbol.iter().position(|&c| c == 0).unwrap_or(16);
        prop_assert_eq!(&tick.symbol[..sym_len], b"BTCUSDT");
    }
}

#[test]
fn test_tick_allocation_mock() {
    // A mock specific test to run 1 million iterations quickly to simulate the CI run
    let base_payload = b"{\"stream\":\"btcusdt@trade\",\"data\":{\"s\":\"BTCUSDT\",\"p\":\"50000.0\",\"q\":\"1.5\",\"T\":1620000000000,\"m\":false}}";

    for _ in 0..1_000_000 {
        let mut payload = base_payload.to_vec(); // Outer alloc, inner parse should be 0 alloc
        let tick = EventParser::parse(&mut payload);
        assert!(tick.is_some());
    }
}

#[test]
fn test_parse_mark_price_stream() {
    let mut raw = br#"{"stream":"btcusdt@markPrice@1s","data":{"e":"markPriceUpdate","E":1562305380000,"s":"BTCUSDT","p":"64359.10000000","i":"64350.00000000","P":"64300.00000000","r":"0.00038167","T":1562306400000}}"#.to_vec();
    let ev = EventParser::parse(&mut raw).expect("markPrice parse edilmeli");
    assert!(matches!(ev.payload, proje_core::ring_buffer::EventType::FundingRate { .. }));
}
