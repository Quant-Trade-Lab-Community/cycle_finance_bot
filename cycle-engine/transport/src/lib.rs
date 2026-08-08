//! Katman 2 — Transport (IPC).
//!
//! Sıfır-kopya, paylaşımlı bellek (/dev/shm) ring buffer'ları. Bu katman
//! değişmez kabul edilir: tüketiciler yalnızca `read_slot(cursor)` sözleşmesini
//! görür, üreticiye dokunmaz.
//!
//! - `ring_buffer`: market data ring'i (GenerationalRing, torn-read korumalı)
//! - `order_ring`: emir ring'i (STRATEGY → EXECUTION)

pub mod ring_buffer;
pub mod order_ring;

pub use ring_buffer::GenerationalRingBuffer;
pub use order_ring::OrderRingBuffer;