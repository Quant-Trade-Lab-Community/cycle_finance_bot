# Iron Fist - Adapter Layer

The `adapter` layer serves as the high-speed data ingestion pipeline for the HFT system. Its primary role is to connect to external exchanges (Binance USDⓈ-M Futures), fetch available trading pairs, and subscribe to real-time market data streams.

## Architecture & Features

1. **Massive Multiplexing**
   Binance imposes a strict limit of 200 stream subscriptions per WebSocket connection. To listen to all ~250 USDT Futures pairs simultaneously across 5 different streams (1,250 total streams), the adapter implements an automated "Chunking" mechanism. It spins up 7 concurrent async `tokio-tungstenite` WebSocket connections to distribute the load.

2. **Data Streams Subscribed**
   For every USDT pair, the adapter ingests the following streams at millisecond precision:
   - `<symbol>@trade`: Real-time trade executions.
   - `<symbol>@depth20@100ms`: 20-level orderbook snapshots updated every 100ms.
   - `<symbol>@forceOrder`: Real-time liquidation events (crucial for momentum/HFT algorithms).
   - `<symbol>@markPrice`: Funding rates and mark prices updated every 3 seconds.
   - `<symbol>@bookTicker`: The best bid and ask prices (BBO) updated in real-time as fast as they change.

3. **Lock-Free Passing**
   Instead of parsing the heavy JSON payload directly in the networking layer, the adapter grabs the raw bytes directly from the TCP socket and throws them into a highly optimized, lock-free `flume` channel. This ensures the network connection never gets bottlenecked or dropped.
