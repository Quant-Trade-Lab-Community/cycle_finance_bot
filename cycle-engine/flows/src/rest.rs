//! REST fallback — bu ağdan Binance WS ile iletilmeyen akışlar için.
//!
//! WS ile gelmeyen akışlar (funding, markprice, indexprice, lastprice, oi)
//! aynı veri hattını korur: REST yanıtı → WS-format frame → `raw_tx`
//! → parse → validate → ring → TimescaleDB. Yani veri akışı kuralları
//! bozulmaz; yalnızca giriş kaynağı REST'tir.
//!
//! Rate koruması:
//! - Her poll döngüsü öncesi ortak `RateGate`'ten token alınır.
//! - HTTP **429** (limit aşımı) → 60 saniye geri çekilme.
//! - HTTP **418** (IP banı/teapot) → 5 dakika geri çekilme.
//!
//! Her akış kendi dakikalık ağırlığını (request sayısı × endpoint weight)
//! `/tmp/cycle_flow_weights/<flow>.weight` dosyasına yazar — monitor sekmesi
//! okur ve toplamı gösterir.
//!
//! Varsayılan poll `CYCLE_REST_POLL_MS` (2 s); open-interest en az 5 s.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use flume::Sender;
use gateway::rate_gate::RateGate;
use serde_json::json;
use transport::flow::FlowKind;

const BASE_URL: &str = "https://fapi.binance.com";
const BACKOFF_429: Duration = Duration::from_secs(60);
const BACKOFF_418: Duration = Duration::from_secs(300);
const WEIGHT_DIR: &str = "/tmp/cycle_flow_weights";

/// Dokümante endpoint ağırlıkları (Binance USDS-M futures).
fn request_weight(kind: FlowKind) -> u64 {
    match kind {
        FlowKind::LastPrice => 2, // ticker/price (symbol)
        _ => 1,                   // premiumIndex, openInterest
    }
}

fn poll_interval(kind: FlowKind) -> Duration {
    let base = std::env::var("CYCLE_REST_POLL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2000);
    // Open interest ~5 sn'de güncellenir; daha sık çekmenin anlamı yok.
    if kind == FlowKind::OpenInterest {
        Duration::from_millis(base.max(5000))
    } else {
        Duration::from_millis(base)
    }
}

fn weight_file(kind: FlowKind) -> String {
    format!("{WEIGHT_DIR}/{}.weight", kind.as_str())
}

enum Fetch {
    Frame(String),
    RateLimited,
    Banned,
    Error,
}

/// REST poller thread'ini başlatır; sonsuza kadar çalışır (`join` ile bloklanır).
pub fn spawn(kind: FlowKind, symbols: Vec<String>, tx: Sender<Vec<u8>>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        std::fs::create_dir_all(WEIGHT_DIR).ok();
        let client = reqwest::blocking::Client::new();
        let gate = RateGate::open_default();
        let interval = poll_interval(kind);
        println!("[{}] REST fallback aktif (poll: {} ms)", kind.as_str(), interval.as_millis());

        // Dakikalık ağırlık sayacı: poller artırır, yazıcı thread dosyaya yazar + dakika başında sıfırlar.
        let weight = Arc::new(AtomicU64::new(0));
        {
            let weight = weight.clone();
            let file = weight_file(kind);
            std::thread::spawn(move || {
                let mut cur_min = now_minute();
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    let m = now_minute();
                    if m != cur_min {
                        weight.store(0, Ordering::Relaxed);
                        cur_min = m;
                    }
                    if let Ok(mut f) = std::fs::File::create(&file) {
                        use std::io::Write;
                        let _ = writeln!(f, "{} {}", kind.as_str(), weight.load(Ordering::Relaxed));
                    }
                }
            });
        }

        loop {
            // Kapıdan token yoksa bu döngüyü atla — Binance limiti korunur.
            if gate.acquire(Duration::from_millis(500)) {
                let mut limited = false;
                let mut banned = false;
                for sym in &symbols {
                    match fetch_frame(&client, kind, sym, &weight) {
                        Fetch::Frame(frame) => {
                            // Bounded kuyruk → geri basınç; drop asla hot path'i bloke etmez.
                            let _ = tx.try_send(frame.into_bytes());
                        }
                        Fetch::RateLimited => limited = true,
                        Fetch::Banned => banned = true,
                        Fetch::Error => {}
                    }
                }
                if banned {
                    println!("[{}] ⛔ 418 (teapot/IP banı) — {} sn geri çekilme", kind.as_str(), BACKOFF_418.as_secs());
                    std::thread::sleep(BACKOFF_418);
                    continue;
                }
                if limited {
                    println!("[{}] ⚠️ 429 (rate limit) — {} sn geri çekilme", kind.as_str(), BACKOFF_429.as_secs());
                    std::thread::sleep(BACKOFF_429);
                    continue;
                }
            }
            std::thread::sleep(interval);
        }
    })
}

/// REST yanıtını `parse_for`'un anlayacağı WS-format frame'e çevirir.
/// HTTP yanıtı alındığında endpoint ağırlığını sayaca ekler.
fn fetch_frame(client: &reqwest::blocking::Client, kind: FlowKind, sym: &str, weight: &AtomicU64) -> Fetch {
    let up = sym.to_uppercase();
    let low = up.to_lowercase();

    let (body, rate, ban) = match client
        .get(format!("{BASE_URL}/{}", endpoint(kind, &up)))
        .send()
    {
        Ok(r) => {
            let status = r.status().as_u16();
            // Yanıt alındı → istek ağırlığı say (429/418 dahil — Binance sayar).
            weight.fetch_add(request_weight(kind), Ordering::Relaxed);
            let rate = status == 429;
            let ban = status == 418;
            match r.json::<serde_json::Value>() {
                Ok(v) => (v, rate, ban),
                Err(_) => return Fetch::Error,
            }
        }
        Err(_) => return Fetch::Error,
    };
    if ban {
        return Fetch::Banned;
    }
    if rate {
        return Fetch::RateLimited;
    }

    let frame = match kind {
        FlowKind::Funding | FlowKind::MarkPrice => json!({
            "stream": format!("{low}@markPrice@1s"),
            "data": {
                "e": "markPriceUpdate",
                "s": body["symbol"],
                "p": body["markPrice"],
                "i": body["indexPrice"],
                "r": body["lastFundingRate"],
                "T": body["nextFundingTime"],
            }
        }),
        FlowKind::IndexPrice => json!({
            "stream": format!("{low}@indexPrice@1s"),
            "data": { "e": "indexPriceUpdate", "s": body["symbol"], "i": body["indexPrice"] }
        }),
        FlowKind::LastPrice => json!({
            "stream": format!("{low}@lastPrice@1s"),
            "data": { "e": "lastPriceUpdate", "s": body["symbol"], "p": body["price"] }
        }),
        FlowKind::OpenInterest => json!({
            "stream": "!openInterest@arr",
            "data": [ { "e": "openInterest", "E": body["time"], "s": body["symbol"], "i": body["openInterest"] } ]
        }),
        _ => return Fetch::Error,
    };
    Fetch::Frame(frame.to_string())
}

fn endpoint(kind: FlowKind, sym: &str) -> String {
    match kind {
        FlowKind::Funding | FlowKind::MarkPrice | FlowKind::IndexPrice => {
            format!("fapi/v1/premiumIndex?symbol={sym}")
        }
        FlowKind::LastPrice => format!("fapi/v1/ticker/price?symbol={sym}"),
        FlowKind::OpenInterest => format!("fapi/v1/openInterest?symbol={sym}"),
        _ => String::new(),
    }
}

fn now_minute() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 60)
        .unwrap_or(0)
}
