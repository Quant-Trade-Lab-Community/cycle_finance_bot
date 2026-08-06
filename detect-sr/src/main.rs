pub mod algorithms;

use clap::Parser;
use ohlcv_engine::client::BinanceClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "HEIUSDT")]
    symbol: String,

    #[arg(short, long, default_value = "1h")]
    interval: String,

    #[arg(short, long, default_value_t = 500)]
    limit: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    println!("==================================================");
    println!("🛡️ DESTEK/DİRENÇ TESPİT MOTORU (HFT QUANT)");
    println!("Sembol: {} | Aralık: {} | Veri: Son {} Mum", args.symbol, args.interval, args.limit);
    println!("==================================================\n");

    let client = BinanceClient::new();

    match client.fetch_klines(&args.symbol, &args.interval, args.limit).await {
        Ok(klines) => {
            println!("✅ Veri Başarıyla Çekildi ({} Adet). Analiz Başlıyor...\n", klines.len());
            
            let current_price = klines.last().map(|k| k.close).unwrap_or(0.0);
            println!("💵 GÜNCEL FİYAT: {:.4}\n", current_price);

            // 1. Fractal / Swing
            let swing_levels = algorithms::swing_extrema(&klines, 5);
            print_levels("1. YEREL TEPELER/DİPLER (FRACTAL & SWING)", &swing_levels, current_price);

            // 2. K-Means (1D)
            let kmeans_levels = algorithms::kmeans_1d(&klines, 5);
            print_levels("2. K-MEANS KÜMELEME (YAPAY ZEKA)", &kmeans_levels, current_price);

            // 3. Volume Profile (POC)
            let vp_levels = algorithms::volume_profile(&klines, 50);
            print_levels("3. HACİM DÜĞÜMLERİ (VOLUME PROFILE - POC)", &vp_levels, current_price);

            // 4. Kernel Density Estimation (KDE)
            let kde_levels = algorithms::kde_peaks(&klines);
            print_levels("4. KERNEL YOĞUNLUK TAHMİNİ (KDE)", &kde_levels, current_price);

        },
        Err(e) => {
            eprintln!("❌ Veri çekilirken hata oluştu: {}", e);
        }
    }
}

fn print_levels(title: &str, levels: &[f64], current_price: f64) {
    println!("📌 {}", title);
    if levels.is_empty() {
        println!("  - Bulunamadı.");
        println!();
        return;
    }

    let mut resistances = Vec::new();
    let mut supports = Vec::new();

    for &lvl in levels {
        if lvl > current_price {
            resistances.push(lvl);
        } else {
            supports.push(lvl);
        }
    }

    // Dirençleri büyükten küçüğe yaz (Fiyata doğru)
    resistances.sort_by(|a, b| b.partial_cmp(a).unwrap());
    for r in resistances {
        let dist = ((r - current_price) / current_price) * 100.0;
        println!("  🔴 DİRENÇ: {:.4} (Fiyata Uzaklık: +{:.2}%)", r, dist);
    }

    println!("  ==============================");
    
    // Destekleri büyükten küçüğe yaz (Fiyattan aşağı doğru)
    supports.sort_by(|a, b| b.partial_cmp(a).unwrap());
    for s in supports {
        let dist = ((current_price - s) / current_price) * 100.0;
        println!("  🟢 DESTEK: {:.4} (Fiyata Uzaklık: -{:.2}%)", s, dist);
    }
    
    println!();
}
