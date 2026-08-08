//! Exec Console — Execution Engine (:3010) için elle komut konsolu.
//!
//! executiond REST API'sine JWT ile bağlanır; kullanıcı komutları interaktif
//! girer. Komutlar doğrudan Binance'e gitmez, executiond preflight/risk
//! katmanından geçer.
//!
//! Ortam değişkenleri:
//!   EXEC_API_ADDR      (varsayılan 127.0.0.1:3010)
//!   EXEC_ADMIN_USER    (varsayılan admin)
//!   EXEC_ADMIN_PASS    (varsayılan changeme123)

use reqwest::blocking::Client;
use reqwest::StatusCode;
use rustyline::DefaultEditor;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

struct Console {
    client: Client,
    base: String,
    user: String,
    pass: String,
    token: String,
}

impl Console {
    fn new() -> Self {
        let base = std::env::var("EXEC_API_ADDR").unwrap_or_else(|_| "http://127.0.0.1:3010".into());
        let user = std::env::var("EXEC_ADMIN_USER").unwrap_or_else(|_| "admin".into());
        let pass = std::env::var("EXEC_ADMIN_PASS").unwrap_or_else(|_| "changeme123".into());
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .expect("http client");
        let mut c = Self { client, base, user, pass, token: String::new() };
        match c.login() {
            Ok(()) => println!("✅ executiond'ye bağlandı: {}", c.base),
            Err(e) => eprintln!("⚠️  Login başarısız: {e}\n   executiond çalışıyor mu? (exec-dry/exec-live)"),
        }
        c
    }

    fn login(&mut self) -> Result<(), String> {
        let url = format!("{}/api/v1/auth/login", self.base);
        let resp = self
            .client
            .post(&url)
            .json(&json!({ "username": self.user, "password": self.pass }))
            .send()
            .map_err(|e| format!("istek hatası: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("login {}", resp.status()));
        }
        let v: Value = resp.json().map_err(|e| format!("yanıt hatası: {e}"))?;
        self.token = v["access_token"]
            .as_str()
            .ok_or_else(|| "access_token yok".to_string())?
            .to_string();
        Ok(())
    }

    /// İmzalı istek; 401 alırsa yeniden login olup bir kez tekrar dener.
    fn call(
        &mut self,
        method: &str,
        path: &str,
        query: Option<&str>,
        body: Option<Value>,
    ) -> Result<Value, String> {
        for attempt in 0..2 {
            let url = match query {
                Some(q) if !q.is_empty() => format!("{}{}?{}", self.base, path, q),
                _ => format!("{}{}", self.base, path),
            };
            let auth = format!("Bearer {}", self.token);
            let send = || -> Result<reqwest::blocking::Response, reqwest::Error> {
                let c = &self.client;
                match method {
                    "GET" => c.get(&url).header("Authorization", &auth).send(),
                    "POST" => {
                        let mut r = c.post(&url).header("Authorization", &auth);
                        if let Some(b) = &body {
                            r = r.json(b);
                        }
                        r.send()
                    }
                    "PUT" => {
                        let mut r = c.put(&url).header("Authorization", &auth);
                        if let Some(b) = &body {
                            r = r.json(b);
                        }
                        r.send()
                    }
                    "DELETE" => c.delete(&url).header("Authorization", &auth).send(),
                    _ => unreachable!(),
                }
            };

            let resp = send().map_err(|e| format!("istek hatası: {e}"))?;
            let status = resp.status();
            let text = resp.text().unwrap_or_default();

            if status == StatusCode::UNAUTHORIZED && attempt == 0 {
                let _ = self.login();
                continue;
            }
            if !status.is_success() {
                return Err(format!("http {}: {}", status, short(&text)));
            }
            if text.trim().is_empty() {
                return Ok(Value::Null);
            }
            return serde_json::from_str(&text).map_err(|e| format!("yanıt ayrıştırılamadı: {e}"));
        }
        Err("yetkilendirme başarısız (401)".into())
    }
}

fn short(s: &str) -> String {
    s.chars().take(300).collect()
}

fn now_cid() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    format!("console_{}", ts)
}

// ── Çıktı yardımcıları ─────────────────────────────────────────────

fn fmt_account(v: &Value) {
    let a = &v["account"];
    let fields = [
        ("total_wallet_balance", "Toplam cüzdan"),
        ("total_unrealized_profit", "Gerçekleşmemiş kazanç"),
        ("total_margin_balance", "Toplam marj"),
        ("available_balance", "Kullanılabilir"),
        ("max_withdraw_amount", "Maks çekilebilir"),
        ("total_initial_margin", "Başlangıç marjı"),
        ("total_maint_margin", "Bakım marjı"),
    ];
    for (k, label) in fields {
        if let Some(x) = a.get(k) {
            println!("  {label:<22} {}", x.as_str().unwrap_or("?"));
        }
    }
    // Varlıklar
    if let Some(assets) = a["assets"].as_array() {
        println!("  --- Varlıklar (bakiye > 0) ---");
        for b in assets {
            let wb = b["wallet_balance"].as_str().unwrap_or("0");
            if wb.parse::<f64>().unwrap_or(0.0) != 0.0 {
                println!(
                    "  {:6} cüzdan: {:>14}  kullanılabilir: {:>14}  uPnL: {}",
                    b["asset"].as_str().unwrap_or(""),
                    wb,
                    b["available_balance"].as_str().unwrap_or("0"),
                    b["unrealized_profit"].as_str().unwrap_or("0"),
                );
            }
        }
    }
}

fn fmt_positions(v: &Value) {
    let items = v.as_array().map(|a| a.clone()).unwrap_or_default();
    let open: Vec<&Value> = items
        .iter()
        .filter(|p| {
            p["position_amt"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|x| x != 0.0)
                .unwrap_or(false)
        })
        .collect();
    if open.is_empty() {
        println!("  (açık pozisyon yok)");
        return;
    }
    for p in open {
        println!(
            "  {:10} {:6} amt: {:>12}  entry: {:>12}  mark: {:>12}  uPnL: {:>12}  lev: {}  margin: {}",
            p["symbol"].as_str().unwrap_or(""),
            p["position_side"].as_str().unwrap_or(""),
            p["position_amt"].as_str().unwrap_or(""),
            p["entry_price"].as_str().unwrap_or(""),
            p["mark_price"].as_str().unwrap_or(""),
            p["un_realized_profit"].as_str().unwrap_or(""),
            p["leverage"].as_str().unwrap_or(""),
            p["margin_type"].as_str().unwrap_or(""),
        );
    }
}

fn fmt_balances(v: &Value) {
    let items = v.as_array().map(|a| a.clone()).unwrap_or_default();
    let mut any = false;
    for b in items {
        let wb = b["wallet_balance"].as_str().unwrap_or("0");
        let bal = wb.parse::<f64>().unwrap_or(0.0);
        if bal != 0.0 {
            any = true;
            println!(
                "  {:6} cüzdan: {:>16}  available: {:>16}  uPnL: {}",
                b["asset"].as_str().unwrap_or(""),
                wb,
                b["available_balance"].as_str().unwrap_or("0"),
                b["unrealized_profit"].as_str().unwrap_or("0"),
            );
        }
    }
    if !any {
        println!("  (sıfırdan büyük bakiye yok — balances ucu varlık listesi döndürüyor)");
    }
}

fn fmt_orders(v: &Value) {
    let items = v.as_array().map(|a| a.clone()).unwrap_or_default();
    if items.is_empty() {
        println!("  (açık emir yok)");
        return;
    }
    for o in items {
        println!(
            "  {} {} {} {} status:{} id:{} cid:{}",
            o["symbol"].as_str().unwrap_or(""),
            o["side"].as_str().unwrap_or(""),
            o["order_type"].as_str().unwrap_or(""),
            o["quantity"].as_str().unwrap_or(""),
            o["status"].as_str().unwrap_or(""),
            o["order_id"].as_str().unwrap_or(""),
            o["client_order_id"].as_str().unwrap_or(""),
        );
    }
}

fn pretty(v: &Value) {
    let s = serde_json::to_string_pretty(v).unwrap_or_default();
    for line in s.lines().take(60) {
        println!("  {line}");
    }
    if s.lines().count() > 60 {
        println!("  ... (çıktı kesildi)");
    }
}

// ── Komut gönderimi ────────────────────────────────────────────────

fn cmd_order(
    c: &mut Console,
    args: &[String],
) -> Result<(), String> {
    // order SYMBOL BUY|SELL TYPE QTY [--usdt N] [--price P] [--stop P] [--tif X] [--pos P] [--reduce] [--close]
    if args.len() < 4 {
        return Err("kullanım: order SYMBOL BUY|SELL LIMIT|MARKET|STOP_MARKET|... QTY|--usdt N [--price P] [--stop P] [--tif GTC|IOC|FOK|GTX] [--pos LONG|SHORT|BOTH] [--reduce] [--close]".into());
    }
    let symbol = args[0].to_uppercase();
    let side = args[1].to_uppercase();
    let order_type = args[2].to_uppercase();
    let mut quantity: Option<String> = None;
    let mut usdt: Option<String> = None;
    let mut price: Option<String> = None;
    let mut stop: Option<String> = None;
    let mut tif: Option<String> = None;
    let mut pos: Option<String> = None;
    let mut reduce = false;
    let mut close = false;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--usdt" => { i += 1; usdt = args.get(i).cloned(); }
            "--price" => { i += 1; price = args.get(i).cloned(); }
            "--stop" | "--stop-price" => { i += 1; stop = args.get(i).cloned(); }
            "--tif" => { i += 1; tif = args.get(i).cloned().map(|x| x.to_uppercase()); }
            "--pos" | "--position-side" => { i += 1; pos = args.get(i).cloned().map(|x| x.to_uppercase()); }
            "--reduce" | "--reduce-only" => reduce = true,
            "--close" | "--close-position" => close = true,
            other => {
                if quantity.is_none() {
                    quantity = Some(other.to_string());
                } else {
                    return Err(format!("bilinmeyen seçenek: {other}"));
                }
            }
        }
        i += 1;
    }
    if quantity.is_none() && usdt.is_none() {
        return Err("QTY veya --usdt N gerekli".into());
    }
    if quantity.is_some() && usdt.is_some() {
        return Err("QTY ve --usdt birlikte verilemez".into());
    }
    let mut m = serde_json::Map::new();
    m.insert("symbol".into(), json!(symbol));
    m.insert("side".into(), json!(side));
    m.insert("type".into(), json!(order_type));
    m.insert("client_order_id".into(), json!(now_cid()));
    if let Some(q) = quantity { m.insert("quantity".into(), json!(q)); }
    if let Some(u) = usdt { m.insert("quote_order_qty".into(), json!(u)); }
    if let Some(p) = price { m.insert("price".into(), json!(p)); }
    if let Some(s) = stop { m.insert("stop_price".into(), json!(s)); }
    if let Some(t) = tif { m.insert("time_in_force".into(), json!(t)); }
    if let Some(p) = pos { m.insert("position_side".into(), json!(p)); }
    if reduce { m.insert("reduce_only".into(), json!(true)); }
    if close { m.insert("close_position".into(), json!(true)); }
    let resp = c.call("POST", "/api/v1/orders", None, Some(Value::Object(m)))?;
    println!("  ✅ {}", pretty_inline(&resp));
    Ok(())
}

fn pretty_inline(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

// ── Ana REPL ───────────────────────────────────────────────────────

fn main() {
    dotenvy::dotenv().ok();
    println!("═══════════════════════════════════════════════════════");
    println!("  🖥️  EXEC CONSOLE — Execution Engine elle komut katmanı");
    println!("  Bağlantı: executiond REST (:3010)  |  help ile komutlar");
    println!("═══════════════════════════════════════════════════════");

    let mut c = Console::new();
    let mut rl = DefaultEditor::new().expect("rustyline");

    loop {
        match rl.readline("exec> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let parts: Vec<String> = line.trim().split_whitespace().map(|s| s.to_string()).collect();
                if parts.is_empty() {
                    continue;
                }
                let cmd = parts[0].to_lowercase();
                let args = &parts[1..];
                let result = dispatch(&mut c, &cmd, args);
                if let Err(e) = result {
                    println!("  ❌ {e}");
                }
            }
            Err(_) => break,
        }
    }
    println!("Konsoldan çıkıldı.");
}

fn dispatch(c: &mut Console, cmd: &str, args: &[String]) -> Result<(), String> {
    match cmd {
        "help" | "?" => print_help(),
        "exit" | "quit" => std::process::exit(0),
        // Durum / kontrol
        "health" | "status" => pretty(&c.call("GET", "/api/v1/healthz", None, None)?),
        "mode" => pretty(&c.call("GET", "/api/v1/mode", None, None)?),
        "risk" => pretty(&c.call("GET", "/api/v1/risk", None, None)?),
        "kill" => {
            match args.first().map(|s| s.to_lowercase()).as_deref() {
                Some("on") | Some("1") | Some("true") => {
                    pretty(&c.call("PUT", "/api/v1/risk/kill-switch", None, Some(json!({"enabled": true})))?)
                }
                Some("off") | Some("0") | Some("false") => {
                    pretty(&c.call("PUT", "/api/v1/risk/kill-switch", None, Some(json!({"enabled": false})))?)
                }
                _ => pretty(&c.call("GET", "/api/v1/risk", None, None)?),
            }
        }
        // Hesap / pozisyon / bakiye
        "account" => fmt_account(&c.call("GET", "/api/v1/account", None, None)?),
        "balance" | "balances" => fmt_balances(&c.call("GET", "/api/v1/balances", None, None)?),
        "positions" | "pos" => {
            if let Some(sym) = args.first() {
                let sym = sym.to_uppercase();
                fmt_positions(&c.call("GET", &format!("/api/v1/positions/{}", sym), None, None)?)
            } else {
                fmt_positions(&c.call("GET", "/api/v1/positions", None, None)?)
            }
        }
        "close" => {
            // close SYMBOL [LONG|SHORT]
            let sym = args.first().ok_or("kullanım: close SYMBOL [LONG|SHORT]")?.to_uppercase();
            let mut m = serde_json::Map::new();
            m.insert("symbol".into(), json!(sym));
            if let Some(side) = args.get(1) {
                let s = side.to_uppercase();
                if s != "LONG" && s != "SHORT" {
                    return Err("LONG veya SHORT gir".into());
                }
                m.insert("position_side".into(), json!(s));
            }
            let resp = c.call("POST", "/api/v1/positions/close", None, Some(Value::Object(m)))?;
            println!("  ✅ {}", pretty_inline(&resp));
        }
        "closeall" | "close-all" => {
            // Tüm açık pozisyonları kapat.
            let resp = c.call("POST", "/api/v1/positions/close", None, Some(json!({})))?;
            println!("  ✅ {}", pretty_inline(&resp));
        }
        // Emirler
        "orders" => {
            let q = match args.first() {
                Some(sym) => format!("symbol={}", sym.to_uppercase()),
                None => String::new(),
            };
            fmt_orders(&c.call("GET", "/api/v1/orders", Some(&q), None)?)
        }
        "query" => {
            // query SYM [--order-id N] [--cid X]
            let sym = args.first().ok_or("kullanım: query SYMBOL [--order-id N] [--cid X]")?.to_uppercase();
            let mut oid = String::new();
            let mut cid = String::new();
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--order-id" => { i += 1; oid = args.get(i).cloned().unwrap_or_default(); }
                    "--cid" | "--client-order-id" => { i += 1; cid = args.get(i).cloned().unwrap_or_default(); }
                    o => return Err(format!("bilinmeyen seçenek: {o}")),
                }
                i += 1;
            }
            let mut q = format!("symbol={}", sym);
            if !oid.is_empty() { q.push_str(&format!("&order_id={}", oid)); }
            if !cid.is_empty() { q.push_str(&format!("&client_order_id={}", cid)); }
            pretty(&c.call("GET", "/api/v1/orders/query", Some(&q), None)?)
        }
        "cancel" => {
            let sym = args.first().ok_or("kullanım: cancel SYMBOL [--order-id N] [--cid X]")?.to_uppercase();
            let mut oid = String::new();
            let mut cid = String::new();
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--order-id" => { i += 1; oid = args.get(i).cloned().unwrap_or_default(); }
                    "--cid" | "--client-order-id" => { i += 1; cid = args.get(i).cloned().unwrap_or_default(); }
                    o => return Err(format!("bilinmeyen seçenek: {o}")),
                }
                i += 1;
            }
            let mut q = format!("symbol={}", sym);
            if !oid.is_empty() { q.push_str(&format!("&order_id={}", oid)); }
            if !cid.is_empty() { q.push_str(&format!("&client_order_id={}", cid)); }
            pretty(&c.call("POST", "/api/v1/orders/cancel", Some(&q), None)?)
        }
        "cancelall" => {
            let sym = args.first().ok_or("kullanım: cancelall SYMBOL")?.to_uppercase();
            pretty(&c.call("DELETE", "/api/v1/orders/open", Some(&format!("symbol={}", sym)), None)?)
        }
        "modify" => {
            // modify SYMBOL CID [--qty N] [--price P] [--stop P]
            let sym = args.first().ok_or("kullanım: modify SYMBOL CID [--qty N] [--price P] [--stop P]")?.to_uppercase();
            let cid = args.get(1).ok_or("cid gerekli")?.clone();
            let mut qty = None; let mut price = None; let mut stop = None;
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--qty" | "--quantity" => { i += 1; qty = args.get(i).cloned(); }
                    "--price" => { i += 1; price = args.get(i).cloned(); }
                    "--stop" => { i += 1; stop = args.get(i).cloned(); }
                    o => return Err(format!("bilinmeyen seçenek: {o}")),
                }
                i += 1;
            }
            let body = json!({
                "symbol": sym,
                "client_order_id": cid,
                "quantity": qty,
                "price": price,
                "stop_price": stop,
            });
            pretty(&c.call("PUT", &format!("/api/v1/orders/{}", cid), None, Some(body))?)
        }
        "buy" | "sell" => {
            // buy/sell SYMBOL QTY | --usdt N  [--pos LONG|SHORT]
            let sym = args.first().ok_or("kullanım: buy/sell SYMBOL QTY|--usdt N [--pos LONG|SHORT]")?.to_uppercase();
            let mut qty: Option<String> = None;
            let mut usdt: Option<String> = None;
            let mut pos: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--usdt" => { i += 1; usdt = args.get(i).cloned(); }
                    "--pos" | "--position-side" => { i += 1; pos = args.get(i).cloned().map(|x| x.to_uppercase()); }
                    other => {
                        if qty.is_none() {
                            qty = Some(other.to_string());
                        } else {
                            return Err(format!("bilinmeyen seçenek: {other}"));
                        }
                    }
                }
                i += 1;
            }
            if qty.is_none() && usdt.is_none() {
                return Err("miktar (QTY) veya --usdt N gerekli".into());
            }
            if qty.is_some() && usdt.is_some() {
                return Err("QTY ve --usdt birlikte verilemez".into());
            }
            let side = if cmd == "buy" { "BUY" } else { "SELL" };
            let mut m = serde_json::Map::new();
            m.insert("symbol".into(), json!(sym));
            m.insert("side".into(), json!(side));
            m.insert("type".into(), json!("MARKET"));
            m.insert("client_order_id".into(), json!(now_cid()));
            if let Some(q) = qty { m.insert("quantity".into(), json!(q)); }
            if let Some(u) = usdt { m.insert("quote_order_qty".into(), json!(u)); }
            if let Some(p) = pos { m.insert("position_side".into(), json!(p)); }
            let resp = c.call("POST", "/api/v1/orders", None, Some(Value::Object(m)))?;
            println!("  ✅ {}", pretty_inline(&resp));
        }
        "order" => cmd_order(c, args)?,
        // Yapılandırma
        "leverage" => {
            let sym = args.first().ok_or("kullanım: leverage SYMBOL N")?.to_uppercase();
            let n: u32 = args.get(1).ok_or("kaldıraç değeri gerekli")?.parse().map_err(|_| "sayı gir")?;
            pretty(&c.call("PUT", &format!("/api/v1/symbols/{}/leverage", sym), None, Some(json!({"leverage": n})))?)
        }
        "margintype" => {
            let sym = args.first().ok_or("kullanım: margintype SYMBOL ISOLATED|CROSSED")?.to_uppercase();
            let mt = args.get(1).ok_or("ISOLATED veya CROSSED gir")?.to_uppercase();
            pretty(&c.call("PUT", &format!("/api/v1/symbols/{}/margin-type", sym), None, Some(json!({"margin_type": mt})))?)
        }
        "margin" => {
            // margin SYMBOL AMOUNT add|remove
            let sym = args.first().ok_or("kullanım: margin SYMBOL AMOUNT add|remove")?.to_uppercase();
            let amount = args.get(1).ok_or("miktar gerekli")?.clone();
            let dir = match args.get(2).map(|s| s.to_lowercase()).as_deref() {
                Some("add") => 1,
                Some("remove") => 2,
                _ => return Err("add veya remove gir".into()),
            };
            pretty(&c.call("POST", &format!("/api/v1/symbols/{}/margin", sym), None, Some(json!({"amount": amount, "direction": dir})))?)
        }
        "hedge" => {
            let v: bool = match args.first().map(|s| s.to_lowercase()).as_deref() {
                Some("on") | Some("1") | Some("true") => true,
                Some("off") | Some("0") | Some("false") => false,
                _ => return Err("kullanım: hedge on|off".into()),
            };
            pretty(&c.call("PUT", "/api/v1/position-mode", None, Some(json!({"dual": v})))?)
        }
        "multiass" => {
            let v: bool = match args.first().map(|s| s.to_lowercase()).as_deref() {
                Some("on") | Some("1") | Some("true") => true,
                Some("off") | Some("0") | Some("false") => false,
                _ => return Err("kullanım: multiass on|off".into()),
            };
            pretty(&c.call("PUT", "/api/v1/multi-assets", None, Some(json!({"enabled": v})))?)
        }
        // Borsa salt-okunur
        "funding" => {
            let sym = args.first().ok_or("kullanım: funding SYMBOL")?.to_uppercase();
            pretty(&c.call("GET", "/api/v1/funding", Some(&format!("symbol={}", sym)), None)?)
        }
        "income" => {
            let mut q = String::new();
            if let Some(sym) = args.first() {
                q.push_str(&format!("symbol={}", sym.to_uppercase()));
            }
            let mut i = if args.first().is_some() { 1 } else { 0 };
            while i < args.len() {
                match args[i].as_str() {
                    "--type" => {
                        i += 1;
                        if let Some(t) = args.get(i) {
                            if !q.is_empty() { q.push('&'); }
                            q.push_str(&format!("type={}", t));
                        }
                    }
                    "--limit" => {
                        i += 1;
                        if let Some(l) = args.get(i) {
                            if !q.is_empty() { q.push('&'); }
                            q.push_str(&format!("limit={}", l));
                        }
                    }
                    o => return Err(format!("bilinmeyen seçenek: {o}")),
                }
                i += 1;
            }
            pretty(&c.call("GET", "/api/v1/income", if q.is_empty() { None } else { Some(&q) }, None)?)
        }
        "forceorders" => {
            let q = args.first().map(|s| format!("symbol={}", s.to_uppercase())).unwrap_or_default();
            pretty(&c.call("GET", "/api/v1/force-orders", if q.is_empty() { None } else { Some(&q) }, None)?)
        }
        "exinfo" => {
            let sym = args.first().ok_or("kullanım: exinfo SYMBOL")?.to_uppercase();
            let v = c.call("GET", &format!("/api/v1/exchange-info/{}", sym), None, None)?;
            let f = v["filters"].as_array().map(|a| a.len()).unwrap_or(0);
            println!("  {} status:{} base:{} quote:{} qtyPrec:{} pricePrec:{} filterSayısı:{}",
                sym, v["status"].as_str().unwrap_or(""), v["base_asset"].as_str().unwrap_or(""),
                v["quote_asset"].as_str().unwrap_or(""), v["quantity_precision"], v["price_precision"], f);
        }
        "commission" => {
            let sym = args.first().ok_or("kullanım: commission SYMBOL")?.to_uppercase();
            pretty(&c.call("GET", &format!("/api/v1/commission-rate/{}", sym), None, None)?)
        }
        "adl" => {
            let sym = args.first().ok_or("kullanım: adl SYMBOL")?.to_uppercase();
            pretty(&c.call("GET", &format!("/api/v1/adl/{}", sym), None, None)?)
        }
        "tradingstatus" => pretty(&c.call("GET", "/api/v1/trading-status", None, None)?),
        "batch" => {
            let orders: Vec<Value> = args
                .chunks(4)
                .map(|ch| {
                    json!({
                        "symbol": ch[0].to_uppercase(),
                        "side": ch[1].to_uppercase(),
                        "type": ch[2].to_uppercase(),
                        "quantity": ch[3].clone(),
                    })
                })
                .collect();
            pretty(&c.call("POST", "/api/v1/orders/batch", None, Some(json!({ "orders": orders })))?)
        }
        _ => {
            println!("  ❌ bilinmeyen komut: {cmd} — 'help' yazın");
        }
    }
    Ok(())
}

fn print_help() {
    println!();
    println!("  ── Durum / Kontrol ────────────────────────────────");
    println!("  health | status           executiond sağlığı");
    println!("  mode                      mod + dry_run");
    println!("  risk                      risk durumu");
    println!("  kill on|off|(durum)       kill switch aç/kapat/gör");
    println!("  ── Hesap ──────────────────────────────────────────");
    println!("  account                   hesap özeti");
    println!("  balance                   bakiyeler");
    println!("  positions [SYM]           açık pozisyonlar");
    println!("  ── Emirler ────────────────────────────────────────");
    println!("  buy SYM QTY|--usdt N [--pos LONG|SHORT]   market BUY (USDT büyüklük de olur)");
    println!("  sell SYM QTY|--usdt N [--pos LONG|SHORT]  market SELL");
    println!("  order SYM SIDE TYPE QTY|--usdt N [--price P] [--stop P] [--tif X] [--pos P] [--reduce] [--close]");
    println!("  batch SYM SIDE TYPE QTY [...]        toplu emir (4'erli gruplar)");
    println!("  orders [SYM]              açık emirler");
    println!("  query SYM [--order-id N] [--cid X]   emir sorgula");
    println!("  cancel SYM [--order-id N] [--cid X]  emir iptal");
    println!("  cancelall SYM             tüm açık emirleri iptal");
    println!("  modify SYM CID [--qty N] [--price P] [--stop P]");
    println!("  close SYM [LONG|SHORT]    sembolün açık pozisyon(lar)ını kapat");
    println!("  closeall                  TÜM açık pozisyonları kapat");
    println!("  ── Yapılandırma ───────────────────────────────────");
    println!("  leverage SYM N            kaldıraç");
    println!("  margintype SYM ISOLATED|CROSSED");
    println!("  margin SYM AMOUNT add|remove");
    println!("  hedge on|off              pozisyon modu");
    println!("  multiass on|off           multi-assets");
    println!("  ── Borsa sorguları ────────────────────────────────");
    println!("  funding SYM  |  income [SYM] [--type T] [--limit N]");
    println!("  forceorders [SYM]  |  exinfo SYM  |  commission SYM");
    println!("  adl SYM  |  tradingstatus");
    println!("  ── ─────────────────────────────────────────────────");
    println!("  help  |  exit");
    println!();
}
