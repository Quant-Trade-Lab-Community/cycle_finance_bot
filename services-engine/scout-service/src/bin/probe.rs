//! Scout ring buffer tüketici örneği (`/dev/shm/demir_yumruk_scout`).
//!
//! Fırsat (Opportunity) ve sembol metriklerini (SymbolMetrics) okur, yazdırır.
//! Kullanım:
//!   cargo run -p scout-service --bin probe           # canlı akış
//!   cargo run -p scout-service --bin probe -- --once # son N slot'u dök

use transport::ring_buffer::GenerationalRingBuffer;
use contracts::events::EventType;
use contracts::wire::decode;
use std::time::Duration;

const RING_NAME: &str = "/demir_yumruk_scout";
const RING_CAPACITY: usize = 20_000;

fn symbol_str(symbol: &[u8; 16]) -> &str {
    let len = symbol.iter().position(|&c| c == 0).unwrap_or(16);
    std::str::from_utf8(&symbol[..len]).unwrap_or("UNKNOWN")
}

fn main() {
    let ring = GenerationalRingBuffer::with_name(RING_NAME, RING_CAPACITY);

    let once = std::env::args().any(|a| a == "--once");

    if once {
        let head = ring.get_head();
        eprintln!("[probe] head={} capacity={}", head, RING_CAPACITY);
        let start = head.saturating_sub(64);
        let mut printed = 0;
        for seq in start..head {
            if let Some(slot) = ring.read_slot(seq) {
                printed += 1;
                print_ev(&slot.data[..slot.len as usize]);
            }
        }
        eprintln!("[probe] {} slot okundu", printed);
        return;
    }

    let mut last = ring.get_head();
    loop {
        let head = ring.get_head();
        if head > last {
            for seq in last..head {
                if let Some(slot) = ring.read_slot(seq) {
                    print_ev(&slot.data[..slot.len as usize]);
                }
            }
            last = head;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn print_ev(buf: &[u8]) {
    let Some(ev) = decode(buf) else { return };
    match ev.payload {
        EventType::Opportunity {
            score,
            efficiency,
            price_bps_per_s,
            price_ticks_per_s,
            ob_changes_per_s,
            spread_bps,
            verdict,
        } => {
            println!(
                "OPP {:16} verdict={} score={} eff={} p_bps={} ticks={} ob={} spread={}",
                symbol_str(&ev.symbol),
                verdict,
                score,
                efficiency,
                price_bps_per_s,
                price_ticks_per_s,
                ob_changes_per_s,
                spread_bps,
            );
        }
        EventType::SymbolMetrics {
            score,
            efficiency,
            price_bps_per_s,
            price_ticks_per_s,
            ob_changes_per_s,
            spread_bps,
        } => {
            println!(
                "MET {:16} score={} eff={} p_bps={} ticks={} ob={} spread={}",
                symbol_str(&ev.symbol),
                score,
                efficiency,
                price_bps_per_s,
                price_ticks_per_s,
                ob_changes_per_s,
                spread_bps,
            );
        }
        _ => {}
    }
}
