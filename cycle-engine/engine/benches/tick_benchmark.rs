use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::prelude::*;
use transport::events::OwnedEvent;
use pipeline::tick::EventParser;
use transport::wire;

fn bench_tick_parsing(c: &mut Criterion) {
    let payload = b"{\"stream\":\"btcusdt@trade\",\"data\":{\"e\":\"trade\",\"E\":1766800000000,\"s\":\"BTCUSDT\",\"t\":123,\"p\":\"50000.0\",\"q\":\"1.5\",\"T\":1620000000000,\"m\":false}}".to_vec();

    c.bench_function("tick_parse_wcet", |b| {
        b.iter(|| {
            let mut data = payload.clone();
            let parsed = EventParser::parse(black_box(&mut data));
            black_box(parsed);
        })
    });
}

fn bench_wire_roundtrip(c: &mut Criterion) {
    let trade = OwnedEvent::new_trade("BTCUSDT", Decimal::from_str("67234.50").unwrap(),
        Decimal::from_str("0.001500").unwrap(), 1_766_800_000_000, true);
    let mut buf = [0u8; wire::MAX_FRAME_SIZE + 64];

    c.bench_function("wire_encode_trade", |b| {
        b.iter(|| {
            let len = wire::encode(black_box(&trade), &mut buf);
            black_box(len);
        })
    });

    c.bench_function("wire_decode_trade", |b| {
        let len = wire::encode(&trade, &mut buf).unwrap();
        b.iter(|| {
            let ev = wire::decode(black_box(&buf[..len]));
            black_box(ev);
        })
    });

    let mut bids = [(rust_decimal::Decimal::ZERO, rust_decimal::Decimal::ZERO); 20];
    let mut asks = [(rust_decimal::Decimal::ZERO, rust_decimal::Decimal::ZERO); 20];
    for i in 0..20 {
        bids[i] = (Decimal::new(67200 + i as i64, 0), Decimal::new(100 - i as i64, 0));
        asks[i] = (Decimal::new(67220 + i as i64, 0), Decimal::new(90 + i as i64, 0));
    }
    let depth = OwnedEvent::new_orderbook("BTCUSDT", bids, asks);

    c.bench_function("wire_encode_depth20", |b| {
        b.iter(|| {
            let len = wire::encode(black_box(&depth), &mut buf);
            black_box(len);
        })
    });

    c.bench_function("wire_decode_depth20", |b| {
        let len = wire::encode(&depth, &mut buf).unwrap();
        b.iter(|| {
            let ev = wire::decode(black_box(&buf[..len]));
            black_box(ev);
        })
    });
}

criterion_group!(benches, bench_tick_parsing, bench_wire_roundtrip);
criterion_main!(benches);
