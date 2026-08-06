use crate::memory::ring_buffer::GenerationalRingBuffer;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

pub fn start_backtester(csv_path: &str) {
    println!("========================================");
    println!("⏪ BACKTEST ENGINE TERMINAL v1.0");
    println!("Loading: {}", csv_path);
    println!("========================================");

    let gen_ring = std::sync::Arc::new(GenerationalRingBuffer::new(160_000));
    
    let file = File::open(csv_path).expect("❌ CSV dosyası bulunamadı!");
    let reader = BufReader::new(file);

    let start = Instant::now();
    let mut count = 0;

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            // Basit CSV formatı: symbol,price,quantity,timestamp
            // Örnek: BTCUSDT,64000.50,1.2,1623821034000
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let symbol = parts[0];
                let price = parts[1];
                let qty = parts[2];
                let ts = parts[3];
                
                // DATA terminalinin ürettiği WebSocket JSON formatına dönüştürüyoruz
                // Böylece STRATEGY terminali canlı mı yoksa backtest mi çalıştığını ayırt edemeyecek
                let mock_json = format!(
                    "{{\"stream\":\"{}@trade\",\"data\":{{\"s\":\"{}\",\"p\":\"{}\",\"q\":\"{}\",\"T\":{}}}}}",
                    symbol.to_lowercase(), symbol, price, qty, ts
                );
                
                gen_ring.push(mock_json.as_bytes());
                count += 1;
                
                // Rate limit (çok hızlı basarsa RingBuffer taşabilir)
                if count % 100_000 == 0 {
                    std::thread::yield_now();
                }
            }
        }
    }

    let elapsed = start.elapsed();
    println!("✅ Backtest Tamamlandı!");
    println!("Yüklenen Tick: {}", count);
    println!("Geçen Süre: {:?}", elapsed);
    println!("Hız: {:.2} Tick / saniye", (count as f64) / elapsed.as_secs_f64());
}
