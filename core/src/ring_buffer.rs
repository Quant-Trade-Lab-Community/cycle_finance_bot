use rust_decimal::Decimal;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum EventType {
    Trade { price: Decimal, quantity: Decimal, timestamp: u64, is_buyer_maker: bool },
    Orderbook {
        bids: [(Decimal, Decimal); 20],
        asks: [(Decimal, Decimal); 20]
    },
    Liquidation { side: u8, price: Decimal, quantity: Decimal, timestamp: u64 },
    FundingRate { mark_price: Decimal, index_price: Decimal, funding_rate: Decimal, next_funding_time: u64 },
    BookTicker { best_bid_price: Decimal, best_bid_qty: Decimal, best_ask_price: Decimal, best_ask_qty: Decimal },
    OpenInterest { open_interest: Decimal, timestamp: u64 },
}

impl std::fmt::Debug for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
                f.debug_struct("Trade")
                    .field("price", price)
                    .field("quantity", quantity)
                    .field("timestamp", timestamp)
                    .field("is_buyer_maker", is_buyer_maker)
                    .finish()
            }
            EventType::Orderbook { bids, asks } => {
                f.debug_struct("Orderbook").field("bids", bids).field("asks", asks).finish()
            }
            EventType::Liquidation { side, price, quantity, timestamp } => {
                f.debug_struct("Liquidation")
                    .field("side", side)
                    .field("price", price)
                    .field("quantity", quantity)
                    .field("timestamp", timestamp)
                    .finish()
            }
            EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time } => {
                f.debug_struct("FundingRate")
                    .field("mark_price", mark_price)
                    .field("index_price", index_price)
                    .field("funding_rate", funding_rate)
                    .field("next_funding_time", next_funding_time)
                    .finish()
            }
            EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty } => {
                f.debug_struct("BookTicker")
                    .field("best_bid_price", best_bid_price)
                    .field("best_bid_qty", best_bid_qty)
                    .field("best_ask_price", best_ask_price)
                    .field("best_ask_qty", best_ask_qty)
                    .finish()
            }
            EventType::OpenInterest { open_interest, timestamp } => {
                f.debug_struct("OpenInterest")
                    .field("open_interest", open_interest)
                    .field("timestamp", timestamp)
                    .finish()
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct OwnedEvent {
    pub symbol: [u8; 16],
    pub payload: EventType,
}

impl std::fmt::Debug for OwnedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedEvent")
            .field("symbol", &self.symbol)
            .field("payload", &self.payload)
            .finish()
    }
}

impl OwnedEvent {
    #[inline(always)]
    pub fn new_trade(sym: &str, price: Decimal, quantity: Decimal, timestamp: u64, is_buyer_maker: bool) -> Self {
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
    pub fn new_orderbook(sym: &str, bids: [(Decimal, Decimal); 20], asks: [(Decimal, Decimal); 20]) -> Self {
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
    pub fn new_liquidation(sym: &str, side: u8, price: Decimal, quantity: Decimal, timestamp: u64) -> Self {
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
    pub fn new_funding_rate(sym: &str, mark_price: Decimal, index_price: Decimal, funding_rate: Decimal, next_funding_time: u64) -> Self {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        Self {
            symbol,
            payload: EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time },
        }
    }

    #[inline(always)]
    pub fn new_bookticker(sym: &str, best_bid_price: Decimal, best_bid_qty: Decimal, best_ask_price: Decimal, best_ask_qty: Decimal) -> Self {
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
    pub fn new_open_interest(sym: &str, open_interest: Decimal, timestamp: u64) -> Self {
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
            payload: EventType::Trade { price: Decimal::ZERO, quantity: Decimal::ZERO, timestamp: 0, is_buyer_maker: false },
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
