# 🏗️ Cycle-Engine Mimari Dokümanı

> Tüm kaynak kodlar (`*.rs`, `Cargo.toml`) satır satır incelenerek hazırlanmıştır.

---

## 1. Genel Bakış

`cycle-engine`, yüksek frekanslı ticaret (HFT) sisteminin **veri toplama → ayrıştırma → doğrulama → IPC taşıma → orkestrasyon → kalıcılık** zincirini oluşturan çekirdek bileşendir. 8 ayrı Rust crate'inden oluşur ve sistemin Katman 0–2 altyapısını sağlar.

```mermaid
graph TB
    subgraph "cycle-engine (8 Crate)"
        GW["gateway<br/>WebSocket İstemci + Rate Kapısı"]
        PL["pipeline<br/>SIMD Parser + Validator"]
        TR["transport<br/>IPC Ring Buffers + Binary Codec + FlowKind"]
        EN["engine<br/>strategy-console + TitaniumOrchestrator"]
        FL["flows<br/>8 bağımsız veri akışı süreci"]
        PE["persistence<br/>TimescaleDB + ClickHouse"]
        IN["infra<br/>Donanım + Güvenlik"]
        SP["splash<br/>Terminal Açılış"]
    end

    BN["Binance WSS"] --> GW
    GW -->|"flume channel<br/>Vec&lt;u8&gt;"| PL
    GW -.->|"rate gate (shm)"| RG["/dev/shm/cycle_finance_api_gate"]
    FL -->|"WS→parse→validate→ring→TSDB"| PE
    PL -->|"OwnedEvent"| TR
    TR -->|"/dev/shm Ring Buffers"| FL
    EN -->|"OrderRingBuffer"| EX["execution-engine"]
    EN -->|"Risk Gate"| RI["risk-engine"]
    SC["strategy-console"] -->|"Strategy Eval (alt-süreç)"| ST["strategies-engine"]
    IN -.->|"CPU Pin, TSC, Vault"| EN
    SP -.->|"Startup Animation"| EN

    style GW fill:#1a1a2e,stroke:#e94560,color:#fff
    style PL fill:#1a1a2e,stroke:#0f3460,color:#fff
    style TR fill:#1a1a2e,stroke:#16213e,color:#fff
    style EN fill:#1a1a2e,stroke:#533483,color:#fff
    style FL fill:#1a1a2e,stroke:#f1c40f,color:#fff
    style PE fill:#1a1a2e,stroke:#e94560,color:#fff
    style IN fill:#1a1a2e,stroke:#0f3460,color:#fff
    style SP fill:#1a1a2e,stroke:#16213e,color:#fff
```

---

## 2. Katmanlı Mimari

| Katman | Crate | Sorumluluk | Gecikme Hedefi |
|:---:|:---|:---|:---:|
| **0** | `transport` | Veri modelleri, binary codec, IPC ring buffer'lar, `FlowKind` | < 100 ns |
| **1** | `gateway` | Binance WebSocket bağlantıları, reconnect, ping/pong, **API rate kapısı** | N/A (ağ) |
| **2** | `pipeline` | SIMD JSON parse, veri doğrulama, circuit breaker | < 1 μs |
| **3** | `engine` | TitaniumOrchestrator (spin-loop) + `strategy-console` (strateji alt-süreç orkestrasyonu) | < 10 μs |
| **4** | `flows` | **8 bağımsız veri akışı süreci**: `WS → parse → validate → ring → TimescaleDB` | N/A |
| **5** | `persistence` | TimescaleDB batch write, ClickHouse şeması | Async |
| **6** | `infra` | CPU affinity, RDTSC timer, Vault, Redis, telemetri | N/A |
| **7** | `splash` | FIGlet ASCII animasyonu, terminal UI | N/A |

---

## 3. Crate Detayları

### 3.1 `gateway` — Borsa Bağlantı Katmanı

[gateway/src/binance.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/gateway/src/binance.rs) · [gateway/src/rate_gate.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/gateway/src/rate_gate.rs)

**Sorumluluk:** Binance Futures/Spot WebSocket API'lerine bağlanır, ham JSON baytlarını `flume` kanalına aktarır. Ayrıca bağımsız akış süreçlerinin Binance API limitlerine takılmaması için **prosesler arası rate kapısı** sağlar.

**Ana Fonksiyonlar:**
```rust
pub async fn start_binance_ws_client(tx: Sender<Vec<u8>>)              // legacy: trade+depth stream seti
pub async fn start_ws_client(tx: Sender<Vec<u8>>, streams: Vec<String>, use_gate: bool)  // genel
pub struct RateGate;                                                    // paylaşımlı bellek token bucket
```

**Temel Mekanizmalar:**

| Mekanizma | Detay |
|:---|:---|
| **Exponential Backoff** | 1s → 2s → 4s → ... → 60s (tavan), başarılı bağlantıda sıfırlanır |
| **Ping/Pong Heartbeat** | 30 sn aralıkla `Message::Ping`, başarısız olursa reconnect |
| **WAF Koruması** | Chunk'lar arası 600ms gecikme ile rate-limit engelleme |
| **Chunked Connections** | Sembol akışları 200'erli gruplara bölünür, her biri ayrı Tokio task |
| **Rate Kapısı (RateGate)** | `/dev/shm/cycle_finance_api_gate` token bucket — tüm akışlar aynı bütçeyi paylaşır; bağlantı öncesi `acquire()` |
| **Akış Türleri** | Per-akış stream setleri: `@trade`, `@depth20@100ms`, `@forceOrder`, `@markPrice@1s`, `@indexPrice@1s`, `@lastPrice@1s`, `!openInterest@arr` |

**Rate Kapısı (API rate limit koruması):**
```mermaid
sequenceDiagram
    participant F as Akış Süreci (ör. flow-trade)
    participant G as RateGate (/dev/shm/cycle_finance_api_gate)
    participant B as Binance WSS

    F->>G: acquire(token)
    alt token varsa
        G-->>F: ok
        F->>B: WebSocket Connect + SUBSCRIBE
    else token yoksa
        G-->>F: bekle (25ms poll, 30s tavan)
    end
```

- Token bucket: kapasite `CYCLE_GATE_CAPACITY` (varsayılan 8), dolum `CYCLE_GATE_RATE` token/sn (varsayılan 4)
- Paylaşımlı bellek olduğundan 8 bağımsız akış tek bütçeyi paylaşır; akışlar birbirine bağımlı değildir

```mermaid
sequenceDiagram
    participant B as Binance WSS
    participant G as Gateway
    participant C as flume Channel

    G->>B: WebSocket Connect (wss://fstream.binance.com)
    G->>B: SUBSCRIBE {params: [...streams]}
    loop Her mesaj
        B-->>G: JSON Text Frame
        G->>C: tx.send_async(bytes)
    end
    loop Her 30sn
        G->>B: Ping
        B-->>G: Pong
    end
    Note over G,B: Bağlantı koparsa Exponential Backoff ile yeniden bağlan
```

**Bağımlılıklar:** `tokio-tungstenite`, `futures-util`, `tokio`, `flume`, `serde_json`, `serde`, `libc`, `memmap2`

---

### 3.2 `pipeline` — Veri İşleme Hattı

**Modüller:** [tick.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/pipeline/src/tick.rs) · [validator.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/pipeline/src/validator.rs) · [queue.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/pipeline/src/queue.rs)

#### A. EventParser (tick.rs)

```rust
pub fn parse(bytes: &mut [u8]) -> Option<OwnedEvent>
```

- `simd_json::to_borrowed_value()` ile **zero-copy SIMD parse**
- Desteklenen akış türleri: `@trade`, `@depth`, `@forceOrder`, `@markPrice`, `@bookTicker`

#### B. DataValidator (validator.rs)

```rust
pub fn is_valid(&mut self, event: &OwnedEvent) -> bool
```

| Kontrol | Koşul | Sonuç |
|:---|:---|:---|
| Fiyat/Miktar | `<= 0` | Reject |
| Stale Data | `> 200ms` gecikme | Reject |
| NTP Drift | `> 5000ms` sapma | Reject |
| Crossed Book | `best_bid >= best_ask` | Reject |
| Circuit Breaker | 1sn'de `> 100` bozuk tick | Tüm akışı durdur |

**Circuit Breaker Durum Makinesi:**
```mermaid
stateDiagram-v2
    [*] --> Normal
    Normal --> Triggered: bad_tick_count > 100 (1sn içinde)
    Triggered --> Recovered: 1sn sonra sayaç sıfırlanır
    Recovered --> Normal: Akış devam
```

#### C. LockFreeDispatcher (queue.rs)

```rust
pub struct LockFreeDispatcher {
    tx: Sender<Vec<u8>>,   // flume bounded(262_144)
    rx: Receiver<Vec<u8>>,
}
```

- 262.144 elemanlı bounded kuyruk → backpressure desteği

**Bağımlılıklar:** `transport`, `simd-json`, `rust_decimal`, `flume`

---

### 3.3 `transport` — IPC Omurgası

**Modüller:** [events.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/transport/src/events.rs) · [wire.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/transport/src/wire.rs) · [flow.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/transport/src/flow.rs) · [ring_buffer.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/transport/src/ring_buffer.rs) · [order_ring.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/transport/src/order_ring.rs) · [calc_ring.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/transport/src/calc_ring.rs) · [stream_ring.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/transport/src/stream_ring.rs)

#### A. Veri Modelleri (events.rs)

```rust
#[repr(C)]
pub struct OwnedEvent {
    pub symbol: [u8; 16],    // Sabit boyutlu sembol (örn: "BTCUSDT\0...")
    pub payload: EventType,
}

#[repr(u8)]
pub enum EventType {
    Trade { price, quantity, timestamp, is_buyer_maker },
    Orderbook { bids: [(Decimal,Decimal); 20], asks: [(Decimal,Decimal); 20] },
    Liquidation { side, price, quantity, timestamp },
    FundingRate { mark_price, index_price, funding_rate, next_funding_time },
    BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty },
    OpenInterest { open_interest, timestamp },
    Opportunity { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict },
    SymbolMetrics { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps },
}
```

#### B. Binary Codec (wire.rs)

| Sabit | Değer | Açıklama |
|:---|:---|:---|
| `DEPTH_FRAME_SIZE` | 659 B | Orderbook 20-derinlik frame boyutu |
| `MAX_FRAME_SIZE` | 659 B | Maksimum frame boyutu |

```rust
pub fn encode(ev: &OwnedEvent, buf: &mut [u8]) -> Option<usize>  // ~659B vs JSON ~1100B (%40+ tasarruf)
pub fn decode(buf: &[u8]) -> Option<OwnedEvent>
```

- `Decimal` → `(mantissa: i64, scale: u8)` dönüşümü ile 9 bayt/değer
- Tag-based compact binary format, sıfır heap allocation

#### B2. FlowKind (flow.rs) — Veri Akışı Tipleri

```rust
pub enum FlowKind {
    Trade, Depth, Liquidation, OpenInterest,
    Funding, MarkPrice, LastPrice, IndexPrice,
}
```

Her akış ayrı bir OS sürecidir ve kendi ring'ine + TimescaleDB hypertable'ına sahiptir:

| Akış | Ring (shm) | Bellek Bütçesi | TimescaleDB Tablosu |
|:---|:---|:---:|:---|
| trade | `/cycle_finance_trades` | 50 MB | `trades` |
| depth | `/cycle_finance_depth` | 100 MB | `orderbooks` |
| liquidation | `/cycle_finance_liquidations` | 20 MB | `liquidations` |
| open-interest | `/cycle_finance_open_interest` | 20 MB | `open_interests` |
| funding | `/cycle_finance_funding` | 10 MB | `funding_rates` |
| mark-price | `/cycle_finance_markprice` | 50 MB | `markprices` |
| last-price | `/cycle_finance_lastprice` | 50 MB | `lastprices` |
| index-price | `/cycle_finance_indexprice` | 50 MB | `indexprices` |

- `ring_capacity()`: bellek bütçesi / `size_of::<MarketDataSlot>()` (768 B)
- Ring kapasitesi bellek bütçesinden türetilir → belirlenen RAM sınırları asla aşılmaz

#### C. Ring Buffer'lar (POSIX Shared Memory IPC)

| Ring Buffer | SHM Yolu | Slot Boyutu | Varsayılan Kapasite | Kullanım |
|:---|:---|:---:|:---:|:---|
| `GenerationalRingBuffer` | `/cycle_finance_ring` (varsayılan) | 768 B | Parametrik | Akış ring'leri `with_name()` ile ayrı isim alır (Bölüm 3.8) |
| `OrderRingBuffer` | `/cycle_finance_orders` | ~64 B | Parametrik | Emir iletimi |
| `CalcRingBuffer` | `/cycle_finance_calc` | 1 MB | Parametrik | İndikatör/OHLCV sonuçları |
| `StreamRingBuffer` | `/cycle_finance_stream_ohlcv` | 4 KB | 8192 | Canlı OHLCV mum akışı |

**Torn-Read Koruması (Lock-Free Güvenlik):**
```mermaid
sequenceDiagram
    participant W as Writer (Producer)
    participant S as Shared Slot
    participant R as Reader (Consumer)

    W->>S: 1. Veri yaz (data + len)
    W->>S: 2. fence(Release)
    W->>S: 3. seq yaz
    W->>S: 4. head.store(Release)
    
    R->>S: 1. slot.seq == hedef_seq ?
    R->>S: 2. Veriyi kopyala
    R->>S: 3. Tekrar kontrol: slot.seq == hedef_seq ?
    Note over R: Eşleşmezse → None (torn read engellendi)
```

**Bellek Düzeni:**
- Tüm slot yapıları `#[repr(C, align(64))]` → CPU Cache Line hizalı (false-sharing yok)
- `AtomicU64` ile lock-free SPMC/MPMC okuma/yazma
- İlk oluşturan süreç `ftruncate` yapar, sonrakiler sadece bağlanır
- Magic number doğrulaması ile eski/bozuk SHM tespiti ve yeniden başlatma

**Bağımlılıklar:** `rust_decimal`, `libc`, `memmap2`

---

### 3.4 `engine` — Strateji Orkestrasyonu

**Modüller:** [orchestrator.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/engine/src/engine/orchestrator.rs) · [strategy_console.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/engine/src/engine/strategy_console.rs) · [detector_bridge.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/engine/src/bridge/detector_bridge.rs)

**Tek binary:** `strategy-console` (strateji orkestrasyon konsolu — `src/bin/strategy-console.rs`). Eski DATA terminali (`src/main.rs`) kaldırılmıştır; canlı veri toplama artık bağımsız `flows` süreçleri tarafından yapılır (Bölüm 3.8).

#### A. TitaniumOrchestrator

```rust
pub struct TitaniumOrchestrator {
    strategies: Vec<ShardedStrategy>,
    risk_manager: RiskEngine,
    gateway_tx: Sender<Signal>,
}
```

| Metot | Açıklama |
|:---|:---|
| `new()` | Stratejileri yükler, risk engine'i başlatır |
| `run_spin_loop(&mut self, ring: &GenerationalRingBuffer)` | Ana hot-path döngüsü |

**Spin-Loop Mimarisi:**
```mermaid
flowchart TD
    A["Ring Buffer head okuma"] --> B{"Yeni veri var mı?"}
    B -->|Hayır| C["spin_loop() hint"]
    C --> A
    B -->|Evet| D["read_slot + wire::decode"]
    D --> E["Strateji Değerlendirme<br/>(catch_unwind ile panic izolasyonu)"]
    E --> F{"Sinyal üretildi mi?"}
    F -->|Hayır| A
    F -->|Evet| G["signal_to_intent → OrderIntent"]
    G --> H["RiskEngine.evaluate()"]
    H --> I{"Onaylandı mı?"}
    I -->|Evet| J["gateway_tx.send(Signal)"]
    I -->|Hayır| K["Reject & Log"]
    J --> A
    K --> A

    style A fill:#0f3460,color:#fff
    style E fill:#533483,color:#fff
    style H fill:#e94560,color:#fff
```

**Panic İzolasyonu:**
```rust
std::panic::catch_unwind(AssertUnwindSafe(|| strategy.evaluate(event)))
// Panic → StrategyState::Poisoned (motor çökmez)
```

**Strateji Durum Makinesi:**
```mermaid
stateDiagram-v2
    [*] --> Active
    Active --> Poisoned: Panic yakalandı
    Active --> Draining: Graceful shutdown
    Poisoned --> [*]: Devre dışı bırakıldı
    Draining --> [*]: Tüm emirler kapandı
```

#### B. DetectorBridge (Scout Ring Okuyucu)

```rust
pub struct DetectorBridge {
    ring: GenerationalRingBuffer,  // /cycle_finance_scout
    cursor: u64,
}
```

- Harici detektörlerin `Opportunity` sinyallerini okur
- `spawn_watcher()` ile arka plan Tokio task'ı başlatır

#### C. STRATEGY Orkestrasyon Konsolu (strategy_console.rs)

[strategy_console.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/engine/src/engine/strategy_console.rs) `strategy-console` binary'sinin gövdesidir (`src/bin/strategy-console.rs`). DATA konsolundan tamamen bağımsız ayrı bir OS sürecidir — `RUN_MODE` kullanılmaz.

```rust
pub fn run_strategy_console()   // sonsuz döngü: komut kuyruğu + stdin + tick
```

- `strategies-engine::StrategyOrchestrator` barındırır; `services-engine/strategies/` klasörünü yönetir
- Komutlar `/tmp/strategy_cmd.d/*.cmd` kuyruğundan (maildir benzeri, 250ms poll) ve stdin'den (rustyline) okunur
- Durum raporu `/tmp/strategy_status.txt`'e yazılır
- Strateji adları dizin adından gelir; `-strategy` sonek takma adı desteklenir (`breakout` → `breakout-strategy`)
- Alt-süreçleri SIGTERM ile durdurur, `tick()` ölen süreçleri toplar (reap)
- Çıkışta tüm yönetilen stratejiler durdurulur

**Bağımlılıklar:** `gateway`, `pipeline`, `transport`, `persistence`, `infra`, `execution-engine`, `risk-engine`, `strategies-engine`, `tokio`, `flume`, `parking_lot`, `crossbeam-channel`, `rustyline`, `criterion` (bench)

---

### 3.5 `persistence` — Kalıcılık Katmanı

**Modüller:** [timescaledb.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/persistence/src/timescaledb.rs) · [clickhouse.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/persistence/src/clickhouse.rs)

> **SQLite kaldırıldı.** Zaman serisi kalıcılığı artık **TimescaleDB** (PostgreSQL uzantısı) ile yapılır. Eski `db.rs` (rusqlite) silinmiştir.

> **Kurulum (PC'ye native — Docker değil):** PostgreSQL 18 (PGDG repo) + TimescaleDB 2.x (`timescale/timescaledb` packagecloud repo), `timescaledb-tune` ile `shared_preload_libraries='timescaledb'`, `CREATE EXTENSION timescaledb`. Kullanıcı `cycle` / şifre `cycle`, DB `market_data`. Bağlantı `TIMESCALEDB_URL` (varsayılan `postgres://cycle:cycle@localhost:5432/market_data`).

#### A. TimescaleDB Batch Writer (timescaledb.rs)

```rust
pub fn start_tsdb_writer(rx: Receiver<OwnedEvent>, kind: FlowKind)
```

| Ayar | Değer |
|:---|:---|
| Sürücü | `sqlx` (tokio, postgres) |
| Bağlantı | `TIMESCALEDB_URL` (varsayılan `postgres://cycle:cycle@localhost:5432/market_data`) |
| Bağlantı yoksa | 2 sn bekleme ile sonsuz yeniden deneme (veri akışı durmaz) |
| Batch Size | 1.000 kayıt |
| Flush Interval | 1.000 ms |
| Hypertable | `timestamp` (BIGINT ms) üzerinde `create_hypertable(..., if_not_exists => TRUE)` |

**Hypertable Şeması (akış başına):**

| Tablo | Sütunlar | Akış |
|:---|:---|:---|
| `trades` | symbol, price, quantity, is_buyer_maker, timestamp | trade |
| `orderbooks` | symbol, bids **JSONB**, asks **JSONB**, timestamp | depth |
| `liquidations` | symbol, side, price, quantity, timestamp | liquidation |
| `open_interests` | symbol, open_interest, timestamp | open-interest |
| `funding_rates` | symbol, mark_price, index_price, funding_rate, next_funding_time, timestamp | funding |
| `markprices` | symbol, price, timestamp | mark-price |
| `lastprices` | symbol, price, timestamp | last-price |
| `indexprices` | symbol, price, timestamp | index-price |

- Ayrı thread'de çalışır, akış sürecini asla bloke etmez
- Bağlantı hatası olsa bile ring yazımı devam eder; yazıcı bekleme ile yeniden bağlanır

#### B. ClickHouse Adapter (clickhouse.rs)

```rust
pub struct ClickHouseAdapter {
    deletion_registry: Mutex<HashMap<String, u64>>,
}
```

| Özellik | Detay |
|:---|:---|
| Sıkıştırma | ZSTD(22) |
| Partisyonlama | `toYear/toMonth/toDayOfMonth` |
| GDPR/KVKK | `execute_right_to_erasure()` ile veri silme |
| Veri Bütünlüğü | Merkle Tree + EC-12/4 Erasure Coding doğrulama |

**Bağımlılıklar:** `transport`, `rust_decimal`, `flume`, `serde_json`, `tokio`, `sqlx`

---

### 3.6 `infra` — Altyapı ve Donanım Soyutlama

**Modüller:** [hal/cpu.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/infra/src/hal/cpu.rs) · [hal/memory.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/infra/src/hal/memory.rs) · [timer/tsc.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/infra/src/timer/tsc.rs) · [redis.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/infra/src/redis.rs) · [vault.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/infra/src/vault.rs) · [pii.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/infra/src/pii.rs) · [telemetry.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/infra/src/telemetry.rs) · [ai.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/infra/src/ai.rs)

#### A. Donanım Soyutlama (HAL)

| Fonksiyon | Açıklama |
|:---|:---|
| `pin_to_core(core_id)` | CPU affinity — context switch gecikme eliminasyonu |
| `allocate_huge_buffer(size)` | Page pre-faulting — çalışma anında page-fault yok |
| `TscTimer::read_tsc()` | `_rdtsc()` intrinsic ile nanosaniye altı zamanlama |

#### B. Güvenlik ve Uyumluluk

| Bileşen | Özellikler |
|:---|:---|
| **VaultAdapter** | Key rotation, JWT üretimi (1h TTL), health check |
| **RedisAdapter** | Atomik idempotency (`SET NX EX 3600`), fail-closed tasarım |
| **PIIMasker** | SHA-3 + Salt hash, GDPR/KVKK 3 yıl log temizleme |

#### C. Telemetri ve AI

| Bileşen | Özellikler |
|:---|:---|
| **TelemetryAgent** | eBPF RTT tracking, adaptif Jaeger sampling (%1 → %100), Chaos Mesh |
| **AIAdapter** | Isolation Forest anomali skoru, LLM trend etiketi |

**Bağımlılıklar:** `core_affinity`, `libc`, `memmap2`, `sha3`, `reqwest`, `serde`, `redis`, `tokio`

---

### 3.7 `splash` — Terminal Açılış Animasyonu

[splash/src/lib.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/splash/src/lib.rs)

```rust
pub fn show_splash()
pub fn show_splash_with(metin: &str, total_ms: u64)
```

- FIGlet ASCII banner ("CYCLE FINANCE")
- True Color Matrix yeşili (`\x1B[38;2;0;255;65m`) animasyon
- Terminal boyutuna göre dinamik ortalama
- `ffplay` ile futuristik ses efekti
- Senkronize ilerleme çubuğu (%0 → %100)

---

### 3.8 `flows` — Bağımsız Veri Akışı Süreçleri

[flows/src/lib.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/flows/src/lib.rs) · [flows/src/parse.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/flows/src/parse.rs)

Eski monolitik DATA terminalinin yerini alır. **Her akış ayrı bir OS sürecidir** ve değişmez hattı izler:

```text
WS | REST → parse → validate → ring buffer → TimescaleDB
```

> **Veri kaynağı:** Bu ağdan Binance `markPrice/indexPrice/lastPrice/forceOrder/openInterest`
> stream'leri WS ile iletilmediğinden bu akışlar **REST fallback** kullanır
> (aynı frame → parse → validate → ring → TSDB hattı; `flows/src/rest.rs`).
> Trade, depth ve liquidation WS ile gelir.

**8 binary (`flows` crate'inde):**

| Akış | Binary | Kaynak | Stream / REST Endpoint | Bellek | Ring |
|:---|:---|:---|:---|:---:|:---|
| 1. Trade | `flow-trade` | **WS** | `{sym}@trade` | 50 MB | `/cycle_finance_trades` |
| 2. Depth20 | `flow-depth` | **WS** | `{sym}@depth20@100ms` | 100 MB | `/cycle_finance_depth` |
| 3. Likidasyon | `flow-liquidation` | **WS** | `{sym}@forceOrder` | 20 MB | `/cycle_finance_liquidations` |
| 4. Open Interest | `flow-oi` | **REST** | `GET /fapi/v1/openInterest` | 20 MB | `/cycle_finance_open_interest` |
| 5. Funding Rate | `flow-funding` | **REST** | `GET /fapi/v1/premiumIndex` | 10 MB | `/cycle_finance_funding` |
| 6. Mark Price | `flow-markprice` | **REST** | `GET /fapi/v1/premiumIndex` | 50 MB | `/cycle_finance_markprice` |
| 7. Last Price | `flow-lastprice` | **REST** | `GET /fapi/v1/ticker/price` | 50 MB | `/cycle_finance_lastprice` |
| 8. Index Price | `flow-indexprice` | **REST** | `GET /fapi/v1/premiumIndex` | 50 MB | `/cycle_finance_indexprice` |

- REST poll `CYCLE_REST_POLL_MS` (varsayılan 2000 ms; oi en az 5000 ms) — her döngü öncesi `RateGate` token'ı alınır
- REST yanıtı `flows/src/rest.rs` içinde WS-format frame'e çevrilir → mevcut parse/validate/ring/TSDB hattı aynen kullanılır
- **Rate koruması:** HTTP **429** → 60 sn, HTTP **418** (teapot/IP banı) → 5 dk geri çekilme
- **Weight izleme:** Her REST akışı dakikalık ağırlığını (`request sayısı × endpoint weight`; premiumIndex=1, ticker/price=2, openInterest=1) `/tmp/cycle_flow_weights/<flow>.weight` dosyasına yazar — **monitor sekmesi** toplar ve `Binance REST Ağırlığı` olarak gösterir (limit 2400/dk)

**Süreç içi yapı (`flows::run(kind)`):**

| # | Parça | Açıklama |
|:---:|:---|:---|
| 1 | **Tüketici thread** (RT 99) | `parse_for()` → `DataValidator` → `wire::encode` → kendi ring'ine `push` → DB kanalına `try_send` |
| 2 | **DB yazıcı thread** | `persistence::timescaledb::start_tsdb_writer(rx, kind)` — kendi hypertable'ına batch commit |
| 3 | **Veri kaynağı** | WS (trade/depth/liquidation) veya REST fallback (funding/markprice/indexprice/lastprice/oi) — ikisi de rate kapısından geçer |

**Akış ayrıştırma (parse.rs):** Mevcut `EventParser` yeniden kullanılır; yeni stream'ler mevcut `EventType` varyantlarına eşlenir (enum'a yeni varyant EKLENMEZ — dış tüketici match'leri bozulmaz):
- `@lastPrice@1s` → `FundingRate { mark_price: p, .. }` (lastprices tablosuna `price` olarak yazar)
- `@indexPrice@1s` → `FundingRate { index_price: i, .. }` (indexprices tablosuna `price` olarak yazar)
- `!openInterest@arr` → `OpenInterest` (dizi öğeleri ayrı ayrı)

**Bağımsızlık + rate kapısı:** Akışlar birbirinden bağımsızdır; yalnızca Binance API limitlerine takılmamak için ortak `RateGate`'e bağlanır (`gateway::rate_gate`).

> **Likidasyon notu:** Binance'te piyasa geneli likidasyon için REST endpoint **yoktur** (`allForceOrders` 404; yalnızca kullanıcının kendi `forceOrders`u vardır). Bu ağda `forceOrder` WS stream'i de iletilmediğinden `flow-liquidation` WS aboneliğinde kalır ve tablo bu ağda boş olur; stream'in çalıştığı ağda anında dolar.

**Ortam değişkenleri:** `CYCLE_FLOW_SYMBOLS` (varsayılan `BTCUSDT,ETHUSDT,SOLUSDT,VELVETUSDT`) · `TIMESCALEDB_URL` · `CYCLE_REST_POLL_MS` · `CYCLE_GATE_CAPACITY` / `CYCLE_GATE_RATE`

**Bağımlılıklar:** `gateway`, `pipeline`, `transport`, `persistence`, `os-utils`, `tokio`, `flume`, `simd-json`, `rust_decimal`, `serde_json`, `reqwest`

---

## 4. Uçtan Uca Veri Akışı

**Akış hattı (her akış için aynı — kaynak WS veya REST fallback olabilir):**

```mermaid
flowchart LR
    subgraph "Giriş"
        BN["Binance WSS / REST<br/>fstream.binance.com / fapi.binance.com"]
        GW["gateway (WS)<br/>veya rest::spawn (REST)"]
        RG["RateGate<br/>/dev/shm/cycle_finance_api_gate"]
    end
    
    subgraph "İşleme (akış süreci)"
        Q["flume bounded<br/>262K"]
        EP["parse_for()<br/>simd_json + EventParser"]
        DV["DataValidator<br/>Circuit Breaker"]
    end
    
    subgraph "Taşıma"
        WE["wire::encode()<br/>compact binary"]
        RB["Akış ring'i<br/>(ör. /cycle_finance_trades)"]
    end
    
    subgraph "Kalıcılık"
        DBQ["flume bounded(1M)"]
        DB["TimescaleDB yazıcı<br/>hypertable (batch 1000/1s)"]
    end

    BN -->|WSS| GW
    GW -.->|"token"| RG
    GW -->|"flume Vec<u8>"| Q
    Q --> EP
    EP --> DV
    DV -->|OwnedEvent| WE
    WE --> RB
    DV -->|"try_send"| DBQ
    DBQ -->|"ayrı thread"| DB

    style GW fill:#1a1a2e,stroke:#e94560,color:#fff
    style EP fill:#0f3460,stroke:#fff,color:#fff
    style RB fill:#533483,stroke:#fff,color:#fff
    style DB fill:#e94560,stroke:#fff,color:#fff
```

**8 akış** aynı deseni kendi sürecinde çalıştırır; her biri kendi ring'ine yazar ve kendi TimescaleDB hypertable'ına besler. Akışlar arasında veri paylaşımı yoktur (bağımsız); tek ortak nokta Binance rate kapısıdır.

---

## 5. Çapraz-Engine Bağımlılık Haritası

```mermaid
graph TD
    TR["transport"] 
    GW["gateway"]
    PL["pipeline"]
    EN["engine"]
    FL["flows"]
    PE["persistence"]
    IN["infra"]
    
    RI["risk-engine"]
    SE["strategies-engine"]
    EE["execution-engine"]
    
    AS["alert-service"]
    PS["paper-service"]
    CI["calc-ind"]
    SO["stream-ohlcv"]

    FL --> GW
    FL --> PL
    FL --> TR
    FL --> PE
    EN --> GW
    EN --> PL
    EN --> TR
    EN --> PE
    EN --> IN
    EN --> RI
    EN --> SE
    EN --> EE
    
    PL --> TR
    PE --> TR
    
    RI --> TR
    SE --> TR
    EE --> RI
    
    AS --> TR
    AS --> EN
    AS --> PL
    PS --> TR
    PS --> EN
    PS --> EE
    CI --> TR
    SO --> TR

    style TR fill:#e94560,stroke:#fff,color:#fff
    style EN fill:#533483,stroke:#fff,color:#fff
    style FL fill:#b7950b,stroke:#fff,color:#fff
```

> **`transport` crate'i tüm sistemin IPC omurgasıdır** — 12+ crate doğrudan bağımlıdır. **`flows`** akış süreçleri gateway/pipeline/transport/persistence'e bağımlıdır (veri hattı).

---

## 6. IPC Ring Buffer Topolojisi

```mermaid
graph LR
    subgraph "/dev/shm — Akış Ring'leri (her akış kendi sürecinde yazar)"
        R1["/cycle_finance_trades<br/>768B slot — Trade"]
        R2["/cycle_finance_depth<br/>768B slot — Depth20"]
        R3["/cycle_finance_liquidations<br/>768B slot — Likidasyon"]
        R4["/cycle_finance_open_interest<br/>768B slot — Open Interest"]
        R5["/cycle_finance_funding<br/>768B slot — Funding"]
        R6["/cycle_finance_markprice<br/>768B slot — Mark Price"]
        R7["/cycle_finance_lastprice<br/>768B slot — Last Price"]
        R8["/cycle_finance_indexprice<br/>768B slot — Index Price"]
        G1["/cycle_finance_api_gate<br/>Rate Kapısı (token bucket)"]
    end

    F1["flow-trade"] -->|write| R1
    F2["flow-depth"] -->|write| R2
    F3["flow-liquidation"] -->|write| R3
    F4["flow-oi"] -->|write| R4
    F5["flow-funding"] -->|write| R5
    F6["flow-markprice"] -->|write| R6
    F7["flow-lastprice"] -->|write| R7
    F8["flow-indexprice"] -->|write| R8
    F1 --> G1
    F2 --> G1
    F3 --> G1
    F4 --> G1
    F5 --> G1
    F6 --> G1
    F7 --> G1
    F8 --> G1

    subgraph "Diğer Ring'ler"
        R9["/cycle_finance_orders<br/>~64B — Emir"]
        R10["/cycle_finance_calc<br/>1MB — İndikatör"]
        R11["/cycle_finance_stream_ohlcv<br/>4KB — OHLCV"]
    end

    ST["strategies"] -->|write| R9
    EX["execution"] -->|read| R9
    CI["calc-ind"] -->|write| R10
    SO["stream-ohlcv"] -->|write| R11

    subgraph "Tüketiciler (flow ring'lerini RAM'den okur)"
        C1["breakout-strategy"] -->|read| R1
        C2["alert-service"] -->|read| R1
        C4["paper-service"] -->|read| R1
        C5["stream-ohlcv"] -->|read| R7
        C6["risk-worker"] -->|read| R6
    end
```

---

## 7. Performans Tasarım İlkeleri

| İlke | Uygulama |
|:---|:---|
| **Zero-Copy** | `simd_json::to_borrowed_value()` — mutable buffer üzerinde parse |
| **Zero-Allocation** | `#[repr(C)]` sabit boyutlu struct'lar, stack-only veri modelleri |
| **Lock-Free IPC** | `AtomicU64` + `fence(Release/Acquire)` ile SPMC ring buffer |
| **Cache-Line Aligned** | `#[repr(C, align(64))]` — false sharing engellenir |
| **CPU Pinning** | `core_affinity::set_for_current()` — context switch yok |
| **RDTSC Timer** | `_rdtsc()` intrinsic — nanosaniye altı ölçüm |
| **Spin-Loop** | `std::hint::spin_loop()` — sıfır syscall bekleme |
| **Compact Binary** | JSON ~1100B → wire ~659B (%40+ tasarruf) |
| **Batch Write** | 1000 kayıt veya 1sn → tek TimescaleDB transaction |
| **Akış İzolasyonu** | Her akış ayrı süreç + ayrı ring + ayrı hypertable (RAM bütçeleri ring kapasitesini belirler) |
| **Rate Kapısı** | Prosesler arası token bucket — Binance limitlerine takılma yok |
| **Panic Isolation** | `catch_unwind()` ile strateji paniklerini izole et |

---

## 8. Güvenlik ve Uyumluluk Kontrolleri

| Kontrol | Mekanizma | Bileşen |
|:---|:---|:---|
| **İdempotency** | Redis `SET NX EX 3600` (fail-closed) | `infra/redis.rs` |
| **Key Rotation** | Vault dual-key + 5dk grace period | `infra/vault.rs` |
| **PII Masking** | SHA-3 + Salt hash | `infra/pii.rs` |
| **GDPR/KVKK** | ClickHouse `ALTER DELETE` + 3 yıl log temizleme | `persistence/clickhouse.rs` |
| **KillSwitch** | `/tmp/exec_kill_switch` dosyası ile acil durdurma | `risk-engine` |
| **Circuit Breaker** | 1sn/100 bad tick → tüm akışı durdur | `pipeline/validator.rs` |

---

## 9. Dosya Yapısı Özeti

```
cycle-engine/
├── gateway/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # pub mod binance, rate_gate
│       ├── binance.rs          # start_ws_client() / start_binance_ws_client()
│       └── rate_gate.rs        # RateGate — prosesler arası API rate kapısı (shm token bucket)
├── pipeline/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # pub mod tick, validator, queue
│       ├── tick.rs             # EventParser::parse() — SIMD JSON
│       ├── validator.rs        # DataValidator — Circuit Breaker
│       └── queue.rs            # LockFreeDispatcher — bounded channel
├── transport/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # pub mod events, wire, flow, ring_buffer, order_ring, calc_ring, stream_ring
│       ├── events.rs           # OwnedEvent, EventType
│       ├── wire.rs             # encode(), decode() — compact binary
│       ├── flow.rs             # FlowKind — 8 veri akışı tanımı (ring, bütçe, tablo)
│       ├── ring_buffer.rs      # GenerationalRingBuffer — akış ring'leri IPC
│       ├── order_ring.rs       # OrderRingBuffer — emir IPC
│       ├── calc_ring.rs        # CalcRingBuffer — indikatör IPC (1MB slot)
│       └── stream_ring.rs      # StreamRingBuffer — OHLCV IPC (4KB slot)
├── flows/
│   ├── Cargo.toml
│   ├── examples/
│   │   ├── probe.rs            # WS stream test aracı
│   │   └── probe2.rs
│   └── src/
│       ├── lib.rs              # flows::run(kind) — akış süreci orkestrasyonu
│       ├── parse.rs            # parse_for() — akış bazlı ayrıştırma
│       ├── rest.rs             # REST fallback — WS ile gelmeyen akışlar için
│       └── bin/
│           ├── flow_trade.rs       # flow-trade
│           ├── flow_depth.rs       # flow-depth
│           ├── flow_liquidation.rs # flow-liquidation
│           ├── flow_oi.rs          # flow-oi
│           ├── flow_funding.rs     # flow-funding
│           ├── flow_markprice.rs   # flow-markprice
│           ├── flow_lastprice.rs   # flow-lastprice
│           └── flow_indexprice.rs  # flow-indexprice
├── engine/
│   ├── Cargo.toml
│   ├── benches/
│   │   └── tick_benchmark.rs   # Criterion: parse + wire roundtrip
│   └── src/
│       ├── lib.rs              # pub mod engine, bridge
│       ├── bin/
│       │   └── strategy-console.rs  # strategy-console binary — orkestrasyon konsolu
│       ├── bridge.rs           # pub mod detector_bridge + yeniden export
│       ├── bridge/
│       │   └── detector_bridge.rs   # DetectorBridge — Scout Ring okuyucu
│       └── engine/
│           ├── mod.rs          # pub mod orchestrator, strategy_console
│           ├── orchestrator.rs # TitaniumOrchestrator — spin-loop
│           └── strategy_console.rs  # STRATEGY konsolu — StrategyOrchestrator yönetimi
├── persistence/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # pub mod timescaledb, clickhouse
│       ├── timescaledb.rs      # start_tsdb_writer() — TimescaleDB hypertable batch writer
│       └── clickhouse.rs       # ClickHouseAdapter — data lake
├── infra/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # pub mod hal, timer, pii, vault, redis, telemetry, ai
│       ├── hal/
│       │   ├── mod.rs
│       │   ├── cpu.rs          # pin_to_core()
│       │   └── memory.rs       # allocate_huge_buffer()
│       ├── timer/
│       │   ├── mod.rs
│       │   └── tsc.rs          # TscTimer — RDTSC intrinsic
│       ├── pii.rs              # PIIMasker — SHA-3
│       ├── redis.rs            # RedisAdapter — idempotency
│       ├── vault.rs            # VaultAdapter — JWT, key rotation
│       ├── telemetry.rs        # TelemetryAgent — Jaeger, Chaos Mesh
│       └── ai.rs               # AIAdapter — Isolation Forest, LLM
├── splash/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # show_splash(), show_splash_with()
│       └── main.rs             # Terminal animasyonu
├── cycle_engine_architecture.md
├── cycle_engine_processes.md
├── YAPILANDIRMA_PLANI.md
├── MIMARI_PLAN.md
└── walkthrough.md
```
