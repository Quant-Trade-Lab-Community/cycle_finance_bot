#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum EventType {
    Trade { price: f64, quantity: f64, timestamp: u64 },
    Orderbook { 
        bids: [(f64, f64); 20], 
        asks: [(f64, f64); 20] 
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct OwnedEvent {
    pub symbol: [u8; 16],
    pub payload: EventType,
}

impl OwnedEvent {
    #[inline(always)]
    pub fn new_trade(sym: &str, price: f64, quantity: f64, timestamp: u64) -> Self {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        Self {
            symbol,
            payload: EventType::Trade { price, quantity, timestamp },
        }
    }

    #[inline(always)]
    pub fn new_orderbook(sym: &str, bids: [(f64, f64); 20], asks: [(f64, f64); 20]) -> Self {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        Self {
            symbol,
            payload: EventType::Orderbook { bids, asks },
        }
    }
}

pub struct RingBuffer {
    buffer: Box<[OwnedEvent]>,
    write_index: usize,
    capacity: usize,
    is_full: bool,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        let size_mb = (capacity * std::mem::size_of::<OwnedEvent>()) / 1024 / 1024;
        println!("Demir Yumruk: Allocating {} MB Ring Buffer ({} elements)...", size_mb, capacity);
        
        let buffer = vec![OwnedEvent {
            symbol: [0; 16],
            payload: EventType::Trade { price: 0.0, quantity: 0.0, timestamp: 0 },
        }; capacity].into_boxed_slice();
        
        Self {
            buffer,
            write_index: 0,
            capacity,
            is_full: false,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, event: OwnedEvent) -> Option<OwnedEvent> {
        let evicted = if self.is_full {
            Some(self.buffer[self.write_index])
        } else {
            None
        };
        
        // Zero-allocation, O(1) override
        self.buffer[self.write_index] = event;
        self.write_index += 1;
        
        if self.write_index >= self.capacity {
            self.write_index = 0;
            self.is_full = true;
        }
        
        evicted
    }
    
    pub fn write_index(&self) -> usize {
        self.write_index
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
