# Iron Fist - Execution Engine

The `execution-engine` layer is the outbound component of the system responsible for firing live orders directly to the exchange. It bypasses the traditional HTTP/REST bottlenecks in favor of a lightning-fast, persistent WebSocket Order connection.

## Architecture & Features

1. **WebSocket Order API (`lib.rs`)**
   Instead of suffering through SSL/HTTP handshakes for every order (which can take dozens of milliseconds), this engine establishes a persistent, always-open WebSocket tunnel directly to `wss://ws-api.binance.com:443/ws-api/v3`. When an algorithmic trigger fires in the `core` logic, the order payload is shot through this tunnel in microseconds.

2. **Ultra-Fast Cryptographic Signer (`signer.rs`)**
   Binance requires every algorithmic order to be authenticated with a cryptographic signature. The engine utilizes Rust's `hmac` and `sha2` crates to compute HMAC-SHA256 signatures in a fraction of a microsecond right before transmission.

3. **Lock-Free Order Queue**
   Orders decided by the main `core` trading loop are passed to the execution engine via a dedicated `flume` lock-free channel. This guarantees that the main thread never blocks or lags while waiting for an order to be dispatched to the network socket.

4. **Secure Configuration**
   Strictly utilizes environment variables (`.env`) for API Key and Secret Key management, completely isolating cryptographic secrets from the source code.
