//! alerts.toml yönetim aracı (Rust) — Python karşılığı: scripts/alerts_cli.py
//!
//! Kullanım:
//!   alerts list
//!   alerts add --symbol HEIUSDT --condition above --price 0.22 [--voice "..."] [--cooldown 30] [--tolerance 0.0005]
//!   alerts update --symbol HEIUSDT --condition above --old-price 0.21628 [--price 0.22] [--voice "..."] [--cooldown 30]
//!   alerts remove --symbol HEIUSDT --condition above --price 0.21628

use std::process::exit;

const CONFIG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../alerts.toml");

// ── Basit blok ayrıştırma ────────────────────────────────────
#[derive(Debug, Clone)]
struct AlertBlock {
    symbol: String,
    condition: String,
    price: String,
    tolerance: Option<String>,
    voice: Option<String>,
    cooldown: Option<String>,
}

fn norm_price(v: &str) -> String {
    match v.trim().parse::<f64>() {
        Ok(f) => format!("{}", f),
        Err(_) => v.trim().to_string(),
    }
}

fn parse_blocks(content: &str) -> (Vec<String>, Vec<AlertBlock>) {
    let mut header = Vec::new();
    let mut blocks = Vec::new();
    let mut cur: Option<AlertBlock> = None;

    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("[[alerts]]") {
            if let Some(b) = cur.take() {
                blocks.push(b);
            }
            cur = Some(AlertBlock {
                symbol: String::new(),
                condition: String::new(),
                price: String::new(),
                tolerance: None,
                voice: None,
                cooldown: None,
            });
            continue;
        }
        match &mut cur {
            Some(b) => {
                if t.starts_with("symbol") {
                    b.symbol = val_of(t);
                } else if t.starts_with("condition") {
                    b.condition = val_of(t);
                } else if t.starts_with("price") {
                    b.price = val_of(t);
                } else if t.starts_with("tolerance_pct") {
                    b.tolerance = Some(val_of(t));
                } else if t.starts_with("voice") {
                    b.voice = Some(val_of(t));
                } else if t.starts_with("cooldown_sec") {
                    b.cooldown = Some(val_of(t));
                }
            }
            None => header.push(line.to_string()),
        }
    }
    if let Some(b) = cur.take() {
        blocks.push(b);
    }
    (header, blocks)
}

fn val_of(line: &str) -> String {
    let (_, v) = line.split_once('=').unwrap_or(("", ""));
    v.trim().trim_matches('"').trim_matches('\'').trim().to_string()
}

fn render_block(b: &AlertBlock) -> String {
    let mut out = String::from("[[alerts]]\n");
    out.push_str(&format!("symbol = \"{}\"\n", b.symbol));
    out.push_str(&format!("condition = \"{}\"\n", b.condition));
    out.push_str(&format!("price = {}\n", norm_price(&b.price)));
    if let Some(t) = &b.tolerance {
        out.push_str(&format!("tolerance_pct = {}\n", norm_price(t)));
    }
    if let Some(v) = &b.voice {
        out.push_str(&format!("voice = \"{}\"\n", v));
    }
    if let Some(c) = &b.cooldown {
        out.push_str(&format!("cooldown_sec = {}\n", c));
    }
    out
}

fn write_config(header: &[String], blocks: &[AlertBlock]) {
    let mut out = header.join("\n");
    if !out.is_empty() && !blocks.is_empty() {
        out.push('\n');
    }
    if !blocks.is_empty() {
        let rendered: Vec<String> = blocks.iter().map(render_block).collect();
        out.push_str(&rendered.join("\n"));
        out.push('\n');
    }
    std::fs::write(CONFIG, out).expect("alerts.toml yazılamadı");
}

// ── Komutlar ────────────────────────────────────────────────
fn cmd_list() {
    let content = std::fs::read_to_string(CONFIG).unwrap_or_default();
    let (_, blocks) = parse_blocks(&content);
    if blocks.is_empty() {
        println!("  📭 Alarmsız");
        return;
    }
    for (i, b) in blocks.iter().enumerate() {
        let voice = b.voice.clone().unwrap_or_default();
        let vdesc = if voice.is_empty() { "🔊 beep".to_string() } else { format!("🗣️ {voice}") };
        let tol = b.tolerance.clone().unwrap_or_else(|| "-".into());
        let cd = b.cooldown.clone().unwrap_or_else(|| "-".into());
        println!(
            "  [{}] {:<9} {:<6} fiyat={:<10} tol={} cooldown={}s {}",
            i + 1, b.symbol, b.condition, b.price, tol, cd, vdesc
        );
    }
}

fn cmd_add(args: &[String]) {
    let sym = arg(&args, "--symbol").unwrap_or_else(|| die("--symbol gerekli"));
    let cond = arg(&args, "--condition").unwrap_or_else(|| die("--condition gerekli"));
    let price = arg(&args, "--price").unwrap_or_else(|| die("--price gerekli"));
    let content = std::fs::read_to_string(CONFIG).unwrap_or_default();
    let (header, mut blocks) = parse_blocks(&content);
    blocks.push(AlertBlock {
        symbol: sym.to_uppercase(),
        condition: cond.to_lowercase(),
        price: price.to_string(),
        tolerance: arg(&args, "--tolerance"),
        voice: arg(&args, "--voice"),
        cooldown: Some(arg(&args, "--cooldown").unwrap_or_else(|| "30".to_string())),
    });
    write_config(&header, &blocks);
    println!("✅ Eklendi: {} {} {}", sym.to_uppercase(), cond, price);
}

fn cmd_update(args: &[String]) {
    let sym = arg(&args, "--symbol").unwrap_or_else(|| die("--symbol gerekli")).to_uppercase();
    let cond = arg(&args, "--condition").unwrap_or_else(|| die("--condition gerekli")).to_lowercase();
    let old = arg(&args, "--old-price").unwrap_or_else(|| die("--old-price gerekli"));
    let content = std::fs::read_to_string(CONFIG).unwrap_or_default();
    let (header, mut blocks) = parse_blocks(&content);
    let target = norm_price(&old);

    for b in blocks.iter_mut() {
        if b.symbol == sym && b.condition == cond && norm_price(&b.price) == target {
            if let Some(p) = arg(&args, "--price") {
                b.price = p.to_string();
            }
            if let Some(v) = arg(&args, "--voice") {
                b.voice = Some(v.to_string());
            }
            if let Some(c) = arg(&args, "--cooldown") {
                b.cooldown = Some(c.to_string());
            }
            if let Some(t) = arg(&args, "--tolerance") {
                b.tolerance = Some(t.to_string());
            }
            write_config(&header, &blocks);
            println!("✅ Güncellendi: {sym} {cond}");
            return;
        }
    }
    eprintln!("❌ Alarm bulunamadı: {sym} {cond} {old}");
    exit(1);
}

fn cmd_remove(args: &[String]) {
    let sym = arg(&args, "--symbol").unwrap_or_else(|| die("--symbol gerekli")).to_uppercase();
    let cond = arg(&args, "--condition").unwrap_or_else(|| die("--condition gerekli")).to_lowercase();
    let price = arg(&args, "--price").unwrap_or_else(|| die("--price gerekli"));
    let content = std::fs::read_to_string(CONFIG).unwrap_or_default();
    let (header, blocks) = parse_blocks(&content);
    let target = norm_price(&price);
    let before = blocks.len();
    let kept: Vec<AlertBlock> = blocks
        .into_iter()
        .filter(|b| !(b.symbol == sym && b.condition == cond && norm_price(&b.price) == target))
        .collect();
    if kept.len() == before {
        eprintln!("❌ Alarm bulunamadı: {sym} {cond} {target}");
        exit(1);
    }
    write_config(&header, &kept);
    println!("✅ Silindi: {sym} {cond} {target}");
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("list");
    match cmd {
        "list" => cmd_list(),
        "add" => cmd_add(&args[1..]),
        "update" => cmd_update(&args[1..]),
        "remove" => cmd_remove(&args[1..]),
        _ => {
            eprintln!("Kullanım: alerts list|add|update|remove");
            exit(1);
        }
    }
}
