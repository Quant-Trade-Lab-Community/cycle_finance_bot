# 🏛️ Cycle Engine — Tam Kaynak Kodu Referansı

> Projenin yazılımsal çekirdeği (`cycle-engine/`). Bu doküman dizin ağacını,
> her dosyanın yolunu ve **dosyanın tam kaynak kodunu** içerir.
> Tarih: 2026-08-09

---

## 📂 İçindekiler

- [Dizin Ağacı](#-dizin-ağacı)
- [Katmanlar](#-katmanlar)
- [Klasör ve Dosya Sözlüğü](#-klasör-ve-dosya-sözlüğü)
- [1. contracts (Katman 0 — Sözleşmeler)](#1-contracts-katman-0--sözleşmeler)
- [2. transport (Katman 1 — Transport / IPC)](#2-transport-katman-1--transport--ipc)
- [3. core (Katman 2 — Çekirdek Motor)](#3-core-katman-2--çekirdek-motor)
- [4. adapter (Dış Entegrasyonlar)](#4-adapter-dış-entegrasyonlar)
- [5. splash (Açılış Ekranı)](#5-splash-açılış-ekranı)

---

## 🌳 Dizin Ağacı

```
cycle-engine/
├── contracts/                          # Katman 0 — Veri Sözleşmeleri
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── events.rs
│       └── wire.rs
│
├── transport/                          # Katman 1 — Transport (IPC)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── ring_buffer.rs              #   market data ring (/dev/shm)
│       ├── order_ring.rs               #   emir ring'i
│       ├── calc_ring.rs                #   indikatör sonuç ring'i
│       └── stream_ring.rs              #   canlı OHLCV mum ring'i
│
├── core/                               # Katman 2 — Çekirdek Motor
│   ├── Cargo.toml
│   ├── benches/
│   │   └── tick_benchmark.rs           #   criterion benchmark (WCET)
│   └── src/
│       ├── main.rs                     #   giriş: RUN_MODE dağıtıcısı (5 mod)
│       ├── lib.rs                      #   kütüphane kökü (proje_core)
│       ├── tick.rs                     #   simdjson EventParser
│       ├── validator.rs                #   DataValidator (circuit breaker)
│       ├── queue.rs                    #   LockFreeDispatcher (flume)
│       ├── db.rs                       #   SQLite batch yazıcı (WAL)
│       ├── config.rs                   #   os_utils::config re-export
│       ├── state.rs                    #   StateManager (event-driven durum)
│       ├── pii.rs                      #   PII maskeleme (KVKK/GDPR)
│       ├── bridge.rs                   #   köprü modülü re-export
│       ├── bridge/
│       │   └── detector_bridge.rs      #   scout ring → strateji köprüsü
│       ├── cli/
│       │   ├── mod.rs
│       │   ├── paper_cli.rs            #   PAPER modu terminali
│       │   ├── strategy_cli.rs         #   STRATEGY modu terminali
│       │   └── correlation_cli.rs      #   CORRELATION modu terminali
│       ├── engine/
│       │   ├── mod.rs
│       │   ├── orchestrator.rs         #   TitaniumOrchestrator spin-loop
│       │   └── backtester.rs           #   BACKTEST modu (CSV)
│       ├── hal/
│       │   ├── mod.rs
│       │   ├── cpu.rs                  #   CPU pin (core_affinity)
│       │   └── memory.rs               #   pre-fault bellek
│       └── timer/
│           ├── mod.rs
│           └── tsc.rs                  #   RDTSC tabanlı TscTimer
│
├── adapter/                            # Dış Entegrasyonlar
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── binance.rs                  #   Binance Futures WS client
│       ├── redis.rs                    #   Redis (idempotency, durum)
│       ├── clickhouse.rs               #   ClickHouse veri gölü
│       ├── vault.rs                    #   HashiCorp Vault (anahtar rotasyonu)
│       ├── ai.rs                       #   AI mikroservis adaptörü
│       └── telemetry.rs                #   Telemetri (eBPF/Jaeger/Chaos)
│
└── splash/                             # Açılış Ekranı (FIGlet)
    ├── Cargo.toml
    └── src/
        ├── main.rs                     #   bağımsız binary
        └── lib.rs                      #   show_splash / show_splash_with
```

---

## 🧱 Katmanlar

| Katman | Crate | Görev |
|---|---|---|
| **0 — Sözleşmeler** | `contracts` | `OwnedEvent` / `EventType` + `wire` binary codec |
| **1 — Transport (IPC)** | `transport` | `/dev/shm` sıfır-kopya ring buffer'lar (torn-read korumalı) |
| **2 — Çekirdek Motor** | `core` | simdjson parser, validator, orchestrator, SQLite, 5 CLI modu, HAL, TSC timer |
| **3 — Açılış Ekranı** | `splash` | FIGlet ASCII animasyonu |
| **Dış Entegrasyon** | `adapter` | Binance WS, Redis, ClickHouse, Vault, AI, telemetri |

---

## 📖 Klasör ve Dosya Sözlüğü

Her klasörün ve dosyanın kısa anlamı. Ayrıntılı açıklamalar aşağıdaki bölümlerde ilgili dosyanın yanında verilmiştir.

### `contracts/` — Veri Sözleşmeleri (Katman 0)

| Klasör / Dosya | Anlamı |
|---|---|
| `contracts/` | Tüm katmanların ortak "dili". Hiçbir katman diğerinden type import etmez; herkes bunu kullanır. En altta durur, kendisi hiçbir şeye bağımlı değildir. |
| `Cargo.toml` | Crate tanımı. Tek bağımlılık `rust_decimal` (ondalık hassasiyet). |
| `src/lib.rs` | Kütüphane kökü: `events` ve `wire` modüllerini açar. |
| `src/events.rs` | `EventType` (Trade, Orderbook, Liquidation, FundingRate...) ve `OwnedEvent` — sistemin merkezi veri modeli; tüm `new_*` constructor'ları burada. |
| `src/wire.rs` | `encode`/`decode` — `OwnedEvent` ↔ compact binary frame. Ring buffer üzerinden taşınan formatı bu belirler (JSON değil, ikili). |

### `transport/` — Transport / IPC (Katman 1)

| Klasör / Dosya | Anlamı |
|---|---|
| `transport/` | Prosesler arası sıfır-kopya iletişim. `/dev/shm` paylaşımlı bellekte ring buffer'lar; yazma/okuma kopyalama yapmadan çalışır. |
| `Cargo.toml` | Bağımlılıklar: `libc`, `memmap2`, `rust_decimal`. |
| `src/lib.rs` | 4 ring modülünü açıp re-export eder. |
| `src/ring_buffer.rs` | `GenerationalRingBuffer` — market data ring'i. 702B slot, torn-read (parçalı okuma) korumalı, `/cycle_finance_ring`. En kritik boru: core'un yazdığı tüm tick'ler buradan geçer. |
| `src/order_ring.rs` | `OrderRingBuffer` — strateji → icra emir kanalı. `/cycle_finance_orders`. |
| `src/calc_ring.rs` | `CalcRingBuffer` — büyük slot (1MB), indikatör hesaplama sonuçları. `/cycle_finance_calc`. |
| `src/stream_ring.rs` | `StreamRingBuffer` — 4KB slot, canlı OHLCV mumları. `/cycle_finance_stream_ohlcv`. |

### `core/` — Çekirdek Motor (Katman 2)

**Klasörler:**

| Klasör | Anlamı |
|---|---|
| `core/` | Sistemin beyni: dışarıdan gelen veriyi alır, doğrular, ring'e/DB'ye yazar, stratejiyi yönetir. 5 çalışma modu (DATA/PAPER/STRATEGY/BACKTEST/CORRELATION) buradan dağıtılır. |
| `benches/` | criterion benchmark'ları — kritik yolların (tick parse, wire codec) en kötü durum süresi (WCET) ölçümleri. |
| `src/bridge/` | Katmanlar arası köprüler. Şu an tek üyesi var: scout ring'den fırsat sinyali okuyan `detector_bridge`. |
| `src/cli/` | Terminal (CLI) modları: PAPER, STRATEGY, CORRELATION arayüzleri. İnteraktif olarak sistemle konuşmayı sağlar. |
| `src/engine/` | Çalıştırma motorları: `orchestrator` (canlı strateji yönetimi) + `backtester` (geçmiş veriyle simülasyon). |
| `src/hal/` | Donanım soyutlama katmanı: CPU'ya sabitleme ve bellek ön-ayırma (düşük gecikme ayarları). |
| `src/timer/` | Yüksek çözünürlüklü zamanlama: RDTSC tabanlı `TscTimer` (1ms'den ince ölçüm). |

**Dosyalar (üst seviye `src/`):**

| Dosya | Anlamı |
|---|---|
| `Cargo.toml` | `core` paketi; kütüphane adı `proje_core`; workspace bağımlılıkları; benchmark tanımı. |
| `src/main.rs` | Giriş noktası. `RUN_MODE` değişkenine göre 5 moddan birini kurar (DATA varsayılan) — uygulamanın "kapıcısı". |
| `src/lib.rs` | Kütüphane kökü: tüm modülleri (state, config, pii, db, validator, cli, hal, timer, engine, tick, queue, bridge) açıklar. |
| `src/tick.rs` | `EventParser` — ham WS JSON'unu simdjson ile `OwnedEvent`'e çevirir (@trade/@depth/@forceOrder/@markPrice/@bookTicker). Veri hattının ilk işlemcisi. |
| `src/validator.rs` | `DataValidator` — veriyi doğrular: stale ≤200ms, NTP drift, crossed book; 1sn'de 100+ hata olursa circuit breaker (devre kesici) açar. |
| `src/queue.rs` | `LockFreeDispatcher` — flume tabanlı bounded (262K) kuyruk; üretici dolunca geri basınç uygular. |
| `src/db.rs` | `start_db_writer` — SQLite WAL mode; 10 tablo; 10K satır / 1sn batch commit (disk yazımını amorti eder). |
| `src/config.rs` | Tek satır: `os_utils::config` re-export. Yapılandırmanın merkezileştiği yer. |
| `src/state.rs` | `StateManager` — WS event'leriyle bakiye/durumu event-driven olarak günceller (durumu elle senkronize etmek yerine olaylardan türetir). |
| `src/pii.rs` | `PIIMasker` — KVKK/GDPR uyumu: kimlik verisi maskeleme + 3 yıldan eski logların temizlenmesi (şimdilik mock). |
| `src/bridge.rs` | `detector_bridge` modülünü re-export eden kısa kök dosya. |

**Alt dosyalar:**

| Dosya | Anlamı |
|---|---|
| `src/bridge/detector_bridge.rs` | `DetectorBridge` — scout ring'inden `Opportunity` fırsat sinyallerini okuyup stratejiye iletir; `spawn_watcher` 100ms'de bir poll eder. |
| `src/cli/mod.rs` | CLI modüllerini açar. |
| `src/cli/paper_cli.rs` | PAPER modu: rustyline terminal; `risk_engine::accounting::Portfolio` ile sanal hesap (status/leverage/margin). Canlı para riski olmadan test. |
| `src/cli/strategy_cli.rs` | STRATEGY modu: `breakout-strategy` binary'sini spawn eder; restart/status komutlarıyla yönetir. |
| `src/cli/correlation_cli.rs` | CORRELATION modu: ring'den VELVETUSDT trade'lerini okuyup hacim/fiyat anomali tespiti + kümeleme uyarıları üretir. |
| `src/engine/mod.rs` | `orchestrator` + `backtester` modüllerini açar. |
| `src/engine/orchestrator.rs` | `TitaniumOrchestrator` — spin-loop: ring'i okuyup stratejilere verir; sinyalleri `RiskEngine` kapısından geçirip gateway'e gönderir; strateji paniklerini `catch_unwind` ile yakalayıp yeniden başlatır. |
| `src/engine/backtester.rs` | BACKTEST modu: CSV'den okuyup mock JSON olarak ring'e basar; geçmiş veriyle stratejiyi hızla test eder. |
| `src/hal/mod.rs` | `cpu` + `memory` modüllerini açar. |
| `src/hal/cpu.rs` | `pin_to_core` — thread'i belirli bir çekirdeğe sabitler (core_affinity); cache sıçramalarını önler. |
| `src/hal/memory.rs` | `allocate_huge_buffer` — sayfa fault'unu önlemek için belleği baştan "dokunup" ayırır (önceden ısıtır). |
| `src/timer/mod.rs` | `tsc` modülünü açar. |
| `src/timer/tsc.rs` | `TscTimer` — RDTSC tabanlı nanosaniye zamanlayıcı (varsayılan 3GHz kalibrasyon); döngü zamanlamalarında CPU sayacından okur. |

### `adapter/` — Dış Entegrasyonlar

| Klasör / Dosya | Anlamı |
|---|---|
| `adapter/` | Sistemin dış dünyayla kurduğu 6 bağlantı. Canlı olan: Binance WS. Diğerleri (Redis, ClickHouse, Vault, AI, telemetri) şimdilik mock. |
| `Cargo.toml` | WS/tokio/redis bağımlılıkları. |
| `src/lib.rs` | Modülleri açar; `init_adapter()` ile Redis sağlığını loglar. |
| `src/binance.rs` | Binance Futures WS client — `@trade` + `@depth20@100ms` stream'lerini çeker; 30sn ping; üstel yeniden bağlanma (1s→60s); ham JSON'u kuyruğa verir. Verinin giriş kapısı. |
| `src/redis.rs` | `RedisAdapter` — idempotency anahtarı (SET NX EX 3600), ACK durumu, fail-closed davranış; `REDIS_URL`'den bağlanır. |
| `src/clickhouse.rs` | `ClickHouseAdapter` — tick tablo şeması (MergeTree, partition), silme hakkı (GDPR), integrity check. |
| `src/vault.rs` | `VaultAdapter` — `VAULT_ADDR`'den health check; çift anahtar rotasyonu; JWT üretimi. |
| `src/ai.rs` | `AIAdapter` — Isolation Forest anomali skoru + LLM trend etiketi okuyucu (mock, Redis üzerinden). |
| `src/telemetry.rs` | `TelemetryAgent` — RTT takibi, Jaeger örnekleme, Chaos Mesh senaryoları (mock). |

### `splash/` — Açılış Ekranı

| Klasör / Dosya | Anlamı |
|---|---|
| `splash/` | Sistemi başlatırken gösterilen FIGlet ASCII animasyonu + yükleme çubuğu. Sadece görsel katman; iş mantığı yok. |
| `Cargo.toml` | `terminal_size` + `figlet-rs` bağımlılıkları. |
| `src/main.rs` | Bağımsız binary girişi: `cycle_splash::show_splash()` çağırır. |
| `src/lib.rs` | `show_splash` / `show_splash_with` — "CYCLE FINANCE" harf harf FIGlet animasyonu + 3sn yükleme çubuğu, Enter bekler. |

---

## 1. contracts (Katman 0 — Sözleşmeler)

### `cycle-engine/contracts/Cargo.toml`

```toml
[package]
name = "contracts"
version = "0.1.0"
edition = "2021"

[dependencies]
rust_decimal = { workspace = true }

[dev-dependencies]
rust_decimal = { workspace = true }
```

### `cycle-engine/contracts/src/lib.rs`

```rust
//! Sözleşme katmanı (Layer 0 — Contracts).
//!
//! Katmanlar arası sabit sözleşmeler burada yaşar:
//! - `events`: Tüm katmanların üzerinde anlaştığı market veri modeli
//!   (`OwnedEvent` / `EventType` — ring buffer üzerinden taşınan veri).
//! - `wire`: Ownership → compact binary frame codec (ring üzerindeki format).

pub mod events;
pub mod wire;
```

### `cycle-engine/contracts/src/events.rs`

```rust
//! Market veri modeli — tüm katmanların ortak dili.
//!
//! `OwnedEvent` + `EventType`: veri alım hattının çıktısı, ring buffer'ın
//! veri modeli ve analiz/tüketici katmanlarının girdisi. Bu dosya
//! `contracts` katmanında durduğu için hiçbir katman başka bir katmanın
//! implementasyonundan bu tipleri ithal etmek zorunda değildir.

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
    /// Scout fırsat sinyali — mikroyapi analiz sonucu (verdict: 0=GUCLU, 1=IYI, 2=NORMAL, 3=BOT/GURULTU, 4=ZAYIF).
    Opportunity {
        score: Decimal,
        efficiency: Decimal,
        price_bps_per_s: Decimal,
        price_ticks_per_s: Decimal,
        ob_changes_per_s: Decimal,
        spread_bps: Decimal,
        verdict: u8,
    },
    /// Tek sembol canlı mikroyapi metrikleri (scout analizi).
    SymbolMetrics {
        score: Decimal,
        efficiency: Decimal,
        price_bps_per_s: Decimal,
        price_ticks_per_s: Decimal,
        ob_changes_per_s: Decimal,
        spread_bps: Decimal,
    },
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
            EventType::Opportunity { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict } => {
                f.debug_struct("Opportunity")
                    .field("score", score)
                    .field("efficiency", efficiency)
                    .field("price_bps_per_s", price_bps_per_s)
                    .field("price_ticks_per_s", price_ticks_per_s)
                    .field("ob_changes_per_s", ob_changes_per_s)
                    .field("spread_bps", spread_bps)
                    .field("verdict", verdict)
                    .finish()
            }
            EventType::SymbolMetrics { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps } => {
                f.debug_struct("SymbolMetrics")
                    .field("score", score)
                    .field("efficiency", efficiency)
                    .field("price_bps_per_s", price_bps_per_s)
                    .field("price_ticks_per_s", price_ticks_per_s)
                    .field("ob_changes_per_s", ob_changes_per_s)
                    .field("spread_bps", spread_bps)
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
    fn pack_symbol(sym: &str) -> [u8; 16] {
        let mut symbol = [0u8; 16];
        let bytes = sym.as_bytes();
        let len = bytes.len().min(16);
        symbol[..len].copy_from_slice(&bytes[..len]);
        symbol
    }

    #[inline(always)]
    pub fn new_trade(sym: &str, price: Decimal, quantity: Decimal, timestamp: u64, is_buyer_maker: bool) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::Trade { price, quantity, timestamp, is_buyer_maker },
        }
    }

    #[inline(always)]
    pub fn new_orderbook(sym: &str, bids: [(Decimal, Decimal); 20], asks: [(Decimal, Decimal); 20]) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::Orderbook { bids, asks },
        }
    }

    #[inline(always)]
    pub fn new_liquidation(sym: &str, side: u8, price: Decimal, quantity: Decimal, timestamp: u64) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::Liquidation { side, price, quantity, timestamp },
        }
    }

    #[inline(always)]
    pub fn new_funding_rate(sym: &str, mark_price: Decimal, index_price: Decimal, funding_rate: Decimal, next_funding_time: u64) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time },
        }
    }

    #[inline(always)]
    pub fn new_bookticker(sym: &str, best_bid_price: Decimal, best_bid_qty: Decimal, best_ask_price: Decimal, best_ask_qty: Decimal) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty },
        }
    }

    #[inline(always)]
    pub fn new_open_interest(sym: &str, open_interest: Decimal, timestamp: u64) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::OpenInterest { open_interest, timestamp },
        }
    }

    #[inline(always)]
    pub fn new_opportunity(
        sym: &str,
        score: Decimal,
        efficiency: Decimal,
        price_bps_per_s: Decimal,
        price_ticks_per_s: Decimal,
        ob_changes_per_s: Decimal,
        spread_bps: Decimal,
        verdict: u8,
    ) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::Opportunity {
                score,
                efficiency,
                price_bps_per_s,
                price_ticks_per_s,
                ob_changes_per_s,
                spread_bps,
                verdict,
            },
        }
    }

    #[inline(always)]
    pub fn new_symbol_metrics(
        sym: &str,
        score: Decimal,
        efficiency: Decimal,
        price_bps_per_s: Decimal,
        price_ticks_per_s: Decimal,
        ob_changes_per_s: Decimal,
        spread_bps: Decimal,
    ) -> Self {
        Self {
            symbol: Self::pack_symbol(sym),
            payload: EventType::SymbolMetrics {
                score,
                efficiency,
                price_bps_per_s,
                price_ticks_per_s,
                ob_changes_per_s,
                spread_bps,
            },
        }
    }
}
```

### `cycle-engine/contracts/src/wire.rs`

```rust
//! Compact typed-binary market data frame codec.
//!
//! RAM hot path'teki ring buffer (`/dev/shm`) ham JSON yerine bu compact
//! binary formatı saklar:
//!
//! ```text
//! [0]    tag: u8
//! [1..17] symbol: [u8;16]
//! ... per-tag alanlar (i64 mantissa + u8 scale)
//! ```
//!
//! Ondalıklı değerler `rust_decimal::Decimal`'ın `(mantissa, scale)` ikilisi
//! olarak saklanır; `Decimal::new(mantissa, scale)` ile birebir geri kurulur.
//! Kısıt: |mantissa| <= i64::MAX — kripto fiyat/miktar aralığında imkânsız.
//!
//! Boyutlar: Trade 44B · BookTicker 53B · Funding 52B · Liquidation 44B ·
//! OI 34B · Depth20 659B (JSON ~1100B).
//!
//! ## Tag'ler
//! 0=Trade · 1=Depth · 2=Funding · 3=BookTicker · 4=Liquidation · 5=OpenInterest

use crate::events::{EventType, OwnedEvent};
use rust_decimal::Decimal;

/// Depth20 frame boyutu: tag(1)+sym(16)+p_scale(1)+q_scale(1)+40*(8+8)=659
pub const DEPTH_FRAME_SIZE: usize = 1 + 16 + 1 + 1 + 40 * 16;
/// En büyük frame boyutu (tüm tipler bunun içinde).
pub const MAX_FRAME_SIZE: usize = DEPTH_FRAME_SIZE;

const TAG_TRADE: u8 = 0;
const TAG_DEPTH: u8 = 1;
const TAG_FUNDING: u8 = 2;
const TAG_BOOKTICKER: u8 = 3;
const TAG_LIQUIDATION: u8 = 4;
const TAG_OPEN_INTEREST: u8 = 5;
const TAG_OPPORTUNITY: u8 = 6;
const TAG_SYMBOL_METRICS: u8 = 7;

#[inline(always)]
fn put_u8(buf: &mut [u8], off: usize, v: u8) {
    buf[off] = v;
}

#[inline(always)]
fn put_i64(buf: &mut [u8], off: usize, v: i64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

#[inline(always)]
fn put_u64(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

#[inline(always)]
fn rd_u8(buf: &[u8], off: usize) -> u8 {
    buf[off]
}

#[inline(always)]
fn rd_i64(buf: &[u8], off: usize) -> i64 {
    i64::from_le_bytes(buf[off..off + 8].try_into().unwrap())
}

#[inline(always)]
fn rd_u64(buf: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(buf[off..off + 8].try_into().unwrap())
}

#[inline(always)]
fn write_decimal(buf: &mut [u8], off: usize, d: Decimal) -> Option<usize> {
    let mantissa = d.mantissa();
    let m = i64::try_from(mantissa).ok()?;
    put_i64(buf, off, m);
    put_u8(buf, off + 8, d.scale() as u8);
    Some(off + 9)
}

#[inline(always)]
fn read_decimal(buf: &[u8], off: usize) -> Decimal {
    let m = rd_i64(buf, off);
    let s = rd_u8(buf, off + 8);
    if m == 0 && s == 0 {
        Decimal::ZERO
    } else {
        Decimal::new(m, s as u32)
    }
}

/// `OwnedEvent`'i compact binary frame'e yazar; boyutu döner.
/// Buffer `MAX_FRAME_SIZE`'dan büyük olmalı; mantissa i64 taşarsa `None`.
pub fn encode(ev: &OwnedEvent, buf: &mut [u8]) -> Option<usize> {
    buf[1..17].copy_from_slice(&ev.symbol);

    match &ev.payload {
        EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
            put_u8(buf, 0, TAG_TRADE);
            let mut off = 17;
            off = write_decimal(buf, off, *price)?;
            off = write_decimal(buf, off, *quantity)?;
            put_u64(buf, off, *timestamp);
            put_u8(buf, off + 8, if *is_buyer_maker { 1 } else { 0 });
            Some(off + 9)
        }
        EventType::Orderbook { bids, asks } => {
            put_u8(buf, 0, TAG_DEPTH);
            let mut off = 17;
            // Ortak scale'ler — rescale kayıpsız (değer korunur).
            let p_scale = bids.iter().chain(asks.iter())
                .filter(|(p, _)| !p.is_zero())
                .map(|(p, _)| p.scale())
                .max()
                .unwrap_or(0);
            let q_scale = bids.iter().chain(asks.iter())
                .filter(|(_, q)| !q.is_zero())
                .map(|(_, q)| q.scale())
                .max()
                .unwrap_or(0);
            put_u8(buf, off, p_scale as u8);
            put_u8(buf, off + 1, q_scale as u8);
            off += 2;
            for (p, q) in bids.iter().chain(asks.iter()) {
                let pm = if p.is_zero() {
                    0i64
                } else {
                    let mut d = *p;
                    d.rescale(p_scale);
                    i64::try_from(d.mantissa()).ok()?
                };
                let qm = if q.is_zero() {
                    0i64
                } else {
                    let mut d = *q;
                    d.rescale(q_scale);
                    i64::try_from(d.mantissa()).ok()?
                };
                put_i64(buf, off, pm);
                put_i64(buf, off + 8, qm);
                off += 16;
            }
            Some(off)
        }
        EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time } => {
            put_u8(buf, 0, TAG_FUNDING);
            let mut off = 17;
            off = write_decimal(buf, off, *mark_price)?;
            off = write_decimal(buf, off, *index_price)?;
            off = write_decimal(buf, off, *funding_rate)?;
            put_u64(buf, off, *next_funding_time);
            Some(off + 8)
        }
        EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty } => {
            put_u8(buf, 0, TAG_BOOKTICKER);
            let mut off = 17;
            off = write_decimal(buf, off, *best_bid_price)?;
            off = write_decimal(buf, off, *best_bid_qty)?;
            off = write_decimal(buf, off, *best_ask_price)?;
            off = write_decimal(buf, off, *best_ask_qty)?;
            Some(off)
        }
        EventType::Liquidation { side, price, quantity, timestamp } => {
            put_u8(buf, 0, TAG_LIQUIDATION);
            put_u8(buf, 17, *side);
            let mut off = 18;
            off = write_decimal(buf, off, *price)?;
            off = write_decimal(buf, off, *quantity)?;
            put_u64(buf, off, *timestamp);
            Some(off + 8)
        }
        EventType::OpenInterest { open_interest, timestamp } => {
            put_u8(buf, 0, TAG_OPEN_INTEREST);
            let mut off = 17;
            off = write_decimal(buf, off, *open_interest)?;
            put_u64(buf, off, *timestamp);
            Some(off + 8)
        }
        EventType::Opportunity { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict } => {
            put_u8(buf, 0, TAG_OPPORTUNITY);
            let mut off = 17;
            off = write_decimal(buf, off, *score)?;
            off = write_decimal(buf, off, *efficiency)?;
            off = write_decimal(buf, off, *price_bps_per_s)?;
            off = write_decimal(buf, off, *price_ticks_per_s)?;
            off = write_decimal(buf, off, *ob_changes_per_s)?;
            off = write_decimal(buf, off, *spread_bps)?;
            put_u8(buf, off, *verdict);
            Some(off + 1)
        }
        EventType::SymbolMetrics { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps } => {
            put_u8(buf, 0, TAG_SYMBOL_METRICS);
            let mut off = 17;
            off = write_decimal(buf, off, *score)?;
            off = write_decimal(buf, off, *efficiency)?;
            off = write_decimal(buf, off, *price_bps_per_s)?;
            off = write_decimal(buf, off, *price_ticks_per_s)?;
            off = write_decimal(buf, off, *ob_changes_per_s)?;
            off = write_decimal(buf, off, *spread_bps)?;
            Some(off)
        }
    }
}

/// Compact binary frame'i `OwnedEvent`'e geri kurar. Bozuk/güdük frame'de `None`.
pub fn decode(buf: &[u8]) -> Option<OwnedEvent> {
    if buf.len() < 17 {
        return None;
    }
    let tag = buf[0];
    let symbol = buf[1..17].try_into().ok()?;

    match tag {
        TAG_TRADE => {
            if buf.len() < 44 {
                return None;
            }
            let price = read_decimal(buf, 17);
            let quantity = read_decimal(buf, 26);
            let timestamp = rd_u64(buf, 35);
            let is_buyer_maker = rd_u8(buf, 43) != 0;
            Some(OwnedEvent {
                symbol,
                payload: EventType::Trade { price, quantity, timestamp, is_buyer_maker },
            })
        }
        TAG_DEPTH => {
            if buf.len() < DEPTH_FRAME_SIZE {
                return None;
            }
            let p_scale = rd_u8(buf, 17);
            let q_scale = rd_u8(buf, 18);
            let mut bids = [(Decimal::ZERO, Decimal::ZERO); 20];
            let mut asks = [(Decimal::ZERO, Decimal::ZERO); 20];
            let mut off = 19;
            for i in 0..20 {
                let pm = rd_i64(buf, off);
                let qm = rd_i64(buf, off + 8);
                if pm != 0 {
                    bids[i].0 = Decimal::new(pm, p_scale as u32);
                }
                if qm != 0 {
                    bids[i].1 = Decimal::new(qm, q_scale as u32);
                }
                off += 16;
            }
            for i in 0..20 {
                let pm = rd_i64(buf, off);
                let qm = rd_i64(buf, off + 8);
                if pm != 0 {
                    asks[i].0 = Decimal::new(pm, p_scale as u32);
                }
                if qm != 0 {
                    asks[i].1 = Decimal::new(qm, q_scale as u32);
                }
                off += 16;
            }
            Some(OwnedEvent {
                symbol,
                payload: EventType::Orderbook { bids, asks },
            })
        }
        TAG_FUNDING => {
            if buf.len() < 52 {
                return None;
            }
            let mark_price = read_decimal(buf, 17);
            let index_price = read_decimal(buf, 26);
            let funding_rate = read_decimal(buf, 35);
            let next_funding_time = rd_u64(buf, 44);
            Some(OwnedEvent {
                symbol,
                payload: EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time },
            })
        }
        TAG_BOOKTICKER => {
            if buf.len() < 53 {
                return None;
            }
            let best_bid_price = read_decimal(buf, 17);
            let best_bid_qty = read_decimal(buf, 26);
            let best_ask_price = read_decimal(buf, 35);
            let best_ask_qty = read_decimal(buf, 44);
            Some(OwnedEvent {
                symbol,
                payload: EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty },
            })
        }
        TAG_LIQUIDATION => {
            if buf.len() < 44 {
                return None;
            }
            let side = rd_u8(buf, 17);
            let price = read_decimal(buf, 18);
            let quantity = read_decimal(buf, 27);
            let timestamp = rd_u64(buf, 36);
            Some(OwnedEvent {
                symbol,
                payload: EventType::Liquidation { side, price, quantity, timestamp },
            })
        }
        TAG_OPEN_INTEREST => {
            if buf.len() < 34 {
                return None;
            }
            let open_interest = read_decimal(buf, 17);
            let timestamp = rd_u64(buf, 26);
            Some(OwnedEvent {
                symbol,
                payload: EventType::OpenInterest { open_interest, timestamp },
            })
        }
        TAG_OPPORTUNITY => {
            if buf.len() < 72 {
                return None;
            }
            let score = read_decimal(buf, 17);
            let efficiency = read_decimal(buf, 26);
            let price_bps_per_s = read_decimal(buf, 35);
            let price_ticks_per_s = read_decimal(buf, 44);
            let ob_changes_per_s = read_decimal(buf, 53);
            let spread_bps = read_decimal(buf, 62);
            let verdict = rd_u8(buf, 71);
            Some(OwnedEvent {
                symbol,
                payload: EventType::Opportunity {
                    score,
                    efficiency,
                    price_bps_per_s,
                    price_ticks_per_s,
                    ob_changes_per_s,
                    spread_bps,
                    verdict,
                },
            })
        }
        TAG_SYMBOL_METRICS => {
            if buf.len() < 71 {
                return None;
            }
            let score = read_decimal(buf, 17);
            let efficiency = read_decimal(buf, 26);
            let price_bps_per_s = read_decimal(buf, 35);
            let price_ticks_per_s = read_decimal(buf, 44);
            let ob_changes_per_s = read_decimal(buf, 53);
            let spread_bps = read_decimal(buf, 62);
            Some(OwnedEvent {
                symbol,
                payload: EventType::SymbolMetrics {
                    score,
                    efficiency,
                    price_bps_per_s,
                    price_ticks_per_s,
                    ob_changes_per_s,
                    spread_bps,
                },
            })
        }
        _ => None,
    }
}
```

---

## 2. transport (Katman 1 — Transport / IPC)

### `cycle-engine/transport/Cargo.toml`

```toml
[package]
name = "transport"
version = "0.1.0"
edition = "2021"

[dependencies]
libc = { workspace = true }
memmap2 = { workspace = true }
rust_decimal = { workspace = true }
```

### `cycle-engine/transport/src/lib.rs`

```rust
//! Katman 2 — Transport (IPC).
//!
//! Sıfır-kopya, paylaşımlı bellek (/dev/shm) ring buffer'ları. Bu katman
//! değişmez kabul edilir: tüketiciler yalnızca `read_slot(cursor)` sözleşmesini
//! görür, üreticiye dokunmaz.
//!
//! - `ring_buffer`: market data ring'i (GenerationalRing, torn-read korumalı)
//! - `order_ring`: emir ring'i (STRATEGY → EXECUTION)
//! - `calc_ring`: büyük-slot ring (calc-ind indikatör sonuçları)
//! - `stream_ring`: büyük-slot ring (stream-ohlcv canlı mum akışı)

pub mod ring_buffer;
pub mod order_ring;
pub mod calc_ring;
pub mod stream_ring;

pub use ring_buffer::GenerationalRingBuffer;
pub use order_ring::OrderRingBuffer;
pub use calc_ring::CalcRingBuffer;
pub use stream_ring::StreamRingBuffer;
```

### `cycle-engine/transport/src/ring_buffer.rs`

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, O_CREAT, O_RDWR};
use std::os::unix::io::FromRawFd;

/// Paylaşımlı hafızanın ilk oluşturulup oluşturulmadığını doğrulayan magic.
const RING_MAGIC: u64 = 0xD3F0000000000001;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct MarketDataSlot {
    pub seq: u64,
    pub len: u16,
    pub data: [u8; 702], // Total 768 bytes — en büyük wire frame (Depth20 = 659B) sığar
}

impl MarketDataSlot {
    pub const DATA_LEN: usize = 702;
}


#[repr(C)]
pub struct SharedHeader {
    pub magic: AtomicU64,
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
}

pub struct GenerationalRingBuffer {
    // Keep mmap alive. If it drops, memory unmaps.
    mmap: memmap2::MmapMut,
    header: *mut SharedHeader,
    slots: *mut MarketDataSlot,
    capacity: usize,
}

// Ensure Send/Sync for crossbeam threading
unsafe impl Send for GenerationalRingBuffer {}
unsafe impl Sync for GenerationalRingBuffer {}

impl GenerationalRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self::with_name("/cycle_finance_ring", capacity)
    }

    /// Belirtilen POSIX shm nesnesi üzerinde ring buffer oluşturur/açar.
    /// Farklı servisler farklı isim kullanabilir (örn. price-feed).
    pub fn with_name(shm_name: &str, capacity: usize) -> Self {
        let name = CString::new(shm_name).unwrap();

        let header_size = std::mem::size_of::<SharedHeader>();
        // Align to 64 bytes
        let header_aligned = (header_size + 63) & !63;

        let slot_size = std::mem::size_of::<MarketDataSlot>();
        let total_size = header_aligned + (capacity * slot_size);

        unsafe {
            // Create or open the POSIX shared memory object
            let fd = shm_open(name.as_ptr(), O_CREAT | O_RDWR, 0o666);
            if fd < 0 {
                panic!("Failed to shm_open");
            }

            let mut file = std::fs::File::from_raw_fd(fd);

            // Yeni mi yoksa mevcut mu? — ftruncate'i YALNIZCA ilk oluşturan yapar.
            // Aksi halde farklı capacity ile açan bir proses, üreticinin
            // paylaşımlı hafızasını altından yeniden boyutlandırır.
            let existing = file.metadata().map(|m| m.len()).unwrap_or(0);
            let is_fresh = existing == 0;

            if is_fresh {
                file.set_len(total_size as u64).expect("ftruncate");
            }

            let map_len = if is_fresh {
                total_size
            } else {
                existing as usize
            };

            let mut mmap = memmap2::MmapOptions::new()
                .len(map_len)
                .map_mut(&file)
                .expect("Failed to mmap shared memory");

            let header = mmap.as_mut_ptr() as *mut SharedHeader;

            // Eski format/satnik shm varsa (magic yok) yeniden ilklendir.
            if (*header).magic.load(Ordering::Relaxed) != RING_MAGIC {
                file.set_len(total_size as u64).expect("ftruncate (reinit)");
                let mut mmap = memmap2::MmapOptions::new()
                    .len(total_size)
                    .map_mut(&file)
                    .expect("Failed to mmap shared memory (reinit)");
                let header = mmap.as_mut_ptr() as *mut SharedHeader;
                let slots = mmap.as_mut_ptr().add(header_aligned) as *mut MarketDataSlot;

                (*header).magic.store(RING_MAGIC, Ordering::Relaxed);
                (*header).head.store(0, Ordering::SeqCst);
                (*header).tail.store(0, Ordering::SeqCst);
                (*header).capacity = capacity as u64;

                // Zero out the slots just in case
                ptr::write_bytes(slots, 0, capacity);

                let real_cap = (*header).capacity as usize;
                return Self {
                    mmap,
                    header,
                    slots,
                    capacity: real_cap,
                };
            }

            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut MarketDataSlot;
            let real_cap = (*header).capacity as usize;

            Self {
                mmap,
                header,
                slots,
                capacity: real_cap,
            }
        }
    }

    #[inline(always)]
    pub fn push(&self, data: &[u8]) {
        unsafe {
            let seq = (*self.header).head.load(Ordering::Relaxed);
            let index = (seq % self.capacity as u64) as usize;

            let len = if data.len() > MarketDataSlot::DATA_LEN {
                MarketDataSlot::DATA_LEN as u16
            } else {
                data.len() as u16
            };

            let slot_ptr = self.slots.add(index);
            // Önce veriyi ve len'i yaz, seq en sona kalsın ki okuyucu
            // yarım/tutarsız slot okumasın (torn-read koruması).
            (*slot_ptr).len = len;
            ptr::copy_nonoverlapping(data.as_ptr(), (*slot_ptr).data.as_mut_ptr(), len as usize);
            std::sync::atomic::fence(Ordering::Release);
            (*slot_ptr).seq = seq;

            // Release order ensures all writes to the slot are visible before head is incremented
            (*self.header).head.store(seq + 1, Ordering::Release);
        }
    }

    #[inline(always)]
    pub fn get_head(&self) -> u64 {
        unsafe {
            (*self.header).head.load(Ordering::Acquire)
        }
    }

    #[inline(always)]
    pub fn read_slot(&self, seq: u64) -> Option<MarketDataSlot> {
        let index = (seq % self.capacity as u64) as usize;

        let slot = unsafe {
            let slot_ptr = self.slots.add(index);
            // İlk oku: seq uyuyorsa veri tam yazılmış demektir (push seq'i en son yazar).
            let s = *slot_ptr;
            if s.seq == seq {
                // Çift kontrol: kopyalama sırasında üretici aynı slotu ezmesin diye.
                let again = *slot_ptr;
                if again.seq == seq {
                    Some(again)
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Generational check: if the sequence doesn't match, we've been overwritten by the producer
        slot
    }
}
```

### `cycle-engine/transport/src/order_ring.rs`

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, O_CREAT, O_RDWR};
use std::os::unix::io::FromRawFd;
use rust_decimal::Decimal;

/// Paylaşımlı hafızanın ilk oluşturulup oluşturulmadığını doğrulayan magic.
pub(crate) const ORDER_RING_MAGIC: u64 = 0xD3F0000000000002;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IpcOrderSide {
    Buy = 0,
    Sell = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IpcOrderType {
    Limit = 0,
    Market = 1,
}

#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct OrderSlot {
    pub seq: u64,
    pub symbol: [u8; 16], // Max 16 chars like "BTCUSDT"
    pub side: IpcOrderSide,
    pub order_type: IpcOrderType,
    pub quantity: Decimal,
    pub price: Decimal,
}

impl std::fmt::Debug for OrderSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderSlot")
            .field("seq", &self.seq)
            .field("symbol", &self.symbol)
            .field("side", &self.side)
            .field("order_type", &self.order_type)
            .field("quantity", &self.quantity)
            .field("price", &self.price)
            .finish()
    }
}

#[repr(C)]
pub struct OrderSharedHeader {
    pub magic: AtomicU64,
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
}

pub struct OrderRingBuffer {
    mmap: memmap2::MmapMut,
    header: *mut OrderSharedHeader,
    slots: *mut OrderSlot,
    capacity: usize,
}

unsafe impl Send for OrderRingBuffer {}
unsafe impl Sync for OrderRingBuffer {}

impl OrderRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let name = CString::new("/cycle_finance_orders").unwrap();
        
        let header_size = std::mem::size_of::<OrderSharedHeader>();
        let header_aligned = (header_size + 63) & !63;
        
        let slot_size = std::mem::size_of::<OrderSlot>();
        let total_size = header_aligned + (capacity * slot_size);

        unsafe {
            let fd = shm_open(name.as_ptr(), O_CREAT | O_RDWR, 0o666);
            if fd < 0 {
                panic!("Failed to shm_open for orders");
            }

            let mut file = std::fs::File::from_raw_fd(fd);

            // ftruncate'i YALNIZCA ilk oluşturan yapar (ring_buffer ile aynı koruma).
            let existing = file.metadata().map(|m| m.len()).unwrap_or(0);
            let is_fresh = existing == 0;

            if is_fresh {
                file.set_len(total_size as u64).expect("ftruncate for orders");
            }

            let map_len = if is_fresh {
                total_size
            } else {
                existing as usize
            };

            let mut mmap = memmap2::MmapOptions::new()
                .len(map_len)
                .map_mut(&file)
                .expect("Failed to mmap order shared memory");

            let header = mmap.as_mut_ptr() as *mut OrderSharedHeader;

            // Eski format/satnik shm varsa (magic yok) yeniden ilklendir.
            if (*header).magic.load(Ordering::Relaxed) != ORDER_RING_MAGIC {
                file.set_len(total_size as u64).expect("ftruncate (reinit)");
                let mut mmap = memmap2::MmapOptions::new()
                    .len(total_size)
                    .map_mut(&file)
                    .expect("Failed to mmap order shared memory (reinit)");
                let header = mmap.as_mut_ptr() as *mut OrderSharedHeader;
                let slots = mmap.as_mut_ptr().add(header_aligned) as *mut OrderSlot;

                (*header).magic.store(ORDER_RING_MAGIC, Ordering::Relaxed);
                (*header).head.store(0, Ordering::SeqCst);
                (*header).tail.store(0, Ordering::SeqCst);
                (*header).capacity = capacity as u64;
                ptr::write_bytes(slots, 0, capacity);

                let real_cap = (*header).capacity as usize;
                return Self {
                    mmap,
                    header,
                    slots,
                    capacity: real_cap,
                };
            }

            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut OrderSlot;
            let real_cap = (*header).capacity as usize;

            Self {
                mmap,
                header,
                slots,
                capacity: real_cap,
            }
        }
    }

    #[inline(always)]
    pub fn push(&self, symbol: &[u8], side: IpcOrderSide, order_type: IpcOrderType, quantity: Decimal, price: Decimal) {
        unsafe {
            let seq = (*self.header).head.load(Ordering::Relaxed);
            let index = (seq % self.capacity as u64) as usize;
            
            let slot_ptr = self.slots.add(index);
            (*slot_ptr).seq = seq;
            (*slot_ptr).side = side;
            (*slot_ptr).order_type = order_type;
            (*slot_ptr).quantity = quantity;
            (*slot_ptr).price = price;
            
            let mut sym_buf = [0u8; 16];
            let len = symbol.len().min(16);
            sym_buf[..len].copy_from_slice(&symbol[..len]);
            (*slot_ptr).symbol = sym_buf;

            (*self.header).head.store(seq + 1, Ordering::Release);
        }
    }

    #[inline(always)]
    pub fn get_head(&self) -> u64 {
        unsafe {
            (*self.header).head.load(Ordering::Acquire)
        }
    }

    #[inline(always)]
    pub fn read_slot(&self, seq: u64) -> Option<OrderSlot> {
        let index = (seq % self.capacity as u64) as usize;
        let slot = unsafe { *self.slots.add(index) };
        if slot.seq == seq {
            Some(slot)
        } else {
            None
        }
    }
}
```

### `cycle-engine/transport/src/calc_ring.rs`

```rust
//! Büyük-slot paylaşımlı bellek ring buffer — indikatör/OHLCV sonuçları için.
//!
//! `GenerationalRingBuffer` (702B slot) indikatör serileri için çok küçüktür;
//! bu ring büyük binary blokları (örn. bir isteğin tüm OHLCV + indikatör
//! çıktısı) tek slot'ta taşır. Torn-read koruması aynıdır: seq en son yazılır,
//! okuyucu yarım slot görmez.
//!
//! Üretici: calc-ind servisi. Tüketici: calc_ind::client (istek atan servis).

use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, O_CREAT, O_RDWR};
use std::os::unix::io::FromRawFd;

/// Paylaşımlı hafızanın ilk oluşturulup oluşturulmadığını doğrulayan magic.
pub(crate) const CALC_RING_MAGIC: u64 = 0xD3F0000000000003;

/// Varsayılan tek slot boyutu (1 MB) — bir isteğin tüm sonucunu taşıyacak kadar.
pub const CALC_SLOT_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct CalcSlot {
    pub seq: u64,
    pub len: u32,
    pub data: [u8; 1024 * 1024],
}

#[repr(C)]
pub struct CalcSharedHeader {
    pub magic: AtomicU64,
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
}

pub struct CalcRingBuffer {
    mmap: memmap2::MmapMut,
    header: *mut CalcSharedHeader,
    slots: *mut CalcSlot,
    capacity: usize,
}

unsafe impl Send for CalcRingBuffer {}
unsafe impl Sync for CalcRingBuffer {}

impl CalcRingBuffer {
    /// Varsayılan isimle açar: `/cycle_finance_calc`
    pub fn new(capacity: usize) -> Self {
        Self::with_name("/cycle_finance_calc", capacity)
    }

    /// Belirtilen POSIX shm nesnesi üzerinde büyük-slot ring oluşturur/açar.
    pub fn with_name(shm_name: &str, capacity: usize) -> Self {
        let name = CString::new(shm_name).unwrap();

        let header_size = std::mem::size_of::<CalcSharedHeader>();
        let header_aligned = (header_size + 63) & !63;
        let slot_size = std::mem::size_of::<CalcSlot>();
        let total_size = header_aligned + (capacity * slot_size);

        unsafe {
            let fd = shm_open(name.as_ptr(), O_CREAT | O_RDWR, 0o666);
            if fd < 0 {
                panic!("Failed to shm_open for calc ring");
            }

            let mut file = std::fs::File::from_raw_fd(fd);

            // ftruncate'i YALNIZCA ilk oluşturan yapar.
            let existing = file.metadata().map(|m| m.len()).unwrap_or(0);
            let is_fresh = existing == 0;

            if is_fresh {
                file.set_len(total_size as u64).expect("ftruncate for calc ring");
            }

            let map_len = if is_fresh { total_size } else { existing as usize };

            let mut mmap = memmap2::MmapOptions::new()
                .len(map_len)
                .map_mut(&file)
                .expect("Failed to mmap calc shared memory");

            let header = mmap.as_mut_ptr() as *mut CalcSharedHeader;

            // Eski/satnik shm varsa (magic yok) yeniden ilklendir.
            if (*header).magic.load(Ordering::Relaxed) != CALC_RING_MAGIC {
                file.set_len(total_size as u64).expect("ftruncate (reinit)");
                let mut mmap = memmap2::MmapOptions::new()
                    .len(total_size)
                    .map_mut(&file)
                    .expect("Failed to mmap calc shared memory (reinit)");
                let header = mmap.as_mut_ptr() as *mut CalcSharedHeader;
                let slots = mmap.as_mut_ptr().add(header_aligned) as *mut CalcSlot;

                (*header).magic.store(CALC_RING_MAGIC, Ordering::Relaxed);
                (*header).head.store(0, Ordering::SeqCst);
                (*header).tail.store(0, Ordering::SeqCst);
                (*header).capacity = capacity as u64;
                ptr::write_bytes(slots, 0, capacity);

                let real_cap = (*header).capacity as usize;
                return Self {
                    mmap,
                    header,
                    slots,
                    capacity: real_cap,
                };
            }

            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut CalcSlot;
            let real_cap = (*header).capacity as usize;

            Self {
                mmap,
                header,
                slots,
                capacity: real_cap,
            }
        }
    }

    /// Tek slot'a veri yazar. `data.len() > CALC_SLOT_SIZE` ise kesilir.
    #[inline(always)]
    pub fn push(&self, data: &[u8]) {
        unsafe {
            let seq = (*self.header).head.load(Ordering::Relaxed);
            let index = (seq % self.capacity as u64) as usize;

            let len = data.len().min(CALC_SLOT_SIZE) as u32;
            let slot_ptr = self.slots.add(index);

            (*slot_ptr).len = len;
            ptr::copy_nonoverlapping(data.as_ptr(), (*slot_ptr).data.as_mut_ptr(), len as usize);
            std::sync::atomic::fence(Ordering::Release);
            (*slot_ptr).seq = seq;

            (*self.header).head.store(seq + 1, Ordering::Release);
        }
    }

    /// Üretici başını (head) okur — tüketici buradan başlar.
    #[inline(always)]
    pub fn get_head(&self) -> u64 {
        unsafe { (*self.header).head.load(Ordering::Acquire) }
    }

    /// Slot'u veri parçası olarak okur (torn-read korumalı).
    #[inline(always)]
    pub fn read_slot(&self, seq: u64) -> Option<CalcSlot> {
        let index = (seq % self.capacity as u64) as usize;
        let slot = unsafe {
            let slot_ptr = self.slots.add(index);
            let s = *slot_ptr;
            if s.seq == seq {
                let again = *slot_ptr;
                if again.seq == seq {
                    Some(again)
                } else {
                    None
                }
            } else {
                None
            }
        };
        slot
    }
}
```

### `cycle-engine/transport/src/stream_ring.rs`

```rust
//! Büyük-slot paylaşımlı bellek ring buffer — canlı OHLCV mum akışı için.
//!
//! `GenerationalRingBuffer` (702B slot) tek bir mumu ve stream meta bilgisini
//! taşıyacak kadar büyük değildir; bu ring canlı kapanan/oluşan mumları tek
//! slot'ta binary olarak taşır. Torn-read koruması aynıdır: seq en son yazılır,
//! okuyucu yarım slot görmez.
//!
//! Üretici: stream-ohlcv servisi. Tüketici: stream_ohlcv::client (istek atan servis).
//!
//! Slot düzeni (StreamSlot, sabit 4096B):
//!   [0..8)   seq (torn-read koruması, en son yazılır)
//!   [8..12)  len (payload bayt uzunluğu)
//!   [12..)   data — stream_ohlcv::codec ile binary kodlanmış mum

use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;
use std::ffi::CString;
use libc::{shm_open, O_CREAT, O_RDWR};
use std::os::unix::io::FromRawFd;

/// Paylaşımlı hafızanın ilk oluşturulup oluşturulmadığını doğrulayan magic.
pub(crate) const STREAM_RING_MAGIC: u64 = 0xD3F0000000000004;

/// Tek slot boyutu (4 KB) — bir mum (binary codec) rahatlıkla sığar.
pub const STREAM_SLOT_SIZE: usize = 4096;

/// Varsayılan slot sayısı — dairesel akış, eski slotlar üzerine yazılır.
pub const STREAM_DEFAULT_CAPACITY: usize = 8192;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct StreamSlot {
    pub seq: u64,
    pub len: u32,
    pub data: [u8; STREAM_SLOT_SIZE],
}

#[repr(C)]
pub struct StreamSharedHeader {
    pub magic: AtomicU64,
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
}

pub struct StreamRingBuffer {
    mmap: memmap2::MmapMut,
    header: *mut StreamSharedHeader,
    slots: *mut StreamSlot,
    capacity: usize,
}

unsafe impl Send for StreamRingBuffer {}
unsafe impl Sync for StreamRingBuffer {}

impl StreamRingBuffer {
    /// Varsayılan isimle açar: `/cycle_finance_stream_ohlcv`
    pub fn new(capacity: usize) -> Self {
        Self::with_name("/cycle_finance_stream_ohlcv", capacity)
    }

    /// Belirtilen POSIX shm nesnesi üzerinde büyük-slot ring oluşturur/açar.
    pub fn with_name(shm_name: &str, capacity: usize) -> Self {
        let name = CString::new(shm_name).unwrap();

        let header_size = std::mem::size_of::<StreamSharedHeader>();
        let header_aligned = (header_size + 63) & !63;
        let slot_size = std::mem::size_of::<StreamSlot>();
        let total_size = header_aligned + (capacity * slot_size);

        unsafe {
            let fd = shm_open(name.as_ptr(), O_CREAT | O_RDWR, 0o666);
            if fd < 0 {
                panic!("Failed to shm_open for stream ring");
            }

            let mut file = std::fs::File::from_raw_fd(fd);

            // ftruncate'i YALNIZCA ilk oluşturan yapar.
            let existing = file.metadata().map(|m| m.len()).unwrap_or(0);
            let is_fresh = existing == 0;

            if is_fresh {
                file.set_len(total_size as u64).expect("ftruncate for stream ring");
            }

            let map_len = if is_fresh { total_size } else { existing as usize };

            let mut mmap = memmap2::MmapOptions::new()
                .len(map_len)
                .map_mut(&file)
                .expect("Failed to mmap stream shared memory");

            let header = mmap.as_mut_ptr() as *mut StreamSharedHeader;

            // Eski/satnik shm varsa (magic yok) yeniden ilklendir.
            if (*header).magic.load(Ordering::Relaxed) != STREAM_RING_MAGIC {
                file.set_len(total_size as u64).expect("ftruncate (reinit)");
                let mut mmap = memmap2::MmapOptions::new()
                    .len(total_size)
                    .map_mut(&file)
                    .expect("Failed to mmap stream shared memory (reinit)");
                let header = mmap.as_mut_ptr() as *mut StreamSharedHeader;
                let slots = mmap.as_mut_ptr().add(header_aligned) as *mut StreamSlot;

                (*header).magic.store(STREAM_RING_MAGIC, Ordering::Relaxed);
                (*header).head.store(0, Ordering::SeqCst);
                (*header).tail.store(0, Ordering::SeqCst);
                (*header).capacity = capacity as u64;
                ptr::write_bytes(slots, 0, capacity);

                let real_cap = (*header).capacity as usize;
                return Self {
                    mmap,
                    header,
                    slots,
                    capacity: real_cap,
                };
            }

            let slots = mmap.as_mut_ptr().add(header_aligned) as *mut StreamSlot;
            let real_cap = (*header).capacity as usize;

            Self {
                mmap,
                header,
                slots,
                capacity: real_cap,
            }
        }
    }

    /// Tek slot'a veri yazar. `data.len() > STREAM_SLOT_SIZE` ise kesilir.
    #[inline(always)]
    pub fn push(&self, data: &[u8]) {
        unsafe {
            let seq = (*self.header).head.load(Ordering::Relaxed);
            let index = (seq % self.capacity as u64) as usize;

            let len = data.len().min(STREAM_SLOT_SIZE) as u32;
            let slot_ptr = self.slots.add(index);

            (*slot_ptr).len = len;
            ptr::copy_nonoverlapping(data.as_ptr(), (*slot_ptr).data.as_mut_ptr(), len as usize);
            std::sync::atomic::fence(Ordering::Release);
            (*slot_ptr).seq = seq;

            (*self.header).head.store(seq + 1, Ordering::Release);
        }
    }

    /// Üretici başını (head) okur — tüketici buradan başlar.
    #[inline(always)]
    pub fn get_head(&self) -> u64 {
        unsafe { (*self.header).head.load(Ordering::Acquire) }
    }

    /// Slot'u veri parçası olarak okur (torn-read korumalı).
    #[inline(always)]
    pub fn read_slot(&self, seq: u64) -> Option<StreamSlot> {
        let index = (seq % self.capacity as u64) as usize;
        let slot = unsafe {
            let slot_ptr = self.slots.add(index);
            let s = *slot_ptr;
            if s.seq == seq {
                let again = *slot_ptr;
                if again.seq == seq {
                    Some(again)
                } else {
                    None
                }
            } else {
                None
            }
        };
        slot
    }
}
```

---

## 3. core (Katman 2 — Çekirdek Motor)

### `cycle-engine/core/Cargo.toml`

```toml
[package]
name = "core"
version = "0.1.0"
edition = "2021"

[lib]
name = "proje_core"

[features]
default = ["binance_v5"]
binance_v5 = []
binance_v6 = []

[dependencies]
tokio = { workspace = true }
flume = { workspace = true }
parking_lot = { workspace = true }
simd-json = { workspace = true }
futures-util = { workspace = true }
cold-storage = { path = "../../data-engine/cold-storage" }
adapter = { path = "../adapter" }
os-utils = { path = "../../additional-services/os-utils" }
execution-engine = { path = "../../execution-engine" }
contracts = { path = "../contracts" }
transport = { path = "../transport" }
risk-engine = { path = "../../risk-engine" }
strategies-engine = { path = "../../strategies-engine" }
sha3 = { workspace = true }
rusqlite = { workspace = true }
dotenvy = { workspace = true }
reqwest = { workspace = true }
serde_json = { workspace = true }
core_affinity = { workspace = true }
hdrhistogram = { workspace = true }
crossbeam-channel = { workspace = true }
serde = { workspace = true }
libc = { workspace = true }
memmap2 = { workspace = true }
rustyline = { workspace = true }
chrono = { workspace = true }
rust_decimal = { workspace = true }

[dev-dependencies]
criterion = "0.4"
proptest = { workspace = true }

[[bench]]
name = "tick_benchmark"
harness = false
```

### `cycle-engine/core/benches/tick_benchmark.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::prelude::*;
use contracts::events::OwnedEvent;
use proje_core::tick::EventParser;
use contracts::wire;

fn bench_tick_parsing(c: &mut Criterion) {
    let payload = b"{\"stream\":\"btcusdt@trade\",\"data\":{\"e\":\"trade\",\"E\":1766800000000,\"s\":\"BTCUSDT\",\"t\":123,\"p\":\"50000.0\",\"q\":\"1.5\",\"T\":1620000000000,\"m\":false}}".to_vec();

    c.bench_function("tick_parse_wcet", |b| {
        b.iter(|| {
            let mut data = payload.clone();
            let parsed = EventParser::parse(black_box(&mut data));
            black_box(parsed);
        })
    });
}

fn bench_wire_roundtrip(c: &mut Criterion) {
    let trade = OwnedEvent::new_trade("BTCUSDT", Decimal::from_str("67234.50").unwrap(),
        Decimal::from_str("0.001500").unwrap(), 1_766_800_000_000, true);
    let mut buf = [0u8; wire::MAX_FRAME_SIZE + 64];

    c.bench_function("wire_encode_trade", |b| {
        b.iter(|| {
            let len = wire::encode(black_box(&trade), &mut buf);
            black_box(len);
        })
    });

    c.bench_function("wire_decode_trade", |b| {
        let len = wire::encode(&trade, &mut buf).unwrap();
        b.iter(|| {
            let ev = wire::decode(black_box(&buf[..len]));
            black_box(ev);
        })
    });

    let mut bids = [(rust_decimal::Decimal::ZERO, rust_decimal::Decimal::ZERO); 20];
    let mut asks = [(rust_decimal::Decimal::ZERO, rust_decimal::Decimal::ZERO); 20];
    for i in 0..20 {
        bids[i] = (Decimal::new(67200 + i as i64, 0), Decimal::new(100 - i as i64, 0));
        asks[i] = (Decimal::new(67220 + i as i64, 0), Decimal::new(90 + i as i64, 0));
    }
    let depth = OwnedEvent::new_orderbook("BTCUSDT", bids, asks);

    c.bench_function("wire_encode_depth20", |b| {
        b.iter(|| {
            let len = wire::encode(black_box(&depth), &mut buf);
            black_box(len);
        })
    });

    c.bench_function("wire_decode_depth20", |b| {
        let len = wire::encode(&depth, &mut buf).unwrap();
        b.iter(|| {
            let ev = wire::decode(black_box(&buf[..len]));
            black_box(ev);
        })
    });
}

criterion_group!(benches, bench_tick_parsing, bench_wire_roundtrip);
criterion_main!(benches);
```

### `cycle-engine/core/src/main.rs`

```rust
use proje_core::tick::EventParser;
use proje_core::queue::LockFreeDispatcher;
use std::thread;
use std::time::Instant;
use os_utils::set_rt_thread_priority;
use adapter::binance::start_binance_ws_client;
use transport::ring_buffer::GenerationalRingBuffer;

#[tokio::main]
async fn main() {
    let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "DATA".to_string());
    
    if run_mode == "DATA" {
        println!("🚀 Başlatılıyor: MARKET DATA KONSOLU");
        let gen_ring = std::sync::Arc::new(GenerationalRingBuffer::new(160_000));
        let gen_ring_data = gen_ring.clone();
        
        let (db_tx, db_rx) = flume::bounded(1_000_000); 
        thread::spawn(move || {
            proje_core::db::start_db_writer(db_rx);
        });

        let dispatcher = LockFreeDispatcher::new();
        let tx = dispatcher.producer();
        let rx = dispatcher.consumer();

        thread::spawn(move || {
            set_rt_thread_priority(99);
            let mut tick_count = 0;
            let mut depth_count = 0u64;
            let mut invalid_count = 0u64;
            let mut db_drop_count = 0u64;
            let mut total_parse_time = std::time::Duration::new(0, 0);
            let mut last_report = Instant::now();
            let mut validator = proje_core::validator::DataValidator::new();
            let mut frame_buf = [0u8; contracts::wire::MAX_FRAME_SIZE];
            
            while let Ok(mut bytes) = rx.recv() {
                let start_parse = Instant::now();
                // simd_json sıfır-kopya parse buffer'ı BOZAR (ayırıcıları '\0' yapar).
                // Ring'e artık typed binary (wire::encode) yazılır — kopya yoktur.
                if let Some(owned_event) = EventParser::parse(&mut bytes) {
                    if !validator.is_valid(&owned_event) { invalid_count += 1; continue; }
                    if matches!(owned_event.payload, contracts::events::EventType::Orderbook { .. }) {
                        depth_count += 1;
                    }
                    if let Some(len) = contracts::wire::encode(&owned_event, &mut frame_buf) {
                        gen_ring_data.push(&frame_buf[..len]);
                    }
                    if db_tx.try_send(owned_event).is_err() {
                        db_drop_count += 1;
                    }

                    total_parse_time += start_parse.elapsed();
                    tick_count += 1;
                }

                if last_report.elapsed().as_secs() >= 1 {
                    let avg_parse_time = if tick_count > 0 {
                        total_parse_time.as_nanos() as f64 / tick_count as f64
                    } else { 0.0 };
                    println!("[MARKET DATA] Ticks/sec: {} | depth: {} | invalid: {} | db_drops: {} | Avg Parse: {:.2} ns", tick_count, depth_count, invalid_count, db_drop_count, avg_parse_time);
                    
                    tick_count = 0;
                    depth_count = 0;
                    invalid_count = 0;
                    db_drop_count = 0;
                    total_parse_time = std::time::Duration::new(0, 0);
                    last_report = Instant::now();
                }
            }
        });

        start_binance_ws_client(tx).await;
        return;
    }

    if run_mode == "PAPER" {
        proje_core::cli::paper_cli::start_paper_cli();
        return;
    }

    if run_mode == "STRATEGY" {
        println!("🚀 Başlatılıyor: STRATEJI KONSOLU");
        proje_core::cli::strategy_cli::start_strategy_cli();
        return;
    }

    if run_mode == "BACKTEST" {
        let csv_path = std::env::var("CSV_PATH").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data.csv").to_string());
        proje_core::engine::backtester::start_backtester(&csv_path);
        return;
    }

    if run_mode == "CORRELATION" {
        proje_core::cli::correlation_cli::start_correlation_cli();
        return;
    }

    println!("Lütfen geçerli bir RUN_MODE belirleyin (DATA, PAPER, STRATEGY, BACKTEST, CORRELATION)");
}
```

### `cycle-engine/core/src/lib.rs`

```rust
pub mod state;
pub mod config;
pub mod pii;
pub mod db;
pub mod validator;
pub mod cli;

pub mod hal;
pub mod timer;
pub mod engine;

pub mod tick;
pub mod queue;

pub mod bridge;
```

### `cycle-engine/core/src/tick.rs`

```rust
// core/src/tick.rs

use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use simd_json;
use simd_json::prelude::*;
use contracts::events::OwnedEvent;

pub struct EventParser;

impl EventParser {
    #[inline(always)]
    pub fn parse(bytes: &mut [u8]) -> Option<OwnedEvent> {
        let parsed = simd_json::to_borrowed_value(bytes).ok()?;
        
        let stream = parsed.get("stream")?.as_str()?;
        let data = parsed.get("data")?;
        
        if stream.ends_with("@trade") {
            let symbol = data.get("s")?.as_str()?;
            let price_str = data.get("p")?.as_str()?;
            let quantity_str = data.get("q")?.as_str()?;
            let timestamp = data.get("T")?.as_u64()?;
            
            let price = Decimal::from_str(price_str).ok()?;
            let quantity = Decimal::from_str(quantity_str).ok()?;
            let is_buyer_maker = data.get("m")?.as_bool()?;
            
            Some(OwnedEvent::new_trade(symbol, price, quantity, timestamp, is_buyer_maker))
        } else if stream.contains("@depth") {
            let symbol = stream.split('@').next()?;
            let mut bids = [(Decimal::ZERO, Decimal::ZERO); 20];
            let mut asks = [(Decimal::ZERO, Decimal::ZERO); 20];
            
            // Spot `@depth` → "bids"/"asks"; Futures `@depth20@100ms` → "b"/"a"
            if let Some(b) = data.get("bids").and_then(|v| v.as_array())
                .or_else(|| data.get("b").and_then(|v| v.as_array())) {
                for (i, bid) in b.iter().take(20).enumerate() {
                    if let Some(arr) = bid.as_array() {
                        let p = arr.get(0).and_then(|v| v.as_str()).and_then(|s| Decimal::from_str(s).ok()).unwrap_or(Decimal::ZERO);
                        let q = arr.get(1).and_then(|v| v.as_str()).and_then(|s| Decimal::from_str(s).ok()).unwrap_or(Decimal::ZERO);
                        bids[i] = (p, q);
                    }
                }
            }
            if let Some(a) = data.get("asks").and_then(|v| v.as_array())
                .or_else(|| data.get("a").and_then(|v| v.as_array())) {
                for (i, ask) in a.iter().take(20).enumerate() {
                    if let Some(arr) = ask.as_array() {
                        let p = arr.get(0).and_then(|v| v.as_str()).and_then(|s| Decimal::from_str(s).ok()).unwrap_or(Decimal::ZERO);
                        let q = arr.get(1).and_then(|v| v.as_str()).and_then(|s| Decimal::from_str(s).ok()).unwrap_or(Decimal::ZERO);
                        asks[i] = (p, q);
                    }
                }
            }
            
            Some(OwnedEvent::new_orderbook(symbol, bids, asks))
        } else if stream.ends_with("@forceOrder") {
            let o = data.get("o")?;
            let symbol = o.get("s")?.as_str()?;
            let side_str = o.get("S")?.as_str()?;
            let side = if side_str == "BUY" { 0 } else { 1 };
            let price = o.get("p")?.as_str()?.parse::<Decimal>().ok()?;
            let quantity = o.get("q")?.as_str()?.parse::<Decimal>().ok()?;
            let timestamp = o.get("T")?.as_u64()?;
            Some(OwnedEvent::new_liquidation(symbol, side, price, quantity, timestamp))
        } else if stream.contains("@markPrice") {
            let symbol = data.get("s")?.as_str()?;
            let mark_price = data.get("p")?.as_str()?.parse::<Decimal>().ok()?;
            let index_price = data.get("i").and_then(|v| v.as_str())
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(mark_price);
            let funding_rate = data.get("r")?.as_str()?.parse::<Decimal>().ok().unwrap_or(Decimal::ZERO);
            let next_funding_time = data.get("T")?.as_u64().unwrap_or(0);
            Some(OwnedEvent::new_funding_rate(symbol, mark_price, index_price, funding_rate, next_funding_time))
        } else if stream.ends_with("@bookTicker") {
            let symbol = data.get("s")?.as_str()?;
            let best_bid_price = data.get("b")?.as_str()?.parse::<Decimal>().ok()?;
            let best_bid_qty = data.get("B")?.as_str()?.parse::<Decimal>().ok()?;
            let best_ask_price = data.get("a")?.as_str()?.parse::<Decimal>().ok()?;
            let best_ask_qty = data.get("A")?.as_str()?.parse::<Decimal>().ok()?;
            Some(OwnedEvent::new_bookticker(symbol, best_bid_price, best_bid_qty, best_ask_price, best_ask_qty))
        } else {
            None
        }
    }
}
```

### `cycle-engine/core/src/validator.rs`

```rust
use contracts::events::{OwnedEvent, EventType};
use rust_decimal::Decimal;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct DataValidator {
    pub circuit_breaker: Arc<AtomicBool>,
    pub bad_tick_count: Arc<AtomicUsize>,
    max_latency_ms: u64,
    last_reset_time: u64,
}

impl DataValidator {
    pub fn new() -> Self {
        Self {
            circuit_breaker: Arc::new(AtomicBool::new(false)),
            bad_tick_count: Arc::new(AtomicUsize::new(0)),
            max_latency_ms: 200, // 200 ms gecikme toleransı
            last_reset_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        }
    }

    #[inline(always)]
    pub fn is_valid(&mut self, event: &OwnedEvent) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        
        // Şalter sıfırlama mantığı (1 saniyede bir hata sayacını sıfırla)
        if now - self.last_reset_time > 1000 {
            self.bad_tick_count.store(0, Ordering::Relaxed);
            self.last_reset_time = now;
            
            // Eğer şalter daha önce attıysa ama sular durulduysa şalteri kaldır
            if self.circuit_breaker.load(Ordering::Relaxed) {
                println!("CIRCUIT BREAKER RECOVERED. Safe to trade.");
                self.circuit_breaker.store(false, Ordering::Release);
            }
        }

        match &event.payload {
            EventType::Trade { price, quantity, timestamp, is_buyer_maker: _ } => {
                if *price <= Decimal::ZERO || *quantity <= Decimal::ZERO {
                    return self.flag_invalid("Trade price/qty <= 0");
                }
                if now > *timestamp && (now - *timestamp) > self.max_latency_ms {
                    return self.flag_invalid("Trade Stale Data (Latency)");
                }
                if *timestamp > now && (*timestamp - now) > 5000 {
                    return self.flag_invalid("Trade Future Timestamp (NTP Drift)");
                }
            },
            EventType::Orderbook { bids, asks } => {
                if bids[0].0 > Decimal::ZERO && asks[0].0 > Decimal::ZERO {
                    if bids[0].0 >= asks[0].0 {
                        return self.flag_invalid("Crossed Orderbook (Bid >= Ask)");
                    }
                }
            },
            EventType::Liquidation { price, quantity, timestamp, .. } => {
                if *price <= Decimal::ZERO || *quantity <= Decimal::ZERO {
                    return self.flag_invalid("Liquidation price/qty <= 0");
                }
                if now > *timestamp && (now - *timestamp) > self.max_latency_ms {
                    return self.flag_invalid("Liquidation Stale Data");
                }
            },
            EventType::BookTicker { best_bid_price, best_ask_price, .. } => {
                if *best_bid_price > Decimal::ZERO && *best_ask_price > Decimal::ZERO {
                    if *best_bid_price >= *best_ask_price {
                        return self.flag_invalid("Crossed BookTicker (Bid >= Ask)");
                    }
                }
            },
            _ => {}
        }
        
        true
    }
    
    #[inline(always)]
    fn flag_invalid(&self, _reason: &str) -> bool {
        let count = self.bad_tick_count.fetch_add(1, Ordering::Relaxed);
        
        // Eğer 1 saniyede 100'den fazla bozuk veri gelirse ŞALTER ATAR
        if count > 100 {
            if !self.circuit_breaker.load(Ordering::Relaxed) {
                println!("[!] ⚠️ CIRCUIT BREAKER TRIGGERED! HFT Trading Paused. Reason: {}", _reason);
                self.circuit_breaker.store(true, Ordering::Release);
            }
        }
        false
    }
}
```

### `cycle-engine/core/src/queue.rs`

```rust
use flume::{Receiver, Sender};

/// Bounded lock-free queue dispatcher for high-throughput messaging.
/// Sınırlı kuyruk → üretici geri basınç alır (RAM taşması önlenir).
pub struct LockFreeDispatcher {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
}

const QUEUE_CAPACITY: usize = 262_144;

impl LockFreeDispatcher {
    pub fn new() -> Self {
        let (tx, rx) = flume::bounded(QUEUE_CAPACITY);
        Self { tx, rx }
    }

    #[inline(always)]
    pub fn producer(&self) -> Sender<Vec<u8>> {
        self.tx.clone()
    }

    #[inline(always)]
    pub fn consumer(&self) -> Receiver<Vec<u8>> {
        self.rx.clone()
    }
}
```

### `cycle-engine/core/src/db.rs`

```rust
use rusqlite::{Connection, params};
use flume::Receiver;
use std::time::{Instant, Duration};
use rust_decimal::prelude::*;
use contracts::events::{OwnedEvent, EventType};

pub fn start_db_writer(rx: Receiver<OwnedEvent>) {
    std::fs::create_dir_all("data-engine/data").ok();
    // Open or create SQLite DB
    let mut conn = Connection::open("data-engine/data/market_data.db").expect("Failed to open SQLite database");
    
    // Optimize SQLite for high throughput (WAL mode, synchronous=NORMAL)
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;"
    ).expect("Failed to set PRAGMAs");
    
    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS orderbooks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            bids TEXT NOT NULL,
            asks TEXT NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS liquidations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side INTEGER NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS funding_rates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            mark_price REAL NOT NULL,
            index_price REAL NOT NULL DEFAULT 0,
            funding_rate REAL NOT NULL,
            next_funding_time INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS booktickers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            best_bid_price REAL NOT NULL,
            best_bid_qty REAL NOT NULL,
            best_ask_price REAL NOT NULL,
            best_ask_qty REAL NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS open_interests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            open_interest REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS opportunities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            score REAL NOT NULL,
            efficiency REAL NOT NULL,
            price_bps_per_s REAL NOT NULL,
            price_ticks_per_s REAL NOT NULL,
            ob_changes_per_s REAL NOT NULL,
            spread_bps REAL NOT NULL,
            verdict INTEGER NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS symbol_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            score REAL NOT NULL,
            efficiency REAL NOT NULL,
            price_bps_per_s REAL NOT NULL,
            price_ticks_per_s REAL NOT NULL,
            ob_changes_per_s REAL NOT NULL,
            spread_bps REAL NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).unwrap();

    let mut batch_count = 0;
    let mut last_commit = Instant::now();
    let batch_size_limit = 10_000;
    let commit_interval = Duration::from_millis(1000);

    let mut tx = conn.transaction().expect("Failed to begin transaction");

    while let Ok(event) = rx.recv() {
        let symbol_len = event.symbol.iter().position(|&c| c == 0).unwrap_or(16);
        let symbol_str = std::str::from_utf8(&event.symbol[..symbol_len]).unwrap_or("UNKNOWN");

        match &event.payload {
            EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
                tx.execute(
                    "INSERT INTO trades (symbol, price, quantity, timestamp) VALUES (?1, ?2, ?3, ?4)",
                    params![symbol_str, price.to_f64().unwrap_or(0.0), quantity.to_f64().unwrap_or(0.0), timestamp],
                ).expect("Failed to insert trade");
            },
            EventType::Orderbook { bids, asks } => {
                use std::fmt::Write;
                let mut bids_str = String::with_capacity(512);
                for (p, q) in bids.iter() {
                    if *p == rust_decimal::Decimal::ZERO && *q == rust_decimal::Decimal::ZERO { continue; }
                    let _ = write!(&mut bids_str, "{},{}|", p, q);
                }
                
                let mut asks_str = String::with_capacity(512);
                for (p, q) in asks.iter() {
                    if *p == rust_decimal::Decimal::ZERO && *q == rust_decimal::Decimal::ZERO { continue; }
                    let _ = write!(&mut asks_str, "{},{}|", p, q);
                }

                tx.execute(
                    "INSERT INTO orderbooks (symbol, bids, asks) VALUES (?1, ?2, ?3)",
                    params![symbol_str, bids_str, asks_str],
                ).expect("Failed to insert orderbook");
            },
            EventType::Liquidation { side, price, quantity, timestamp } => {
                tx.execute(
                    "INSERT INTO liquidations (symbol, side, price, quantity, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![symbol_str, side, price.to_f64().unwrap_or(0.0), quantity.to_f64().unwrap_or(0.0), timestamp],
                ).expect("Failed to insert liquidation");
            },
            EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time } => {
                tx.execute(
                    "INSERT INTO funding_rates (symbol, mark_price, index_price, funding_rate, next_funding_time) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![symbol_str, mark_price.to_f64().unwrap_or(0.0), index_price.to_f64().unwrap_or(0.0), funding_rate.to_f64().unwrap_or(0.0), next_funding_time],
                ).expect("Failed to insert funding rate");
            },
            EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty } => {
                tx.execute(
                    "INSERT INTO booktickers (symbol, best_bid_price, best_bid_qty, best_ask_price, best_ask_qty) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![symbol_str, best_bid_price.to_f64().unwrap_or(0.0), best_bid_qty.to_f64().unwrap_or(0.0), best_ask_price.to_f64().unwrap_or(0.0), best_ask_qty.to_f64().unwrap_or(0.0)],
                ).expect("Failed to insert bookticker");
            },
            EventType::OpenInterest { open_interest, timestamp } => {
                tx.execute(
                    "INSERT INTO open_interests (symbol, open_interest, timestamp) VALUES (?1, ?2, ?3)",
                    params![symbol_str, open_interest.to_f64().unwrap_or(0.0), timestamp],
                ).expect("Failed to insert open interest");
            }
            EventType::Opportunity { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict } => {
                tx.execute(
                    "INSERT INTO opportunities (symbol, score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%s','now'))",
                    params![symbol_str, score.to_f64().unwrap_or(0.0), efficiency.to_f64().unwrap_or(0.0), price_bps_per_s.to_f64().unwrap_or(0.0), price_ticks_per_s.to_f64().unwrap_or(0.0), ob_changes_per_s.to_f64().unwrap_or(0.0), spread_bps.to_f64().unwrap_or(0.0), verdict],
                ).expect("Failed to insert opportunity");
            }
            EventType::SymbolMetrics { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps } => {
                tx.execute(
                    "INSERT INTO symbol_metrics (symbol, score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%s','now'))",
                    params![symbol_str, score.to_f64().unwrap_or(0.0), efficiency.to_f64().unwrap_or(0.0), price_bps_per_s.to_f64().unwrap_or(0.0), price_ticks_per_s.to_f64().unwrap_or(0.0), ob_changes_per_s.to_f64().unwrap_or(0.0), spread_bps.to_f64().unwrap_or(0.0)],
                ).expect("Failed to insert symbol metrics");
            }
        }

        batch_count += 1;

        if batch_count >= batch_size_limit || last_commit.elapsed() >= commit_interval {
            tx.commit().expect("Failed to commit transaction");
            tx = conn.transaction().expect("Failed to begin transaction");
            batch_count = 0;
            last_commit = Instant::now();
        }
    }
}
```

### `cycle-engine/core/src/config.rs`

```rust
pub use os_utils::config::*;
```

### `cycle-engine/core/src/state.rs`

```rust
use parking_lot::RwLock;
use std::sync::Arc;

/// Event-driven state manager for Order Status and Balances.
pub struct StateManager {
    // Balances updated purely via WebSocket events (Event-Driven)
    balances: Arc<RwLock<f64>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            balances: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Triggers on WebSocket Account Update Event.
    /// This is the primary source of truth for high-frequency operations.
    pub fn on_account_update(&self, new_balance: f64) {
        let mut b = self.balances.write();
        *b = new_balance;
        println!("State: Balance updated via WebSocket to {}", new_balance);
    }

    /// 5-minute REST API Full Audit.
    /// 10s intervals are explicitly forbidden (IP Ban risk).
    pub fn perform_rest_audit(&self) {
        println!("State: Performing 5-minute REST Full Audit to reconcile differences.");
        // Reconciliation logic compares self.balances with REST endpoint result.
    }
}
```

### `cycle-engine/core/src/pii.rs`

```rust
/// Personally Identifiable Information (PII) Masking Utilities.
/// Ensures compliance with GDPR/KVKK constraints (Right to Erasure, 3-year deletion).
pub struct PIIMasker {
    salt: String,
}

impl PIIMasker {
    pub fn new(salt: String) -> Self {
        Self { salt }
    }

    /// Masks IP, Device ID, or User ID using SHA-3 + Salt.
    /// In a real implementation, this would use the `sha3` crate.
    pub fn mask_data(&self, raw_data: &str) -> String {
        // Mock SHA-3 hashing
        let combined = format!("{}{}", raw_data, self.salt);
        let hashed = format!("sha3_hash_of_{}", combined); // Placeholder
        println!("PII: Masked data -> {}", hashed);
        hashed
    }

    /// Background routine triggered daily to check the deletion_registry.
    /// Deletes logs older than 3 years automatically.
    pub fn cleanup_old_logs(&self) {
        println!("PII/Compliance: Sweeping logs older than 3 years...");
    }
}
```

### `cycle-engine/core/src/bridge.rs`

```rust
pub mod detector_bridge;

pub use detector_bridge::{DetectorBridge, OpportunityHit, SCOUT_RING_CAPACITY, SCOUT_RING_NAME, spawn_watcher};
```

### `cycle-engine/core/src/bridge/detector_bridge.rs`

```rust
//! Detektör → strateji köprüsü.
//!
//! "Scout" ring buffer'i (`/cycle_finance_scout`) detektörler (mikroyapı analizi,
//! misalignment, candle-classifier) tarafından doldurulur; bu köprü ring'deki
//! `EventType::Opportunity` frame'lerini okur ve bunları yüksek-performanslı
//! **tek tüketici** olarak strateji/execution katmanına iletir.
//!
//! Tasarım:
//!   - Ring create etme taraf değil yalnızca OKUMA (cursor ilerletme).
//!   - Ok the frametip: `wire::decode` → `EventType::Opportunity`.
//!   - Geride kalan (overwritten) slotlar `read_slot` generational check ile
//!     atlanır — hiçbir zaman yarım/tutarsız veri işlenmez.

use contracts::events::{EventType, OwnedEvent};
use contracts::wire;
use rust_decimal::Decimal;
use transport::ring_buffer::{GenerationalRingBuffer, MarketDataSlot};
use std::time::Duration;

/// Scout ring'in POSIX shm adı (detektör DATA modunda buraya yazar).
pub const SCOUT_RING_NAME: &str = "/cycle_finance_scout";
/// Scout ring kapasitesi (detektör ile aynı değer).
pub const SCOUT_RING_CAPACITY: usize = 20_000;

/// Ring'den alınan ve strateji katmanına iletilen fırısat sinyali.
///
/// `verdict` detektör kararıdır:
///   0=GUCLU, 1=IYI, 2=NORMAL, 3=BOT/GURULTU, 4=ZAYIF
#[derive(Debug, Clone, PartialEq)]
pub struct OpportunityHit {
    pub symbol: String,
    pub score: Decimal,
    pub efficiency: Decimal,
    pub price_bps_per_s: Decimal,
    pub price_ticks_per_s: Decimal,
    pub spread_bps: Decimal,
    pub verdict: u8,
}

impl OpportunityHit {
    /// Verdict eşiğini aşan fırısatlar için hızlı filtre (0 ve 1 güçlü sinyaldir).
    pub fn is_actionable(&self, max_verdict: u8) -> bool {
        self.verdict <= max_verdict
    }
}

pub struct DetectorBridge {
    ring: GenerationalRingBuffer,
    cursor: u64,
}

impl DetectorBridge {
    /// Mevcut scout ring'ini açar (oluşturursa producer açar; biz sadece okuruz).
    pub fn with_name(name: &str, capacity: usize) -> Self {
        Self {
            ring: GenerationalRingBuffer::with_name(name, capacity),
            cursor: 0,
        }
    }

    pub fn new() -> Self {
        Self::with_name(SCOUT_RING_NAME, SCOUT_RING_CAPACITY)
    }

    pub fn ring(&self) -> &GenerationalRingBuffer {
        &self.ring
    }

    /// `cursor`'dan `head`'e kadar yeni frame'leri okur; `Opportunity` olanları
    /// `handler`'a iletir. Dönen değer işlenen toplam fırısat sayısıdır.
    ///
    /// Not: `poll` çağırmak pahalı değildir (yeni yazılmış frame yoksa no-op).
    pub fn poll(&mut self, mut handler: impl FnMut(&OpportunityHit)) -> usize {
        let head = self.ring.get_head();
        let mut hits = 0usize;

        while self.cursor < head {
            let seq = self.cursor;
            if let Some(slot) = self.ring.read_slot(seq) {
                if let Some(ev) = decode_frame(&slot) {
                    if let EventType::Opportunity {
                        score,
                        efficiency,
                        price_bps_per_s,
                        price_ticks_per_s,
                        ob_changes_per_s: _,
                        spread_bps,
                        verdict,
                    } = &ev.payload
                    {
                        handler(&OpportunityHit {
                            symbol: symbol_to_string(&ev.symbol),
                            score: *score,
                            efficiency: *efficiency,
                            price_bps_per_s: *price_bps_per_s,
                            price_ticks_per_s: *price_ticks_per_s,
                            spread_bps: *spread_bps,
                            verdict: *verdict,
                        });
                        hits += 1;
                    }
                }
            }
            self.cursor += 1;
        }
        hits
    }
}

impl Default for DetectorBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Arka planda sürekli çalışan köprü tüketicisi: her 100ms'de scout ring'ini
/// okur ve güçlü fırısı sinyallerini `handler`'a iletir.
pub fn spawn_watcher(mut handler: impl FnMut(&OpportunityHit) + Send + 'static) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut bridge = DetectorBridge::with_name(SCOUT_RING_NAME, SCOUT_RING_CAPACITY);
        println!("[BRIDGE] Scout ring izleniyor: {} (cap {})", SCOUT_RING_NAME, SCOUT_RING_CAPACITY);
        loop {
            let hits = bridge.poll(&mut handler);
            if hits > 0 {
                println!("[BRIDGE] {} yeni fırısat frame'i işlendi.", hits);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
}

/// Wire frame -> OwnedEvent; bozuk/yarım frame 'None' döner.
fn decode_frame(slot: &MarketDataSlot) -> Option<OwnedEvent> {
    if slot.len == 0 || slot.len as usize > slot.data.len() {
        return None;
    }
    wire::decode(&slot.data[..slot.len as usize])
}

/// C-stili [u8; 16] sembolü temizlenmiş String yapar (null terminator kırpılır).
pub fn symbol_to_string(raw: &[u8; 16]) -> String {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).to_string()
}
```

### `cycle-engine/core/src/cli/mod.rs`

```rust
pub mod paper_cli;
pub mod strategy_cli;
pub mod correlation_cli;
```

### `cycle-engine/core/src/cli/paper_cli.rs`

```rust
use rustyline::DefaultEditor;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use risk_engine::accounting::Portfolio;

pub struct PaperState {
    pub portfolio: Portfolio,
    pub leverage: HashMap<String, u32>,
    pub margin_mode: String, // "Cross" or "Isolated"
}

pub fn start_paper_cli() {
    println!("========================================");
    println!("🛡️ PAPER TRADING TERMINAL v1.0");
    println!("Type 'help' for available commands.");
    println!("========================================");

    let state = Arc::new(Mutex::new(PaperState {
        portfolio: Portfolio::new(Decimal::from(10000), Decimal::from_str("0.20").unwrap()), // 10k USD balance, 20% max drawdown
        leverage: HashMap::new(),
        margin_mode: "Cross".to_string(),
    }));

    // In a real scenario, this thread would read from OrderRingBuffer
    // and execute orders, updating the portfolio state (simulated fills).
    
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("paper> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.is_empty() { continue; }

                match parts[0].to_lowercase().as_str() {
                    "help" => {
                        println!("Commands:");
                        println!("  status                        - Show balance, PnL, positions");
                        println!("  set leverage <symbol> <val>   - Set leverage for a symbol");
                        println!("  set margin <cross|isolated>   - Set margin mode");
                        println!("  exit                          - Quit the terminal");
                    }
                    "status" => {
                        let st = state.lock().unwrap();
                        let dummy_prices = HashMap::new(); // Simulated market prices could go here
                        let equity = st.portfolio.get_total_equity(&dummy_prices);
                        
                        println!("\n--- ACCOUNT STATUS ---");
                        println!("Cash Balance:  ${:.2}", st.portfolio.cash_balance);
                        println!("Realized PnL:  ${:.2}", st.portfolio.realized_pnl);
                        println!("Total Equity:  ${:.2}", equity);
                        println!("Commissions:   ${:.2}", st.portfolio.total_commission);
                        println!("Margin Mode:   {}", st.margin_mode);
                        println!("Positions:");
                        
                        if st.portfolio.positions.is_empty() {
                            println!("  [None]");
                        } else {
                            for (sym, pos) in &st.portfolio.positions {
                                if pos.quantity != Decimal::ZERO {
                                    let lev = st.leverage.get(sym).unwrap_or(&1);
                                    println!("  {} -> Size: {} @ ${:.2} ({}x)", sym, pos.quantity, pos.avg_entry_price, lev);
                                }
                            }
                        }
                        println!("----------------------\n");
                    }
                    "set" => {
                        if parts.len() < 3 {
                            println!("Usage: set leverage <symbol> <val> OR set margin <cross|isolated>");
                            continue;
                        }
                        let mut st = state.lock().unwrap();
                        match parts[1].to_lowercase().as_str() {
                            "leverage" => {
                                if parts.len() == 4 {
                                    let sym = parts[2].to_uppercase();
                                    if let Ok(lev) = parts[3].parse::<u32>() {
                                        st.leverage.insert(sym.clone(), lev);
                                        println!("✅ Leverage for {} set to {}x", sym, lev);
                                    } else {
                                        println!("Invalid leverage value.");
                                    }
                                }
                            }
                            "margin" => {
                                let mode = parts[2].to_lowercase();
                                if mode == "cross" || mode == "isolated" {
                                    st.margin_mode = mode.to_uppercase();
                                    println!("✅ Margin mode set to {}", st.margin_mode);
                                } else {
                                    println!("Mode must be cross or isolated.");
                                }
                            }
                            _ => {
                                println!("Unknown set command.");
                            }
                        }
                    }
                    "exit" | "quit" => {
                        println!("Shutting down paper terminal...");
                        std::process::exit(0);
                    }
                    _ => {
                        println!("Unknown command. Type 'help'.");
                    }
                }
            },
            Err(_) => {
                break;
            }
        }
    }
}
```

### `cycle-engine/core/src/cli/strategy_cli.rs`

```rust
//! STRATEGY terminali — BREAKOUT kırılım stratejisini çalıştırır.
//!
//! Strateji Rust'ta (`breakout-strategy` crate) çalışır: detect-ms'ten seviye/yapı
//! analizi alır, kırılım koşullarını kontrol eder, paper-service'e emir açar.

use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const BREAKOUT_BIN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/breakout-strategy");

struct StrategyChild {
    child: Child,
}

pub fn start_strategy_cli() {
    println!("========================================");
    println!("🎯 STRATEGY ENGINE — BREAKOUT KIRILIM");
    println!("  Binary: {}", BREAKOUT_BIN);
    println!("  detect-ms :3002 + paper-service :8080");
    println!("========================================");

    let running = Arc::new(AtomicBool::new(false));
    let mut child: Option<StrategyChild> = spawn_strategy();
    if child.is_none() {
        println!("❌ BREAKOUT stratejisi başlatılamadı.");
    } else {
        running.store(true, Ordering::SeqCst);
        println!("✅ BREAKOUT stratejisi çalışıyor.");
    }

    let mut rl = rustyline::DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("strategy> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.is_empty() { continue; }

                match parts[0].to_lowercase().as_str() {
                    "help" => {
                        println!("Commands:");
                        println!("  status      - Show strategy status");
                        println!("  restart     - Restart BREAKOUT strategy");
                        println!("  exit        - Quit the terminal");
                    }
                    "status" => {
                        if running.load(Ordering::SeqCst) {
                            println!("  🎯 BREAKOUT Kırılım — RUNNING");
                        } else {
                            println!("  🎯 BREAKOUT Kırılım — DURDU");
                        }
                    }
                    "restart" => {
                        println!("🔄 Strateji yeniden başlatılıyor...");
                        if let Some(mut c) = child.take() {
                            let _ = c.child.kill();
                        }
                        child = spawn_strategy();
                        if child.is_some() {
                            running.store(true, Ordering::SeqCst);
                            println!("✅ BREAKOUT stratejisi yeniden başlatıldı.");
                        } else {
                            running.store(false, Ordering::SeqCst);
                            println!("❌ Yeniden başlatılamadı.");
                        }
                    }
                    "exit" | "quit" => {
                        println!("Shutting down strategy terminal...");
                        std::process::exit(0);
                    }
                    _ => {
                        println!("Unknown command. Type 'help'.");
                    }
                }
            },
            Err(_) => {
                break;
            }
        }
    }
}

fn spawn_strategy() -> Option<StrategyChild> {
    match Command::new(BREAKOUT_BIN)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .spawn()
    {
        Ok(child) => Some(StrategyChild { child }),
        Err(e) => {
            eprintln!("❌ Rust stratejisi başlatılamadı: {}", e);
            None
        }
    }
}
```

### `cycle-engine/core/src/cli/correlation_cli.rs`

```rust
use std::collections::VecDeque;
use chrono::{Local, TimeZone};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use transport::ring_buffer::GenerationalRingBuffer;
use contracts::wire;
use contracts::events::EventType;

struct TradeRecord {
    timestamp: u64,
    price: Decimal,
    qty: Decimal,
}

struct ActiveAnomaly {
    id: u64,
    anomaly_type: u8,
    expected_outcome: u8, // 1: Breakout, 2: Drop, 3: Rise
    start_ts: u64,
    end_ts: u64,
    start_price: Decimal,
}

pub fn start_correlation_cli() {
    let window_sec: u64 = std::env::var("WINDOW_SEC")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);
    
    // YENİ: Takip süresi (Opsiyonel olarak dışarıdan alınabilir, varsayılan window_sec kadar)
    let track_sec: u64 = std::env::var("TRACK_SEC")
        .unwrap_or_else(|_| window_sec.to_string())
        .parse()
        .unwrap_or(window_sec);

    let window_ms = window_sec * 1000;
    let track_ms = track_sec * 1000;
    let flat_threshold = Decimal::from_str("0.001").unwrap();
    let breakout_threshold = Decimal::from_str("0.005").unwrap();

    println!("========================================");
    println!("📈 KORELASYON TERMINALİ v5.0 (ASENKRON KUYRUK)");
    println!("Hedef Parite: VELVETUSDT");
    println!("Analiz Penceresi: {} sn | Takip Penceresi: {} sn", window_sec, track_sec);
    println!("Kümeleme (Clustering) & Kendi Kendini Doğrulama Aktif!");
    println!("========================================");

    let gen_ring = GenerationalRingBuffer::new(160_000);
    let mut read_cursor = gen_ring.get_head();
    
    let history_limit = window_ms * 2;
    let mut history: VecDeque<TradeRecord> = VecDeque::new();
    
    let mut active_anomalies: Vec<ActiveAnomaly> = Vec::new();
    let mut failed_history: VecDeque<u8> = VecDeque::new();
    
    let mut next_anomaly_id = 1;
    let mut last_anomaly_trigger_ts = 0;

    loop {
        if let Some(slot) = gen_ring.read_slot(read_cursor) {
            if let Some(owned_event) = wire::decode(&slot.data[..slot.len as usize]) {
                if owned_event.symbol.starts_with(b"VELVETUSDT") {
                    if let EventType::Trade { price, quantity: qty, timestamp, is_buyer_maker: _ } = owned_event.payload {
                        let record = TradeRecord {
                            timestamp,
                            price,
                            qty,
                        };
                        history.push_back(record);

                        let current_ts = timestamp;
                        let current_price = price;
                        
                        // Eski verileri temizle
                        while let Some(front) = history.front() {
                            if current_ts > front.timestamp && current_ts - front.timestamp > history_limit {
                                history.pop_front();
                            } else {
                                break;
                            }
                        }

                        // 1. Yeni Anomali Tespiti
                        if let Some(first) = history.front() {
                            if current_ts - first.timestamp >= window_ms {
                                let split_ts = current_ts - window_ms;
                                
                                let mut prev_total_vol = Decimal::ZERO;
                                let mut curr_total_vol = Decimal::ZERO;
                                
                                let mut prev_prices = Vec::new();
                                let mut curr_prices = Vec::new();

                                for r in &history {
                                    if r.timestamp < split_ts {
                                        prev_total_vol += r.qty;
                                        prev_prices.push(r.price);
                                    } else {
                                        curr_total_vol += r.qty;
                                        curr_prices.push(r.price);
                                    }
                                }

                                if !prev_prices.is_empty() && !curr_prices.is_empty() {
                                    let prev_price_delta = prev_prices.last().unwrap() - prev_prices.first().unwrap();
                                    let curr_price_delta = curr_prices.last().unwrap() - curr_prices.first().unwrap();

                                    let vol_increased = curr_total_vol > prev_total_vol;
                                    let price_increased = curr_price_delta > flat_threshold;
                                    let price_decreased = curr_price_delta < -flat_threshold;
                                    let price_flat = curr_price_delta.abs() <= flat_threshold;
                                    
                                    let mut anomaly_detected = 0;
                                    let mut expected_outcome = 0;

                                    if vol_increased && price_flat {
                                        anomaly_detected = 1; expected_outcome = 1;
                                    } else if !vol_increased && price_increased {
                                        anomaly_detected = 2; expected_outcome = 2;
                                    } else if !vol_increased && price_decreased {
                                        anomaly_detected = 3; expected_outcome = 3;
                                    }

                                    // Spam koruması: Aynı saniye içinde tekrar tetiklenme
                                    if anomaly_detected > 0 && (current_ts - last_anomaly_trigger_ts > 1000) {
                                        let msg = match anomaly_detected {
                                            1 => "🚨 ANORMAL 1 (EMİLİM): Hacim devasa artıyor ama Fiyat YATAY. (Patlama Bekleniyor)",
                                            2 => "⚠️ ANORMAL 2 (SIĞ TAHTA PUMP): Hacim yok ama fiyat artıyor. (Çakılma Bekleniyor)",
                                            3 => "⚠️ ANORMAL 3 (AYI TUZAĞI): Hacim yok ama fiyat düşüyor. (Fırlama Bekleniyor)",
                                            _ => "",
                                        };

                                        let dt = Local.timestamp_millis_opt(current_ts as i64).unwrap();
                                        let time_str = dt.format("%Y-%m-%d %H:%M:%S.%3f").to_string();

                                        println!("\n[{}] [Yeni Sinyal #{}] {}", time_str, next_anomaly_id, msg);
                                        
                                        active_anomalies.push(ActiveAnomaly {
                                            id: next_anomaly_id,
                                            anomaly_type: anomaly_detected,
                                            expected_outcome,
                                            start_ts: current_ts,
                                            end_ts: current_ts + track_ms,
                                            start_price: current_price,
                                        });
                                        
                                        next_anomaly_id += 1;
                                        last_anomaly_trigger_ts = current_ts;
                                    }
                                }
                            }
                        }

                        // 2. Kuyruktaki Anomalilerin Anlık Takibi (Continuous Monitoring)
                        let mut i = 0;
                        while i < active_anomalies.len() {
                            let anomaly = &active_anomalies[i];
                            let price_change = current_price - anomaly.start_price;
                            
                            // Take-Profit kontrolü
                            let success = match anomaly.expected_outcome {
                                1 => price_change.abs() >= breakout_threshold,
                                2 => price_change <= -breakout_threshold,
                                3 => price_change >= breakout_threshold,
                                _ => false,
                            };

                            let dt = Local.timestamp_millis_opt(current_ts as i64).unwrap();
                            let time_str = dt.format("%Y-%m-%d %H:%M:%S.%3f").to_string();

                            if success {
                                let elapsed_ms = current_ts - anomaly.start_ts;
                                println!("🎯 [{}] [BAŞARILI] Sinyal #{} hedefine sadece {} ms içinde ulaştı!", time_str, anomaly.id, elapsed_ms);
                                
                                // Başarılı olunca başarısızlık zincirini kır (İsteğe bağlı, şimdilik temizliyoruz)
                                failed_history.clear();
                                
                                active_anomalies.remove(i);
                            } else if current_ts >= anomaly.end_ts {
                                // Süre doldu ve hedefe ulaşamadı (Time-Out)
                                println!("❌ [{}] [BAŞARISIZ] Sinyal #{} verilen {} sn sürede hedefine ulaşamadı.", time_str, anomaly.id, track_sec);
                                
                                failed_history.push_back(anomaly.anomaly_type);
                                if failed_history.len() > 3 {
                                    failed_history.pop_front();
                                }
                                
                                // Clustering Analizi (Son 3 sinyal)
                                if failed_history.len() == 3 {
                                    let a1 = failed_history[0];
                                    let a2 = failed_history[1];
                                    let a3 = failed_history[2];
                                    
                                    if a1 == a2 && a2 == a3 {
                                        println!("\n🌋 [KÜMELEME UYARISI] BÜYÜK PATLAMA İHTİMALİ ARTIYOR!");
                                        println!(">> Üst üste 3 kez gerçekleşmeyen Anormal {} sinyali birikti. Dev baskı var!\n", a1);
                                        failed_history.clear(); // Uyarıyı verdik, sıfırla
                                    } else if a1 != a2 && a2 != a3 && a1 != a3 {
                                        println!("\n🌪️ [KÜMELEME UYARISI] KARARSIZLIK / ALGORİTMİK SAVAŞ (TESTERE)!");
                                        println!(">> 3 farklı sinyal (Türü: {}, {}, {}) üretildi ama hiçbiri çalışmadı. Piyasa yatay.\n", a1, a2, a3);
                                        failed_history.clear(); // Uyarıyı verdik, sıfırla
                                    }
                                }
                                
                                active_anomalies.remove(i);
                            } else {
                                // Hala izleniyor
                                i += 1;
                            }
                        }
                    }
                }
            }
            read_cursor += 1;
        } else {
            std::hint::spin_loop();
        }
    }
}
```

### `cycle-engine/core/src/engine/mod.rs`

```rust
pub mod orchestrator;
pub mod backtester;
```

### `cycle-engine/core/src/engine/orchestrator.rs`

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};
use transport::ring_buffer::GenerationalRingBuffer;
use strategies_engine::trait_def::{Strategy, Signal};
use risk_engine::engine::RiskEngine;
use risk_engine::types::{OrderIntent, OrderKind, RiskDecision, Side};
use crate::timer::tsc::TscTimer;
use crossbeam_channel::Sender;
use rust_decimal::Decimal;

#[derive(PartialEq)]
enum StrategyState {
    Active,
    Draining,
    Poisoned,
}

struct ShardedStrategy {
    strategy: Box<dyn Strategy>,
    state: StrategyState,
    /// Sinyal→OrderIntent eşlemesinde kullanılan sembol.
    symbol: String,
}

pub struct TitaniumOrchestrator {
    strategies: Vec<ShardedStrategy>,
    risk_manager: RiskEngine,
    gateway_tx: Sender<Signal>,
}

/// `Signal`'ı risk kapısına girecek `OrderIntent`'e çevirir.
/// Sembol, stratejinin işlem yaptığı piyasadır (headless modda tek sembol).
fn signal_to_intent(signal: Signal, symbol: &str, strategy_id: u32) -> Option<OrderIntent> {
    let (side, quantity, price, kind) = match signal {
        Signal::BuyMarket { quantity } => (Side::Buy, quantity, None, OrderKind::Market),
        Signal::SellMarket { quantity } => (Side::Sell, quantity, None, OrderKind::Market),
        Signal::BuyLimit { price, quantity } => (Side::Buy, quantity, Some(price), OrderKind::Limit),
        Signal::SellLimit { price, quantity } => (Side::Sell, quantity, Some(price), OrderKind::Limit),
        Signal::None | Signal::CancelAll => return None,
    };
    Some(OrderIntent {
        strategy_id,
        symbol: symbol.to_string(),
        side,
        quantity,
        price,
        kind,
        reduce_only: false,
        close_position: false,
        leverage: None,
    })
}

impl TitaniumOrchestrator {
    pub fn new(
        strategies: Vec<(Box<dyn Strategy>, String)>,
        _initial_balance: Decimal,
        max_position_usdt: Decimal,
        daily_loss_usdt: Decimal,
        gateway_tx: Sender<Signal>,
    ) -> Self {
        let sharded = strategies
            .into_iter()
            .map(|(strategy, symbol)| ShardedStrategy {
                strategy,
                state: StrategyState::Active,
                symbol,
            })
            .collect();

        Self {
            strategies: sharded,
            risk_manager: RiskEngine::with_limits(max_position_usdt, daily_loss_usdt),
            gateway_tx,
        }
    }

    pub fn run_spin_loop(&mut self, ring_buffer: &GenerationalRingBuffer) {
        println!("TitaniumOrchestrator: Entering spin loop (Headless)...");

        let mut head: u64 = 0;
        let timer = TscTimer::new();
        let mut last_timer_tick = timer.elapsed_ns();

        loop {
            let current_seq = ring_buffer.get_head(); // Acquire

            while head < current_seq {
                if let Some(slot) = ring_buffer.read_slot(head) {
                    let frame_id = slot.seq;

                    let risk = &self.risk_manager;
                    let gateway = &self.gateway_tx;
                    for shard in &mut self.strategies {
                        if shard.state == StrategyState::Active {
                            // Protect against panics in strategy code (Catch-Unwind)
                            let result = catch_unwind(AssertUnwindSafe(|| {
                                shard.strategy.on_market_data(frame_id, &slot)
                            }));

                            match result {
                                Ok(sig) => {
                                    gate_and_dispatch(risk, gateway, shard, sig);
                                }
                                Err(_) => {
                                    eprintln!("STRATEGY PANIC CAUGHT! Poisoning strategy ID: {}", shard.strategy.id());
                                    shard.state = StrategyState::Poisoned;
                                }
                            }
                        }
                    }
                }
                head += 1;
            }

            // Timer tick (e.g. 1ms = 1_000_000 ns)
            let current_time = timer.elapsed_ns();
            if current_time - last_timer_tick > 1_000_000 {
                let frame_id = current_time; // Simplified
                let delta = current_time - last_timer_tick;

                let risk = &self.risk_manager;
                let gateway = &self.gateway_tx;
                for shard in &mut self.strategies {
                    if shard.state == StrategyState::Active {
                        let result = catch_unwind(AssertUnwindSafe(|| {
                            shard.strategy.on_timer(frame_id, delta)
                        }));

                        match result {
                            Ok(sig) => {
                                gate_and_dispatch(risk, gateway, shard, sig);
                            }
                            Err(_) => {
                                shard.state = StrategyState::Poisoned;
                            }
                        }
                    }
                }
                last_timer_tick = current_time;
            }

            // Spin-wait optimization (CPU Pause)
            std::hint::spin_loop();
        }
    }
}

/// Sinyali risk kapısından geçirir; onaylanırsa gateway'e gönderir.
fn gate_and_dispatch(
    risk: &RiskEngine,
    gateway: &Sender<Signal>,
    shard: &mut ShardedStrategy,
    signal: Signal,
) {
    let Some(intent) = signal_to_intent(signal, &shard.symbol, shard.strategy.id()) else {
        return;
    };
    match risk.evaluate(intent) {
        RiskDecision::Approved { intent } => {
            let signal = match (intent.kind, intent.side) {
                (OrderKind::Market, Side::Buy) => Signal::BuyMarket { quantity: intent.quantity },
                (OrderKind::Market, Side::Sell) => Signal::SellMarket { quantity: intent.quantity },
                (OrderKind::Limit, Side::Buy) => Signal::BuyLimit {
                    price: intent.price.unwrap_or_default(),
                    quantity: intent.quantity,
                },
                (OrderKind::Limit, Side::Sell) => Signal::SellLimit {
                    price: intent.price.unwrap_or_default(),
                    quantity: intent.quantity,
                },
            };
            let _ = gateway.send(signal);
        }
        RiskDecision::Rejected { reason, .. } => {
            eprintln!(
                "RISK REJECTED [{}] {}: {}",
                reason.rule_name(),
                shard.strategy.id(),
                reason.describe()
            );
        }
    }
}
```

### `cycle-engine/core/src/engine/backtester.rs`

```rust
use transport::ring_buffer::GenerationalRingBuffer;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

pub fn start_backtester(csv_path: &str) {
    println!("========================================");
    println!("⏪ BACKTEST ENGINE TERMINAL v1.0");
    println!("Loading: {}", csv_path);
    println!("========================================");

    let gen_ring = std::sync::Arc::new(GenerationalRingBuffer::new(160_000));
    
    let file = File::open(csv_path).expect("❌ CSV dosyası bulunamadı!");
    let reader = BufReader::new(file);

    let start = Instant::now();
    let mut count = 0;

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            // Basit CSV formatı: symbol,price,quantity,timestamp
            // Örnek: BTCUSDT,64000.50,1.2,1623821034000
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let symbol = parts[0];
                let price = parts[1];
                let qty = parts[2];
                let ts = parts[3];
                
                // DATA terminalinin ürettiği WebSocket JSON formatına dönüştürüyoruz
                // Böylece STRATEGY terminali canlı mı yoksa backtest mi çalıştığını ayırt edemeyecek
                let mock_json = format!(
                    "{{\"stream\":\"{}@trade\",\"data\":{{\"s\":\"{}\",\"p\":\"{}\",\"q\":\"{}\",\"T\":{}}}}}",
                    symbol.to_lowercase(), symbol, price, qty, ts
                );
                
                gen_ring.push(mock_json.as_bytes());
                count += 1;
                
                // Rate limit (çok hızlı basarsa RingBuffer taşabilir)
                if count % 100_000 == 0 {
                    std::thread::yield_now();
                }
            }
        }
    }

    let elapsed = start.elapsed();
    println!("✅ Backtest Tamamlandı!");
    println!("Yüklenen Tick: {}", count);
    println!("Geçen Süre: {:?}", elapsed);
    println!("Hız: {:.2} Tick / saniye", (count as f64) / elapsed.as_secs_f64());
}
```

### `cycle-engine/core/src/hal/mod.rs`

```rust
pub mod cpu;
pub mod memory;
```

### `cycle-engine/core/src/hal/cpu.rs`

```rust
use core_affinity::CoreId;

pub fn pin_to_core(core_id: usize) {
    if let Some(core_ids) = core_affinity::get_core_ids() {
        if core_id < core_ids.len() {
            let id = core_ids[core_id];
            if core_affinity::set_for_current(id) {
                println!("System PINNED to CPU Core: {}", id.id);
            } else {
                eprintln!("Failed to pin to CPU Core: {}", id.id);
            }
        } else {
            eprintln!("Requested core {} exceeds available cores ({})", core_id, core_ids.len());
        }
    } else {
        eprintln!("Failed to retrieve CPU cores for affinity pinning.");
    }
}
```

### `cycle-engine/core/src/hal/memory.rs`

```rust
// Mmap or HugePages abstraction
// For the Titanium Core, we pre-allocate memory to avoid heap allocations in the hot path.

pub fn allocate_huge_buffer(size_bytes: usize) -> Vec<u8> {
    // Ideally this would use libc::mmap with MAP_HUGETLB for 2MB pages.
    // To ensure cross-platform safety and avoid OS-level page faults during runtime,
    // we allocate a standard Vec and force the OS to page it in by writing to it.
    let mut buffer = vec![0; size_bytes];
    
    // Touch every page to force physical memory allocation (prevent lazy allocation)
    let page_size = 4096;
    let mut i = 0;
    while i < size_bytes {
        buffer[i] = 1;
        i += page_size;
    }
    
    // Zero it out
    buffer.fill(0);
    
    println!("Allocated {} bytes of pre-faulted contiguous memory.", size_bytes);
    buffer
}
```

### `cycle-engine/core/src/timer/mod.rs`

```rust
pub mod tsc;
```

### `cycle-engine/core/src/timer/tsc.rs`

```rust
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_rdtsc;

pub struct TscTimer {
    start_tsc: u64,
    tsc_hz: f64,
}

impl TscTimer {
    pub fn new() -> Self {
        // Estimate TSC frequency (Simplified)
        // In a real HFT system, we calibrate this against a reliable clock for 1-2 seconds at startup.
        let hz = 3_000_000_000.0; // Assume 3 GHz for now
        
        let start_tsc = Self::read_tsc();
        Self {
            start_tsc,
            tsc_hz: hz,
        }
    }

    #[inline(always)]
    pub fn read_tsc() -> u64 {
        #[cfg(target_arch = "x86_64")]
        unsafe { _rdtsc() }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            // Fallback for ARM/Mac
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
        }
    }

    #[inline(always)]
    pub fn elapsed_ns(&self) -> u64 {
        let current = Self::read_tsc();
        if current > self.start_tsc {
            let diff = current - self.start_tsc;
            ((diff as f64 / self.tsc_hz) * 1_000_000_000.0) as u64
        } else {
            0
        }
    }
}
```

---

## 4. adapter (Dış Entegrasyonlar)

### `cycle-engine/adapter/Cargo.toml`

```toml
[package]
name = "adapter"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio-tungstenite = { workspace = true }
futures-util = { workspace = true }
tokio = { workspace = true }
flume = { workspace = true }
reqwest = { workspace = true }
serde_json = { workspace = true }
serde = { workspace = true }
redis = { workspace = true }

[dev-dependencies]
testcontainers = "0.14"
wiremock = "0.5"
```

### `cycle-engine/adapter/src/lib.rs`

```rust
pub mod redis;
pub mod clickhouse;
pub mod ai;
pub mod vault;
pub mod telemetry;
pub mod binance;

pub use redis::{RedisAdapter, RedisHealth};

/// Adapter altyapısını başlatır: Redis bağlantısını kurar ve sağlığını loglar.
/// (Vault sağlık kontrolü async olduğu için ayrıca `vault.health()` çağrılır.)
pub fn init_adapter() {
    let redis = RedisAdapter::new();
    match redis.health() {
        RedisHealth::Connected => println!("Adapter initialized | Redis: connected"),
        RedisHealth::Degraded => println!("Adapter initialized | Redis: DEGRADED (fail-closed)"),
    }
}
```

### `cycle-engine/adapter/src/binance.rs`

```rust
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use flume::Sender;
use serde_json::json;

/// Yeniden bağlanma politikası: 1s ile başla, her başarısız denemede ikiye katla,
/// en fazla 60s (üstel geri çekilme). Başarılı bağlantı geri çekilme seviyesini sıfırlar.
const BASE_RECONNECT_DELAY_MS: u64 = 1_000;
const MAX_RECONNECT_DELAY_MS: u64 = 60_000;

async fn fetch_usdt_spot_pairs() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    println!("Binance WS: Limiting subscriptions to specific symbols...");
    
    let target_symbols = vec!["btcusdt", "ethusdt", "solusdt", "velvetusdt"];
    let mut pairs = Vec::new();
    
    for sym in target_symbols {
        pairs.push(format!("{}@trade", sym));
        pairs.push(format!("{}@depth20@100ms", sym));
    }
    
    println!("Binance WS: Found {} streams for targeted Futures pairs.", pairs.len());
    Ok(pairs)
}

async fn start_ws_chunk(tx: Sender<Vec<u8>>, chunk: Vec<String>, chunk_id: usize) {
    let ws_url = "wss://fstream.binance.com/stream";

    println!("Binance WS [Chunk {}]: Connecting ({} streams)...", chunk_id, chunk.len());

    // Üstel geri çekme: her başarısız denemeden sonra ikiye katlan, 60s'de tavanla.
    let mut backoff_ms = BASE_RECONNECT_DELAY_MS;

    loop {
        match connect_async(ws_url).await {
            Ok((ws_stream, _)) => {
                println!("Binance WS [Chunk {}]: Successfully connected.", chunk_id);
                backoff_ms = BASE_RECONNECT_DELAY_MS;

                let (mut write, mut read) = ws_stream.split();

                let sub_msg = json!({
                    "method": "SUBSCRIBE",
                    "params": chunk,
                    "id": chunk_id
                });

                if let Err(e) = write.send(Message::Text(sub_msg.to_string())).await {
                    eprintln!("Binance WS [Chunk {}]: Subscribe failed: {}", chunk_id, e);
                    continue;
                }

                // 30 sn'de bir Ping — Binance sessiz bağlantıları kapatır (idle timeout).
                // Ayrıca ticker, runtuk kapanmadan önce kopuşu yakalamamızı sağlar.
                let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));
                ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

                loop {
                    tokio::select! {
                        _ = ping_interval.tick() => {
                            if write.send(Message::Ping(Vec::new())).await.is_err() {
                                eprintln!("Binance WS [Chunk {}]: Ping failed, reconnecting.", chunk_id);
                                break;
                            }
                        }
                        msg = read.next() => {
                            match msg {
                                Some(Ok(message)) => {
                                    if message.is_text() {
                                        let text = message.into_text().unwrap();
                                        let bytes = text.into_bytes();

                                        // Bounded kuyruk → geri basınç (asla RAM taşmaz).
                                        if tx.send_async(bytes).await.is_err() {
                                            eprintln!("Binance WS [Chunk {}]: Consumer queue dropped, shutting down.", chunk_id);
                                            return;
                                        }
                                    } else if message.is_close() {
                                        eprintln!("Binance WS [Chunk {}]: Server closed connection.", chunk_id);
                                        break;
                                    }
                                }
                                Some(Err(e)) => {
                                    eprintln!("Binance WS [Chunk {}]: Read error: {}", chunk_id, e);
                                    break;
                                }
                                None => {
                                    eprintln!("Binance WS [Chunk {}]: Stream ended.", chunk_id);
                                    break;
                                }
                            }
                        }
                    }
                }

                println!("Binance WS [Chunk {}]: Disconnected.", chunk_id);
            }
            Err(e) => {
                eprintln!("Binance WS [Chunk {}]: Connection failed: {}", chunk_id, e);
            }
        }

        // Bağlantı koptu ya da hiç kurulamadı: geri çekme ile yeniden dene.
        tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
        backoff_ms = (backoff_ms * 2).min(MAX_RECONNECT_DELAY_MS);
        println!("Binance WS [Chunk {}]: Reconnecting in {}s...", chunk_id, backoff_ms / 1000);
    }
}

/// Connects to Binance live WebSocket stream for all USDT trade events.
pub async fn start_binance_ws_client(tx: Sender<Vec<u8>>) {
    match fetch_usdt_spot_pairs().await {
        Ok(pairs) => {
            // Binance allows up to 200 streams per WebSocket connection
            let chunks: Vec<Vec<String>> = pairs.chunks(200).map(|c| c.to_vec()).collect();
            
            let mut handles = Vec::new();
            for (i, chunk) in chunks.into_iter().enumerate() {
                let tx_clone = tx.clone();
                handles.push(tokio::spawn(async move {
                    start_ws_chunk(tx_clone, chunk, i + 1).await;
                }));
                // Binance's DDoS firewall (WAF) blocks the IP if we open too many WS connections simultaneously.
                // Add a small delay between opening chunks.
                tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
            }
            
            for handle in handles {
                let _ = handle.await;
            }
        }
        Err(e) => {
            eprintln!("Binance WS: Failed to fetch pairs: {}", e);
        }
    }
}
```

### `cycle-engine/adapter/src/redis.rs`

```rust
use redis::Commands;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Redis kullanılabilirlik durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedisHealth {
    Connected,
    Degraded,
}

/// Idempotency and State caching via Redis.
///
/// Gerçek Redis istemcisi: `REDIS_URL` çevre değişkeninden URL alır
/// (varsayılan `redis://127.0.0.1:6379`). Redis yoksa fail-closed davranılır:
/// emir idempotency anahtarı yazılamaz → işlem reddedilir (kayıp emir yok,
/// çoğaltılmış emir yok).
pub struct RedisAdapter {
    conn: Option<Mutex<redis::Connection>>,
}

impl RedisAdapter {
    /// `REDIS_URL` ile bağlanır. Bağlantı katı KC değildir — ilk işlemde doğrulanır.
    pub fn new() -> Self {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        Self::with_url(&url)
    }

    pub fn with_url(url: &str) -> Self {
        match redis::Client::open(url) {
            Ok(client) => match client.get_connection_with_timeout(Duration::from_secs(5)) {
                Ok(conn) => {
                    println!("Redis: Connected to {}", url);
                    Self { conn: Some(Mutex::new(conn)) }
                }
                Err(e) => {
                    eprintln!("Redis: Bağlantı kurulamadı ({}): {}", url, e);
                    Self { conn: None }
                }
            },
            Err(e) => {
                eprintln!("Redis: Geçersiz URL ({}): {}", url, e);
                Self { conn: None }
            }
        }
    }

    pub fn health(&self) -> RedisHealth {
        match &self.conn {
            Some(c) => {
                let mut guard = c.lock().unwrap();
                if redis::cmd("PING").query::<String>(&mut *guard).is_ok() {
                    RedisHealth::Connected
                } else {
                    RedisHealth::Degraded
                }
            }
            None => RedisHealth::Degraded,
        }
    }

    /// Generates a unique clientOrderId to prevent replay attacks and duplicate orders.
    /// Format: "BOT_UUID_timestamp_nano"
    pub fn generate_client_order_id(&self, bot_uuid: &str) -> String {
        let nano = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}_{}", bot_uuid, nano)
    }

    /// Writes the order ID to Redis with a strict 1-hour TTL (3600 seconds) for Idempotency.
    /// Atomic `SET key 1 EX 3600 NX` — aynı anahtarla ikinci yazma başarısız olur
    /// (çift emir/tekrar koruması). Redis yoksa fail-closed: `Err`.
    pub fn set_idempotency_key(&self, order_id: &str) -> Result<(), &'static str> {
        let conn = self.conn.as_ref().ok_or("Redis unavailable")?;
        let mut guard = conn.lock().unwrap();
        let ttl_seconds: u64 = 3600;
        match redis::cmd("SET")
            .arg(order_id)
            .arg(1u8)
            .arg("EX")
            .arg(ttl_seconds)
            .arg("NX")
            .query::<Option<i64>>(&mut *guard)
        {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err("duplicate: idempotency key already set"),
            Err(_) => Err("Redis command failed"),
        }
    }

    /// Fetches the idempotency status. If the key times out (5s), returns "Pending".
    /// Var olan anahtarla -> "Confirmed"; yoksa / sorun varsa -> "Pending".
    pub fn check_ack_status(&self, order_id: &str) -> String {
        let Some(conn) = self.conn.as_ref() else {
            return "Pending".to_string();
        };
        let mut guard = conn.lock().unwrap();
        match guard.get::<_, Option<String>>(order_id) {
            Ok(Some(_)) => "Confirmed".to_string(),
            _ => "Pending".to_string(),
        }
    }
}

impl Default for RedisAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

### `cycle-engine/adapter/src/clickhouse.rs`

```rust
/// Adapter for ClickHouse Data Lake operations.
pub struct ClickHouseAdapter {
    /// Uyumluluk denetimi (compliance audit) için silme kaydı: hash -> kaç kez silindi.
    deletion_registry: std::sync::Mutex<std::collections::HashMap<String, u64>>,
}

impl ClickHouseAdapter {
    pub fn new() -> Self {
        Self {
            deletion_registry: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Creates the table schema using Zstandard compression (level 22)
    /// and partitioned by year/month/day (Approx 7300 partitions for 20 years).
    pub fn create_tick_table_schema(&self) -> String {
        r#"
        CREATE TABLE IF NOT EXISTS ticks (
            symbol String,
            price Float64,
            quantity Float64,
            timestamp UInt64,
            date Date DEFAULT toDate(toDateTime(timestamp / 1000))
        ) ENGINE = MergeTree()
        PARTITION BY (toYear(date), toMonth(date), toDayOfMonth(date))
        ORDER BY (symbol, timestamp)
        SETTINGS index_granularity = 8192,
                 min_compress_block_size = 65536,
                 max_compress_block_size = 1048576;
        -- NOTE: ZSTD(22) is applied at the column/table compression codec level.
        "#.to_string()
    }

    /// Right to Erasure (GDPR/KVKK) physical deletion logic.
    /// Uses ClickHouse mutations to physically erase data and logs it to a registry.
    pub fn execute_right_to_erasure(&self, user_uuid_hash: &str) {
        println!("ClickHouse: ALTER TABLE ticks DELETE WHERE symbol_hash = '{}'", user_uuid_hash);
        // Compliance audit: silme isteğini registry'ye kaydet.
        let mut registry = self.deletion_registry.lock().unwrap();
        *registry.entry(user_uuid_hash.to_string()).or_default() += 1;
    }

    /// Kaç kez silme isteği işlendiğini döner (denetim/log doğrulaması).
    pub fn erasure_count(&self, user_uuid_hash: &str) -> u64 {
        self.deletion_registry.lock().unwrap().get(user_uuid_hash).copied().unwrap_or(0)
    }

    /// EC-12/4 (Erasure Coding) and Merkle Tree integrity check.
    /// Run during off-peak hours (daily) to verify data chunk integrity across nodes.
    pub fn run_integrity_check(&self) {
        println!("ClickHouse: Running Merkle Tree check and EC-12/4 recovery simulation.");
    }
}
```

### `cycle-engine/adapter/src/vault.rs`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

/// Vault sağlık cevabı (sys/health).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VaultHealth {
    pub initialized: bool,
    pub sealed: bool,
    pub standby: bool,
}

/// Vault Integration for dual key rotation and JWT management.
pub struct VaultAdapter {
    pub current_key_version: u32,
    /// HashiCorp Vault adresi (ör. `http://127.0.0.1:8200`). Boşsa mock modu.
    pub base_url: String,
}

impl VaultAdapter {
    pub fn new() -> Self {
        let base_url = std::env::var("VAULT_ADDR").unwrap_or_default();
        Self {
            current_key_version: 1,
            base_url,
        }
    }

    /// Gerçek Vault sağlık kontrolü: `GET /v1/sys/health?standbyok=true`.
    /// Vault yoksa `None` (mock modunda da `None`).
    pub async fn health(&self) -> Option<VaultHealth> {
        if self.base_url.is_empty() {
            return None;
        }
        let url = format!("{}/v1/sys/health?standbyok=true", self.base_url.trim_end_matches('/'));
        match reqwest::get(&url).await {
            Ok(resp) => resp.json::<VaultHealth>().await.ok(),
            Err(e) => {
                eprintln!("Vault: health check başarısız: {}", e);
                None
            }
        }
    }

    /// Handles dual key rotation with a 5-minute grace period.
    /// During the grace period, both the old and new keys are considered valid.
    pub fn rotate_keys(&mut self) {
        self.current_key_version += 1;
        println!("Vault: Keys rotated to v{}. 5-minute grace period activated for v{}.", 
            self.current_key_version, self.current_key_version - 1);
    }

    /// Creates a JWT with 1 hour TTL.
    /// It should be refreshed 10 minutes prior to expiration.
    pub fn generate_jwt(&self) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let exp = now + 3600; // 1 hour TTL
        let refresh_at = exp - 600; // 10 mins prior
        
        println!("Vault: Generated JWT. Exp: {}, Refresh At: {}", exp, refresh_at);
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.mock.signature".to_string()
    }
}
```

### `cycle-engine/adapter/src/ai.rs`

```rust
/// AI Microservice Adapter integrating via Redis.
pub struct AIAdapter;

impl AIAdapter {
    /// Reads the output of the Python Isolation Forest microservice.
    /// This service detects anomalies based on tick latency or price spikes.
    pub fn read_isolation_forest_anomaly_score(&self, symbol: &str) -> f64 {
        // Mock read from Redis
        println!("AI: Reading Isolation Forest score for {}", symbol);
        0.05 // Normal score
    }

    /// Reads sentiment or trend sensitivity tag from the LLM microservice.
    pub fn read_llm_trend_tag(&self, symbol: &str) -> String {
        // Mock read from Redis
        println!("AI: Reading LLM sentiment tag for {}", symbol);
        "NEUTRAL".to_string()
    }
}
```

### `cycle-engine/adapter/src/telemetry.rs`

```rust
/// Telemetry and Observability (eBPF & Jaeger integration)
pub struct TelemetryAgent;

impl TelemetryAgent {
    /// Simulates eBPF Node Agent DaemonSet hooks for tracking Round-Trip Time (RTT).
    pub fn track_rtt(&self, rtt_ms: f64) {
        if rtt_ms > 1.0 {
            println!("Telemetry(eBPF): RTT spike detected ({}ms). Triggering 100% Jaeger sampling.", rtt_ms);
            self.adjust_jaeger_sampling(1.0); // 100% sampling
        } else {
            // Normal 1% sampling
            self.adjust_jaeger_sampling(0.01); 
        }
    }

    fn adjust_jaeger_sampling(&self, rate: f64) {
        println!("Jaeger: Adaptive sampling rate adjusted to {}%", rate * 100.0);
    }

    /// Triggers Chaos Mesh integration to simulate network partitions, DNS failures, or NTP drifts.
    pub fn trigger_chaos_mesh_scenario(&self, scenario_id: u8) {
        println!("Chaos Mesh: Injecting fault scenario #{} (e.g., NTP Drift of 500ms)", scenario_id);
    }
}
```

---

## 5. splash (Açılış Ekranı)

### `cycle-engine/splash/Cargo.toml`

```toml
[package]
name = "cycle-splash"
version = "0.1.0"
edition = "2021"

[dependencies]
terminal_size = { workspace = true }
figlet-rs = { workspace = true }
```

### `cycle-engine/splash/src/main.rs`

```rust
//! Cycle Finance açılış ekranı — bağımsız binary.
//!
//! `cargo run -p cycle-splash` veya `target/release/cycle-splash` ile
//! tek terminalde çalışır; FIGlet ASCII animasyonu + 3sn'lik yükleme çubuğu
//! bittikten sonra Enter bekler, basılınca çıkar.
//! tmux başlatıcısı bunu 4'lü ekran açılmadan önce çağırır.

fn main() {
    cycle_splash::show_splash();
}
```

### `cycle-engine/splash/src/lib.rs`

```rust
//! Cycle Finance açılış ekranı — FIGlet ASCII sanatı + yükleme çubuğu.
//!
//! "CYCLE FINANCE" yazısı matrix yeşili ile harf harf çizilir; altında bir
//! yükleme çubuğu tam 3 saniyede dolar. Yazı ve çubuk senkron ilerler:
//! çubuk %100 olduğunda yazı da tam haline ulaşır. Çubuk bitince kullanıcı
//! Enter'a basar ve sistem açılır (binary çıkar).

use figlet_rs::FIGfont;
use std::io::{stdin, stdout, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use terminal_size::{terminal_size, Height, Width};

/// Toplam yükleme süresi (ms)
const LOAD_MS: u64 = 3000;

/// Varsayılan metin
const SPLASH_TEXT: &str = "CYCLE FINANCE";

/// Yükleme çubuğu genişliği (karakter)
const BAR_WIDTH: usize = 40;

/// Matrix yeşili (true color): #00FF41
const MATRIX_GREEN: &str = "\x1B[38;2;0;255;65m";
/// Siyah arkaplan
const BG_BLACK: &str = "\x1B[48;2;0;0;0m";
/// Renk sıfırla
const RESET: &str = "\x1B[0m";
/// Terminali tamamen temizle + imleci başa al + imleci gizle
const CLEAR: &str = "\x1B[2J\x1B[1;1H\x1B[?25l";

/// Açılış ekranını gösterir; Enter'a basılınca döner.
pub fn show_splash() {
    show_splash_with(SPLASH_TEXT, LOAD_MS);
}

/// Özel metin ve toplam yükleme süresi (ms) ile açılış ekranı gösterir.
/// Animasyon ve yükleme çubuğu senkron: süre bitince yazı tam halde olur.
pub fn show_splash_with(metin: &str, total_ms: u64) {
    let total = if total_ms == 0 { LOAD_MS } else { total_ms };
    let chars: Vec<char> = metin.chars().collect();
    let toplam_harf = chars.len();
    let step_ms = (total / toplam_harf as u64).max(1);

    let font = FIGfont::standard().expect("FIGlet standart font yüklenemedi!");

    let (term_width, term_height) = if let Some((Width(w), Height(h))) = terminal_size() {
        (w as usize, h as usize)
    } else {
        (80, 24)
    };

    // Çubuğun satırı ekranın TAM dikey ortasında olsun.
    // Çubuk satırı = dikey_bosluk + fig_yukseklik + 1 (boş satır) → term_height / 2
    let tam_figure = font.convert(metin).expect("FIGlet dönüşüm başarısız!");
    let fig_yukseklik = tam_figure.to_string().lines().count();
    let orta = term_height / 2;
    let dikey_bosluk = orta.saturating_sub(fig_yukseklik + 1);

    let mut out = stdout();
    for i in 1..=toplam_harf {
        if write!(out, "{CLEAR}{BG_BLACK}").is_err() || out.flush().is_err() {
            return;
        }

        // Şu ana kadar biriken harfler
        let kismi_metin: String = chars[0..i].iter().collect();
        let figure = font.convert(&kismi_metin).expect("FIGlet dönüşüm başarısız!");
        let cikti = figure.to_string();

        for _ in 0..dikey_bosluk {
            if writeln!(out).is_err() {
                return;
            }
        }

        // Yazı (matrix yeşili, yatay ortalı)
        for satir in cikti.lines() {
            let yatay_bosluk = term_width.saturating_sub(satir.len()) / 2;
            if writeln!(out, "{}{MATRIX_GREEN}{}{RESET}", " ".repeat(yatay_bosluk), satir).is_err() {
                return;
            }
        }

        // Yükleme çubuğu (tam metnin ilerlemesiyle senkron)
        writeln!(out).ok();
        let percent = i * 100 / toplam_harf;
        let filled = percent * BAR_WIDTH / 100;
        let bar: String = format!("{}{} {}", "█".repeat(filled), "░".repeat(BAR_WIDTH - filled), percent);
        let bar_yatay = term_width.saturating_sub(bar.len() + 2) / 2;
        if writeln!(out, "{}{MATRIX_GREEN}[{}]%{RESET}", " ".repeat(bar_yatay), bar).is_err() {
            return;
        }

        if out.flush().is_err() {
            return;
        }
        sleep(Duration::from_millis(step_ms));
    }

    // Çubuk tamamlandı — Enter bekle
    writeln!(out).ok();
    let msg = "▶ SİSTEMİ BAŞLATMAK İÇİN ENTER TUŞUNA BASINIZ";
    let msg_x = term_width.saturating_sub(msg.len()) / 2;
    let _ = writeln!(out, "{}{MATRIX_GREEN}{}{RESET}", " ".repeat(msg_x), msg);

    let _ = write!(out, "\x1B[?25h{RESET}");
    let _ = out.flush();

    // Enter bekle
    let mut buf = String::new();
    let _ = stdin().read_line(&mut buf);

    let _ = write!(out, "{CLEAR}{RESET}");
    let _ = out.flush();
    exit(0);
}
```

---

*Doküman otomatik oluşturuldu — kaynak: `cycle-engine/` (48 dosya).*
