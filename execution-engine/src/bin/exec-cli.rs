//! `exec-cli` — execution servisi yönetim aracı.
//!
//! Doğrudan Binance REST'e bağlanır (REST API'den bağımsız, acil durum için).
//! Okuma/yazma komutları kimlik gerektirir; salt-okunur pazar komutları da
//! kimlikle çalışır (varsayılan güvenlik).

use clap::{Parser, Subcommand};
use execution_engine::client::BinanceClient;
use execution_engine::config::ExecConfig;
use execution_engine::error::{ExecError, Result};
use execution_engine::order::{OrderPositionSide, OrderRequest, TimeInForce};
use execution_engine::types::account::MarginType;
use rust_decimal::Decimal;

#[derive(Parser, Debug)]
#[command(name = "exec-cli", about = "Binance Futures execution yönetim CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Sunucu saatini yazdır.
    ServerTime,
    /// Hesap özeti (bakiye + pozisyon + açık emir).
    Account,
    /// Varlık bakiyeleri.
    Balance,
    /// Pozisyonlar [sembol].
    Positions { symbol: Option<String> },
    /// Emir gönder.
    Order {
        symbol: String,
        #[arg(value_parser = ["BUY", "SELL"])]
        side: String,
        #[arg(value_parser = ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET", "TRAILING_STOP_MARKET", "LIMIT_MAKER"])]
        order_type: String,
        quantity: Option<Decimal>,
        /// MARKET emirlerde USDT bazlı büyüklük (quantity yerine quoteOrderQty).
        #[arg(long)]
        usdt: Option<Decimal>,
        #[arg(long)]
        price: Option<Decimal>,
        #[arg(long)]
        stop_price: Option<Decimal>,
        #[arg(long, value_parser = ["GTC", "IOC", "FOK", "GTX"])]
        tif: Option<String>,
        #[arg(long, value_parser = ["BOTH", "LONG", "SHORT"])]
        position_side: Option<String>,
        #[arg(long)]
        reduce_only: bool,
        #[arg(long)]
        close_position: bool,
        #[arg(long)]
        client_order_id: Option<String>,
    },
    /// Açık emirleri listele.
    Orders { symbol: Option<String> },
    /// Emir sorgula.
    Query { symbol: String, #[arg(long)] order_id: Option<i64>, #[arg(long)] client_order_id: Option<String> },
    /// Emir iptal et.
    Cancel { symbol: String, #[arg(long)] order_id: Option<i64>, #[arg(long)] client_order_id: Option<String> },
    /// Sembolün tüm açık emirlerini iptal et.
    CancelAll { symbol: String },
    /// Kaldıraç ayarla.
    Leverage { symbol: String, value: u32 },
    /// Marjin tipi ayarla (ISOLATED/CROSSED).
    MarginType { symbol: String, #[arg(value_parser = ["ISOLATED", "CROSSED"])] value: String },
    /// İzole marj ekle/çek (--remove ile çeker).
    Margin { symbol: String, amount: Decimal, #[arg(long)] remove: bool },
    /// Hedge modu aç/kapat.
    Hedge { enabled: bool },
    /// Multi-assets modu aç/kapat.
    MultiAssets { enabled: bool },
    /// Funding oranı.
    Funding { symbol: String },
    /// Gelir geçmişi (FUNDING_FEE filtresi --type ile).
    Income { symbol: Option<String>, #[arg(long, default_value = "FUNDING_FEE")] r#type: String },
    /// Sembol kuralları.
    ExchangeInfo { symbol: String },
    /// Force orders (likidasyon/ADL).
    ForceOrders { symbol: Option<String> },
    /// listenKey üret/yenile/sil.
    ListenKey { action: String },
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| ExecError::Other(format!("geçersiz değer '{s}': {e}")))
}

fn client() -> Result<std::sync::Arc<BinanceClient>> {
    let config = ExecConfig::load_from_env();
    BinanceClient::new(&config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let args = Cli::parse();

    // Kimlik gerektirmeyen işlemler için config istemcisi.
    let c = client()?;
    c.sync_server_time().await?;

    match args.cmd {
        Command::ServerTime => {
            let t = c.server_time().await?;
            println!("serverTime: {t}");
        }
        Command::Account => {
            let acc = c.account_info().await?;
            println!("── Hesap ──────────────────────────────");
            println!("Cüzdan      : {} USDT", acc.total_wallet_balance);
            println!("Kullanılabilir: {} USDT", acc.available_balance);
            println!("Gerçekleşmemiş PnL: {} USDT", acc.total_unrealized_profit);
            println!("Marjin      : {} USDT", acc.total_margin_balance);
            println!("canTrade    : {}", acc.can_trade);
            let positions = c.position_risk(None).await?;
            println!("── Pozisyonlar ({}) ───────────────────", positions.iter().filter(|p| p.is_open()).count());
            for p in positions.iter().filter(|p| p.is_open()) {
                println!(
                    "  {} {} {} @ entry {} lev {} {} PnL {}",
                    p.position_side, p.symbol, p.position_amt, p.entry_price, p.leverage, p.margin_type, p.un_realized_profit
                );
            }
            let orders = c.query_open_orders(None).await?;
            println!("── Açık Emirler ({}) ──────────────────", orders.len());
            for o in orders {
                println!("  #{} {} {} {} {} {}", o.order_id, o.symbol, o.side.unwrap_or_default(), o.order_type.unwrap_or_default(), o.price.unwrap_or_default(), o.status);
            }
        }
        Command::Balance => {
            for b in c.balance().await? {
                if b.wallet_balance != Decimal::ZERO || b.available_balance != Decimal::ZERO {
                    println!("{}: wallet={} available={} unrealized={}", b.asset, b.wallet_balance, b.available_balance, b.unrealized_profit);
                }
            }
        }
        Command::Positions { symbol } => {
            let positions = c.position_risk(symbol.as_deref()).await?;
            for p in positions {
                println!(
                    "{:<5} {:<12} amt={} entry={} mark={} lev={} margin={} liq={} PnL={}",
                    p.position_side, p.symbol, p.position_amt, p.entry_price, p.mark_price, p.leverage, p.margin_type, p.liquidation_price, p.un_realized_profit
                );
            }
        }
        Command::Order {
            symbol,
            side,
            order_type,
            quantity,
            usdt,
            price,
            stop_price,
            tif,
            position_side,
            reduce_only,
            close_position,
            client_order_id,
        } => {
            if quantity.is_none() && usdt.is_none() {
                return Err(ExecError::Other("quantity veya --usdt gerekli".into()));
            }
            if quantity.is_some() && usdt.is_some() {
                return Err(ExecError::Other("quantity ve --usdt birlikte verilemez".into()));
            }
            let qty = quantity.unwrap_or_default();
            let order = OrderRequest {
                symbol: symbol.to_uppercase(),
                side: parse_enum(&side)?,
                order_type: parse_enum(&order_type)?,
                quantity: qty,
                quote_order_qty: usdt,
                price,
                stop_price,
                time_in_force: tif.as_deref().map(parse_enum::<TimeInForce>).transpose()?,
                position_side: position_side.as_deref().map(parse_enum::<OrderPositionSide>).transpose()?.unwrap_or(OrderPositionSide::Both),
                reduce_only: Some(reduce_only),
                close_position: Some(close_position),
                client_order_id,
                ..Default::default()
            };
            println!("Emir gönderiliyor: {} {} {} qty={} usdt={:?} @ {:?}", symbol, side, order_type, qty, usdt, price);
            let resp = c.place_order(&order).await?;
            println!("OK: orderId={} status={} cid={}", resp.order_id, resp.status, resp.client_order_id);
        }
        Command::Orders { symbol } => {
            let orders = c.query_open_orders(symbol.as_deref()).await?;
            for o in orders {
                println!(
                    "#{} {} {} {} price={} executed={}/{} status={}",
                    o.order_id,
                    o.symbol,
                    o.side.unwrap_or_default(),
                    o.order_type.unwrap_or_default(),
                    o.price.unwrap_or_default(),
                    o.executed_qty.unwrap_or_default(),
                    o.orig_qty.unwrap_or_default(),
                    o.status
                );
            }
        }
        Command::Query { symbol, order_id, client_order_id } => {
            let o = c.query_order(&symbol, order_id, client_order_id.as_deref()).await?;
            println!("{:?}", o);
        }
        Command::Cancel { symbol, order_id, client_order_id } => {
            let o = c.cancel_order(&symbol, order_id, client_order_id.as_deref()).await?;
            println!("İptal: #{} {} {}", o.order_id, o.symbol, o.status);
        }
        Command::CancelAll { symbol } => {
            let n = c.cancel_all_open(&symbol).await?.len();
            println!("{symbol}: {n} emir iptal edildi");
        }
        Command::Leverage { symbol, value } => {
            let v = c.set_leverage(&symbol, value).await?;
            println!("{} leverage → {}x ({})", symbol, value, v.get("leverage").and_then(|x| x.as_str()).unwrap_or(""));
        }
        Command::MarginType { symbol, value } => {
            let mt = if value == "ISOLATED" { MarginType::Isolated } else { MarginType::Crossed };
            let _ = c.set_margin_type(&symbol, mt).await?;
            println!("{} margin → {}", symbol, value);
        }
        Command::Margin { symbol, amount, remove } => {
            let direction = if remove { 2 } else { 1 };
            let _ = c.adjust_position_margin(&symbol, amount, direction).await?;
            println!("{} izole marj {} {} USDT", symbol, if remove { "-" } else { "+" }, amount);
        }
        Command::Hedge { enabled } => {
            let _ = c.set_position_mode(enabled).await?;
            println!("position mode → {}", if enabled { "HEDGE" } else { "ONE_WAY" });
        }
        Command::MultiAssets { enabled } => {
            let _ = c.set_multi_assets(enabled).await?;
            println!("multi-assets → {}", if enabled { "AÇIK" } else { "KAPALI" });
        }
        Command::Funding { symbol } => {
            for f in c.funding_rate(&symbol, Some(5)).await? {
                println!("fundingTime={} rate={}", f.get("fundingTime").and_then(|x| x.as_u64()).unwrap_or(0), f.get("fundingRate").and_then(|x| x.as_str()).unwrap_or(""));
            }
        }
        Command::Income { symbol, r#type } => {
            let rows = c.income(symbol.as_deref(), Some(&r#type), None, None, Some(20)).await?;
            for i in rows {
                println!("{} {} {} {} {}", i.time, i.asset, i.income, i.income_type, i.symbol);
            }
        }
        Command::ExchangeInfo { symbol } => {
            let info = c.exchange_info().await?;
            match info.symbol(&symbol.to_uppercase()) {
                Some(s) => {
                    println!("symbol={} status={} contract={}", s.symbol, s.status, s.contract_type);
                    println!("qty_precision={} price_precision={}", s.quantity_precision, s.price_precision);
                    for f in &s.filters {
                        println!("  {:?}", f);
                    }
                }
                None => println!("{symbol} bulunamadı"),
            }
        }
        Command::ForceOrders { symbol } => {
            for f in c.force_orders(symbol.as_deref()).await? {
                println!("{}", serde_json::to_string_pretty(&f).unwrap_or_default());
            }
        }
        Command::ListenKey { action } => {
            match action.to_uppercase().as_str() {
                "CREATE" => println!("listenKey: {}", c.create_listen_key().await?),
                "REFRESH" | "KEEPALIVE" | "PING" => {
                    let key = c.create_listen_key().await?;
                    let _ = c.delete_listen_key(&key).await;
                    let key2 = c.create_listen_key().await?;
                    c.refresh_listen_key(&key2).await?;
                    println!("yenilendi: {key2}");
                }
                "DELETE" => println!("lütfen geçerli bir listenKey verin"),
                other => println!("bilinmeyen aksiyon: {other} (CREATE/REFRESH/DELETE)"),
            }
        }
    }
    Ok(())
}
