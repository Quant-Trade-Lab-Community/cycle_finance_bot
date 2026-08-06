# Iron Fist - Core Layer

The `core` layer is the heart of the High-Frequency Trading (HFT) engine. It is responsible for orchestrating data parsing, memory management, risk validation, and database persistence with absolutely zero heap allocations in the hot path.

## Architecture & Components

1. **Ring Buffer (`ring_buffer.rs`)**
   A fixed-size, pre-allocated block of memory (~100MB) designed to hold 160,000 events without ever calling `malloc` or triggering garbage collection. As new data arrives, it overwrites the oldest data in $O(1)$ time. 

2. **SIMD Parser (`tick.rs`)**
   Utilizes `simd-json` to parse incoming JSON payloads directly from the network socket into our `OwnedEvent` C-style struct in under 10 microseconds. It natively supports parsing Trades, Orderbooks, Liquidations, Funding Rates, and BookTickers.

3. **Data Validation & Circuit Breaker (`validator.rs`)**
   An enterprise-scale risk management layer. Validates every tick before it enters the `RingBuffer` or the database.
   - Rejects stale data (latency > 200ms).
   - Rejects corrupted data (Price <= 0, Crossed Books).
   - Dynamically triggers a global `CIRCUIT_BREAKER` if anomaly frequency spikes (>100 bad ticks/sec).

4. **Persistence Engine (`db.rs`)**
   An asynchronous SQLite writer running on a separate thread via lock-free `flume` channels. It utilizes SQLite `WAL` mode and `PRAGMA synchronous = NORMAL` for high-throughput batch inserts of Liquidations, Funding Rates, Orderbooks, Trades, and Open Interest.

5. **Open Interest Poller (`main.rs`)**
   A dedicated asynchronous background task that polls Binance REST API for Open Interest metrics across major trading pairs without blocking the hot-path websocket stream.
