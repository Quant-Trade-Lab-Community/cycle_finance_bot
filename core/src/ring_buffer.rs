#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum EventType {
    Trade { price: f64, quantity: f64, timestamp: u64, is_buyer_maker: bool },
    Orderbook { 
        bids: [(f64, f64); 20], 
        asks: [(f64, f64); 20] 
    },
    Liquidation { side: u8, price: f64, quantity: f64, timestamp: u64 },
    FundingRate { mark_price: f64, funding_rate: f64, next_funding_time: u64 },
    BookTicker { best_bid_price: f64, best_bid_qty: f64, best_ask_price: f64, best_ask_qty: f64 },
    OpenInterest { open_interest: f64, timestamp: u64 },
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct OwnedEvent {
    pub symbol: [u8; 16],
    pub payload: EventType,
}

impl OwnedEvent {
    #[inline(always)]
    pub fn new_trade(sym: &str, price: f64, quantity: f64, timestamp: u64, is_buyer_maker: bool) -> Self {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        Self {
            symbol,
            payload: EventType::Trade { price, quantity, timestamp, is_buyer_maker },
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

    #[inline(always)]
    pub fn new_liquidation(sym: &str, side: u8, price: f64, quantity: f64, timestamp: u64) -> Self {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        Self {
            symbol,
            payload: EventType::Liquidation { side, price, quantity, timestamp },
        }
    }

    #[inline(always)]
    pub fn new_funding_rate(sym: &str, mark_price: f64, funding_rate: f64, next_funding_time: u64) -> Self {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        Self {
            symbol,
            payload: EventType::FundingRate { mark_price, funding_rate, next_funding_time },
        }
    }

    #[inline(always)]
    pub fn new_bookticker(sym: &str, best_bid_price: f64, best_bid_qty: f64, best_ask_price: f64, best_ask_qty: f64) -> Self {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        Self {
            symbol,
            payload: EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty },
        }
    }

    #[inline(always)]
    pub fn new_open_interest(sym: &str, open_interest: f64, timestamp: u64) -> Self {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        Self {
            symbol,
            payload: EventType::OpenInterest { open_interest, timestamp },
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
            payload: EventType::Trade { price: 0.0, quantity: 0.0, timestamp: 0, is_buyer_maker: false },
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
