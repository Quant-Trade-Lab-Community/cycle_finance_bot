//! Büyük-slot paylaşımlı bellek ring buffer — indikatör/OHLCV sonuçları için.
//!
//! `GenerationalRingBuffer` (702B slot) indikatör serileri için çok küçüktür;
//! bu ring büyük binary blokları (örn. bir isteğin tüm OHLCV + indikatör
//! çıktısı) tek slot'ta taşır. Torn-read koruması aynıdır: seq en son yazılır,
//! okuyucu yarım slot görmez.
//!
//! Üretici: calc-ind servisi. Tüketici: calc_ind::client (istek atan servis).

use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, O_CREAT, O_RDWR};
use std::os::unix::io::FromRawFd;

/// Paylaşımlı hafızanın ilk oluşturulup oluşturulmadığını doğrulayan magic.
pub(crate) const CALC_RING_MAGIC: u64 = 0xD3F0000000000003;

/// Varsayılan tek slot boyutu (1 MB) — bir isteğin tüm sonucunu taşıyacak kadar.
pub const CALC_SLOT_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct CalcSlot {
    pub seq: u64,
    pub len: u32,
    pub data: [u8; 1024 * 1024],
}

#[repr(C)]
pub struct CalcSharedHeader {
    pub magic: AtomicU64,
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
}

pub struct CalcRingBuffer {
    mmap: memmap2::MmapMut,
    header: *mut CalcSharedHeader,
    slots: *mut CalcSlot,
    capacity: usize,
}

unsafe impl Send for CalcRingBuffer {}
unsafe impl Sync for CalcRingBuffer {}

impl CalcRingBuffer {
    /// Varsayılan isimle açar: `/cycle_finance_calc`
    pub fn new(capacity: usize) -> Self {
        Self::with_name("/cycle_finance_calc", capacity)
    }

    /// Belirtilen POSIX shm nesnesi üzerinde büyük-slot ring oluşturur/açar.
    pub fn with_name(shm_name: &str, capacity: usize) -> Self {
        let name = CString::new(shm_name).unwrap();

        let header_size = std::mem::size_of::<CalcSharedHeader>();
        let header_aligned = (header_size + 63) & !63;
        let slot_size = std::mem::size_of::<CalcSlot>();
        let total_size = header_aligned + (capacity * slot_size);

        unsafe {
            let fd = shm_open(name.as_ptr(), O_CREAT | O_RDWR, 0o666);
            if fd < 0 {
                panic!("Failed to shm_open for calc ring");
            }

            let mut file = std::fs::File::from_raw_fd(fd);

            // ftruncate'i YALNIZCA ilk oluşturan yapar.
            let existing = file.metadata().map(|m| m.len()).unwrap_or(0);
            let is_fresh = existing == 0;

            if is_fresh {
                file.set_len(total_size as u64).expect("ftruncate for calc ring");
            }

            let map_len = if is_fresh { total_size } else { existing as usize };

            let mut mmap = memmap2::MmapOptions::new()
                .len(map_len)
                .map_mut(&file)
                .expect("Failed to mmap calc shared memory");

            let header = mmap.as_mut_ptr() as *mut CalcSharedHeader;

            // Eski/satnik shm varsa (magic yok) yeniden ilklendir.
            if (*header).magic.load(Ordering::Relaxed) != CALC_RING_MAGIC {
                file.set_len(total_size as u64).expect("ftruncate (reinit)");
                let mut mmap = memmap2::MmapOptions::new()
                    .len(total_size)
                    .map_mut(&file)
                    .expect("Failed to mmap calc shared memory (reinit)");
                let header = mmap.as_mut_ptr() as *mut CalcSharedHeader;
                let slots = mmap.as_mut_ptr().add(header_aligned) as *mut CalcSlot;

                (*header).magic.store(CALC_RING_MAGIC, Ordering::Relaxed);
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

            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut CalcSlot;
            let real_cap = (*header).capacity as usize;

            Self {
                mmap,
                header,
                slots,
                capacity: real_cap,
            }
        }
    }

    /// Tek slot'a veri yazar. `data.len() > CALC_SLOT_SIZE` ise kesilir.
    #[inline(always)]
    pub fn push(&self, data: &[u8]) {
        unsafe {
            let seq = (*self.header).head.load(Ordering::Relaxed);
            let index = (seq % self.capacity as u64) as usize;

            let len = data.len().min(CALC_SLOT_SIZE) as u32;
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
    pub fn read_slot(&self, seq: u64) -> Option<CalcSlot> {
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
