use criterion::{black_box, criterion_group, criterion_main, Criterion};

// A mock struct and parser to simulate the zero-allocation parser for benchmark.
// In reality, this would import `core::tick::TickData`.
#[derive(Debug)]
pub struct TickData<'a> {
    pub symbol: &'a str,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

impl<'a> TickData<'a> {
    #[inline(always)]
    pub fn parse(bytes: &'a mut [u8]) -> Option<Self> {
        let value = simd_json::to_borrowed_value(bytes).ok()?;
        let symbol = value.get("s")?.as_str()?;
        let price = value.get("p")?.as_str()?.parse().ok()?;
        let quantity = value.get("q")?.as_str()?.parse().ok()?;
        let timestamp = value.get("T")?.as_u64()?;
        
        Some(Self {
            symbol,
            price,
            quantity,
            timestamp,
        })
    }
}

fn bench_tick_parsing(c: &mut Criterion) {
    let mut payload = b"{\"s\":\"BTCUSDT\",\"p\":\"50000.0\",\"q\":\"1.5\",\"T\":1620000000000}".to_vec();

    c.bench_function("tick_parse_wcet", |b| {
        b.iter(|| {
            // black_box prevents the compiler from optimizing away the function
            let mut data = payload.clone();
            let parsed = TickData::parse(black_box(&mut data));
            black_box(parsed);
        })
    });
}

criterion_group!(benches, bench_tick_parsing);
criterion_main!(benches);
