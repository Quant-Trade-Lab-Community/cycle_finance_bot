//! Bağlam toplayıcı — mevcut ring'lerden ve REST servislerinden sembol başına
//! birleşik `MarketContext` üretir.
//!
//! Kaynaklar:
//!   - fiyat    : price-feed :3004  `GET /api/lastprice/{symbol}`
//!   - indik.   : calc-ind   :3007  `POST /api/calc` + `/cycle_finance_calc` ring okuma
//!   - yapı     : detect-ms  :3002  `GET /api/ms?symbol=&interval=&limit=`
//!   - hesap    : paper      :8080  (JWT) `GET /api/v1/account/{balance,positions}`
//!   - haber    : `news_feed_url` (opsiyonel)

use crate::config::AiConfig;
use crate::{AccountSnapshot, IndicatorSnapshot, MarketContext, PositionSummary, PriceSnapshot, StructureSnapshot, now_ms};
use serde::Deserialize;
use std::collections::HashMap;

const INDICATORS: &[&str] = &["rsi", "macd", "bbands", "vwap", "atr"];

pub struct ContextBuilder {
    client: reqwest::Client,
    price_feed_url: String,
    detect_ms_url: String,
    calc_ind_url: String,
    news_feed_url: String,
    indicator_interval: String,
    structure_interval: String,
    structure_limit: u32,
    paper_url: String,
    paper_user: String,
    paper_pass: String,
}

impl ContextBuilder {
    pub fn new(cfg: &AiConfig) -> Self {
        let paper_url = std::env::var("PAPER_API_ADDR").unwrap_or_else(|_| cfg.execution.paper_url.clone());
        Self {
            client: reqwest::Client::new(),
            price_feed_url: cfg.context.price_feed_url.clone(),
            detect_ms_url: cfg.context.detect_ms_url.clone(),
            calc_ind_url: cfg.context.calc_ind_url.clone(),
            news_feed_url: cfg.context.news_feed_url.clone(),
            indicator_interval: cfg.context.indicator_interval.clone(),
            structure_interval: cfg.context.structure_interval.clone(),
            structure_limit: cfg.context.structure_limit,
            paper_user: std::env::var("PAPER_ADMIN_USER").unwrap_or_else(|_| cfg.execution.paper_admin_user.clone()),
            paper_pass: std::env::var("PAPER_ADMIN_PASS").unwrap_or_else(|_| cfg.execution.paper_admin_pass.clone()),
            paper_url,
        }
    }

    pub async fn build(&self, symbol: &str, all_symbols: &[String]) -> MarketContext {
        let price = self.fetch_price(symbol).await;
        let (indicators, structure) = tokio::join!(
            self.fetch_indicators(symbol),
            self.fetch_structure(symbol),
        );
        let account = self.fetch_account(all_symbols).await;
        let recent_news = self.fetch_news().await;

        MarketContext {
            generated_at_ms: now_ms(),
            price,
            indicators,
            structure,
            account,
            recent_news,
        }
    }

    // ── Fiyat ──────────────────────────────────────────────────────
    async fn fetch_price(&self, symbol: &str) -> PriceSnapshot {
        let url = format!("{}/api/lastprice/{}", self.price_feed_url, symbol);
        let resp = self.client.get(&url).send().await;
        let v: serde_json::Value = match resp {
            Ok(r) => match r.json().await {
                Ok(v) => v,
                Err(_) => return PriceSnapshot::default(),
            },
            Err(_) => return PriceSnapshot::default(),
        };
        let price = &v["price"];
        PriceSnapshot {
            symbol: v["symbol"].as_str().unwrap_or(symbol).to_string(),
            last: price["last"].as_f64().unwrap_or(0.0),
            mark: price["mark"].as_f64().unwrap_or(0.0),
            bid: price["bid"].as_f64().unwrap_or(0.0),
            ask: price["ask"].as_f64().unwrap_or(0.0),
            ts: price["ts"].as_u64().unwrap_or(0),
        }
    }

    // ── İndikatörler (calc-ind + ring) ─────────────────────────────
    async fn fetch_indicators(&self, symbol: &str) -> IndicatorSnapshot {
        let mut out = IndicatorSnapshot { symbol: symbol.to_string(), ..Default::default() };

        let mut handles = Vec::new();
        for ind in INDICATORS {
            let symbol = symbol.to_string();
            let interval = self.indicator_interval.clone();
            let addr = self.calc_ind_url.clone();
            handles.push(tokio::spawn(async move {
                let req = calc_ind::IndRequest::new(&symbol, &interval, None, None, ind);
                let outcome = calc_ind::client::request(&addr, &req).await.map_err(|e| e.to_string());
                match outcome {
                    Ok(id) => {
                        // read_result her slot'ta 1MB CalcSlot kopyalar; tokio blocking
                        // pool'unun 2MB stack'i taşmaz — geniş stack'li ayrı thread kullan.
                        let handle = std::thread::Builder::new()
                            .name("calc-ring-read".into())
                            .stack_size(64 * 1024 * 1024)
                            .spawn(move || calc_ind::client::read_result(id, 2, 50))
                            .map(|h| h.join().ok().flatten())
                            .unwrap_or(None);
                        (ind.to_string(), handle)
                    }
                    Err(_) => (ind.to_string(), None),
                }
            }));
        }

        for h in handles {
            if let Ok((name, res)) = h.await {
                if let Some(res) = res {
                    fill_indicators(&mut out, &name, &res.series);
                }
            }
        }
        out
    }

    // ── Piyasa yapısı (detect-ms) ──────────────────────────────────
    async fn fetch_structure(&self, symbol: &str) -> StructureSnapshot {
        let url = format!(
            "{}/api/ms?symbol={}&interval={}&limit={}",
            self.detect_ms_url, symbol, self.structure_interval, self.structure_limit
        );
        let mut out = StructureSnapshot { symbol: symbol.to_string(), ..Default::default() };
        let Ok(resp) = self.client.get(&url).send().await else {
            return out;
        };
        let Ok(v) = resp.json::<serde_json::Value>().await else {
            return out;
        };
        let ms: Option<MsmpResponse> = serde_json::from_value(v).ok();
        let Some(ms) = ms else { return out };

        out.ats = ms.ats;
        out.hurst = ms.hurst;
        out.r_squared = ms.r_squared;
        out.trend_label = ms.trend_label;
        out.confluence_index = ms.confluence_index;
        out.vwap = ms.vwap;
        out.poc = ms.poc;
        out.bsl_ssl_ratio = ms.bsl_ssl_ratio;
        out.atr = ms.atr;
        out.current_price = ms.current_price;
        out.levels = ms
            .levels
            .iter()
            .take(8)
            .map(|l| format!("{}@{} pri:{}", l.level_type, l.price, l.priority_score))
            .collect();
        out
    }

    // ── Hesap durumu (paper JWT) ───────────────────────────────────
    async fn fetch_account(&self, symbols: &[String]) -> Option<AccountSnapshot> {
        let token = self.paper_token().await?;
        let auth = format!("Bearer {}", token);

        let bal_url = format!("{}/api/v1/account/balance", self.paper_url);
        let pos_url = format!("{}/api/v1/account/positions", self.paper_url);

        let bal_fut = self.client.get(&bal_url).header("Authorization", &auth).send();
        let pos_fut = self.client.get(&pos_url).header("Authorization", &auth).send();

        let (bal, pos) = tokio::join!(bal_fut, pos_fut);
        let bal_v: serde_json::Value = bal.ok()?.json().await.ok()?;
        let pos_v: serde_json::Value = pos.ok()?.json().await.ok()?;

        let mut positions = Vec::new();
        if let Some(arr) = pos_v["positions"].as_array() {
            for p in arr {
                let symbol = p["symbol"].as_str().unwrap_or("").to_string();
                if !symbols.is_empty() && !symbols.contains(&symbol) {
                    continue;
                }
                let qty = p["quantity"].as_f64().or_else(|| p["positionAmt"].as_f64()).unwrap_or(0.0);
                if qty.abs() < 1e-9 {
                    continue;
                }
                positions.push(PositionSummary {
                    symbol,
                    side: p["side"].as_str().or_else(|| p["positionSide"].as_str()).unwrap_or("").to_string(),
                    quantity: qty,
                    entry_price: p["entry_price"].as_f64().or_else(|| p["entryPrice"].as_f64()).unwrap_or(0.0),
                    unrealized_pnl: p["unrealized_pnl"].as_f64().unwrap_or(0.0),
                });
            }
        }

        Some(AccountSnapshot {
            equity: bal_v["equity"].as_str().and_then(|s| s.parse().ok()),
            cash_balance: bal_v["cash_balance"].as_str().and_then(|s| s.parse().ok()),
            positions,
        })
    }

    async fn paper_token(&self) -> Option<String> {
        let url = format!("{}/api/v1/auth/login", self.paper_url);
        let body = serde_json::json!({
            "username": self.paper_user,
            "password": self.paper_pass,
        });
        let resp = self.client.post(&url).json(&body).send().await.ok()?;
        let v: serde_json::Value = resp.json().await.ok()?;
        v["access_token"].as_str().map(str::to_string)
    }

    // ── Haber (opsiyonel dış kaynak) ───────────────────────────────
    async fn fetch_news(&self) -> Vec<String> {
        if self.news_feed_url.trim().is_empty() {
            return Vec::new();
        }
        let Ok(resp) = self.client.get(&self.news_feed_url).send().await else {
            return Vec::new();
        };
        let Ok(v) = resp.json::<serde_json::Value>().await else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if let Some(arr) = v.as_array() {
            for item in arr {
                if let Some(t) = item["title"].as_str().or_else(|| item.as_str()) {
                    out.push(t.to_string());
                }
            }
        } else if let Some(arr) = v["articles"].as_array() {
            for item in arr {
                if let Some(t) = item["title"].as_str() {
                    out.push(t.to_string());
                }
            }
        }
        out
    }
}

fn fill_indicators(out: &mut IndicatorSnapshot, name: &str, series: &HashMap<String, Vec<Option<f64>>>) {
    let last = |key: &str| -> Option<f64> {
        series
            .get(key)
            .and_then(|v| v.iter().rev().find_map(|x| *x))
    };
    match name {
        "rsi" => {
            if let Some(v) = last("rsi") {
                out.rsi = Some(v);
            }
        }
        "macd" => {
            out.macd = last("macd").or_else(|| last("value"));
            out.macd_signal = last("signal");
        }
        "bbands" => {
            out.bbands_upper = last("upper");
            out.bbands_middle = last("middle");
            out.bbands_lower = last("lower");
            out.sma20 = out.bbands_middle;
        }
        "vwap" => out.vwap = last("vwap"),
        "atr" => out.atr = last("atr"),
        _ => {}
    }
}

/// detect-ms raporunun f64 sürümü (Decimal → f64 çözülür).
#[derive(Debug, Default, Deserialize)]
struct MsmpResponse {
    ats: Option<f64>,
    hurst: Option<f64>,
    r_squared: Option<f64>,
    trend_label: Option<String>,
    confluence_index: Option<f64>,
    vwap: Option<f64>,
    poc: Option<f64>,
    bsl_ssl_ratio: Option<f64>,
    atr: Option<f64>,
    levels: Vec<MsmpLevel>,
    current_price: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
struct MsmpLevel {
    level_type: String,
    price: f64,
    priority_score: f64,
}
