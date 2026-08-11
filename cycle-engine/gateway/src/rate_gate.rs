//! Ortak API rate kapısı — prosesler arası paylaşımlı bellek (POSIX shm).
//!
//! Bağımsız akış prosesleri Binance API limitlerine takılmamak için bu
//! kapıdan token alarak bağlantı kurar. Token bucket: kapasite
//! `CYCLE_GATE_CAPACITY`, dolum hızı `CYCLE_GATE_RATE` (token/sn).
//! Kapı `/dev/shm/cycle_finance_api_gate` üzerinde olduğundan tüm akışlar
//! aynı bütçeyi paylaşır; akışlar birbirinden bağımsız kalır.

use std::ffi::CString;
use std::os::unix::io::FromRawFd;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use libc::{shm_open, O_CREAT, O_RDWR};
use memmap2::{MmapMut, MmapOptions};

const GATE_MAGIC: u64 = 0xD3F0000000000005;
const DEFAULT_SHM: &str = "/cycle_finance_api_gate";
const DEFAULT_CAPACITY: u64 = 8;
const DEFAULT_RATE_PER_SEC: u64 = 4;

#[repr(C)]
struct GateHeader {
    magic: AtomicU64,
    max_tokens: AtomicU64,
    tokens: AtomicU64,
    /// Dolum hızı × 1000 (küsüratsız sabit nokta; token/sn).
    refill_per_sec_x1000: AtomicU64,
    last_refill: AtomicU64, // unix nanos
}

pub struct RateGate {
    // mmap canlı kaldıkça shm açık kalır; drop edilirse bellek unmaps.
    _mmap: MmapMut,
    header: *mut GateHeader,
}

// Prosesler arası paylaşımlı bellek — Send/Sync gereklidir.
unsafe impl Send for RateGate {}
unsafe impl Sync for RateGate {}

impl RateGate {
    /// Varsayılan kapıyı açar/açar (`/cycle_finance_api_gate`).
    pub fn open_default() -> Self {
        Self::open(DEFAULT_SHM)
    }

    /// Belirtilen POSIX shm nesnesi üzerinde kapıyı oluşturur/açar.
    /// İlk oluşturan proses başlatır; diğerleri mevcut başlığı kullanır.
    pub fn open(name: &str) -> Self {
        let capacity = env_u64("CYCLE_GATE_CAPACITY", DEFAULT_CAPACITY);
        let rate = env_u64("CYCLE_GATE_RATE", DEFAULT_RATE_PER_SEC);
        let cname = CString::new(name).unwrap();

        let header_size = std::mem::size_of::<GateHeader>();
        let header_aligned = (header_size + 63) & !63;

        unsafe {
            let fd = shm_open(cname.as_ptr(), O_CREAT | O_RDWR, 0o666);
            assert!(fd >= 0, "rate gate: shm_open başarısız");
            let file = std::fs::File::from_raw_fd(fd);

            let existing = file.metadata().map(|m| m.len()).unwrap_or(0);
            let is_fresh = existing == 0;
            if is_fresh {
                file.set_len(header_aligned as u64).expect("rate gate: ftruncate");
            }

            let map_len = if is_fresh { header_aligned } else { existing as usize };
            let mut mmap = MmapOptions::new()
                .len(map_len)
                .map_mut(&file)
                .expect("rate gate: mmap başarısız");

            let header = mmap.as_mut_ptr() as *mut GateHeader;

            // Eski/satnik shm varsa (magic yok) yeniden ilklendir.
            if (*header).magic.load(Ordering::Relaxed) != GATE_MAGIC {
                file.set_len(header_aligned as u64).expect("rate gate: ftruncate (reinit)");
                let mut mmap = MmapOptions::new()
                    .len(header_aligned)
                    .map_mut(&file)
                    .expect("rate gate: mmap (reinit)");
                let header = mmap.as_mut_ptr() as *mut GateHeader;
                (*header).magic.store(GATE_MAGIC, Ordering::Relaxed);
                (*header).max_tokens.store(capacity, Ordering::Relaxed);
                (*header).tokens.store(capacity, Ordering::Relaxed);
                (*header).refill_per_sec_x1000.store(rate * 1000, Ordering::Relaxed);
                (*header).last_refill.store(now_ns(), Ordering::Relaxed);
                return Self { _mmap: mmap, header };
            }

            Self { _mmap: mmap, header }
        }
    }

    /// Bloklamadan bir token almayı dener. Token yoksa `false`.
    pub fn try_acquire(&self) -> bool {
        self.refill();
        unsafe {
            let t = (*self.header).tokens.load(Ordering::Relaxed);
            if t == 0 {
                return false;
            }
            (*self.header)
                .tokens
                .compare_exchange(t, t - 1, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
        }
    }

    /// Token gelene kadar (en fazla `timeout`) bekler.
    pub fn acquire(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if self.try_acquire() {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    /// Token bucket dolumu — yalnızca `last_refill` CAS'ini kazanan proses yapar.
    /// CAS tabanlı olduğu için eşzamanlı erişim güvenlidir.
    fn refill(&self) {
        let now = now_ns();
        unsafe {
            let last = (*self.header).last_refill.load(Ordering::Relaxed);
            if now <= last {
                return;
            }
            if (*self.header)
                .last_refill
                .compare_exchange(last, now, Ordering::SeqCst, Ordering::Relaxed)
                .is_err()
            {
                return; // başka bir proses dolumu yaptı
            }

            let rate = (*self.header).refill_per_sec_x1000.load(Ordering::Relaxed);
            let max = (*self.header).max_tokens.load(Ordering::Relaxed);
            let elapsed_ns = now - last;
            let add = (elapsed_ns as u128 * rate as u128) / 1_000_000_000_000;
            if add == 0 {
                return;
            }

            let mut t = (*self.header).tokens.load(Ordering::Relaxed);
            loop {
                let new_t = (t + add as u64).min(max);
                if (*self.header)
                    .tokens
                    .compare_exchange(t, new_t, Ordering::SeqCst, Ordering::Relaxed)
                    .is_ok()
                {
                    break;
                }
                t = (*self.header).tokens.load(Ordering::Relaxed);
            }
        }
    }
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn now_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}
