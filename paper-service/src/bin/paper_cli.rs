//! paper-cli: PAPER sisteminin REST API üzerinden çalışan komut satırı arayüzü.

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "paper-trading", version, about = "🛡️ Paper Trading CLI")]
struct Cli {
    /// API adresi (varsayılan: http://127.0.0.1:8080)
    #[arg(long, env = "PAPER_API_ADDR", default_value = "http://127.0.0.1:8080")]
    api: String,

    /// Kullanıcı adı (varsayılan: admin)
    #[arg(long, env = "PAPER_ADMIN_USER", default_value = "admin")]
    user: String,

    /// Şifre (varsayılan: changeme123)
    #[arg(long, env = "PAPER_ADMIN_PASS", default_value = "changeme123")]
    password: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Hesap bakiyesi ve risk durumu
    Status,
    /// Açık pozisyonlar
    Positions,
    /// İşlem geçmişi (son 200)
    History,
    /// Pozisyonun likidasyon fiyatı
    Liquidation { symbol: String },
    /// Emir gönder
    Order {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        side: String,
        #[arg(long)]
        order_type: String,
        #[arg(long)]
        qty: String,
        #[arg(long)]
        price: Option<String>,
        #[arg(long)]
        client_oid: Option<String>,
    },
}

struct ApiClient {
    base: String,
    token: Option<String>,
    http: reqwest::Client,
}

impl ApiClient {
    async fn login(&mut self, user: &str, password: &str) -> Result<(), String> {
        let resp: Value = self
            .http
            .post(format!("{}/api/v1/auth/login", self.base))
            .json(&json!({"username": user, "password": password}))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        self.token = resp.get("access_token").and_then(|t| t.as_str()).map(|s| s.to_string());
        if self.token.is_none() {
            return Err("login başarısız".to_string());
        }
        Ok(())
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        let token = self.token.as_ref().ok_or("login yapılmadı")?;
        let resp = self
            .http
            .get(format!("{}{}", self.base, path))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    async fn post(&self, path: &str, body: Value) -> Result<Value, String> {
        let token = self.token.as_ref().ok_or("login yapılmadı")?;
        let resp = self
            .http
            .post(format!("{}{}", self.base, path))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }
}

fn fmt_decimal(v: &Value) -> String {
    v.as_str().unwrap_or("0").to_string()
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("❌ Hata: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), String> {
    let mut client = ApiClient {
        base: cli.api,
        token: None,
        http: reqwest::Client::new(),
    };

    client.login(&cli.user, &cli.password).await.map_err(|e| format!("Giriş başarısız: {}", e))?;

    match &cli.command {
        Commands::Status => {
            let b = client.get("/api/v1/account/balance").await?;
            let h = client.get("/api/v1/system/health").await?;
            println!("========================================");
            println!("🛡️ PAPER HESAP DURUMU");
            println!("========================================");
            println!("Cash Balance : ${}", fmt_decimal(&b["cash_balance"]));
            println!("Equity       : ${}", fmt_decimal(&b["equity"]));
            println!("Realized PnL : ${}", fmt_decimal(&b["realized_pnl"]));
            println!("Risk Status  : {}", b["risk_status"].as_str().unwrap_or("?"));
            println!("Last Price   : {}", fmt_decimal(&h["last_price"]));
            println!("========================================");
            Ok(())
        }
        Commands::Positions => {
            let p = client.get("/api/v1/account/positions").await?;
            println!("AÇIK POZİSYONLAR:");
            if p["positions"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
                println!("  [None]");
            } else {
                for pos in p["positions"].as_array().unwrap() {
                    println!(
                        "  {} | {} | qty: {} @ {} ({}x) | liq: {}",
                        pos["symbol"].as_str().unwrap_or("?"),
                        pos["side"].as_str().unwrap_or("?"),
                        fmt_decimal(&pos["quantity"]),
                        fmt_decimal(&pos["avg_entry_price"]),
                        fmt_decimal(&pos["leverage"]),
                        pos["liquidation_price"].as_str().unwrap_or("n/a"),
                    );
                }
            }
            Ok(())
        }
        Commands::History => {
            let t = client.get("/api/v1/account/trade-history").await?;
            println!("İŞLEM GEÇMİŞİ:");
            if t["trades"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
                println!("  [None]");
            } else {
                for tr in t["trades"].as_array().unwrap().iter().rev().take(20) {
                    println!(
                        "  {} {} {} @ {} qty={} fee={}",
                        tr["symbol"].as_str().unwrap_or("?"),
                        tr["side"].as_str().unwrap_or("?"),
                        tr["order_id"].as_str().unwrap_or("?"),
                        fmt_decimal(&tr["price"]),
                        fmt_decimal(&tr["quantity"]),
                        fmt_decimal(&tr["fee"]),
                    );
                }
            }
            Ok(())
        }
        Commands::Liquidation { symbol } => {
            let liq = client
                .get(&format!("/api/v1/risk/liquidation-price/{}", symbol))
                .await?;
            println!("{} likidasyon fiyatı: {}", symbol, fmt_decimal(&liq["liquidation_price"]));
            Ok(())
        }
        Commands::Order { symbol, side, order_type, qty, price, client_oid } => {
            let mut body = HashMap::new();
            body.insert("client_order_id", client_oid.clone().unwrap_or_else(|| format!("cli_{}", now_ms())));
            body.insert("symbol", symbol.clone());
            body.insert("side", side.clone());
            body.insert("order_type", order_type.clone());
            body.insert("quantity", qty.clone());
            if let Some(p) = price {
                body.insert("price", p.clone());
            }
            let resp = client.post("/api/v1/order", serde_json::to_value(&body).unwrap()).await?;
            if let Some(err) = resp.get("error").and_then(|e| e.as_str()) {
                println!("❌ Emir reddedildi: {}", err);
            } else {
                println!("✅ Emir gönderildi: order_id={} avg={} qty={}",
                    resp["order_id"].as_str().unwrap_or("?"),
                    resp["avg_price"].as_str().unwrap_or("?"),
                    resp["executed_qty"].as_str().unwrap_or("?"),
                );
            }
            Ok(())
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
