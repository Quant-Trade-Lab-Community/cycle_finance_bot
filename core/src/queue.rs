use flume::{Receiver, Sender};

/// Lock-free queue dispatcher for high-throughput messaging.
/// Uses unbounded flume channel.
pub struct LockFreeDispatcher {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
}

impl LockFreeDispatcher {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();
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
