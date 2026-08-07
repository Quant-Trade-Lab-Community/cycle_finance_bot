use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, ftruncate, mmap, O_CREAT, O_RDWR, PROT_READ, PROT_WRITE, MAP_SHARED};
use std::os::unix::io::FromRawFd;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct MarketDataSlot {
    pub seq: u64,
    pub len: u16,
    pub data: [u8; 246], // Total 256 bytes (8 + 2 + 246)
}

#[repr(C)]
pub struct SharedHeader {
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
        Self::with_name("/demir_yumruk_ring", capacity)
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

            // Set the size of the shared memory object
            if ftruncate(fd, total_size as i64) < 0 {
                panic!("Failed to ftruncate");
            }

            // Map the shared memory into our process space
            let mut file = std::fs::File::from_raw_fd(fd);
            let mut mmap = memmap2::MmapOptions::new()
                .len(total_size)
                .map_mut(&file)
                .expect("Failed to mmap shared memory");

            let header = mmap.as_mut_ptr() as *mut SharedHeader;
            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut MarketDataSlot;

            // Only initialize if we are the ones who created it (head == 0).
            // A more robust way is to use a magic number, but for this HFT demo, 
            // if head is 0 we assume it's fresh.
            if (*header).capacity == 0 {
                (*header).head.store(0, Ordering::SeqCst);
                (*header).tail.store(0, Ordering::SeqCst);
                (*header).capacity = capacity as u64;
                
                // Zero out the slots just in case
                ptr::write_bytes(slots, 0, capacity);
            }

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
            
            let len = if data.len() > 246 { 246 } else { data.len() as u16 };

            let slot_ptr = self.slots.add(index);
            (*slot_ptr).seq = seq;
            (*slot_ptr).len = len;
            ptr::copy_nonoverlapping(data.as_ptr(), (*slot_ptr).data.as_mut_ptr(), len as usize);

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
            *slot_ptr
        };

        // Generational check: if the sequence doesn't match, we've been overwritten by the producer
        if slot.seq == seq {
            Some(slot)
        } else {
            None
        }
    }
}
