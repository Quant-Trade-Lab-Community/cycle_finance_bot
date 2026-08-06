use crate::Kline;
use reqwest::Client;
use serde_json::Value;

pub struct BinanceClient {
    http: Client,
}

impl BinanceClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }

    /// Fetches historical Klines (OHLCV) from Binance Futures
    /// https://fapi.binance.com/fapi/v1/klines
    pub async fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: usize,
    ) -> Result<Vec<Kline>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://fapi.binance.com/fapi/v1/klines?symbol={}&interval={}&limit={}",
            symbol, interval, limit
        );

        let response = self.http.get(&url).send().await?;
        let data: Vec<Value> = response.json().await?;

        let mut klines = Vec::new();

        for row in data {
            if let Some(arr) = row.as_array() {
                if arr.len() >= 11 {
                    let kline = Kline {
                        open_time: arr[0].as_u64().unwrap_or(0),
                        open: arr[1].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        high: arr[2].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        low: arr[3].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        close: arr[4].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        volume: arr[5].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        close_time: arr[6].as_u64().unwrap_or(0),
                        quote_asset_volume: arr[7].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        trades: arr[8].as_u64().unwrap_or(0),
                        taker_buy_base_asset_volume: arr[9].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        taker_buy_quote_asset_volume: arr[10].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    };
                    klines.push(kline);
                }
            }
        }

        Ok(klines)
    }
}
