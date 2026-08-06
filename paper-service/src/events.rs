//! Event Sourcing katmanı.
//!
//! Tüm state değişiklikleri `DomainEvent` olarak saklanır. Çökme durumunda
//! olaylar tekrar oynatılarak (replay) son duruma ulaşılır.
//!
//! Depolama stratejisi (plan §11):
//!   - **Sled WAL**: her event önce diske (yedekli, Postgres yokken bile)
//!   - **PostgreSQL**: `--features full` ile event store olarak senkronize
//!   - **Snapshot**: `account_snapshots` tablosu (her 1000 event'te bir)

pub use execution_engine::paper::domain_event::DomainEvent;

use std::sync::Arc;

pub trait EventStore: Send + Sync {
    fn append(&mut self, event: &DomainEvent);
    fn replay(&self) -> Vec<DomainEvent>;
    fn snapshot(&mut self) {}
}

/// Uçucu (dev) store — process sonunda kaybolur.
#[derive(Default)]
pub struct InMemoryEventStore {
    events: Vec<DomainEvent>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl EventStore for InMemoryEventStore {
    fn append(&mut self, event: &DomainEvent) {
        self.events.push(event.clone());
    }
    fn replay(&self) -> Vec<DomainEvent> {
        self.events.clone()
    }
}

/// Sled (embedded) WAL store — her event önce diske, sıralı olarak yazılır.
pub struct SledEventStore {
    db: sled::Db,
    counter: u64,
}

impl SledEventStore {
    pub fn open(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        // Son kullanılan sayacı oku
        let counter = db
            .get(b"__counter")
            .map(|v| v.map(|iv| u64::from_be_bytes(iv.as_ref().try_into().unwrap())).unwrap_or(0))
            .unwrap_or(0);
        Ok(Self { db, counter })
    }

    pub fn count(&self) -> u64 {
        self.counter
    }
}

impl EventStore for SledEventStore {
    fn append(&mut self, event: &DomainEvent) {
        let key = self.counter.to_be_bytes();
        let val = serde_json::to_vec(event).expect("serialize domain event");
        let _ = self.db.insert(key, val);
        self.counter += 1;
        let _ = self.db.insert(b"__counter", &self.counter.to_be_bytes());
    }

    fn replay(&self) -> Vec<DomainEvent> {
        let mut events: Vec<(u64, Vec<u8>)> = Vec::new();
        for item in self.db.iter() {
            if let Ok((k, v)) = item {
                if k.as_ref() == b"__counter" {
                    continue;
                }
                if let Ok(arr) = <[u8; 8]>::try_from(k.as_ref()) {
                    let u = u64::from_be_bytes(arr);
                    events.push((u, v.to_vec()));
                }
            }
        }
        events.sort_by_key(|(u, _)| *u);
        events
            .into_iter()
            .filter_map(|(_, v)| serde_json::from_slice(&v).ok())
            .collect()
    }
}

/// Event replay'i ile actor state'ini yeniden inşa eder.
/// Geri dönüş: (başlangıç bakiyesi, uygulanan nakit deltaları ve pozisyon fill'leri)
#[derive(Debug, Default)]
pub struct ReplayResult {
    pub events: Vec<DomainEvent>,
}

pub fn load_snapshot_path() -> String {
    std::env::var("PAPER_SLED_PATH").unwrap_or_else(|_| "./paper_wal".to_string())
}

pub fn open_wal_store() -> Arc<std::sync::Mutex<Box<dyn EventStore>>> {
    let path = load_snapshot_path();
    match SledEventStore::open(&path) {
        Ok(store) => {
            tracing::info!("Sled WAL açıldı: {} ({} event)", path, store.count());
            Arc::new(std::sync::Mutex::new(Box::new(store) as Box<dyn EventStore>))
        }
        Err(e) => {
            tracing::warn!("Sled açılamadı ({}), in-memory store kullanılıyor: {}", path, e);
            Arc::new(std::sync::Mutex::new(Box::new(InMemoryEventStore::new()) as Box<dyn EventStore>))
        }
    }
}
