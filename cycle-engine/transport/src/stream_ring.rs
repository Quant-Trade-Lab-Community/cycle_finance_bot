//! Büyük-slot paylaşımlı bellek ring buffer — canlı OHLCV mum akışı için.
//!
//! `GenerationalRingBuffer` (702B slot) tek bir mumu ve stream meta bilgisini
//! taşıyacak kadar büyük değildir; bu ring canlı kapanan/oluşan mumları tek
//! slot'ta binary olarak taşır. Torn-read koruması aynıdır: seq en son yazılır,
//! okuyucu yarım slot görmez.
//!
//! Üretici: stream-ohlcv servisi. Tüketici: stream_ohlcv::client (istek atan servis).
//!
//! Slot düzeni (StreamSlot, sabit 4096B):
//!   [0..8)   seq (torn-read koruması, en son yazılır)
//!   [8..12)  len (payload bayt uzunluğu)
//!   [12..)   data — stream_ohlcv::codec ile binary kodlanmış mum

use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, O_CREAT, O_RDWR};
use std::os::unix::io::FromRawFd;

/// Paylaşımlı hafızanın ilk oluşturulup oluşturulmadığını doğrulayan magic.
pub(crate) const STREAM_RING_MAGIC: u64 = 0xD3F0000000000004;

/// Tek slot boyutu (4 KB) — bir mum (binary codec) rahatlıkla sığar.
pub const STREAM_SLOT_SIZE: usize = 4096;

/// Varsayılan slot sayısı — dairesel akış, eski slotlar üzerine yazılır.
pub const STREAM_DEFAULT_CAPACITY: usize = 8192;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct StreamSlot {
    pub seq: u64,
    pub len: u32,
    pub data: [u8; STREAM_SLOT_SIZE],
}

#[repr(C)]
pub struct StreamSharedHeader {
    pub magic: AtomicU64,
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
}

pub struct StreamRingBuffer {
    mmap: memmap2::MmapMut,
    header: *mut StreamSharedHeader,
    slots: *mut StreamSlot,
    capacity: usize,
}

unsafe impl Send for StreamRingBuffer {}
unsafe impl Sync for StreamRingBuffer {}

impl StreamRingBuffer {
    /// Varsayılan isimle açar: `/cycle_finance_stream_ohlcv`
    pub fn new(capacity: usize) -> Self {
        Self::with_name("/cycle_finance_stream_ohlcv", capacity)
    }

    /// Belirtilen POSIX shm nesnesi üzerinde büyük-slot ring oluşturur/açar.
    pub fn with_name(shm_name: &str, capacity: usize) -> Self {
        let name = CString::new(shm_name).unwrap();

        let header_size = std::mem::size_of::<StreamSharedHeader>();
        let header_aligned = (header_size + 63) & !63;
        let slot_size = std::mem::size_of::<StreamSlot>();
        let total_size = header_aligned + (capacity * slot_size);

        unsafe {
            let fd = shm_open(name.as_ptr(), O_CREAT | O_RDWR, 0o666);
            if fd < 0 {
                panic!("Failed to shm_open for stream ring");
            }

            let mut file = std::fs::File::from_raw_fd(fd);

            // ftruncate'i YALNIZCA ilk oluşturan yapar.
            let existing = file.metadata().map(|m| m.len()).unwrap_or(0);
            let is_fresh = existing == 0;

            if is_fresh {
                file.set_len(total_size as u64).expect("ftruncate for stream ring");
            }

            let map_len = if is_fresh { total_size } else { existing as usize };

            let mut mmap = memmap2::MmapOptions::new()
                .len(map_len)
                .map_mut(&file)
                .expect("Failed to mmap stream shared memory");

            let header = mmap.as_mut_ptr() as *mut StreamSharedHeader;

            // Eski/satnik shm varsa (magic yok) yeniden ilklendir.
            if (*header).magic.load(Ordering::Relaxed) != STREAM_RING_MAGIC {
                file.set_len(total_size as u64).expect("ftruncate (reinit)");
                let mut mmap = memmap2::MmapOptions::new()
                    .len(total_size)
                    .map_mut(&file)
                    .expect("Failed to mmap stream shared memory (reinit)");
                let header = mmap.as_mut_ptr() as *mut StreamSharedHeader;
                let slots = mmap.as_mut_ptr().add(header_aligned) as *mut StreamSlot;

                (*header).magic.store(STREAM_RING_MAGIC, Ordering::Relaxed);
                (*header).head.store(0, Ordering::SeqCst);
                (*header).tail.store(0, Ordering::SeqCst);
                (*header).capacity = capacity as u64;
                ptr::write_bytes(slots, 0, capacity);

                let real_cap = (*header).capacity as usize;
                return Self {
                    mmap,
                    header,
                    slots,
                    capacity: real_cap,
                };
            }

            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut StreamSlot;
            let real_cap = (*header).capacity as usize;

            Self {
                mmap,
                header,
                slots,
                capacity: real_cap,
            }
        }
    }

    /// Tek slot'a veri yazar. `data.len() > STREAM_SLOT_SIZE` ise kesilir.
    #[inline(always)]
    pub fn push(&self, data: &[u8]) {
        unsafe {
            let seq = (*self.header).head.load(Ordering::Relaxed);
            let index = (seq % self.capacity as u64) as usize;

            let len = data.len().min(STREAM_SLOT_SIZE) as u32;
            let slot_ptr = self.slots.add(index);

            (*slot_ptr).len = len;
            ptr::copy_nonoverlapping(data.as_ptr(), (*slot_ptr).data.as_mut_ptr(), len as usize);
            std::sync::atomic::fence(Ordering::Release);
            (*slot_ptr).seq = seq;

            (*self.header).head.store(seq + 1, Ordering::Release);
        }
    }

    /// Üretici başını (head) okur — tüketici buradan başlar.
    #[inline(always)]
    pub fn get_head(&self) -> u64 {
        unsafe { (*self.header).head.load(Ordering::Acquire) }
    }

    /// Slot'u veri parçası olarak okur (torn-read korumalı).
    #[inline(always)]
    pub fn read_slot(&self, seq: u64) -> Option<StreamSlot> {
        let index = (seq % self.capacity as u64) as usize;
        let slot = unsafe {
            let slot_ptr = self.slots.add(index);
            let s = *slot_ptr;
            if s.seq == seq {
                let again = *slot_ptr;
                if again.seq == seq {
                    Some(again)
                } else {
                    None
                }
            } else {
                None
            }
        };
        slot
    }
}
