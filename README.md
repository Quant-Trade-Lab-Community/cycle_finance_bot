# Demir Yumruk 2.0 - High Frequency Trading (HFT) Engine

Demir Yumruk 2.0 is an ultra-low latency, zero-allocation High Frequency Trading (HFT) and Market Data ingestion engine built in Rust. It is designed to consume, parse, and persist massive amounts of cryptocurrency market data with extreme efficiency.

## 🚀 Core Architecture & Features

### 1. Zero-Allocation (0-Allocation) Ring Buffer
At the heart of the engine is a massive **Zero-Allocation Ring Buffer**. Instead of allocating and freeing memory on the heap for every incoming event (which causes GC/allocator pauses), the system pre-allocates a continuous memory block (e.g., 100 MB or 1 GB) at startup.
- Incoming data is parsed and written directly into this pre-allocated buffer in **O(1)** time.
- When the buffer reaches capacity, it seamlessly wraps around and overwrites the oldest data (FIFO) without ever touching the system's memory allocator.

### 2. Massive WebSocket Multiplexing
The engine directly connects to the Binance WebSocket API (`wss://stream.binance.com:9443/stream`).
- It automatically fetches all active USDT Spot pairs from the Binance REST API (currently ~479 pairs).
- It subscribes to both `@trade` and `@depth20@100ms` (Top 20 Orderbook) streams for **every single pair**.
- Because Binance limits connections to 200 streams per WebSocket, the engine automatically **multiplexes and chunks** the subscriptions across 5+ concurrent asynchronous WebSocket tunnels.

### 3. Ultra-Low Latency Parsing (SIMD-JSON)
The raw JSON byte payloads from Binance are parsed using `simd-json`, which utilizes CPU vector instructions (AVX2/NEON) to parse JSON natively without creating temporary Rust strings.
- **Performance:** Parses a massive 40-level Orderbook payload (20 Bids + 20 Asks) in just **~8 to 12 microseconds (µs)**.
- **Struct Representation:** The parsed data is immediately mapped to a highly compact, fixed-size `#[repr(C)]` enum (`OwnedEvent`) which contains `[u8; 16]` for symbols and fixed arrays `[(f64, f64); 20]` for orderbooks.

### 4. Lock-Free Asynchronous DB Persistence (SQLite)
To prevent the main hot-path from blocking on disk I/O, the engine uses a **Lock-Free bridging architecture (Flume)** for database persistence.
- When the Ring Buffer is full and overwrites an old event, the evicted event is yielded.
- The hot-path attempts a lock-free `try_send` to push the evicted event to a background channel.
- A dedicated background `db_writer` thread reads from this channel and bulk-inserts the data into a local **SQLite** database.
- **SQLite Optimizations:** The DB uses `PRAGMA journal_mode = WAL` and executes chunked batch-inserts (e.g., every 10,000 rows) to comfortably support +100,000 inserts per second without slowing down the trading engine.

### 5. OS-Level Optimizations
The engine attempts to enforce OS-level Real-Time limits to guarantee deterministic execution:
- Automatically pins threads to specific CPU cores.
- Attempts to acquire `SCHED_FIFO` real-time priority (Requires `CAP_SYS_NICE` or root privileges) to prevent the Linux kernel from interrupting the parsing thread.

## 📦 Project Structure
- `core/`: The main heart of the application containing the Ring Buffer, SIMD parser, Lock-Free queues, and SQLite Database Writer.
- `adapter/`: Contains the Binance WebSocket integration and multiplexing logic.
- `os-utils/`: Contains operating system level bindings for CPU pinning and Thread scheduling.
- `cold-storage/`: Memory-mapped file operations (mmap) for alternative persistence paths.
- `risk-worker/`: Independent services dedicated to financial operations and risk matrices.

## 🛠️ Build & Run

### Prerequisites
- Rust (Cargo) 1.70+
- Linux OS (for real-time scheduling features, though it can compile on MacOS/Windows without real-time priorities).

### Running the Engine
Compile and run the engine in release mode for maximum compiler optimization (Mandatory for HFT):

```bash
cargo run -p core --release
```

**Expected Output:**
```text
Demir Yumruk 2.0 Core Initialization...
Binance WS: Fetching active USDT Spot pairs from REST API...
Demir Yumruk: Allocating 101 MB Ring Buffer (160000 elements)...
Binance WS: Found 958 active USDT Spot pairs.
Binance WS [Chunk 1]: Successfully connected.
...
[METRICS] Ticks/sec: 1237 | Avg Parse Latency: 6157.18 ns | RAM Buffer: 8252/160000
```
