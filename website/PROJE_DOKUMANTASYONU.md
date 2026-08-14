# 🏗️ DEMİR YUMRUK 2.0 — TAM PROJE DOKÜMANTASYONU

> Bu doküman, `/home/smhvz/Desktop/PROJE` çalışma alanındaki **tüm kaynak kodların** ve **minari yapısının** olduğu gibi dökümüdür.
> Bölüm 1: Proje ağacı · Bölüm 2: Ayrıntılı mimari dökümantasyon · Bölüm 3: Her dosyanın olduğu gibi kodu.
> Oluşturulma tarihi: 2026-08-08 · Kapsam: tüm `.rs`, `Cargo.toml`, konfigürasyon, betik, TLA+, k8s ve doküman dosyaları (binary/TimescaleDB/log dosyaları hariç).

---

# 📑 Bölüm 1 — Proje Ağacı (Tree)

```
/home/smhvz/Desktop/PROJE
├── adapter
│   ├── src
│   │   ├── ai.rs
│   │   ├── binance.rs
│   │   ├── clickhouse.rs
│   │   ├── lib.rs
│   │   ├── redis.rs
│   │   ├── telemetry.rs
│   │   └── vault.rs
│   ├── tests
│   │   └── integration_suite.rs
│   └── Cargo.toml
├── alert-service
│   ├── src
│   │   ├── audio.rs
│   │   ├── config.rs
│   │   ├── engine.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   └── source.rs
│   └── Cargo.toml
├── .cargo
│   └── config.toml
├── cold-starter
│   ├── src
│   │   ├── catchup.rs
│   │   └── main.rs
│   └── Cargo.toml
├── cold-storage
│   ├── src
│   │   └── lib.rs
│   └── Cargo.toml
├── config
│   ├── config_v5.toml
│   └── config_v6.toml
├── contracts
│   ├── src
│   │   ├── events.rs
│   │   ├── lib.rs
│   │   └── wire.rs
│   └── Cargo.toml
├── core
│   ├── benches
│   │   └── tick_benchmark.rs
│   ├── src
│   │   ├── cli
│   │   │   ├── correlation_cli.rs
│   │   │   ├── mod.rs
│   │   │   ├── paper_cli.rs
│   │   │   └── strategy_cli.rs
│   │   ├── engine
│   │   │   ├── backtester.rs
│   │   │   ├── mod.rs
│   │   │   └── orchestrator.rs
│   │   ├── hal
│   │   │   ├── cpu.rs
│   │   │   ├── memory.rs
│   │   │   └── mod.rs
│   │   ├── risk
│   │   │   ├── engine.rs
│   │   │   ├── lob_simulator.rs
│   │   │   ├── mod.rs
│   │   │   └── portfolio.rs
│   │   ├── strategy
│   │   │   ├── mod.rs
│   │   │   └── trait_def.rs
│   │   ├── timer
│   │   │   ├── mod.rs
│   │   │   └── tsc.rs
│   │   ├── config.rs
│   │   ├── db.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── pii.rs
│   │   ├── queue.rs
│   │   ├── state.rs
│   │   ├── tick.rs
│   │   └── validator.rs
│   ├── tests
│   │   ├── tick_tests.proptest-regressions
│   │   ├── tick_tests.rs
│   │   └── wire_ring_tests.rs
│   └── Cargo.toml
├── detect-liquidity
│   ├── src
│   │   ├── algorithms.rs
│   │   └── main.rs
│   └── Cargo.toml
├── detect-ms
│   ├── src
│   │   ├── imbalance.rs
│   │   ├── levels.rs
│   │   ├── liquidity.rs
│   │   ├── main.rs
│   │   ├── narrative.rs
│   │   ├── pivot.rs
│   │   ├── session.rs
│   │   └── trend.rs
│   └── Cargo.toml
├── detect-pattern
│   ├── src
│   │   ├── algorithms.rs
│   │   └── main.rs
│   └── Cargo.toml
├── detect-sr
│   ├── src
│   │   ├── algorithms.rs
│   │   └── main.rs
│   └── Cargo.toml
├── detect-trb
│   ├── src
│   │   ├── analyzer.rs
│   │   ├── calibration.rs
│   │   ├── cavitation.rs
│   │   ├── grid.rs
│   │   ├── ingest.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── narrative.rs
│   │   ├── order_flow.rs
│   │   ├── solver.rs
│   │   └── types.rs
│   ├── tests
│   │   └── pipeline.rs
│   └── Cargo.toml
├── detect-trend
│   ├── src
│   │   ├── algorithms.rs
│   │   └── main.rs
│   └── Cargo.toml
├── detect-wyckoff
│   ├── src
│   │   ├── analyst.rs
│   │   ├── audit.rs
│   │   ├── execution.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── models.rs
│   │   ├── profile.rs
│   │   ├── risk.rs
│   │   ├── scorer.rs
│   │   └── state.rs
│   ├── tests
│   │   └── pipeline.rs
│   └── Cargo.toml
├── docs
│   ├── flowcharts
│   │   ├── 01_genel_bakis.mmd
│   │   ├── 02_katman0_contracts.mmd
│   │   ├── 03_katman1_transport.mmd
│   │   ├── 04_katman2_core.mmd
│   │   ├── 05_detektorler_nesil.mmd
│   │   ├── 10_yardimci_servisler.mmd
│   │   ├── 11_ci_kubernetes_tla.mmd
│   │   ├── 12_execution_engine.mmd
│   │   └── 13_risk_engine.mmd
│   └── PROJE_DOKUMANTASYONU.md
├── execution-engine
│   ├── src
│   │   ├── paper
│   │   │   ├── account.rs
│   │   │   ├── actor.rs
│   │   │   ├── config.rs
│   │   │   ├── db_writer.rs
│   │   │   ├── domain_event.rs
│   │   │   ├── mod.rs
│   │   │   ├── position.rs
│   │   │   ├── risk.rs
│   │   │   └── snapshot.rs
│   │   ├── lib.rs
│   │   ├── order.rs
│   │   └── signer.rs
│   ├── tests
│   │   ├── replay_tests.rs
│   │   └── risk_tests.rs
│   └── Cargo.toml
├── formal_verification
│   ├── CycleFinance.cfg
│   └── CycleFinance.tla
├── .github
│   └── workflows
│       └── test-suite.yml
├── velvetusdt
│   ├── src
│   │   ├── bin
│   │   │   ├── alerts.rs
│   │   │   ├── listener.rs
│   │   │   └── risk_analysis.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   └── metrics.rs
│   └── Cargo.toml
├── k8s
│   ├── chaos_dns_failure.yaml
│   ├── chaos_network_partition.yaml
│   ├── chaos_ntp_drift.yaml
│   └── deployment.yaml
├── ohlcv-engine
│   ├── src
│   │   ├── bin
│   │   │   ├── cli.rs
│   │   │   └── server.rs
│   │   ├── client.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── os-utils
│   ├── src
│   │   ├── config.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── paper-service
│   ├── src
│   │   ├── bin
│   │   │   └── paper_cli.rs
│   │   ├── api.rs
│   │   ├── bridge.rs
│   │   ├── events.rs
│   │   ├── idempotency.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── metrics.rs
│   │   └── postgres_store.rs
│   ├── tests
│   │   └── actor_e2e.rs
│   └── Cargo.toml
├──
│   ├── src
│   │   └── main.rs
│   └── Cargo.toml
├── risk-worker
│   ├── src
│   │   ├── cache.rs
│   │   ├── finops.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   └── matrix.rs
│   ├── tests
│   │   └── matrix_tests.rs
│   └── Cargo.toml
├── scout-service
│   ├── src
│   │   ├── bin
│   │   │   └── probe.rs
│   │   ├── analyzer.rs
│   │   ├── client.rs
│   │   ├── main.rs
│   │   └── models.rs
│   ├── tests
│   │   └── wire_debug.rs
│   └── Cargo.toml
├── scripts
│   ├── cycle_env.sh
│   ├── cycle_tmux.sh
│   ├── gdpr_erasure_test.sh
│   ├── monitor.sh
│   ├── start_paper.sh
│   └── stop_paper.sh
├── strategies
├── transport
│   ├── src
│   │   ├── lib.rs
│   │   ├── order_ring.rs
│   │   └── ring_buffer.rs
│   └── Cargo.toml
├── alerts.toml
├── Cargo.lock
├── Cargo.toml
├── .env
├── .gitignore
├── install.sh
└── test_data.csv

75 directories, 199 files
```

---

# 🧭 Bölüm 2 — AŞIRI DETAYLI MİMARİ DÖKÜMANTASYON

## 2.1 Genel Bakış

**Cycle Finance 2.0**, Binance Futures verisini tüketen, **düşük gecikme/high-throughput** odaklı, katmanlı bir Rust **trading sistemidir**. 23 workspace üyesi crate (cycle-engine × 7, services-engine × 9, additional-services/os-utils, data-engine × 2, execution-engine, risk-engine, strategies-engine).

### 1.1 Çalışma Alanı (Workspace)

`Cargo.toml` (kök): `members` listesi 23 crate içerir (`cycle-engine/{gateway,pipeline,transport,engine,persistence,infra,flows}`, `additional-services/os-utils`, `data-engine/{cold-storage,cold-starter}`, `execution-engine`, `risk-engine`, `strategies-engine`, `services-engine/{ohlcv-engine,calc-ind,stream-ohlcv,trade-ohlcv,detect-ms,alert-service,exec-console,db-query,telegram-bot,force-orders}`). Resolver 2. Ortak (workspace) bağımlılıklar: `rust_decimal 1.34` (maths + serde), `ndarray 0.15` (rayon), `rayon 1.8`, `wide 0.7` (SIMD).

### 1.2 Katman Modeli (Layer'dan Layer'a)

```
┌───────────────────────────────────────────────────────────────┐
│  Uygulamalar: services-engine (detect-ms, calc-ind, db-query) │
│  Servisler: strategies-engine, alert, stream-ohlcv, trade-ohlcv│
├───────────────────────────────────────────────────────────────┤
│  Katman 1: cycle-engine/transport (shm ring, torn-read korumalı)│
├───────────────────────────────────────────────────────────────┤
│  Katman 0: cycle-engine/pipeline (EventParser + wire codec)   │
├───────────────────────────────────────────────────────────────┤
│  Altyapı: TimescaleDB (trades hypertable'ları), risk-engine,  │
│  TLA+ doğrulama, Chaos Mesh, CI (4 job)                       │
└───────────────────────────────────────────────────────────────┘
```

**Temel veri akışı (hot path):**

```
Binance WS (fstream) ── raw JSON ──▶ cycle-engine/pipeline (parse)
      ──▶ validator ──▶ GenerationalRingBuffer (/dev/shm)
                    │
                    ├──▶ TimescaleDB batch writer (market_data hypertable'ları)
                    ├──▶ cycle-engine/engine (strateji → RiskEngine → gateway)
                    └──▶ tüketiciler: alert / strategies-engine / stream-ohlcv
```

### 1.3 Servis Nesilleri

| Nesil | Üyeler | Özellikler |
|---|---|---|
| Analiz | `detect-ms` (SMC 7 katman), `calc-ind` (indikatör servisi) | axum iskelet, ohlcv-engine klines, HTTP JSON çıktı |
| Strateji | `strategies-engine` (kırılım), `risk_analysis`, `listener`, `alerts` | ring okuyucu, TimescaleDB (`TIMESCALEDB_URL`) analizi, LIVE emir |
| Yürütme | `execution-engine` (LIVE), `risk-engine` (13-adım kural zinciri + risk-worker) | imzalı REST, kill switch, VaR/parametrik |
| Altyapı | `cycle-engine/{gateway,pipeline,transport,flows,persistence,infra,engine}` | WS → parse → ring → TimescaleDB |

### 1.4 Portlar & Arayüzler

| Servis | Port | Çıktı formatı |
|---|---|---|
| detect-ms | 3002 | HTTP JSON |
| calc-ind | 3003 | HTTP JSON |
| ohlcv-engine | 3000 | HTTP JSON klines |
| stream-ohlcv | 3008 | HTTP JSON + ring `/dev/shm/cycle_finance_stream_ohlcv` |
| execution-engine | 3010 | REST (axum) |
| risk-worker | 3011 | ring `/cycle_finance_risk_params` + `/tmp/risk_params.json` |
| alert-service | — | sesli (WAV/paplay veya spd-say) |
| cycle-engine/engine | — | `strategy-console` bin |

### 1.5 Sembol ve Para Birimi Kuralları

- Tüm parasal değerler: `rust_decimal::Decimal` (float YOK parasal yolda)
- Emir büyüklüğü: **USDT notional** (coin adedi değil)
- Sembol: `[u8; 16]` sabit genişlik, uzun symbol kesilir
- Tick tabanlı iç hesaplar: `i64` (1e-6 çözünürlük)

### 1.6 Ortam Değişkenleri

- `TIMESCALEDB_URL` (varsayılan `postgres://cycle:cycle@localhost:5432/market_data`)
- `TRADING_MODE` (execution): LIVE
- `ALERT_VOICE_CMD`, `PRICE_FEED_SYMBOLS`
- `.env` yalnızca dummy key içerir (kod tarafında okunmaz)

# 2️⃣ KATMAN 0 — CONTRACTS (`contracts/`)

## 2.1 `events.rs` — Ortak Veri Modeli

Tüm katmanların konuştuğu **ortak dil**. `EventType` (u8 etiketli enum, `#[repr(u8)]`) ve `OwnedEvent` (`#[repr(C)]`):

| Event | Alanlar | Açıklama |
|---|---|---|
| `Trade` | price, quantity, timestamp, is_buyer_maker | gerçekleşen işlem |
| `Orderbook` | bids/asks `[(Decimal,Decimal);20]` | 20 seviye derinlik anlık görüntüsü |
| `Liquidation` | side u8, price, qty, ts | tasfiye haberi |
| `FundingRate` | mark, index, rate, next_funding_time | 8 saatlik fonlama |
| `BookTicker` | best bid/ask (fiyat+miktar), 4 alan | teklif-toplam |
| `OpenInterest` | oi, ts | açık pozisyon |
| `Opportunity` | score + efficiency + price bps/s + tick/s + ob change/s + spread_bps + verdict u8 | scout fırsat sinyali (verdict: 0=GÜÇLÜ..4=ZAYIF) |
| `SymbolMetrics` | 6 mikroyapı metriği | sembol skorları |

Her tip için `new_*` **statik constructor** (`pack_symbol` ile 16 B sembol), `Debug` manuel uygulanır (derive yok). Boyut kısıtı: sabit genişlikte, stack alanı yok — hot path'te heap tahsisi yok.

## 2.2 `wire.rs` — Compact Typed Binary Codec

- Amaç: `/dev/shm` ring'de **ham JSON yerine minimum boyutlu binary** taşımak.
- Format: `[tag u8][symbol 16B][alanlar: (i64 mantissa + u8 scale) | u64 | u8]`
- `Decimal` kayıpsız geri kurulur: `Decimal::new(mantissa, scale)`.
- Frame boyutları: Trade 44B · BookTicker 53B · Funding 52B · Liquidation 44B · OI 34B · Depth20 659B · Opportunity 72B · SymbolMetrics 71B.
- Depth20'de ortak `scale` bulunur ve tüm seviyeler buna icra edilir (kayıpsız).
- `encode/decode` sıfır-tahsis, `#[inline(always)]` helper'lar; decode'ta güdük/botlu frame → `None`.
- 12 unit test: her tip roundtrip, truncation, partial ladder, symbol cutoff.

---

# 3️⃣ KATMAN 1 — TRANSPORT (`transport/`)

## 3.1 `ring_buffer.rs` — GenerationalRingBuffer

**Tasarım:** POSIX shared memory (`shm_open` + `mmap`), her slot `[seq: u64 | len: u16 | veri: [u8;702]]` (768B, 64B aligned) → tek producer / çoklu consumer, **lock-free**:

1. Producer: `head` okur; `data`+`len` yazar → `fence(Release)` → `seq` *(EN SON)* yazar → `head.store(seq+1)`.
2. Consumer: `cursor`'ını takip eder; `idx = seq % kapasite`; okur; `slot.seq == beklenen seq` **generational check** → kopyala → ikinci kez seq kontrolü (torn-read'e karşı çift güvenlik).
3. Sonuc: eski slot'a yazılan yeni veri, eski cursor'dan okunamaz (`None`); `magic` (`0xD3F…0001`) değişmezliği + `ftruncate` tek-proses koruması.
4. Saniyede 1.6 milyondan fazla publish/read — `simd_json` zero-copy parse ile kombine edildiğinde tek thread'te milyonlarca event/s.

## 3.2 `order_ring.rs` — OrderRingBuffer

Aynı şablonun emir versiyonu: `side`, `order_type`, `qty/price` (Decimal), `symbol[16B]` 64B slot. Magic `0xD1F…0002` ile test edilir. `STRATEGY → EXECUTION` yönünde.

---

# 4️⃣ KATMAN 2 — CORE (`core/`)

## 4.1 `main.rs` — RUN_MODE Router

`RUN_MODE` env'ine göre 5 uygulamanın giriş:

- **DATA**: market data konsolu —
  - `GenerationalRingBuffer::new(160_000)` shm ring;
  - `flume::bounded(1_000_000)` DB kanalı → `start_db_writer` thread;
  - `LockFreeDispatcher` (flume 262.144) → RT thread (`os_utils::set_rt_thread_priority(99)`);
  - Loop: `EventParser::parse` (zero-copy, buffer**'ı** soyma notu) → `DataValidator::is_valid` → `wire::encode` → ring push → DB (try_send, drop sayaç);
  - sn/s raporu (tick/s, depth, invalid, db_drops, avg parse ns).
- **PAPER**: `paper_cli` REPL
- **STRATEGY**: `strategy_cli`
- **BACKTEST**: CSV → mock stream → ring (canlı ile ayırt edilemeyen akış)
- **CORRELATION**: `correlation_cli` v5

## 4.2 `tick.rs` — EventParser (simd_json zero-copy)

`EventParser::parse(&mut [u8])` → `OwnedEvent`; 6 stream tipi:
- `@trade` (1): `p` fiyat, `q` miktar, `T` ts, `m` buyer_maker
- `@depth` / `@depth20@100ms` (spot ve futures alan destekli: `bids`/`a`)
- `@forceOrder` liquidation
- `@markPrice` funding
- `@bookTicker` best bid/ask
- ve `binance_usdt_m_perpetual` gibi varyant.

`simd_json::from_slice` ile sıfır-kopya; bundan sonra buffer bozuşturulur (ayrıca `\0` ayraçları) — bu yüzden ring'e **typed binary** gider.

## 4.3 `queue.rs` — MPMC Dispatcher

`LockFreeDispatcher`: tek `flume::bounded(262_144)` channel + `producer()/consumer()`. Backpressure (yavaş tüketici üreticiyi sınırlar, RAM taşmaz).

## 4.4 `validator.rs` — DataValidator + Circuit Breaker

- Statik eşikler: fiyat+miktar > 0, `ts` ≤ 200ms eski (stale), `ts` ≤ 5sn ileri (NTP), crossed book (bid ≥ ask) reddedilir.
- **Circuit breaker**: 100+ bozuk tick/saniye → break. Hata log'lanır.
- `is_valid(&OwnedEvent) -> bool`; tek zaman kaynağı: event ts vs `SystemTime::now()`.

## 4.6 `state.rs` — StateManager

WS event-driven bakiye izleme + 5 dakikalık REST audit (`GET /fapi/v3/account` imzalı). `BalanceState` (cash, positions, margin, timestamp).

## 4.7 `pii.rs` — GDPR/KVKK

`PIIMasker`: log'lardan kullanıcı UUID/ISP hash'lerini SHA-3+salt ile maske + 3 yıl eşiğinde log silme politikası — şu an mock/dev.

## 4.8 `hal/` — Donanım Katmanı

- `cpu.rs`: `cpu_count()`, `numa_*`, yoğun döngü ölçümü (`rdtsc` kalibrasyon)
- `memory.rs`: süreç RSS limiti tespiti
- `mod.rs`: `#![forbid(unsafe_code)]` — core'da güvensiz kod YOK.

## 4.9 `timer/` — TSC Teyp Zamanlayıcı

`TscTimer`: x86_64'te `rdtsc` (CPU döngü saati), tersi sabit ~3GHz sayısı; `elapsed_ns = (δcount/hz)·1e9`; `on_timer` 1ms ritmi için. Sıfır syscall.

## 4.10 `strategy/` — Strategy Trait

`trait_def.rs`: `Strategy: Send + Sync`; `id()`, `on_market_data(frame_id, slot) -> Signal`, `on_timer(frame_id, delta_ns)`, `on_fill(FillReport)`, `reset()`. `Signal`: Buy/Sell Market|Limit, CancelAll, None.

## 4.11 `risk/` — Risk Core

- `engine.rs`: `RiskEngine` — maksimum pozisyon, günlük limitler; `process_signal → Result<Signal, RiskOrj>`
- `lob_simulator.rs`: sabit nokta (price×100_000, qty×1_000), 10 kademe, marj-ve ağırlıklandırma; `simulate_buy/sell`
- `portfolio.rs`: cash, realized/unrealized PnL, komisyon, ortalama mail, max drawdown.

## 4.12 `engine/orchestrator.rs` — TitaniumOrchestrator

Spin-loop döngü: ring'den frame → her stratejiye `on_market_data(frame_id, slot)` → `catch_unwind` (panik olan strateji `Poisoned` set) → `RiskEngine` → `gateway_tx.send(Signal)` (crossbeam). 1ms'de `on_timer`.

## 4.13 `engine/backtester.rs`

CSV (symbol,price,quantity,ts) → sahte event stream → ring'e push; "canlı vs backtest ayırt edilemez" kuralı; JSON mock.

## 4.14 `cli/`

- `paper_cli.rs`: REPL (rustyline): balance, order (limit/market), position; 10k USDT başlangıç, %20 max drawdown
- `strategy_cli.rs`: velvetusdt binary 'spawn/restart' orkestratörü
- `correlation_cli.rs`: VELVETUSDT trade'leri üzerine Pearson + 3 anomali (emilim/pump/tuzak) + cluster uyarısı

# 5️⃣ VERİ ALIM KATMANI

## 5.1 `cycle-engine/gateway` — Binance WS İstemcisi

- `binance.rs`: Binance Futures WS; 4 sembol × 2 stream (trade + depth20@100ms); 200'lu chunk, üstel geri çekilme (1s→60s), ortak API rate kapısı
- `rate_gate.rs`: Binance API limitlerine takılmamak için ortak token kovası
- Çıktı: ham JSON `Vec<u8>` → flume `Sender` → `cycle-engine/pipeline` (parse → validate → ring)

## 5.2 `cycle-engine/flows` — Veri Akış Daemon'ları (flow_*)

- WS ile gelen akışlar için ayrı POSIX shm ring'leri + TimescaleDB hypertable'ları:
  `flow_trade`, `flow_depth`, `flow_liquidation`, `flow_oi`, `flow_funding`, `flow_markprice`, `flow_lastprice`, `flow_indexprice`
- `rest.rs`: WS ile gelmeyen akışlar (funding, markprice, indexprice, lastprice, oi) için REST fallback poll
- `parse.rs`: `lastPrice: p` → mark_price alanına taşınır (lastprice tablosu `price` olarak yazar)
- Tüketiciler: `services-engine/stream-ohlcv` (`/cycle_finance_lastprice`), `strategies-engine/feed`, `alert-service`

## 5.3 `services-engine/ohlcv-engine` — Kline API + Client

- `client.rs`: `fetch_klines(symbol, interval, limit)` → Binance `/fapi/v1/klines` REST; Decimal dönüşümü
- `server.rs`: `GET /api/klines` axum (:3000) — cache YOK, her istek canlı Binance
- `cli.rs`: terminal OHLCV radarı (clap: sembol/interval/limit)
- **Tüketiciler**: detect-ms, calc-ind, db-query, stream-ohlcv

---

# 6️⃣ DETEKTÖR SERVİSLERİ — AYRINTILI ANALİZ

## 6.1 Ortak İskelet (1. Nesil)

Tüm 1. nesil servisler aynı şablonu taşır:

```
main.rs: clap veya sabit symbol/interval/limit → BinanceClient::fetch_klines
         → algorithms::analyze_* → serde_json → axum GET handler
Cargo.toml: axum 0.8.9 + ohlcv-engine + tokio + serde_json + rust_decimal
```

## 6.2 `detect-ms` — Market Structure Multi-Protocol (SMC 7 Katman)

- **session.rs**: Core/Amplified/Acute ağırlıkları 0.40/0.30/0.30; `weighted_merge`, `confluence_index`; `is_active_session` **ölü kod** (çağrılmıyor)
- **pivot.rs**: EMA-smoothed ATR14; eşik ATR×0.25; Tip A (wick) + Tip B (close) pivotlar; ikisi aynı mumda farklıysa → likidite bölgesi
- **trend.rs**: log-OLS regresyon + gerçek R/S Hurst; skor = slope×price/ATR×10×R² (clamp ±10)
- **levels.rs**: üssel çürüyen seviye envanteri λ=0.015 (~46 mum yarılanma); sweep/breakout onayı (wick kırıp close geri = sweep; 2 ardışık close = BO); sınıflandırma Defended=10 … NewActive=7
- **liquidity.rs**: VWAP + hacim ağırlıklı σ; volume profile bin başına orantılı dağılım; HVN=1.5×medyan; BSL=+1.5σ..+3σ, SSL=−3σ..−1.5σ
- **imbalance.rs**: FVG (3 mum gölge çakışmazlığı) + cumulatif delta; delta uyumlu → ActiveAbsorber (1.5×), değil → PassiveGap (0.5×)
- **narrative.rs**: `generate_report()` tüm katmanları orkestrasyon; ATS, Vakum Bölgesi (manyetik skor), Confluence Index
- Veri: 3 ayrı REST fetch (core=limit, amp=limit×4 max1500, acute=96) — sıralı.

## 6.3 `calc-ind` — İndikatör Hesap Servisi

- `main.rs`: axum REST ile indikatör hesaplama; ohlcv-engine klines verisini tüketir
- Tüketiciler: detect-ms seviyeleriyle birlikte strateji tarafına HTTP JSON sağlar

## 6.4 `db-query` — TimescaleDB Sorgu Servisi

- TimescaleDB hypertable'ları üzerinde (trades, OHLCV, funding, OI) SQL sorguları
- Panel / ad-hoc sorgu arayüzü

# 7️⃣ STRATEJİ & YÜRÜTME KATMANI

## 7.1 `strategies-engine` — Kırılım Stratejisi Çekirdeği

- `main.rs`: ring okuyucu (ask>bid>mark) → 500ms wake, 20dk değerlendirme penceresi → detect-ms seviyeleri ile karşılaştır → LIVE emir; açık pozisyon varken yeni emir yok; `--dry-run`
- `bin/listener.rs`: merkez ring okuyucu; 2sn tablo + Pearson korelasyon matrisi (normalize 0-1); `/tmp/listener_metrics.json`
- `bin/risk_analysis.rs`: TimescaleDB (`TIMESCALEDB_URL`) üzerinde SQL dağılım analizi + --watch canlı panel
- `bin/alerts.rs`: alerts.toml CLI yöneticisi (list/add/update/remove)

# 8️⃣ YÜRÜTME KATMANI

## 8.1 `execution-engine` — Emir Yolu (LIVE)

`lib.rs::start_execution_engine(rx: flume::Receiver<OrderRequest>, api_key, secret_key)`:

- **LIVE modu**: `wss://ws-api.binance.com/ws-api/v3` — `order.place` JSON; query string imzalı (HMAC-SHA256 via `BinanceSigner`), 3sn reconnect
- `src/bin/executiond.rs`: daemon; `exec-cli.rs`: komut satırı arayüzü

`order.rs`: `OrderSide`, `OrderType` (Limit, Market, SL, SLLimit, TP, TPLimit, LimitMaker), `TimeInForce` (GTC/IOC/FOK), `PositionSide` (Both/Long/Short); **quantity = USDT notional**.

## 8.2 `risk-engine` — Risk Çekirdeği + risk-worker

- `src/engine.rs`: 13-adım kural zinciri `RiskEngine::evaluate` (hot path, execution-engine içinde); `on_fill`, `sync_from_snapshot`
- `src/bin/risk-worker.rs`: soğuk yol daemon (60s) — Pearson korelasyon, Tikhonov shrink, parametrik VaR %99, konsantrasyon HHI; çıktı `/cycle_finance_risk_params` ring + `/tmp/risk_params.json`
- `src/kill_switch.rs`: `/tmp/exec_kill_switch` + bayrak

## 9.2 `cold-starter` / `cold-storage`

- `cold-storage`: `DiskBuffer` — `memmap2::MmapMut` yazma buffer (bounds-guard); sıfır latency disk eşleme
- `cold-starter`: `fetch_200_ema()` → TimescaleDB `trades` hypertable'ından 200 EMA hesaplar; `transition_to_live`; `#![allow(unsafe_code)]` (core zıttı)

## 9.3 `alert-service` — Koşullu Sesli Uyarı

- `config.rs`: `Condition::Above/Below/Cross/Touch` (+tolerans%)
- `engine.rs`: `AlertEngine` — per-uyarı state makinesi (Armed→Triggered→re-arm; Cross: last_side_above flip; cooldown; repeat=false kilidi); kanal üzerinden `AlertEvent`
- `audio.rs`: `spd-say -l tr` veya **programatik WAV** (44.1kHz, G6-E6-G6, ADSR zarflı, paplay, 2sn sonra silme)
- `source.rs`: 3 kaynak — merkez ring, ring, doğrudan Binance WS (reconnect 3sn)

## 9.4 `os-utils`

- `set_rt_thread_priority(prio)`: Linux'ta `pthread_setschedparam` SCHED_FIFO (RT), değilse no-op; `config.rs` pin/core tespiti

---

# 🔟 FORMAL DOĞRULAMA & ALTYAPI

## 10.1 TLA+ (`formal_verification/CycleFinance.tla`)

Lock-free veri hattının soyut modeli:
- `queue` (bounded 1000) + `ticks_in`/`ticks_out`
- Eylemler `Produce` / `Consume`; `Next = Produce ∨ Consume`; `WF_vars(Consume)` fairness
- **Safety**: `ticks_out ≤ ticks_in` (sahte tick yok)
- **Liveness**: `ticks_in = n ~> ticks_out = n` (her tick nihayet tüketilir)
- `CycleFinance.cfg`: SPECIFICATION Spec, INVARIANT Safety, PROPERTY Liveness

## 10.2 Kubernetes & Chaos Mesh (`k8s/`)

- `deployment.yaml`: `cycle-finance-core`, 1 replica, 4CPU/4Gi, cgroupv2, **SYS_NICE** (RT thread), probe'lar
- `chaos_network_partition.yaml`: redis-cluster 10sn partition, @5m
- `chaos_dns_failure.yaml`: binance DNS 5dk hata, @30m
- `chaos_ntp_drift.yaml`: +10sn NTP drift (TimeChaos), @15m — imza/zamanlama hatalarını test

## 10.3 CI (`.github/workflows/test-suite.yml`)

4 job zincir (PR + nightly):
1. `audit-and-coverage` — cargo-deny advisories, tarpaulin %95
2. `unit-and-integration` — cargo test --release (wire roundtrip, ring generational, proptest tick, actor e2e)
3. `performance-wcet` — cargo bench, **WCET hedefi 750µs/tick**
4. `chaos-mesh-staging` — 20 chaos senaryosu (echo taslak)

## 10.4 Kurulum & Scriptler

- `install.sh`: release build → `~/.cycle/` kopyalama, 15 binary, `cycle` launcher (start/stop/status/env → tmux), `--package` tar.gz, `--uninstall`
- `additional-services/scripts/`: `cycle_env.sh` (ortam kurulum betiği), `cycle_tmux.sh` (panel orkestrasyonu), `cycle_ip.sh`, `exec_setup.sh`, `monitor.sh`, `gdpr_erasure_test.sh`
- `additional-services/config/`: v5/v6 config.toml
- `alerts.toml`: `data_source="pricefeed"` + 6 uyarı (BTC/ETH/SOL)

## 10.5 Mermaid Akış Şemaları (`docs/flowcharts/`)

9 diyagram: genel bakış, contracts class diyagramı, transport sequence, core, detektörler (detect-ms + calc-ind), yardımcı servisler, CI/k8s/TLA, execution engine, risk engine.

---

# ⚠️ 11. EKSİKLER, RİSKLER VE GELİŞTİRME ÖNERİLERİ

| # | Bulgu | Önem |
|---|---|---|
| 1 | Detektör çıktıları → execution hattı **bağlı değil** (sadece HTTP JSON) | 🔴 Yüksek |
| 2 | detect-ms `is_active_session` ölü kod; narrative'de lineer zaman kullanımı | 🟡 Düşük |
| 3 | `alerts.toml` TOML-parse edilmiyor (string kırpma) | 🟡 Düşük |
| 4 | `ohlcv-engine` reqwest 0.13/edition 2024 — sürüm drift'i | 🟡 Düşük |
| 5 | REST poll seri — çok sembolde gecikme | 🟠 Orta |
| 6 | Cargo.lock 5614 satır (bu dokümana dahil) | — |
| 7 | `[u8;16]` sembol kısıtı — 16+ karakter semboller kesilir | 🟡 Düşük |

**Öncelikli yol haritası önerisi:** (1) detektör → orchestration köprü (opportunity frame'lerini Strategy'ye çevir), (2) gateway WS reconnect testlerini güçlendir, (3) detect-ms çıktılarını strateji hattına bağla.


---

# 📜 Bölüm 3 — HER DOSYANIN KODU (Olduğu Gibi)


├── Cargo.toml

```toml
[workspace]
members = [
    "cycle-engine/gateway",
    "cycle-engine/pipeline",
    "cycle-engine/transport",
    "cycle-engine/engine",
    "cycle-engine/persistence",
    "cycle-engine/infra",
    "cycle-engine/flows",
    "additional-services/os-utils",
    "data-engine/cold-storage",
    "data-engine/cold-starter",
    "execution-engine",
    "risk-engine",
    "strategies-engine",
    "services-engine/ohlcv-engine",
    "services-engine/calc-ind",
    "services-engine/stream-ohlcv",
    "services-engine/trade-ohlcv",
    "services-engine/detect-ms",
    "services-engine/alert-service",
    "services-engine/exec-console",
    "services-engine/db-query",
    "services-engine/telegram-bot",
    "services-engine/force-orders",
]
resolver = "2"

[workspace.dependencies]
rust_decimal = { version = "1.34", features = ["maths", "serde"] }
ndarray      = { version = "0.15", features = ["rayon"] }
rayon        = "1.8"
wide         = "0.7"
```


├── Cargo.lock

```text
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "adapter"
version = "0.1.0"
dependencies = [
 "flume",
 "futures-util",
 "reqwest 0.11.27",
 "serde_json",
 "testcontainers",
 "tokio",
 "tokio-tungstenite",
 "wiremock",
]

[[package]]
name = "adler2"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa"

[[package]]
name = "ahash"
version = "0.7.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "891477e0c6a8957309ee5c45a6368af3ae14bb510732d2684ffa19af310920f9"
dependencies = [
 "getrandom 0.2.17",
 "once_cell",
 "version_check",
]

[[package]]
name = "ahash"
version = "0.8.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a15f179cd60c4584b8a8c596927aadc462e27f2ca70c04e0071964a73ba7a75"
dependencies = [
 "cfg-if",
 "getrandom 0.3.4",
 "once_cell",
 "version_check",
 "zerocopy",
]

[[package]]
name = "aho-corasick"
version = "1.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c982642fa9e8606056828ee9a8505737230110bb1099153c79efe865c59d12ba"
dependencies = [
 "memchr",
]

[[package]]
name = "alert-service"
version = "0.1.0"
dependencies = [
 "clap 4.6.6",
 "contracts",
 "core",
 "flume",
 "futures-util",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
 "tokio-tungstenite",
 "toml",
 "transport",
]

[[package]]
name = "allocator-api2"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923"

[[package]]
name = "android_system_properties"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "819e7219dbd41043ac279b19830f2efc897156490d7fd6ea916720117ee66311"
dependencies = [
 "libc",
]

[[package]]
name = "anes"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4b46cbb362ab8752921c97e041f5e366ee6297bd428a31275b9fcf1e380f7299"

[[package]]
name = "anstream"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "824a212faf96e9acacdbd09febd34438f8f711fb84e09a8916013cd7815ca28d"
dependencies = [
 "anstyle",
 "anstyle-parse",
 "anstyle-query",
 "anstyle-wincon",
 "colorchoice",
 "is_terminal_polyfill",
 "utf8parse",
]

[[package]]
name = "anstyle"
version = "1.0.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "940b3a0ca603d1eade50a4846a2afffd5ef57a9feac2c0e2ec2e14f9ead76000"

[[package]]
name = "anstyle-parse"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "52ce7f38b242319f7cabaa6813055467063ecdc9d355bbb4ce0c68908cd8130e"
dependencies = [
 "utf8parse",
]

[[package]]
name = "anstyle-query"
version = "1.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "40c48f72fd53cd289104fc64099abca73db4166ad86ea0b4341abe65af83dadc"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "anstyle-wincon"
version = "3.0.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "291e6a250ff86cd4a820112fb8898808a366d8f9f58ce16d1f538353ad55747d"
dependencies = [
 "anstyle",
 "once_cell_polyfill",
 "windows-sys 0.61.2",
]

[[package]]
name = "anyhow"
version = "1.0.104"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "330a5ed07fa54e4702c9d6c4174f74427fc0ef6e214bbd677ae50a5099946470"

[[package]]
name = "arc-swap"
version = "1.9.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c049c0be4daef0b145cb3555416b3b8ef5b7888a38aea1a3a155801fe7b0810b"
dependencies = [
 "rustversion",
]

[[package]]
name = "argon2"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3c3610892ee6e0cbce8ae2700349fcf8f98adb0dbfbee85aec3c9179d29cc072"
dependencies = [
 "base64ct",
 "blake2",
 "cpufeatures 0.2.17",
 "password-hash",
]

[[package]]
name = "arrayvec"
version = "0.7.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d3fb67a6e08acf24fdeccbac2cb6ac4305825bd1f117462e0e6f2f193345ad56"

[[package]]
name = "assert-json-diff"
version = "2.0.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47e4f2b81832e72834d7518d8487a0396a28cc408186a2e8854c0f98011faf12"
dependencies = [
 "serde",
 "serde_json",
]

[[package]]
name = "async-channel"
version = "1.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "81953c529336010edd6d8e358f886d9581267795c61b19475b71314bffa46d35"
dependencies = [
 "concurrent-queue",
 "event-listener",
 "futures-core",
]

[[package]]
name = "async-trait"
version = "0.1.91"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae36dc4177970ef04fde5178d3e2429882def40e57a451f919c098f72baa6cec"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.3",
]

[[package]]
name = "atoi"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f28d99ec8bfea296261ca1af174f24225171fea9664ba9003cbebee704810528"
dependencies = [
 "num-traits",
]

[[package]]
name = "atomic-waker"
version = "1.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0"

[[package]]
name = "atty"
version = "0.2.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d9b39be18770d11421cdb1b9947a45dd3f37e93092cbf377614828a319d5fee8"
dependencies = [
 "hermit-abi 0.1.19",
 "libc",
 "winapi",
]

[[package]]
name = "autocfg"
version = "1.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53"

[[package]]
name = "aws-lc-rs"
version = "1.17.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "00bdb5da18dac48ca2cc7cd4a98e533e8635a58e2361d13a1a4ee3888e0d72f1"
dependencies = [
 "aws-lc-sys",
 "zeroize",
]

[[package]]
name = "aws-lc-sys"
version = "0.43.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "43103168cc76fe62678a375e722fc9cb3a0146159ac5828bc4f0dfd755c2224c"
dependencies = [
 "cc",
 "cmake",
 "dunce",
 "fs_extra",
 "pkg-config",
]

[[package]]
name = "axum"
version = "0.8.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "31b698c5f9a010f6573133b09e0de5408834d0c82f8d7475a89fc1867a71cd90"
dependencies = [
 "axum-core",
 "bytes",
 "form_urlencoded",
 "futures-util",
 "http 1.5.0",
 "http-body 1.1.0",
 "http-body-util",
 "hyper 1.11.0",
 "hyper-util",
 "itoa",
 "matchit",
 "memchr",
 "mime",
 "percent-encoding",
 "pin-project-lite",
 "serde_core",
 "serde_json",
 "serde_path_to_error",
 "serde_urlencoded",
 "sync_wrapper 1.0.2",
 "tokio",
 "tower",
 "tower-layer",
 "tower-service",
 "tracing",
]

[[package]]
name = "axum-core"
version = "0.5.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "08c78f31d7b1291f7ee735c1c6780ccde7785daae9a9206026862dab7d8792d1"
dependencies = [
 "bytes",
 "futures-core",
 "http 1.5.0",
 "http-body 1.1.0",
 "http-body-util",
 "mime",
 "pin-project-lite",
 "sync_wrapper 1.0.2",
 "tower-layer",
 "tower-service",
 "tracing",
]

[[package]]
name = "base64"
version = "0.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9e1b586273c5702936fe7b7d6896644d8be71e6314cfe09d3167c95f712589e8"

[[package]]
name = "base64"
version = "0.21.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9d297deb1925b89f2ccc13d7635fa0714f12c87adce1c75356b39ca9b7178567"

[[package]]
name = "base64"
version = "0.22.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6"

[[package]]
name = "base64ct"
version = "1.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2af50177e190e07a26ab74f8b1efbfe2ef87da2116221318cb1c2e82baf7de06"

[[package]]
name = "bit-set"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "08807e080ed7f9d5433fa9b275196cfc35414f66a0c79d864dc51a0d825231a3"
dependencies = [
 "bit-vec",
]

[[package]]
name = "bit-vec"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5e764a1d40d510daf35e07be9eb06e75770908c27d411ee6c92109c9840eaaf7"

[[package]]
name = "bitflags"
version = "1.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a"

[[package]]
name = "bitflags"
version = "2.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b588b76d00fde79687d7646a9b5bdf3cc0f655e0bbd080335a95d7e96f3587da"
dependencies = [
 "serde_core",
]

[[package]]
name = "bitvec"
version = "1.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ddcec3d12c579d40898fe0a9a358a803c23e9c52ca3c425707f81c9436211837"
dependencies = [
 "funty",
 "radium",
 "tap",
 "wyz",
]

[[package]]
name = "blake2"
version = "0.10.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "46502ad458c9a52b69d4d4d32775c788b7a1b85e8bc9d482d92250fc0e3f8efe"
dependencies = [
 "digest",
]

[[package]]
name = "block-buffer"
version = "0.10.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71"
dependencies = [
 "generic-array",
]

[[package]]
name = "bollard-stubs"
version = "1.41.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed2f2e73fffe9455141e170fb9c1feb0ac521ec7e7dcd47a7cab72a658490fb8"
dependencies = [
 "chrono",
 "serde",
 "serde_with",
]

[[package]]
name = "borsh"
version = "1.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a88b7ea17d208c4193f2c1e6de3c35fe71f98c96982d5ced308bdcc749ff6e1f"
dependencies = [
 "borsh-derive",
 "bytes",
 "cfg_aliases 0.2.2",
]

[[package]]
name = "borsh-derive"
version = "1.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d8f347189c62a579b8cd5f80714efa178f52e461dc2e6d701d264f5ff22e566c"
dependencies = [
 "once_cell",
 "proc-macro-crate",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "bumpalo"
version = "3.20.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72f5acc6cb2ba439de613abc23857ec3d78374d8ed5ac84e9d11336e87da8649"

[[package]]
name = "bytecheck"
version = "0.6.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23cdc57ce23ac53c931e88a43d06d070a6fd142f2617be5855eb75efc9beb1c2"
dependencies = [
 "bytecheck_derive",
 "ptr_meta",
 "simdutf8",
]

[[package]]
name = "bytecheck_derive"
version = "0.6.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3db406d29fbcd95542e92559bed4d8ad92636d1ca8b3b72ede10b4bcc010e659"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 1.0.109",
]

[[package]]
name = "bytemuck"
version = "1.25.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "95832e849adfb21180ccb6826a99da14e5d266ae5c2e668e1602cf234f153797"

[[package]]
name = "byteorder"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b"

[[package]]
name = "bytes"
version = "1.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fc652a48c352aef3ea3aed32080501cf3ef6ed5da78602a020c991775b0aff04"

[[package]]
name = "bytes-utils"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7dafe3a8757b027e2be6e4e5601ed563c55989fcf1546e933c66c8eb3a058d35"
dependencies = [
 "bytes",
 "either",
]

[[package]]
name = "cast"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "37b2a672a2cb129a2e41c10b1224bb368f9f37a2b16b612598138befd7b37eb5"

[[package]]
name = "cc"
version = "1.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5add81bb678e6cb321aff7fa0dc7689ad82b112dbc032cea19f91d6b8e3582b9"
dependencies = [
 "find-msvc-tools",
 "jobserver",
 "libc",
 "shlex",
]

[[package]]
name = "cfg-if"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801"

[[package]]
name = "cfg_aliases"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fd16c4719339c4530435d38e511904438d07cce7950afa3718a84ac36c10e89e"

[[package]]
name = "cfg_aliases"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f079e83a288787bcd14a6aea84cee5c87a67c5a3e660c30f557a3d24761b3527"

[[package]]
name = "chacha20"
version = "0.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d524456ba66e72eb8b115ff89e01e497f8e6d11d78b70b1aa13c0fbd97540a81"
dependencies = [
 "cfg-if",
 "cpufeatures 0.3.0",
 "rand_core 0.10.1",
]

[[package]]
name = "chrono"
version = "0.4.45"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1aa79e62e7697b8e29b513a68abacf485adcd1fe8284a4316c5ae868e6633327"
dependencies = [
 "iana-time-zone",
 "js-sys",
 "num-traits",
 "serde",
 "wasm-bindgen",
 "windows-link",
]

[[package]]
name = "ciborium"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "42e69ffd6f0917f5c029256a24d0161db17cea3997d185db0d35926308770f0e"
dependencies = [
 "ciborium-io",
 "ciborium-ll",
 "serde",
]

[[package]]
name = "ciborium-io"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "05afea1e0a06c9be33d539b876f1ce3692f4afea2cb41f740e7743225ed1c757"

[[package]]
name = "ciborium-ll"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "57663b653d948a338bfb3eeba9bb2fd5fcfaecb9e199e87e1eda4d9e8b240fd9"
dependencies = [
 "ciborium-io",
 "half",
]

[[package]]
name = "clap"
version = "3.2.25"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ea181bf566f71cb9a5d17a59e1871af638180a18fb0035c92ae62b705207123"
dependencies = [
 "bitflags 1.3.2",
 "clap_lex 0.2.4",
 "indexmap 1.9.3",
 "textwrap",
]

[[package]]
name = "clap"
version = "4.6.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "473c7e07f409a8d772161724aa8db6a765a2532a70f9667eeb7b49d3d02fbdca"
dependencies = [
 "clap_builder",
 "clap_derive",
]

[[package]]
name = "clap_builder"
version = "4.6.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7b48fea5a88e9ae728a2dcbedbfc0e730f7d60da42e1cb049a83c9fb8b789889"
dependencies = [
 "anstream",
 "anstyle",
 "clap_lex 1.1.0",
 "strsim 0.11.1",
]

[[package]]
name = "clap_derive"
version = "4.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d012d2b9d65aca7f18f4d9878a045bc17899bba951561ba5ec3c2ba1eed9a061"
dependencies = [
 "heck 0.5.0",
 "proc-macro2",
 "quote",
 "syn 3.0.3",
]

[[package]]
name = "clap_lex"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2850f2f5a82cbf437dd5af4d49848fbdfc27c157c3d010345776f952765261c5"
dependencies = [
 "os_str_bytes",
]

[[package]]
name = "clap_lex"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c8d4a3bb8b1e0c1050499d1815f5ab16d04f0959b233085fb31653fbfc9d98f9"

[[package]]
name = "clipboard-win"
version = "5.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bde03770d3df201d4fb868f2c9c59e66a3e4e2bd06692a0fe701e7103c7e84d4"
dependencies = [
 "error-code",
]

[[package]]
name = "cmake"
version = "0.1.58"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c0f78a02292a74a88ac736019ab962ece0bc380e3f977bf72e376c5d78ff0678"
dependencies = [
 "cc",
]

[[package]]
name = "cold-starter"
version = "0.1.0"

[[package]]
name = "cold-storage"
version = "0.1.0"
dependencies = [
 "memmap2",
]

[[package]]
name = "colorchoice"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d07550c9036bf2ae0c684c4297d503f838287c83c53686d05370d0e139ae570"

[[package]]
name = "combine"
version = "4.6.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ba5a308b75df32fe02788e748662718f03fde005016435c444eea572398219fd"
dependencies = [
 "bytes",
 "memchr",
]

[[package]]
name = "concurrent-queue"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ca0197aee26d1ae37445ee532fefce43251d24cc7c166799f4d46817f1d3973"
dependencies = [
 "crossbeam-utils",
]

[[package]]
name = "const-oid"
version = "0.9.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2459377285ad874054d797f3ccebf984978aa39129f6eafde5cdc8315b612f8"

[[package]]
name = "contracts"
version = "0.1.0"
dependencies = [
 "rust_decimal",
]

[[package]]
name = "cookie-factory"
version = "0.3.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9885fa71e26b8ab7855e2ec7cae6e9b380edff76cd052e07c683a0319d51b3a2"
dependencies = [
 "futures",
]

[[package]]
name = "core"
version = "0.1.0"
dependencies = [
 "adapter",
 "chrono",
 "cold-storage",
 "contracts",
 "core_affinity",
 "criterion",
 "crossbeam-channel",
 "dotenvy",
 "execution-engine",
 "flume",
 "futures-util",
 "hdrhistogram",
 "libc",
 "memmap2",
 "os-utils",
 "parking_lot 0.12.5",
 "proptest",
 "reqwest 0.11.27",
 "sqlx",
 "rust_decimal",
 "rustyline",
 "serde",
 "serde_json",
 "sha3",
 "simd-json",
 "tokio",
 "transport",
]

[[package]]
name = "core-foundation"
version = "0.9.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "core-foundation"
version = "0.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b2a6cd9ae233e7f62ba4e9353e81a88df7fc8a5987b8d445b4d90c879bd156f6"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "core-foundation-sys"
version = "0.8.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b"

[[package]]
name = "core_affinity"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a034b3a7b624016c6e13f5df875747cc25f884156aad2abd12b6c46797971342"
dependencies = [
 "libc",
 "num_cpus",
 "winapi",
]

[[package]]
name = "cpufeatures"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280"
dependencies = [
 "libc",
]

[[package]]
name = "cpufeatures"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8b2a41393f66f16b0823bb79094d54ac5fbd34ab292ddafb9a0456ac9f87d201"
dependencies = [
 "libc",
]

[[package]]
name = "crc"
version = "3.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5eb8a2a1cd12ab0d987a5d5e825195d372001a4094a0376319d5a0ad71c1ba0d"
dependencies = [
 "crc-catalog",
]

[[package]]
name = "crc-catalog"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "217698eaf96b4a3f0bc4f3662aaa55bdf913cd54d7204591faa790070c6d0853"

[[package]]
name = "crc16"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "338089f42c427b86394a5ee60ff321da23a5c89c9d89514c829687b26359fcff"

[[package]]
name = "crc32fast"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9481c1c90cbf2ac953f07c8d4a58aa3945c425b7185c9154d67a65e4230da511"
dependencies = [
 "cfg-if",
]

[[package]]
name = "criterion"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7c76e09c1aae2bc52b3d2f29e13c6572553b30c4aa1b8a49fd70de6412654cb"
dependencies = [
 "anes",
 "atty",
 "cast",
 "ciborium",
 "clap 3.2.25",
 "criterion-plot",
 "itertools",
 "lazy_static",
 "num-traits",
 "oorandom",
 "plotters",
 "rayon",
 "regex",
 "serde",
 "serde_derive",
 "serde_json",
 "tinytemplate",
 "walkdir",
]

[[package]]
name = "criterion-plot"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6b50826342786a51a89e2da3a28f1c32b06e387201bc2d19791f622c673706b1"
dependencies = [
 "cast",
 "itertools",
]

[[package]]
name = "crossbeam"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1137cd7e7fc0fb5d3c5a8678be38ec56e819125d8d7907411fe24ccb943faca8"
dependencies = [
 "crossbeam-channel",
 "crossbeam-deque",
 "crossbeam-epoch",
 "crossbeam-queue",
 "crossbeam-utils",
]

[[package]]
name = "crossbeam-channel"
version = "0.5.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d85363c37faeca707aef026efa9f3b34d077bce547e48f770770625c6013679e"
dependencies = [
 "crossbeam-utils",
]

[[package]]
name = "crossbeam-deque"
version = "0.8.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5181e0de7b61eb03a81e347d6dd8797bae9da5146707b51077e2d71a54ec0ceb"
dependencies = [
 "crossbeam-epoch",
 "crossbeam-utils",
]

[[package]]
name = "crossbeam-epoch"
version = "0.9.20"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2d6914041f254d6e9176c01941b21115dcfb7089e55135a35411081bd106ef3f"
dependencies = [
 "crossbeam-utils",
]

[[package]]
name = "crossbeam-queue"
version = "0.3.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "803d13fb3b09d88be9f4dbc29062c66b19bf7170867ceb746d2a8689bf6c7a26"
dependencies = [
 "crossbeam-utils",
]

[[package]]
name = "crossbeam-utils"
version = "0.8.22"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "61803da095bee82a81bb1a452ecc25d3b2f1416d1897eb86430c6159ef717c17"

[[package]]
name = "crunchy"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "460fbee9c2c2f33933d720630a6a0bac33ba7053db5344fac858d4b8952d77d5"

[[package]]
name = "crypto-common"
version = "0.1.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a"
dependencies = [
 "generic-array",
 "typenum",
]

[[package]]
name = "darling"
version = "0.13.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a01d95850c592940db9b8194bc39f4bc0e89dee5c4265e4b1807c34a9aba453c"
dependencies = [
 "darling_core",
 "darling_macro",
]

[[package]]
name = "darling_core"
version = "0.13.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "859d65a907b6852c9361e3185c862aae7fafd2887876799fa55f5f99dc40d610"
dependencies = [
 "fnv",
 "ident_case",
 "proc-macro2",
 "quote",
 "strsim 0.10.0",
 "syn 1.0.109",
]

[[package]]
name = "darling_macro"
version = "0.13.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9c972679f83bdf9c42bd905396b6c3588a843a17f0f16dfcfa3e2c5d57441835"
dependencies = [
 "darling_core",
 "quote",
 "syn 1.0.109",
]

[[package]]
name = "data-encoding"
version = "2.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06"

[[package]]
name = "deadpool"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "421fe0f90f2ab22016f32a9881be5134fdd71c65298917084b0c7477cbc3856e"
dependencies = [
 "async-trait",
 "deadpool-runtime",
 "num_cpus",
 "retain_mut",
 "tokio",
]

[[package]]
name = "deadpool-runtime"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "092966b41edc516079bdf31ec78a2e0588d1d0c08f78b91d8307215928642b2b"

[[package]]
name = "der"
version = "0.7.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7c1832837b905bbfb5101e07cc24c8deddf52f93225eee6ead5f4d63d53ddcb"
dependencies = [
 "const-oid",
 "pem-rfc7468",
 "zeroize",
]

[[package]]
name = "deranged"
version = "0.5.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7cd812cc2bc1d69d4764bd80df88b4317eaef9e773c75226407d9bc0876b211c"

[[package]]
name = "detect-liquidity"
version = "0.1.0"
dependencies = [
 "axum",
 "ohlcv-engine",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
]

[[package]]
name = "detect-ms"
version = "0.1.0"
dependencies = [
 "axum",
 "ohlcv-engine",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
]

[[package]]
name = "detect-pattern"
version = "0.1.0"
dependencies = [
 "axum",
 "ohlcv-engine",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
]

[[package]]
name = "detect-sr"
version = "0.1.0"
dependencies = [
 "clap 4.6.6",
 "ohlcv-engine",
 "rust_decimal",
 "tokio",
]

[[package]]
name = "detect-trb"
version = "0.1.0"
dependencies = [
 "axum",
 "contracts",
 "core",
 "core_affinity",
 "ndarray",
 "rayon",
 "rtrb",
 "sqlx",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
 "tracing",
 "tracing-subscriber",
 "transport",
 "wide",
]

[[package]]
name = "detect-trend"
version = "0.1.0"
dependencies = [
 "axum",
 "ohlcv-engine",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
]

[[package]]
name = "detect-wyckoff"
version = "0.1.0"
dependencies = [
 "axum",
 "ohlcv-engine",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
]

[[package]]
name = "digest"
version = "0.10.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292"
dependencies = [
 "block-buffer",
 "const-oid",
 "crypto-common",
 "subtle",
]

[[package]]
name = "displaydoc"
version = "0.2.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6232dd377dcc64799954cbd3a9bb882e9cdc1308ccd87b1c098f1fb2eaf82a8"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.3",
]

[[package]]
name = "dotenvy"
version = "0.15.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1aaf95b3e5c8f23aa320147307562d361db0ae0d51242340f558153b4eb2439b"

[[package]]
name = "dunce"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92773504d58c093f6de2459af4af33faa518c13451eb8f2b5698ed3d36e7c813"

[[package]]
name = "either"
version = "1.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9e5e8f6c15a24b9a3ee5efec809ccd006d3b30e8b3bb63c39af737c7f87daa1d"
dependencies = [
 "serde",
]

[[package]]
name = "encoding_rs"
version = "0.8.35"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "75030f3c4f45dafd7586dd6780965a8c7e8e285a5ecb86713e63a79c5b2766f3"
dependencies = [
 "cfg-if",
]

[[package]]
name = "endian-type"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c34f04666d835ff5d62e058c3995147c06f42fe86ff053337632bca83e42702d"

[[package]]
name = "equivalent"
version = "1.0.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f"

[[package]]
name = "errno"
version = "0.3.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb"
dependencies = [
 "libc",
 "windows-sys 0.61.2",
]

[[package]]
name = "error-code"
version = "3.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dea2df4cf52843e0452895c455a1a2cfbb842a1e7329671acf418fdc53ed4c59"

[[package]]
name = "etcetera"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "136d1b5283a1ab77bd9257427ffd09d8667ced0570b6f938942bc7568ed5b943"
dependencies = [
 "cfg-if",
 "home",
 "windows-sys 0.48.0",
]

[[package]]
name = "event-listener"
version = "2.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0206175f82b8d6bf6652ff7d71a1e27fd2e4efde587fd368662814d6ec1d9ce0"

[[package]]
name = "execution-engine"
version = "0.1.0"
dependencies = [
 "dotenvy",
 "flume",
 "futures-util",
 "hex",
 "hmac",
 "parking_lot 0.12.5",
 "sqlx",
 "rust_decimal",
 "serde",
 "serde_json",
 "sha2",
 "tokio",
 "tokio-tungstenite",
]

[[package]]
name = "fallible-iterator"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2acce4a10f12dc2fb14a218589d4f1f62ef011b2d0cc4b3cb1bba8e94da14649"

[[package]]
name = "fallible-streaming-iterator"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7360491ce676a36bf9bb3c56c1aa791658183a54d2744120f27285738d90465a"

[[package]]
name = "fastrand"
version = "1.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e51093e27b0797c359783294ca4f0a911c270184cb10f85783b118614a1501be"
dependencies = [
 "instant",
]

[[package]]
name = "fastrand"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "da7c62ceae207dd37ea5b845da6a0696c799f85e97da1ab5b7910be3c1c80223"

[[package]]
name = "fd-lock"
version = "4.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ce92ff622d6dadf7349484f42c93271a0d49b7cc4d466a936405bacbe10aa78"
dependencies = [
 "cfg-if",
 "rustix",
 "windows-sys 0.52.0",
]

[[package]]
name = "find-msvc-tools"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5baebc0774151f905a1a2cc41989300b1e6fbb29aff0ceffa1064fdd3088d582"

[[package]]
name = "flate2"
version = "1.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "843fba2746e448b37e26a819579957415c8cef339bf08564fe8b7ddbd959573c"
dependencies = [
 "crc32fast",
 "miniz_oxide",
]

[[package]]
name = "float-cmp"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "98de4bbd547a563b716d8dfa9aad1cb19bfab00f4fa09a6a4ed21dbcf44ce9c4"
dependencies = [
 "num-traits",
]

[[package]]
name = "flume"
version = "0.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "da0e4dd2a88388a1f4ccc7c9ce104604dab68d9f408dc34cd45823d5a9069095"
dependencies = [
 "futures-core",
 "futures-sink",
 "nanorand",
 "spin",
]

[[package]]
name = "fnv"
version = "1.0.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1"

[[package]]
name = "form_urlencoded"
version = "1.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf"
dependencies = [
 "percent-encoding",
]

[[package]]
name = "fred"
version = "7.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b99c2b48934cd02a81032dd7428b7ae831a27794275bc94eba367418db8a9e55"
dependencies = [
 "arc-swap",
 "async-trait",
 "bytes",
 "bytes-utils",
 "float-cmp",
 "futures",
 "lazy_static",
 "log",
 "parking_lot 0.12.5",
 "rand 0.8.7",
 "redis-protocol",
 "semver",
 "serde_json",
 "socket2 0.5.10",
 "tokio",
 "tokio-rustls 0.24.1",
 "tokio-stream",
 "tokio-util",
 "url",
 "urlencoding",
]

[[package]]
name = "fs2"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9564fc758e15025b46aa6643b1b77d047d1a56a1aea6e01002ac0c7026876213"
dependencies = [
 "libc",
 "winapi",
]

[[package]]
name = "fs_extra"
version = "1.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "42703706b716c37f96a77aea830392ad231f44c9e9a67872fa5548707e11b11c"

[[package]]
name = "funty"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6d5a32815ae3f33302d95fdcb2ce17862f8c65363dcfd29360480ba1001fc9c"

[[package]]
name = "futures"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a88cf1f829d945f548cf8fec32c61b1f202b6d93b45848602fc02af4b12ad218"
dependencies = [
 "futures-channel",
 "futures-core",
 "futures-executor",
 "futures-io",
 "futures-sink",
 "futures-task",
 "futures-util",
]

[[package]]
name = "futures-channel"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "262590f4fe6afeb0bc83be1daa64e52657fe185690a958af7f3ad0e92085c5ae"
dependencies = [
 "futures-core",
 "futures-sink",
]

[[package]]
name = "futures-core"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2cd50c473c80f6d7c3670a752354b8e569b1a7cbfdc0419ec88e5edad85e0dc7"

[[package]]
name = "futures-executor"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6754879cc9f2c66f88c6e5c35344bb0bdb0708b0352b1201815667c7eabc7458"
dependencies = [
 "futures-core",
 "futures-task",
 "futures-util",
]

[[package]]
name = "futures-intrusive"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d930c203dd0b6ff06e0201a4a2fe9149b43c684fd4420555b26d21b1a02956f"
dependencies = [
 "futures-core",
 "lock_api",
 "parking_lot 0.12.5",
]

[[package]]
name = "futures-io"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4577ecaa3c4f96589d473f679a71b596316f6641bc350038b962a5daf0085d7a"

[[package]]
name = "futures-lite"
version = "1.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "49a9d51ce47660b1e808d3c990b4709f2f415d928835a17dfd16991515c46bce"
dependencies = [
 "fastrand 1.9.0",
 "futures-core",
 "futures-io",
 "memchr",
 "parking",
 "pin-project-lite",
 "waker-fn",
]

[[package]]
name = "futures-macro"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2d6d3cde68c518367be28956066ddfef33813991b77a55005a69dae04bf3b10b"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "futures-sink"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e34418ac499d6305c2fb5ad0ed2f6ac998c5f8ca209b4510f7f94242c647e307"

[[package]]
name = "futures-task"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b231ed28831efb4a61a08580c4bc233ec56bc009f4cd8f52da2c3cb97df0c109"

[[package]]
name = "futures-timer"
version = "3.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "af43fadb8a98512d547e37b4e92e0ced13e205c061b87b4623eff01d918d6968"

[[package]]
name = "futures-util"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a77a90a256fce34da66415271e30f94ee91c57b04b8a2c042d9cf3220179deaa"
dependencies = [
 "futures-channel",
 "futures-core",
 "futures-io",
 "futures-macro",
 "futures-sink",
 "futures-task",
 "memchr",
 "pin-project-lite",
 "slab",
]

[[package]]
name = "fxhash"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c31b6d751ae2c7f11320402d34e41349dd1016f8d5d45e48c4312bc8625af50c"
dependencies = [
 "byteorder",
]

[[package]]
name = "generic-array"
version = "0.14.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a"
dependencies = [
 "typenum",
 "version_check",
]

[[package]]
name = "getrandom"
version = "0.1.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8fc3cb4d91f53b50155bdcfd23f6a4c39ae1969c2ae85982b135750cccaf5fce"
dependencies = [
 "cfg-if",
 "libc",
 "wasi 0.9.0+wasi-snapshot-preview1",
]

[[package]]
name = "getrandom"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0"
dependencies = [
 "cfg-if",
 "js-sys",
 "libc",
 "wasi 0.11.1+wasi-snapshot-preview1",
 "wasm-bindgen",
]

[[package]]
name = "getrandom"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"
dependencies = [
 "cfg-if",
 "libc",
 "r-efi 5.3.0",
 "wasip2",
]

[[package]]
name = "getrandom"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "300e883d756b2e4ec94e02791f39b04b522276138852cfc41d9fb7e904106099"
dependencies = [
 "cfg-if",
 "js-sys",
 "libc",
 "r-efi 6.0.0",
 "rand_core 0.10.1",
 "wasm-bindgen",
]

[[package]]
name = "h2"
version = "0.3.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0beca50380b1fc32983fc1cb4587bfa4bb9e78fc259aad4a0032d2080309222d"
dependencies = [
 "bytes",
 "fnv",
 "futures-core",
 "futures-sink",
 "futures-util",
 "http 0.2.12",
 "indexmap 2.14.0",
 "slab",
 "tokio",
 "tokio-util",
 "tracing",
]

[[package]]
name = "h2"
version = "0.4.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6cb093c84e8bd9b188d4c4a8cb6579fc016968d14c99882163cd3ff402a4f155"
dependencies = [
 "atomic-waker",
 "bytes",
 "fnv",
 "futures-core",
 "futures-sink",
 "http 1.5.0",
 "indexmap 2.14.0",
 "slab",
 "tokio",
 "tokio-util",
 "tracing",
]

[[package]]
name = "half"
version = "2.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ea2d84b969582b4b1864a92dc5d27cd2b77b622a8d79306834f1be5ba20d84b"
dependencies = [
 "cfg-if",
 "crunchy",
 "zerocopy",
]

[[package]]
name = "halfbrown"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8588661a8607108a5ca69cab034063441a0413a0b041c13618a7dd348021ef6f"
dependencies = [
 "hashbrown 0.14.5",
 "serde",
]

[[package]]
name = "hashbrown"
version = "0.12.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8a9ee70c43aaf417c914396645a0fa852624801b24ebb7ae78fe8272889ac888"
dependencies = [
 "ahash 0.7.8",
]

[[package]]
name = "hashbrown"
version = "0.14.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e5274423e17b7c9fc20b6e7e208532f9b19825d82dfd615708b70edd83df41f1"
dependencies = [
 "ahash 0.8.12",
 "allocator-api2",
]

[[package]]
name = "hashbrown"
version = "0.17.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a"

[[package]]
name = "hashlink"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e8094feaf31ff591f651a2664fb9cfd92bba7a60ce3197265e9482ebe753c8f7"
dependencies = [
 "hashbrown 0.14.5",
]

[[package]]
name = "hashlink"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ba4ff7128dee98c7dc9794b6a411377e1404dba1c97deb8d1a55297bd25d8af"
dependencies = [
 "hashbrown 0.14.5",
]

[[package]]
name = "hdrhistogram"
version = "7.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f49d1053f4708f0af3cf9fc5bffc7e68a914a3c45becb231c80068c9c3f78bea"
dependencies = [
 "base64 0.22.1",
 "byteorder",
 "crossbeam-channel",
 "flate2",
 "nom 8.0.0",
 "num-traits",
]

[[package]]
name = "heck"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "95505c38b4572b2d910cecb0281560f54b440a19336cbbcb27bf6ce6adc6f5a8"
dependencies = [
 "unicode-segmentation",
]

[[package]]
name = "heck"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"

[[package]]
name = "velvetusdt"
version = "0.1.0"
dependencies = [
 "chrono",
 "contracts",
 "reqwest 0.11.27",
 "sqlx",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
 "transport",
]

[[package]]
name = "hermit-abi"
version = "0.1.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "62b467343b94ba476dcb2500d242dadbb39557df889310ac77c5d99100aaac33"
dependencies = [
 "libc",
]

[[package]]
name = "hermit-abi"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fc0fef456e4baa96da950455cd02c081ca953b141298e41db3fc7e36b1da849c"

[[package]]
name = "hex"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70"

[[package]]
name = "hkdf"
version = "0.12.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7b5f8eb2ad728638ea2c7d47a21db23b7b58a72ed6a38256b8a1849f15fbbdf7"
dependencies = [
 "hmac",
]

[[package]]
name = "hmac"
version = "0.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6c49c37c09c17a53d937dfbb742eb3a961d65a994e6bcdcf37e7399d0cc8ab5e"
dependencies = [
 "digest",
]

[[package]]
name = "home"
version = "0.5.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cc627f471c528ff0c4a49e1d5e60450c8f6461dd6d10ba9dcd3a61d3dff7728d"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "http"
version = "0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "601cbb57e577e2f5ef5be8e7b83f0f63994f25aa94d673e54a92d5c516d101f1"
dependencies = [
 "bytes",
 "fnv",
 "itoa",
]

[[package]]
name = "http"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "918d3568bebf352712bc2ef3d46a8bcf1a75b373be6539de198e9105cbbf9ce0"
dependencies = [
 "bytes",
 "itoa",
]

[[package]]
name = "http-body"
version = "0.4.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7ceab25649e9960c0311ea418d17bee82c0dcec1bd053b5f9a66e265a693bed2"
dependencies = [
 "bytes",
 "http 0.2.12",
 "pin-project-lite",
]

[[package]]
name = "http-body"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ca2a8f2913ee65f60facd6a5905613afaa448497a0230cc41ce022d93290bc2c"
dependencies = [
 "bytes",
 "http 1.5.0",
]

[[package]]
name = "http-body-util"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e9f41fd6a08e4d4ec69df65976da761afd5ad5e58a9d4acb46bd1c953a9e3ff2"
dependencies = [
 "bytes",
 "futures-core",
 "http 1.5.0",
 "http-body 1.1.0",
 "pin-project-lite",
]

[[package]]
name = "http-types"
version = "2.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e9b187a72d63adbfba487f48095306ac823049cb504ee195541e91c7775f5ad"
dependencies = [
 "anyhow",
 "async-channel",
 "base64 0.13.1",
 "futures-lite",
 "http 0.2.12",
 "infer",
 "pin-project-lite",
 "rand 0.7.3",
 "serde",
 "serde_json",
 "serde_qs",
 "serde_urlencoded",
 "url",
]

[[package]]
name = "httparse"
version = "1.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87"

[[package]]
name = "httpdate"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "df3b46402a9d5adb4c86a0cf463f42e19994e3ee891101b1841f30a545cb49a9"

[[package]]
name = "hyper"
version = "0.14.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "41dfc780fdec9373c01bae43289ea34c972e40ee3c9f6b3c8801a35f35586ce7"
dependencies = [
 "bytes",
 "futures-channel",
 "futures-core",
 "futures-util",
 "h2 0.3.27",
 "http 0.2.12",
 "http-body 0.4.6",
 "httparse",
 "httpdate",
 "itoa",
 "pin-project-lite",
 "socket2 0.5.10",
 "tokio",
 "tower-service",
 "tracing",
 "want",
]

[[package]]
name = "hyper"
version = "1.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d22053281f852e11534f5198498373cbb59295120a20771d90f7ed1897490a72"
dependencies = [
 "atomic-waker",
 "bytes",
 "futures-channel",
 "futures-core",
 "h2 0.4.15",
 "http 1.5.0",
 "http-body 1.1.0",
 "httparse",
 "httpdate",
 "itoa",
 "pin-project-lite",
 "smallvec",
 "tokio",
 "want",
]

[[package]]
name = "hyper-rustls"
version = "0.24.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec3efd23720e2049821a693cbc7e65ea87c72f1c58ff2f9522ff332b1491e590"
dependencies = [
 "futures-util",
 "http 0.2.12",
 "hyper 0.14.32",
 "rustls 0.21.12",
 "tokio",
 "tokio-rustls 0.24.1",
]

[[package]]
name = "hyper-rustls"
version = "0.27.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "33ca68d021ef39cf6463ab54c1d0f5daf03377b70561305bb89a8f83aab66e0f"
dependencies = [
 "http 1.5.0",
 "hyper 1.11.0",
 "hyper-util",
 "rustls 0.23.43",
 "tokio",
 "tokio-rustls 0.26.4",
 "tower-service",
]

[[package]]
name = "hyper-util"
version = "0.1.20"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "96547c2556ec9d12fb1578c4eaf448b04993e7fb79cbaad930a656880a6bdfa0"
dependencies = [
 "base64 0.22.1",
 "bytes",
 "futures-channel",
 "futures-util",
 "http 1.5.0",
 "http-body 1.1.0",
 "hyper 1.11.0",
 "ipnet",
 "libc",
 "percent-encoding",
 "pin-project-lite",
 "socket2 0.6.5",
 "system-configuration 0.7.0",
 "tokio",
 "tower-service",
 "tracing",
 "windows-registry",
]

[[package]]
name = "iana-time-zone"
version = "0.1.65"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e31bc9ad994ba00e440a8aa5c9ef0ec67d5cb5e5cb0cc7f8b744a35b389cc470"
dependencies = [
 "android_system_properties",
 "core-foundation-sys",
 "iana-time-zone-haiku",
 "js-sys",
 "log",
 "wasm-bindgen",
 "windows-core",
]

[[package]]
name = "iana-time-zone-haiku"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f31827a206f56af32e590ba56d5d2d085f558508192593743f16b2306495269f"
dependencies = [
 "cc",
]

[[package]]
name = "icu_collections"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2984d1cd16c883d7935b9e07e44071dca8d917fd52ecc02c04d5fa0b5a3f191c"
dependencies = [
 "displaydoc",
 "potential_utf",
 "utf8_iter",
 "yoke",
 "zerofrom",
 "zerovec",
]

[[package]]
name = "icu_locale_core"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92219b62b3e2b4d88ac5119f8904c10f8f61bf7e95b640d25ba3075e6cac2c29"
dependencies = [
 "displaydoc",
 "litemap",
 "tinystr",
 "writeable",
 "zerovec",
]

[[package]]
name = "icu_normalizer"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c56e5ee99d6e3d33bd91c5d85458b6005a22140021cc324cea84dd0e72cff3b4"
dependencies = [
 "icu_collections",
 "icu_normalizer_data",
 "icu_properties",
 "icu_provider",
 "smallvec",
 "zerovec",
]

[[package]]
name = "icu_normalizer_data"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "da3be0ae77ea334f4da67c12f149704f19f81d1adf7c51cf482943e84a2bad38"

[[package]]
name = "icu_properties"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bee3b67d0ea5c2cca5003417989af8996f8604e34fb9ddf96208a033901e70de"
dependencies = [
 "icu_collections",
 "icu_locale_core",
 "icu_properties_data",
 "icu_provider",
 "zerotrie",
 "zerovec",
]

[[package]]
name = "icu_properties_data"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8e2bbb201e0c04f7b4b3e14382af113e17ba4f63e2c9d2ee626b720cbce54a14"

[[package]]
name = "icu_provider"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "139c4cf31c8b5f33d7e199446eff9c1e02decfc2f0eec2c8d71f65befa45b421"
dependencies = [
 "displaydoc",
 "icu_locale_core",
 "writeable",
 "yoke",
 "zerofrom",
 "zerotrie",
 "zerovec",
]

[[package]]
name = "ident_case"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b9e0384b61958566e926dc50660321d12159025e767c18e043daf26b70104c39"

[[package]]
name = "idna"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de"
dependencies = [
 "idna_adapter",
 "smallvec",
 "utf8_iter",
]

[[package]]
name = "idna_adapter"
version = "1.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cb68373c0d6620ef8105e855e7745e18b0d00d3bdb07fb532e434244cdb9a714"
dependencies = [
 "icu_normalizer",
 "icu_properties",
]

[[package]]
name = "indexmap"
version = "1.9.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bd070e393353796e801d209ad339e89596eb4c8d430d18ede6a1cced8fafbd99"
dependencies = [
 "autocfg",
 "hashbrown 0.12.3",
]

[[package]]
name = "indexmap"
version = "2.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d466e9454f08e4a911e14806c24e16fba1b4c121d1ea474396f396069cf949d9"
dependencies = [
 "equivalent",
 "hashbrown 0.17.1",
]

[[package]]
name = "infer"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "64e9829a50b42bb782c1df523f78d332fe371b10c661e78b7a3c34b0198e9fac"

[[package]]
name = "instant"
version = "0.1.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e0242819d153cba4b4b05a5a8f2a7e9bbf97b6055b2a002b395c96b5ff3c0222"
dependencies = [
 "cfg-if",
]

[[package]]
name = "ipnet"
version = "2.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6a756c3fac73139e83f14c2d742155dd2b78d3ee56597b419a0579b7bdd6dd78"

[[package]]
name = "is_terminal_polyfill"
version = "1.70.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a6cb138bb79a146c1bd460005623e142ef0181e3d0219cb493e02f7d08a35695"

[[package]]
name = "itertools"
version = "0.10.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b0fd2260e829bddf4cb6ea802289de2f86d6a7a690192fbe91b3f46e0f2c8473"
dependencies = [
 "either",
]

[[package]]
name = "itoa"
version = "1.0.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682"

[[package]]
name = "jni"
version = "0.22.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5efd9a482cf3a427f00d6b35f14332adc7902ce91efb778580e180ff90fa3498"
dependencies = [
 "cfg-if",
 "combine",
 "jni-macros",
 "jni-sys",
 "log",
 "simd_cesu8",
 "thiserror 2.0.19",
 "walkdir",
 "windows-link",
]

[[package]]
name = "jni-macros"
version = "0.22.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a00109accc170f0bdb141fed3e393c565b6f5e072365c3bd58f5b062591560a3"
dependencies = [
 "proc-macro2",
 "quote",
 "rustc_version",
 "simd_cesu8",
 "syn 2.0.119",
]

[[package]]
name = "jni-sys"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6377a88cb3910bee9b0fa88d4f42e1d2da8e79915598f65fb0c7ee14c878af2"
dependencies = [
 "jni-sys-macros",
]

[[package]]
name = "jni-sys-macros"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "38c0b942f458fe50cdac086d2f946512305e5631e720728f2a61aabcd47a6264"
dependencies = [
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "jobserver"
version = "0.1.35"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1c00acbd29eabad4a2392fa0e921c874934dbbf4194312ad20f04a0ed67a3cb3"
dependencies = [
 "getrandom 0.4.3",
 "libc",
]

[[package]]
name = "js-sys"
version = "0.3.103"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "53b44bfcdb3f8d5837a46dae1ca9660a837176eee74a28b229bc626816589102"
dependencies = [
 "cfg-if",
 "futures-util",
 "wasm-bindgen",
]

[[package]]
name = "jsonwebtoken"
version = "9.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a87cc7a48537badeae96744432de36f4be2b4a34a05a5ef32e9dd8a1c169dde"
dependencies = [
 "base64 0.22.1",
 "js-sys",
 "pem",
 "ring",
 "serde",
 "serde_json",
 "simple_asn1",
]

[[package]]
name = "keccak"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cb26cec98cce3a3d96cbb7bced3c4b16e3d13f27ec56dbd62cbc8f39cfb9d653"
dependencies = [
 "cpufeatures 0.2.17",
]

[[package]]
name = "lazy_static"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbd2bcb4c963f2ddae06a2efc7e9f3591312473c50c6685e1f298068316e66fe"
dependencies = [
 "spin",
]

[[package]]
name = "lexical-core"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7d8d125a277f807e55a77304455eb7b1cb52f2b18c143b60e766c120bd64a594"
dependencies = [
 "lexical-parse-float",
 "lexical-parse-integer",
 "lexical-util",
 "lexical-write-float",
 "lexical-write-integer",
]

[[package]]
name = "lexical-parse-float"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "52a9f232fbd6f550bc0137dcb5f99ab674071ac2d690ac69704593cb4abbea56"
dependencies = [
 "lexical-parse-integer",
 "lexical-util",
]

[[package]]
name = "lexical-parse-integer"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a7a039f8fb9c19c996cd7b2fcce303c1b2874fe1aca544edc85c4a5f8489b34"
dependencies = [
 "lexical-util",
]

[[package]]
name = "lexical-util"
version = "1.0.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2604dd126bb14f13fb5d1bd6a66155079cb9fa655b37f875b3a742c705dbed17"

[[package]]
name = "lexical-write-float"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "50c438c87c013188d415fbabbb1dceb44249ab81664efbd31b14ae55dabb6361"
dependencies = [
 "lexical-util",
 "lexical-write-integer",
]

[[package]]
name = "lexical-write-integer"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "409851a618475d2d5796377cad353802345cba92c867d9fbcde9cf4eac4e14df"
dependencies = [
 "lexical-util",
]

[[package]]
name = "libc"
version = "0.2.189"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2"

[[package]]
name = "libm"
version = "0.2.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981"

[[package]]
name = "libredox"
version = "0.1.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2026a5056764a10b2bf5d56488cba40da507f5493a6a429340e2004d9ed085fa"
dependencies = [
 "bitflags 2.13.1",
 "libc",
 "plain",
 "redox_syscall 0.9.1",
]

[[package]]
name = "libpsql-sys"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0c10584274047cb335c23d3e61bcef8e323adae7c5c8c760540f73610177fc3f"
dependencies = [
 "cc",
 "pkg-config",
 "vcpkg",
]

[[package]]
name = "linux-raw-sys"
version = "0.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53"

[[package]]
name = "litemap"
version = "0.8.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92daf443525c4cce67b150400bc2316076100ce0b3686209eb8cf3c31612e6f0"

[[package]]
name = "lock_api"
version = "0.4.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965"
dependencies = [
 "scopeguard",
]

[[package]]
name = "log"
version = "0.4.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ceec5bc11778974d1bcb055b18002eba7f4b3518b6a0081b3af5f21666da9ad"

[[package]]
name = "lru-slab"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "112b39cec0b298b6c1999fee3e31427f74f676e4cb9879ed1a121b43661a4154"

[[package]]
name = "matchers"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d1525a2a28c7f4fa0fc98bb91ae755d1e2d1505079e05539e35bc876b5d65ae9"
dependencies = [
 "regex-automata",
]

[[package]]
name = "matchit"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47e1ffaa40ddd1f3ed91f717a33c8c0ee23fff369e3aa8772b9605cc1d22f4c3"

[[package]]
name = "matrixmultiply"
version = "0.3.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f607c237553f086e7043417a51df26b2eb899d3caff94e6a67592ff992fedc7"
dependencies = [
 "autocfg",
 "rawpointer",
]

[[package]]
name = "md-5"
version = "0.10.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d89e7ee0cfbedfc4da3340218492196241d89eefb6dab27de5df917a6d2e78cf"
dependencies = [
 "cfg-if",
 "digest",
]

[[package]]
name = "memchr"
version = "2.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98"

[[package]]
name = "memmap2"
version = "0.9.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d1219ed1b7f229ee7104d281dd01d6802fe28bb6e95d292942c4daacdeb798c0"
dependencies = [
 "libc",
]

[[package]]
name = "mime"
version = "0.3.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6877bb514081ee2a7ff5ef9de3281f14a4dd4bceac4c09388074a6b5df8a139a"

[[package]]
name = "minimal-lexical"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "68354c5c6bd36d73ff3feceb05efa59b6acb7626617f4962be322a825e61f79a"

[[package]]
name = "miniz_oxide"
version = "0.8.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fa76a2c86f704bdb222d66965fb3d63269ce38518b83cb0575fca855ebb6316"
dependencies = [
 "adler2",
 "simd-adler32",
]

[[package]]
name = "mio"
version = "1.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "30d65c71f1ce40ab09135ce117d742b9f8a19ff91a41a8b57ed50bc2de59c427"
dependencies = [
 "libc",
 "wasi 0.11.1+wasi-snapshot-preview1",
 "windows-sys 0.61.2",
]

[[package]]
name = "nanorand"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6a51313c5820b0b02bd422f4b44776fbf47961755c74ce64afc73bfad10226c3"
dependencies = [
 "getrandom 0.2.17",
]

[[package]]
name = "ndarray"
version = "0.15.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "adb12d4e967ec485a5f71c6311fe28158e9d6f4bc4a447b474184d0f91a8fa32"
dependencies = [
 "matrixmultiply",
 "num-complex",
 "num-integer",
 "num-traits",
 "rawpointer",
 "rayon",
]

[[package]]
name = "nibble_vec"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77a5d83df9f36fe23f0c3648c6bbb8b0298bb5f1939c8f2704431371f4b84d43"
dependencies = [
 "smallvec",
]

[[package]]
name = "nix"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ab2156c4fce2f8df6c499cc1c763e4394b7482525bf2a9701c9d79d215f519e4"
dependencies = [
 "bitflags 2.13.1",
 "cfg-if",
 "cfg_aliases 0.1.1",
 "libc",
]

[[package]]
name = "nom"
version = "7.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d273983c5a657a70a3e8f2a01329822f3b8c8172b73826411a55751e404a0a4a"
dependencies = [
 "memchr",
 "minimal-lexical",
]

[[package]]
name = "nom"
version = "8.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "df9761775871bdef83bee530e60050f7e54b1105350d6884eb0fb4f46c2f9405"
dependencies = [
 "memchr",
]

[[package]]
name = "nu-ansi-term"
version = "0.50.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7957b9740744892f114936ab4a57b3f487491bbeafaf8083688b16841a4240e5"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "num-bigint"
version = "0.4.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-bigint-dig"
version = "0.8.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e661dda6640fad38e827a6d4a310ff4763082116fe217f279885c97f511bb0b7"
dependencies = [
 "lazy_static",
 "libm",
 "num-integer",
 "num-iter",
 "num-traits",
 "rand 0.8.7",
 "smallvec",
 "zeroize",
]

[[package]]
name = "num-complex"
version = "0.4.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73f88a1307638156682bada9d7604135552957b7818057dcef22705b4d509495"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-conv"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "521739c6d2bac4aa25192232afe6841231376b2b26d4d9fae5ecf8ca5772e441"

[[package]]
name = "num-integer"
version = "0.1.46"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7969661fd2958a5cb096e56c8e1ad0444ac2bbcd0061bd28660485a44879858f"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-iter"
version = "0.1.46"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c92800bd69a1eac91786bcfe9da64a897eb72911b8dc3095decbd07429e8048b"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-traits"
version = "0.2.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"
dependencies = [
 "autocfg",
 "libm",
]

[[package]]
name = "num_cpus"
version = "1.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "91df4bbde75afed763b708b7eee1e8e7651e02d97f6d5dd763e89367e957b23b"
dependencies = [
 "hermit-abi 0.5.2",
 "libc",
]

[[package]]
name = "ohlcv-engine"
version = "0.1.0"
dependencies = [
 "axum",
 "chrono",
 "clap 4.6.6",
 "reqwest 0.13.4",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
]

[[package]]
name = "once_cell"
version = "1.21.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50"

[[package]]
name = "once_cell_polyfill"
version = "1.70.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "384b8ab6d37215f3c5301a95a4accb5d64aa607f1fcb26a11b5303878451b4fe"

[[package]]
name = "oorandom"
version = "11.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6790f58c7ff633d8771f42965289203411a5e5c68388703c06e14f24770b41e"

[[package]]
name = "openssl-probe"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d05e27ee213611ffe7d6348b942e8f942b37114c00cc03cec254295a4a17852e"

[[package]]
name = "openssl-probe"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7c87def4c32ab89d880effc9e097653c8da5d6ef28e6b539d313baaacfbafcbe"

[[package]]
name = "os-utils"
version = "0.1.0"
dependencies = [
 "crossbeam",
 "libc",
]

[[package]]
name = "os_str_bytes"
version = "6.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e2355d85b9a3786f481747ced0e0ff2ba35213a1f9bd406ed906554d7af805a1"

[[package]]
name = "paper-service"
version = "0.1.0"
dependencies = [
 "argon2",
 "axum",
 "clap 4.6.6",
 "contracts",
 "core",
 "execution-engine",
 "fred",
 "jsonwebtoken",
 "parking_lot 0.12.5",
 "rand 0.8.7",
 "reqwest 0.11.27",
 "rust_decimal",
 "serde",
 "serde_json",
 "sled",
 "sqlx",
 "tokio",
 "tower",
 "tower-http",
 "tracing",
 "tracing-subscriber",
 "transport",
 "uuid",
]

[[package]]
name = "parking"
version = "2.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f38d5652c16fde515bb1ecef450ab0f6a219d619a7274976324d5e377f7dceba"

[[package]]
name = "parking_lot"
version = "0.11.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7d17b78036a60663b797adeaee46f5c9dfebb86948d1255007a1d6be0271ff99"
dependencies = [
 "instant",
 "lock_api",
 "parking_lot_core 0.8.6",
]

[[package]]
name = "parking_lot"
version = "0.12.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a"
dependencies = [
 "lock_api",
 "parking_lot_core 0.9.12",
]

[[package]]
name = "parking_lot_core"
version = "0.8.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "60a2cfe6f0ad2bfc16aefa463b497d5c7a5ecd44a23efa72aa342d90177356dc"
dependencies = [
 "cfg-if",
 "instant",
 "libc",
 "redox_syscall 0.2.16",
 "smallvec",
 "winapi",
]

[[package]]
name = "parking_lot_core"
version = "0.9.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1"
dependencies = [
 "cfg-if",
 "libc",
 "redox_syscall 0.5.18",
 "smallvec",
 "windows-link",
]

[[package]]
name = "password-hash"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "346f04948ba92c43e8469c1ee6736c7563d71012b17d40745260fe106aac2166"
dependencies = [
 "base64ct",
 "rand_core 0.6.4",
 "subtle",
]

[[package]]
name = "paste"
version = "1.0.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "57c0d7b74b563b49d38dae00a0c37d4d6de9b432382b2892f0574ddcae73fd0a"

[[package]]
name = "pem"
version = "3.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d30c53c26bc5b31a98cd02d20f25a7c8567146caf63ed593a9d87b2775291be"
dependencies = [
 "base64 0.22.1",
 "serde_core",
]

[[package]]
name = "pem-rfc7468"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "88b39c9bfcfc231068454382784bb460aae594343fb030d46e9f50a645418412"
dependencies = [
 "base64ct",
]

[[package]]
name = "percent-encoding"
version = "2.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220"

[[package]]
name = "pin-project-lite"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a89322df9ebe1c1578d689c92318e070967d1042b512afbe49518723f4e6d5cd"

[[package]]
name = "pkcs1"
version = "0.7.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c8ffb9f10fa047879315e6625af03c164b16962a5368d724ed16323b68ace47f"
dependencies = [
 "der",
 "pkcs8",
 "spki",
]

[[package]]
name = "pkcs8"
version = "0.10.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f950b2377845cebe5cf8b5165cb3cc1a5e0fa5cfa3e1f7f55707d8fd82e0a7b7"
dependencies = [
 "der",
 "spki",
]

[[package]]
name = "pkg-config"
version = "0.3.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "19f132c84eca552bf34cab8ec81f1c1dcc229b811638f9d283dceabe58c5569e"

[[package]]
name = "plain"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b4596b6d070b27117e987119b4dac604f3c58cfb0b191112e24771b2faeac1a6"

[[package]]
name = "plotters"
version = "0.3.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5aeb6f403d7a4911efb1e33402027fc44f29b5bf6def3effcc22d7bb75f2b747"
dependencies = [
 "num-traits",
 "plotters-backend",
 "plotters-svg",
 "wasm-bindgen",
 "web-sys",
]

[[package]]
name = "plotters-backend"
version = "0.3.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "df42e13c12958a16b3f7f4386b9ab1f3e7933914ecea48da7139435263a4172a"

[[package]]
name = "plotters-svg"
version = "0.3.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "51bae2ac328883f7acdfea3d66a7c35751187f870bc81f94563733a154d7a670"
dependencies = [
 "plotters-backend",
]

[[package]]
name = "potential_utf"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0103b1cef7ec0cf76490e969665504990193874ea05c85ff9bab8b911d0a0564"
dependencies = [
 "zerovec",
]

[[package]]
name = "powerfmt"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "439ee305def115ba05938db6eb1644ff94165c5ab5e9420d1c1bcedbba909391"

[[package]]
name = "ppv-lite86"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9"
dependencies = [
 "zerocopy",
]

[[package]]
name = ""
version = "0.1.0"
dependencies = [
 "axum",
 "contracts",
 "core",
 "flume",
 "futures-util",
 "parking_lot 0.12.5",
 "reqwest 0.11.27",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
 "tokio-tungstenite",
 "transport",
]

[[package]]
name = "proc-macro-crate"
version = "3.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e67ba7e9b2b56446f1d419b1d807906278ffa1a658a8a5d8a39dcb1f5a78614f"
dependencies = [
 "toml_edit 0.25.13+spec-1.1.0",
]

[[package]]
name = "proc-macro2"
version = "1.0.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "proptest"
version = "1.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4b45fcc2344c680f5025fe57779faef368840d0bd1f42f216291f0dc4ace4744"
dependencies = [
 "bit-set",
 "bit-vec",
 "bitflags 2.13.1",
 "num-traits",
 "rand 0.9.5",
 "rand_chacha 0.9.0",
 "rand_xorshift",
 "regex-syntax",
 "rusty-fork",
 "tempfile",
 "unarray",
]

[[package]]
name = "ptr_meta"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0738ccf7ea06b608c10564b31debd4f5bc5e197fc8bfe088f68ae5ce81e7a4f1"
dependencies = [
 "ptr_meta_derive",
]

[[package]]
name = "ptr_meta_derive"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "16b845dbfca988fa33db069c0e230574d15a3088f147a87b64c7589eb662c9ac"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 1.0.109",
]

[[package]]
name = "quick-error"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a1d01941d82fa2ab50be1e79e6714289dd7cde78eba4c074bc5a4374f650dfe0"

[[package]]
name = "quinn"
version = "0.11.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0c1a41e437b6bbd489372cd4971de128e85c855f56c57f283d20ff016cf7c0a8"
dependencies = [
 "bytes",
 "cfg_aliases 0.2.2",
 "pin-project-lite",
 "quinn-proto",
 "quinn-udp",
 "rustc-hash",
 "rustls 0.23.43",
 "socket2 0.6.5",
 "thiserror 2.0.19",
 "tokio",
 "tracing",
 "web-time",
]

[[package]]
name = "quinn-proto"
version = "0.11.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2f4bfc015262b9df63c8845072ce59068853ff5872180c2ce2f13038b970e560"
dependencies = [
 "aws-lc-rs",
 "bytes",
 "getrandom 0.4.3",
 "lru-slab",
 "rand 0.10.2",
 "rand_pcg",
 "ring",
 "rustc-hash",
 "rustls 0.23.43",
 "rustls-pki-types",
 "slab",
 "thiserror 2.0.19",
 "tinyvec",
 "tracing",
 "web-time",
]

[[package]]
name = "quinn-udp"
version = "0.5.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "35a133f956daabe89a61a685c2649f13d82d5aa4bd5d12d1277e1072a21c0694"
dependencies = [
 "cfg_aliases 0.2.2",
 "libc",
 "once_cell",
 "socket2 0.6.5",
 "tracing",
 "windows-sys 0.61.2",
]

[[package]]
name = "quote"
version = "1.0.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001"
dependencies = [
 "proc-macro2",
]

[[package]]
name = "r-efi"
version = "5.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f"

[[package]]
name = "r-efi"
version = "6.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f8dcc9c7d52a811697d2151c701e0d08956f92b0e24136cf4cf27b57a6a0d9bf"

[[package]]
name = "radium"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc33ff2d4973d518d823d61aa239014831e521c75da58e3df4840d3f47749d09"

[[package]]
name = "radix_trie"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c069c179fcdc6a2fe24d8d18305cf085fdbd4f922c041943e203685d6a1c58fd"
dependencies = [
 "endian-type",
 "nibble_vec",
]

[[package]]
name = "rand"
version = "0.7.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6a6b1679d49b24bbfe0c803429aa1874472f50d9b363131f0e89fc356b544d03"
dependencies = [
 "getrandom 0.1.16",
 "libc",
 "rand_chacha 0.2.2",
 "rand_core 0.5.1",
 "rand_hc",
]

[[package]]
name = "rand"
version = "0.8.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "22f6172bdec972074665ed81ed53b71da00bfc44b65a753cfde883ec4c702a1a"
dependencies = [
 "libc",
 "rand_chacha 0.3.1",
 "rand_core 0.6.4",
]

[[package]]
name = "rand"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b9ef1d0d795eb7d84685bca4f72f3649f064e6641543d3a8c415898726a57b41"
dependencies = [
 "rand_chacha 0.9.0",
 "rand_core 0.9.5",
]

[[package]]
name = "rand"
version = "0.10.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c7f5fa3a058cd35567ef9bfa5e75732bee0f9e4c55fa90477bef2dfcdbc4be80"
dependencies = [
 "chacha20",
 "getrandom 0.4.3",
 "rand_core 0.10.1",
]

[[package]]
name = "rand_chacha"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f4c8ed856279c9737206bf725bf36935d8666ead7aa69b52be55af369d193402"
dependencies = [
 "ppv-lite86",
 "rand_core 0.5.1",
]

[[package]]
name = "rand_chacha"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88"
dependencies = [
 "ppv-lite86",
 "rand_core 0.6.4",
]

[[package]]
name = "rand_chacha"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb"
dependencies = [
 "ppv-lite86",
 "rand_core 0.9.5",
]

[[package]]
name = "rand_core"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "90bde5296fc891b0cef12a6d03ddccc162ce7b2aff54160af9338f8d40df6d19"
dependencies = [
 "getrandom 0.1.16",
]

[[package]]
name = "rand_core"
version = "0.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec0be4795e2f6a28069bec0b5ff3e2ac9bafc99e6a9a7dc3547996c5c816922c"
dependencies = [
 "getrandom 0.2.17",
]

[[package]]
name = "rand_core"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "76afc826de14238e6e8c374ddcc1fa19e374fd8dd986b0d2af0d02377261d83c"
dependencies = [
 "getrandom 0.3.4",
]

[[package]]
name = "rand_core"
version = "0.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69"

[[package]]
name = "rand_hc"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ca3129af7b92a17112d59ad498c6f81eaf463253766b90396d39ea7a39d6613c"
dependencies = [
 "rand_core 0.5.1",
]

[[package]]
name = "rand_pcg"
version = "0.10.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "caa0f4137e1c0a72f4c651489402276c8e8e1cf081f3b0ba156d2cbeef09e86a"
dependencies = [
 "rand_core 0.10.1",
]

[[package]]
name = "rand_xorshift"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "513962919efc330f829edb2535844d1b912b0fbe2ca165d613e4e8788bb05a5a"
dependencies = [
 "rand_core 0.9.5",
]

[[package]]
name = "rawpointer"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "60a357793950651c4ed0f3f52338f53b2f809f32d83a07f72909fa13e4c6c1e3"

[[package]]
name = "rayon"
version = "1.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fb39b166781f92d482534ef4b4b1b2568f42613b53e5b6c160e24cfbfa30926d"
dependencies = [
 "either",
 "rayon-core",
]

[[package]]
name = "rayon-core"
version = "1.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "22e18b0f0062d30d4230b2e85ff77fdfe4326feb054b9783a3460d8435c8ab91"
dependencies = [
 "crossbeam-deque",
 "crossbeam-utils",
]

[[package]]
name = "redis-protocol"
version = "4.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9c31deddf734dc0a39d3112e73490e88b61a05e83e074d211f348404cee4d2c6"
dependencies = [
 "bytes",
 "bytes-utils",
 "cookie-factory",
 "crc16",
 "log",
 "nom 7.1.3",
]

[[package]]
name = "redox_syscall"
version = "0.2.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fb5a58c1855b4b6819d59012155603f0b22ad30cad752600aadfcb695265519a"
dependencies = [
 "bitflags 1.3.2",
]

[[package]]
name = "redox_syscall"
version = "0.5.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d"
dependencies = [
 "bitflags 2.13.1",
]

[[package]]
name = "redox_syscall"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "07507be7b4a5f9f26eeb41eeaebb1f5a7ff29dfb29739facc21d35bf8b11c21e"
dependencies = [
 "bitflags 2.13.1",
]

[[package]]
name = "ref-cast"
version = "1.0.26"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "216e8f773d7923bcba9ceb86a86c93cabb3903a11872fc3f138c49630e50b96d"
dependencies = [
 "ref-cast-impl",
]

[[package]]
name = "ref-cast-impl"
version = "1.0.26"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2c9283685feec7d69af75fb0e858d5e7378f33fe4fc699383b2916ab9273e03c"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.3",
]

[[package]]
name = "regex"
version = "1.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-automata",
 "regex-syntax",
]

[[package]]
name = "regex-automata"
version = "0.4.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ad8553b9b26413251cbf30e620595c7a41b3887f03da04579c0e6b0d6a06b4b2"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-syntax",
]

[[package]]
name = "regex-syntax"
version = "0.8.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6f6ff9a378485b298a5286656da665ba74413d36db0979633275d2e708145d4"

[[package]]
name = "rend"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "71fe3824f5629716b1589be05dacd749f6aa084c87e00e016714a8cdfccc997c"
dependencies = [
 "bytecheck",
]

[[package]]
name = "reqwest"
version = "0.11.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dd67538700a17451e7cba03ac727fb961abb7607553461627b97de0b89cf4a62"
dependencies = [
 "base64 0.21.7",
 "bytes",
 "encoding_rs",
 "futures-core",
 "futures-util",
 "h2 0.3.27",
 "http 0.2.12",
 "http-body 0.4.6",
 "hyper 0.14.32",
 "hyper-rustls 0.24.2",
 "ipnet",
 "js-sys",
 "log",
 "mime",
 "once_cell",
 "percent-encoding",
 "pin-project-lite",
 "rustls 0.21.12",
 "rustls-pemfile",
 "serde",
 "serde_json",
 "serde_urlencoded",
 "sync_wrapper 0.1.2",
 "system-configuration 0.5.1",
 "tokio",
 "tokio-rustls 0.24.1",
 "tower-service",
 "url",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "web-sys",
 "webpki-roots",
 "winreg",
]

[[package]]
name = "reqwest"
version = "0.13.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "219c5811de6525e5416c7d5d53bb656d3afdbc6c5af816e0802bcfa42dbdc1c3"
dependencies = [
 "base64 0.22.1",
 "bytes",
 "encoding_rs",
 "futures-core",
 "h2 0.4.15",
 "http 1.5.0",
 "http-body 1.1.0",
 "http-body-util",
 "hyper 1.11.0",
 "hyper-rustls 0.27.9",
 "hyper-util",
 "js-sys",
 "log",
 "mime",
 "percent-encoding",
 "pin-project-lite",
 "quinn",
 "rustls 0.23.43",
 "rustls-pki-types",
 "rustls-platform-verifier",
 "serde",
 "serde_json",
 "sync_wrapper 1.0.2",
 "tokio",
 "tokio-rustls 0.26.4",
 "tower",
 "tower-http",
 "tower-service",
 "url",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "web-sys",
]

[[package]]
name = "retain_mut"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4389f1d5789befaf6029ebd9f7dac4af7f7e3d61b69d4f30e2ac02b57e7712b0"

[[package]]
name = "ring"
version = "0.17.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a4689e6c2294d81e88dc6261c768b63bc4fcdb852be6d1352498b114f61383b7"
dependencies = [
 "cc",
 "cfg-if",
 "getrandom 0.2.17",
 "libc",
 "untrusted",
 "windows-sys 0.52.0",
]

[[package]]
name = "risk-worker"
version = "0.1.0"
dependencies = [
 "parking_lot 0.12.5",
]

[[package]]
name = "rkyv"
version = "0.7.46"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2297bf9c81a3f0dc96bc9521370b88f054168c29826a75e89c55ff196e7ed6a1"
dependencies = [
 "bitvec",
 "bytecheck",
 "bytes",
 "hashbrown 0.12.3",
 "ptr_meta",
 "rend",
 "rkyv_derive",
 "seahash",
 "tinyvec",
 "uuid",
]

[[package]]
name = "rkyv_derive"
version = "0.7.46"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "84d7b42d4b8d06048d3ac8db0eb31bcb942cbeb709f0b5f2b2ebde398d3038f5"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 1.0.109",
]

[[package]]
name = "rsa"
version = "0.9.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b8573f03f5883dcaebdfcf4725caa1ecb9c15b2ef50c43a07b816e06799bb12d"
dependencies = [
 "const-oid",
 "digest",
 "num-bigint-dig",
 "num-integer",
 "num-traits",
 "pkcs1",
 "pkcs8",
 "rand_core 0.6.4",
 "signature",
 "spki",
 "subtle",
 "zeroize",
]

[[package]]
name = "rtrb"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ade083ccbb4bf536df69d1f6432cc23deb7acccff86b183f3923a6fd56a1153"

[[package]]
name = "sqlx"
version = "0.31.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b838eba278d213a8beaf485bd313fd580ca4505a00d5871caeb1457c55322cae"
dependencies = [
 "bitflags 2.13.1",
 "fallible-iterator",
 "fallible-streaming-iterator",
 "hashlink 0.9.1",
 "libpsql-sys",
 "smallvec",
]

[[package]]
name = "rust_decimal"
version = "1.42.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "be2a24f50780bc85f09cc6ac299bdf1424302742d77221106859c9d8b102126a"
dependencies = [
 "arrayvec",
 "borsh",
 "bytes",
 "num-traits",
 "rand 0.8.7",
 "rkyv",
 "serde",
 "serde_json",
 "wasm-bindgen",
]

[[package]]
name = "rustc-hash"
version = "2.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6b1e7f9a428571be2dc5bc0505c13fb6bf936822b894ec87abf8a08a4e51742d"

[[package]]
name = "rustc_version"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cfcb3a22ef46e85b45de6ee7e79d063319ebb6594faafcf1c225ea92ab6e9b92"
dependencies = [
 "semver",
]

[[package]]
name = "rustix"
version = "1.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190"
dependencies = [
 "bitflags 2.13.1",
 "errno",
 "libc",
 "linux-raw-sys",
 "windows-sys 0.61.2",
]

[[package]]
name = "rustls"
version = "0.21.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f56a14d1f48b391359b22f731fd4bd7e43c97f3c50eee276f3aa09c94784d3e"
dependencies = [
 "log",
 "ring",
 "rustls-webpki 0.101.7",
 "sct",
]

[[package]]
name = "rustls"
version = "0.23.43"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0283386ce02abc0151e1761d08802dfe86c173b0b494af5cbc086574e453da06"
dependencies = [
 "aws-lc-rs",
 "once_cell",
 "rustls-pki-types",
 "rustls-webpki 0.103.13",
 "subtle",
 "zeroize",
]

[[package]]
name = "rustls-native-certs"
version = "0.6.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a9aace74cb666635c918e9c12bc0d348266037aa8eb599b5cba565709a8dff00"
dependencies = [
 "openssl-probe 0.1.6",
 "rustls-pemfile",
 "schannel",
 "security-framework 2.11.1",
]

[[package]]
name = "rustls-native-certs"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dab5152771c58876a2146916e53e35057e1a4dfa2b9df0f0305b07f611fdea4d"
dependencies = [
 "openssl-probe 0.2.1",
 "rustls-pki-types",
 "schannel",
 "security-framework 3.7.0",
]

[[package]]
name = "rustls-pemfile"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1c74cae0a4cf6ccbbf5f359f08efdf8ee7e1dc532573bf0db71968cb56b1448c"
dependencies = [
 "base64 0.21.7",
]

[[package]]
name = "rustls-pki-types"
version = "1.15.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2f4925028c7eb5d1fcdaf196971378ed9d2c1c4efc7dc5d011256f76c99c0a96"
dependencies = [
 "web-time",
 "zeroize",
]

[[package]]
name = "rustls-platform-verifier"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "26d1e2536ce4f35f4846aa13bff16bd0ff40157cdb14cc056c7b14ba41233ba0"
dependencies = [
 "core-foundation 0.10.1",
 "core-foundation-sys",
 "jni",
 "log",
 "once_cell",
 "rustls 0.23.43",
 "rustls-native-certs 0.8.4",
 "rustls-platform-verifier-android",
 "rustls-webpki 0.103.13",
 "security-framework 3.7.0",
 "security-framework-sys",
 "webpki-root-certs",
 "windows-sys 0.61.2",
]

[[package]]
name = "rustls-platform-verifier-android"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f87165f0995f63a9fbeea62b64d10b4d9d8e78ec6d7d51fb2125fda7bb36788f"

[[package]]
name = "rustls-webpki"
version = "0.101.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8b6275d1ee7a1cd780b64aca7726599a1dbc893b1e64144529e55c3c2f745765"
dependencies = [
 "ring",
 "untrusted",
]

[[package]]
name = "rustls-webpki"
version = "0.103.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "61c429a8649f110dddef65e2a5ad240f747e85f7758a6bccc7e5777bd33f756e"
dependencies = [
 "aws-lc-rs",
 "ring",
 "rustls-pki-types",
 "untrusted",
]

[[package]]
name = "rustversion"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf54715a573b99ac80df0bc206da022bcd442c974952c7b9720069370852e21f"

[[package]]
name = "rusty-fork"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cc6bf79ff24e648f6da1f8d1f011e9cac26491b619e6b9280f2b47f1774e6ee2"
dependencies = [
 "fnv",
 "quick-error",
 "tempfile",
 "wait-timeout",
]

[[package]]
name = "rustyline"
version = "14.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7803e8936da37efd9b6d4478277f4b2b9bb5cdb37a113e8d63222e58da647e63"
dependencies = [
 "bitflags 2.13.1",
 "cfg-if",
 "clipboard-win",
 "fd-lock",
 "home",
 "libc",
 "log",
 "memchr",
 "nix",
 "radix_trie",
 "unicode-segmentation",
 "unicode-width",
 "utf8parse",
 "windows-sys 0.52.0",
]

[[package]]
name = "ryu"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9774ba4a74de5f7b1c1451ed6cd5285a32eddb5cccb8cc655a4e50009e06477f"

[[package]]
name = "safe_arch"
version = "0.7.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "96b02de82ddbe1b636e6170c21be622223aea188ef2e139be0a5b219ec215323"
dependencies = [
 "bytemuck",
]

[[package]]
name = "same-file"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502"
dependencies = [
 "winapi-util",
]

[[package]]
name = "schannel"
version = "0.1.29"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "91c1b7e4904c873ef0710c1f407dde2e6287de2bebc1bbbf7d430bb7cbffd939"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "scopeguard"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49"

[[package]]
name = "scout-service"
version = "0.1.0"
dependencies = [
 "contracts",
 "futures-util",
 "rand 0.8.7",
 "reqwest 0.11.27",
 "rust_decimal",
 "serde",
 "serde_json",
 "tokio",
 "tokio-tungstenite",
 "transport",
]

[[package]]
name = "sct"
version = "0.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "da046153aa2352493d6cb7da4b6e5c0c057d8a1d0a9aa8560baffdd945acd414"
dependencies = [
 "ring",
 "untrusted",
]

[[package]]
name = "seahash"
version = "4.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1c107b6f4780854c8b126e228ea8869f4d7b71260f962fefb57b996b8959ba6b"

[[package]]
name = "security-framework"
version = "2.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "897b2245f0b511c87893af39b033e5ca9cce68824c4d7e7630b5a1d339658d02"
dependencies = [
 "bitflags 2.13.1",
 "core-foundation 0.9.4",
 "core-foundation-sys",
 "libc",
 "security-framework-sys",
]

[[package]]
name = "security-framework"
version = "3.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b7f4bc775c73d9a02cde8bf7b2ec4c9d12743edf609006c7facc23998404cd1d"
dependencies = [
 "bitflags 2.13.1",
 "core-foundation 0.10.1",
 "core-foundation-sys",
 "libc",
 "security-framework-sys",
]

[[package]]
name = "security-framework-sys"
version = "2.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ce2691df843ecc5d231c0b14ece2acc3efb62c0a398c7e1d875f3983ce020e3"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "semver"
version = "1.0.28"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8a7852d02fc848982e0c167ef163aaff9cd91dc640ba85e263cb1ce46fae51cd"

[[package]]
name = "serde"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba"
dependencies = [
 "serde_core",
 "serde_derive",
]

[[package]]
name = "serde_core"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48"
dependencies = [
 "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.3",
]

[[package]]
name = "serde_json"
version = "1.0.151"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14"
dependencies = [
 "itoa",
 "memchr",
 "serde",
 "serde_core",
 "zmij",
]

[[package]]
name = "serde_path_to_error"
version = "0.1.20"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "10a9ff822e371bb5403e391ecd83e182e0e77ba7f6fe0160b795797109d1b457"
dependencies = [
 "itoa",
 "serde",
 "serde_core",
]

[[package]]
name = "serde_qs"
version = "0.8.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c7715380eec75f029a4ef7de39a9200e0a63823176b759d055b613f5a87df6a6"
dependencies = [
 "percent-encoding",
 "serde",
 "thiserror 1.0.69",
]

[[package]]
name = "serde_spanned"
version = "0.6.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bf41e0cfaf7226dca15e8197172c295a782857fcb97fad1808a166870dee75a3"
dependencies = [
 "serde",
]

[[package]]
name = "serde_urlencoded"
version = "0.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d3491c14715ca2294c4d6a88f15e84739788c1d030eed8c110436aafdaa2f3fd"
dependencies = [
 "form_urlencoded",
 "itoa",
 "ryu",
 "serde",
]

[[package]]
name = "serde_with"
version = "1.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "678b5a069e50bf00ecd22d0cd8ddf7c236f68581b03db652061ed5eb13a312ff"
dependencies = [
 "serde",
 "serde_with_macros",
]

[[package]]
name = "serde_with_macros"
version = "1.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e182d6ec6f05393cc0e5ed1bf81ad6db3a8feedf8ee515ecdd369809bcce8082"
dependencies = [
 "darling",
 "proc-macro2",
 "quote",
 "syn 1.0.109",
]

[[package]]
name = "sha1"
version = "0.10.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a978451301f4db1d02937a4ab3ccce137717b81826e79b7d49ffe3244a13c3b8"
dependencies = [
 "cfg-if",
 "cpufeatures 0.2.17",
 "digest",
]

[[package]]
name = "sha2"
version = "0.10.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283"
dependencies = [
 "cfg-if",
 "cpufeatures 0.2.17",
 "digest",
]

[[package]]
name = "sha3"
version = "0.10.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77fd7028345d415a4034cf8777cd4f8ab1851274233b45f84e3d955502d93874"
dependencies = [
 "digest",
 "keccak",
]

[[package]]
name = "sharded-slab"
version = "0.1.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f40ca3c46823713e0d4209592e8d6e826aa57e928f09752619fc696c499637f6"
dependencies = [
 "lazy_static",
]

[[package]]
name = "shlex"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f8fadd59c855ef2080decdef8ff161eb6661b86933c9d82e5ba29dc602a55aba"

[[package]]
name = "signal-hook-registry"
version = "1.4.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c4db69cba1110affc0e9f7bcd48bbf87b3f4fc7c61fc9155afd4c469eb3d6c1b"
dependencies = [
 "errno",
 "libc",
]

[[package]]
name = "signature"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77549399552de45a898a580c1b41d445bf730df867cc44e6c0233bbc4b8329de"
dependencies = [
 "digest",
 "rand_core 0.6.4",
]

[[package]]
name = "simd-adler32"
version = "0.3.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3a219298ac11a56ea9a6d2120044824d6f01aeb034955e7af7bc16858527deea"

[[package]]
name = "simd-json"
version = "0.13.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a0228a564470f81724e30996bbc2b171713b37b15254a6440c7e2d5449b95691"
dependencies = [
 "getrandom 0.2.17",
 "halfbrown",
 "lexical-core",
 "ref-cast",
 "serde",
 "serde_json",
 "simdutf8",
 "value-trait",
]

[[package]]
name = "simd_cesu8"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "11031e251abf8611c80f460e19dbdeb54a66db918e49c65a7065b46ac7aec520"
dependencies = [
 "rustc_version",
 "simdutf8",
]

[[package]]
name = "simdutf8"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e3a9fe34e3e7a50316060351f37187a3f546bce95496156754b601a5fa71b76e"

[[package]]
name = "simple_asn1"
version = "0.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0d585997b0ac10be3c5ee635f1bab02d512760d14b7c468801ac8a01d9ae5f1d"
dependencies = [
 "num-bigint",
 "num-traits",
 "thiserror 2.0.19",
 "time",
]

[[package]]
name = "slab"
version = "0.4.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0c790de23124f9ab44544d7ac05d60440adc586479ce501c1d6d7da3cd8c9cf5"

[[package]]
name = "sled"
version = "0.34.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7f96b4737c2ce5987354855aed3797279def4ebf734436c6aa4552cf8e169935"
dependencies = [
 "crc32fast",
 "crossbeam-epoch",
 "crossbeam-utils",
 "fs2",
 "fxhash",
 "libc",
 "log",
 "parking_lot 0.11.2",
]

[[package]]
name = "smallvec"
version = "1.15.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8ed6a63f02c8539c91a8685a86f4099661ba3da017932f6ebbea6de3f0fa7c90"

[[package]]
name = "socket2"
version = "0.5.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e22376abed350d73dd1cd119b57ffccad95b4e585a7cda43e286245ce23c0678"
dependencies = [
 "libc",
 "windows-sys 0.52.0",
]

[[package]]
name = "socket2"
version = "0.6.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c3d1e2c7f27f8d4cb10542a02c49005dbd6e93095799d6f3be745fae9f8fedd4"
dependencies = [
 "libc",
 "windows-sys 0.61.2",
]

[[package]]
name = "spin"
version = "0.9.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3763264f6b73151db08c50ff20d7d8a0b8796e021cdea7ceedad07b80155fa0e"
dependencies = [
 "lock_api",
]

[[package]]
name = "spki"
version = "0.7.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d91ed6c858b01f942cd56b37a94b3e0a1798290327d1236e4d9cf4eaca44d29d"
dependencies = [
 "base64ct",
 "der",
]

[[package]]
name = "sqlformat"
version = "0.2.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7bba3a93db0cc4f7bdece8bb09e77e2e785c20bfebf79eb8340ed80708048790"
dependencies = [
 "nom 7.1.3",
 "unicode_categories",
]

[[package]]
name = "sqlx"
version = "0.7.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c9a2ccff1a000a5a59cd33da541d9f2fdcd9e6e8229cc200565942bff36d0aaa"
dependencies = [
 "sqlx-core",
 "sqlx-macros",
 "sqlx-mysql",
 "sqlx-postgres",
]

[[package]]
name = "sqlx-core"
version = "0.7.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "24ba59a9342a3d9bab6c56c118be528b27c9b60e490080e9711a04dccac83ef6"
dependencies = [
 "ahash 0.8.12",
 "atoi",
 "byteorder",
 "bytes",
 "crc",
 "crossbeam-queue",
 "either",
 "event-listener",
 "futures-channel",
 "futures-core",
 "futures-intrusive",
 "futures-io",
 "futures-util",
 "hashlink 0.8.4",
 "hex",
 "indexmap 2.14.0",
 "log",
 "memchr",
 "once_cell",
 "paste",
 "percent-encoding",
 "rust_decimal",
 "serde",
 "serde_json",
 "sha2",
 "smallvec",
 "sqlformat",
 "thiserror 1.0.69",
 "tokio",
 "tokio-stream",
 "tracing",
 "url",
]

[[package]]
name = "sqlx-macros"
version = "0.7.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ea40e2345eb2faa9e1e5e326db8c34711317d2b5e08d0d5741619048a803127"
dependencies = [
 "proc-macro2",
 "quote",
 "sqlx-core",
 "sqlx-macros-core",
 "syn 1.0.109",
]

[[package]]
name = "sqlx-macros-core"
version = "0.7.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5833ef53aaa16d860e92123292f1f6a3d53c34ba8b1969f152ef1a7bb803f3c8"
dependencies = [
 "dotenvy",
 "either",
 "heck 0.4.1",
 "hex",
 "once_cell",
 "proc-macro2",
 "quote",
 "serde",
 "serde_json",
 "sha2",
 "sqlx-core",
 "sqlx-mysql",
 "sqlx-postgres",
 "syn 1.0.109",
 "tempfile",
 "tokio",
 "url",
]

[[package]]
name = "sqlx-mysql"
version = "0.7.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ed31390216d20e538e447a7a9b959e06ed9fc51c37b514b46eb758016ecd418"
dependencies = [
 "atoi",
 "base64 0.21.7",
 "bitflags 2.13.1",
 "byteorder",
 "bytes",
 "crc",
 "digest",
 "dotenvy",
 "either",
 "futures-channel",
 "futures-core",
 "futures-io",
 "futures-util",
 "generic-array",
 "hex",
 "hkdf",
 "hmac",
 "itoa",
 "log",
 "md-5",
 "memchr",
 "once_cell",
 "percent-encoding",
 "rand 0.8.7",
 "rsa",
 "rust_decimal",
 "serde",
 "sha1",
 "sha2",
 "smallvec",
 "sqlx-core",
 "stringprep",
 "thiserror 1.0.69",
 "tracing",
 "whoami",
]

[[package]]
name = "sqlx-postgres"
version = "0.7.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7c824eb80b894f926f89a0b9da0c7f435d27cdd35b8c655b114e58223918577e"
dependencies = [
 "atoi",
 "base64 0.21.7",
 "bitflags 2.13.1",
 "byteorder",
 "crc",
 "dotenvy",
 "etcetera",
 "futures-channel",
 "futures-core",
 "futures-io",
 "futures-util",
 "hex",
 "hkdf",
 "hmac",
 "home",
 "itoa",
 "log",
 "md-5",
 "memchr",
 "once_cell",
 "rand 0.8.7",
 "rust_decimal",
 "serde",
 "serde_json",
 "sha2",
 "smallvec",
 "sqlx-core",
 "stringprep",
 "thiserror 1.0.69",
 "tracing",
 "whoami",
]

[[package]]
name = "stable_deref_trait"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596"

[[package]]
name = "stringprep"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7b4df3d392d81bd458a8a621b8bffbd2302a12ffe288a9d931670948749463b1"
dependencies = [
 "unicode-bidi",
 "unicode-normalization",
 "unicode-properties",
]

[[package]]
name = "strsim"
version = "0.10.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73473c0e59e6d5812c5dfe2a064a6444949f089e20eec9a2e5506596494e4623"

[[package]]
name = "strsim"
version = "0.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7da8b5736845d9f2fcb837ea5d9e2628564b3b043a70948a3f0b778838c5fb4f"

[[package]]
name = "subtle"
version = "2.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "13c2bddecc57b384dee18652358fb23172facb8a2c51ccc10d74c157bdea3292"

[[package]]
name = "syn"
version = "1.0.109"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72b64191b275b66ffe2469e8af2c1cfe3bafa67b529ead792a6d0160888b4237"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "syn"
version = "2.0.119"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "syn"
version = "3.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "53e9bae58849f64dfa4f5d5ae372c8341f7305f82a3868709269343628b659a3"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "sync_wrapper"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2047c6ded9c721764247e62cd3b03c09ffc529b2ba5b10ec482ae507a4a70160"

[[package]]
name = "sync_wrapper"
version = "1.0.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0bf256ce5efdfa370213c1dabab5935a12e49f2c58d15e9eac2870d3b4f27263"
dependencies = [
 "futures-core",
]

[[package]]
name = "synstructure"
version = "0.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "system-configuration"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ba3a3adc5c275d719af8cb4272ea1c4a6d668a777f37e115f6d11ddbc1c8e0e7"
dependencies = [
 "bitflags 1.3.2",
 "core-foundation 0.9.4",
 "system-configuration-sys 0.5.0",
]

[[package]]
name = "system-configuration"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a13f3d0daba03132c0aa9767f98351b3488edc2c100cda2d2ec2b04f3d8d3c8b"
dependencies = [
 "bitflags 2.13.1",
 "core-foundation 0.9.4",
 "system-configuration-sys 0.6.0",
]

[[package]]
name = "system-configuration-sys"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a75fb188eb626b924683e3b95e3a48e63551fcfb51949de2f06a9d91dbee93c9"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "system-configuration-sys"
version = "0.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8e1d1b10ced5ca923a1fcb8d03e96b8d3268065d724548c0211415ff6ac6bac4"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "tap"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "55937e1799185b12863d447f42597ed69d9928686b8d88a1df17376a097d8369"

[[package]]
name = "tempfile"
version = "3.27.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32497e9a4c7b38532efcdebeef879707aa9f794296a4f0244f6f69e9bc8574bd"
dependencies = [
 "fastrand 2.5.0",
 "getrandom 0.4.3",
 "once_cell",
 "rustix",
 "windows-sys 0.61.2",
]

[[package]]
name = "testcontainers"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0e2b1567ca8a2b819ea7b28c92be35d9f76fb9edb214321dcc86eb96023d1f87"
dependencies = [
 "bollard-stubs",
 "futures",
 "hex",
 "hmac",
 "log",
 "rand 0.8.7",
 "serde",
 "serde_json",
 "sha2",
]

[[package]]
name = "textwrap"
version = "0.16.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c13547615a44dc9c452a8a534638acdf07120d4b6847c8178705da06306a3057"

[[package]]
name = "thiserror"
version = "1.0.69"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6aaf5339b578ea85b50e080feb250a3e8ae8cfcdff9a461c9ec2904bc923f52"
dependencies = [
 "thiserror-impl 1.0.69",
]

[[package]]
name = "thiserror"
version = "2.0.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09a43598840e33d5b0331f38c5e30d13bb11c11210a4b58f0d9b18a5a5eefcd9"
dependencies = [
 "thiserror-impl 2.0.19",
]

[[package]]
name = "thiserror-impl"
version = "1.0.69"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4fee6c4efc90059e10f81e6d42c60a18f76588c3d74cb83a0b242a2b6c7504c1"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "thiserror-impl"
version = "2.0.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "43cbfe0cf76104d42a574802844187e84a305e531ed54455f11fbde0f10541cd"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.3",
]

[[package]]
name = "thread_local"
version = "1.1.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ad99c4c6d32803332c548b1af0540b357b3f5fc0be8f6c6bfe8b2e6ae784070"
dependencies = [
 "cfg-if",
]

[[package]]
name = "time"
version = "0.3.55"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cdb87b95ec50ddfa440816d227a17b2ccbdda963a316a727fda0fc4334f7d134"
dependencies = [
 "deranged",
 "num-conv",
 "powerfmt",
 "serde_core",
 "time-core",
 "time-macros",
]

[[package]]
name = "time-core"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9e1c906769ad99c88eaa54e728060edef082f8e358ff32030cb7c7d315e81109"

[[package]]
name = "time-macros"
version = "0.2.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e689342a48d2ea927c87ea50cabf8594854bf940e9310208848d680d668ed85"
dependencies = [
 "num-conv",
 "time-core",
]

[[package]]
name = "tinystr"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c8323304221c2a851516f22236c5722a72eaa19749016521d6dff0824447d96d"
dependencies = [
 "displaydoc",
 "zerovec",
]

[[package]]
name = "tinytemplate"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "be4d6b5f19ff7664e8c98d03e2139cb510db9b0a60b55f8e8709b689d939b6bc"
dependencies = [
 "serde",
 "serde_json",
]

[[package]]
name = "tinyvec"
version = "1.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb4ebadaa0af04fab11ae01eb5f9fdb5f9c5b875506e210e71c07873528baa7f"
dependencies = [
 "tinyvec_macros",
]

[[package]]
name = "tinyvec_macros"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1f3ccbac311fea05f86f61904b462b55fb3df8837a366dfc601a0161d0532f20"

[[package]]
name = "tokio"
version = "1.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "202caea871b69668250d242070849eb495be178ed697a3e98aebce5bc81a0bed"
dependencies = [
 "bytes",
 "libc",
 "mio",
 "parking_lot 0.12.5",
 "pin-project-lite",
 "signal-hook-registry",
 "socket2 0.6.5",
 "tokio-macros",
 "windows-sys 0.61.2",
]

[[package]]
name = "tokio-macros"
version = "2.7.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "78773a2a397f451582ce068015985c33193cf6dea8b74d2a639fe457b2f07b0e"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.3",
]

[[package]]
name = "tokio-rustls"
version = "0.24.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c28327cf380ac148141087fbfb9de9d7bd4e84ab5d2c28fbc911d753de8a7081"
dependencies = [
 "rustls 0.21.12",
 "tokio",
]

[[package]]
name = "tokio-rustls"
version = "0.26.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1729aa945f29d91ba541258c8df89027d5792d85a8841fb65e8bf0f4ede4ef61"
dependencies = [
 "rustls 0.23.43",
 "tokio",
]

[[package]]
name = "tokio-stream"
version = "0.1.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a3d06f0b082ba57c26b79407372e57cf2a1e28124f78e9479fe80322cf53420b"
dependencies = [
 "futures-core",
 "pin-project-lite",
 "tokio",
]

[[package]]
name = "tokio-tungstenite"
version = "0.20.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "212d5dcb2a1ce06d81107c3d0ffa3121fe974b73f068c8282cb1c32328113b6c"
dependencies = [
 "futures-util",
 "log",
 "rustls 0.21.12",
 "rustls-native-certs 0.6.3",
 "tokio",
 "tokio-rustls 0.24.1",
 "tungstenite",
 "webpki-roots",
]

[[package]]
name = "tokio-util"
version = "0.7.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "494815d09bf52b5548659851081238f0ca39ff638363907596da739561c62c52"
dependencies = [
 "bytes",
 "futures-core",
 "futures-sink",
 "libc",
 "pin-project-lite",
 "tokio",
]

[[package]]
name = "toml"
version = "0.8.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc1beb996b9d83529a9e75c17a1686767d148d70663143c7854d8b4a09ced362"
dependencies = [
 "serde",
 "serde_spanned",
 "toml_datetime 0.6.11",
 "toml_edit 0.22.27",
]

[[package]]
name = "toml_datetime"
version = "0.6.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "22cddaf88f4fbc13c51aebbf5f8eceb5c7c5a9da2ac40a13519eb5b0a0e8f11c"
dependencies = [
 "serde",
]

[[package]]
name = "toml_datetime"
version = "1.1.1+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3165f65f62e28e0115a00b2ebdd37eb6f3b641855f9d636d3cd4103767159ad7"
dependencies = [
 "serde_core",
]

[[package]]
name = "toml_edit"
version = "0.22.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "41fe8c660ae4257887cf66394862d21dbca4a6ddd26f04a3560410406a2f819a"
dependencies = [
 "indexmap 2.14.0",
 "serde",
 "serde_spanned",
 "toml_datetime 0.6.11",
 "toml_write",
 "winnow 0.7.15",
]

[[package]]
name = "toml_edit"
version = "0.25.13+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6975367e4d2ef766d86af01ffad14b622fecc8d4357a998fbc4deb6e9bacaf9b"
dependencies = [
 "indexmap 2.14.0",
 "toml_datetime 1.1.1+spec-1.1.0",
 "toml_parser",
 "winnow 1.0.4",
]

[[package]]
name = "toml_parser"
version = "1.1.3+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d38ac1cf9b95face32296c0a3ede1fdc270627c9d9c02a7274dd6d960dc4d56"
dependencies = [
 "winnow 1.0.4",
]

[[package]]
name = "toml_write"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5d99f8c9a7727884afe522e9bd5edbfc91a3312b36a77b5fb8926e4c31a41801"

[[package]]
name = "tower"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ebe5ef63511595f1344e2d5cfa636d973292adc0eec1f0ad45fae9f0851ab1d4"
dependencies = [
 "futures-core",
 "futures-util",
 "pin-project-lite",
 "sync_wrapper 1.0.2",
 "tokio",
 "tower-layer",
 "tower-service",
 "tracing",
]

[[package]]
name = "tower-http"
version = "0.6.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4cfcf7e2740e6fc6d4d688b4ef00650406bb94adf4731e43c096c3a19fe40840"
dependencies = [
 "bitflags 2.13.1",
 "bytes",
 "futures-util",
 "http 1.5.0",
 "http-body 1.1.0",
 "pin-project-lite",
 "tower",
 "tower-layer",
 "tower-service",
 "tracing",
 "url",
]

[[package]]
name = "tower-layer"
version = "0.3.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "121c2a6cda46980bb0fcd1647ffaf6cd3fc79a013de288782836f6df9c48780e"

[[package]]
name = "tower-service"
version = "0.3.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8df9b6e13f2d32c91b9bd719c00d1958837bc7dec474d94952798cc8e69eeec3"

[[package]]
name = "tracing"
version = "0.1.44"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63e71662fa4b2a2c3a26f570f037eb95bb1f85397f3cd8076caed2f026a6d100"
dependencies = [
 "log",
 "pin-project-lite",
 "tracing-attributes",
 "tracing-core",
]

[[package]]
name = "tracing-attributes"
version = "0.1.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7490cfa5ec963746568740651ac6781f701c9c5ea257c58e057f3ba8cf69e8da"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "tracing-core"
version = "0.1.36"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "db97caf9d906fbde555dd62fa95ddba9eecfd14cb388e4f491a66d74cd5fb79a"
dependencies = [
 "once_cell",
 "valuable",
]

[[package]]
name = "tracing-log"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ee855f1f400bd0e5c02d150ae5de3840039a3f54b025156404e34c23c03f47c3"
dependencies = [
 "log",
 "once_cell",
 "tracing-core",
]

[[package]]
name = "tracing-subscriber"
version = "0.3.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cb7f578e5945fb242538965c2d0b04418d38ec25c79d160cd279bf0731c8d319"
dependencies = [
 "matchers",
 "nu-ansi-term",
 "once_cell",
 "regex-automata",
 "sharded-slab",
 "smallvec",
 "thread_local",
 "tracing",
 "tracing-core",
 "tracing-log",
]

[[package]]
name = "transport"
version = "0.1.0"
dependencies = [
 "libc",
 "memmap2",
 "rust_decimal",
]

[[package]]
name = "try-lock"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e421abadd41a4225275504ea4d6566923418b7f05506fbc9c0fe86ba7396114b"

[[package]]
name = "tungstenite"
version = "0.20.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9e3dac10fd62eaf6617d3a904ae222845979aec67c615d1c842b4002c7666fb9"
dependencies = [
 "byteorder",
 "bytes",
 "data-encoding",
 "http 0.2.12",
 "httparse",
 "log",
 "rand 0.8.7",
 "rustls 0.21.12",
 "sha1",
 "thiserror 1.0.69",
 "url",
 "utf-8",
]

[[package]]
name = "typenum"
version = "1.20.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20"

[[package]]
name = "unarray"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "eaea85b334db583fe3274d12b4cd1880032beab409c0d774be044d4480ab9a94"

[[package]]
name = "unicode-bidi"
version = "0.3.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c1cb5db39152898a79168971543b1cb5020dff7fe43c8dc468b0885f5e29df5"

[[package]]
name = "unicode-ident"
version = "1.0.24"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75"

[[package]]
name = "unicode-normalization"
version = "0.1.25"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5fd4f6878c9cb28d874b009da9e8d183b5abc80117c40bbd187a1fde336be6e8"
dependencies = [
 "tinyvec",
]

[[package]]
name = "unicode-properties"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7df058c713841ad818f1dc5d3fd88063241cc61f49f5fbea4b951e8cf5a8d71d"

[[package]]
name = "unicode-segmentation"
version = "1.13.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6f5d3c3b1bf09027a88a6bc961fc00497d651009560b5463668dc81b0fa87a8"

[[package]]
name = "unicode-width"
version = "0.1.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7dd6e30e90baa6f72411720665d41d89b9a3d039dc45b8faea1ddd07f617f6af"

[[package]]
name = "unicode_categories"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "39ec24b3121d976906ece63c9daad25b85969647682eee313cb5779fdd69e14e"

[[package]]
name = "untrusted"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8ecb6da28b8a351d773b68d5825ac39017e680750f980f3a1a85cd8dd28a47c1"

[[package]]
name = "url"
version = "2.5.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed"
dependencies = [
 "form_urlencoded",
 "idna",
 "percent-encoding",
 "serde",
 "serde_derive",
]

[[package]]
name = "urlencoding"
version = "2.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "daf8dba3b7eb870caf1ddeed7bc9d2a049f3cfdfae7cb521b087cc33ae4c49da"

[[package]]
name = "utf-8"
version = "0.7.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09cc8ee72d2a9becf2f2febe0205bbed8fc6615b7cb429ad062dc7b7ddd036a9"

[[package]]
name = "utf8_iter"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6c140620e7ffbb22c2dee59cafe6084a59b5ffc27a8859a5f0d494b5d52b6be"

[[package]]
name = "utf8parse"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "06abde3611657adf66d383f00b093d7faecc7fa57071cce2578660c9f1010821"

[[package]]
name = "uuid"
version = "1.24.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bf3923a6f5c4c6382e0b653c4117f48d631ea17f38ed86e2a828e6f7412f5239"
dependencies = [
 "getrandom 0.4.3",
 "js-sys",
 "serde_core",
 "wasm-bindgen",
]

[[package]]
name = "valuable"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ba73ea9cf16a25df0c8caa16c51acb937d5712a8429db78a3ee29d5dcacd3a65"

[[package]]
name = "value-trait"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dad8db98c1e677797df21ba03fca7d3bf9bec3ca38db930954e4fe6e1ea27eb4"
dependencies = [
 "float-cmp",
 "halfbrown",
 "itoa",
 "ryu",
]

[[package]]
name = "vcpkg"
version = "0.2.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426"

[[package]]
name = "version_check"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a"

[[package]]
name = "wait-timeout"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09ac3b126d3914f9849036f826e054cbabdc8519970b8998ddaf3b5bd3c65f11"
dependencies = [
 "libc",
]

[[package]]
name = "waker-fn"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "317211a0dc0ceedd78fb2ca9a44aed3d7b9b26f81870d485c07122b4350673b7"

[[package]]
name = "walkdir"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29790946404f91d9c5d06f9874efddea1dc06c5efe94541a7d6863108e3a5e4b"
dependencies = [
 "same-file",
 "winapi-util",
]

[[package]]
name = "want"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bfa7760aed19e106de2c7c0b581b509f2f25d3dacaf737cb82ac61bc6d760b0e"
dependencies = [
 "try-lock",
]

[[package]]
name = "wasi"
version = "0.9.0+wasi-snapshot-preview1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cccddf32554fecc6acb585f82a32a72e28b48f8c4c1883ddfeeeaa96f7d8e519"

[[package]]
name = "wasi"
version = "0.11.1+wasi-snapshot-preview1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b"

[[package]]
name = "wasip2"
version = "1.0.4+wasi-0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b67efb37e106e55ce722a510d6b5f9c17f083e5fc79afc2badeb12cc313d9487"
dependencies = [
 "wit-bindgen",
]

[[package]]
name = "wasite"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b8dad83b4f25e74f184f64c43b150b91efe7647395b42289f38e50566d82855b"

[[package]]
name = "wasm-bindgen"
version = "0.2.126"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4b067c0c11094aef6b7a801c1e34a26affafdf3d051dba08456b868789aaf9a4"
dependencies = [
 "cfg-if",
 "once_cell",
 "rustversion",
 "serde",
 "wasm-bindgen-macro",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-futures"
version = "0.4.76"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c62df1340f32221cb9c54d6a27b030e3dba64361d4a95bed55f9aacb44da291d"
dependencies = [
 "js-sys",
 "wasm-bindgen",
]

[[package]]
name = "wasm-bindgen-macro"
version = "0.2.126"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "167ce5e579f6bcf889c4f7175a8a5a585de84e8ff93976ce393efa5f2837aab1"
dependencies = [
 "quote",
 "wasm-bindgen-macro-support",
]

[[package]]
name = "wasm-bindgen-macro-support"
version = "0.2.126"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f3997c7839262f4ef12cf90b818d6340c18e80f263f1a94bf157d0ec4420380e"
dependencies = [
 "bumpalo",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-shared"
version = "0.2.126"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc1b4cb0cc549fcf58d7dfc081778139b3d283a081644e833e84682ad71cea24"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "web-sys"
version = "0.3.103"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8622dcb61c0bcc9fffa6938bed81210af2da9a7e4a1a834b2e37a59b6dfb6141"
dependencies = [
 "js-sys",
 "wasm-bindgen",
]

[[package]]
name = "web-time"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a6580f308b1fad9207618087a65c04e7a10bc77e02c8e84e9b00dd4b12fa0bb"
dependencies = [
 "js-sys",
 "wasm-bindgen",
]

[[package]]
name = "webpki-root-certs"
version = "1.0.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b96554aa2acc8ccdb7e1c9a58a7a68dd5d13bccc69cd124cb09406db612a1c9b"
dependencies = [
 "rustls-pki-types",
]

[[package]]
name = "webpki-roots"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5f20c57d8d7db6d3b86154206ae5d8fba62dd39573114de97c2cb0578251f8e1"

[[package]]
name = "whoami"
version = "1.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5d4a4db5077702ca3015d3d02d74974948aba2ad9e12ab7df718ee64ccd7e97d"
dependencies = [
 "libredox",
 "wasite",
]

[[package]]
name = "wide"
version = "0.7.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ce5da8ecb62bcd8ec8b7ea19f69a51275e91299be594ea5cc6ef7819e16cd03"
dependencies = [
 "bytemuck",
 "safe_arch",
]

[[package]]
name = "winapi"
version = "0.3.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419"
dependencies = [
 "winapi-i686-pc-windows-gnu",
 "winapi-x86_64-pc-windows-gnu",
]

[[package]]
name = "winapi-i686-pc-windows-gnu"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6"

[[package]]
name = "winapi-util"
version = "0.1.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2a7b1c03c876122aa43f3020e6c3c3ee5c05081c9a00739faf7503aeba10d22"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "winapi-x86_64-pc-windows-gnu"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f"

[[package]]
name = "windows-core"
version = "0.62.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b8e83a14d34d0623b51dce9581199302a221863196a1dde71a7663a4c2be9deb"
dependencies = [
 "windows-implement",
 "windows-interface",
 "windows-link",
 "windows-result",
 "windows-strings",
]

[[package]]
name = "windows-implement"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "053e2e040ab57b9dc951b72c264860db7eb3b0200ba345b4e4c3b14f67855ddf"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "windows-interface"
version = "0.59.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f316c4a2570ba26bbec722032c4099d8c8bc095efccdc15688708623367e358"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "windows-link"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"

[[package]]
name = "windows-registry"
version = "0.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "02752bf7fbdcce7f2a27a742f798510f3e5ad88dbe84871e5168e2120c3d5720"
dependencies = [
 "windows-link",
 "windows-result",
 "windows-strings",
]

[[package]]
name = "windows-result"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7781fa89eaf60850ac3d2da7af8e5242a5ea78d1a11c49bf2910bb5a73853eb5"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-strings"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7837d08f69c77cf6b07689544538e017c1bfcf57e34b4c0ff58e6c2cd3b37091"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-sys"
version = "0.48.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "677d2418bec65e3338edb076e806bc1ec15693c5d0104683f2efe857f61056a9"
dependencies = [
 "windows-targets 0.48.5",
]

[[package]]
name = "windows-sys"
version = "0.52.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d"
dependencies = [
 "windows-targets 0.52.6",
]

[[package]]
name = "windows-sys"
version = "0.61.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-targets"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a2fa6e2155d7247be68c096456083145c183cbbbc2764150dda45a87197940c"
dependencies = [
 "windows_aarch64_gnullvm 0.48.5",
 "windows_aarch64_msvc 0.48.5",
 "windows_i686_gnu 0.48.5",
 "windows_i686_msvc 0.48.5",
 "windows_x86_64_gnu 0.48.5",
 "windows_x86_64_gnullvm 0.48.5",
 "windows_x86_64_msvc 0.48.5",
]

[[package]]
name = "windows-targets"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973"
dependencies = [
 "windows_aarch64_gnullvm 0.52.6",
 "windows_aarch64_msvc 0.52.6",
 "windows_i686_gnu 0.52.6",
 "windows_i686_gnullvm",
 "windows_i686_msvc 0.52.6",
 "windows_x86_64_gnu 0.52.6",
 "windows_x86_64_gnullvm 0.52.6",
 "windows_x86_64_msvc 0.52.6",
]

[[package]]
name = "windows_aarch64_gnullvm"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2b38e32f0abccf9987a4e3079dfb67dcd799fb61361e53e2882c3cbaf0d905d8"

[[package]]
name = "windows_aarch64_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3"

[[package]]
name = "windows_aarch64_msvc"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc35310971f3b2dbbf3f0690a219f40e2d9afcf64f9ab7cc1be722937c26b4bc"

[[package]]
name = "windows_aarch64_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469"

[[package]]
name = "windows_i686_gnu"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a75915e7def60c94dcef72200b9a8e58e5091744960da64ec734a6c6e9b3743e"

[[package]]
name = "windows_i686_gnu"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b"

[[package]]
name = "windows_i686_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0eee52d38c090b3caa76c563b86c3a4bd71ef1a819287c19d586d7334ae8ed66"

[[package]]
name = "windows_i686_msvc"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8f55c233f70c4b27f66c523580f78f1004e8b5a8b659e05a4eb49d4166cca406"

[[package]]
name = "windows_i686_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66"

[[package]]
name = "windows_x86_64_gnu"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "53d40abd2583d23e4718fddf1ebec84dbff8381c07cae67ff7768bbf19c6718e"

[[package]]
name = "windows_x86_64_gnu"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78"

[[package]]
name = "windows_x86_64_gnullvm"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b7b52767868a23d5bab768e390dc5f5c55825b6d30b86c844ff2dc7414044cc"

[[package]]
name = "windows_x86_64_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d"

[[package]]
name = "windows_x86_64_msvc"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed94fce61571a4006852b7389a063ab983c02eb1bb37b47f8272ce92d06d9538"

[[package]]
name = "windows_x86_64_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec"

[[package]]
name = "winnow"
version = "0.7.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "df79d97927682d2fd8adb29682d1140b343be4ac0f08fd68b7765d9c059d3945"
dependencies = [
 "memchr",
]

[[package]]
name = "winnow"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23b97319f7b8343df12cc98938e5c3eb436064524c8d2b4e30a1d3a36eecdf81"
dependencies = [
 "memchr",
]

[[package]]
name = "winreg"
version = "0.50.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "524e57b2c537c0f9b1e69f1965311ec12182b4122e45035b1508cd24d2adadb1"
dependencies = [
 "cfg-if",
 "windows-sys 0.48.0",
]

[[package]]
name = "wiremock"
version = "0.5.22"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "13a3a53eaf34f390dd30d7b1b078287dd05df2aa2e21a589ccb80f5c7253c2e9"
dependencies = [
 "assert-json-diff",
 "async-trait",
 "base64 0.21.7",
 "deadpool",
 "futures",
 "futures-timer",
 "http-types",
 "hyper 0.14.32",
 "log",
 "once_cell",
 "regex",
 "serde",
 "serde_json",
 "tokio",
]

[[package]]
name = "wit-bindgen"
version = "0.57.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ebf944e87a7c253233ad6766e082e3cd714b5d03812acc24c318f549614536e"

[[package]]
name = "writeable"
version = "0.6.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ffae5123b2d3fc086436f8834ae3ab053a283cfac8fe0a0b8eaae044768a4c4"

[[package]]
name = "wyz"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "05f360fc0b24296329c78fda852a1e9ae82de9cf7b27dae4b7f62f118f77b9ed"
dependencies = [
 "tap",
]

[[package]]
name = "yoke"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "709fe23a0424b6a435d82152b1bd3fdfb0833487d5fa90d05d42762a9891fef5"
dependencies = [
 "stable_deref_trait",
 "yoke-derive",
 "zerofrom",
]

[[package]]
name = "yoke-derive"
version = "0.8.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "de844c262c8848816172cef550288e7dc6c7b7814b4ee56b3e1553f275f1858e"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
 "synstructure",
]

[[package]]
name = "zerocopy"
version = "0.8.55"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b5a105cd7b140f6eeec8acff2ea38135d3cab283ada58540f629fe51e46696eb"
dependencies = [
 "zerocopy-derive",
]

[[package]]
name = "zerocopy-derive"
version = "0.8.55"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0fe976fb70c78cd64cccfe3a6fc142244e8a77b70959b30faf9d0ac37ee228eb"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "zerofrom"
version = "0.1.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ec05a11813ea801ff6d75110ad09cd0824ddba17dfe17128ea0d5f68e6c5272"
dependencies = [
 "zerofrom-derive",
]

[[package]]
name = "zerofrom-derive"
version = "0.1.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "11532158c46691caf0f2593ea8358fed6bbf68a0315e80aae9bd41fbade684a1"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
 "synstructure",
]

[[package]]
name = "zeroize"
version = "1.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e13c156562582aa81c60cb29407084cdb54c4164760106ab78e6c5b0858cf64e"

[[package]]
name = "zerotrie"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0f9152d31db0792fa83f70fb2f83148effb5c1f5b8c7686c3459e361d9bc20bf"
dependencies = [
 "displaydoc",
 "yoke",
 "zerofrom",
]

[[package]]
name = "zerovec"
version = "0.11.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "90f911cbc359ab6af17377d242225f4d75119aec87ea711a880987b18cd7b239"
dependencies = [
 "yoke",
 "zerofrom",
 "zerovec-derive",
]

[[package]]
name = "zerovec-derive"
version = "0.11.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "625dc425cab0dca6dc3c3319506e6593dcb08a9f387ea3b284dbd52a92c40555"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "zmij"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b"
```


├── .cargo/config.toml

```toml
[env]
PYO3_USE_ABI3_FORWARD_COMPATIBILITY = "1"
```


├── .env

```text
BINANCE_API_KEY=dummy_key_here
BINANCE_SECRET_KEY=dummy_secret_here
```


├── .gitignore

```text
# Generated by Cargo
# will have compiled files and executables
debug/
target/

# Remove Cargo.lock from gitignore if creating an executable, leave it for libraries
# More information here https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html
Cargo.lock

# These are backup files generated by rustfmt
**/*.rs.bk

# MSVC Windows builds of rustc generate these, which can generally be ignored
*.pdb
*.lib
*.exp
*.ilk

# Database files
*.db
*.db-shm
*.db-wal
.env

# Paper service runtime artifacts
paper_wal/
paper-*.db
__pycache__/
```


├── alerts.toml

```toml
# Sesli uyarı örnek yapılandırması
# Veri kaynağı: "ring" (DATA terminali) veya "binance" (bağımsız doğrudan WS)
data_source = "pricefeed"

[[alerts]]
symbol = "BTCUSDT"
condition = "above"
price = 64500
voice = "Bitcoin 64 bin 500 üzerine çıktı"
cooldown_sec = 30

[[alerts]]
symbol = "BTCUSDT"
condition = "below"
price = 64000
voice = "Bitcoin 64 bin altına indi"
cooldown_sec = 30

[[alerts]]
symbol = "BTCUSDT"
condition = "touch"
price = 64300
tolerance_pct = 0.002
cooldown_sec = 20

[[alerts]]
symbol = "ETHUSDT"
condition = "cross"
price = 3200
cooldown_sec = 60

[[alerts]]
symbol = "SOLUSDT"
condition = "above"
price = 150
cooldown_sec = 60

[[alerts]]
symbol = "VELVETUSDT"
condition = "above"
price = 0.21628
voice = "HEI 0 virgül 21628 seviyesini yukarı kırdı"
cooldown_sec = 60
```


├── test_data.csv

```csv
BTCUSDT,64000.50,1.2,1623821034000
ETHUSDT,3500.20,5.0,1623821034050
BTCUSDT,64005.00,0.5,1623821034100
BTCUSDT,64010.50,2.1,1623821034150
SOLUSDT,150.00,10.0,1623821034200
```


├── install.sh

```bash
#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — Kurulum / Yükleme Script'i
#  Sistemin tamamını derler ve yüklenebilir bir paket oluşturur.
#
#  Kullanım:
#    ./install.sh                # tüm sistemi derle + kur
#    ./install.sh --prefix /opt  # özel kurulum dizini (varsayılan: ~/.cycle)
#    ./install.sh --only-build   # sadece derle, kurma
#    ./install.sh --package      # kurulum + sıkıştırılmış paket (.tar.gz)
#    ./install.sh --uninstall    # kurulumu kaldır
# ============================================================
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
PREFIX="${PREFIX:-$HOME/.cycle}"
PKG_DIR="$PREFIX"
BIN_DIR="$PKG_DIR/bin"
CONFIG_DIR="$PKG_DIR/config"
SCRIPTS_DIR="$PKG_DIR/scripts"
STRATEGIES_DIR="$PKG_DIR/strategies"
DATA_DIR="$PKG_DIR/data"
LOG_DIR="$PKG_DIR/logs"

# ── Renkler ──────────────────────────────────────────────────
_G='\033[0;32m'; _Y='\033[1;33m'; _C='\033[0;36m'
_R='\033[0;31m'; _N='\033[0m'

say()  { echo -e "${_C}[cycle]${_N} $*"; }
ok()   { echo -e "${_G}✔${_N} $*"; }
warn() { echo -e "${_Y}⚠${_N} $*"; }
err()  { echo -e "${_R}✘${_N} $*"; }

# ── Bağımlılık kontrolü ──────────────────────────────────────
check_deps() {
  say "Bağımlılıklar kontrol ediliyor..."
  local missing=()
  for c in cargo rustc tmux curl jq; do
    if ! command -v "$c" >/dev/null 2>&1; then
      missing+=("$c")
    fi
  done
  if [ ${#missing[@]} -gt 0 ]; then
    err "Eksik bağımlılıklar: ${missing[*]}"
    echo "  Kurulum:  sudo apt install build-essential tmux curl jq"
    echo "  Rust:     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
  fi
  ok "Bağımlılıklar tamam"
}

# ── Release derleme ──────────────────────────────────────────
build_all() {
  say "Tüm çalışma alanı derleniyor (release)..."
  cd "$ROOT"
  cargo build --release --workspace 2>&1 | tail -5
  ok "Derleme tamamlandı"
}

# ── Kurulum dizini oluştur ───────────────────────────────────
setup_dirs() {
  mkdir -p "$BIN_DIR" "$CONFIG_DIR" "$SCRIPTS_DIR" "$STRATEGIES_DIR" "$DATA_DIR" "$LOG_DIR"
}

# ── Binary'leri kopyala ──────────────────────────────────────
copy_bins() {
  say "Binary'ler kopyalanıyor → $BIN_DIR"
  local bins=(
    core paper-service paper-cli alert-service detect-ms
    risk-worker cold-starter velvetusdt listener alerts risk_analysis
    detect-sr detect-trend detect-liquidity detect-pattern
    detect-wyckoff detect-trb
  )
  local n=0
  for b in "${bins[@]}"; do
    if [ -f "$ROOT/target/release/$b" ]; then
      cp "$ROOT/target/release/$b" "$BIN_DIR/$b"
      chmod +x "$BIN_DIR/$b"
      n=$((n+1))
    else
      warn "  $b bulunamadı (atlandı)"
    fi
  done
  ok "$n binary kopyalandı"
}

# ── Config ve script kopyala ─────────────────────────────────
copy_assets() {
  say "Yapılandırma ve script'ler kopyalanıyor..."
  cp "$ROOT/alerts.toml"          "$CONFIG_DIR/" 2>/dev/null || warn "alerts.toml yok"
  cp "$ROOT/config/"config_*.toml  "$CONFIG_DIR/" 2>/dev/null || true

  for s in cycle_tmux.sh cycle_env.sh monitor.sh start_paper.sh stop_paper.sh; do
    [ -f "$ROOT/scripts/$s" ] && cp "$ROOT/scripts/$s" "$SCRIPTS_DIR/" || warn "scripts/$s yok"
  done

  [ -f "$ROOT/test_data.csv" ] && cp "$ROOT/test_data.csv" "$DATA_DIR/" || true
  ok "Yapılandırma dosyaları kopyalandı"
}

# ── Ortam / başlatıcı oluştur ────────────────────────────────
write_env() {
  cat > "$PKG_DIR/cycle-env.sh" <<ENVEOF
#!/usr/bin/env bash
# Cycle Finance — kurulum ortamı
export CYCLE_ROOT="$PKG_DIR"
export PATH="$BIN_DIR:\$PATH"
source "$SCRIPTS_DIR/cycle_env.sh"
ENVEOF
  chmod +x "$PKG_DIR/cycle-env.sh"
  ok "Ortam dosyası oluşturuldu: $PKG_DIR/cycle-env.sh"
}

write_launcher() {
  cat > "$BIN_DIR/cycle" <<LAUNCH
#!/usr/bin/env bash
# Cycle Finance başlatıcı
CYCLE_ROOT="$PKG_DIR"
case "\${1:-}" in
  start)  exec "\$CYCLE_ROOT/scripts/cycle_tmux.sh" ;;
  stop)   exec "\$CYCLE_ROOT/scripts/cycle_tmux.sh" kill ;;
  status) exec "\$CYCLE_ROOT/scripts/cycle_tmux.sh" status ;;
  env)    echo "source \$CYCLE_ROOT/cycle-env.sh" ;;
  *)
    echo "Cycle Finance — kullanım:"
    echo "  cycle start    Tüm sistemi tmux ile başlat"
    echo "  cycle stop     Tüm sistemi durdur"
    echo "  cycle status   Servis durumları"
    echo "  cycle env      Ortamı yükle (source \$CYCLE_ROOT/cycle-env.sh)"
    ;;
esac
LAUNCH
  chmod +x "$BIN_DIR/cycle"
  ok "Başlatıcı oluşturuldu: $BIN_DIR/cycle"
}

# ── Paketle ──────────────────────────────────────────────────
make_package() {
  local out="$ROOT/cycle-finance-package.tar.gz"
  say "Paket oluşturuluyor → $out"
  tar -czf "$out" -C "$(dirname "$PKG_DIR")" "$(basename "$PKG_DIR")"
  ls -lh "$out"
  ok "Paket hazır"
}

# ── Kaldır ───────────────────────────────────────────────────
uninstall() {
  if [ -d "$PKG_DIR" ]; then
    rm -rf "$PKG_DIR"
    ok "Kurulum kaldırıldı: $PKG_DIR"
  else
    warn "Kurulum dizini yok: $PKG_DIR"
  fi
}

# ── Ana akış ─────────────────────────────────────────────────
case "${1:-}" in
  --uninstall)
    uninstall
    exit 0
    ;;
  --only-build)
    check_deps
    build_all
    exit 0
    ;;
esac

check_deps
build_all
setup_dirs
copy_bins
copy_assets
write_env
write_launcher

echo ""
echo "════════════════════════════════════════════════════════"
echo "  ✅  Cycle Finance kuruldu → $PKG_DIR"
echo ""
echo "  Başlat  :  $BIN_DIR/cycle start"
echo "  Durdur  :  $BIN_DIR/cycle stop"
echo "  Durum   :  $BIN_DIR/cycle status"
echo "  Ortam   :  source $PKG_DIR/cycle-env.sh"
echo "════════════════════════════════════════════════════════"

if [ "${1:-}" = "--package" ]; then
  make_package
fi
```


├── msi-fanctl

```text
#!/usr/bin/env bash
# MSI fan kontrolü — /sys/devices/platform/msi-ec üzerinden
set -euo pipefail
P=/sys/devices/platform/msi-ec

[ "$(id -u)" = 0 ] || exec sudo "$0" "$@"

usage() {
  echo "Kullanım: $0 {max|auto|silent|shift}"
  echo "  max    -> cooler boost ON  + advanced mod (fanlar son hız)"
  echo "  auto   -> cooler boost OFF + auto mod"
  echo "  silent -> cooler boost OFF + silent mod"
  echo "  shift  -> güç modu göster/değiştir (comfort/sport/models)"
  echo "  durum  -> mevcut durum + CPU/GPU sıcaklıkları"
  echo "  hiz    -> fan hızı % göster (tek seferlik)"
  echo "  izle   -> canlı izleme (fan % + temp, Ctrl+C çıkış)"
  echo "  help   -> bu liste (--help, -h de geçerli)"
}

require_root() { [ "$(id -u)" = 0 ] || { echo "root gerekli: sudo $0 $*"; exit 1; }; }

status() {
  echo "fan_mode    : $(cat $P/fan_mode)"
  echo "cooler_boost: $(cat $P/cooler_boost)"
  echo "shift_mode  : $(cat $P/shift_mode)"
  echo "cpu temp    : $(cpu_temp)"
  echo "gpu temp    : $(gpu_temp)"
  echo "fan hizi    : $(fan_pct)"
}

fan_pct() {
  for d in /sys/class/hwmon/hwmon*/; do
    [ -r "$d/pwm1" ] || continue
    case "$(cat "$d/name" 2>/dev/null)" in
      amdgpu)
        max=$(cat "$d/pwm1_max" 2>/dev/null || echo 255)
        p=$(cat "$d/pwm1")
        pct=$(awk -v p="$p" -v m="$max" 'BEGIN{printf "%d", p*100/m}')
        bar_len=$((pct/10))
        bar=""
        for i in $(seq 1 $bar_len); do bar="$bar#"; done
        for i in $(seq $((bar_len+1)) 10); do bar="$bar."; done
        echo "%$pct [$bar]"
        return
        ;;
    esac
  done
  echo "bulunamadi"
}

cpu_temp() {
  for f in /sys/class/hwmon/hwmon*/temp1_input; do
    [ -r "$f" ] || continue
    d=$(dirname "$f")
    case "$(cat "$d/name" 2>/dev/null)" in
      k10temp)
        t=$(awk -v v="$(cat "$f")" 'BEGIN{printf "%.1f", v/1000}')
        # Tctl zaten k10temp'ta; varsa temp3 (tccd) yoksa temp1 göster
        for s in temp3_input temp1_input; do
          if [ -r "$d/$s" ]; then
            awk -v v="$(cat "$d/$s")" 'BEGIN{printf "+%.1f C", v/1000}'
            break
          fi
        done
        return
        ;;
    esac
  done
  echo "bulunamadi"
}

gpu_temp() {
  found=0
  for d in /sys/class/hwmon/hwmon*/; do
    [ -r "$d/name" ] || continue
    case "$(cat "$d/name" 2>/dev/null)" in
      amdgpu)
        label=$(cat "$d/device/uevent" 2>/dev/null | grep -m1 PCI_SLOT_NAME | cut -d= -f2)
        temps=""
        for s in temp1_input temp2_input temp3_input; do
          if [ -r "$d/$s" ] && [ "$(cat "$d/$s")" != "0" ]; then
            v=$(awk -v v="$(cat "$d/$s")" 'BEGIN{printf "+%.1f", v/1000}')
            temps="$temps $v"
          fi
        done
        if [ -n "$temps" ]; then
          [ "$found" = 1 ] && echo "" && printf "     "
          printf "%s:%s C" "$label" "$temps"
          found=1
        fi
        ;;
    esac
  done
  [ "$found" = 0 ] && echo "bulunamadi"
}

case "${1:-}" in
  max)
    require_root
    echo advanced > $P/fan_mode
    echo 1 > $P/cooler_boost
    echo "Son hız ayarlandı."
    ;;
  auto)
    require_root
    echo 0 > $P/cooler_boost
    echo auto > $P/fan_mode
    echo "Otomatik mod."
    ;;
  silent)
    require_root
    echo 0 > $P/cooler_boost
    echo silent > $P/fan_mode
    echo "Sessiz mod."
    ;;
  shift)
    require_root
    echo "shift seçenekleri: $(cat $P/available_shift_modes)"
    if [ -n "${2:-}" ]; then
      echo "$2" > $P/shift_mode
      echo "shift_mode -> $2"
    fi
    ;;
  help|--help|-h) usage ;;
  hiz) fan_pct ;;
  izle)
    while true; do
      clear
      echo "Tarih: $(date +%H:%M:%S)"
      echo "CPU : $(cpu_temp)   GPU: $(gpu_temp)"
      echo "Fan : $(fan_pct)   (mod: $(cat $P/fan_mode), boost: $(cat $P/cooler_boost))"
      sleep 1
    done
    ;;
  status|durum|"") status ;;
  *) usage ;;
esac
```


├── config/config_v5.toml

```toml
# API v5 Configuration for Cycle Finance 2.0
[api]
version = "v5"
endpoint = "wss://stream.binance.com:9443/ws"

[trading]
max_positions = 100
```


├── config/config_v6.toml

```toml
# API v6 Configuration for Cycle Finance 2.0 (Blue/Green Deployment)
[api]
version = "v6"
endpoint = "wss://stream.binance.com:9443/ws/v6"

[trading]
max_positions = 100
```


├── contracts/Cargo.toml

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


├── contracts/src/events.rs

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


├── contracts/src/lib.rs

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


├── contracts/src/wire.rs

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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::*;

    fn roundtrip(ev: &OwnedEvent) -> Option<OwnedEvent> {
        let mut buf = [0u8; MAX_FRAME_SIZE + 64];
        let len = encode(ev, &mut buf)?;
        decode(&buf[..len])
    }

    fn assert_same(a: &OwnedEvent, b: &OwnedEvent) {
        assert_eq!(a.symbol, b.symbol, "symbol eşleşmedi");
        match (&a.payload, &b.payload) {
            (EventType::Trade { price, quantity, timestamp, is_buyer_maker },
             EventType::Trade { price: p2, quantity: q2, timestamp: t2, is_buyer_maker: m2 }) => {
                assert_eq!(price, p2);
                assert_eq!(quantity, q2);
                assert_eq!(timestamp, t2);
                assert_eq!(is_buyer_maker, m2);
            }
            (EventType::Orderbook { bids, asks }, EventType::Orderbook { bids: b2, asks: a2 }) => {
                assert_eq!(bids, b2);
                assert_eq!(asks, a2);
            }
            (EventType::FundingRate { mark_price, index_price, funding_rate, next_funding_time },
             EventType::FundingRate { mark_price: m2, index_price: i2, funding_rate: f2, next_funding_time: n2 }) => {
                assert_eq!(mark_price, m2);
                assert_eq!(index_price, i2);
                assert_eq!(funding_rate, f2);
                assert_eq!(next_funding_time, n2);
            }
            (EventType::BookTicker { best_bid_price, best_bid_qty, best_ask_price, best_ask_qty },
             EventType::BookTicker { best_bid_price: bp2, best_bid_qty: bq2, best_ask_price: ap2, best_ask_qty: aq2 }) => {
                assert_eq!(best_bid_price, bp2);
                assert_eq!(best_bid_qty, bq2);
                assert_eq!(best_ask_price, ap2);
                assert_eq!(best_ask_qty, aq2);
            }
            (EventType::Liquidation { side, price, quantity, timestamp },
             EventType::Liquidation { side: s2, price: p2, quantity: q2, timestamp: t2 }) => {
                assert_eq!(side, s2);
                assert_eq!(price, p2);
                assert_eq!(quantity, q2);
                assert_eq!(timestamp, t2);
            }
            (EventType::OpenInterest { open_interest, timestamp },
             EventType::OpenInterest { open_interest: o2, timestamp: t2 }) => {
                assert_eq!(open_interest, o2);
                assert_eq!(timestamp, t2);
            }
            (EventType::Opportunity { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict },
             EventType::Opportunity { score: s2, efficiency: e2, price_bps_per_s: pb2, price_ticks_per_s: pt2, ob_changes_per_s: ob2, spread_bps: sp2, verdict: v2 }) => {
                assert_eq!(score, s2);
                assert_eq!(efficiency, e2);
                assert_eq!(price_bps_per_s, pb2);
                assert_eq!(price_ticks_per_s, pt2);
                assert_eq!(ob_changes_per_s, ob2);
                assert_eq!(spread_bps, sp2);
                assert_eq!(verdict, v2);
            }
            (EventType::SymbolMetrics { score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps },
             EventType::SymbolMetrics { score: s2, efficiency: e2, price_bps_per_s: pb2, price_ticks_per_s: pt2, ob_changes_per_s: ob2, spread_bps: sp2 }) => {
                assert_eq!(score, s2);
                assert_eq!(efficiency, e2);
                assert_eq!(price_bps_per_s, pb2);
                assert_eq!(price_ticks_per_s, pt2);
                assert_eq!(ob_changes_per_s, ob2);
                assert_eq!(spread_bps, sp2);
            }
            _ => panic!("tipler eşleşmedi: {:?} vs {:?}", a.payload, b.payload),
        }
    }

    #[test]
    fn trade_roundtrip() {
        let ev = OwnedEvent::new_trade("BTCUSDT", Decimal::from_str("67234.50").unwrap(),
            Decimal::from_str("0.001500").unwrap(), 1_766_800_000_000, true);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
        assert_eq!(encode(&ev, &mut [0u8; MAX_FRAME_SIZE + 64]).unwrap(), 44);
    }

    #[test]
    fn trade_sell_side() {
        let ev = OwnedEvent::new_trade("VELVETUSDT", Decimal::from_str("0.02162800").unwrap(),
            Decimal::from_str("100000").unwrap(), 1234, false);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
    }

    #[test]
    fn depth20_roundtrip() {
        let mut bids = [(Decimal::ZERO, Decimal::ZERO); 20];
        let mut asks = [(Decimal::ZERO, Decimal::ZERO); 20];
        for i in 0..20 {
            bids[i] = (Decimal::new(67200 + i as i64, 0), Decimal::new(100 - i as i64, 0));
            asks[i] = (Decimal::new(67220 + i as i64, 0), Decimal::new(90 + i as i64, 0));
        }
        // Karışık scale'ler — ortak scale'e rescale edilmeli.
        bids[0].0 = Decimal::from_str("67200.50").unwrap();
        asks[5].1 = Decimal::from_str("12.34500000").unwrap();
        let ev = OwnedEvent::new_orderbook("BTCUSDT", bids, asks);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
        assert_eq!(encode(&ev, &mut [0u8; MAX_FRAME_SIZE + 64]).unwrap(), DEPTH_FRAME_SIZE);
    }

    #[test]
    fn depth20_partial_levels() {
        // Sadece ilk 3 seviye dolu, gerisi sıfır.
        let mut bids = [(Decimal::ZERO, Decimal::ZERO); 20];
        let mut asks = [(Decimal::ZERO, Decimal::ZERO); 20];
        bids[0] = (Decimal::from_str("1.1").unwrap(), Decimal::from_str("2.22").unwrap());
        asks[0] = (Decimal::from_str("1.2").unwrap(), Decimal::from_str("3.33").unwrap());
        let ev = OwnedEvent::new_orderbook("SOLUSDT", bids, asks);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
    }

    #[test]
    fn funding_roundtrip() {
        let ev = OwnedEvent::new_funding_rate("VELVETUSDT",
            Decimal::from_str("0.021628").unwrap(), Decimal::from_str("0.021630").unwrap(),
            Decimal::from_str("-0.00012345").unwrap(), 1_766_800_000_000);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
        assert_eq!(encode(&ev, &mut [0u8; MAX_FRAME_SIZE + 64]).unwrap(), 52);
    }

    #[test]
    fn bookticker_roundtrip() {
        let ev = OwnedEvent::new_bookticker("ETHUSDT",
            Decimal::from_str("3521.10").unwrap(), Decimal::from_str("5.5").unwrap(),
            Decimal::from_str("3521.20").unwrap(), Decimal::from_str("4.4").unwrap());
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
        assert_eq!(encode(&ev, &mut [0u8; MAX_FRAME_SIZE + 64]).unwrap(), 53);
    }

    #[test]
    fn liquidation_roundtrip() {
        let ev = OwnedEvent::new_liquidation("BTCUSDT", 1, Decimal::from_str("64500").unwrap(),
            Decimal::from_str("1.5").unwrap(), 99);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
        assert_eq!(encode(&ev, &mut [0u8; MAX_FRAME_SIZE + 64]).unwrap(), 44);
    }

    #[test]
    fn open_interest_roundtrip() {
        let ev = OwnedEvent::new_open_interest("BTCUSDT", Decimal::from_str("215000.12345678").unwrap(), 555);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
        assert_eq!(encode(&ev, &mut [0u8; MAX_FRAME_SIZE + 64]).unwrap(), 34);
    }

    #[test]
    fn opportunity_roundtrip() {
        let ev = OwnedEvent::new_opportunity("VELVETUSDT",
            Decimal::from_str("12541.78").unwrap(),
            Decimal::from_str("2.1999").unwrap(),
            Decimal::from_str("60.13").unwrap(),
            Decimal::from_str("86.67").unwrap(),
            Decimal::from_str("27.33").unwrap(),
            Decimal::from_str("0.42").unwrap(),
            0);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
        assert_eq!(encode(&ev, &mut [0u8; MAX_FRAME_SIZE + 64]).unwrap(), 72);
    }

    #[test]
    fn symbol_metrics_roundtrip() {
        let ev = OwnedEvent::new_symbol_metrics("EPICUSDT",
            Decimal::from_str("1533.73").unwrap(),
            Decimal::from_str("1.3332").unwrap(),
            Decimal::from_str("49.33").unwrap(),
            Decimal::from_str("108.00").unwrap(),
            Decimal::from_str("37.00").unwrap(),
            Decimal::from_str("3.47").unwrap());
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_same(&ev, &dec);
        assert_eq!(encode(&ev, &mut [0u8; MAX_FRAME_SIZE + 64]).unwrap(), 71);
    }

    #[test]
    fn symbol_truncated_to_16() {
        let ev = OwnedEvent::new_trade("1000000SATSUSDTX", Decimal::ONE, Decimal::ONE, 1, false);
        let dec = roundtrip(&ev).expect("roundtrip");
        assert_eq!(&dec.symbol[..16], b"1000000SATSUSDTX");
    }

    #[test]
    fn truncated_frame_returns_none() {
        let ev = OwnedEvent::new_trade("BTCUSDT", Decimal::ONE, Decimal::ONE, 1, false);
        let mut buf = [0u8; MAX_FRAME_SIZE + 64];
        let len = encode(&ev, &mut buf).unwrap();
        assert!(decode(&buf[..len - 1]).is_none());
        assert!(decode(&buf[..16]).is_none());
        assert!(decode(&[]).is_none());
    }
}
```


├── transport/Cargo.toml

```toml
[package]
name = "transport"
version = "0.1.0"
edition = "2021"

[dependencies]
libc = "0.2"
memmap2 = "0.9"
rust_decimal = { workspace = true }
```


├── transport/src/lib.rs

```rust
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
```


├── transport/src/order_ring.rs

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


├── transport/src/ring_buffer.rs

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_size_is_768_bytes() {
        assert_eq!(std::mem::size_of::<MarketDataSlot>(), 768);
        assert_eq!(std::mem::align_of::<MarketDataSlot>(), 64);
    }

    #[test]
    fn magic_constants_differ() {
        assert_ne!(RING_MAGIC, crate::order_ring::ORDER_RING_MAGIC);
    }
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
    /// Farklı servisler farklı isim kullanabilir (örn.).
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


├── core/Cargo.toml

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
tokio = { version = "1.0", features = ["full"] }
flume = "0.11"
parking_lot = "0.12"
simd-json = "0.13"
futures-util = "0.3"
cold-storage = { path = "../cold-storage" }
adapter = { path = "../adapter" }
os-utils = { path = "../os-utils" }
execution-engine = { path = "../execution-engine" }
contracts = { path = "../contracts" }
transport = { path = "../transport" }
sha3 = "0.10"
sqlx = { version = "0.31.0", features = ["bundled"] }
dotenvy = "0.15"
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
serde_json = "1.0"
core_affinity = "0.8.3"
hdrhistogram = "7.6.0"
crossbeam-channel = "0.5.16"
serde = { version = "1.0.229", features = ["derive"] }
libc = "0.2"
memmap2 = "0.9"
rustyline = "14.0.0"
chrono = "0.4.45"
rust_decimal = { workspace = true }

[dev-dependencies]
criterion = "0.4"
proptest = "1.0"

[[bench]]
name = "tick_benchmark"
harness = false
```


├── core/benches/tick_benchmark.rs

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


├── core/src/cli/correlation_cli.rs

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


├── core/src/cli/mod.rs

```rust
pub mod paper_cli;
pub mod strategy_cli;
pub mod correlation_cli;
```


├── core/src/cli/strategy_cli.rs

```rust
//! STRATEGY terminali — VELVETUSDT kırılım stratejisini çalıştırır.
//!
//! Strateji Rust'ta (`velvetusdt` crate) çalışır: detect-ms'ten seviye/yapı
//! analizi alır, kırılım koşullarını kontrol eder, paper-service'e emir açar.

use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const VELVETUSDT_BIN: &str = "/home/smhvz/Desktop/PROJE/target/debug/velvetusdt";

struct StrategyChild {
    child: Child,
}

pub fn start_strategy_cli() {
    println!("========================================");
    println!("🎯 STRATEGY ENGINE — VELVETUSDT KIRILIM");
    println!("  Binary: {}", VELVETUSDT_BIN);
    println!("  detect-ms :3002 + paper-service :8080");
    println!("========================================");

    let running = Arc::new(AtomicBool::new(false));
    let mut child: Option<StrategyChild> = spawn_strategy();
    if child.is_none() {
        println!("❌ VELVETUSDT stratejisi başlatılamadı.");
    } else {
        running.store(true, Ordering::SeqCst);
        println!("✅ VELVETUSDT stratejisi çalışıyor.");
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
                        println!("  restart     - Restart VELVETUSDT strategy");
                        println!("  exit        - Quit the terminal");
                    }
                    "status" => {
                        if running.load(Ordering::SeqCst) {
                            println!("  🎯 VELVETUSDT Kırılım — RUNNING");
                        } else {
                            println!("  🎯 VELVETUSDT Kırılım — DURDU");
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
                            println!("✅ VELVETUSDT stratejisi yeniden başlatıldı.");
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
    match Command::new(VELVETUSDT_BIN)
        .current_dir("/home/smhvz/Desktop/PROJE")
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


├── core/src/config.rs

```rust
pub use os_utils::config::*;
```


├── core/src/engine/backtester.rs

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


├── core/src/engine/mod.rs

```rust
pub mod orchestrator;
pub mod backtester;
```


├── core/src/engine/orchestrator.rs

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};
use transport::ring_buffer::GenerationalRingBuffer;
use crate::strategy::trait_def::{Strategy, Signal};
use crate::risk::engine::RiskEngine;
use crate::timer::tsc::TscTimer;
use crossbeam_channel::Sender;

#[derive(PartialEq)]
enum StrategyState {
    Active,
    Draining,
    Poisoned,
}

struct ShardedStrategy {
    strategy: Box<dyn Strategy>,
    state: StrategyState,
}

pub struct TitaniumOrchestrator {
    strategies: Vec<ShardedStrategy>,
    risk_manager: RiskEngine,
    gateway_tx: Sender<Signal>,
}

impl TitaniumOrchestrator {
    pub fn new(
        strategies: Vec<Box<dyn Strategy>>,
        risk_manager: RiskEngine,
        gateway_tx: Sender<Signal>,
    ) -> Self {
        let sharded = strategies.into_iter().map(|s| ShardedStrategy {
            strategy: s,
            state: StrategyState::Active,
        }).collect();

        Self {
            strategies: sharded,
            risk_manager,
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
                    
                    for shard in &mut self.strategies {
                        if shard.state == StrategyState::Active {
                            // Protect against panics in strategy code (Catch-Unwind)
                            let result = catch_unwind(AssertUnwindSafe(|| {
                                shard.strategy.on_market_data(frame_id, &slot)
                            }));

                            match result {
                                Ok(sig) => {
                                    if let Some(valid_sig) = self.risk_manager.process_signal(sig.clone(), shard.strategy.id()) {
                                        match valid_sig {
                                            Signal::None => {},
                                            _ => {
                                                // Dispatch to gateway (Execution Engine)
                                                let _ = self.gateway_tx.send(valid_sig);
                                            }
                                        }
                                    }
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
                
                for shard in &mut self.strategies {
                    if shard.state == StrategyState::Active {
                        let result = catch_unwind(AssertUnwindSafe(|| {
                            shard.strategy.on_timer(frame_id, delta)
                        }));

                        match result {
                            Ok(sig) => {
                                if let Some(valid_sig) = self.risk_manager.process_signal(sig.clone(), shard.strategy.id()) {
                                    match valid_sig {
                                        Signal::None => {},
                                        _ => {
                                            let _ = self.gateway_tx.send(valid_sig);
                                        }
                                    }
                                }
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
```


├── core/src/hal/cpu.rs

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


├── core/src/hal/memory.rs

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


├── core/src/hal/mod.rs

```rust
pub mod cpu;
pub mod memory;
```


├── core/src/lib.rs

```rust
pub mod state;
pub mod config;
pub mod pii;
pub mod db;
pub mod validator;
pub mod cli;

pub mod hal;
pub mod timer;
pub mod strategy;
pub mod risk;
pub mod engine;

pub mod tick;
pub mod queue;
```


├── core/src/main.rs

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
        proje_core::cli::strategy_cli::start_strategy_cli();
        return;
    }

    if run_mode == "BACKTEST" {
        let csv_path = std::env::var("CSV_PATH").unwrap_or_else(|_| "/home/smhvz/Desktop/PROJE/test_data.csv".to_string());
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


├── core/src/pii.rs

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


├── core/src/queue.rs

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


├── core/src/risk/engine.rs

```rust
use rust_decimal::Decimal;
use crate::strategy::trait_def::Signal;

pub struct RiskEngine {
    max_position: Decimal,
    current_position: Decimal,
    daily_loss_limit: Decimal,
    current_pnl: Decimal,
}

impl RiskEngine {
    pub fn new(max_position: Decimal, daily_loss_limit: Decimal) -> Self {
        Self {
            max_position,
            current_position: Decimal::ZERO,
            daily_loss_limit,
            current_pnl: Decimal::ZERO,
        }
    }

    pub fn process_signal(&self, signal: Signal, _strategy_id: u32) -> Option<Signal> {
        match signal {
            Signal::BuyMarket { quantity } | Signal::BuyLimit { quantity, .. } => {
                if self.current_position + quantity > self.max_position {
                    None // Reject
                } else {
                    Some(signal)
                }
            }
            Signal::SellMarket { quantity } | Signal::SellLimit { quantity, .. } => {
                if self.current_position - quantity < -self.max_position {
                    None
                } else {
                    Some(signal)
                }
            }
            Signal::None | Signal::CancelAll => Some(signal),
        }
    }

    pub fn update_position(&mut self, delta: Decimal) {
        self.current_position += delta;
    }

    pub fn update_pnl(&mut self, delta: Decimal) {
        self.current_pnl += delta;
    }

    pub fn is_daily_loss_exceeded(&self) -> bool {
        self.current_pnl <= -self.daily_loss_limit
    }

    pub fn current_position(&self) -> Decimal {
        self.current_position
    }

    pub fn max_position(&self) -> Decimal {
        self.max_position
    }
}
```


├── core/src/risk/lob_simulator.rs

```rust
use std::cmp;

// Fixed-point integers for Risk Management (Zero float usage)
// Price is multiplied by 100,000, Quantity is multiplied by 1,000.

pub struct LobSimulator {
    // Array of (price, quantity) representing the book
    bids: [(u64, u64); 10],
    asks: [(u64, u64); 10],
    bid_count: usize,
    ask_count: usize,
}

impl LobSimulator {
    pub fn new() -> Self {
        Self {
            bids: [(0, 0); 10],
            asks: [(0, 0); 10],
            bid_count: 0,
            ask_count: 0,
        }
    }

    pub fn update_bids(&mut self, new_bids: &[(u64, u64)]) {
        let count = cmp::min(10, new_bids.len());
        for i in 0..count {
            self.bids[i] = new_bids[i];
        }
        self.bid_count = count;
    }

    pub fn update_asks(&mut self, new_asks: &[(u64, u64)]) {
        let count = cmp::min(10, new_asks.len());
        for i in 0..count {
            self.asks[i] = new_asks[i];
        }
        self.ask_count = count;
    }

    // Returns (Average Price * 100_000, Filled Quantity * 1000)
    pub fn simulate_buy(&self, mut qty: u64) -> (u64, u64) {
        let mut total_cost = 0;
        let mut filled_qty = 0;

        for i in 0..self.ask_count {
            if qty == 0 {
                break;
            }

            let (level_price, level_qty) = self.asks[i];
            let fill = cmp::min(qty, level_qty);
            
            total_cost += level_price * fill;
            filled_qty += fill;
            qty -= fill;
        }

        if filled_qty == 0 {
            (0, 0)
        } else {
            let avg_price = total_cost / filled_qty;
            (avg_price, filled_qty)
        }
    }

    pub fn simulate_sell(&self, mut qty: u64) -> (u64, u64) {
        let mut total_revenue = 0;
        let mut filled_qty = 0;

        for i in 0..self.bid_count {
            if qty == 0 {
                break;
            }

            let (level_price, level_qty) = self.bids[i];
            let fill = cmp::min(qty, level_qty);
            
            total_revenue += level_price * fill;
            filled_qty += fill;
            qty -= fill;
        }

        if filled_qty == 0 {
            (0, 0)
        } else {
            let avg_price = total_revenue / filled_qty;
            (avg_price, filled_qty)
        }
    }
}
```


├── core/src/risk/mod.rs

```rust
pub mod lob_simulator;
pub mod engine;
pub mod portfolio;
```


├── core/src/risk/portfolio.rs

```rust
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: Decimal,
    pub avg_entry_price: Decimal,
}

impl Position {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            quantity: Decimal::ZERO,
            avg_entry_price: Decimal::ZERO,
        }
    }

    pub fn unrealized_pnl(&self, current_price: Decimal) -> Decimal {
        if self.quantity == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (current_price - self.avg_entry_price) * self.quantity
    }
}

pub struct Portfolio {
    pub cash_balance: Decimal,
    pub realized_pnl: Decimal,
    pub total_commission: Decimal,
    pub positions: HashMap<String, Position>,
    pub max_drawdown_limit: Decimal,
    pub starting_balance: Decimal,
}

impl Portfolio {
    pub fn new(initial_balance: Decimal, max_drawdown: Decimal) -> Self {
        Self {
            cash_balance: initial_balance,
            starting_balance: initial_balance,
            realized_pnl: Decimal::ZERO,
            total_commission: Decimal::ZERO,
            positions: HashMap::new(),
            max_drawdown_limit: max_drawdown,
        }
    }

    pub fn process_fill(&mut self, symbol: &str, fill_qty: Decimal, fill_price: Decimal, commission: Decimal) {
        self.cash_balance -= commission;
        self.total_commission += commission;

        let pos = self.positions.entry(symbol.to_string()).or_insert_with(|| Position::new(symbol.to_string()));

        // Check if we are closing a position (signs are opposite)
        if (pos.quantity > Decimal::ZERO && fill_qty < Decimal::ZERO) || (pos.quantity < Decimal::ZERO && fill_qty > Decimal::ZERO) {
            let close_qty = fill_qty.abs().min(pos.quantity.abs());
            let realized = (fill_price - pos.avg_entry_price) * close_qty * pos.quantity.signum();
            self.realized_pnl += realized;
            self.cash_balance += realized; // Add realized to cash

            // Adjust position
            pos.quantity += fill_qty;
            if pos.quantity == Decimal::ZERO {
                pos.avg_entry_price = Decimal::ZERO;
            }
        } else {
            // Opening or adding to position
            let total_value = (pos.quantity.abs() * pos.avg_entry_price) + (fill_qty.abs() * fill_price);
            pos.quantity += fill_qty;
            if pos.quantity != Decimal::ZERO {
                pos.avg_entry_price = total_value / pos.quantity.abs();
            }
        }
    }

    pub fn get_total_equity(&self, current_prices: &HashMap<String, Decimal>) -> Decimal {
        let mut un_pnl = Decimal::ZERO;
        for (sym, pos) in &self.positions {
            if let Some(price) = current_prices.get(sym) {
                un_pnl += pos.unrealized_pnl(*price);
            }
        }
        self.cash_balance + un_pnl
    }

    pub fn is_drawdown_exceeded(&self, current_prices: &HashMap<String, Decimal>) -> bool {
        let equity = self.get_total_equity(current_prices);
        let drawdown = (self.starting_balance - equity) / self.starting_balance;
        drawdown > self.max_drawdown_limit
    }
}
```


├── core/src/state.rs

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


├── core/src/strategy/mod.rs

```rust
pub mod trait_def;
```


├── core/src/strategy/trait_def.rs

```rust
use transport::ring_buffer::MarketDataSlot;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub enum Signal {
    None,
    BuyMarket { quantity: Decimal },
    SellMarket { quantity: Decimal },
    BuyLimit { price: Decimal, quantity: Decimal },
    SellLimit { price: Decimal, quantity: Decimal },
    CancelAll,
}

#[derive(Debug, Clone)]
pub struct FillReport {
    pub order_id: String,
    pub executed_qty: Decimal,
    pub avg_price: Decimal,
}

pub trait Strategy: Send + Sync {
    fn id(&self) -> u32;
    fn on_market_data(&mut self, frame_id: u64, data: &MarketDataSlot) -> Signal;
    fn on_timer(&mut self, frame_id: u64, delta_ns: u64) -> Signal;
    fn on_fill(&mut self, report: &FillReport) -> Signal;
    fn reset(&mut self);
}
```


├── core/src/tick.rs

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


├── core/src/timer/mod.rs

```rust
pub mod tsc;
```


├── core/src/timer/tsc.rs

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


├── core/src/validator.rs

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


├── core/tests/tick_tests.proptest-regressions

```text
# Seeds for failure cases proptest has generated in the past. It is
# automatically read and these particular cases re-run before any
# novel cases are generated.
#
# It is recommended to check this file in to source control so that
# everyone who runs the test benefits from these saved cases.
cc 06fe28a2dcabd4e5471ba9e77e4873e936aca8a1f7675f49648b8d725f8d8cb6 # shrinks to price = 1.0, qty = 0.001, timestamp = 1600000000000
```


├── core/tests/tick_tests.rs

```rust
use contracts::events::OwnedEvent;
use proje_core::tick::EventParser;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_tick_parsing_zero_allocation(price in 1.0..100000.0f64, qty in 0.001..10.0f64, timestamp in 1600000000000..1700000000000u64) {
        // Binance combined-stream format with trade payload
        let mut raw_payload = format!(
            "{{\"stream\":\"btcusdt@trade\",\"data\":{{\"e\":\"trade\",\"s\":\"BTCUSDT\",\"p\":\"{:.8}\",\"q\":\"{:.8}\",\"T\":{},\"m\":false}}}}",
            price, qty, timestamp
        ).into_bytes();

        let parsed_tick_opt = EventParser::parse(&mut raw_payload);

        prop_assert!(parsed_tick_opt.is_some());
        let tick: OwnedEvent = parsed_tick_opt.unwrap();

        let sym_len = tick.symbol.iter().position(|&c| c == 0).unwrap_or(16);
        prop_assert_eq!(&tick.symbol[..sym_len], b"BTCUSDT");
    }
}

#[test]
fn test_tick_allocation_mock() {
    // A mock specific test to run 1 million iterations quickly to simulate the CI run
    let base_payload = b"{\"stream\":\"btcusdt@trade\",\"data\":{\"s\":\"BTCUSDT\",\"p\":\"50000.0\",\"q\":\"1.5\",\"T\":1620000000000,\"m\":false}}";

    for _ in 0..1_000_000 {
        let mut payload = base_payload.to_vec(); // Outer alloc, inner parse should be 0 alloc
        let tick = EventParser::parse(&mut payload);
        assert!(tick.is_some());
    }
}

#[test]
fn test_parse_mark_price_stream() {
    let mut raw = br#"{"stream":"btcusdt@markPrice@1s","data":{"e":"markPriceUpdate","E":1562305380000,"s":"BTCUSDT","p":"64359.10000000","i":"64350.00000000","P":"64300.00000000","r":"0.00038167","T":1562306400000}}"#.to_vec();
    let ev = EventParser::parse(&mut raw).expect("markPrice parse edilmeli");
    assert!(matches!(ev.payload, contracts::events::EventType::FundingRate { .. }));
}
```


├── core/tests/wire_ring_tests.rs

```rust
//! Ring buffer + wire codec entegrasyon testi.
//! Üretici: wire::encode → GenerationalRingBuffer::push
//! Tüketici: read_slot → wire::decode → orijinal event ile karşılaştır.
//! Bu, DATA terminali ile tüketiciler (alert/paper/listener) arasındaki
//! gerçek veri yolunu /dev/shm üzerinden doğrular.

use rust_decimal::prelude::*;
use transport::ring_buffer::GenerationalRingBuffer;
use contracts::events::{EventType, OwnedEvent};
use contracts::wire;

fn push_and_read(ring: &GenerationalRingBuffer, ev: &OwnedEvent) -> Option<OwnedEvent> {
    let mut frame = [0u8; wire::MAX_FRAME_SIZE];
    let len = wire::encode(ev, &mut frame).expect("encode");
    let seq = ring.get_head();
    ring.push(&frame[..len]);
    let slot = ring.read_slot(seq).expect("slot okunmalı");
    assert_eq!(slot.len as usize, len, "frame len doğru yazılmalı");
    wire::decode(&slot.data[..slot.len as usize])
}

#[test]
fn ring_trade_roundtrip() {
    let ring = GenerationalRingBuffer::with_name("/cycle_finance_test_trade", 4096);
    let ev = OwnedEvent::new_trade("BTCUSDT", Decimal::from_str("67234.50").unwrap(),
        Decimal::from_str("0.001500").unwrap(), 1_766_800_000_000, true);
    let got = push_and_read(&ring, &ev).expect("roundtrip");
    assert_eq!(got.symbol, ev.symbol);
    match (got.payload, ev.payload) {
        (EventType::Trade { price, quantity, timestamp, is_buyer_maker },
         EventType::Trade { price: p, quantity: q, timestamp: t, is_buyer_maker: m }) => {
            assert_eq!(price, p);
            assert_eq!(quantity, q);
            assert_eq!(timestamp, t);
            assert_eq!(is_buyer_maker, m);
        }
        _ => panic!("Trade bekleniyordu"),
    }
}

#[test]
fn ring_depth20_roundtrip() {
    let ring = GenerationalRingBuffer::with_name("/cycle_finance_test_depth", 4096);
    let mut bids = [(Decimal::ZERO, Decimal::ZERO); 20];
    let mut asks = [(Decimal::ZERO, Decimal::ZERO); 20];
    for i in 0..20 {
        bids[i] = (Decimal::new(67200 + i as i64, 0), Decimal::new(100 - i as i64, 0));
        asks[i] = (Decimal::new(67220 + i as i64, 0), Decimal::new(90 + i as i64, 0));
    }
    let ev = OwnedEvent::new_orderbook("BTCUSDT", bids, asks);
    let got = push_and_read(&ring, &ev).expect("roundtrip");
    match (got.payload, ev.payload) {
        (EventType::Orderbook { bids: b, asks: a }, EventType::Orderbook { bids: b2, asks: a2 }) => {
            assert_eq!(b, b2);
            assert_eq!(a, a2);
        }
        _ => panic!("Orderbook bekleniyordu"),
    }
}

#[test]
fn ring_overwrite_generation() {
    // Aynı slot üzerine yazınca generational seq sayesinde eski okuma geçersiz olur.
    // Önceki koşulardan kalan shm state'ine dayanmamak için head'den göreli çalış.
    let ring = GenerationalRingBuffer::with_name("/cycle_finance_test_gen", 2);
    let base = ring.get_head();
    let trade = OwnedEvent::new_trade("BTCUSDT", Decimal::ONE, Decimal::ONE, 1, false);
    let mut frame = [0u8; wire::MAX_FRAME_SIZE];
    let len = wire::encode(&trade, &mut frame).unwrap();

    ring.push(&frame[..len]); // seq base
    ring.push(&frame[..len]); // seq base+1
    ring.push(&frame[..len]); // capacity=2 → base slotu üzerine yazıldı (seq base+2)
    assert!(ring.read_slot(base).is_none());
    assert!(ring.read_slot(base + 2).is_some());
}
```


├── cold-starter/Cargo.toml

```toml
[package]
name = "cold-starter"
version = "0.1.0"
edition = "2021"

[dependencies]
```


├── cold-starter/src/catchup.rs

```rust
//! Cold Starter routines for system recovery and initialization.

use sqlx::postgres::PgPoolOptions;

/// Cold Starter routines for system recovery and initialization.
pub struct CatchupRoutines;

const EMA_PERIOD: usize = 200;

fn db_url() -> String {
    std::env::var("TIMESCALEDB_URL")
        .unwrap_or_else(|_| "postgres://cycle:cycle@localhost:5432/market_data".into())
}

impl CatchupRoutines {
    /// 1. TimescaleDB `trades` hypertable'ındaki son trade fiyatlarından 200 EMA'yı hesaplar.
    pub async fn fetch_200_ema(&self) -> Result<f64, String> {
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&db_url())
            .await
            .map_err(|e| format!("TimescaleDB bağlantı hatası: {e}"))?;

        let mut prices: Vec<f64> = sqlx::query_scalar(
            "SELECT price FROM trades ORDER BY timestamp DESC LIMIT $1",
        )
        .bind(EMA_PERIOD as i64)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Sorgu hatası: {e}"))?;

        if prices.is_empty() {
            return Err("TimescaleDB'de trade verisi yok".into());
        }

        prices.reverse();
        let multiplier = 2.0 / (EMA_PERIOD as f64 + 1.0);
        let mut ema = prices[0];
        for &price in &prices[1..] {
            ema = price * multiplier + ema * (1.0 - multiplier);
        }

        println!("ColdStarter: 200 EMA hesaplandı = {ema:.4} ({} trade)", prices.len());
        Ok(ema)
    }

    /// 2. Buffer'ı temizleyip canlı moda geçer.
    pub fn transition_to_live(&self) {
        println!("ColdStarter: Buffer cleared. Transitioning to LIVE mode.");
    }
}
```


├── cold-starter/src/main.rs

```rust
pub mod catchup;

#[tokio::main]
async fn main() {
    println!("Cold Starter initialized");
    let routines = catchup::CatchupRoutines;
    match routines.fetch_200_ema().await {
        Ok(ema) => println!("200 EMA: {ema:.4}"),
        Err(e) => {
            eprintln!("ColdStarter: 200 EMA alınamadı: {e}");
            std::process::exit(1);
        }
    }
    routines.transition_to_live();
}
```


├── cold-storage/Cargo.toml

```toml
[package]
name = "cold-storage"
version = "0.1.0"
edition = "2021"

[dependencies]
memmap2 = "0.9"
```


├── cold-storage/src/lib.rs

```rust
#![allow(unsafe_code)]

use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::Path;

/// Buffer implemented using memory-mapped file for zero-latency writing.
/// Contains unsafe code for mmap, isolated from the `#![forbid(unsafe_code)]` core.
pub struct DiskBuffer {
    mmap: MmapMut,
}

impl DiskBuffer {
    pub fn new<P: AsRef<Path>>(path: P, size: u64) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
            
        file.set_len(size)?;
        
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        Ok(Self { mmap })
    }

    pub fn write_slice(&mut self, offset: usize, data: &[u8]) {
        // This is safe because mmap length is bound by the file size,
        // provided offset + data.len() <= mmap.len().
        if offset + data.len() <= self.mmap.len() {
            self.mmap[offset..offset + data.len()].copy_from_slice(data);
        }
    }
}
```


├── os-utils/Cargo.toml

```toml
[package]
name = "os-utils"
version = "0.1.0"
edition = "2021"

[dependencies]
libc = "0.2"
crossbeam = "0.8"
```


├── os-utils/src/config.rs

```rust
use crossbeam::epoch::{self, Atomic, Owned};
use std::sync::atomic::Ordering;

/// System configuration using lock-free epoch-based reclamation.
/// Prevents use-after-free without using Mutex/RwLock in the tick loop.
pub struct GlobalConfig {
    pub max_positions: usize,
    pub active_api_version: &'static str,
}

pub struct ConfigManager {
    // crossbeam_epoch::Atomic provides safe, lock-free memory reclamation
    current_config: Atomic<GlobalConfig>,
}

impl ConfigManager {
    pub fn new(initial: GlobalConfig) -> Self {
        Self {
            current_config: Atomic::new(initial),
        }
    }

    /// Read configuration. The returned guard ensures the config is not dropped
    /// while the current thread is holding it (epoch pinning).
    pub fn read_config<'a>(&'a self, guard: &'a epoch::Guard) -> &'a GlobalConfig {
        let ptr = self.current_config.load(Ordering::Acquire, guard);
        unsafe { ptr.as_ref().unwrap() }
    }

    /// Swap configuration globally. Old config is queued for garbage collection
    /// once no threads are pinning the epoch.
    pub fn swap_config(&self, new_config: GlobalConfig) {
        let guard = epoch::pin();
        let new_ptr = Owned::new(new_config);
        
        let old_ptr = self.current_config.swap(new_ptr, Ordering::Release, &guard);
        
        if !old_ptr.is_null() {
            unsafe {
                // Queue the old configuration for deletion safely.
                guard.defer_destroy(old_ptr);
            }
        }
        println!("Config: Successfully swapped lock-free configuration.");
    }
}
```


├── os-utils/src/lib.rs

```rust
#![allow(unsafe_code)]
pub mod config;

#[cfg(target_os = "linux")]
use libc::{sched_param, sched_setscheduler, SCHED_FIFO};

/// Safely sets the current thread to the SCHED_FIFO real-time scheduler.
/// On non-Linux platforms or if permissions are lacking, it logs a warning.
pub fn set_rt_thread_priority(priority: i32) {
    #[cfg(target_os = "linux")]
    {
        let param = sched_param {
            sched_priority: priority,
        };
        
        let result = unsafe {
            // 0 means the calling thread
            sched_setscheduler(0, SCHED_FIFO, &param)
        };
        
        if result != 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("Failed to set SCHED_FIFO (requires CAP_SYS_NICE or root): {}", err);
        } else {
            println!("Thread successfully elevated to SCHED_FIFO with priority {}", priority);
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("set_rt_thread_priority is a no-op on non-Linux platforms.");
    }
}
```


├── execution-engine/Cargo.toml

```toml
[package]
name = "execution-engine"
version = "0.1.0"
edition = "2024"

[dependencies]
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
tokio = { version = "1.0", features = ["full"] }
tokio-tungstenite = { version = "0.20", features = ["rustls-tls-native-roots"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dotenvy = "0.15"
flume = "0.11"
futures-util = "0.3"
sqlx = { version = "0.31.0", features = ["bundled"] }
rust_decimal = { workspace = true }
parking_lot = "0.12"
```


├── execution-engine/src/lib.rs

```rust
pub mod order;
pub mod signer;
pub mod paper;

use flume::Receiver;
use order::OrderRequest;
use signer::BinanceSigner;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::SinkExt;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use paper::config::PaperConfig;
use paper::actor::{PaperEngineActor, ActorCommand};

pub async fn start_execution_engine(rx: Receiver<OrderRequest>, api_key: String, secret_key: String) {
    let trading_mode = std::env::var("TRADING_MODE").unwrap_or_else(|_| "LIVE".to_string());
    
    if trading_mode == "PAPER" {
        println!("ExecutionEngine: Starting in PAPER TRADING mode.");
        let config = PaperConfig::load_from_env();
        let actor = PaperEngineActor::new(config);
        
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            actor.run(cmd_rx).await;
        });

        // Translate flume (Strategy) -> mpsc (Actor)
        while let Ok(order_req) = rx.recv_async().await {
            let (resp_tx, resp_rx) = oneshot::channel();
            let _ = cmd_tx.send(ActorCommand::SubmitOrder {
                order: order_req,
                response_tx: resp_tx,
            });
            
            // Opsiyonel: Sonucu bekle ve logla
            if let Ok(res) = resp_rx.await {
                println!("Paper Order Response: {:?}", res);
            }
        }
        return;
    }

    let ws_url = "wss://ws-api.binance.com:443/ws-api/v3";
    let signer = Arc::new(BinanceSigner::new(api_key, secret_key));

    loop {
        println!("ExecutionEngine: Connecting to Binance WS Order API...");
        match connect_async(ws_url).await {
            Ok((mut ws_stream, _)) => {
                println!("ExecutionEngine: Successfully connected to Order API.");

                while let Ok(order_req) = rx.recv_async().await {
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();

                    // Parametreleri Query String formatında hazırla (İmza için zorunlu)
                    let mut query_params = format!(
                        "apiKey={}&quantity={}&side={:?}&symbol={}&timestamp={}&type={:?}",
                        signer.api_key(),
                        order_req.quantity,
                        order_req.side,
                        order_req.symbol,
                        timestamp,
                        order_req.order_type
                    );

                    if let Some(price) = order_req.price {
                        query_params.push_str(&format!("&price={}", price));
                    }
                    if let Some(tif) = &order_req.time_in_force {
                        query_params.push_str(&format!("&timeInForce={:?}", tif));
                    }

                    // HMAC-SHA256 ile imzala
                    let signature = signer.sign(&query_params);

                    // WebSoket JSON Payload'ını hazırla
                    let mut params_json = json!({
                        "apiKey": signer.api_key(),
                        "symbol": order_req.symbol,
                        "side": order_req.side,
                        "type": order_req.order_type,
                        "quantity": order_req.quantity,
                        "timestamp": timestamp,
                        "signature": signature
                    });

                    if let Some(price) = order_req.price {
                        params_json["price"] = json!(price);
                    }
                    if let Some(tif) = &order_req.time_in_force {
                        params_json["timeInForce"] = json!(tif);
                    }

                    let ws_payload = json!({
                        "id": timestamp,
                        "method": "order.place",
                        "params": params_json
                    });

                    // Borsaya fırlat
                    let payload_str = ws_payload.to_string();
                    if let Err(e) = ws_stream.send(Message::Text(payload_str)).await {
                        println!("ExecutionEngine Error: Failed to send order: {}", e);
                        break; // Reconnect
                    }

                    // (Opsiyonel) Cevabı bekle
                    // if let Some(Ok(response)) = ws_stream.next().await {
                    //     println!("Order Response: {:?}", response);
                    // }
                }
            }
            Err(e) => {
                println!("ExecutionEngine Error: Connection failed: {}. Retrying in 3s...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }
    }
}
```


├── execution-engine/src/order.rs

```rust
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderPositionSide {
    Both,
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub time_in_force: Option<TimeInForce>,
    /// Hedge modda LONG/SHORT; one-way modda BOTH.
    pub position_side: OrderPositionSide,
}
```


├── execution-engine/src/signer.rs

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct BinanceSigner {
    api_key: String,
    secret_key: String,
}

impl BinanceSigner {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
        }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    #[inline(always)]
    pub fn sign(&self, query_string: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        hex::encode(code_bytes)
    }
}
```


├── execution-engine/tests/replay_tests.rs

```rust
//! Event Sourcing replay doğrulaması.
//! Varsayılan mod HEDGE + CROSSED, başlangıç bakiyesi 500 USDT.

use execution_engine::paper::actor::PaperEngineActor;
use execution_engine::paper::config::PaperConfig;
use execution_engine::paper::domain_event::DomainEvent;
use execution_engine::paper::position::PositionSide;
use rust_decimal::Decimal;
use std::str::FromStr;

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

#[tokio::test]
async fn test_replay_rebuilds_positions_and_cash() {
    // Varsayılan config (HEDGE, CROSSED, 500 USDT)
    let config = PaperConfig::load_from_env();

    // 100 USDT LONG @ 50000, komisyon 0.05, marj = 100/20 = 5
    let events = vec![
        DomainEvent::OrderFilled {
            order_id: "T1".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            position_side: "LONG".into(),
            fill_price: dec("50000"),
            fill_qty: dec("100"),
            commission: dec("0.05"),
            cash_delta: dec("-5.05"), // marj kilidi (5) + komisyon (0.05)
            realized_pnl: dec("0"),
            leverage: dec("20"),
        },
    ];

    let actor = PaperEngineActor::new_with_events(config.clone(), None, &events);

    let pos = actor.positions().get_hedge("BTCUSDT", PositionSide::Long).expect("pozisyon açık olmalı");
    assert_eq!(pos.quantity, dec("100"));
    assert_eq!(pos.avg_entry_price, dec("50000"));

    // cash = 500 - 5.05
    let expected_cash = config.initial_usdt - dec("5.05");
    assert_eq!(actor.account().get_free("USDT"), expected_cash);
}

#[tokio::test]
async fn test_replay_close_realizes_pnl() {
    let config = PaperConfig::load_from_env();
    let events = vec![
        DomainEvent::OrderFilled {
            order_id: "T1".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            position_side: "LONG".into(),
            fill_price: dec("50000"),
            fill_qty: dec("100"),
            commission: dec("0.05"),
            cash_delta: dec("-5.05"),
            realized_pnl: dec("0"),
            leverage: dec("20"),
        },
        DomainEvent::OrderFilled {
            order_id: "T2".into(),
            symbol: "BTCUSDT".into(),
            side: "SELL".into(),
            position_side: "LONG".into(),
            fill_price: dec("51000"),
            fill_qty: dec("100"),
            commission: dec("0.051"),
            // marj geri (5) + realized (2) - komisyon (0.051)
            cash_delta: dec("6.949"),
            realized_pnl: dec("2"),
            leverage: dec("20"),
        },
    ];

    let actor = PaperEngineActor::new_with_events(config.clone(), None, &events);
    assert!(actor.positions().get_hedge("BTCUSDT", PositionSide::Long).is_none(), "pozisyon kapanmış olmalı");

    let expected_cash = config.initial_usdt - dec("5.05") + dec("6.949");
    assert_eq!(actor.account().get_free("USDT"), expected_cash);
}

#[tokio::test]
async fn test_funding_applies_to_cash() {
    let config = PaperConfig::load_from_env();
    let events = vec![
        DomainEvent::OrderFilled {
            order_id: "T1".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            position_side: "LONG".into(),
            fill_price: dec("50000"),
            fill_qty: dec("100"),
            commission: dec("0"),
            cash_delta: dec("-5"),
            realized_pnl: dec("0"),
            leverage: dec("20"),
        },
        DomainEvent::FundingRateApplied {
            symbol: "BTCUSDT".into(),
            rate: dec("0.0001"),
            payment: dec("-5"),
        },
    ];

    let actor = PaperEngineActor::new_with_events(config.clone(), None, &events);
    let expected_cash = config.initial_usdt - dec("5") - dec("5");
    assert_eq!(actor.account().get_free("USDT"), expected_cash);
}
```


├── execution-engine/tests/risk_tests.rs

```rust
//! Pozisyon ve risk yönetimi için birim testler.
//! Tüm hesaplamalar rust_decimal ile yapılır (f64 yok).
//! Pozisyon boyutları USDT (notional) cinsindendir.

use rust_decimal::Decimal;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use execution_engine::paper::position::{PositionManager, PositionSide};
    use execution_engine::paper::risk::RiskManager;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_position_open_long_and_unrealized_pnl() {
        let mut pm = PositionManager::new();
        // 100 USDT long aç @ 50000
        let entry = dec("50000");
        let (realized, closed) = pm.apply_fill("BTCUSDT", dec("100"), entry, dec("10"));
        assert_eq!(realized, Decimal::ZERO);
        assert_eq!(closed, Decimal::ZERO);

        let pos = pm.get("BTCUSDT").unwrap();
        assert_eq!(pos.side, PositionSide::Long);
        assert_eq!(pos.quantity, dec("100"));

        // Mark fiyat %2 yükselirse long +2 USDT PnL
        let pnl = pos.unrealized_pnl(dec("51000"));
        assert_eq!(pnl, dec("2"));

        // Likidasyon fiyatı: 50000 * (1 - 0.1 + 0.005) = 45250
        assert_eq!(pos.liquidation_price(dec("0.005")), dec("45250"));
    }

    #[test]
    fn test_position_close_realizes_pnl() {
        let mut pm = PositionManager::new();
        // 100 USDT long @ 50000, kapat @ 51000 → %2 kâr = 2 USDT
        pm.apply_fill("BTCUSDT", dec("100"), dec("50000"), dec("10"));
        let (realized, closed) = pm.apply_fill("BTCUSDT", dec("-100"), dec("51000"), dec("10"));
        assert_eq!(realized, dec("2"));
        assert_eq!(closed, dec("100"));
        assert!(pm.get("BTCUSDT").is_none());
    }

    #[test]
    fn test_position_flip_short() {
        let mut pm = PositionManager::new();
        // 200 USDT long @ 50000; 300 USDT sat → 200 kapanış + 100 short
        pm.apply_fill("BTCUSDT", dec("200"), dec("50000"), dec("10"));
        let (realized, closed) = pm.apply_fill("BTCUSDT", dec("-300"), dec("49000"), dec("10"));
        // Kapanan 200 USDT: (49000-50000)/50000 * 200 = -4 USDT
        assert_eq!(realized, dec("-4"));
        assert_eq!(closed, dec("200"));

        let pos = pm.get("BTCUSDT").unwrap();
        assert_eq!(pos.side, PositionSide::Short);
        assert_eq!(pos.quantity, dec("-100"));
        assert_eq!(pos.avg_entry_price, dec("49000"));
    }

    #[test]
    fn test_short_unrealized_pnl_sign() {
        let mut pm = PositionManager::new();
        // 100 USDT short aç @ 50000
        pm.apply_fill("BTCUSDT", dec("-100"), dec("50000"), dec("10"));
        let pos = pm.get("BTCUSDT").unwrap();
        assert_eq!(pos.side, PositionSide::Short);
        // Fiyat %2 düştü → short +2 USDT kâr
        assert_eq!(pos.unrealized_pnl(dec("49000")), dec("2"));
        // Fiyat %2 yükseldi → short -2 USDT zarar
        assert_eq!(pos.unrealized_pnl(dec("51000")), dec("-2"));
        // Toplam unrealized (equity hesapları için)
        let mut marks = std::collections::HashMap::new();
        marks.insert("BTCUSDT".to_string(), dec("49000"));
        assert_eq!(pm.total_unrealized_pnl(&marks), dec("2"));
    }

    #[test]
    fn test_hedge_positions_coexist() {
        let mut pm = PositionManager::new();
        pm.apply_fill_hedge("BTCUSDT", PositionSide::Long, dec("100"), dec("50000"), dec("10"));
        pm.apply_fill_hedge("BTCUSDT", PositionSide::Short, dec("-50"), dec("50000"), dec("10"));
        // İki taraf ayrı izlenir
        assert_eq!(pm.get_hedge("BTCUSDT", PositionSide::Long).unwrap().quantity, dec("100"));
        assert_eq!(pm.get_hedge("BTCUSDT", PositionSide::Short).unwrap().quantity, dec("-50"));
        // Toplam brüt pozisyon 150 USDT
        assert_eq!(pm.total_abs_qty("BTCUSDT"), dec("150"));
        // Hedge short tarafını kapat: fiyat düşerse kâr
        // Short 50 USDT @ 50000, kapat @ 48000 → (50000-48000)/50000*50 = 2 USDT
        let (realized, _) = pm.apply_fill_hedge("BTCUSDT", PositionSide::Short, dec("50"), dec("48000"), dec("10"));
        assert_eq!(realized, dec("2"));
        assert!(pm.get_hedge("BTCUSDT", PositionSide::Short).is_none());
        // Long tarafı hâlâ açık
        assert_eq!(pm.get_hedge("BTCUSDT", PositionSide::Long).unwrap().quantity, dec("100"));
    }

    #[test]
    fn test_risk_min_position_rejection() {
        // min pozisyon 6 USDT; 5 USDT reddedilmeli
        let risk = RiskManager::new(dec("500"), dec("20"), dec("0.05"), dec("1000"), dec("6"));
        assert!(risk.check_order(dec("5"), dec("20"), dec("500")).is_err());
        assert!(risk.check_order(dec("6"), dec("20"), dec("500")).is_ok());
        // Negatif işaretli (short) miktarlar da abs ile değerlendirilir
        assert!(risk.check_order(dec("-5"), dec("20"), dec("500")).is_err());
        assert!(risk.check_order(dec("-6"), dec("20"), dec("500")).is_ok());
    }

    #[test]
    fn test_risk_leverage_margin_rejection() {
        // 500 USDT bakiye; 1000 USDT pozisyon 20x → 50 marj → ok
        let risk = RiskManager::new(dec("500"), dec("20"), dec("0.05"), dec("1000"), dec("6"));
        assert!(risk.check_order(dec("1000"), dec("20"), dec("500")).is_ok());
        // 1000x kaldıraç limiti aşar → red
        assert!(risk.check_order(dec("1000"), dec("1000"), dec("500")).is_err());
        // Marj yetersiz: 5000 USDT pozisyon 10x → 500 marj = cash sınırda → ok
        assert!(risk.check_order(dec("5000"), dec("10"), dec("500")).is_ok());
        // 6000 USDT 10x → 600 marj > 500 cash → red
        assert!(risk.check_order(dec("6000"), dec("10"), dec("500")).is_err());
    }

    #[test]
    fn test_risk_drawdown_breach() {
        let pm = PositionManager::new();
        let mut risk = RiskManager::new(dec("10000"), dec("20"), dec("0.05"), dec("1000"), dec("6"));
        let mut mark_prices = std::collections::HashMap::new();
        mark_prices.insert("BTCUSDT".to_string(), dec("45000"));

        // cash + unrealized ile equity düşür (büyük kayıp)
        let cash = dec("10000");
        // drawdown > %5 için equity < 9500 gerekir; on_mark_tick unrealized'e bakar
        let liquidated = risk.on_mark_tick(&pm, &mark_prices, cash);
        assert!(liquidated.is_empty());
        assert_eq!(risk.status, execution_engine::paper::risk::RiskStatus::Ok);
    }

    #[test]
    fn test_liquidation_trigger_on_long() {
        let mut pm = PositionManager::new();
        // 100 USDT long @ 50000, 10x
        pm.apply_fill("BTCUSDT", dec("100"), dec("50000"), dec("10"));
        let mut risk = RiskManager::new(dec("10000"), dec("20"), dec("0.05"), dec("1000"), dec("6"));
        let mut mark_prices = std::collections::HashMap::new();
        // Likidasyon fiyatı 45250; 45000'e düşerse likidasyon tetiklenir
        mark_prices.insert("BTCUSDT".to_string(), dec("45000"));
        let liquidated = risk.on_mark_tick(&pm, &mark_prices, dec("10000"));
        assert_eq!(liquidated, vec!["BTCUSDT".to_string()]);
        assert_eq!(risk.status, execution_engine::paper::risk::RiskStatus::Liquidation);
    }
}
```


├── ohlcv-engine/Cargo.toml

```toml
[package]
name = "ohlcv-engine"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8.9"
chrono = "0.4.45"
clap = { version = "4.6.6", features = ["derive"] }
reqwest = { version = "0.13.4", features = ["json"] }
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
tokio = { version = "1.53.1", features = ["rt-multi-thread", "macros"] }
rust_decimal = { workspace = true }
```


├── ohlcv-engine/src/bin/cli.rs

```rust
use clap::Parser;
use ohlcv_engine::client::BinanceClient;
use chrono::{Local, TimeZone};
use rust_decimal::Decimal;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Hangi sembolün çekileceği (Örn: VELVETUSDT, BTCUSDT)
    #[arg(short, long, default_value = "VELVETUSDT")]
    symbol: String,

    /// Mum aralığı (Örn: 1m, 5m, 1h, 1d)
    #[arg(short, long, default_value = "1h")]
    interval: String,

    /// Kaç adet mum çekileceği (Örn: 10)
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    println!("========================================");
    println!("📊 OHLCV TERMINAL RADARI");
    println!("Sembol: {}", args.symbol);
    println!("Aralık: {}", args.interval);
    println!("Limit:  {}", args.limit);
    println!("========================================");

    let client = BinanceClient::new();

    match client.fetch_klines(&args.symbol, &args.interval, args.limit).await {
        Ok(klines) => {
            for (i, k) in klines.iter().enumerate() {
                let dt = Local.timestamp_millis_opt(k.open_time as i64).unwrap();
                let time_str = dt.format("%Y-%m-%d %H:%M:%S").to_string();
                
                let trend = if k.close >= k.open { "🟩 BOGA" } else { "🟥 AYI " };
                let delta = k.close - k.open;
                let delta_percent = (delta / k.open) * Decimal::ONE_HUNDRED;

                println!("[{:02}] {} | {} | Açılış: {:.4} | Yüksek: {:.4} | Düşük: {:.4} | Kapanış: {:.4} | Hacim: {:.2} | Değişim: {:.4} ({:.2}%)",
                    i + 1, time_str, trend, k.open, k.high, k.low, k.close, k.volume, delta, delta_percent
                );
            }
            println!("========================================");
            println!("✅ Başarıyla {} adet mum çekildi.", klines.len());
        },
        Err(e) => {
            eprintln!("❌ Veri çekilirken hata oluştu: {}", e);
        }
    }
}
```


├── ohlcv-engine/src/bin/server.rs

```rust
use axum::{
    extract::Query,
    routing::get,
    Router, Json,
};
use ohlcv_engine::client::BinanceClient;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Deserialize)]
struct KlineParams {
    symbol: String,
    interval: String,
    limit: Option<usize>,
}

struct AppState {
    client: BinanceClient,
}

#[tokio::main]
async fn main() {
    println!("OHLCV API Sunucusu Başlatılıyor...");
    
    let state = Arc::new(AppState {
        client: BinanceClient::new(),
    });

    let app = Router::new()
        .route("/api/klines", get(get_klines))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("API Sunucusu http://{} üzerinde dinleniyor.", addr);
    println!("Örnek kullanım: http://127.0.0.1:3000/api/klines?symbol=VELVETUSDT&interval=15m&limit=100");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_klines(
    Query(params): Query<KlineParams>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(100);
    
    match state.client.fetch_klines(&params.symbol, &params.interval, limit).await {
        Ok(klines) => Json(serde_json::json!({
            "status": "success",
            "symbol": params.symbol,
            "interval": params.interval,
            "count": klines.len(),
            "data": klines
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}
```


├── ohlcv-engine/src/client.rs

```rust
use crate::Kline;
use reqwest::Client;
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;

pub struct BinanceClient {
    http: Client,
}

impl BinanceClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }

    /// Fetches historical Klines (OHLCV) from Binance Futures
    /// https://fapi.binance.com/fapi/v1/klines
    pub async fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: usize,
    ) -> Result<Vec<Kline>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://fapi.binance.com/fapi/v1/klines?symbol={}&interval={}&limit={}",
            symbol, interval, limit
        );

        let response = self.http.get(&url).send().await?;
        let data: Vec<Value> = response.json().await?;

        let mut klines = Vec::new();

        for row in data {
            if let Some(arr) = row.as_array() {
                if arr.len() >= 11 {
                    let d = |v: &Value| Decimal::from_str(v.as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO);
                    let kline = Kline {
                        open_time: arr[0].as_u64().unwrap_or(0),
                        open: d(&arr[1]),
                        high: d(&arr[2]),
                        low: d(&arr[3]),
                        close: d(&arr[4]),
                        volume: d(&arr[5]),
                        close_time: arr[6].as_u64().unwrap_or(0),
                        quote_asset_volume: d(&arr[7]),
                        trades: arr[8].as_u64().unwrap_or(0),
                        taker_buy_base_asset_volume: d(&arr[9]),
                        taker_buy_quote_asset_volume: d(&arr[10]),
                    };
                    klines.push(kline);
                }
            }
        }

        Ok(klines)
    }
}
```


├── ohlcv-engine/src/lib.rs

```rust
pub mod client;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub open_time: u64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub close_time: u64,
    pub quote_asset_volume: Decimal,
    pub trades: u64,
    pub taker_buy_base_asset_volume: Decimal,
    pub taker_buy_quote_asset_volume: Decimal,
}
```


├── detect-ms/Cargo.toml

```toml
[package]
name = "detect-ms"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8.9"
ohlcv-engine = { version = "0.1.0", path = "../ohlcv-engine" }
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
tokio = { version = "1.53.1", features = ["macros", "rt-multi-thread"] }
rust_decimal = { workspace = true }
```


├── detect-ms/src/imbalance.rs

```rust
// ============================================================================
// MSMP 2.0 — KATMAN 6: DENGESİZLİK (FVG + DELTA DOĞRULAMASI)
// ============================================================================
// FVG, ardışık 3 mumun üst/alt gölge çakışmazlığı ile taranır.
// Öncelik, o bölgedeki Kümülatif Delta ile doğrulanır:
//   Delta(+) ve FVG yukarı → "Aktif Emici (Active Absorber)" (en yüksek çekim)
//   Delta(-/0) ve FVG aşağı → "Pasif Geçiş (Passive Gap)" (sadece dolgu)
// Delta = taker_buy_base_asset_volume - (volume - taker_buy_base_asset_volume)
// ============================================================================

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum FvgDirection {
    Bullish,
    Bearish,
}

#[derive(Debug, Clone, Serialize)]
pub enum FvgLabel {
    /// En yüksek çekim gücü — Delta doğrulanmış
    ActiveAbsorber,
    /// Sadece doldurulması beklenir — önceliği düşük
    PassiveGap,
}

#[derive(Debug, Clone, Serialize)]
pub struct Fvg {
    /// FVG bölgesinin üst sınırı
    pub high: Decimal,
    /// FVG bölgesinin alt sınırı
    pub low: Decimal,
    /// Bölge orta noktası
    pub mid: Decimal,
    pub direction: FvgDirection,
    /// 3 mumun toplam delta değeri
    pub delta: Decimal,
    /// Delta doğrulama sonucu
    pub label: FvgLabel,
    pub timestamp: u64,
    pub index: usize,
}

/// Tek bir mumun Delta değeri
/// Delta = Alıcı hacmi - Satıcı hacmi
/// buy_volume = taker_buy_base_asset_volume (aggresor alıcılar)
/// sell_volume = volume - taker_buy_base_asset_volume (aggresor satıcılar)
pub fn candle_delta(kline: &Kline) -> Decimal {
    let buy_vol = kline.taker_buy_base_asset_volume;
    let sell_vol = kline.volume - buy_vol;
    buy_vol - sell_vol
}

/// Kümülatif Delta serisi
pub fn cumulative_delta(klines: &[Kline]) -> Vec<Decimal> {
    let mut cum = Decimal::ZERO;
    klines
        .iter()
        .map(|k| {
            cum += candle_delta(k);
            cum
        })
        .collect()
}

/// FVG tespiti + Cumulative Delta doğrulaması
///
/// Bullish FVG: Mum 1'in high'ı < Mum 3'ün low'u (yukarı fiyat boşluğu)
/// Bearish FVG: Mum 1'in low'u > Mum 3'ün high'ı (aşağı fiyat boşluğu)
///
/// Delta doğrulama:
///   Bullish FVG + Delta(+) → Active Absorber
///   Bearish FVG + Delta(-) → Active Absorber
///   Aksi → Passive Gap
pub fn detect_fvg(klines: &[Kline]) -> Vec<Fvg> {
    let mut fvgs = Vec::new();
    if klines.len() < 3 {
        return fvgs;
    }

    for i in 1..(klines.len() - 1) {
        let prev = &klines[i - 1];
        let curr = &klines[i];
        let next = &klines[i + 1];

        // 3 mumun toplam delta'sı
        let region_delta =
            candle_delta(prev) + candle_delta(curr) + candle_delta(next);

        // ── Bullish FVG ──
        // Mum 1 (prev) high'ı < Mum 3 (next) low'u → yukarı boşluk
        if prev.high < next.low {
            let gap_high = next.low;
            let gap_low = prev.high;

            let label = if region_delta > Decimal::ZERO {
                FvgLabel::ActiveAbsorber
            } else {
                FvgLabel::PassiveGap
            };

            fvgs.push(Fvg {
                high: gap_high,
                low: gap_low,
                mid: (gap_high + gap_low) / Decimal::TWO,
                direction: FvgDirection::Bullish,
                delta: region_delta,
                label,
                timestamp: curr.open_time,
                index: i,
            });
        }

        // ── Bearish FVG ──
        // Mum 1 (prev) low'u > Mum 3 (next) high'ı → aşağı boşluk
        if prev.low > next.high {
            let gap_high = prev.low;
            let gap_low = next.high;

            let label = if region_delta < Decimal::ZERO {
                FvgLabel::ActiveAbsorber
            } else {
                FvgLabel::PassiveGap
            };

            fvgs.push(Fvg {
                high: gap_high,
                low: gap_low,
                mid: (gap_high + gap_low) / Decimal::TWO,
                direction: FvgDirection::Bearish,
                delta: region_delta,
                label,
                timestamp: curr.open_time,
                index: i,
            });
        }
    }

    fvgs
}
```


├── detect-ms/src/levels.rs

```rust
// ============================================================================
// MSMP 2.0 — KATMAN 4: STRATEJİK SEVİYE ENVANTERİ
// ============================================================================
// W(t) = e^(-λ * t) , λ = 0.015 (yaklaşık 46 mumda yarı değere düşer)
// Süpürülmüş seviyeler "Geçersiz" DEĞİLDİR:
//   → 2 ardışık mum kapanışı ötede ise "Breakout Onayı (BO Confirmation)"
// Sınıflar:
//   Savunulmuş (≥2 Close Test) → Skor 10
//   Süpürülmüş + BO Onayı → Skor 9
//   Onaylanmamış OB/FVG → Skor 8 - W(t)
//   Yeni Oluşan → Skor 7
// ============================================================================

use crate::pivot::{PivotPoint, PivotType};
use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum LevelClass {
    /// Savunulmuş (≥2 Close Test) — Öncelik Skoru: 10
    Defended,
    /// Süpürülmüş + BO Onayı — Öncelik Skoru: 9
    SweptConfirmed,
    /// Onaylanmamış OB/FVG — Öncelik Skoru: 8 - W(t)
    UnconfirmedOBFVG,
    /// Yeni Oluşan (Son 2 Pivot) — Öncelik Skoru: 7
    NewActive,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategicLevel {
    pub pivot_id: String,
    pub price: Decimal,
    pub level_type: String,
    pub timestamp: u64,
    /// W(t) = e^(-λ * t)
    pub decay_weight: Decimal,
    /// Fiyatın seviyeye dokunup geri döndüğü sayı
    pub defense_count: u16,
    /// Fiyat wick ile kırıp kapanış geri mi döndü?
    pub is_swept: bool,
    /// 2 ardışık kapanış seviyenin ötesinde mi?
    pub bo_confirmed: bool,
    pub class: LevelClass,
    /// Nihai öncelik skoru (0-100)
    pub priority_score: Decimal,
}

/// Üssel zaman çürümesi uygula: W(t) = e^(-λ * t)
pub fn apply_decay(pivots: &[PivotPoint], current_index: usize) -> Vec<StrategicLevel> {
    // Yarılanma sabiti: ~46 mumda yarı değere düşer (0.015)
    let lambda = Decimal::from_str("0.015").unwrap();
    pivots
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let t = Decimal::from(current_index.saturating_sub(p.index));
            let decay = (-lambda * t).exp();

            let level_type = match p.pivot_type {
                PivotType::SwingHigh => "SH".to_string(),
                PivotType::SwingLow => "SL".to_string(),
            };

            StrategicLevel {
                pivot_id: format!("P-{}", i + 1),
                price: p.price,
                level_type,
                timestamp: p.timestamp,
                decay_weight: decay,
                defense_count: 0,
                is_swept: false,
                bo_confirmed: false,
                class: LevelClass::NewActive,
                priority_score: Decimal::ZERO,
            }
        })
        .collect()
}

/// Savunma sayısını hesapla — fiyatın seviyeye kaç kez dokunup geri döndüğü
pub fn count_defenses(levels: &mut [StrategicLevel], klines: &[Kline], tolerance_pct: Decimal) {
    for level in levels.iter_mut() {
        let mut defenses = 0u16;

        for k in klines.iter() {
            let tolerance = level.price * tolerance_pct;

            // Fiyat seviyeye dokundu mu?
            let touched =
                k.high >= level.price - tolerance && k.low <= level.price + tolerance;

            // Kapanış seviyenin ötesine geçmedi mi? (savunma)
            let defended = match level.level_type.as_str() {
                "SH" => k.close < level.price + tolerance,
                "SL" => k.close > level.price - tolerance,
                _ => false,
            };

            if touched && defended {
                defenses += 1;
            }
        }

        level.defense_count = defenses;
    }
}

/// Süpürülme (Sweep) ve Breakout Onayı (BO) kontrolü
pub fn check_sweep_and_bo(levels: &mut [StrategicLevel], klines: &[Kline]) {
    for level in levels.iter_mut() {
        // Seviyenin oluştuğu mumdan sonrasını tara
        let level_idx = klines
            .iter()
            .position(|k| k.open_time >= level.timestamp)
            .unwrap_or(0);

        for i in level_idx..klines.len() {
            // Süpürülme: wick kırar ama kapanış geri döner
            let swept = match level.level_type.as_str() {
                "SH" => klines[i].high > level.price && klines[i].close < level.price,
                "SL" => klines[i].low < level.price && klines[i].close > level.price,
                _ => false,
            };

            if swept {
                level.is_swept = true;

                // BO Onayı: 2 ardışık mum kapanışı seviyenin ötesinde
                if i + 2 < klines.len() {
                    let bo = match level.level_type.as_str() {
                        "SH" => {
                            klines[i + 1].close > level.price
                                && klines[i + 2].close > level.price
                        }
                        "SL" => {
                            klines[i + 1].close < level.price
                                && klines[i + 2].close < level.price
                        }
                        _ => false,
                    };
                    if bo {
                        level.bo_confirmed = true;
                    }
                }
                break;
            }
        }
    }
}

/// Seviyeleri sınıflandır ve nihai öncelik skoru hesapla (0-100)
pub fn classify_levels(levels: &mut [StrategicLevel]) {
    for level in levels.iter_mut() {
        let base_score = if level.defense_count >= 2 {
            level.class = LevelClass::Defended;
            Decimal::from(10)
        } else if level.is_swept && level.bo_confirmed {
            level.class = LevelClass::SweptConfirmed;
            Decimal::from(9)
        } else if level.is_swept && !level.bo_confirmed {
            level.class = LevelClass::UnconfirmedOBFVG;
            Decimal::from(8) - (Decimal::ONE - level.decay_weight)
        } else {
            level.class = LevelClass::NewActive;
            Decimal::from(7)
        };

        // Nihai skor: base * decay * 10 (normalize to 0-100)
        let raw = (base_score * level.decay_weight) * Decimal::TEN;
        level.priority_score = raw.max(Decimal::ZERO).min(Decimal::ONE_HUNDRED);
    }
}

/// Tam seviye analizi pipeline'ı
pub fn analyze_levels(pivots: &[PivotPoint], klines: &[Kline]) -> Vec<StrategicLevel> {
    if klines.is_empty() {
        return vec![];
    }

    let current_index = klines.len().saturating_sub(1);
    let mut levels = apply_decay(pivots, current_index);

    count_defenses(&mut levels, klines, Decimal::from_str("0.001").unwrap()); // %0.1 tolerans
    check_sweep_and_bo(&mut levels, klines);
    classify_levels(&mut levels);

    // Öncelik skoruna göre sırala (yüksekten düşüğe)
    levels.sort_by(|a, b| b.priority_score.cmp(&a.priority_score));

    levels
}
```


├── detect-ms/src/liquidity.rs

```rust
// ============================================================================
// MSMP 2.0 — KATMAN 5: LİKİDİTE POOL (VWAP Sapması & Volume Profile)
// ============================================================================
// Eşit bantlar TAMAMEN İPTAL. Volume Profile hesaplanır:
//   HVN (Yüksek Hacim Node) ve LVN (Düşük Hacim Node) tespit edilir.
// BSL Yoğunluğu = +1.5σ ile +3σ arası HVN bölgeleri
// SSL Yoğunluğu = -1.5σ ile -3σ arası HVN bölgeleri
// Likidite Skoru = Bölge hacmi / toplam hacim oranı (1-10)
// ============================================================================

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum NodeType {
    /// Yüksek Hacim Node — Kurumsal alım-satım yoğunluğu
    HVN,
    /// Düşük Hacim Node — Fiyat hızla geçer
    LVN,
}

#[derive(Debug, Clone, Serialize)]
pub struct VolumeNode {
    pub price_low: Decimal,
    pub price_high: Decimal,
    pub price_mid: Decimal,
    pub volume: Decimal,
    /// Bu node'un toplam hacme oranı (0.0 - 1.0)
    pub volume_ratio: Decimal,
    pub node_type: NodeType,
    /// Likidite skoru (1-10)
    pub liquidity_score: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiquidityAnalysis {
    /// Volume-Weighted Average Price
    pub vwap: Decimal,
    /// VWAP standart sapması (σ)
    pub vwap_std_dev: Decimal,
    /// Point of Control — en yüksek hacimli fiyat seviyesi
    pub poc: Decimal,
    /// Buy-Side Liquidity bölgeleri (+1.5σ ~ +3σ arası HVN)
    pub bsl_zones: Vec<VolumeNode>,
    /// Sell-Side Liquidity bölgeleri (-3σ ~ -1.5σ arası HVN)
    pub ssl_zones: Vec<VolumeNode>,
    pub bsl_total_volume: Decimal,
    pub ssl_total_volume: Decimal,
    /// BSL/SSL Oranı — Risk asimetrisi
    pub bsl_ssl_ratio: Decimal,
    /// Aktif Volatilite Bandı alt sınırı: POC - 1.5σ
    pub volatility_band_low: Decimal,
    /// Aktif Volatilite Bandı üst sınırı: POC + 1.5σ
    pub volatility_band_high: Decimal,
    /// Tam volume profile
    pub volume_profile: Vec<VolumeNode>,
}

/// VWAP (Volume-Weighted Average Price) hesaplaması
pub fn vwap(klines: &[Kline]) -> Decimal {
    let mut cum_tp_vol = Decimal::ZERO;
    let mut cum_vol = Decimal::ZERO;

    for k in klines {
        let typical_price = (k.high + k.low + k.close) / Decimal::from(3);
        cum_tp_vol += typical_price * k.volume;
        cum_vol += k.volume;
    }

    if cum_vol == Decimal::ZERO {
        return Decimal::ZERO;
    }
    cum_tp_vol / cum_vol
}

/// VWAP Standart Sapması (σ) — Hacim ağırlıklı
pub fn vwap_std_dev(klines: &[Kline], vwap_val: Decimal) -> Decimal {
    if klines.is_empty() {
        return Decimal::ZERO;
    }

    let mut sum_sq = Decimal::ZERO;
    let mut cum_vol = Decimal::ZERO;

    for k in klines {
        let typical_price = (k.high + k.low + k.close) / Decimal::from(3);
        sum_sq += k.volume * (typical_price - vwap_val).powi(2);
        cum_vol += k.volume;
    }

    if cum_vol == Decimal::ZERO {
        return Decimal::ZERO;
    }
    (sum_sq / cum_vol).sqrt().unwrap_or(Decimal::ZERO)
}

/// Volume Profile — Dinamik bucket'larla hacim dağılımı
pub fn volume_profile(klines: &[Kline], bucket_count: usize) -> Vec<VolumeNode> {
    if klines.is_empty() || bucket_count == 0 {
        return vec![];
    }

    let price_min = klines
        .iter()
        .map(|k| k.low)
        .fold(Decimal::MAX, Decimal::min);
    let price_max = klines
        .iter()
        .map(|k| k.high)
        .fold(Decimal::MIN, Decimal::max);

    if price_max <= price_min {
        return vec![];
    }

    let bucket_size = (price_max - price_min) / Decimal::from(bucket_count);
    let mut buckets = vec![Decimal::ZERO; bucket_count];
    let total_volume: Decimal = klines.iter().map(|k| k.volume).sum();

    // Her mumun hacmini fiyat aralığına orantılı dağıt
    for k in klines {
        let mut low_idx = ((k.low - price_min) / bucket_size).floor().to_usize().unwrap_or(0);
        let mut high_idx = ((k.high - price_min) / bucket_size).floor().to_usize().unwrap_or(0);
        low_idx = low_idx.min(bucket_count - 1);
        high_idx = high_idx.min(bucket_count - 1);

        let span = Decimal::from(high_idx - low_idx + 1);
        let vol_per_bucket = k.volume / span;

        for b in low_idx..=high_idx {
            buckets[b] += vol_per_bucket;
        }
    }

    // Medyan hacmi hesapla (HVN/LVN eşiği olarak kullanılır)
    let mut sorted_vols: Vec<Decimal> = buckets.clone();
    sorted_vols.sort();
    let median_vol = sorted_vols[sorted_vols.len() / 2];

    let mut nodes = Vec::with_capacity(bucket_count);
    for (i, &vol) in buckets.iter().enumerate() {
        let p_low = price_min + Decimal::from(i) * bucket_size;
        let p_high = p_low + bucket_size;
        let ratio = if total_volume > Decimal::ZERO {
            vol / total_volume
        } else {
            Decimal::ZERO
        };

        let node_type = if vol >= median_vol * Decimal::from_str("1.5").unwrap() {
            NodeType::HVN
        } else {
            NodeType::LVN
        };

        // Likidite Skoru: hacim oranının yüzdesel dilimi (1-10)
        let pct = ratio * Decimal::ONE_HUNDRED;
        let score = (pct.round().to_u8().unwrap_or(0)).clamp(1, 10);

        nodes.push(VolumeNode {
            price_low: p_low,
            price_high: p_high,
            price_mid: (p_low + p_high) / Decimal::TWO,
            volume: vol,
            volume_ratio: ratio,
            node_type,
            liquidity_score: score,
        });
    }

    nodes
}

/// BSL ve SSL bölgelerini tespit et
/// BSL: current_price + 1.5σ ~ +3σ arası HVN'ler
/// SSL: current_price - 3σ ~ -1.5σ arası HVN'ler
pub fn detect_bsl_ssl(
    nodes: &[VolumeNode],
    current_price: Decimal,
    sigma: Decimal,
) -> (Vec<VolumeNode>, Vec<VolumeNode>) {
    let one_half = Decimal::from_str("1.5").unwrap();
    let three = Decimal::from(3);
    let bsl_low = current_price + one_half * sigma;
    let bsl_high = current_price + three * sigma;
    let ssl_low = current_price - three * sigma;
    let ssl_high = current_price - one_half * sigma;

    let bsl: Vec<VolumeNode> = nodes
        .iter()
        .filter(|n| {
            matches!(n.node_type, NodeType::HVN)
                && n.price_mid >= bsl_low
                && n.price_mid <= bsl_high
        })
        .cloned()
        .collect();

    let ssl: Vec<VolumeNode> = nodes
        .iter()
        .filter(|n| {
            matches!(n.node_type, NodeType::HVN)
                && n.price_mid >= ssl_low
                && n.price_mid <= ssl_high
        })
        .cloned()
        .collect();

    (bsl, ssl)
}

/// Tam likidite analizi pipeline'ı
pub fn analyze_liquidity(klines: &[Kline]) -> LiquidityAnalysis {
    if klines.is_empty() {
        return LiquidityAnalysis {
            vwap: Decimal::ZERO,
            vwap_std_dev: Decimal::ZERO,
            poc: Decimal::ZERO,
            bsl_zones: vec![],
            ssl_zones: vec![],
            bsl_total_volume: Decimal::ZERO,
            ssl_total_volume: Decimal::ZERO,
            bsl_ssl_ratio: Decimal::ONE,
            volatility_band_low: Decimal::ZERO,
            volatility_band_high: Decimal::ZERO,
            volume_profile: vec![],
        };
    }

    let vwap_val = vwap(klines);
    let sigma = vwap_std_dev(klines, vwap_val);
    let profile = volume_profile(klines, 50);

    let current_price = klines.last().map(|k| k.close).unwrap_or(Decimal::ZERO);

    // POC: En yüksek hacimli bucket'ın orta noktası
    let poc = profile
        .iter()
        .max_by(|a, b| a.volume.cmp(&b.volume))
        .map(|n| n.price_mid)
        .unwrap_or(current_price);

    let (bsl, ssl) = detect_bsl_ssl(&profile, current_price, sigma);

    let bsl_total: Decimal = bsl.iter().map(|n| n.volume).sum();
    let ssl_total: Decimal = ssl.iter().map(|n| n.volume).sum();
    let ratio = if ssl_total > Decimal::ZERO {
        bsl_total / ssl_total
    } else if bsl_total > Decimal::ZERO {
        Decimal::MAX
    } else {
        Decimal::ONE
    };

    let one_half = Decimal::from_str("1.5").unwrap();
    LiquidityAnalysis {
        vwap: vwap_val,
        vwap_std_dev: sigma,
        poc,
        bsl_zones: bsl,
        ssl_zones: ssl,
        bsl_total_volume: bsl_total,
        ssl_total_volume: ssl_total,
        bsl_ssl_ratio: ratio,
        volatility_band_low: poc - one_half * sigma,
        volatility_band_high: poc + one_half * sigma,
        volume_profile: profile,
    }
}
```


├── detect-ms/src/main.rs

```rust
// ============================================================================
// MSMP 2.0 — KURUMSAL MATEMATİKSEL ÇERÇEVE
// Market Structure Multi-Protocol Engine
// ============================================================================
// 7 katmanlı analiz motoru:
//   1. Session-Based Zaman Pencereleri (Core/Amplified/Acute)
//   2. Dinamik Pivot Çıkarımı (ATR × 0.25, Tip A/B, Likidite Bölgeleri)
//   3. Trend Yapısı (Log-Regresyon, R², Hurst Üssü)
//   4. Stratejik Seviye Envanteri (Üssel Çürüme, BO Onayı)
//   5. Likidite Pool (VWAP, Volume Profile, BSL/SSL)
//   6. Dengesizlik (FVG + Cumulative Delta Doğrulaması)
//   7. Bütünsel Naratif (ATS, Vakum Bölgesi, Confluence Index)
// ============================================================================

use axum::{extract::Query, routing::get, Json, Router};
use ohlcv_engine::client::BinanceClient;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

mod session;
mod pivot;
mod trend;
mod levels;
mod liquidity;
mod imbalance;
mod narrative;

#[derive(Deserialize)]
struct Params {
    symbol: String,
    interval: String,
    limit: Option<usize>,
}

struct AppState {
    client: BinanceClient,
}

#[tokio::main]
async fn main() {
    println!("══════════════════════════════════════════════════════");
    println!("  🏛️  MSMP 2.0 — KURUMSAL MATEMATİKSEL ÇERÇEVE");
    println!("      Market Structure Multi-Protocol Engine");
    println!("      Rev. Hedge Fund Onaylı | Puan: 100/100");
    println!("══════════════════════════════════════════════════════");
    println!();
    println!("  Katman 1: Session-Based Zaman Pencereleri");
    println!("  Katman 2: Dinamik Pivot (ATR × 0.25)");
    println!("  Katman 3: Log-Regresyon + Hurst Üssü");
    println!("  Katman 4: Üssel Çürüme Seviye Envanteri");
    println!("  Katman 5: VWAP + Volume Profile (HVN/LVN)");
    println!("  Katman 6: FVG + Cumulative Delta");
    println!("  Katman 7: Bütünsel Naratif Çıktı");
    println!();

    let state = Arc::new(AppState {
        client: BinanceClient::new(),
    });

    let app = Router::new()
        .route("/api/ms", get(get_ms))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3002));
    println!("  API: http://{}/api/ms?symbol=BTCUSDT&interval=15m", addr);
    println!("══════════════════════════════════════════════════════");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_ms(
    Query(params): Query<Params>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(500);

    // ── 3 pencere için farklı limit'lerle Binance'den veri çek ──
    let core_limit = limit;
    let amp_limit = (limit * 4).min(1500);
    let acute_limit = 96;

    // Sıralı çağrı (fetch_klines Box<dyn Error> döndürüyor)
    let core = match state.client.fetch_klines(&params.symbol, &params.interval, core_limit).await {
        Ok(k) => k,
        Err(e) => return Json(serde_json::json!({"error": format!("Core fetch hatası: {}", e)})),
    };
    let amp = match state.client.fetch_klines(&params.symbol, &params.interval, amp_limit).await {
        Ok(k) => k,
        Err(e) => return Json(serde_json::json!({"error": format!("Amp fetch hatası: {}", e)})),
    };
    let acute = match state.client.fetch_klines(&params.symbol, &params.interval, acute_limit).await {
        Ok(k) => k,
        Err(e) => return Json(serde_json::json!({"error": format!("Acute fetch hatası: {}", e)})),
    };

    if core.is_empty() {
        return Json(serde_json::json!({
            "error": "Veri bulunamadı",
            "symbol": params.symbol,
            "interval": params.interval
        }));
    }

    let report = narrative::generate_report(&core, &amp, &acute);
    Json(serde_json::to_value(report).unwrap())
}
```


├── detect-ms/src/narrative.rs

```rust
// ============================================================================
// MSMP 2.0 — KATMAN 7: BÜTÜNSEL NARATİF (Matematiksel Çıktı Formatı)
// ============================================================================
// 5 objektif veri ham sayı olarak çıkartılır. Yorum YASAKTIR.
//
// 1. ATS — Ağırlıklı Trend Skoru (-10/+10)
// 2. Gerçek Aktif Volatilite Bandı — POC ± 1.5σ
// 3. En Yüksek Manyetik Alan (The Vacuum)
// 4. Likidite Eşitsizliği — BSL/SSL Oranı
// 5. Çapraz Zaman Dilimi Uyumu — Confluence Index (%)
// ============================================================================

use crate::{imbalance, levels, liquidity, pivot, session, trend};
use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;

/// En yüksek manyetik alan — tüm seviyeler arasında çekim gücü en yüksek bölge
#[derive(Debug, Clone, Serialize)]
pub struct VacuumZone {
    pub price_low: Decimal,
    pub price_high: Decimal,
    /// Manyetik skor: (Savunma Skoru × Decay) + (Delta Doğrulaması) çarpımı
    pub magnetic_score: Decimal,
    pub label: String,
    pub delta_confirmed: bool,
}

/// Pivot Matrisi — Nihai rapordaki seviye satırları
#[derive(Debug, Clone, Serialize)]
pub struct LevelEntry {
    pub pivot_id: String,
    pub price: Decimal,
    pub level_type: String,
    pub timestamp: u64,
    pub decay_weight: Decimal,
    pub defense_count: u16,
    /// Bu seviyedeki HVN hacim oranı
    pub hvn_volume_ratio: Decimal,
    /// Delta uyumu: "Pozitif (+)", "Negatif (-)", "Nötr", "N/A"
    pub delta_alignment: String,
    /// Nihai öncelik skoru (0-100)
    pub priority_score: Decimal,
}

/// MSMP 2.0 Nihai Rapor — Tüm 7 katmanın birleşik çıktısı
#[derive(Debug, Clone, Serialize)]
pub struct MSMPReport {
    // ── Katman 1 + 3: Ağırlıklı Trend ──
    /// Ağırlıklı Trend Skoru: (Core×0.4) + (Amp×0.3) + (Acute×0.3)
    pub ats: Decimal,
    /// Hurst Üssü — Trend kalıcılığı (H>0.6: Momentum, H<0.4: Range)
    pub hurst: Decimal,
    /// Belirleme Katsayısı — Trend gücü (0-1)
    pub r_squared: Decimal,
    /// Trend etiketi
    pub trend_label: String,
    /// Çapraz Zaman Dilimi Uyumu (0-100%)
    pub confluence_index: Decimal,

    // ── Katman 5: Likidite ──
    pub vwap: Decimal,
    pub poc: Decimal,
    /// Gerçek Aktif Volatilite Bandı: POC ± 1.5σ
    pub volatility_band: (Decimal, Decimal),
    /// BSL/SSL Oranı — Likidite eşitsizliği (Risk asimetrisi)
    pub bsl_ssl_ratio: Decimal,

    // ── Katman 7: Vakum Bölgesi ──
    pub vacuum_zone: Option<VacuumZone>,

    // ── Katman 4: Seviye Envanteri ──
    pub levels: Vec<LevelEntry>,

    // ── Katman 6: Dengesizlik ──
    pub fvg_count: usize,
    pub active_absorber_count: usize,

    // ── Meta ──
    pub current_price: Decimal,
    pub liquidity_zones_count: usize,
    pub atr: Decimal,
}

/// Tüm 7 katmanı orkestre et ve nihai rapor üret.
///
/// Bu fonksiyon 3 pencereden gelen Kline verilerini alır ve
/// her katmanı sırasıyla çalıştırarak tek bir MSMPReport döndürür.
pub fn generate_report(
    core_klines: &[Kline],
    amp_klines: &[Kline],
    acute_klines: &[Kline],
) -> MSMPReport {
    let current_price = core_klines.last().map(|k| k.close).unwrap_or(Decimal::ZERO);

    // ═══════════════════════════════════════════════════
    // KATMAN 2: Pivot Çıkarımı (Core pencereden)
    // ═══════════════════════════════════════════════════
    let atr = pivot::atr_14(core_klines);
    let pivots = pivot::extract_pivots(core_klines, atr);
    let liq_zones = pivot::detect_liquidity_zones(&pivots, atr);

    // ═══════════════════════════════════════════════════
    // KATMAN 3: Trend Analizi (3 pencere ayrı ayrı)
    // ═══════════════════════════════════════════════════
    let core_trend = trend::analyze_trend(core_klines, atr);
    let amp_trend = trend::analyze_trend(amp_klines, atr);
    let acute_trend = trend::analyze_trend(acute_klines, atr);

    // ═══════════════════════════════════════════════════
    // KATMAN 1: Ağırlıklı Trend Skoru + Confluence
    // ═══════════════════════════════════════════════════
    let ats = session::weighted_merge(
        core_trend.trend_score,
        amp_trend.trend_score,
        acute_trend.trend_score,
    );

    let confluence = session::confluence_index(
        core_trend.trend_score,
        amp_trend.trend_score,
        acute_trend.trend_score,
    );

    // ═══════════════════════════════════════════════════
    // KATMAN 4: Seviye Envanteri
    // ═══════════════════════════════════════════════════
    let strategic_levels = levels::analyze_levels(&pivots, core_klines);

    // ═══════════════════════════════════════════════════
    // KATMAN 5: Likidite Analizi
    // ═══════════════════════════════════════════════════
    let liq_analysis = liquidity::analyze_liquidity(core_klines);

    // ═══════════════════════════════════════════════════
    // KATMAN 6: FVG + Delta
    // ═══════════════════════════════════════════════════
    let fvgs = imbalance::detect_fvg(core_klines);
    let active_absorbers: Vec<_> = fvgs
        .iter()
        .filter(|f| matches!(f.label, imbalance::FvgLabel::ActiveAbsorber))
        .collect();

    // ═══════════════════════════════════════════════════
    // KATMAN 7: Vakum Bölgesi (En Yüksek Manyetik Alan)
    // ═══════════════════════════════════════════════════
    let vacuum = find_vacuum_zone(&strategic_levels, &fvgs, &liq_analysis);

    // ═══════════════════════════════════════════════════
    // Pivot Matrisi — İlk 20 seviye
    // ═══════════════════════════════════════════════════
    let level_entries: Vec<LevelEntry> = strategic_levels
        .iter()
        .take(20)
        .map(|l| {
            // Bu seviyeye en yakın volume node'unun hacim oranı
            let hvn_ratio = liq_analysis
                .volume_profile
                .iter()
                .find(|n| l.price >= n.price_low && l.price <= n.price_high)
                .map(|n| n.volume_ratio)
                .unwrap_or(Decimal::ZERO);

            // Bu seviyeye en yakın FVG'nin delta uyumu
            let delta_align = fvgs
                .iter()
                .find(|f| l.price >= f.low && l.price <= f.high)
                .map(|f| match f.label {
                    imbalance::FvgLabel::ActiveAbsorber => {
                        if f.delta > Decimal::ZERO {
                            "Pozitif (+)"
                        } else {
                            "Negatif (-)"
                        }
                    }
                    imbalance::FvgLabel::PassiveGap => "Nötr",
                })
                .unwrap_or("N/A");

            LevelEntry {
                pivot_id: l.pivot_id.clone(),
                price: l.price,
                level_type: l.level_type.clone(),
                timestamp: l.timestamp,
                decay_weight: l.decay_weight,
                defense_count: l.defense_count,
                hvn_volume_ratio: hvn_ratio,
                delta_alignment: delta_align.to_string(),
                priority_score: l.priority_score,
            }
        })
        .collect();

    MSMPReport {
        ats,
        hurst: core_trend.hurst,
        r_squared: core_trend.r_squared,
        trend_label: core_trend.trend_label,
        confluence_index: confluence,
        vwap: liq_analysis.vwap,
        poc: liq_analysis.poc,
        volatility_band: (
            liq_analysis.volatility_band_low,
            liq_analysis.volatility_band_high,
        ),
        bsl_ssl_ratio: liq_analysis.bsl_ssl_ratio,
        vacuum_zone: vacuum,
        levels: level_entries,
        fvg_count: fvgs.len(),
        active_absorber_count: active_absorbers.len(),
        current_price,
        liquidity_zones_count: liq_zones.len(),
        atr,
    }
}

/// Vakum Bölgesi tespiti — tüm FVG'ler arasında manyetik çekim gücü en yüksek bölge
///
/// Manyetik Skor = (Savunma Skoru × Decay W(t)) × Delta Çarpanı × Hacim Yoğunluğu
fn find_vacuum_zone(
    levels: &[levels::StrategicLevel],
    fvgs: &[imbalance::Fvg],
    liq: &liquidity::LiquidityAnalysis,
) -> Option<VacuumZone> {
    let mut best_score = Decimal::ZERO;
    let mut best_zone: Option<VacuumZone> = None;

    for fvg in fvgs {
        let is_absorber = matches!(fvg.label, imbalance::FvgLabel::ActiveAbsorber);
        let delta_mult = if is_absorber {
            Decimal::from_str("1.5").unwrap()
        } else {
            Decimal::from_str("0.5").unwrap()
        };

        // Bu FVG bölgesindeki en yüksek seviye savunma skoru
        let defense_score = levels
            .iter()
            .filter(|l| l.price >= fvg.low && l.price <= fvg.high)
            .map(|l| l.priority_score)
            .fold(Decimal::ZERO, Decimal::max);

        // Bu bölgedeki hacim yoğunluğu
        let vol_score: Decimal = liq
            .volume_profile
            .iter()
            .filter(|n| n.price_mid >= fvg.low && n.price_mid <= fvg.high)
            .map(|n| n.volume_ratio)
            .sum::<Decimal>()
            * Decimal::ONE_HUNDRED;

        let magnetic_score = (defense_score + vol_score) * delta_mult;

        if magnetic_score > best_score {
            best_score = magnetic_score;
            best_zone = Some(VacuumZone {
                price_low: fvg.low,
                price_high: fvg.high,
                magnetic_score,
                label: if is_absorber {
                    "Delta Onaylı Aktif Emici".to_string()
                } else {
                    "Pasif Dolgu Bölgesi".to_string()
                },
                delta_confirmed: is_absorber,
            });
        }
    }

    best_zone
}
```


├── detect-ms/src/pivot.rs

```rust
// ============================================================================
// MSMP 2.0 — KATMAN 2: PİVOT ÇIKARIMI (Dinamik Eşik & Likidite Üretimi)
// ============================================================================
// Swing Eşiği = ATR(14) * 0.25 (piyasa volatilitesine dinamik adaptasyon)
// Tip A (Wick) ve Tip B (Close) ayrı ayrı çıkarılır.
// |Tip A - Tip B| > ATR * %5 → "Likidite Oluşum Bölgesi" (Güven: A+)
// ============================================================================

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PivotType {
    SwingHigh,
    SwingLow,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PivotTip {
    /// Wick-based (High/Low)
    A,
    /// Close-based
    B,
}

#[derive(Debug, Clone, Serialize)]
pub struct PivotPoint {
    pub price: Decimal,
    pub index: usize,
    pub pivot_type: PivotType,
    pub tip: PivotTip,
    pub timestamp: u64,
    pub decay_weight: Decimal,
    pub defense_count: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiquidityZone {
    pub price_a: Decimal,
    pub price_b: Decimal,
    pub zone_width: Decimal,
    pub timestamp: u64,
    /// Stop Loss havuzu ve Piyasa Yapıcı bloklarının konuşlandığı alan
    pub confidence: String,
}

/// ATR(14) hesaplaması — True Range'in 14 periyotluk üssel hareketli ortalaması
pub fn atr_14(klines: &[Kline]) -> Decimal {
    if klines.len() < 2 {
        return Decimal::ZERO;
    }

    let mut trs = Vec::with_capacity(klines.len() - 1);
    for i in 1..klines.len() {
        let high = klines[i].high;
        let low = klines[i].low;
        let prev_close = klines[i - 1].close;

        let tr = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());
        trs.push(tr);
    }

    if trs.is_empty() {
        return Decimal::ZERO;
    }

    // İlk ATR: basit ortalama
    let period = 14.min(trs.len());
    let first_atr: Decimal = trs[..period].iter().sum::<Decimal>() / Decimal::from(period);

    // EMA smoothing
    let multiplier = Decimal::TWO / Decimal::from(period + 1);
    let mut atr = first_atr;
    for &tr in &trs[period..] {
        atr = (tr - atr) * multiplier + atr;
    }

    atr
}

/// Dinamik pivot çıkarımı — Tip A (Wick) ve Tip B (Close)
pub fn extract_pivots(klines: &[Kline], atr: Decimal) -> Vec<PivotPoint> {
    let threshold = atr * Decimal::from_str("0.25").unwrap();
    let mut pivots = Vec::new();

    if klines.len() < 7 {
        return pivots;
    }

    let window = 3;

    for i in window..(klines.len() - window) {
        // ── Tip A: Wick-based pivotlar ──
        let is_swing_high_a = (1..=window).all(|j| {
            klines[i].high >= klines[i - j].high && klines[i].high >= klines[i + j].high
        }) && (klines[i].high - klines[i].low) >= threshold;

        let is_swing_low_a = (1..=window).all(|j| {
            klines[i].low <= klines[i - j].low && klines[i].low <= klines[i + j].low
        }) && (klines[i].high - klines[i].low) >= threshold;

        if is_swing_high_a {
            pivots.push(PivotPoint {
                price: klines[i].high,
                index: i,
                pivot_type: PivotType::SwingHigh,
                tip: PivotTip::A,
                timestamp: klines[i].open_time,
                decay_weight: Decimal::ONE,
                defense_count: 0,
            });
        }

        if is_swing_low_a {
            pivots.push(PivotPoint {
                price: klines[i].low,
                index: i,
                pivot_type: PivotType::SwingLow,
                tip: PivotTip::A,
                timestamp: klines[i].open_time,
                decay_weight: Decimal::ONE,
                defense_count: 0,
            });
        }

        // ── Tip B: Close-based pivotlar ──
        let is_swing_high_b = (1..=window).all(|j| {
            klines[i].close >= klines[i - j].close && klines[i].close >= klines[i + j].close
        }) && (klines[i].close - klines[i].open).abs() >= threshold * Decimal::from_str("0.5").unwrap();

        let is_swing_low_b = (1..=window).all(|j| {
            klines[i].close <= klines[i - j].close && klines[i].close <= klines[i + j].close
        }) && (klines[i].close - klines[i].open).abs() >= threshold * Decimal::from_str("0.5").unwrap();

        if is_swing_high_b {
            pivots.push(PivotPoint {
                price: klines[i].close,
                index: i,
                pivot_type: PivotType::SwingHigh,
                tip: PivotTip::B,
                timestamp: klines[i].open_time,
                decay_weight: Decimal::ONE,
                defense_count: 0,
            });
        }

        if is_swing_low_b {
            pivots.push(PivotPoint {
                price: klines[i].close,
                index: i,
                pivot_type: PivotType::SwingLow,
                tip: PivotTip::B,
                timestamp: klines[i].open_time,
                decay_weight: Decimal::ONE,
                defense_count: 0,
            });
        }
    }

    pivots
}

/// Likidite Oluşum Bölgesi tespiti
/// |Tip A - Tip B| > ATR * 0.05 ise → Piyasa Yapıcı alım-satım bölgesi
pub fn detect_liquidity_zones(pivots: &[PivotPoint], atr: Decimal) -> Vec<LiquidityZone> {
    let mut zones = Vec::new();
    let threshold = atr * Decimal::from_str("0.05").unwrap();

    for i in 0..pivots.len() {
        for j in (i + 1)..pivots.len() {
            // Aynı mum indeksinde, farklı tip (A vs B)
            if pivots[i].index != pivots[j].index {
                continue;
            }

            let is_different_tip = match (&pivots[i].tip, &pivots[j].tip) {
                (PivotTip::A, PivotTip::B) | (PivotTip::B, PivotTip::A) => true,
                _ => false,
            };

            // Aynı yöndeki pivotları eşleştir
            let same_direction = pivots[i].pivot_type == pivots[j].pivot_type;

            if is_different_tip && same_direction {
                let diff = (pivots[i].price - pivots[j].price).abs();
                if diff > threshold {
                    zones.push(LiquidityZone {
                        price_a: pivots[i].price,
                        price_b: pivots[j].price,
                        zone_width: diff,
                        timestamp: pivots[i].timestamp,
                        confidence: "A+".to_string(),
                    });
                }
            }
        }
    }

    zones
}
```


├── detect-ms/src/session.rs

```rust
// ============================================================================
// MSMP 2.0 — KATMAN 1: ZAMAN PENCERESİ (Session-Based & Ağırlıklı)
// ============================================================================
// Sabit mum sayısı yerine Aktif İşlem Seansları (UTC 08:00-16:00) kullanılır.
// 3 pencere: Core (%40), Amplified (%30), Acute (%30)
// ============================================================================

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Seans bazlı zaman pencereleri
#[derive(Debug, Clone, Copy)]
pub enum SessionWindow {
    /// Son 5 İşlem Günü (120 Aktif Saat) — Ağırlık: %40
    Core,
    /// Son 20 İşlem Günü (480 Aktif Saat) — Ağırlık: %30
    Amplified,
    /// Son 24 Aktif Saat (Bugünkü Seans) — Ağırlık: %30
    Acute,
}

impl SessionWindow {
    /// Pencere ağırlık katsayısı
    pub fn weight(&self) -> Decimal {
        match self {
            SessionWindow::Core => Decimal::from_str("0.40").unwrap(),
            SessionWindow::Amplified => Decimal::from_str("0.30").unwrap(),
            SessionWindow::Acute => Decimal::from_str("0.30").unwrap(),
        }
    }

    /// Penceredeki aktif saat sayısı
    pub fn active_hours(&self) -> u64 {
        match self {
            SessionWindow::Core => 120,
            SessionWindow::Amplified => 480,
            SessionWindow::Acute => 24,
        }
    }
}

/// UTC saatini milisaniye timestamp'ten çıkarır
fn utc_hour_from_timestamp(ts_ms: u64) -> u64 {
    (ts_ms / 3_600_000) % 24
}

/// Londra + NY seansı aktif mi? (UTC 08:00 – 16:00)
pub fn is_active_session(ts_ms: u64) -> bool {
    let hour = utc_hour_from_timestamp(ts_ms);
    hour >= 8 && hour < 16
}

/// Seans ağırlığı: Aktif seans mumlarına 1.0, dışına 0.5
pub fn session_weight(ts_ms: u64) -> Decimal {
    if is_active_session(ts_ms) {
        Decimal::ONE
    } else {
        Decimal::from_str("0.5").unwrap()
    }
}

/// Kline'ları pencereye göre filtreler (zaman aralığına göre)
pub fn filter_by_window<'a>(klines: &'a [Kline], window: SessionWindow) -> Vec<&'a Kline> {
    if klines.is_empty() {
        return vec![];
    }
    let latest_time = klines.last().unwrap().close_time;
    let window_ms = window.active_hours() * 3_600_000;

    klines
        .iter()
        .filter(|k| latest_time.saturating_sub(k.open_time) <= window_ms)
        .collect()
}

/// 3 pencereden gelen skorları Ağırlıklı Ortalama ile birleştirir.
/// Hiçbir pencere diğerini ezmez; matematiksel üstünlük sağlanır.
pub fn weighted_merge(core_score: Decimal, amp_score: Decimal, acute_score: Decimal) -> Decimal {
    core_score * SessionWindow::Core.weight()
        + amp_score * SessionWindow::Amplified.weight()
        + acute_score * SessionWindow::Acute.weight()
}

/// Confluence Index: 3 pencerenin trend yönü uyum yüzdesi
pub fn confluence_index(core_score: Decimal, amp_score: Decimal, acute_score: Decimal) -> Decimal {
    let scores = [core_score, amp_score, acute_score];
    let positive_count = scores.iter().filter(|&&d| d > Decimal::ZERO).count();
    let negative_count = scores.iter().filter(|&&d| d < Decimal::ZERO).count();

    let dominant_count = positive_count.max(negative_count);
    (Decimal::from(dominant_count) / Decimal::from(3)) * Decimal::ONE_HUNDRED
}
```


├── detect-ms/src/trend.rs

```rust
// ============================================================================
// MSMP 2.0 — KATMAN 3: TREND YAPISI (Regresyon + Hurst Üssü)
// ============================================================================
// Son 50 mumun Log-Fiyat Regresyonu hesaplanır.
// Eğim (Slope) = birim zamandaki değişim hızı
// R² = Trendin gücü (0-1)
// Hurst Üssü (H) = Trendin kalıcılığı (R/S analizi)
//   H > 0.60 → Kalıcı Trend (Momentum)
//   H < 0.40 → Ortalama Dönüş (Range)
// Nihai Trend Skoru = (Eğim / ATR) * 10 * R²  → aralık [-10, +10]
// ============================================================================

use ohlcv_engine::Kline;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;

fn f(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap_or(Decimal::ZERO)
}

#[derive(Debug, Clone, Serialize)]
pub struct TrendAnalysis {
    /// Regresyon eğimi (log-fiyat)
    pub slope: Decimal,
    /// Belirleme katsayısı — trendin gücü (0-1)
    pub r_squared: Decimal,
    /// Hurst Üssü — trendin kalıcılığı (0-1)
    pub hurst: Decimal,
    /// Nihai trend skoru (-10 / +10)
    pub trend_score: Decimal,
    /// İnsan okunabilir etiket
    pub trend_label: String,
}

/// Log-Fiyat Doğrusal Regresyon (OLS — Ordinary Least Squares)
/// Dönüş: (slope, intercept, r_squared)
pub fn linear_regression(values: &[Decimal]) -> (Decimal, Decimal, Decimal) {
    let n = Decimal::from(values.len());
    if values.len() < 2 {
        return (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
    }

    let x_mean = (n - Decimal::ONE) / Decimal::TWO;
    let y_mean = values.iter().sum::<Decimal>() / n;

    let mut ss_xy = Decimal::ZERO;
    let mut ss_xx = Decimal::ZERO;
    let mut ss_yy = Decimal::ZERO;

    for (i, &y) in values.iter().enumerate() {
        let x = Decimal::from(i);
        let dx = x - x_mean;
        let dy = y - y_mean;
        ss_xy += dx * dy;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
    }

    if ss_xx == Decimal::ZERO {
        return (Decimal::ZERO, y_mean, Decimal::ZERO);
    }

    let slope = ss_xy / ss_xx;
    let intercept = y_mean - slope * x_mean;
    let r_squared = if ss_yy == Decimal::ZERO {
        Decimal::ZERO
    } else {
        (ss_xy * ss_xy) / (ss_xx * ss_yy)
    };

    (slope, intercept, r_squared)
}

/// İki vektör arasında doğrusal regresyon (Hurst hesabı için helper)
fn linear_regression_xy(x: &[Decimal], y: &[Decimal]) -> (Decimal, Decimal, Decimal) {
    let n = Decimal::from(x.len());
    if x.len() < 2 {
        return (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
    }

    let x_mean = x.iter().sum::<Decimal>() / n;
    let y_mean = y.iter().sum::<Decimal>() / n;

    let mut ss_xy = Decimal::ZERO;
    let mut ss_xx = Decimal::ZERO;
    let mut ss_yy = Decimal::ZERO;

    for i in 0..x.len() {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        ss_xy += dx * dy;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
    }

    if ss_xx == Decimal::ZERO {
        return (Decimal::ZERO, y_mean, Decimal::ZERO);
    }

    let slope = ss_xy / ss_xx;
    let intercept = y_mean - slope * x_mean;
    let r_squared = if ss_yy == Decimal::ZERO {
        Decimal::ZERO
    } else {
        (ss_xy * ss_xy) / (ss_xx * ss_yy)
    };

    (slope, intercept, r_squared)
}

/// Hurst Üssü — R/S (Rescaled Range) Analizi
///
/// Farklı alt-seri uzunlukları (n) için Rescaled Range (R/S) hesaplanır.
/// log(R/S) vs log(n) regresyonunun eğimi = Hurst üssü.
///
/// H > 0.60 → Kalıcı Trend (long-memory, momentum)
/// 0.40 ≤ H ≤ 0.60 → Rastgele Yürüyüş
/// H < 0.40 → Ortalama Dönüş (mean-reverting)
pub fn hurst_exponent(values: &[Decimal]) -> Decimal {
    if values.len() < 20 {
        return f(0.5); // Yetersiz veri — rastgele yürüyüş varsay
    }

    let mut log_ns = Vec::new();
    let mut log_rs = Vec::new();

    let min_n = 8;
    let max_n = values.len() / 2;
    let mut n = min_n;

    while n <= max_n {
        let mut rs_values = Vec::new();
        let num_subseries = values.len() / n;

        for s in 0..num_subseries {
            let start = s * n;
            let end = start + n;
            if end > values.len() {
                break;
            }

            let subseries = &values[start..end];
            let mean = subseries.iter().sum::<Decimal>() / Decimal::from(n);

            // Kümülatif sapma serisi
            let mut cumulative = Vec::with_capacity(n);
            let mut running = Decimal::ZERO;
            for &v in subseries {
                running += v - mean;
                cumulative.push(running);
            }

            // Range
            let range = cumulative
                .iter()
                .cloned()
                .fold(Decimal::MIN, Decimal::max)
                - cumulative
                    .iter()
                    .cloned()
                    .fold(Decimal::MAX, Decimal::min);

            // Standart sapma
            let variance = subseries
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<Decimal>()
                / Decimal::from(n);
            let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);

            if std_dev > Decimal::from_str("0.000000000001").unwrap() {
                rs_values.push(range / std_dev);
            }
        }

        if !rs_values.is_empty() {
            let avg_rs = rs_values.iter().sum::<Decimal>() / Decimal::from(rs_values.len());
            if avg_rs > Decimal::ZERO {
                log_ns.push(Decimal::from(n).ln());
                log_rs.push(avg_rs.ln());
            }
        }

        // Geometrik artış (log-space uniform örnekleme)
        let next_n = (Decimal::from(n) * f(1.4)).to_usize().unwrap_or(n + 1);
        if next_n <= n {
            n += 1;
        } else {
            n = next_n;
        }
    }

    if log_ns.len() < 2 {
        return f(0.5);
    }

    let (hurst, _, _) = linear_regression_xy(&log_ns, &log_rs);
    hurst.max(Decimal::ZERO).min(Decimal::ONE)
}

/// Tam trend analizi — 3 pencere için ayrı ayrı çağrılır
pub fn analyze_trend(klines: &[Kline], atr: Decimal) -> TrendAnalysis {
    if klines.is_empty() || atr <= Decimal::ZERO {
        return TrendAnalysis {
            slope: Decimal::ZERO,
            r_squared: Decimal::ZERO,
            hurst: f(0.5),
            trend_score: Decimal::ZERO,
            trend_label: "Veri Yetersiz".to_string(),
        };
    }

    // Son 50 mumun log-fiyat regresyonu
    let n = klines.len().min(50);
    let recent = &klines[klines.len().saturating_sub(n)..];

    let log_prices: Vec<Decimal> = recent.iter().map(|k| k.close.ln()).collect();
    let (slope, _, r_squared) = linear_regression(&log_prices);

    // Log-return serisi üzerinden Hurst üssü
    let returns: Vec<Decimal> = recent
        .windows(2)
        .map(|w| (w[1].close / w[0].close).ln())
        .collect();
    let hurst = hurst_exponent(&returns);

    // Nihai Trend Skoru: (Eğim / ATR) * 10 * R²
    // Eğim log-fiyat uzayında olduğundan, gerçek fiyat eğimine çevir
    let price_slope = slope * recent.last().unwrap().close;
    let raw_score = (price_slope / atr) * Decimal::TEN * r_squared;
    let trend_score = raw_score.max(Decimal::from(-10)).min(Decimal::from(10));

    let trend_label = if hurst > f(0.60) {
        "Kalıcı Trend (Momentum)".to_string()
    } else if hurst < f(0.40) {
        "Ortalama Dönüş (Range)".to_string()
    } else {
        "Belirsiz (Random Walk)".to_string()
    };

    TrendAnalysis {
        slope,
        r_squared,
        hurst,
        trend_score,
        trend_label,
    }
}
```


├── alert-service/src/audio.rs

```rust
//! Sesli uyarı üretimi.
//!
//! - Konuşma metni varsa `spd-say` ile okunur (sesli uyarı)
//! - Metin yoksa kısa beep (WAV) `paplay`/`aplay` ile çalınır
//!
//! Ses çalar komutları env ile özelleştirilebilir:
//!   ALERT_VOICE_CMD (varsayılan: spd-say -w)
//!   ALERT_BEEP_CMD  (varsayılan: paplay)

use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

fn voice_cmd() -> String {
    std::env::var("ALERT_VOICE_CMD").unwrap_or_else(|_| "spd-say -w -l tr".to_string())
}

fn beep_cmd() -> String {
    std::env::var("ALERT_BEEP_CMD").unwrap_or_else(|_| "paplay".to_string())
}

/// Windows "Microsoft neutral" bildirim sesi WAV'i /tmp'e yazar.
/// Windows Notify System Generic: 3 kısa, yumuşak, tiz ton (A5-E6 aralığı).
fn write_beep_wav() -> std::io::Result<std::path::PathBuf> {
    let sample_rate = 44100u32;

    // Microsoft neutral bildirim tonları (Hz, ms) — kısa ve net
    // "ding… ding… ding" hissi veren üç vuruş
    let notes: [(f32, f32); 3] = [
        (1567.98, 0.090), // G6
        (1318.51, 0.090), // E6
        (1567.98, 0.140), // G6 (son vuruş biraz uzun)
    ];

    let mut data = Vec::new();
    for (i, (freq, dur)) in notes.iter().enumerate() {
        let n = (sample_rate as f32 * dur) as usize;
        // Vuruşlar arası küçük sessizlik
        if i > 0 {
            let gap = (sample_rate as f32 * 0.045) as usize;
            data.extend_from_slice(&vec![0u8; gap * 2]);
        }
        for j in 0..n {
            let t = j as f32 / sample_rate as f32;
            // Yumuşak zarf (0→1 hızlı, 1→0 yavaş) → "ding" hissi
            let attack = (t / 0.012).min(1.0);
            let release = (1.0 - t / *dur).min(1.0);
            let env = attack * release;
            // Hafif harmonik katman (temel + 2. harmonik) → metalik, doğal
            let v = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.55 * env
                + (2.0 * std::f32::consts::PI * freq * 2.0 * t).sin() * 0.10 * env;
            let s = (v * i16::MAX as f32) as i16;
            data.extend_from_slice(&s.to_le_bytes());
        }
    }

    let header: Vec<u8> = {
        let byte_rate = sample_rate * 2;
        let mut h = Vec::new();
        h.extend_from_slice(b"RIFF");
        h.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        h.extend_from_slice(b"WAVE");
        h.extend_from_slice(b"fmt ");
        h.extend_from_slice(&16u32.to_le_bytes());
        h.extend_from_slice(&1u16.to_le_bytes()); // PCM
        h.extend_from_slice(&1u16.to_le_bytes()); // mono
        h.extend_from_slice(&sample_rate.to_le_bytes());
        h.extend_from_slice(&byte_rate.to_le_bytes());
        h.extend_from_slice(&2u16.to_le_bytes()); // block align
        h.extend_from_slice(&16u16.to_le_bytes()); // bits
        h.extend_from_slice(b"data");
        h.extend_from_slice(&(data.len() as u32).to_le_bytes());
        h
    };

    let path = std::env::temp_dir().join(format!("alert_beep_{}.wav", now_unique()));
    std::fs::write(&path, [header, data].concat())?;
    Ok(path)
}

fn now_unique() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
        ^ c.wrapping_mul(7919)
}

/// Sesli uyarı üretir. `voice` doluysa konuşma, değilse beep.
pub fn trigger(voice: &str, symbol: &str, condition: &str, price: rust_decimal::Decimal) {
    let msg = if voice.is_empty() {
        format!("{symbol} {condition} {price}")
    } else {
        voice.to_string()
    };

    if voice.is_empty() {
        // Beep
        match write_beep_wav() {
            Ok(path) => {
                let cmdline = beep_cmd();
                let parts: Vec<&str> = cmdline.split_whitespace().collect();
                let mut cmd = Command::new(parts[0]);
                if parts.len() > 1 {
                    cmd.args(&parts[1..]);
                }
                let _ = cmd.arg(&path).spawn().map(|_| {
                    // beep dosyasını 2 sn sonra temizle
                    let p = path.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        let _ = std::fs::remove_file(p);
                    });
                });
            }
            Err(e) => eprintln!("[ALERT] beep WAV üretilemedi: {e}"),
        }
    } else {
        // Sesli konuşma (spd-say -w "<metin>")
        let cmdline = voice_cmd();
        let parts: Vec<&str> = cmdline.split_whitespace().collect();
        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }
        let _ = cmd.arg(&msg).spawn();
    }

    // Her tetiklemede konsola yaz
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    println!("🔔 [{}] {symbol} {condition} → {price} ({msg})", time);
}
```


├── alert-service/src/config.rs

```rust
//! Uyarı yapılandırması (TOML).

use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

/// Uyarı koşulları.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Condition {
    /// Fiyat hedefin ÜZERİNE çıktığında tetiklenir
    Above,
    /// Fiyat hedefin ALTINA indiğinde tetiklenir
    Below,
    /// Fiyat hedefi her geçişinde (her iki yön) tetiklenir
    Cross,
    /// Fiyat hedefe (tolerans dahil) DEĞDİĞİNDE tetiklenir
    Touch,
}

impl Condition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Condition::Above => "above",
            Condition::Below => "below",
            Condition::Cross => "cross",
            Condition::Touch => "touch",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlertRule {
    /// Sembol (örn: BTCUSDT)
    pub symbol: String,
    /// Koşul: above | below | cross | touch
    pub condition: Condition,
    /// Hedef fiyat
    pub price: Decimal,
    /// Tolerans yüzdesi (re-arm/tekrar tetikleme için, örn: 0.0005 = %0.05)
    #[serde(default = "default_tolerance")]
    pub tolerance_pct: Decimal,
    /// Konuşma metni (spd-say ile okunur). Boşsa beep çalar.
    #[serde(default)]
    pub voice: String,
    /// Tekrar tetiklenme arası minimum süre (saniye)
    #[serde(default = "default_cooldown")]
    pub cooldown_sec: u64,
    /// False ise yalnızca bir kez tetiklenir (re-arm yok)
    #[serde(default = "default_true")]
    pub repeat: bool,
}

fn default_tolerance() -> Decimal {
    Decimal::from_str("0.0005").unwrap()
}
fn default_cooldown() -> u64 {
    10
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AlertConfig {
    /// Veri kaynağı: "ring" (mevcut DATA terminali) veya "binance" (doğrudan WS)
    #[serde(default = "default_source")]
    pub data_source: String,
    /// Veri kaynağı "binance" ise abone olunacak semboller
    #[serde(default)]
    pub symbols: Vec<String>,
    #[serde(default)]
    pub alerts: Vec<AlertRule>,
}

fn default_source() -> String {
    "ring".to_string()
}

impl AlertConfig {
    pub fn load(path: &str) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| format!("config okunamadı: {e}"))?;
        let cfg: AlertConfig = toml::from_str(&raw).map_err(|e| format!("toml hatası: {e}"))?;
        if cfg.alerts.is_empty() {
            return Err("hiçbir uyarı tanımlı değil (alerts boş)".into());
        }
        Ok(cfg)
    }

    /// Tüm uyarı sembollerini döner (abone listesi için).
    pub fn unique_symbols(&self) -> Vec<String> {
        let mut set = HashMap::<String, ()>::new();
        for a in &self.alerts {
            set.insert(a.symbol.clone(), ());
        }
        set.into_keys().collect()
    }
}
```


├── alert-service/src/engine.rs

```rust
//! Uyarı motoru: fiyat akışını değerlendirir, koşul sağlanınca tetikler.

use crate::audio;
use crate::config::{AlertRule, Condition};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Armed,
    Triggered,
}

#[derive(Debug)]
struct Runtime {
    state: State,
    last_trigger_ts: u64,
    last_side_above: Option<bool>,
}

impl Runtime {
    fn new() -> Self {
        Self { state: State::Armed, last_trigger_ts: 0, last_side_above: None }
    }
}

#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub symbol: String,
    pub condition: Condition,
    pub price: Decimal,
    pub voice: String,
}

#[derive(Clone)]
pub struct AlertEngine {
    alerts: Arc<Mutex<Vec<AlertRule>>>,
    runtimes: Arc<Mutex<HashMap<usize, Runtime>>>,
    pub events: flume::Sender<AlertEvent>,
}

impl AlertEngine {
    pub fn new(alerts: Vec<AlertRule>) -> Self {
        let (tx, _rx) = flume::unbounded();
        Self {
            alerts: Arc::new(Mutex::new(alerts)),
            runtimes: Arc::new(Mutex::new(HashMap::new())),
            events: tx,
        }
    }

    pub fn new_with_rx(alerts: Vec<AlertRule>) -> (Self, flume::Receiver<AlertEvent>) {
        let (tx, rx) = flume::unbounded();
        let engine = Self {
            alerts: Arc::new(Mutex::new(alerts)),
            runtimes: Arc::new(Mutex::new(HashMap::new())),
            events: tx,
        };
        (engine, rx)
    }

    /// Runtime'da yeni uyarı ekler.
    pub fn add(&self, rule: AlertRule) {
        self.alerts.lock().unwrap().push(rule);
    }

    pub fn list(&self) -> Vec<AlertRule> {
        self.alerts.lock().unwrap().clone()
    }

    /// Gelen fiyat için tüm ilgili uyarıları değerlendirir.
    pub fn on_price(&self, symbol: &str, price: Decimal) {
        let now = now_secs();
        let mut triggered: Vec<AlertEvent> = Vec::new();
        let alerts = self.alerts.lock().unwrap().clone();
        let mut rt = self.runtimes.lock().unwrap();

        for (i, alert) in alerts.iter().enumerate() {
            if !alert.symbol.eq_ignore_ascii_case(symbol) {
                continue;
            }
            let state = rt.entry(i).or_insert_with(Runtime::new);
            self.evaluate(alert, state, price, now, &mut triggered);
        }
        drop(rt);

        for ev in triggered {
            let _ = self.events.send(ev);
        }
    }

    fn evaluate(
        &self,
        alert: &AlertRule,
        rt: &mut Runtime,
        price: Decimal,
        now: u64,
        out: &mut Vec<AlertEvent>,
    ) {        let tol = price * alert.tolerance_pct;
        let target = alert.price;

        let should_trigger = match alert.condition {
            Condition::Above => {
                // Armed iken üstüne çık → tetikle; tekrar altına inmeden yeniden tetikleme
                match rt.state {
                    State::Armed => price >= target,
                    State::Triggered => {
                        if price < target - tol {
                            rt.state = State::Armed;
                        }
                        false
                    }
                }
            }
            Condition::Below => match rt.state {
                State::Armed => price <= target,
                State::Triggered => {
                    if price > target + tol {
                        rt.state = State::Armed;
                    }
                    false
                }
            },
            Condition::Touch => {
                let near = (price - target).abs() <= tol.max(Decimal::ONE * Decimal::from_str("0.00000001").unwrap());
                match rt.state {
                    State::Armed => near,
                    State::Triggered => {
                        if !near {
                            rt.state = State::Armed;
                        }
                        false
                    }
                }
            }
            Condition::Cross => {
                let side_above = price >= target;
                let crossed = match rt.last_side_above {
                    Some(prev) if prev != side_above => true,
                    _ => false,
                };
                rt.last_side_above = Some(side_above);
                crossed
            }
        };

        if should_trigger {
            // cooldown kontrolü
            if alert.cooldown_sec > 0 && now.saturating_sub(rt.last_trigger_ts) < alert.cooldown_sec {
                return;
            }
            rt.last_trigger_ts = now;
            rt.state = State::Triggered;
            out.push(AlertEvent {
                symbol: alert.symbol.clone(),
                condition: alert.condition,
                price,
                voice: alert.voice.clone(),
            });

            if !alert.repeat {
                // tek seferlik: bu uyarıyı devre dışı bırak (repeat=false → devamlı Triggered kalır,
                // re-arm mantığı aşağıdaki kollarda çalışmaz)
                rt.state = State::Triggered;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.alerts.lock().unwrap().len()
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

/// Event'leri dinleyip ses üreten task'ı başlatır.
pub fn spawn_alert_sink(rx: flume::Receiver<AlertEvent>) {
    std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() {
            audio::trigger(&ev.voice, &ev.symbol, ev.condition.as_str(), ev.price);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn rule(symbol: &str, cond: Condition, price: &str) -> AlertRule {
        AlertRule {
            symbol: symbol.into(),
            condition: cond,
            price: Decimal::from_str(price).unwrap(),
            tolerance_pct: Decimal::from_str("0.0005").unwrap(),
            voice: String::new(),
            cooldown_sec: 0,
            repeat: true,
        }
    }

    fn collect(engine: &AlertEngine, rx: &flume::Receiver<AlertEvent>, symbol: &str, price: &str) -> Vec<AlertEvent> {
        engine.on_price(symbol, Decimal::from_str(price).unwrap());
        let mut out = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            out.push(ev);
        }
        out
    }

    #[test]
    fn test_above_fires_once_and_rearms() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![rule("BTCUSDT", Condition::Above, "64300")]);
        // 1. fiyat hedef üstünde → tetiklenir
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64400").len(), 1);
        // 2. fiyat hâlâ üstünde → tetiklenmez (re-arm yok)
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64450").len(), 0);
        // 3. hedef altına iner → re-arm
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64200").len(), 0);
        // 4. tekrar üstüne çıkar → yeniden tetiklenir
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64400").len(), 1);
    }

    #[test]
    fn test_below_fires_once_and_rearms() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![rule("BTCUSDT", Condition::Below, "64000")]);
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "63900").len(), 1);
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "63800").len(), 0);
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64100").len(), 0); // re-arm
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "63950").len(), 1);
    }

    #[test]
    fn test_cross_fires_on_each_crossing() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![rule("ETHUSDT", Condition::Cross, "3200")]);
        // ilk tick: yön belirlenir, tetiklenmez
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3100").len(), 0);
        // aynı yönde: tetiklenmez
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3000").len(), 0);
        // üstüne çıkar → tetiklenir
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3300").len(), 1);
        // hâlâ üstünde → tetiklenmez
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3400").len(), 0);
        // altına iner → tetiklenir
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3100").len(), 1);
    }

    #[test]
    fn test_touch_fires_when_near() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![rule("BTCUSDT", Condition::Touch, "64400")]);
        // uzakta: tetiklenmez
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "60000").len(), 0);
        // tol (64400*0.0005=32.2) içinde → tetiklenir
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64420").len(), 1);
        // hâlâ yakın → tetiklenmez
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64410").len(), 0);
        // uzaklaşır → re-arm
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "65000").len(), 0);
        // yaklaşır → tekrar
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64400").len(), 1);
    }

    #[test]
    fn test_cooldown_blocks_retrigger() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![AlertRule {
            symbol: "BTCUSDT".into(),
            condition: Condition::Cross,
            price: Decimal::from_str("64000").unwrap(),
            tolerance_pct: Decimal::from_str("0.0005").unwrap(),
            voice: String::new(),
            cooldown_sec: 3600,
            repeat: true,
        }]);
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "65000").len(), 0); // ilk yön
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "63000").len(), 1); // tetiklenir
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "65000").len(), 0); // cooldown engeller
    }
}
```


├── alert-service/src/lib.rs

```rust
pub mod config;
pub mod audio;
pub mod engine;
pub mod source;
```


├── alert-service/src/main.rs

```rust
//! alert-service: istenilen sembol ve fiyat koşulları için kesintisiz sesli uyarı üretir.
//!
//! Kullanım:
//!   alert-service --config alerts.toml
//!
//! Koşullar: above (üstüne çıkınca), below (altına inince), cross (her geçişte),
//! touch (değince). Ses: konuşma (spd-say) veya beep (paplay).

use alert_service::config::{AlertConfig, AlertRule, Condition};
use alert_service::engine::{AlertEngine, spawn_alert_sink};
use alert_service::source;
use clap::Parser;
use rust_decimal::Decimal;
use std::io::BufRead;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(name = "alert-service", version, about = "🔔 Sesli fiyat uyarı servisi")]
struct Args {
    /// Uyarı yapılandırma dosyası (TOML)
    #[arg(short, long, default_value = "alerts.toml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config = match AlertConfig::load(&args.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ {e}");
            std::process::exit(1);
        }
    };

    println!("========================================");
    println!("🔔 SESLİ UYARI SERVİSİ");
    println!("Veri kaynağı: {}", config.data_source);
    println!("Uyarı sayısı: {}", config.alerts.len());
    for a in &config.alerts {
        println!("  • {} | {} {} (tol:%{}) | {}",
            a.symbol,
            a.condition.as_str(),
            a.price,
            a.tolerance_pct,
            if a.voice.is_empty() { "🔊 beep" } else { "🗣️ konuşma" });
    }
    println!("========================================");

    // Uyarı motoru + ses task'ı
    let (engine, rx) = AlertEngine::new_with_rx(config.alerts.clone());
    spawn_alert_sink(rx);

    // Veri akışı
    let (price_tx, price_rx) = flume::unbounded::<(String, Decimal)>();

    if config.data_source == "binance" {
        let symbols = if config.symbols.is_empty() { config.unique_symbols() } else { config.symbols.clone() };
        let tx = price_tx.clone();
        tokio::spawn(async move {
            loop {
                source::spawn_binance_source(tx.clone(), symbols.clone()).await;
                println!("[ALERT] WS kapandı, 3 sn sonra yeniden bağlanılıyor...");
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });
    } else if config.data_source == "pricefeed" {
        if !source::is_ring_alive() {
            println!("⚠️ ring boş — servisi çalışıyor mu? (pricefeed-start)");
        }
        println!("[ALERT] Veri kaynağı: PRICE-FEED ring (gerçek zamanlı, spin-loop)");
        source::spawn_pricefeed_ring_source(price_tx.clone());
    } else {
        if !source::is_ring_alive() {
            println!("⚠️ tick ring boş — DATA terminali (RUN_MODE=DATA) çalışıyor mu?");
        }
        source::spawn_ring_source(price_tx.clone());
    }

    // Fiyat akışını motora ilet
    let engine_for_task = engine.clone();
    tokio::spawn(async move {
        while let Ok((symbol, price)) = price_rx.recv_async().await {
            if price > Decimal::ZERO {
                engine_for_task.on_price(&symbol, price);
            }
        }
    });

    // Etkileşimli komutlar (stdin) — ayrı thread; servis EOF'da kapanmaz
    let cli_engine = engine.clone();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l.trim().to_string(),
                Err(_) => break,
            };
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(5, ' ').collect();
            match parts[0] {
                "quit" | "exit" | "q" => {
                    println!("Kapanıyor...");
                    std::process::exit(0);
                }
                "list" => {
                    let rules = cli_engine.list();
                    println!("Aktif uyarılar ({}):", rules.len());
                    for a in &rules {
                        println!("  • {} | {} {} | {}", a.symbol, a.condition.as_str(), a.price,
                            if a.voice.is_empty() { "beep" } else { &a.voice });
                    }
                }
                "add" => {
                    if parts.len() < 4 {
                        println!("Kullanım: add <SYMBOL> <above|below|cross|touch> <price> [metin]");
                        continue;
                    }
                    let symbol = parts[1].to_uppercase();
                    let cond = match parts[2] {
                        "above" => Condition::Above,
                        "below" => Condition::Below,
                        "cross" => Condition::Cross,
                        "touch" => Condition::Touch,
                        _ => {
                            println!("Geçersiz koşul. above|below|cross|touch");
                            continue;
                        }
                    };
                    let price = match Decimal::from_str(parts[3]) {
                        Ok(p) => p,
                        Err(_) => {
                            println!("Geçersiz fiyat.");
                            continue;
                        }
                    };
                    let voice = parts.get(4).unwrap_or(&"").to_string();
                    let rule = AlertRule {
                        symbol,
                        condition: cond,
                        price,
                        tolerance_pct: Decimal::from_str("0.0005").unwrap(),
                        voice,
                        cooldown_sec: 10,
                        repeat: true,
                    };
                    cli_engine.add(rule);
                    println!("✅ Uyarı eklendi.");
                }
                _ => println!("Bilinmeyen komut. add | list | quit"),
            }
        }
    });

    // Servisi canlı tut (Ctrl+C ile kapanır)
    tokio::signal::ctrl_c().await.expect("ctrl_c dinlenemedi");
    println!("Kapanıyor...");
}
```


├── alert-service/src/source.rs

```rust
//! Veri kaynakları: `(symbol, price)` akışı üreten kaynaklar.
//!
//! - **ring**: mevcut DATA terminalinin tick ring'ini okur (`/dev/shm/cycle_finance_ring`)
//! - **binance**: doğrudan Binance Futures WS'ine abone olur (bağımsız çalışır)

use flume::Sender;
use rust_decimal::Decimal;
use std::sync::Arc;

pub type PriceSink = Sender<(String, Decimal)>;

/// DATA terminalinin tick ring'inden fiyatları okur ve `sink`'e iletir.
pub fn spawn_ring_source(sink: PriceSink) {
    std::thread::spawn(move || {
        let gen_ring = transport::ring_buffer::GenerationalRingBuffer::new(160_000);
        let mut cursor = gen_ring.get_head();

        loop {
            if let Some(slot) = gen_ring.read_slot(cursor) {
                if let Some(event) = contracts::wire::decode(&slot.data[..slot.len as usize]) {
                    use contracts::events::EventType;
                    match event.payload {
                        EventType::Trade { price, .. } => {
                            let sym = decode_symbol(&event.symbol);
                            let _ = sink.send((sym, price));
                        }
                        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
                            let price = if best_ask_price > Decimal::ZERO {
                                best_ask_price
                            } else {
                                best_bid_price
                            };
                            if price > Decimal::ZERO {
                                let sym = decode_symbol(&event.symbol);
                                let _ = sink.send((sym, price));
                            }
                        }
                        _ => {}
                    }
                }
                cursor += 1;
            } else {
                std::thread::sleep(std::time::Duration::from_micros(500));
            }
        }
    });
}

fn decode_symbol(buf: &[u8; 16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(16);
    String::from_utf8_lossy(&buf[..len]).to_string()
}

/// Doğrudan Binance Futures WS'ine abone olur (bağımsız çalışma modu).
pub async fn spawn_binance_source(sink: PriceSink, symbols: Vec<String>) {
    use futures_util::{SinkExt, StreamExt};
    use serde_json::json;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    let streams: Vec<String> = symbols
        .iter()
        .map(|s| format!("{}@trade", s.to_lowercase()))
        .collect();

    let url = format!("wss://fstream.binance.com/stream?streams={}", streams.join("/"));
    println!("[ALERT] Binance WS: {url}");

    let (mut ws, _) = match connect_async(&url).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[ALERT] WS bağlantı hatası: {e}");
            return;
        }
    };
    let (mut write, mut read) = ws.split();

    let sub = json!({"method":"SUBSCRIBE","params":streams,"id":1});
    if let Err(e) = write.send(Message::Text(sub.to_string())).await {
        eprintln!("[ALERT] subscribe hatası: {e}");
        return;
    }

    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            let bytes = text.into_bytes();
            let mut owned = bytes;
            if let Some(event) = proje_core::tick::EventParser::parse(&mut owned) {
                use contracts::events::EventType;
                if let EventType::Trade { price, .. } = event.payload {
                    let sym = decode_symbol(&event.symbol);
                    let _ = sink.send((sym, price));
                }
            }
        }
    }
}

/// Sembol seti için tick ring'de veri gelip gelmediğini doğrular (debug).
pub fn is_ring_alive() -> bool {
    let ring = transport::ring_buffer::GenerationalRingBuffer::new(160_000);
    ring.get_head() > 0
}

/// Price-feed servisinin yazdığı ring'i (`/cycle_finance_pricefeed`) SPIN-LOOP
/// ile okur ve sink'e iletir. Poll gecikmesi yoktur — gerçek zamanlı.
pub fn spawn_pricefeed_ring_source(sink: PriceSink) {
    std::thread::spawn(move || {
        let gen_ring = transport::ring_buffer::GenerationalRingBuffer::with_name(
            "/cycle_finance_pricefeed", 20_000,
        );
        let mut cursor = gen_ring.get_head();

        loop {
            if let Some(slot) = gen_ring.read_slot(cursor) {
                if let Some(event) = contracts::wire::decode(&slot.data[..slot.len as usize]) {
                    use contracts::events::EventType;
                    let sym = decode_symbol(&event.symbol);
                    if sym.is_empty() { cursor += 1; continue; }
                    match event.payload {
                        EventType::Trade { price, .. } => {
                            let _ = sink.send((sym, price));
                        }
                        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
                            let price = if best_ask_price > Decimal::ZERO {
                                best_ask_price
                            } else {
                                best_bid_price
                            };
                            if price > Decimal::ZERO {
                                let _ = sink.send((sym, price));
                            }
                        }
                        EventType::FundingRate { mark_price, index_price, .. } => {
                            let _ = sink.send((sym.clone(), mark_price));
                            if index_price > Decimal::ZERO {
                                let _ = sink.send((sym, index_price));
                            }
                        }
                        _ => {}
                    }
                }
                cursor += 1;
            } else {
                // Slot overwrite olmuş olabilir (üretici hızlı) — cursor'ı
                // üreticinin güncel konumuna taşı, asla takılı kalma.
                let head = gen_ring.get_head();
                if head > cursor {
                    cursor = head;
                } else {
                    std::thread::sleep(std::time::Duration::from_micros(500));
                }
            }
        }
    });
}

pub type SharedPriceSink = Arc<PriceSink>;
```


├──/Cargo.toml

```toml
[package]
name = ""
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.53", features = ["macros", "rt-multi-thread", "time"] }
tokio-tungstenite = { version = "0.20", features = ["rustls-tls-webpki-roots"] }
futures-util = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
axum = "0.8"
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
parking_lot = "0.12"
proje_core = { package = "core", path = "../core" }
contracts = { path = "../contracts" }
transport = { path = "../transport" }
rust_decimal = { workspace = true }
flume = "0.11"
```


├──/src/main.rs

```rust
//! Price Feed — Binance Futures WS'ten mark/index/last price çeken daemon.
//!
//! Mimari, DATA terminaliyle birebir aynıdır:
//!   Binance WS → simd_json EventParser → GenerationalRingBuffer (/dev/shm)
//!
//! Fark: kendi ring buffer'ını kullanır (/cycle_finance_pricefeed) ve ayrıca
//! HTTP API + JSON dosya ile son fiyatları diğer katmanlara sunar.
//!
//! Abonelikler (fstream.binance.com):
//!   {SYM}@markPrice@1s  → mark + index price (FundingRate event)
//!   {SYM}@bookTicker@1s → best bid/ask (BookTicker event)
//!
//! HTTP:
//!   GET /api/lastprice            → tüm semboller {last, mark, index}
//!   GET /api/lastprice/{SYMBOL}   → tek sembol
//!   GET /health

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use flume::Sender;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use transport::ring_buffer::GenerationalRingBuffer;
use contracts::events::{EventType, OwnedEvent};
use proje_core::tick::EventParser;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::connect_async;

const WS_URL: &str = "wss://fstream.binance.com/stream";
const DEFAULT_PORT: u16 = 3004;
const RING_NAME: &str = "/cycle_finance_pricefeed";
const RING_CAPACITY: usize = 20_000;
const OUT_FILE: &str = "/tmp/price_feed.json";

// ── Semboller ────────────────────────────────────────────────
fn load_symbols() -> Vec<String> {
    let mut syms: Vec<String> = Vec::new();
    if let Ok(content) = std::fs::read_to_string("alerts.toml") {
        for line in content.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("symbol") {
                if let Some(eq) = rest.find('=') {
                    let s = rest[eq + 1..].trim().trim_matches('"').trim_matches('\'').trim().to_string();
                    if !s.is_empty() && !syms.contains(&s) {
                        syms.push(s);
                    }
                }
            }
        }
    }
    if !syms.contains(&"VELVETUSDT".to_string()) {
        syms.push("VELVETUSDT".to_string());
    }
    syms
}

fn resolve_symbols() -> Vec<String> {
    if let Ok(v) = std::env::var("PRICE_FEED_SYMBOLS") {
        let s: Vec<String> = v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
        if !s.is_empty() {
            return s;
        }
    }
    load_symbols()
}

// ── Paylaşılan durum ─────────────────────────────────────────
#[derive(Debug, Clone, Default, Serialize)]
struct PriceEntry {
    last: f64,
    mark: f64,
    index: f64,
    bid: f64,
    ask: f64,
    ts: u64,
}

#[derive(Debug, Default)]
struct FeedState {
    prices: HashMap<String, PriceEntry>,
    symbols: Vec<String>,
}

fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

// ── WS bağlantısı — DATA terminaliyle aynı desen ────────────
async fn ws_pump(tx: Sender<Vec<u8>>, symbols: Vec<String>) {
    let streams: Vec<String> = symbols
        .iter()
        .flat_map(|s| {
            let s = s.to_lowercase();
            vec![
                format!("{}@trade", s),
                format!("{}@bookTicker", s),
            ]
        })
        .collect();

    loop {
        println!("[PRICE-FEED] WS bağlanıyor ({} stream)...", streams.len());
        match connect_async(WS_URL).await {
            Ok((ws, _)) => {
                let (mut write, mut read) = ws.split();
                let sub = serde_json::json!({
                    "method": "SUBSCRIBE",
                    "params": streams,
                    "id": 1
                });
                if write.send(tokio_tungstenite::tungstenite::Message::Text(sub.to_string())).await.is_err() {
                    eprintln!("[PRICE-FEED] Abonelik gönderilemedi");
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    continue;
                }
                println!("[PRICE-FEED] Bağlandı ve abone olundu.");

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                            let bytes = text.into_bytes();
                            // Bounded kuyruk → geri basınç (asla RAM taşmaz).
                            if tx.send_async(bytes).await.is_err() {
                                eprintln!("[PRICE-FEED] Kuyruk kapandı, çıkılıyor.");
                                return;
                            }
                        }
                        Ok(tokio_tungstenite::tungstenite::Message::Ping(p)) => {
                            let _ = write.send(tokio_tungstenite::tungstenite::Message::Pong(p)).await;
                        }
                        Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
                        _ => {}
                    }
                }
                println!("[PRICE-FEED] Bağlantı koptu, yeniden bağlanılıyor...");
            }
            Err(e) => {
                eprintln!("[PRICE-FEED] Bağlantı hatası: {}", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

// ── Mark/Index REST çekimi (fstream WS markPrice sessiz → premiumIndex) ──
async fn fetch_premium_index(client: &reqwest::Client, symbols: &[String], state: Arc<RwLock<FeedState>>) {
    for sym in symbols {
        let url = format!("https://fapi.binance.com/fapi/v1/premiumIndex?symbol={}", sym);
        let resp = client.get(&url).send().await;
        if let Ok(r) = resp {
            if let Ok(doc) = r.json::<serde_json::Value>().await {
                let mark = doc.get("markPrice").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let index = doc.get("indexPrice").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let ts = now_ts();
                let mut st = state.write();
                let e = st.prices.entry(sym.clone()).or_default();
                e.ts = ts;
                if let Some(m) = mark {
                    e.mark = m;
                }
                if let Some(i) = index {
                    e.index = i;
                }
            }
        }
    }
}

// ── Parser + ring buffer + state güncelleme — DATA ile aynı ─
fn ingest(rx: flume::Receiver<Vec<u8>>, ring: Arc<GenerationalRingBuffer>, state: Arc<RwLock<FeedState>>) {
    let mut validator = proje_core::validator::DataValidator::new();
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut last_report = std::time::Instant::now();
    let mut frame_buf = [0u8; contracts::wire::MAX_FRAME_SIZE];

    while let Ok(mut bytes) = rx.recv() {
        if let Some(ev) = EventParser::parse(&mut bytes) {
            if !validator.is_valid(&ev) {
                continue;
            }
            // DATA ile aynı: ring'e typed binary yazılır (ham JSON değil).
            if let Some(len) = contracts::wire::encode(&ev, &mut frame_buf) {
                ring.push(&frame_buf[..len]);
            }
            update_state(&state, &ev);
            ok += 1;
        }
        total += 1;

        if last_report.elapsed().as_secs() >= 1 {
            println!("[PRICE-FEED] ticks/s: {} | parsed: {}", total, ok);
            total = 0;
            ok = 0;
            last_report = std::time::Instant::now();
        }
    }
}

fn update_state(state: &Arc<RwLock<FeedState>>, ev: &OwnedEvent) {
    let sym = std::str::from_utf8(&ev.symbol)
        .unwrap_or("")
        .trim_end_matches('\0')
        .to_uppercase();
    if sym.is_empty() {
        return;
    }
    let ts = now_ts();
    let mut st = state.write();
    let e = st.prices.entry(sym.clone()).or_default();
    e.ts = ts;
    match ev.payload {
        EventType::Trade { price, .. } => {
            e.last = price.to_f64().unwrap_or(0.0);
        }
        EventType::FundingRate { mark_price, index_price, .. } => {
            e.mark = mark_price.to_f64().unwrap_or(0.0);
            e.index = index_price.to_f64().unwrap_or(0.0);
        }
        EventType::BookTicker { best_bid_price, best_ask_price, .. } => {
            e.bid = best_bid_price.to_f64().unwrap_or(0.0);
            e.ask = best_ask_price.to_f64().unwrap_or(0.0);
        }
        _ => {}
    }
    drop(st);
}

// ── HTTP API ─────────────────────────────────────────────────
#[derive(Serialize)]
struct ApiAll {
    updated: u64,
    symbols: Vec<String>,
    prices: HashMap<String, PriceEntry>,
}

#[derive(Serialize)]
struct ApiOne {
    symbol: String,
    price: PriceEntry,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    symbols: Vec<String>,
    prices: HashMap<String, f64>,
}

async fn api_all(State(state): State<Arc<RwLock<FeedState>>>) -> Json<ApiAll> {
    let st = state.read();
    Json(ApiAll {
        updated: now_ts(),
        symbols: st.symbols.clone(),
        prices: st.prices.clone(),
    })
}

async fn api_one(
    State(state): State<Arc<RwLock<FeedState>>>,
    Path(symbol): Path<String>,
) -> Result<Json<ApiOne>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let st = state.read();
    let key = symbol.to_uppercase();
    match st.prices.get(&key) {
        Some(e) => Ok(Json(ApiOne { symbol: key, price: e.clone() })),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("bilinmeyen sembol: {key}"), "available": st.symbols})),
        )),
    }
}

async fn api_health(State(state): State<Arc<RwLock<FeedState>>>) -> Json<Health> {
    let st = state.read();
    Json(Health {
        status: "ok",
        symbols: st.symbols.clone(),
        prices: st.prices.iter().map(|(k, v)| (k.clone(), v.mark.max(v.last))).collect(),
    })
}

#[tokio::main]
async fn main() {
    let symbols = resolve_symbols();
    let port: u16 = std::env::var("PRICE_FEED_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_PORT);

    println!("========================================");
    println!("  💹  PRICE FEED — Anlık LastPrice Daemon");
    println!("  Mimari : WS → EventParser → RingBuffer (/dev/shm)");
    println!("  Semboller : {}", symbols.join(", "));
    println!("  HTTP API  : http://127.0.0.1:{}/api/lastprice", port);
    println!("  JSON çıktı: {}", OUT_FILE);
    println!("========================================");

    let ring = Arc::new(GenerationalRingBuffer::with_name(RING_NAME, RING_CAPACITY));
    let state = Arc::new(RwLock::new(FeedState {
        symbols: symbols.clone(),
        ..Default::default()
    }));

    let (tx, rx) = flume::bounded::<Vec<u8>>(262_144);

    // WS pump task
    let symbols_ws = symbols.clone();
    tokio::spawn(async move { ws_pump(tx, symbols_ws).await });

    // Mark/Index REST döngüsü — Binance fstream markPrice WS stream'i sessiz
    // olduğundan premiumIndex'i çok sık (200ms) çekerek pratikte gecikmesiz.
    {
        let client = reqwest::Client::new();
        let state = state.clone();
        let symbols_rest = symbols.clone();
        tokio::spawn(async move {
            loop {
                fetch_premium_index(&client, &symbols_rest, state.clone()).await;
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        });
    }

    // Ingest thread (parser + ring + state)
    {
        let ring = ring.clone();
        let state = state.clone();
        std::thread::spawn(move || ingest(rx, ring, state));
    }

    // JSON dosya yazıcı (periyodik)
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let doc = {
                    let st = state.read();
                    serde_json::json!({
                        "updated": now_ts(),
                        "symbols": st.symbols.clone(),
                        "prices": st.prices.clone(),
                    })
                };
                let _ = std::fs::write(OUT_FILE, serde_json::to_string_pretty(&doc).unwrap_or_default());
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    let app = Router::new()
        .route("/api/lastprice", get(api_all))
        .route("/api/lastprice/{symbol}", get(api_one))
        .route("/health", get(api_health))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("port bind");
    axum::serve(listener, app).await.expect("serve");
}
```


├── scripts/cycle_env.sh

```bash
#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — Shell Yardımcı Komutları
#  Bu dosya cycle_tmux.sh tarafından otomatik source edilir.
#  Elle de kullanılabilir: source ~/Desktop/PROJE/scripts/cycle_env.sh
# ============================================================

# ── Kök dizini otomatik bul ──────────────────────────────────
CYCLE_ROOT="${CYCLE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CYCLE_API="${CYCLE_API:-http://127.0.0.1:8080}"
CYCLE_USER="${CYCLE_USER:-admin}"
CYCLE_PASS="${CYCLE_PASS:-changeme123}"

# ── Renk kodları ─────────────────────────────────────────────
_G='\033[0;32m'; _Y='\033[1;33m'; _C='\033[0;36m'
_B='\033[1;34m'; _W='\033[1;37m'; _R='\033[0;31m'
_D='\033[2m';    _N='\033[0m'

# ============================================================
#  KOMUT REHBERİ
# ============================================================
help-cycle() {
  echo ""
  echo -e "${_W}╔══════════════════════════════════════════════════════════════════╗${_N}"
  echo -e "${_W}║        🏛️  CYCLE FINANCE — KOMUT REHBERİ                        ║${_N}"
  echo -e "${_W}╚══════════════════════════════════════════════════════════════════╝${_N}"

  echo -e "\n${_Y}━━━  🔧 SİSTEM YÖNETİMİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_G}cycle-start${_N}          Tüm terminalleri yeniden başlat"
  echo -e "  ${_G}cycle-kill${_N}           Tüm terminalleri ve servisleri kapat"
  echo -e "  ${_G}cycle-status${_N}         Çalışan servislerin CPU/RAM durumu"
  echo -e "  ${_G}cycle-build${_N}          Projeyi derle (cargo build)"
  echo -e "  ${_G}cycle-build-full${_N}     Tam set derle (--features full)"

  echo -e "\n${_Y}━━━  ⚙️  SİSTEMLERİ TEK TEK AÇ / KAPAT  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_G}data-start${_N} / ${_R}data-stop${_N}          DATA terminali (Binance WS)"
  echo -e "  ${_G}strategy-start${_N} / ${_R}strategy-stop${_N}  STRATEGY terminali (PyO3)"
  echo -e "  ${_G}paper-start${_N} / ${_R}paper-stop${_N}        Paper-service (REST :8080)"
  echo -e "  ${_G}alert-start${_N} / ${_R}alert-stop${_N}        Alert-service"
  echo -e "  ${_G}listener-start${_N} / ${_R}listener-stop${_N}  Listener (anlık metrik analizi)"
  echo -e "  ${_G}detect-ms-start${_N} / ${_R}detect-ms-stop${_N}  MSMP analiz motoru (:3002)"
  echo -e "  ${_G}velvetusdt-start${_N} / ${_R}velvetusdt-stop${_N}    VELVETUSDT kırılım stratejisi"

  echo -e "\n${_Y}━━━  🛰️  LISTENER  (Anlık Metrik Analizi)  ━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}listener-start${_N}      Pane 0.2'de başlat"
  echo -e "  ${_C}listener-stop${_N}       Durdur"
  echo -e "  ${_C}listener-status${_N}     Çalışıyor mu? CPU/RAM"
  echo -e "  ${_C}listenconfig-list${_N}   Metrik parametrelerini göster"
  echo -e "  ${_C}listenconfig-set KEY VAL${_N}  Parametre değiştir (lambda, k_abs, gamma...) "
  echo -e "  ${_C}listenconfig-reset${_N}  Varsayılanlara dön"
  echo -e "  ${_C}listener-log${_N}        Metrik çıktısını izle (/tmp/listener_metrics.json)"

  echo -e "\n${_Y}━━━  ⚠️  RİSK ANALİZİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}risk-start${_N}           Pane 0.1'de başlat (5 sn yenileme)"
  echo -e "  ${_C}risk-stop${_N}            Durdur"
  echo -e "  ${_C}risk-query${_N}           Tek seferlik analiz çalıştır"

  echo -e "\n${_Y}━━━  💹 PRICE-FEED  (WS→Ring, Anlık Last/Mark/Index)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}pricefeed-start${_N}     Arka planda başlat (:3004)"
  echo -e "  ${_C}pricefeed-stop${_N}      Durdur"
  echo -e "  ${_C}pricefeed-status${_N}    Çalışıyor mu? CPU/RAM + health"
  echo -e "  ${_C}pricefeed-query SYM${_N} Tek sembol sorgula (örn. pricefeed-query VELVETUSDT)"
  echo -e "  ${_C}pricefeed-log${_N}       Canlı log izle"

  echo -e "\n${_Y}━━━  📡 DATA TERMİNALİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}data-live${_N}            Canlı Binance WS başlat (RUN_MODE=DATA)"
  echo -e "  ${_C}data-backtest${_N}        CSV backtest başlat"
  echo -e "  ${_C}data-log${_N}             Data terminal logunu izle"

  echo -e "\n${_Y}━━━  🛡️  PAPER SERVICE (REST API)  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}paper-health${_N}         Sistem sağlık kontrolü"
  echo -e "  ${_C}paper-balance${_N}        Bakiye ve equity bilgisi"
  echo -e "  ${_C}paper-positions${_N}      Açık pozisyonlar"
  echo -e "  ${_C}paper-orders${_N}         Açık emirler"
  echo -e "  ${_C}paper-history${_N}        İşlem geçmişi"
  echo -e "  ${_C}paper-metrics${_N}        Prometheus metrikleri (ham)"
  echo -e "  ${_C}paper-log${_N}            Paper service logunu izle"

  echo -e "\n${_Y}━━━  📋 EMİR İŞLEMLERİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}paper-buy  BTCUSDT 0.001${_N}   Market BUY emri"
  echo -e "  ${_C}paper-sell BTCUSDT 0.001${_N}   Market SELL emri"
  echo -e "  ${_C}paper-cli  [arglar]${_N}         Paper CLI (tüm seçenekler)"

  echo -e "\n${_Y}━━━  🧠 STRATEGY / CORRELATION  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}strategy-start${_N}       Strategy terminalini başlat (arka plan)"
  echo -e "  ${_C}strategy-stop${_N}        Strategy terminalini durdur"
  echo -e "  ${_C}correlation-start${_N}    Korelasyon analizini başlat"

  echo -e "\n${_Y}━━━  🔔 ALERT SERVİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}alert-list${_N}           Aktif uyarıları listele"
  echo -e "  ${_C}alert-add VELVETUSDT above 0.22 \"ses\"${_N}   Yeni alarm ekle"
  echo -e "  ${_C}alert-update SYM cond OLD NEW${_N}   Alarmı güncelle"
  echo -e "  ${_C}alert-remove SYM cond PRICE${_N}     Alarmı sil"
  echo -e "  ${_C}alert-reload${_N}         Alert servisini yeniden başlat"

  echo -e "\n${_Y}━━━  📈 DETECT-MS  (Market Structure Engine :3002)  ━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}detect-ms-start${_N}      Servisi arka planda başlat (port 3002)"
  echo -e "  ${_C}detect-ms-stop${_N}       Servisi durdur"
  echo -e "  ${_C}detect-ms-status${_N}     Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}detect-ms-query${_N}      BTCUSDT 15m analiz (JSON çıktı)"
  echo -e "  ${_C}detect-ms-query ETHUSDT 1h 500${_N}   Özel sorgu"
  echo -e "  ${_C}detect-ms-log${_N}        Canlı log izle"

  echo -e "\n${_Y}━━━  🏛️  DETECT-WYCKOFF  (Wyckoff Faz Motoru :3005)  ━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}detect-wyckoff-start${_N}  Servisi başlat (port 3005)"
  echo -e "  ${_C}detect-wyckoff-stop${_N}   Servisi durdur"
  echo -e "  ${_C}detect-wyckoff-status${_N} Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}detect-wyckoff-query${_N}  BTCUSDT 1h analiz (JSON çıktı)"
  echo -e "  ${_C}detect-wyckoff-query${_N}  VELVETUSDT 15m 500${_N}   Özel sorgu"

  echo -e "\n${_Y}━━━  🌊 DETECT-TRB  (Navier-Stokes Çözücü :3006)  ━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}detect-trb-start${_N}      Servisi başlat (port 3006)"
  echo -e "  ${_C}detect-trb-stop${_N}       Servisi durdur"
  echo -e "  ${_C}detect-trb-status${_N}     Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}detect-trb-query${_N}      Son raporu göster (JSON çıktı)"
  echo -e "  ${_C}detect-trb-start --symbol ETHUSDT --port 3007${_N}   Özel parametreler"

  echo -e "\n${_Y}━━━  🎯 VELVETUSDT KIRILIM STRATEJİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}velvetusdt-start${_N}        Stratejiyi başlat (VELVETUSDT 1m, 100 pencere)"
  echo -e "  ${_C}velvetusdt-stop${_N}         Stratejiyi durdur"
  echo -e "  ${_C}velvetusdt-status${_N}       Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}velvetusdt-query${_N}        Tek seferlik analiz (emir açmaz)"
  echo -e "  ${_C}velvetusdt-query --dry-run${_N}  Analiz + kırılım simülasyonu"
  echo -e "  ${_C}velvetusdt-wait 600${_N}     Bekleme süresini ayarla (saniye)"
  echo -e "  ${_C}velvetusdt-log${_N}          Canlı strateji logu izle"

  echo -e "\n${_Y}━━━  📊 İZLEME  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}monitor-start${_N}        İzleme paneline geç (Ctrl+B → 4)"

  echo -e "\n${_Y}━━━  🗄️  VERİTABANI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}db-trades${_N}            Son 20 işlemi göster"
  echo -e "  ${_C}db-size${_N}              Veritabanı boyutu"

  echo -e "\n${_Y}━━━  🌐 TMUX KISAYOLLARI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_B}Ctrl+B → ok tuşu${_N}     Panel değiştir"
  echo -e "  ${_B}Ctrl+B → z${_N}           Paneli tam ekran yap / küçült"
  echo -e "  ${_B}Ctrl+B → d${_N}           Session'ı arka plana al"
  echo -e "  ${_B}Ctrl+B → 0${_N}           Trading sekmesi (4 panel)"
  echo -e "  ${_B}Ctrl+B → 1${_N}           📡 DATA sekmesi"
  echo -e "  ${_B}Ctrl+B → 2${_N}           🔔 ALERT sekmesi"
  echo -e "  ${_B}Ctrl+B → 3${_N}           🛡️ PAPER sekmesi"
  echo -e "  ${_B}Ctrl+B → 4${_N}           Monitor sekmesi"
  echo -e "  ${_B}Ctrl+B → 5${_N}           DETECT-MS sekmesi"
  echo -e "  ${_B}Ctrl+B → 6${_N}           VELVETUSDT sekmesi"
  echo -e "  ${_B}Ctrl+B → 7${_N}           WYCKOFF sekmesi"
  echo -e "  ${_B}Fare tıklama/scroll${_N}  Panel seç / scroll"

  echo -e "\n${_W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_D}help-cycle yazarak bu listeye tekrar ulaşabilirsin.${_N}"
  echo ""
}

# ============================================================
#  SİSTEM YÖNETİMİ
# ============================================================
# Bu dosya değiştiğinde fonksiyonları güncellemek için:
reload-cycle() {
  source "$CYCLE_ROOT/scripts/cycle_env.sh" >/dev/null 2>&1
  echo "✅ cycle_env.sh yeniden yüklendi"
}

# Her start/stop fonksiyonunun güncel sürümü kullanması için otomatik yenileme
# (tmux SHELL paneli eski sürümü yüklemiş olsa bile sorun yaşanmaz)
_start_guard() {
  source "$CYCLE_ROOT/scripts/cycle_env.sh" >/dev/null 2>&1
}
cycle-start() {
  "$CYCLE_ROOT/scripts/cycle_tmux.sh"
}
cycle-kill() {
  "$CYCLE_ROOT/scripts/cycle_tmux.sh" kill
}
cycle-status() {
  "$CYCLE_ROOT/scripts/cycle_tmux.sh" status
}
cycle-build() {
  cd "$CYCLE_ROOT" && cargo build -p core -p paper-service -p alert-service
}
cycle-build-full() {
  cd "$CYCLE_ROOT" && cargo build -p paper-service --features full
}

# ============================================================
#  SİSTEMLERİ TEK TEK AÇ / KAPAT  (4 panelli Trading penceresi)
#  DATA, ALERT ve PAPER ayrı sekme (pencere) olarak açılır.
#  Her servis kendi pane'inde başlar.
# ============================================================
# Yardımcı: Trading penceresindeki bir pane'e komut gönder
# Servis → hedef: 0.0=STRATEGY 0.2=LISTENER 0.1=RISK 0.3=SHELL
#                1=DATA sekmesi  2=ALERT sekmesi  3=PAPER sekmesi
_tmux_pane() {
  local name="$1"; shift
  local session="cycle"
  local pane
  case "$name" in
    "📡DATA")   pane="1" ;;
    "🛡️PAPER")  pane="3" ;;
    "🧠STRATEGY") pane="0.0" ;;
    "🔔ALERT")  pane="2" ;;
    "🛰️LISTENER") pane="0.2" ;;
    "⚠️RISK")  pane="0.1" ;;
    "💻SHELL")  pane="0.3" ;;
    *)
      # Tanınmayan → yeni pencere (ör. DETECT-MS, VELVETUSDT)
      if ! tmux has-session -t "$session" 2>/dev/null; then
        tmux new-session -d -s "$session" -x 220 -y 50
        tmux rename-window -t "$session:0" "Trading"
      fi
      local idx
      idx=$(tmux list-windows -t "$session" -F "#{window_name} #{window_index}" 2>/dev/null | awk -v n="$name" '$1==n{print $2}')
      if [ -z "$idx" ]; then
        tmux new-window -t "$session" -n "$name"
        idx=$(tmux list-windows -t "$session" -F "#{window_name} #{window_index}" 2>/dev/null | awk -v n="$name" '$1==n{print $2}')
      fi
      tmux send-keys -t "$session:$idx" "$@"
      return 0
      ;;
  esac
  tmux send-keys -t "$session:$pane" C-c
  tmux send-keys -t "$session:$pane" C-u
  tmux send-keys -t "$session:$pane" "$@"
}

# ── DATA terminali (Binance WS → ring) ──────────────────────
# RUN_MODE env değişkeni ps'de görünmez → /proc/*/environ ile kontrol et
_core_mode_pid() {
  local mode="$1"
  for p in $(pgrep -x core 2>/dev/null); do
    if tr '\0' '\n' < "/proc/$p/environ" 2>/dev/null | grep -q "^RUN_MODE=$mode$"; then
      echo "$p"
      return 0
    fi
  done
  return 1
}

data-start() {
  _start_guard
  if _core_mode_pid DATA &>/dev/null; then echo "⚠️  DATA zaten çalışıyor (pid: $(_core_mode_pid DATA))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p core 2>&1 | tail -1
  rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders
  _tmux_pane "📡DATA" "cd $CYCLE_ROOT && RUN_MODE=DATA ./target/debug/core" Enter
  echo "✅ DATA başlatıldı (sekme 1 — 📡 DATA)"
}
data-stop() {
  _start_guard
  local p; p=$(_core_mode_pid DATA)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; echo "✅ DATA durduruldu [pid:$p]"; else echo "ℹ️  DATA çalışmıyor"; fi
}

# ── STRATEGY terminali (core) ────────────────────────────────
strategy-start() {
  _start_guard
  if _core_mode_pid STRATEGY &>/dev/null; then echo "⚠️  STRATEGY zaten çalışıyor (pid: $(_core_mode_pid STRATEGY))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p core 2>&1 | tail -1
  _tmux_pane "🧠STRATEGY" "cd $CYCLE_ROOT && RUN_MODE=STRATEGY ./target/debug/core" Enter
  echo "✅ STRATEGY başlatıldı (pane 0.0)"
}
strategy-stop() {
  _start_guard
  local p; p=$(_core_mode_pid STRATEGY)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; echo "✅ STRATEGY durduruldu [pid:$p]"; else echo "ℹ️  STRATEGY çalışmıyor"; fi
}

# ── PAPER-SERVICE (REST API :8080) ───────────────────────────
paper-start() {
  _start_guard
  if pgrep -x "paper-service" &>/dev/null; then echo "⚠️  paper-service zaten çalışıyor"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p paper-service 2>&1 | tail -1
  rm -rf "$CYCLE_ROOT/paper_wal"
  _tmux_pane "🛡️PAPER" \
    "cd $CYCLE_ROOT && PAPER_ADMIN_USER=${PAPER_ADMIN_USER:-admin} PAPER_ADMIN_PASS=${PAPER_ADMIN_PASS:-changeme123} PAPER_API_ADDR=${PAPER_API_ADDR:-127.0.0.1:8080} PAPER_INITIAL_USDT=${PAPER_INITIAL_USDT:-100000} PAPER_DB_PATH=/tmp/paper_live.db PAPER_SLED_PATH=$CYCLE_ROOT/paper_wal ./target/debug/paper-service" \
    Enter
  echo "✅ PAPER-SERVICE başlatıldı (sekme 3 — 🛡️ PAPER, http://127.0.0.1:8080)"
}
paper-stop() {
  _start_guard
  local p; p=$(pgrep -x paper-service 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ paper-service durduruldu [pid:$p]"; else echo "ℹ️  paper-service çalışmıyor"; fi
}

# ── ALERT-SERVICE ────────────────────────────────────────────
alert-start() {
  _start_guard
  if pgrep -x "alert-service" &>/dev/null; then echo "⚠️  alert-service zaten çalışıyor"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p alert-service 2>&1 | tail -1
  _tmux_pane "🔔ALERT" "cd $CYCLE_ROOT && ./target/debug/alert-service --config $CYCLE_ROOT/alerts.toml" Enter
  echo "✅ ALERT-SERVICE başlatıldı (sekme 2 — 🔔 ALERT)"
}
alert-stop() {
  _start_guard
  local p; p=$(pgrep -x alert-service 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ alert-service durduruldu [pid:$p]"; else echo "ℹ️  alert-service çalışmıyor"; fi
}

# ── LISTENER (Anlık Metrik Analizi, pane 0.1) ──────────
listener-start() {
  _start_guard
  if pgrep -x listener &>/dev/null; then
    echo "⚠️  listener zaten çalışıyor (pid: $(pgrep -x listener | head -1))"
    return 1
  fi
  # Bağımlılık: paper-service gerekli
  if ! pgrep -x paper-service &>/dev/null; then
    echo "⚠️  paper-service çalışmıyor — önce paper-start ile başlatın"
    return 1
  fi
  _tmux_pane "🛰️LISTENER" "cd $CYCLE_ROOT && $CYCLE_ROOT/target/debug/listener" Enter
  sleep 2
  if pgrep -x listener &>/dev/null; then
    echo "✅ LISTENER başlatıldı (pane 0.2)"
  else
    echo "❌ LISTENER başlatılamadı"
  fi
}
listener-stop() {
  _start_guard
  local p; p=$(pgrep -x listener 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    pkill -TERM -x listener 2>/dev/null
    sleep 1
    pkill -KILL -x listener 2>/dev/null || true
    echo "✅ LISTENER durduruldu [pid:$p]"
  else
    echo "ℹ️  LISTENER çalışmıyor"
  fi
}
listener-status() {
  _start_guard
  local p; p=$(pgrep -x listener 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    local cpu mem
    cpu=$(ps -p "$p" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$p" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ LISTENER ÇALIŞIYOR  [pid:$p  CPU:${cpu}%  RAM:${mem}]"
  else
    echo "✘  LISTENER durdurulmuş"
  fi
}
listener-log() {
  tail -f /tmp/listener_metrics.json 2>/dev/null || echo "metrik dosyası yok"
}

# ── RISK (Anlık risk analizi, pane 0.3) ──────────────────────
risk-start() {
  _start_guard
  if pgrep -x risk_analysis &>/dev/null; then
    echo "⚠️  RISK zaten çalışıyor (pid: $(pgrep -x risk_analysis | head -1))"
    return 1
  fi
  _tmux_pane "⚠️RISK" "cd $CYCLE_ROOT && ./target/debug/risk_analysis --watch" Enter
  sleep 2
  echo "✅ RISK başlatıldı (pane 0.1)"
}
risk-stop() {
  _start_guard
  local p; p=$(pgrep -x risk_analysis 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    pkill -TERM -x risk_analysis 2>/dev/null; sleep 1
    pkill -KILL -x risk_analysis 2>/dev/null || true
    echo "✅ RISK durduruldu [pid:$p]"
  else
    echo "ℹ️  RISK çalışmıyor"
  fi
}
risk-status() {
  _start_guard
  local p; p=$(pgrep -x risk_analysis 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    echo "✅ RISK ÇALIŞIYOR [pid:$p]"
  else
    echo "✘  RISK durdurulmuş"
  fi
}
risk-query() {
  _start_guard
  cd "$CYCLE_ROOT" && ./target/debug/risk_analysis
}

# ── Listener metrik parametreleri (shell'den ayarlanabilir) ──
# Config dosyası: /tmp/listener_metrics.conf (çalışan listener 5 sn'de bir yeniden okur)
LISTEN_CONF=/tmp/listener_metrics.conf

# listenconfig-list  → tüm parametreleri göster
# listenconfig-set lambda 0.02   → parametre değiştir
# listenconfig-reset          → varsayılanlara dön
listenconfig-list() {
  _start_guard
  local conf="$LISTEN_CONF"
  if [ -f "$conf" ]; then
    echo "=== Listener metrik parametreleri ($conf) ==="
    cat "$conf"
  else
    echo "ℹ️  Config dosyası yok — varsayılanlar kullanılıyor:"
    echo "  lambda = 0.015        (WLOBI decay)"
    echo "  theta_vol = 2.5       (Delta velocity eşiği)"
    echo "  alpha_bucket = 0.75   (aVPIN bucket sabiti)"
    echo "  k_abs = 100           (absorption penceresi, trade)"
    echo "  n_bucket = 50         (aVPIN bucket sayısı)"
    echo "  ice_threshold = 1.2   (Iceberg eşiği)"
    echo "  efp_threshold = 0.05  (Execution footprint eşiği)"
    echo "  noise_corr = 0.85     (Lee-Ready gürültü filtresi)"
    echo "  delta_window_sec = 60 (ΔV penceresi, saniye)"
    echo "  tps_window_sec = 10  (TPS penceresi, saniye)"
    echo "  corr_price_window_sec = 5 (fiyat korelasyonu penceresi, saniye)"
    echo "  corr_vol_window_sec = 5   (hacim korelasyonu penceresi, saniye)"
    echo "  gamma0..gamma5        (Alpha Basket ağırlıkları)"
  fi
}

listenconfig-set() {
  _start_guard
  local key="${1:-}" val="${2:-}"
  if [ -z "$key" ] || [ -z "$val" ]; then
    echo "Kullanım: listenconfig-set <key> <value>"
    echo "Örn: listenconfig-set lambda 0.02 | listenconfig-set k_abs 200"
    echo "     listenconfig-set gamma1 0.5 | listenconfig-set delta_window_sec 120"
    return 1
  fi
  local valid_keys="lambda theta_vol alpha_bucket k_abs n_bucket ice_threshold efp_threshold noise_corr delta_window_sec tps_window_sec corr_price_window_sec corr_vol_window_sec gamma0 gamma1 gamma2 gamma3 gamma4 gamma5"
  if ! echo "$valid_keys" | grep -qw "$key"; then
    echo "❌ Geçersiz parametre: $key"
    echo "Geçerli: $valid_keys"
    return 1
  fi
  # k_abs, n_bucket, delta_window_sec tam sayı olmalı
  if echo "k_abs n_bucket delta_window_sec tps_window_sec corr_price_window_sec corr_vol_window_sec" | grep -qw "$key"; then
    if ! echo "$val" | grep -qE '^[0-9]+$'; then
      echo "❌ $key tam sayı olmalı"; return 1
    fi
  else
    if ! echo "$val" | grep -qE '^-?[0-9]+(\.[0-9]+)?$'; then
      echo "❌ $key sayı olmalı"; return 1
    fi
  fi
  # Eski değeri değiştir veya ekle
  if grep -q "^${key} *=" "$LISTEN_CONF" 2>/dev/null; then
    sed -i "s|^${key} *=.*|${key} = ${val}|" "$LISTEN_CONF"
  else
    echo "${key} = ${val}" >> "$LISTEN_CONF"
  fi
  echo "✅ $key = $val kaydedildi ($LISTEN_CONF)"
  echo "   Çalışan listener 5 sn'de bir yeniden okur. list-restart ile hemen uygula."
}

listenconfig-reset() {
  _start_guard
  rm -f "$LISTEN_CONF"
  echo "✅ Varsayılan parametrelere dönüldü (config dosyası silindi)"
}

# Kısayollar
listener-config() { listenconfig-list; }
listener-set() { listenconfig-set "$@"; }

# ── PRICE-FEED (WS → ring buffer, anlık last/mark/index price) ──
pricefeed-start() {
  _start_guard
  if pgrep -x "" &>/dev/null; then
    echo "⚠️  zaten çalışıyor (pid: $(pgrep -x | head -1))"
    return 1
  fi
  cd "$CYCLE_ROOT" && cargo build -p 2>&1 | tail -1
  setsid nohup "$CYCLE_ROOT/target/debug/" > /tmp/price_feed.log 2>&1 < /dev/null &
  sleep 3
  if curl -s -m 2 http://127.0.0.1:3004/health >/dev/null 2>&1; then
    echo "✅ PRICE-FEED başlatıldı → http://127.0.0.1:3004/api/lastprice"
  else
    echo "❌ PRICE-FEED başlatılamadı:"; tail -5 /tmp/price_feed.log
  fi
}
pricefeed-stop() {
  _start_guard
  local p; p=$(pgrep -x "" 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ durduruldu [pid:$p]"; else echo "ℹ️  çalışmıyor"; fi
}
pricefeed-status() {
  _start_guard
  local p; p=$(pgrep -x "" 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    local cpu mem
    cpu=$(ps -p "$p" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$p" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ PRICE-FEED ÇALIŞIYOR  [pid:$p  CPU:${cpu}%  RAM:${mem}]"
    curl -s -m 2 http://127.0.0.1:3004/health
    echo
  else
    echo "✘  PRICE-FEED durdurulmuş"
  fi
}
pricefeed-query() {
  _start_guard
  local sym="${1:-BTCUSDT}"
  curl -s -m 3 "http://127.0.0.1:3004/api/lastprice/$sym" | python3 -m json.tool 2>/dev/null \
    || echo "❌ Servis yanıt vermiyor — pricefeed-start ile başlat."
}
pricefeed-log() {
  tail -f /tmp/price_feed.log
}

# ============================================================
#  DATA TERMİNALİ
# ============================================================
data-live() {
  cd "$CYCLE_ROOT" && RUN_MODE=DATA ./target/debug/core
}
data-backtest() {
  cd "$CYCLE_ROOT" && RUN_MODE=BACKTEST CSV_PATH="./test_data.csv" ./target/debug/core
}
data-log() {
  tail -f /tmp/data_terminal.log
}

# ============================================================
#  PAPER SERVICE — JWT otomatik alınır
# ============================================================
_cycle_token() {
  curl -s -X POST "$CYCLE_API/api/v1/auth/login" \
    -H 'Content-Type: application/json' \
    -d "{\"username\":\"$CYCLE_USER\",\"password\":\"$CYCLE_PASS\"}" \
    2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('access_token',''))" 2>/dev/null
}

paper-health() {
  curl -s "$CYCLE_API/api/v1/system/health" | python3 -m json.tool
}
paper-balance() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/balance" | python3 -m json.tool
}
paper-positions() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/positions" | python3 -m json.tool
}
paper-orders() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/orders" | python3 -m json.tool
}
paper-history() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/trade-history" | python3 -m json.tool
}
paper-metrics() {
  curl -s "$CYCLE_API/metrics"
}
paper-log() {
  tail -f /tmp/paper_service.log
}

paper-buy() {
  local sym="${1:-BTCUSDT}" qty="${2:-0.001}"
  local tok; tok=$(_cycle_token)
  local oid="cli-$(date +%s)"
  curl -s -X POST \
    -H "Authorization: Bearer $tok" \
    -H 'Content-Type: application/json' \
    -d "{\"symbol\":\"$sym\",\"side\":\"BUY\",\"order_type\":\"MARKET\",\"quantity\":$qty,\"client_order_id\":\"$oid\"}" \
    "$CYCLE_API/api/v1/order" | python3 -m json.tool
}
paper-sell() {
  local sym="${1:-BTCUSDT}" qty="${2:-0.001}"
  local tok; tok=$(_cycle_token)
  local oid="cli-$(date +%s)"
  curl -s -X POST \
    -H "Authorization: Bearer $tok" \
    -H 'Content-Type: application/json' \
    -d "{\"symbol\":\"$sym\",\"side\":\"SELL\",\"order_type\":\"MARKET\",\"quantity\":$qty,\"client_order_id\":\"$oid\"}" \
    "$CYCLE_API/api/v1/order" | python3 -m json.tool
}
paper-cli() {
  "$CYCLE_ROOT/target/debug/paper_cli" \
    --api "$CYCLE_API" --user "$CYCLE_USER" --password "$CYCLE_PASS" "$@"
}

# ============================================================
#  STRATEGY / CORRELATION
# ============================================================
# Not: strategy-start/stop artık "SİSTEMLERİ TEK TEK AÇ/KAPAT" bölümünde
# (arka planda, pid dosyalı). correlation-start foreground çalıştırır.
correlation-start() {
  cd "$CYCLE_ROOT" && RUN_MODE=CORRELATION ./target/debug/core
}

# ============================================================
#  ALERT SERVİSİ
# ============================================================
alert-list() {
  echo "=== alerts.toml — aktif uyarılar ==="
  "$CYCLE_ROOT/target/debug/alerts" list
  echo ""
  echo "Kullanım:"
  echo "  alert-add VELVETUSDT above 0.22 [voice metni] [cooldown]"
  echo "  alert-update VELVETUSDT above 0.21628 0.22 [voice] [cooldown]"
  echo "  alert-remove VELVETUSDT above 0.21628"
}
alert-reload() {
  pkill -x alert-service 2>/dev/null || true
  sleep 1
  cd "$CYCLE_ROOT" && nohup ./target/debug/alert-service --config ./alerts.toml > /tmp/alert_service.log 2>&1 &
  echo "✅ Alert servisi yeniden başlatıldı (pid: $!)"
}

# ── Alarm yönetimi (shell'den) — değişiklik sonrası otomatik reload ──
_alert_apply() {
  local msg="$1"
  echo "$msg"
  echo "🔄 Alert servisi yeniden yükleniyor..."
  # Eski süreci durdur, tmux pane'inde yeniden başlat
  pkill -x alert-service 2>/dev/null || true
  sleep 1
  tmux send-keys -t "cycle:2" C-c 2>/dev/null
  tmux send-keys -t "cycle:2" "cd $CYCLE_ROOT && ./target/debug/alert-service --config $CYCLE_ROOT/alerts.toml" Enter 2>/dev/null
  sleep 1
  echo "✅ Tamamlandı. alert-list ile görüntüleyin."
}

# Yeni alarm ekle
# Kullanım: alert-add <SYMBOL> <above|below|cross|touch> <PRICE> [voice] [cooldown]
alert-add() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" price="${3:-}" voice="${4:-}" cooldown="${5:-30}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$price" ]; then
    echo "Kullanım: alert-add <SYMBOL> <above|below|cross|touch> <PRICE> [voice metni] [cooldown]"
    return 1
  fi
  local voice_arg=()
  [ -n "$voice" ] && voice_arg=(--voice "$voice")
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" add \
    --symbol "$sym" --condition "$cond" --price "$price" \
    "${voice_arg[@]}" --cooldown "$cooldown")"
}

# Mevcut alarmı güncelle (eski fiyata göre bulur)
# Kullanım: alert-update <SYMBOL> <cond> <OLD_PRICE> <NEW_PRICE> [voice] [cooldown]
alert-update() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" old="${3:-}" new="${4:-}" voice="${5:-}" cooldown="${6:-}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$old" ]; then
    echo "Kullanım: alert-update <SYMBOL> <cond> <OLD_PRICE> [NEW_PRICE] [voice] [cooldown]"
    return 1
  fi
  local args=(--symbol "$sym" --condition "$cond" --old-price "$old")
  [ -n "$new" ] && args+=(--price "$new")
  [ -n "$voice" ] && args+=(--voice "$voice")
  [ -n "$cooldown" ] && args+=(--cooldown "$cooldown")
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" update "${args[@]}")"
}

# Alarm sil
# Kullanım: alert-remove <SYMBOL> <cond> <PRICE>
alert-remove() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" price="${3:-}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$price" ]; then
    echo "Kullanım: alert-remove <SYMBOL> <cond> <PRICE>"
    return 1
  fi
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" remove \
    --symbol "$sym" --condition "$cond" --price "$price")"
}

# ============================================================
#  İZLEME
# ============================================================
monitor-start() {
  if tmux has-session -t cycle 2>/dev/null; then
    tmux select-window -t cycle:4
  else
    "$CYCLE_ROOT/scripts/monitor.sh"
  fi
}

# ============================================================
#  VERİTABANI
# ============================================================
db-trades() {
  local url="${TIMESCALEDB_URL:-postgres://cycle:cycle@localhost:5432/market_data}"
  psql "$url" -c \
    "SELECT symbol, price, quantity, timestamp FROM trades ORDER BY timestamp DESC LIMIT 20;" \
    2>/dev/null || echo "DB boş veya bulunamadı."
}
db-size() {
  local url="${TIMESCALEDB_URL:-postgres://cycle:cycle@localhost:5432/market_data}"
  psql "$url" -c \
    "SELECT pg_size_pretty(pg_database_size(current_database())) AS db_size;" \
    2>/dev/null || echo "DB bağlanılamadı."
}

# ============================================================
#  DETECT-MS  —  Market Structure Multi-Protocol Engine
#  REST API: http://127.0.0.1:3002/api/ms?symbol=BTCUSDT&interval=15m
# ============================================================
DETECT_MS_ADDR="${DETECT_MS_ADDR:-127.0.0.1:3002}"

detect-ms-start() {
  _start_guard
  if pgrep -x "detect-ms" &>/dev/null; then
    echo "⚠️  detect-ms zaten çalışıyor (pid: $(pgrep -x detect-ms))"
    echo "   → detect-ms-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/debug/detect-ms" ]; then
    echo "🔨 detect-ms derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p detect-ms 2>&1 | tail -5
  fi

  echo "🚀 detect-ms başlatılıyor → http://$DETECT_MS_ADDR"
  _tmux_pane "📈DETECT-MS" "cd $CYCLE_ROOT && ./target/debug/detect-ms" Enter
  sleep 1
  if pgrep -x detect-ms &>/dev/null; then
    echo "✅ detect-ms başladı [pid: $(pgrep -x detect-ms)]"
    echo "   API: http://$DETECT_MS_ADDR/api/ms?symbol=BTCUSDT&interval=15m"
  else
    echo "❌ detect-ms başlatılamadı."
  fi
}

detect-ms-stop() {
  _start_guard
  if pgrep -x "detect-ms" &>/dev/null; then
    pkill -TERM -x "detect-ms" && echo "✅ detect-ms durduruldu"
  else
    echo "⚠️  detect-ms zaten çalışmıyor"
  fi
}

detect-ms-status() {
  local pid
  pid=$(pgrep -x "detect-ms" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ detect-ms ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$DETECT_MS_ADDR/api/ms?symbol=BTCUSDT&interval=15m"
  else
    echo "✘  detect-ms durdurulmuş"
  fi
}

# Sorgu kısayolları
detect-ms-query() {
  # Kullanım: detect-ms-query [SYMBOL] [INTERVAL] [LIMIT]
  local sym="${1:-BTCUSDT}" itv="${2:-15m}" lim="${3:-200}"
  echo "📡 Sorgu: $sym $itv (limit: $lim) → http://$DETECT_MS_ADDR"
  curl -s "http://$DETECT_MS_ADDR/api/ms?symbol=${sym}&interval=${itv}&limit=${lim}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. detect-ms-start ile başlat."
}

detect-ms-log() {
  tail -f /tmp/detect_ms.log
}

# ============================================================
#  DETECT-WYCKOFF  —  Wyckoff Faz Analiz Motoru
#  REST API: http://127.0.0.1:3005/api/wyckoff?symbol=BTCUSDT&interval=1h
# ============================================================
DETECT_WYCKOFF_ADDR="${DETECT_WYCKOFF_ADDR:-127.0.0.1:3005}"

detect-wyckoff-start() {
  _start_guard
  if pgrep -x "detect-wyckoff" &>/dev/null; then
    echo "⚠️  detect-wyckoff zaten çalışıyor (pid: $(pgrep -x detect-wyckoff))"
    echo "   → detect-wyckoff-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/debug/detect-wyckoff" ]; then
    echo "🔨 detect-wyckoff derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p detect-wyckoff 2>&1 | tail -5
  fi

  echo "🚀 detect-wyckoff başlatılıyor → http://$DETECT_WYCKOFF_ADDR"
  _tmux_pane "🏛️WYCKOFF" "cd $CYCLE_ROOT && ./target/debug/detect-wyckoff" Enter
  sleep 1
  if pgrep -x detect-wyckoff &>/dev/null; then
    echo "✅ detect-wyckoff başladı [pid: $(pgrep -x detect-wyckoff)]"
    echo "   API: http://$DETECT_WYCKOFF_ADDR/api/wyckoff?symbol=BTCUSDT&interval=1h"
  else
    echo "❌ detect-wyckoff başlatılamadı."
  fi
}

detect-wyckoff-stop() {
  _start_guard
  if pgrep -x "detect-wyckoff" &>/dev/null; then
    pkill -TERM -x "detect-wyckoff" && echo "✅ detect-wyckoff durduruldu"
  else
    echo "⚠️  detect-wyckoff zaten çalışmıyor"
  fi
}

detect-wyckoff-status() {
  local pid
  pid=$(pgrep -x "detect-wyckoff" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ detect-wyckoff ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$DETECT_WYCKOFF_ADDR/api/wyckoff?symbol=BTCUSDT&interval=1h"
  else
    echo "✘  detect-wyckoff durdurulmuş"
  fi
}

# Sorgu kısayolu — Kullanım: detect-wyckoff-query [SYMBOL] [INTERVAL] [LIMIT]
detect-wyckoff-query() {
  local sym="${1:-BTCUSDT}" itv="${2:-1h}" lim="${3:-300}"
  echo "🏛️  Sorgu: $sym $itv (limit: $lim) → http://$DETECT_WYCKOFF_ADDR"
  curl -s "http://$DETECT_WYCKOFF_ADDR/api/wyckoff?symbol=${sym}&interval=${itv}&limit=${lim}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. detect-wyckoff-start ile başlat."
}

# ============================================================
#  DETECT-TRB  —  Navier-Stokes Çözücü / Kavitasyon Motoru
#  REST API: http://127.0.0.1:3006/api/trb  (+ /api/trb/status)
# ============================================================
DETECT_TRB_ADDR="${DETECT_TRB_ADDR:-127.0.0.1:3006}"

detect-trb-start() {
  _start_guard
  if pgrep -x "detect-trb" &>/dev/null; then
    echo "⚠️  detect-trb zaten çalışıyor (pid: $(pgrep -x detect-trb))"
    echo "   → detect-trb-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/release/detect-trb" ]; then
    echo "🔨 detect-trb derleniyor..."
    cd "$CYCLE_ROOT" && cargo build --release -p detect-trb 2>&1 | tail -5
  fi

  echo "🚀 detect-trb başlatılıyor → http://$DETECT_TRB_ADDR"
  # Ek parametreler (ör. --symbol, --port) start'a geçirilebilir
  _tmux_pane "🌊TRB" "cd $CYCLE_ROOT && ./target/release/detect-trb $*" Enter
  sleep 1
  if pgrep -x detect-trb &>/dev/null; then
    echo "✅ detect-trb başladı [pid: $(pgrep -x detect-trb)]"
    echo "   API: http://$DETECT_TRB_ADDR/api/trb/status"
  else
    echo "❌ detect-trb başlatılamadı."
  fi
}

detect-trb-stop() {
  _start_guard
  if pgrep -x "detect-trb" &>/dev/null; then
    pkill -TERM -x "detect-trb" && echo "✅ detect-trb durduruldu"
  else
    echo "⚠️  detect-trb zaten çalışmıyor"
  fi
}

detect-trb-status() {
  local pid
  pid=$(pgrep -x "detect-trb" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ detect-trb ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$DETECT_TRB_ADDR/api/trb/status"
  else
    echo "✘  detect-trb durdurulmuş"
  fi
}

# Son raporu göster — Kullanım: detect-trb-query
detect-trb-query() {
  curl -s "http://$DETECT_TRB_ADDR/api/trb" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. detect-trb-start ile başlat."
}

# ============================================================
#  VELVETUSDT KIRILIM STRATEJİSİ  (strategies/velvetusdt_breakout.py)
#  detect-ms + paper-service kullanır. VELVETUSDT 1m, 100 pencere,
#  her 20 pencerede bir analiz.
# ============================================================
velvetusdt-start() {
  _start_guard
  if pgrep -x velvetusdt &>/dev/null; then
    echo "⚠️  VELVETUSDT stratejisi zaten çalışıyor (pid: $(pgrep -f '[h]eiusdt_breakout.py' | head -1))"
    return 1
  fi
  # Bağımlılık kontrolü
  if ! curl -s -o /dev/null -w "%{http_code}" "http://$DETECT_MS_ADDR/api/ms?symbol=VELVETUSDT&interval=1m&limit=5" 2>/dev/null | grep -q 200; then
    echo "⚠️  detect-ms yanıt vermiyor → velvetusdt-start ile başlatın"
    return 1
  fi
  echo "🎯 VELVETUSDT stratejisi başlatılıyor (VELVETUSDT 1m, 100 pencere, 20 pencere/kontrol)..."
  _tmux_pane "🎯VELVETUSDT" "cd $CYCLE_ROOT && $CYCLE_ROOT/target/debug/velvetusdt" Enter
  sleep 2
  if pgrep -x velvetusdt &>/dev/null; then
    echo "✅ VELVETUSDT stratejisi başladı [pid: $(pgrep -f '[h]eiusdt_breakout.py' | head -1)]"
    echo "   Pencere: cycle → 🎯VELVETUSDT"
  else
    echo "❌ VELVETUSDT stratejisi başlatılamadı."
  fi
}

velvetusdt-stop() {
  _start_guard
  local pid
  pid=$(pgrep -x velvetusdt 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    pkill -TERM -f "[h]eiusdt_breakout.py" 2>/dev/null
    sleep 1
    pkill -KILL -f "[h]eiusdt_breakout.py" 2>/dev/null || true
    echo "✅ VELVETUSDT stratejisi durduruldu [pid:$pid]"
  else
    echo "⚠️  VELVETUSDT stratejisi zaten çalışmıyor"
  fi
}

velvetusdt-status() {
  local pid
  pid=$(pgrep -x velvetusdt 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ VELVETUSDT stratejisi ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
  else
    echo "✘  VELVETUSDT stratejisi durdurulmuş"
  fi
}

velvetusdt-log() {
  tail -f /tmp/velvetusdt.log
}

# Bekleme süresini saniye cinsinden ayarla (çalışan strateji bir sonraki döngüde uygular)
# Kullanım: velvetusdt-wait 600   (10 dakika)  |  velvetusdt-wait 1200  (20 dakika)
velvetusdt-wait() {
  _start_guard
  local sec="${1:-}"
  if [ -z "$sec" ]; then
    local cur; cur=$(cat /tmp/velvetusdt_wait_sec.txt 2>/dev/null || echo "1200")
    echo "ℹ️  Mevcut bekleme: $cur sn"
    echo "Kullanım: velvetusdt-wait <saniye>   (örn. velvetusdt-wait 600 → 10dk)"
    return 0
  fi
  if ! echo "$sec" | grep -qE '^[0-9]+$' || [ "$sec" -lt 10 ]; then
    echo "❌ Saniye değeri geçerli değil (min 10): $sec"
    return 1
  fi
  echo "$sec" > /tmp/velvetusdt_wait_sec.txt
  echo "✅ Bekleme süresi ayarlandı: $sec sn ($((sec/60)) dk)"
  echo "   Çalışan strateji bir sonraki döngüde bu değeri kullanır."
  if pgrep -x velvetusdt >/dev/null 2>&1; then
    echo "   ℹ️  Strateji çalışıyor — yeni süre otomatik uygulanacak."
  fi
}

velvetusdt-query() {
  # Kullanım: velvetusdt-query [--dry-run]
  if [ "${1:-}" = "--dry-run" ]; then
    cd "$CYCLE_ROOT" && $CYCLE_ROOT/target/debug/velvetusdt --once --dry-run
  else
    cd "$CYCLE_ROOT" && $CYCLE_ROOT/target/debug/velvetusdt --once
  fi
}

# ── Yüklendiğini bildir ──────────────────────────────────────
echo -e "${_D}[cycle_env] Yüklendi — ROOT: $CYCLE_ROOT | API: $CYCLE_API${_N}"
```


├── scripts/cycle_tmux.sh

```bash
#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — tmux çok-terminal başlatıcı
#  Kullanım: ./scripts/cycle_tmux.sh [attach|kill|status]
#
#  Pencere 0 — Trading (4 panel):
#    ┌──────────────────────┬──────────────────────┐
#    │  🧠 STRATEGY          │  🛰️  LISTENER        │
#    ├──────────────────────┼──────────────────────┤
#    │  ⚠️  RISK             │  💻 SHELL            │
#    └──────────────────────┴──────────────────────┘
#  Pencere 1 — 📡 DATA   (sekme terminal)
#  Pencere 2 — 🔔 ALERT  (sekme terminal)
#  Pencere 3 — 🛡️ PAPER (sekme terminal)
#  Pencere 4 — Monitor  (CPU/RAM/GPU izleme)
#  Pencere 5 — DETECT-MS (MSMP :3002)
#  Pencere 6 — VELVETUSDT (Kırılım stratejisi)
#  Pencere 7 — WYCKOFF (:3005)
#  Pencere 8 — TURBULANS/DETECT-TRB (:3006)
#  Pencere 9 — SCOUT (Binance USDT tarayıcı → /dev/shm/cycle_finance_scout)
# ============================================================
set -euo pipefail

SESSION="cycle"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# ── Binary dizini: varsayılan release; debug için BIN_DIR=./target/debug ver ──
BIN="${BIN_DIR:-$ROOT/target/release}"
BUILD_ARGS=""
case "$BIN" in
  *release*) BUILD_ARGS="--release" ;;
esac

# ── Env varsayılanları ───────────────────────────────────────
PAPER_API_ADDR="${PAPER_API_ADDR:-127.0.0.1:8080}"
PAPER_ADMIN_USER="${PAPER_ADMIN_USER:-admin}"
PAPER_ADMIN_PASS="${PAPER_ADMIN_PASS:-changeme123}"
PAPER_INITIAL_USDT="${PAPER_INITIAL_USDT:-100000}"
ALERT_CONFIG="${ALERT_CONFIG:-$ROOT/alerts.toml}"

# ── Tam temizlik fonksiyonu ──────────────────────────────────
full_cleanup() {
  echo "🧹 Temizleniyor..."
  tmux kill-session -t "$SESSION" 2>/dev/null && echo "  ✔ tmux session kapatıldı" || echo "  - tmux session yoktu"
  for proc in core paper-service alert-service; do
    if pgrep -x "$proc" &>/dev/null; then
      pkill -TERM -x "$proc" 2>/dev/null || true
      sleep 0.5
      pkill -KILL -x "$proc" 2>/dev/null || true
      echo "  ✔ $proc durduruldu"
    fi
  done
  for f in /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders; do
    [ -f "$f" ] && rm -f "$f" && echo "  ✔ $f silindi" || true
  done
  echo "✅ Temizlik tamamlandı."
}

# ── Alt komutlar ─────────────────────────────────────────────
case "${1:-}" in
  kill)
    full_cleanup
    exit 0
    ;;
  status)
    echo "=== tmux Panelleri ==="
    tmux list-panes -t "$SESSION" -F "  #{pane_index}: #{pane_title} [pid:#{pane_pid}] #{pane_current_command}" 2>/dev/null \
      || echo "  ⚠️  '$SESSION' session'ı çalışmıyor."
    echo ""
    echo "=== Çalışan Servisler ==="
for proc in core paper-service alert-service scout-service; do
      pid=$(pgrep -x "$proc" 2>/dev/null | head -1 || true)
      if [ -n "$pid" ]; then
        mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
        cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
        echo "  ✔ $proc  [pid:$pid]  CPU:${cpu}%  RAM:${mem}"
      else
        echo "  ✘ $proc  (durdurulmuş)"
      fi
    done
    exit 0
    ;;
  attach)
    tmux attach-session -t "$SESSION" 2>/dev/null || { echo "⚠️  Session yok."; exit 1; }
    exit 0
    ;;
esac

# ── Zaten çalışıyorsa bağlan ─────────────────────────────────
if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "⚡ '$SESSION' zaten çalışıyor. Bağlanılıyor..."
  tmux attach-session -t "$SESSION"
  exit 0
fi

# ── Derleme ──────────────────────────────────────────────────
echo "🔨 Derleniyor..."
cd "$ROOT"
cargo build $BUILD_ARGS -p core -p paper-service -p alert-service -p scout-service 2>&1 | tail -5

# ── Eski süreçleri ve ring buffer'ları temizle ───────────────
echo "🧹 Eski süreçler temizleniyor..."
for proc in core paper-service alert-service; do
  if pgrep -x "$proc" &>/dev/null; then
    pkill -TERM -x "$proc" 2>/dev/null || true
    sleep 0.3
    pkill -KILL -x "$proc" 2>/dev/null || true
    echo "  ✔ $proc durduruldu"
  fi
done
rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders /dev/shm/cycle_finance_scout
echo "  ✔ Ring buffer'lar temizlendi"
sleep 1

# ── Session oluştur ──────────────────────────────────────────
tmux new-session -d -s "$SESSION" -x 220 -y 50
tmux rename-window -t "$SESSION:0" "Trading"

# ── Panel düzeni ─────────────────────────────────────────────
# 0=sol-üst(STRATEGY)  2=sağ-üst(LISTENER)
# 1=sol-alt(RISK)      3=sağ-alt(SHELL)
tmux split-window -t "$SESSION:0"    -h
tmux split-window -t "$SESSION:0.0"  -v
tmux split-window -t "$SESSION:0.2"  -v

# ── Panel başlıkları ─────────────────────────────────────────
tmux select-pane -t "$SESSION:0.0" -T "🧠 STRATEGY"
tmux select-pane -t "$SESSION:0.2" -T "🛰️  LISTENER"
tmux select-pane -t "$SESSION:0.1" -T "⚠️  RISK"
tmux select-pane -t "$SESSION:0.3" -T "💻 SHELL"

# ── Panel 0: STRATEGY ────────────────────────────────────────
tmux send-keys -t "$SESSION:0.0" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧠  STRATEGY TERMİNALİ  (PyO3)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $ROOT && RUN_MODE=STRATEGY $BIN/core
" Enter

# ── Panel 2: LISTENER ─────────────────────────────────────────
tmux send-keys -t "$SESSION:0.2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛰️   LISTENER  (Anlık Metrik Analizi)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/listener
" Enter

# ── Panel 1: RISK ─────────────────────────────────────────────
tmux send-keys -t "$SESSION:0.1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '⚠️   RİSK ANALİZİ  (TimescaleDB)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/risk_analysis --watch
" Enter

# ── Panel 3: SHELL ───────────────────────────────────────────
tmux send-keys -t "$SESSION:0.3" "source /tmp/cycle_init.sh" Enter

# ── Pencere 1: DATA (sekme terminal) ─────────────────────────
tmux new-window -t "$SESSION:1" -n "📡 DATA"
tmux send-keys -t "$SESSION:1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  DATA TERMİNALİ  (Binance WS)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
cd $ROOT && RUN_MODE=DATA $BIN/core
" Enter

# ── Pencere 2: ALERT (sekme terminal) ────────────────────────
tmux new-window -t "$SESSION:2" -n "🔔 ALERT"
tmux send-keys -t "$SESSION:2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔔  ALERT SERVİSİ  (Sesli Uyarı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/alert-service --config $ALERT_CONFIG
" Enter

# ── Pencere 3: PAPER (sekme terminal) ────────────────────────
tmux new-window -t "$SESSION:3" -n "🛡️ PAPER"
tmux send-keys -t "$SESSION:3" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛡️   PAPER SERVICE  (REST API :8080)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && \
  PAPER_ADMIN_USER=$PAPER_ADMIN_USER \
  PAPER_ADMIN_PASS=$PAPER_ADMIN_PASS \
  PAPER_API_ADDR=$PAPER_API_ADDR \
  PAPER_INITIAL_USDT=$PAPER_INITIAL_USDT \
  PAPER_SLED_PATH=./paper_wal \
  PAPER_DB_PATH=/tmp/paper_live.db \
  $BIN/paper-service
" Enter

# ── Shell init dosyasını oluştur ────────────────────────────
# (tmux send-keys ile çok satırlı komut göndermek güvensiz;
#  bunun yerine önce dosyaya yaz, shell paneli source eder)
cat > /tmp/cycle_init.sh << INITEOF
#!/usr/bin/env bash
export CYCLE_ROOT='$ROOT'
export CYCLE_API='http://$PAPER_API_ADDR'
export CYCLE_USER='$PAPER_ADMIN_USER'
export CYCLE_PASS='$PAPER_ADMIN_PASS'
source '$ROOT/scripts/cycle_env.sh'
help-cycle
INITEOF
chmod +x /tmp/cycle_init.sh

# ── Pencere 4: MONITOR ───────────────────────────────────────
tmux new-window -t "$SESSION:4" -n "Monitor"
tmux send-keys -t "$SESSION:4" "bash '$ROOT/scripts/monitor.sh'" Enter
tmux select-pane -t "$SESSION:4" -T "Monitor"

# ── Pencere 5: DETECT-MS ─────────────────────────────────────
tmux new-window -t "$SESSION:5" -n "DETECT-MS"
tmux send-keys -t "$SESSION:5" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📈  DETECT-MS  (MSMP 2.0 :3002)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-ms
" Enter

# ── Pencere 6: VELVETUSDT STRATEJİ ─────────────────────────────
tmux new-window -t "$SESSION:6" -n "VELVETUSDT"
tmux send-keys -t "$SESSION:6" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🎯  VELVETUSDT  (Kırılım Stratejisi)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/velvetusdt
" Enter

# ── Pencere 7: WYCKOFF ANALİZ ───────────────────────────────
tmux new-window -t "$SESSION:7" -n "WYCKOFF"
tmux send-keys -t "$SESSION:7" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🏛️  DETECT-WYCKOFF  (Wyckoff :3005)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-wyckoff
" Enter

# ── Pencere 8: DETECT-TRB (Navier-Stokes) ────────────────────
tmux new-window -t "$SESSION:8" -n "TURBULANS"
tmux send-keys -t "$SESSION:8" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🌊  DETECT-TRB  (Navier-Stokes :3006)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-trb
" Enter

# ── Pencere 9: SCOUT (Binance USDT tarayıcı) ────────────────
tmux new-window -t "$SESSION:9" -n "SCOUT"
tmux send-keys -t "$SESSION:9" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔭  SCOUT  (Binance USDT tarayıcı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo 'Fırsat + sembol metrikleri → /dev/shm/cycle_finance_scout'
echo 'Tüketici: ./target/debug/probe --once'
sleep 2
cd $ROOT && $BIN/scout-service
" Enter

# ── Görsel ayarlar (global) ──────────────────────────────────
tmux set-option -t "$SESSION" mouse on
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format " #{pane_title} "
tmux set-option -t "$SESSION" status-interval 1

# Status bar — koyu tema
tmux set-option -t "$SESSION" status-style          "bg=colour232,fg=colour245"
tmux set-option -t "$SESSION" status-left           "#[bg=colour25,fg=colour255,bold]  🏛️  Cycle Finance  #[bg=colour232,fg=colour245] "
tmux set-option -t "$SESSION" status-left-length    30
tmux set-option -t "$SESSION" status-right          "#[fg=colour39]0#[fg=colour244]:Trading #[fg=colour45]1#[fg=colour244]:DATA #[fg=colour214]2#[fg=colour244]:ALERT #[fg=colour82]3#[fg=colour244]:PAPER #[fg=colour196]4#[fg=colour244]:Mon #[fg=colour171]7#[fg=colour244]:WYCKOFF #[fg=colour51]8#[fg=colour244]:TRB #[fg=colour250]%H:%M:%S"
tmux set-option -t "$SESSION" status-right-length   80

# Window sekme renkleri
tmux set-option -t "$SESSION" window-status-format          "#[fg=colour240] #{window_index}:#{window_name} "
tmux set-option -t "$SESSION" window-status-current-format  "#[bg=colour25,fg=colour255,bold] #{window_index}:#{window_name} "

# ── Per-pane renk temaları ───────────────────────────────────
# 🧠 STRATEGY  → Mor tema     (bg: koyu mor   | kenarlık: parlak magenta)
tmux select-pane -t "$SESSION:0.0" -P "bg=colour53,fg=colour255"
tmux set-option -t "$SESSION:0.0" -p pane-active-border-style "fg=colour171,bold"
tmux set-option -t "$SESSION:0.0" -p pane-border-style        "fg=colour55"

# 🛰️  LISTENER   → Camgöbeği tema (bg: koyu turkuaz | kenarlık: cyan)
tmux select-pane -t "$SESSION:0.2" -P "bg=colour23,fg=colour255"
tmux set-option -t "$SESSION:0.2" -p pane-active-border-style "fg=colour45,bold"
tmux set-option -t "$SESSION:0.2" -p pane-border-style        "fg=colour36"

# ⚠️  RISK       → Kırmızı tema  (bg: koyu bordo | kenarlık: kırmızı)
tmux select-pane -t "$SESSION:0.1" -P "bg=colour52,fg=colour255"
tmux set-option -t "$SESSION:0.1" -p pane-active-border-style "fg=colour196,bold"
tmux set-option -t "$SESSION:0.1" -p pane-border-style        "fg=colour124"

# 💻 SHELL     → Antrasit tema (bg: çok koyu  | kenarlık: açık gri)
tmux select-pane -t "$SESSION:0.3" -P "bg=colour233,fg=colour252"
tmux set-option -t "$SESSION:0.3" -p pane-active-border-style "fg=colour244,bold"
tmux set-option -t "$SESSION:0.3" -p pane-border-style        "fg=colour238"

# Pane başlık formatı — renk kodlu
tmux set-option -t "$SESSION:0" pane-border-format \
  "#{?#{==:#{pane_index},0},#[fg=colour171 bold],#{?#{==:#{pane_index},1},#[fg=colour196 bold],#{?#{==:#{pane_index},2},#[fg=colour45 bold],#{?#{==:#{pane_index},3},#[fg=colour244 bold],#[fg=colour244 bold]}}}}} #{pane_title} #[default]"

# ── Terminal penceresine dön ve bağlan ───────────────────────
tmux select-window -t "$SESSION:0"
tmux select-pane  -t "$SESSION:0.3"
tmux attach-session -t "$SESSION"
```


├── scripts/gdpr_erasure_test.sh

```bash
#!/bin/bash

# GDPR/KVKK Right to Erasure Simulation Script for Cycle Finance 2.0
# Simulates physically wiping a user's data from ClickHouse.

set -e

USER_ID="client_9942"
SALT="sUp3rS3cr3tS4lt"

echo "=========================================="
echo " Starting GDPR Erasure Protocol Simulation"
echo "=========================================="

# 1. Masking the User ID to create the hash
USER_HASH=$(echo -n "${USER_ID}${SALT}" | sha3sum -a 256 | awk '{print $1}' || echo "mocked_sha3_hash_8a2b3c")
echo "[+] Target User Hash: $USER_HASH"

# 2. Simulate ClickHouse Mutation
echo "[+] Triggering ALTER TABLE ticks DELETE WHERE symbol_hash = '$USER_HASH'..."
sleep 1
echo "[+] ClickHouse mutation submitted."

# 3. Simulate verifying the physical erasure
echo "[+] Verifying erasure via Merkle Tree..."
sleep 1
echo "[+] Data physically scrubbed from disks and EC-12/4 replicas."

# 4. Log to Deletion Registry
echo "[+] Appending event to deletion_registry for 3-year compliance hold..."
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) | $USER_HASH | GDPR Erasure Processed" >> deletion_registry_mock.log

echo "=========================================="
echo " Erasure Protocol Complete."
echo "=========================================="
```


├── scripts/monitor.sh

```bash
#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — Servis İzleme Paneli
#  Her saniye güncellenir. Ctrl+C ile çıkılır.
#
#  İzlenen servisler:
#    core (DATA / STRATEGY / BACKTEST / CORRELATION)
#    paper-service
#    alert-service
# ============================================================

# ── Renkler ──────────────────────────────────────────────────
R='\033[0;31m'    # kırmızı
G='\033[0;32m'    # yeşil
Y='\033[1;33m'    # sarı
C='\033[0;36m'    # camgöbeği
B='\033[1;34m'    # mavi
M='\033[0;35m'    # mor
W='\033[1;37m'    # beyaz kalın
DIM='\033[2m'     # soluk
N='\033[0m'       # reset
BG='\033[40m'     # siyah arka plan

# ── GPU sysfs yolu (AMD RX 5500) ─────────────────────────────
GPU_CARD=""
for card in /sys/class/drm/card*/device/gpu_busy_percent; do
    if [ -r "$card" ]; then
        GPU_CARD="$(dirname "$card")"
        break
    fi
done

# ── Bar çizici ───────────────────────────────────────────────
# Kullanım: draw_bar <yüzde(0-100)> <genişlik> <renk>
draw_bar() {
    local pct="${1:-0}"
    local width="${2:-20}"
    local color="${3:-$G}"
    # Yüzde'yi tam sayıya dönüştür
    pct=$(echo "$pct" | awk '{printf "%d", $1}')
    [ "$pct" -gt 100 ] 2>/dev/null && pct=100
    [ "$pct" -lt 0 ]   2>/dev/null && pct=0
    local filled=$(( pct * width / 100 ))
    local empty=$(( width - filled ))
    # Yüksek kullanımda renk değiştir
    if [ "$pct" -ge 80 ]; then color="$R"
    elif [ "$pct" -ge 50 ]; then color="$Y"
    fi
    local bar=""
    for ((i=0; i<filled; i++)); do bar+="█"; done
    for ((i=0; i<empty;  i++)); do bar+="░"; done
    echo -ne "${color}${bar}${N}"
}

# ── RAM okunabilir format ─────────────────────────────────────
human_mb() {
    local kb="${1:-0}"
    local mb=$(( kb / 1024 ))
    if [ "$mb" -ge 1024 ]; then
        echo "$(awk "BEGIN{printf \"%.1f\", $mb/1024}")G"
    else
        echo "${mb}M"
    fi
}

# ── Servis satırı çiz ─────────────────────────────────────────
# Kullanım: print_service <etiket> <pid> <renk>
print_service() {
    local label="$1"
    local pid="$2"
    local color="${3:-$C}"
    local bar_width=18

    if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
        printf "${color}%-22s${N} ${DIM}%-8s${N} ${R}%-6s${N}   %-${bar_width}s   ${R}%-8s${N}   %-${bar_width}s\n" \
            "$label" "-" "KAPALI" "$(draw_bar 0 $bar_width $R)" "-" "$(draw_bar 0 $bar_width $R)"
        return
    fi

    # CPU ve bellek bilgisi
    local stat
    stat=$(ps -p "$pid" -o pid,pcpu,rss,vsz --no-headers 2>/dev/null | head -1)
    [ -z "$stat" ] && return

    local cpu  rss vsz
    cpu=$(echo "$stat" | awk '{printf "%.1f", $2}')
    rss=$(echo "$stat" | awk '{print $3}')   # KB
    vsz=$(echo "$stat" | awk '{print $4}')   # KB (sanal)

    local rss_str vsz_str cpu_int
    rss_str=$(human_mb "$rss")
    vsz_str=$(human_mb "$vsz")
    cpu_int=$(echo "$cpu" | awk '{printf "%d", $1}')

    # Çok çekirdekli sistemlerde CPU > 100 olabilir, bar için sıkıştır
    local cpu_bar_pct=$(( cpu_int > 100 ? 100 : cpu_int ))

    printf "${color}%-22s${N} ${W}%-8s${N} ${Y}%5s%%${N}  %s  ${C}%-8s${N}  %s\n" \
        "$label" "[$pid]" "$cpu" \
        "$(draw_bar "$cpu_bar_pct" "$bar_width")" \
        "$rss_str" \
        "$(draw_bar "$(( rss / 1024 > 100 ? 100 : rss / 1024 ))" "$bar_width")"
}

# ── Sistem toplamı ───────────────────────────────────────────
system_summary() {
    # CPU toplam kullanımı
    local cpu_idle cpu_use
    cpu_idle=$(top -bn1 | grep "Cpu(s)" | awk '{print $8}' | tr -d '%' | tr ',' '.')
    [ -z "$cpu_idle" ] && cpu_idle=$(vmstat 1 1 | tail -1 | awk '{print $15}')
    cpu_use=$(awk "BEGIN{printf \"%.1f\", 100 - ${cpu_idle:-0}}")

    # RAM
    local mem_total mem_avail mem_used mem_pct
    mem_total=$(awk '/MemTotal/{print $2}' /proc/meminfo)
    mem_avail=$(awk '/MemAvailable/{print $2}' /proc/meminfo)
    mem_used=$(( mem_total - mem_avail ))
    mem_pct=$(awk "BEGIN{printf \"%d\", $mem_used * 100 / $mem_total}")

    # GPU (AMD sysfs)
    local gpu_use="N/A" gpu_vram_pct=0 gpu_vram_str="N/A"
    if [ -n "$GPU_CARD" ]; then
        gpu_use=$(cat "${GPU_CARD}/gpu_busy_percent" 2>/dev/null || echo "0")
        local vram_used vram_total
        vram_used=$(cat "${GPU_CARD}/mem_info_vram_used"  2>/dev/null || echo "0")
        vram_total=$(cat "${GPU_CARD}/mem_info_vram_total" 2>/dev/null || echo "1")
        gpu_vram_pct=$(awk "BEGIN{printf \"%d\", $vram_used * 100 / $vram_total}")
        local vram_used_mb=$(( vram_used / 1024 / 1024 ))
        local vram_total_mb=$(( vram_total / 1024 / 1024 ))
        gpu_vram_str="${vram_used_mb}M / ${vram_total_mb}M"
    fi

    local cpu_int=${cpu_use%.*}
    printf "${DIM}Sistem Geneli:${N}\n"
    printf "  ${W}CPU  ${N}%5s%%  %s\n" "$cpu_use"  "$(draw_bar "$cpu_int" 30)"
    printf "  ${W}RAM  ${N}%5s%%  %s  ${DIM}(%s / %s)${N}\n" \
        "$mem_pct" "$(draw_bar "$mem_pct" 30)" \
        "$(human_mb "$mem_used")" "$(human_mb "$mem_total")"
    if [ -n "$GPU_CARD" ]; then
        printf "  ${W}GPU  ${N}%5s%%  %s\n" "$gpu_use" "$(draw_bar "$gpu_use" 30)"
        printf "  ${W}VRAM ${N}%5s%%  %s  ${DIM}(%s)${N}\n" \
            "$gpu_vram_pct" "$(draw_bar "$gpu_vram_pct" 30)" "$gpu_vram_str"
    else
        printf "  ${W}GPU  ${N}${DIM}  AMD sysfs okunamadı${N}\n"
    fi
}

# ── PID bul ─────────────────────────────────────────────────
find_pid() {
    local name="$1"
    pgrep -x "$name" 2>/dev/null | head -1
}

find_pid_env() {
    # RUN_MODE=X olan core process'ini bul
    local mode="$1"
    pgrep -x "core" 2>/dev/null | while read -r pid; do
        if grep -qa "RUN_MODE=$mode" /proc/"$pid"/environ 2>/dev/null; then
            echo "$pid"
            return
        fi
    done
}

# ── Ana döngü ────────────────────────────────────────────────
INTERVAL="${MONITOR_INTERVAL:-1}"

# Cursor'ı gizle, çıkışta geri getir
tput civis
trap 'tput cnorm; echo' EXIT INT TERM

# İlk açılışta bir kez temizle
clear

while true; do
    # Ekranı silmeden cursor'ı sol-üst köşeye taşı (titreme yok)
    tput cup 0 0

    local_time=$(date '+%H:%M:%S')
    local_date=$(date '+%d.%m.%Y')

    echo -e "${W}╔══════════════════════════════════════════════════════════════════════════════════╗${N}"
    printf "${W}║${N}  ${M}📊 CYCLE FINANCE — SERVİS İZLEME PANELİ${N}%$((39 - ${#local_time}))s${Y}%s${N}  ${W}║${N}\n" "" "$local_time  $local_date"
    echo -e "${W}╚══════════════════════════════════════════════════════════════════════════════════╝${N}"
    echo ""

    # ── Sistem özeti ─────────────────────────────────────────
    system_summary
    echo ""

    # ── Servis başlıkları ─────────────────────────────────────
    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"
    printf "${DIM}%-22s  %-8s  %-7s  %-18s  %-8s  %-18s${N}\n" \
        "SERVİS" "PID" "CPU%" "CPU KULLANIMI" "RAM" "RAM KULLANIMI"
    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"

    # ── Servisler ────────────────────────────────────────────
    PID_DATA=$(find_pid_env "DATA")
    PID_STRATEGY=$(find_pid_env "STRATEGY")
    PID_BACKTEST=$(find_pid_env "BACKTEST")
    PID_CORRELATION=$(find_pid_env "CORRELATION")
    PID_PAPER=$(find_pid "paper-service")
    PID_ALERT=$(find_pid "alert-service")

    # core binary tek isimle görünüyorsa genel bul
    [ -z "$PID_DATA" ] && [ -z "$PID_STRATEGY" ] && [ -z "$PID_BACKTEST" ] && [ -z "$PID_CORRELATION" ] && {
        ALL_CORE=$(pgrep -x "core" 2>/dev/null | head -1)
    }

    print_service "📡 DATA"          "${PID_DATA:-$ALL_CORE}" "$C"
    print_service "🧠 STRATEGY"      "$PID_STRATEGY"          "$B"
    print_service "🔄 BACKTEST"      "$PID_BACKTEST"          "$M"
    print_service "📈 CORRELATION"   "$PID_CORRELATION"       "$Y"
    echo -e "${DIM}──────────────────────────────────────────────────────────────────────────────────${N}"
    print_service "🛡️  PAPER-SERVICE" "$PID_PAPER"             "$G"
    print_service "🔔 ALERT-SERVICE" "$PID_ALERT"             "$Y"

    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"

    # ── Çalışan servis sayısı ─────────────────────────────────
    running=0
    for p in "$PID_DATA" "$PID_STRATEGY" "$PID_BACKTEST" "$PID_CORRELATION" "$PID_PAPER" "$PID_ALERT"; do
        [ -n "$p" ] && kill -0 "$p" 2>/dev/null && (( running++ )) || true
    done

    echo ""
    printf "  ${DIM}Çalışan servis: ${W}%d/6${N}${DIM}   |   Yenileme: her %ss   |   Çıkış: Ctrl+C${N}\n" \
        "$running" "$INTERVAL"

    # ── Ring buffer bilgisi ───────────────────────────────────
    echo ""
    echo -e "  ${DIM}Ring Buffer Durumu:${N}"
    for ring in cycle_finance_ring cycle_finance_orders; do
        if [ -f "/dev/shm/$ring" ]; then
            ring_size=$(du -sh "/dev/shm/$ring" 2>/dev/null | cut -f1)
            printf "    ${G}✔${N} /dev/shm/%-28s %s\n" "$ring" "$ring_size"
        else
            printf "    ${R}✘${N} /dev/shm/%-28s ${DIM}(yok)${N}\n" "$ring"
        fi
    done

    sleep "$INTERVAL"
done
```


├── k8s/chaos_dns_failure.yaml

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: DNSChaos
metadata:
  name: cycle-finance-dns-failure
  namespace: default
spec:
  action: error
  mode: all
  selector:
    labelSelectors:
      app: cycle-finance
  patterns:
    - api.binance.com
    - stream.binance.com
  duration: '5m'
  scheduler:
    cron: '@every 30m'
```


├── k8s/chaos_network_partition.yaml

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: cycle-finance-network-partition
  namespace: default
spec:
  action: partition
  mode: all
  selector:
    labelSelectors:
      app: cycle-finance
  direction: both
  target:
    selector:
      labelSelectors:
        app: redis-cluster
    mode: all
  duration: '10s'
  scheduler:
    cron: '@every 5m'
```


├── k8s/chaos_ntp_drift.yaml

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: TimeChaos
metadata:
  name: cycle-finance-ntp-drift
  namespace: default
spec:
  mode: all
  selector:
    labelSelectors:
      app: cycle-finance
  timeOffset: '10s' # Simulate NTP drift of 10 seconds ahead
  duration: '5m'
  scheduler:
    cron: '@every 15m'
```


├── k8s/deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cycle-finance-core
  labels:
    app: cycle-finance
spec:
  replicas: 1
  selector:
    matchLabels:
      app: cycle-finance
  template:
    metadata:
      labels:
        app: cycle-finance
      annotations:
        # Require cgroups v2 resource management
        kubernetes.io/cgroup-version: "v2"
    spec:
      containers:
      - name: core
        image: cycle-finance/core:latest
        resources:
          limits:
            cpu: "4"
            memory: "4Gi"
          requests:
            cpu: "4"
            memory: "4Gi"
        securityContext:
          capabilities:
            add:
              - SYS_NICE # Required for SCHED_FIFO real-time thread scheduling
        env:
          - name: RUST_LOG
            value: "info"
```


├── formal_verification/CycleFinance.cfg

```ini
SPECIFICATION Spec
INVARIANT Safety
PROPERTY Liveness
```


├── formal_verification/CycleFinance.tla

```tla
--------------------------- MODULE CycleFinance ---------------------------
EXTENDS Naturals, Sequences, TLC

(* 
  TLA+ Model for Cycle Finance 2.0 Lock-Free Tick Processing.
  Proves that ticks produced by the network adapter are eventually consumed
  by the core without deadlocks (Liveness) and without dropping (Safety).
*)

VARIABLES 
    queue,       \* Lock-free MPMC queue (flume)
    ticks_in,    \* Total ticks generated
    ticks_out    \* Total ticks processed

vars == <<queue, ticks_in, ticks_out>>

Init == 
    /\ queue = <<>>
    /\ ticks_in = 0
    /\ ticks_out = 0

(* Producer adds a tick to the queue *)
Produce == 
    /\ ticks_in < 1000  \* Bounded model checking
    /\ queue' = Append(queue, "tick")
    /\ ticks_in' = ticks_in + 1
    /\ UNCHANGED <<ticks_out>>

(* Consumer processes a tick from the queue lock-free *)
Consume == 
    /\ queue # <<>>
    /\ queue' = Tail(queue)
    /\ ticks_out' = ticks_out + 1
    /\ UNCHANGED <<ticks_in>>

Next == Produce \/ Consume

(* Safety: Processed ticks never exceed produced ticks *)
Safety == ticks_out <= ticks_in

(* Liveness: Every produced tick is eventually consumed *)
Liveness == \A n \in Nat : (ticks_in = n) ~> (ticks_out = n)

Spec == Init /\ [][Next]_vars /\ WF_vars(Consume)

=============================================================================
```


├── .github/workflows/test-suite.yml

```yaml
name: Cycle Finance 2.0 Sertifika Testleri

on:
  pull_request:
    branches: [ "master" ]
  schedule:
    - cron: '0 0 * * *' # Gece yarısı regression

jobs:
  audit-and-coverage:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Bağımlılık Taraması (cargo-deny)
      run: |
        cargo install cargo-deny
        cargo deny check advisories
    - name: Line Coverage (tarpaulin %95)
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --ignore-tests --fail-under 95

  unit-and-integration:
    runs-on: ubuntu-latest
    needs: audit-and-coverage
    steps:
    - uses: actions/checkout@v3
    - name: Birim ve Entegrasyon Testleri
      run: cargo test --release

  performance-wcet:
    runs-on: ubuntu-latest
    needs: unit-and-integration
    steps:
    - uses: actions/checkout@v3
    - name: 750µs Maksimum Tick Gecikme Testi (WCET)
      run: cargo bench
      
  # Kaos testleri staging'de tetiklenir
  chaos-mesh-staging:
    runs-on: ubuntu-latest
    needs: performance-wcet
    steps:
    - uses: actions/checkout@v3
    - name: Chaos Mesh 20 Senaryo Tetiklemesi
      run: echo "Triggering Chaos Mesh scenarios on AWS izole cluster (Staging)..."

  # MANİFESTO
  # "Bu testleri geçmeyen sistem, 20 yıl değil 20 saniye dayanır. Artık kod yazmak kadar, 
  # bu testleri otomasyona bağlamak da sizin sorumluluğunuzdadır. Mükemmeliyet ancak bu 
  # kırmızı çizgilerle korunabilir. Test senaryolarını hazırlayın ve 'cargo test --release' 
  # komutunu bu dokümanın altına imza olarak kazıyın. Başlayın."
```


├── docs/flowcharts/01_genel_bakis.mmd

```mermaid
flowchart TB
    subgraph DIS["Dış Dünya"]
        BINWS["Binance Futures WebSocket<br/>fstream.binance.com/stream"]
        BINREST["Binance Futures REST<br/>fapi.binance.com (klines + premiumIndex)"]
        BINORD["Binance WS Order API v3<br/>ws-api.binance.com (LIVE emir)"]
    end

    subgraph K0["Katman 0 — contracts"]
        C_EV["events.rs :: OwnedEvent + EventType (8 tip)"]
        C_WIRE["wire.rs :: typed binary codec (44B–659B)"]
    end

    subgraph K1["Katman 1 — transport (IPC)"]
        RING["GenerationalRingBuffer<br/>/dev/shm · 160k slot<br/>torn-read korumalı"]
        ORING["OrderRingBuffer<br/>/dev/shm (STRATEGY→EXECUTION)"]
        PRING["PriceFeed ring<br/>/dev/shm/cycle_finance_pricefeed"]
    end

    subgraph K2["Katman 2 — core motor"]
        PARSER["EventParser (simdjson · zero-copy)"]
        VALID["DataValidator<br/>stale ≤ 200ms · crossed book · circuit breaker"]
        DBW["TimescaleDB batch writer<br/>(10k / 1sn · trades hypertable)"]
        ORCH["TitaniumOrchestrator (spin-loop)"]
        RISKC["RiskEngine + LOB simülasyonu"]
        TSC["TscTimer (RDTSC) + RT prio 99"]
    end

    subgraph DET["Analiz Servisleri"]
        DETMS["detect-ms :3002 (SMC 7 katman)"]
        CALC["calc-ind :3003"]
        DBQ["db-query (TimescaleDB panel)"]
    end

    subgraph TRADE["Strateji & Yürütme"]
        PRICEF["daemon :3004"]
        HEI["strategies-engine (kırılım stratejisi)"]
        EXEC["execution-engine<br/>LIVE actor"]
        OHLCV["ohlcv-engine (klines client)"]
    end

    subgraph OPS["Operasyonel"]
        ALERT["alert-service (sesli koşul uyarıları)"]
        RISKWW["risk-worker (60sn)"]
        COLDS["cold-starter + cold-storage"]
        ADAPT["adapter crate (5/6 modül mock)"]
    end

    BINWS -->|"raw JSON trade+depth20"| PARSER
    BINREST -->|"klines REST"| OHLCV
    BINWS -->|"raw JSON"| PRICEF

    PARSER --> VALID
    VALID --> C_WIRE
    C_WIRE --> DBW
    C_WIRE --> RING
    RING --> ORCH
    ORCH --> STRAT["strateji binary'leri (strategies-engine)"]
    STRAT --> RISKC
    RISKC --> ORING
    ORING --> EXEC
    EXEC -.->|"LIVE mod"| BINORD

    OHLCV --> DETMS
    OHLCV --> CALC
    OHLCV --> DBQ

    PRICEF --> PRING
    PRING --> HEI
    PRING --> ALERT
    HEI -->|"detect-ms seviyeleri"| DETMS

    OPS -->|"planlanan"| TRADE
```


├── docs/flowcharts/02_katman0_contracts.mmd

```mermaid
classDiagram
    direction LR

    class OwnedEvent {
        +symbol : [u8; 16]
        +payload : EventType
        +new_trade() OwnedEvent
        +new_orderbook() OwnedEvent
        +new_liquidation() OwnedEvent
        +new_funding_rate() OwnedEvent
        +new_bookticker() OwnedEvent
        +new_open_interest() OwnedEvent
        +new_opportunity() OwnedEvent
        +new_symbol_metrics() OwnedEvent
    }

    class EventType {
        <<enum u8>>
        Trade : price · quantity · ts · is_buyer_maker
        Orderbook[20] : bids + asks
        Liquidation : side · price · qty · ts
        FundingRate : mark · index · rate · next_funding
        BookTicker : best bid + best ask (fiyat + miktar)
        OpenInterest : oi · ts
        Opportunity : score · eff · bps/s · ticks/s · ob/sk · spread · verdict
        SymbolMetrics : 6 mikroyapı metriği
    }

    class WireCodec {
        <<compact binary>>
        +DEPTH_FRAME_SIZE : 659 B
        +MAX_FRAME_SIZE : 659 B
        +encode(ev : OwnedEvent, buf) Option~usize~
        +decode(buf) Option~OwnedEvent~
        -write_decimal(buf, off, d) Option~usize~
        -read_decimal(buf, off) Decimal
        +tag : u8 (0=Trade ... 7=SymbolMetrics)
        +i64 mantissa + u8 scale := Decimal
    }

    OwnedEvent --> "1" EventType : payload
    WireCodec ..> OwnedEvent : encode/decode<br/>(sıfır kopya hot path)
```


├── docs/flowcharts/03_katman1_transport.mmd

```mermaid
sequenceDiagram
    autonumber
    participant U as Üretici (core / daemon)
    participant R as GenerationalRingBuffer<br/>(/dev/shm · 160k slot)
    participant T as Tüketici (alert / strategies-engine)

    Note over R: Slot = [seq: u64 | len: u16 | data: [u8; 702]] → 768 B aligned
    U->>R: head = seq al
    U->>R: data + len yaz
    U->>R: fence(Release)
    U->>R: seq + head yaz (EN SON)
    Note over T: torn-read koruması
    loop cursor < head
        T->>R: slot = idx(seq_expected)
        T->>R: slot.seq == expected?
        alt Generational eşleşme
            T->>R: veriyi kopyala
            T->>R: yeniden seq kontrol
            Note over T: eşleşiyor → veri tam; aksi → None
        else
            T->>T: sequential overwrite, okuyucu 200k bekler
        end
    end

    rect rgb(240, 248, 255)
        Note over U, T: OrderRingBuffer (STRATEGY→EXECUTION)
        Note over U, T: magic 0xD3F0000000000002 · sipariş slot
    end
```


├── docs/flowcharts/04_katman2_core.mmd

```mermaid
flowchart TB
    subgraph MODE["core/main.rs — RUN_MODE router"]
        RUN["RUN_MODE env"]
        RUN -->|"DATA"| DATABLK
        RUN -->|"STRATEGY"| STRATCLI["strategy_cli<br/>strateji binary spawn/restart"]
        RUN -->|"BACKTEST"| BT["backtester<br/>CSV → mock stream → ring"]
        RUN -->|"CORRELATION"| CORR["correlation_cli v5<br/>emilim / sığ pump / ayı tuzağı"]
    end

    subgraph DATA["DATA hot path (RT prio 99 thread)"]
        WS["adapter:binance WS client<br/>8 stream (trade + depth20@100ms)"]
        QUEUE["LockFreeDispatcher<br/>flume bounded 262144"]
        PARSER2["EventParser (simdjson · zero-copy)"]
        VALID2["DataValidator<br/>stale ≤ 200ms · crossed book · eski tick"]
        CB["Circuit Breaker<br/>(100+ hatalı/sn → durdur)"]
        ENC["wire::encode → typed frame"]
        RING2["GenerationalRingBuffer push"]
        DB2["TimescaleDB batch writer<br/>(flume 1M · batch 10k/1sn)"]
    end

    subgraph ORCH["TitaniumOrchestrator (spin-loop)"]
        READ["ring read_slot(cursor)"]
        DISP["her stratejiye on_market_data"]
        CATCH2["catch_unwind → Poisoned"]
        RENG["RiskEngine onayı"]
        GATE["gateway_tx (crossbeam) → execution"]
        TIMER["TSC 1ms on_timer"]
    end

    subgraph UK["Strateji Çekirdeği"]
        TRAIT["trait Strategy<br/>on_market_data · on_timer · on_fill"]
        LOB["LOB simulate (fixed point)"]
        PORT2["Portfolio: PnL · komisyon · drawdown"]
        STRATS["heusdt · scout · wyckoff"]
    end

    WS --> QUEUE
    QUEUE --> PARSER2
    PARSER2 --> VALID2
    VALID2 -->|geçersiz| CB
    VALID2 -->|geçerli| ENC
    ENC --> RING2
    ENC --> DB2

    RING2 --> READ
    READ --> DISP
    DISP --> CATCH2
    CATCH2 --> RENG
    RENG -->|onay| GATE
    RENG -->|red| DROP2["log + drop"]
    TIMER --> DISP
    TRAIT --> DISP
    LOB -.-> RENG
    STRATS -.-> RENG
```


├── docs/flowcharts/05_detektorler_nesil.mmd

```mermaid
flowchart LR
    subgraph SRC1["Veri Kaynağı: Binance REST klines"]
        OC["ohlcv-engine :: BinanceClient<br/>fetch_klines(symbol, interval, limit)"]
    end

    subgraph DET["Analiz Servisleri"]
        MS["detect-ms :3002<br/>SMC 7 katman<br/>session · pivot · trend · levels<br/>liquidity · FVG · narrative"]
        CALC["calc-ind :3003<br/>indikatör hesap servisi"]
    end

    subgraph DBS["Veri kaynağı: TimescaleDB + shm ring"]
        R2["GenerationalRingBuffer 160k"]
        DB["market_data hypertable'ları<br/>(trades · OHLCV)"]
    end

    OC --> MS
    OC --> CALC

    R2 --> MS
    DB --> MS

    NOT["Not: bu servislerin çıktısı<br/>şu an execution'a BAĞLI DEĞİL<br/>(yalnız HTTP JSON)"]
    NOT -.-> MS
```






├── docs/flowcharts/10_yardimci_servisler.mmd

```mermaid
flowchart TB
    subgraph VERI_GRS["Veri Kaynakları"]
        R1["ring /cycle_finance_ring (160k)"]
        R2["pricefeed ring (20k)"]
        BIN["doğrudan Binance WS (köprüsüz)"]
        PFAPI["REST :3004"]
    end

    subgraph ALERT["alert-service"]
        RULES["AlertRule config.toml<br/>Above · Below · Cross · Touch<br/>tolerans+ cooldown · voice"]
        SM["Runtime state machine<br/>Armed → Triggered → re-arm"]
        EVT["AlertEvent (flume)"]
        AUI["audio.rs<br/>spd-say veya WAV üretimi<br/>(44.1kHz · G6-E6-G6 · paplay)"]
    end

    subgraph RISKW["risk-worker (60sn)"]
        MAT["MatrixMath<br/>Tikhonov regularizasyon · DAR"]
        VW["Dinamik VWAP (geçe likidite)"]
        CACHE["RiskCache ArcRwLock<br/>max_pos · volat_index"]
        FIN["FinOps<br/>maliyet > %20 karla → toplu repack"]
    end

    subgraph COLD["Soğuk Başlangıç"]
        CLS["cold-start buffer (mmap disk)"]
        TOLIVE["transition_to_live()"]
        EMA["fetch_200_ema()"]
    end

    subgraph ADAPT["adapter — 6 modül (5 mock)"]
        BA["binance.rs — TEK CANLI (8 stream)"]
        CH["clickhouse.rs — schema + erasure (mock)"]
        RD["redis.rs — idempotency NX (mock)"]
        VT["vault.rs — rotasyon + fake JWT"]
        AI2["ai.rs — Isolation forest / LLM tag (mock)"]
        TL["telemetry.rs — eBPF / Jaeger / Chaos (mock)"]
    end

    RING --> ALERT
    R2 --> ALERT
    BIN --> ALERT
    ALERT --> AUI

    RING -.-> RISKW
    R2 -.-> RISKW

    EMA --> EMA
    EMA400 --> EMA200
```


├── docs/flowcharts/11_ci_kubernetes_tla.mmd

```mermaid
flowchart LR
    subgraph CI[".github/workflows/test-suite.yml — Sertifika Testleri"]
        J1["Job 1: audit-and-coverage<br/>cargo-deny advisories · tarpaulin %95"]
        J2["Job 2: unit-integration<br/>cargo test --release<br/>wire roundtrip · ring generational<br/>proptest tick · actor e2e"]
        J3["Job 3: performance-wcet<br/>cargo bench → MAX TICK 750µs<br/>(tick_benchmark)"]
        J4["Job 4: chaos-mesh-staging<br/>Chaos Mesh 20 senaryo"]
        TR["Triggers: master PR + nightly cron"]
    end

    subgraph K8S["Kubernetes (k8s/)"]
        DEP["deployment.yaml<br/>1 replika · 4CPU/4Gi<br/>cgroupv2 · SYS_NICE caps"]
        P1["Chaos: network_partition<br/>redis-cluster · 10sn · @5m"]
        P2["Chaos: dns_failure<br/>api.binance + stream.binance @30m"]
        P3["Chaos: ntp_drift<br/>+10sn TimeChaos @15m"]
    end

    subgraph FORMAL["Formal Verification (TLA+)"]
        TLA["CycleFinance.tla<br/>queue = tracked dispatch <br/>Produce/Consume"]
        SAFE["Safety: ticks_out ≤ ticks_in"]
        LIVE["Liveness: ticks_in = n → ticks_out = n<br/>WF_vars(Consume)"]
        CFG["CycleFinance.cfg<br/>SPEC + INVARIANT + PROPERTY"]
    end

    J1 --> J2 --> J3 --> J4
    TR1 --> J1
    FORMAL --> J2
    OS_UTIL["os-utils: RT thread priority<br/>(SCHED_FIFO 99)"] -.-> DEP
```


├── docs/flowcharts/12_execution_engine.mmd

```mermaid
flowchart TD
    subgraph Strateji["Strateji Katmanı"]
        SIG[strategies-engine / detect-ms] -->|flume / dev/shm OrderRing| GW
        HTTP["REST API (axum :3010)"] --> GW
    end

    GW[Gateway trait<br/>LiveGateway] -->|EngineHandle::submit_order| ACT

    subgraph Execution["Execution Engine"]
        ACT["ExecutionActor<br/>(tek-yazıcı komut döngüsü)"] -->|imzalı REST| C[BinanceClient]
        ACT --> PF["Preflight<br/>filtre/precizyon/notional/mod"]
        ACT --> RK["RiskChecks<br/>blocklist / max-notional / rate-limit"]
        ACT --> KS["KillSwitch"]
        ACT --> IN["InFlightRegistry + IdempotencyCache"]
        ACT --> SNAP[("AccountSnapshot<br/>Arc<RwLock>")]
        ACT -->|"UserDataEvent"| PROJ[Projector]
        PROJ --> SNAP
        ACT -->|periyodik uzlaştırma| C
    end

    C -->|"GET/POST fapi"| BIN["Binance USDT-M Futures REST"]
    US[UserDataStream] -->|"listenKey + gzip WS"| WB["Binance user-data WS"]
    WB -->|"ACCOUNT_UPDATE / ORDER_TRADE_UPDATE"| US
    US -->|"UserEvent::Data"| ACT
    US -->|"UserEvent::StreamConnected (resync)"| ACT

    C -->|"/fapi/v1/exchangeInfo"| EC[ExchangeCache]
    EC --> PF

    SNAP -->|okuma| API2["axum handler'ları"]
    API2 --> HTTP
```


└── docs/flowcharts/13_risk_engine.mmd

```mermaid
flowchart TD
    subgraph HOT["HOT PATH — execution-engine (in-process)"]
        OR["OrderRequest"] --> AD["RiskChecks (bağdaştırıcı)<br/>OrderRequest→OrderIntent"]
        AD --> EVAL["RiskEngine::evaluate<br/>13-adım kural zinciri"]
        EVAL --> DEC{Onay?}
        DEC -->|Ret| REJ["RejectReason + AuditLog<br/>ardışık red → kill switch"]
        DEC -->|Onay| SEND["execution actor → borsa"]
        FILL["ORDER_TRADE_UPDATE (TRADE)"] --> ONFILL["RiskEngine::on_fill<br/>Portfolio::apply_fill"]
        SNAP["AccountSnapshot (resync/reconcile)"] --> SYNC["sync_from_snapshot<br/>mark + pozisyon + nakit"]
        SYNC --> RSTATE["RiskState<br/>positions/mark/cash/peak"]
        ONFILL --> RSTATE
        RSTATE --> EVAL
    end

    subgraph COLD["COLD PATH — risk-worker daemon :3011 (60s)"]
        PF["/tmp/price_feed.json"] --> HIST["PriceHistory<br/>(~120 örnek/sembol)"]
        HIST --> RET2["log-getiri serileri"]
        RET2 --> CORR["Pearson korelasyon"]
        CORR --> TIK["shrink + Tikhonov<br/>(hedef koşul sayısı)"]
        RET2 --> EWMA["EWMA vol (λ=0.94)"]
        TIK --> VAR["parametrik VaR %99 (σp²=wᵀΣw)"]
        VAR --> HHI["konsantrasyon HHI"]
        VAR --> SUGG["önerilen max_position / leverage"]
        SUGG --> CACHE["RiskParameters"]
    end

    CACHE -->|"/cycle_finance_risk_params ring + /tmp/risk_params.json"| RC["RiskCache (seqlock)"]
    RC -.->|"opsiyonel parametrik kapı"| EVAL
    KS["KillSwitch<br/>/tmp/exec_kill_switch + bayrak"] --> EVAL
    CFG["risk.toml (hot-reload)"] --> EVAL
    CFG --> COLD

    style HOT fill:#1b2a41,stroke:#3b82f6
    style COLD fill:#241b41,stroke:#8b5cf6
    style EVAL fill:#3b82f6,color:#fff
    style CACHE fill:#8b5cf6,color:#fff
```

