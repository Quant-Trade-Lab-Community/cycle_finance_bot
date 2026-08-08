//! Detektör → strateji köprüsü.
//!
//! "Scout" ring buffer'i (`/cycle_finance_scout`) detektörler (mikroyapı analizi,
//! misalignment, candle-classifier) tarafından doldurulur; bu köprü ring'deki
//! `EventType::Opportunity` frame'lerini okur ve bunları yüksek-performanslı
//! **tek tüketici** olarak strateji/execution katmanına iletir.
//!
//! Tasarım:
//!   - Ring create etme taraf değil yalnızca OKUMA (cursor ilerletme).
//!   - Ok the frametip: `wire::decode` → `EventType::Opportunity`.
//!   - Geride kalan (overwritten) slotlar `read_slot` generational check ile
//!     atlanır — hiçbir zaman yarım/tutarsız veri işlenmez.

use contracts::events::{EventType, OwnedEvent};
use contracts::wire;
use rust_decimal::Decimal;
use transport::ring_buffer::{GenerationalRingBuffer, MarketDataSlot};
use std::time::Duration;

/// Scout ring'in POSIX shm adı (detektör DATA modunda buraya yazar).
pub const SCOUT_RING_NAME: &str = "/cycle_finance_scout";
/// Scout ring kapasitesi (detektör ile aynı değer).
pub const SCOUT_RING_CAPACITY: usize = 20_000;

/// Ring'den alınan ve strateji katmanına iletilen fırısat sinyali.
///
/// `verdict` detektör kararıdır:
///   0=GUCLU, 1=IYI, 2=NORMAL, 3=BOT/GURULTU, 4=ZAYIF
#[derive(Debug, Clone, PartialEq)]
pub struct OpportunityHit {
    pub symbol: String,
    pub score: Decimal,
    pub efficiency: Decimal,
    pub price_bps_per_s: Decimal,
    pub price_ticks_per_s: Decimal,
    pub spread_bps: Decimal,
    pub verdict: u8,
}

impl OpportunityHit {
    /// Verdict eşiğini aşan fırısatlar için hızlı filtre (0 ve 1 güçlü sinyaldir).
    pub fn is_actionable(&self, max_verdict: u8) -> bool {
        self.verdict <= max_verdict
    }
}

pub struct DetectorBridge {
    ring: GenerationalRingBuffer,
    cursor: u64,
}

impl DetectorBridge {
    /// Mevcut scout ring'ini açar (oluşturursa producer açar; biz sadece okuruz).
    pub fn with_name(name: &str, capacity: usize) -> Self {
        Self {
            ring: GenerationalRingBuffer::with_name(name, capacity),
            cursor: 0,
        }
    }

    pub fn new() -> Self {
        Self::with_name(SCOUT_RING_NAME, SCOUT_RING_CAPACITY)
    }

    pub fn ring(&self) -> &GenerationalRingBuffer {
        &self.ring
    }

    /// `cursor`'dan `head`'e kadar yeni frame'leri okur; `Opportunity` olanları
    /// `handler`'a iletir. Dönen değer işlenen toplam fırısat sayısıdır.
    ///
    /// Not: `poll` çağırmak pahalı değildir (yeni yazılmış frame yoksa no-op).
    pub fn poll(&mut self, mut handler: impl FnMut(&OpportunityHit)) -> usize {
        let head = self.ring.get_head();
        let mut hits = 0usize;

        while self.cursor < head {
            let seq = self.cursor;
            if let Some(slot) = self.ring.read_slot(seq) {
                if let Some(ev) = decode_frame(&slot) {
                    if let EventType::Opportunity {
                        score,
                        efficiency,
                        price_bps_per_s,
                        price_ticks_per_s,
                        ob_changes_per_s: _,
                        spread_bps,
                        verdict,
                    } = &ev.payload
                    {
                        handler(&OpportunityHit {
                            symbol: symbol_to_string(&ev.symbol),
                            score: *score,
                            efficiency: *efficiency,
                            price_bps_per_s: *price_bps_per_s,
                            price_ticks_per_s: *price_ticks_per_s,
                            spread_bps: *spread_bps,
                            verdict: *verdict,
                        });
                        hits += 1;
                    }
                }
            }
            self.cursor += 1;
        }
        hits
    }
}

impl Default for DetectorBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Arka planda sürekli çalışan köprü tüketicisi: her 100ms'de scout ring'ini
/// okur ve güçlü fırısı sinyallerini `handler`'a iletir.
pub fn spawn_watcher(mut handler: impl FnMut(&OpportunityHit) + Send + 'static) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut bridge = DetectorBridge::with_name(SCOUT_RING_NAME, SCOUT_RING_CAPACITY);
        println!("[BRIDGE] Scout ring izleniyor: {} (cap {})", SCOUT_RING_NAME, SCOUT_RING_CAPACITY);
        loop {
            let hits = bridge.poll(&mut handler);
            if hits > 0 {
                println!("[BRIDGE] {} yeni fırısat frame'i işlendi.", hits);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
}

/// Wire frame -> OwnedEvent; bozuk/yarım frame 'None' döner.
fn decode_frame(slot: &MarketDataSlot) -> Option<OwnedEvent> {
    if slot.len == 0 || slot.len as usize > slot.data.len() {
        return None;
    }
    wire::decode(&slot.data[..slot.len as usize])
}

/// C-stili [u8; 16] sembolü temizlenmiş String yapar (null terminator kırpılır).
pub fn symbol_to_string(raw: &[u8; 16]) -> String {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).to_string()
}
