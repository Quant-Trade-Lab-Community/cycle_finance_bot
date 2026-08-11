# 🚀 Walkthrough — 6 Katmanlı Mimari Yapılandırması Tamamlandı

Projenin mevcut derleme ve çalışma bütünlüğüne zarar vermeden, `cycle-engine` sistemi üzerinde mutabık kalınan 6 mimari katmana dayalı klasör yapılandırması başarıyla tamamlanmıştır.

> [!NOTE]
> **Güncelleme (2026-08-11):** Yapılandırma sonrasında veri toplama monolitik `engine` (DATA konsolu) yerine **`flows` crate'indeki 8 bağımsız akış sürecine** taşınmıştır:
> - `engine/src/main.rs` (DATA terminali) kaldırıldı; kalıcılık SQLite yerine **TimescaleDB**'dir (PC'ye native kurulum).
> - `gateway`'e prosesler arası **API rate kapısı** (`rate_gate.rs`) eklendi; akışlar `start_ws_client(..., use_gate=true)` ile bağlanır.
> - Bu ağdan WS ile gelmeyen 6 akış (funding, markprice, indexprice, lastprice, oi) **REST fallback** (`flows/src/rest.rs`) ile beslenir; her akış dakikalık weight'ini `/tmp/cycle_flow_weights/` dosyasına yazar (monitor sekmesi gösterir). Likidasyon WS'te kaldı (REST endpoint yok).
> - **`price-feed` servisi kaldırıldı**; tüketicileri (breakout, alert, paper, stream-ohlcv, risk-worker) artık flow ring'lerini RAM'den okur. `ai-engine` de Desktop'a taşındı.
> - Yeni crate: `flows` (8 binary: `flow-trade` … `flow-indexprice`). Güncel mimari için [cycle_engine_architecture.md](cycle_engine_architecture.md) ve [cycle_engine_processes.md](cycle_engine_processes.md).

## Gerçekleştirilen İşlemler

### 1. Klasör ve Crate Organizasyonu
Aşağıdaki 6 yeni Rust crate'i oluşturulmuş ve ilgili kaynak kodları katmanlarına göre taşınmıştır:
*   `gateway`: Binance WebSocket istemcisi (`binance.rs`)
*   `pipeline`: simd_json olay ayrıştırıcı (`tick.rs`), DataValidator ve Circuit Breaker (`validator.rs`), Lock-free Dispatcher (`queue.rs`)
*   `transport`: OwnedEvent ve EventType tanımları (`events.rs`), Compact binary wire codec (`wire.rs`) ve IPC shared memory ring buffer'ları (`ring_buffer.rs`, `order_ring.rs`, `calc_ring.rs`, `stream_ring.rs`)
*   `engine`: TitaniumOrchestrator spin-loop ve catch_unwind izolasyonu (`orchestrator.rs`), state yönetimi (`state.rs`), CLI araçları (`correlation_cli.rs`, `paper_cli.rs`, `strategy_cli.rs`) ve ana giriş noktası (`main.rs`)
*   `persistence`: SQLite WAL batch writer (`db.rs`) ve ClickHouse adaptörü (`clickhouse.rs`)
*   `infra`: CPU Affinity (`cpu.rs`), Bellek ön ısıtma (`memory.rs`), RDTSC timer (`tsc.rs`), PII maskeleme (`pii.rs`), Vault (`vault.rs`), Redis (`redis.rs`), Telemetry/Observability (`telemetry.rs`) ve AI arayüzü (`ai.rs`)

### 2. Eski Klasörlerin Kaldırılması ve Dış Bağımlılıkların Güncellenmesi
Eski mimariye ait `core`, `adapter`, `contracts` ve `transport` klasörleri tamamen temizlenmiştir.
*   Workspace içerisindeki bağımlı servislerin (`alert-service`, `paper-service`, `price-feed`, `breakout-strategy`, `calc-ind` vb.) `Cargo.toml` dosyaları ve kaynak kodlarındaki import yolları yeni katman crate'lerine yönlendirilerek güncellenmiştir.
*   Workspace root `Cargo.toml` üyeler listesi yeni 6 katmanlı yapıyı barındıracak şekilde yapılandırılmıştır.

### 3. Çalıştırma Doğrulaması
*   `RUN_MODE=DATA cargo run -p engine` komutu ile sistem canlı modda başarıyla çalıştırılmıştır.
*   Binance WebSocket bağlantısı kurulmuş, `[MARKET DATA] Ticks/sec` vb. çıktılarıyla veri akışı doğrulanmıştır.
