//! Örnek tüketici: calc-ind servisine RSI isteği atar, sonucu ring'den okur.

use calc_ind::IndRequest;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let mut params = HashMap::new();
    params.insert("period".to_string(), 14.0);

    let req = IndRequest::new("BTCUSDT", "1h", None, None, "rsi").with_params(params);

    match calc_ind::client::request_default(&req).await {
        Ok(id) => {
            println!("İstek gönderildi → request_id={id}");
            match calc_ind::client::read_result(id, 5, 200) {
                Some(res) => {
                    println!("Sonuç okundu:");
                    println!("  sembol={} indikatör={} kline={}", res.symbol, res.indicator, res.klines.len());
                    for (name, s) in &res.series {
                        let ilk_gecerli = s.iter().find(|v| v.is_some()).copied().flatten();
                        println!("  seri={} len={} ilk_gecerli={:?}", name, s.len(), ilk_gecerli);
                    }
                }
                None => println!("Sonuç ring'de bulunamadı"),
            }
        }
        Err(e) => println!("İstek hatası: {e}"),
    }
}
