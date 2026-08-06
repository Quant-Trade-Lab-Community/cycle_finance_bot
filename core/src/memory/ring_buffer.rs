use std::sync::atomic::{AtomicU64, Ordering};
use crate::hal::memory::allocate_huge_buffer;
use std::ptr;

// 256 byte slot ensures it aligns nicely with cache lines (4x64 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct MarketDataSlot {
    pub seq: u64,
    pub len: u16,
    pub data: [u8; 246], // Total 256 bytes (8 + 2 + 246)
}

pub struct GenerationalRingBuffer {
    buffer: Vec<MarketDataSlot>, // Pre-allocated vector
    capacity: usize,
    head: AtomicU64,
    tail: AtomicU64,
}

impl GenerationalRingBuffer {
    pub fn new(capacity: usize) -> Self {
        // We use our HAL allocator (currently just falls back to Vec with page touching)
        // But we cast it into our specific Slot structures.
        // For simplicity in Rust without unsafe transmutes of the whole buffer, 
        // we'll just initialize a Vec of MarketDataSlot directly.
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(MarketDataSlot { seq: 0, len: 0, data: [0; 246] });
        }

        Self {
            buffer,
            capacity,
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub fn push(&self, data: &[u8]) {
        let seq = self.head.load(Ordering::Relaxed);
        let index = (seq % self.capacity as u64) as usize;
        
        let len = if data.len() > 246 { 246 } else { data.len() as u16 };

        unsafe {
            let slot_ptr = self.buffer.as_ptr().add(index) as *mut MarketDataSlot;
            (*slot_ptr).seq = seq;
            (*slot_ptr).len = len;
            ptr::copy_nonoverlapping(data.as_ptr(), (*slot_ptr).data.as_mut_ptr(), len as usize);
        }

        // Release order ensures all writes to the slot are visible before head is incremented
        self.head.store(seq + 1, Ordering::Release);
    }

    #[inline(always)]
    pub fn get_head(&self) -> u64 {
        self.head.load(Ordering::Acquire)
    }

    #[inline(always)]
    pub fn read_slot(&self, seq: u64) -> Option<MarketDataSlot> {
        let index = (seq % self.capacity as u64) as usize;
        
        // Unsafe block for raw pointer reading without bounds checking overhead in hot path
        let slot = unsafe {
            let slot_ptr = self.buffer.as_ptr().add(index);
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
