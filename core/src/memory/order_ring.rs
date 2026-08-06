use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, ftruncate, mmap, O_CREAT, O_RDWR, PROT_READ, PROT_WRITE, MAP_SHARED};
use std::os::unix::io::FromRawFd;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IpcOrderSide {
    Buy = 0,
    Sell = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IpcOrderType {
    Limit = 0,
    Market = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct OrderSlot {
    pub seq: u64,
    pub symbol: [u8; 16], // Max 16 chars like "BTCUSDT"
    pub side: IpcOrderSide,
    pub order_type: IpcOrderType,
    pub quantity: f64,
    pub price: f64,
}

#[repr(C)]
pub struct OrderSharedHeader {
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
}

pub struct OrderRingBuffer {
    mmap: memmap2::MmapMut,
    header: *mut OrderSharedHeader,
    slots: *mut OrderSlot,
    capacity: usize,
}

unsafe impl Send for OrderRingBuffer {}
unsafe impl Sync for OrderRingBuffer {}

impl OrderRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let name = CString::new("/demir_yumruk_orders").unwrap();
        
        let header_size = std::mem::size_of::<OrderSharedHeader>();
        let header_aligned = (header_size + 63) & !63;
        
        let slot_size = std::mem::size_of::<OrderSlot>();
        let total_size = header_aligned + (capacity * slot_size);

        unsafe {
            let fd = shm_open(name.as_ptr(), O_CREAT | O_RDWR, 0o666);
            if fd < 0 {
                panic!("Failed to shm_open for orders");
            }

            if ftruncate(fd, total_size as i64) < 0 {
                panic!("Failed to ftruncate for orders");
            }

            let file = std::fs::File::from_raw_fd(fd);
            let mut mmap = memmap2::MmapOptions::new()
                .len(total_size)
                .map_mut(&file)
                .expect("Failed to mmap order shared memory");

            let header = mmap.as_mut_ptr() as *mut OrderSharedHeader;
            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut OrderSlot;

            if (*header).capacity == 0 {
                (*header).head.store(0, Ordering::SeqCst);
                (*header).tail.store(0, Ordering::SeqCst);
                (*header).capacity = capacity as u64;
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
    pub fn push(&self, symbol: &[u8], side: IpcOrderSide, order_type: IpcOrderType, quantity: f64, price: f64) {
        unsafe {
            let seq = (*self.header).head.load(Ordering::Relaxed);
            let index = (seq % self.capacity as u64) as usize;
            
            let slot_ptr = self.slots.add(index);
            (*slot_ptr).seq = seq;
            (*slot_ptr).side = side;
            (*slot_ptr).order_type = order_type;
            (*slot_ptr).quantity = quantity;
            (*slot_ptr).price = price;
            
            let mut sym_buf = [0u8; 16];
            let len = symbol.len().min(16);
            sym_buf[..len].copy_from_slice(&symbol[..len]);
            (*slot_ptr).symbol = sym_buf;

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
    pub fn read_slot(&self, seq: u64) -> Option<OrderSlot> {
        let index = (seq % self.capacity as u64) as usize;
        let slot = unsafe { *self.slots.add(index) };
        if slot.seq == seq {
            Some(slot)
        } else {
            None
        }
    }
}
