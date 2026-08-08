use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, O_CREAT, O_RDWR};
use std::os::unix::io::FromRawFd;

/// Paylaşımlı hafızanın ilk oluşturulup oluşturulmadığını doğrulayan magic.
const RING_MAGIC: u64 = 0xD3F0000000000001;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct MarketDataSlot {
    pub seq: u64,
    pub len: u16,
    pub data: [u8; 702], // Total 768 bytes — en büyük wire frame (Depth20 = 659B) sığar
}

impl MarketDataSlot {
    pub const DATA_LEN: usize = 702;
}


#[repr(C)]
pub struct SharedHeader {
    pub magic: AtomicU64,
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
}

pub struct GenerationalRingBuffer {
    // Keep mmap alive. If it drops, memory unmaps.
    mmap: memmap2::MmapMut,
    header: *mut SharedHeader,
    slots: *mut MarketDataSlot,
    capacity: usize,
}

// Ensure Send/Sync for crossbeam threading
unsafe impl Send for GenerationalRingBuffer {}
unsafe impl Sync for GenerationalRingBuffer {}

impl GenerationalRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self::with_name("/cycle_finance_ring", capacity)
    }

    /// Belirtilen POSIX shm nesnesi üzerinde ring buffer oluşturur/açar.
    /// Farklı servisler farklı isim kullanabilir (örn. price-feed).
    pub fn with_name(shm_name: &str, capacity: usize) -> Self {
        let name = CString::new(shm_name).unwrap();

        let header_size = std::mem::size_of::<SharedHeader>();
        // Align to 64 bytes
        let header_aligned = (header_size + 63) & !63;

        let slot_size = std::mem::size_of::<MarketDataSlot>();
        let total_size = header_aligned + (capacity * slot_size);

        unsafe {
            // Create or open the POSIX shared memory object
            let fd = shm_open(name.as_ptr(), O_CREAT | O_RDWR, 0o666);
            if fd < 0 {
                panic!("Failed to shm_open");
            }

            let mut file = std::fs::File::from_raw_fd(fd);

            // Yeni mi yoksa mevcut mu? — ftruncate'i YALNIZCA ilk oluşturan yapar.
            // Aksi halde farklı capacity ile açan bir proses, üreticinin
            // paylaşımlı hafızasını altından yeniden boyutlandırır.
            let existing = file.metadata().map(|m| m.len()).unwrap_or(0);
            let is_fresh = existing == 0;

            if is_fresh {
                file.set_len(total_size as u64).expect("ftruncate");
            }

            let map_len = if is_fresh {
                total_size
            } else {
                existing as usize
            };

            let mut mmap = memmap2::MmapOptions::new()
                .len(map_len)
                .map_mut(&file)
                .expect("Failed to mmap shared memory");

            let header = mmap.as_mut_ptr() as *mut SharedHeader;

            // Eski format/satnik shm varsa (magic yok) yeniden ilklendir.
            if (*header).magic.load(Ordering::Relaxed) != RING_MAGIC {
                file.set_len(total_size as u64).expect("ftruncate (reinit)");
                let mut mmap = memmap2::MmapOptions::new()
                    .len(total_size)
                    .map_mut(&file)
                    .expect("Failed to mmap shared memory (reinit)");
                let header = mmap.as_mut_ptr() as *mut SharedHeader;
                let slots = mmap.as_mut_ptr().add(header_aligned) as *mut MarketDataSlot;

                (*header).magic.store(RING_MAGIC, Ordering::Relaxed);
                (*header).head.store(0, Ordering::SeqCst);
                (*header).tail.store(0, Ordering::SeqCst);
                (*header).capacity = capacity as u64;

                // Zero out the slots just in case
                ptr::write_bytes(slots, 0, capacity);

                let real_cap = (*header).capacity as usize;
                return Self {
                    mmap,
                    header,
                    slots,
                    capacity: real_cap,
                };
            }

            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut MarketDataSlot;
            let real_cap = (*header).capacity as usize;

            Self {
                mmap,
                header,
                slots,
                capacity: real_cap,
            }
        }
    }

    #[inline(always)]
    pub fn push(&self, data: &[u8]) {
        unsafe {
            let seq = (*self.header).head.load(Ordering::Relaxed);
            let index = (seq % self.capacity as u64) as usize;

            let len = if data.len() > MarketDataSlot::DATA_LEN {
                MarketDataSlot::DATA_LEN as u16
            } else {
                data.len() as u16
            };

            let slot_ptr = self.slots.add(index);
            // Önce veriyi ve len'i yaz, seq en sona kalsın ki okuyucu
            // yarım/tutarsız slot okumasın (torn-read koruması).
            (*slot_ptr).len = len;
            ptr::copy_nonoverlapping(data.as_ptr(), (*slot_ptr).data.as_mut_ptr(), len as usize);
            std::sync::atomic::fence(Ordering::Release);
            (*slot_ptr).seq = seq;

            // Release order ensures all writes to the slot are visible before head is incremented
            (*self.header).head.store(seq + 1, Ordering::Release);
        }
    }

    #[inline(always)]
    pub fn get_head(&self) -> u64 {
        unsafe {
            (*self.header).head.load(Ordering::Acquire)
        }
    }

    #[inline(always)]
    pub fn read_slot(&self, seq: u64) -> Option<MarketDataSlot> {
        let index = (seq % self.capacity as u64) as usize;

        let slot = unsafe {
            let slot_ptr = self.slots.add(index);
            // İlk oku: seq uyuyorsa veri tam yazılmış demektir (push seq'i en son yazar).
            let s = *slot_ptr;
            if s.seq == seq {
                // Çift kontrol: kopyalama sırasında üretici aynı slotu ezmesin diye.
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

        // Generational check: if the sequence doesn't match, we've been overwritten by the producer
        slot
    }
}
