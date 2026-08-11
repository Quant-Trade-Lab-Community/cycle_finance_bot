# SERVICES-Engine Mimari Dokümanı

## Genel Bakış

**services-engine**, Cycle Finance sisteminin **orta katmanını** (intermediate layer) oluşturur. 9 paket (8 servis + breakout-strategy):

- `alert-service`: Sembol + fiyat koşulu (above/below/cross/touch) için sesli uyarı.
- `calc-ind`: `POST /api/calc` ile indikatör hesaplama (SMA/EMA/MACD/RSI/BBANDS/VWAP/ATR...). Ring `get()`.
- `detect-ms`: MSMP 2.0 (7 katmanlı yapı analizi).
- `exec-console`: executiond (canlı execution) için JWT interaktif komut konsolu.
- `ohlcv-engine`: Binance Futures REST Kline çeker, HTTP sunucusu + CLI.
- `paper-service`: Event-sourcing + actor tabanlı sanal (kağıt) motoru.
- `price-feed`: Binance Futures WS'den fiyat çeker, ring'e yayar.
- `stream-ohlcv`: Çoklu stream OHLCV mum akışı.
- `breakout-strategy`: Event-driven kırılım stratejisi (sinyal üretici, emir açmaz).

**Yapı:** Her servis bir Cargo crate'dır; `src/bin/` folderunda her biride binary. `src/main.rs` (default bin).

**Service-to-service bağlantısı:** `transport` (POSIX shm ring) ve `execution-engine` (executiond REST API).

---

## Katmanlar ve Modül Sorumlulukları

| Servis | Amaç | Dosyalar |
|:---|:---|:---|
| **alert-service** | Sembol + fiyat koşulu (above/below/cross/touch) için sesli uyarı | src/main.rs, source.rs, engine.rs, config.rs |
| **calc-ind** | `POST /api/calc` ile indikatör hesaplama; OHLCV ring'e yazar | src/lib.rs, main.rs, indicators.rs |
| **detect-ms** | MSMP 2.0 (7 katmanlı yapı analizi) | src/main.rs, detect-ms, levels.rs, pivot.rs, trend.rs, liquidity.rs, narrative.rs |
| **exec-console** | executiond için JWT interaktif komut konsolu | src/main.rs, exec-console |
| **ohlcv-engine** | Binance Futures REST Kline çeker; HTTP sunucusu + CLI | src/lib.rs, client.rs, bin/cli.rs, bin/server.rs |
| **paper-service** | Event-sourcing + actor tabanlı sanal motoru | src/main.rs, bridge.rs, events.rs, sqlite_projection.rs |
| **price-feed** | Binance Futures WS'den fiyat çeker, ring'e yayar | src/main.rs, pipeline.rs |
| **stream-ohlcv** | Çoklu stream OHLCV mum akışı | src/lib.rs, main.rs, stream_ring.rs |
| **breakout-strategy** | Event-driven kırılım stratejisi | src/lib.rs, main.rs, metrics.rs, bin/listener.rs, bin/alerts.rs, bin/risk_analysis.rs |

---

## Veri Akışı

### Ring Buffer (POSIX shm, `/dev/shm`)

| Ring | Kapasite | Üretici | Tüketiciler |
|:---|:---|:---|:---|
| `/cycle_finance_ring` | 160_000 | DATA terminali (cycle-engine, RUN_MODE=DATA) | alert-service source.rs:15; breakout listener.rs:77; breakout metrics.rs:3 (yorum) |
| `/cycle_finance_pricefeed` | 20_000 | price-feed main.rs:38,305 | alert-service source.rs:110; paper-service bridge.rs:30; breakout main.rs:169 |
| `/cycle_finance_calc` | 64 | calc-ind main.rs:20-21,39 | calc_ind client lib.rs:97,128 (HTTP+ring read) |
| `/cycle_finance_stream_ohlcv` | 8192 | stream-ohlcv main.rs:34,513 (STREAM_DEFAULT_CAPACITY, stream_ring.rs:28) | stream_ohlcv client lib.rs:14,233 |
| `/cycle_finance_orders` | 10_000 | STRATEGY terminali / execution tarafı (order_ring.rs:53) | paper-service bridge.rs:90 |

> **Note:** alert-service ayrıca `is_ring_alive()` ile price-feed ring'ini yoklar (source.rs:100-103).

### HTTP API Kaynakları

| Kaynak | Yöntem | Adres | Kod |
|:---|:---|:---|:---|
| price-feed | `GET /api/lastprice/{symbol}` | `:3004` | `context.rs:70-71` |
| calc-ind | `POST /api/calc` + `/cycle_finance_orders` ring okuma | `:3007` | `context.rs:92-128` |
| detect-ms | `GET /api/ms?symbol=&interval=&limit=` (MSMP 2.0) | `:3002` | `context.rs:131-135` |
| paper hesap | JWT login → balance + positions | `:8080` | `context.rs:166-217` |
| risk politikası | `risk.toml` dosyası | kök dizin | `gates.rs:39` |

### Veri Akışı Özet

```
Binance Futures (REST /fapi/v1/klines)
  └── ohlcv-engine (kütüphane/client)
  │
  ├── calc-ind (port 3007) → /cycle_finance_calc ring → tüketici client'lar
  ├── detect-ms (port 3002) → /api/ms raporu → breakout-strategy
  └── stream-ohlcv (port 3008) → geçmiş mumlar → /cycle_finance_stream_ohlcv ring

Binance Futures (WS fstream)
  └── price-feed (port 3004)
        ├── /cycle_finance_pricefeed ring → alert-service, paper-service, breakout-strategy
        └── HTTP /api/lastprice/{sym} → stream-ohlcv (canlı mum poll)

DATA terminali (cycle-engine)
  └── /cycle_finance_ring → alert-service (ring modu), breakout listener

STRATEGY terminali
  └── /cycle_finance_orders → paper-service bridge → PaperEngineActor
```

### Ring Buffer (cycle-engine/src/transport/ring_buffer.rs)

```
GenerationalRingBuffer::with_name("/cycle_finance_ring", 160_000, 1024)
  └── shm_open, set_len, mmap → ring_buffer.rs:62
  └── push/read → ring_buffer.rs:45-93
  └── GenerationalRingBuffer name: /cycle_finance_ring
```

### HTTP Client (ohlcv-engine)

`ohlcv-engine/src/client.rs:7-9`:
- `BinanceClient { http: reqwest::Client }`
- `fetch_klines` (client.rs:20-27): symbol/interval/limit → son N kline.
- `fetch_klines_range` (client.rs:31-78): start_ms/end_ms opsiyonel. Endpoint: `https://fapi.binance.com/fapi/v1/klines` (client.rs:40). 11 elemanlı array'i `Kline`'a çözer (client.rs:59-71).

---

## Thread / Task Yapısı

### alert-service (tokio + std::thread karışımı)

- tokio task: Binance WS reconnect döngüsü (main.rs:61), fiyat→motor taşıyıcı (main.rs:83).
- std thread: ring okuyucular (source.rs:14,108), ses sink (engine.rs:183), stdin CLI (main.rs:93).

### calc-ind (tek axum server)

- Handler başına request; ek thread yok.
- `calc-ind` (port 3007) → `/cycle_finance_calc` ring → tüketici client'lar.

### detect-ms (tek axum server)

- Handler'da 3 sıralı Binance fetch (main.rs:85-96); ek thread yok.
- `detect-ms` (port 3002) → `/api/ms` raporu → breakout-strategy.

### exec-console (tek thread, blocking)

- reqwest blocking + rustyline REPL; thread spawn yok.

### ohlcv-engine (cli: tek async; server: axum)

- Ek thread yok.

### paper-service (tokio + std::thread)

- tokio task: event persistence loop (main.rs:63, Sled+PG+SQLite).
- tokio task: actor loop (main.rs:110).
- tokio task: REST API (main.rs:140).
- std thread: bridge okuyucular (bridge.rs:29, price-feed) ve (bridge.rs:89, order ring).

### price-feed (tokio + std::thread)

- tokio task: ws_pump (main.rs:315), premiumIndex poll (main.rs:323), JSON yazıcı (main.rs:341).
- std thread: ingest (parser+ring+state) (main.rs:335).
- flume bounded 262_144 (main.rs:311).

### stream-ohlcv (tokio, her stream bir task)

- `start_stream` → `tokio::spawn(run_stream)` (main.rs:340).
- stream durumu: tokio::sync::RwLock, ring write kilidi: Mutex (main.rs:54).

### breakout-strategy (tokio + std::thread)

- `main.rs:168` std thread ring okuyucu → mpsc::unbounded_channel (main.rs:276) → tokio actor döngüsü (main.rs:283).
- `listener.rs:38` std thread price corr: 200 ms'de bir :3004/api/lastprice çeker, fiyat CorrSeries'lerine yazar.
- Ana döngü: `REFRESH_MS=2000`, 2 sn'de bir render + JSON.

---

## Satır Sayıları (wc -l)

| Servis | Toplam | Dosya |
|:---|:---|:---|
| alert-service | 746 | audio.rs 136, config.rs 99, engine.rs 189, lib.rs 4, main.rs 160, source.rs 158 |
| calc-ind | 397 (+30 örnek) | lib.rs 154, main.rs 108, indicators.rs 135; examples/read_ring.rs 30 |
| detect-ms | 1534 | imbalance.rs 143, levels.rs 194, liquidity.rs 271, main.rs 108, narrative.rs 272, pivot.rs 204, session.rs 93, trend.rs 249 |
| exec-console | 664 | main.rs 664 |
| ohlcv-engine | 215 | bin/cli.rs 56, bin/server.rs 61, client.rs 79, lib.rs 19 |
| paper-service | 1476 | api.rs 447, bin/paper_cli.rs 228, bridge.rs 144, events.rs 117, idempotency.rs 40, lib.rs 9, main.rs 153, metrics.rs 70, postgres_store.rs 98, sqlite_projection.rs 170 |
| price-feed | 366 | main.rs 366 |
| stream-ohlcv | 858 | lib.rs 321, main.rs 537 |
| breakout-strategy | 1508 | bin/alerts.rs 223, bin/listener.rs 282, bin/risk_analysis.rs 112, lib.rs 3, main.rs 308, metrics.rs 580 |
| **TOPLAM** | **7764** (+30 örnek = 7794) | |

---

## Servisler Arası İlişki (İç-Tek)

### HTTP tüketiciler

| Tüketici | Bağlantı |
|:---|:---|
| calc-ind | ohlcv-engine (port 3007) → /cycle_finance_calc ring |
| detect-ms | ohlcv-engine (port 3002) → /api/ms raporu → breakout-strategy |
| stream-ohlcv | ohlcv-engine (port 3008) → geçmiş mumlar → /cycle_finance_stream_ohlcv ring |
| breakout-strategy | detect-ms (:3002) + price-feed (:3004) |
| exec-console | executiond (:3010) |
| stream-ohlcv | price-feed (:3004) |

### Ring Akışı

- price-feed ring'i en çok bağlanan ortak veri yoludur (3 servis okur).
- calc-ind ve stream-ohlcv ring'leri üretici/tüketici çiftidir (isteğe bağlı binary akış).

### DB tüketicileri

- paper-service → Sled WAL + SQLite (+ ops. Postgres).
- breakout-strategy risk_analysis.rs:20 → data-engine/data/market_data.db.

---

## Komut Başlatmaları

| Servis | Komut | Port |
|:---|:---|:---|
| alert-service | `alert-service` | `src/main.rs` (default bin) |
| calc-ind | `calc-ind` (lib: calc_ind) | `src/main.rs` (default bin) |
| detect-ms | `detect-ms` | `src/main.rs` (default bin) |
| exec-console | `exec-console` | `src/main.rs` (default bin) |
| ohlcv-engine | `cli`, `server` | `src/bin/cli.rs`, `src/bin/server.rs` |
| paper-service | `paper-service` (default-run) + `paper-cli` | `src/main.rs` + `src/bin/paper_cli.rs` |
| price-feed | `price-feed` | `src/main.rs` (default bin) |
| stream-ohlcv | `stream-ohlcv` (lib: stream_ohlcv) | `src/main.rs` (default bin) |
| breakout-strategy | `breakout-strategy` + `alerts` + `listener` + `risk_analysis` | `src/main.rs` (`[[bin]]`) + `src/bin/alerts.rs`, `src/bin/listener.rs`, `src/bin/risk_analysis.rs` |

---

## Sonuç

services-engine, 9 paket ve 30+ binary'i içerir. Her birinde kendi thread modeli ve altyapısı vardır. Ring buffer (POSIX shm) ile veri paylaşımı: `/cycle_finance_ring` en çok bağlanır; `calculate-ind`, `detect-ms`, `stream-ohlcv` ise `ohlcv-engine` (kütüphane) ile `calc-ind`, `stream-ohlcv` tarafından kullanılır.

Breakout-strategy, 1508 satırda; alert-service, 746 satırda. Veri akışı ring buffer+HTTP+WS'lerle gerçekleştirilir.