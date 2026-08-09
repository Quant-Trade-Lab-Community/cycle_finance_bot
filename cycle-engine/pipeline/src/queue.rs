use flume::{Receiver, Sender};

/// Bounded lock-free queue dispatcher for high-throughput messaging.
/// Sınırlı kuyruk → üretici geri basınç alır (RAM taşması önlenir).
pub struct LockFreeDispatcher {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
}

const QUEUE_CAPACITY: usize = 262_144;

impl LockFreeDispatcher {
    pub fn new() -> Self {
        let (tx, rx) = flume::bounded(QUEUE_CAPACITY);
        Self { tx, rx }
    }

    #[inline(always)]
    pub fn producer(&self) -> Sender<Vec<u8>> {
        self.tx.clone()
    }

    #[inline(always)]
    pub fn consumer(&self) -> Receiver<Vec<u8>> {
        self.rx.clone()
    }
}
