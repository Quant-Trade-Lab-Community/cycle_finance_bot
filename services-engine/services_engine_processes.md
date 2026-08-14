# SERVICES-Engine Fonksiyonel Süreçler

## Giriş

services-engine, 9 paket (8 servis + breakout-strategy) içerir. Her birinde kendi Cargo crate'li. `Cargo.toml` workspace üyesi.

Başlatma:
- `cargo run -p alert-service` (kök workspace)
- `cargo run -p calc-ind`
- `cargo run -p detect-ms`
- `cargo run -p exec-console`
- `cargo run -p ohlcv-engine` (cli: `cli`, server: `server`)
- `cargo run -p`
- `cargo run -p stream-ohlcv`
- `cargo run -p trade-ohlcv`
- `cargo run -p breakout-strategy`

---

## Süreç 1: alert-service (Sesli Uyarı)

### Amaç
Sembol + fiyat koşulu (above/below/cross/touch) için sesli uyarı üretir. Veriyi 3 kaynaktan alır: DATA ring'i, ring'i veya doğrudan Binance WS.

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/main.rs` | 160 | Ana döngü (tokio task) — Binance WS reconnect döngüsü |
| `src/source.rs` | 158 | Ring okuyucular (ring'inden, DATA ring'inden) |
| `src/engine.rs` | 189 | Sesi sink (alertService) |
| `src/config.rs` | 99 | Config yükleme |
| `src/lib.rs` | 4 | Modül tanımı |

### Veri Akışı

```
Binance Futures (WS):
  └── (port 3004) → /cycle_finance_pricefeed ring

alert-service:
  ├── listen: /dev/shm/cycle_finance_ring (DATA terminali)
  ├── fiyat oku (ring: source.rs:15)
  ├── koşul kontrol: above/below/cross/touch
  └── uyarı çıkar (sound/audio)
```

### Altyapı

- Ring okuyucu: source.rs:14 (DATA ring), source.rs:108 (ring).
- Ses sink: engine.rs:183 (ses çıkışı).
- Çevrimli stdin: main.rs:93 (CLI girişi).

---

## Süreç 2: calc-ind (Indikatör Hesaplama)

### Amaç
`POST /api/calc` ile indikatör hesaplama (SMA/EMA/MACD/RSI/BBANDS/VWAP/ATR...). OHLCV'yi ohlcv-engine client'ından çeker, `ferro_ta_core` ile hesaplar, sonucu binary olarak `/cycle_finance_calc` ring'ine yazar, `request_id` döndürür.

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/lib.rs` | 154 | Modül tanımı (calc_ind, Kline struct) |
| `src/main.rs` | 108 | Ana döngü (axum server) |
| `src/indicators.rs` | 135 | İndikatör hesaplama (SMA, EMA, MACD, RSI, BBANDS, VWAP, ATR) |
| `src/client.rs` | 79 | Binance REST client (fetcher) |

### Veri Akışı

```
calc-ind (port 3007):
  ├── GET /api/calc (axum)
  ├── ohlcv_engine client → Binance Futures REST
  │   └── fetch_klines (client.rs:20-27) → Kline struct
  ├── ring okuma (calc_ind client: lib.rs:97,128)
  ├── hesaplama (indicators.rs:135)
  └── /cycle_finance_calc ring → tüketici client'lar
```

### Ring Buffer (calc-ind)

```
/cycle_finance_calc (64 slot, 702 byte)
  └── calc-ind: push → ring → tüketici client'lar
  └── calc_ind client: lib.rs:97,128 (HTTP+ring read)
```

---

## Süreç 3: detect-ms (MSMP 2.0 Analizi)

### Amaç
MSMP 2.0 (Market Structure Multi-Protocol) — 7 katmanlı yapı analizi:

1. seans pencereleri (seans pencereleri)
2. pivot çıkarımı
3. trend (regresyon+Hurst)
4. seviye envanteri
5. likidite (VWAP/volume profile/BSL-SSL)
6. FVG+delta
7. nihai naratif rapor

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/main.rs` | 108 | Ana döngü (axum server, 3 sıralı Binance fetch) |
| `src/imbalance.rs` | 143 | Seans pencereleri |
| `src/levels.rs` | 194 | Pivot çıkarımı |
| `src/liquidity.rs` | 271 | Likidite (VWAP/volume profile/BSL-SSL) |
| `src/narrative.rs` | 272 | Nihai naratif rapor |
| `src/pivot.rs` | 204 | Pivot çıkarma |
| `src/session.rs` | 93 | Seans pencereleri |
| `src/trend.rs` | 249 | Trend (regresyon+Hurst) |

### Veri Akışı

```
detect-ms (port 3002):
  ├── GET /api/ms?symbol=&interval=&limit= (axum)
  ├── 3 sıralı Binance fetch:
  │   ├── imleç (detect-ms)
  │   ├── seviye envanteri (levels.rs)
  │   ├── trend analizi (trend.rs)
  │   ├── likidite analizi (liquidity.rs)
  │   └── FVG+delta analizi
  └── nihai rapor (narrative.rs:272)
```

---

## Süreç 4: exec-console (Komut Konsolu)

### Amaç
`executiond` (canlı execution) için JWT'li interaktif komut konsolu (rustyline REPL). Emir/pozisyon/bakiye/risk/kill-switch yönetimi.

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/main.rs` | 664 | Ana döngü (tek thread) |

### Akış

```
exec-console (terminal REPL)
  ├── rustyline: "strategy> " prompt
  ├── komut satırı: strat run breakout momentum
  ├── execute: reqwest blocking (executiond:3010 REST API)
  ├── result: emir/pozisyon/bakiye/yönetimi (executiond)
```

---

## Süreç 5: ohlcv-engine (Kline Sürümü)

### Amaç
Binance Futures REST'ten (fapi/v1/klines) Kline çeker; küçük bir HTTP sunucusu (port 3000) ve terminal CLI'i barındırır.

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/lib.rs` | 19 | Modül tanımı (Kline struct) |
| `src/client.rs` | 79 | Binance REST client (fetch_klines, fetch_klines_range) |
| `src/bin/cli.rs` | 56 | CLI (terminal radar tablosu) |
| `src/bin/server.rs` | 61 | HTTP sunucusu (axum: 127.0.0.1:3000) |

### Veri Akışı

```
ohlcv-engine:
  ├── Kline (client): Binance Futures REST (/fapi/v1/klines)
  ├── Kline struct: open_time → taker_buy_quote_asset_volume (client.rs:7-9)
  ├── server: GET /api/klines?symbol=&interval=&limit= (axum)
  └── cli: terminal radar tablosu (cli.rs:35-51)
```

### Server (port 3000)

```
axum (server.rs:31)
  ├── GET /api/klines?symbol=&interval=&limit=
  ├── bind: 127.0.0.1:3000 (server.rs:34)
  └── yanıt: {status, symbol, interval, count, data} (server.rs:49-55)
```

---

## Süreç 7: (Veri Dağıtım)

### Amaç
Binance Futures WS'den (trade/bookTicker) fiyat çeker; ring'e yayar.

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/main.rs` | 366 | Ana döngü (tokio task) |
| `src/pipeline.rs` | 305 | HTTP API: /api/lastprice/{sym} |

### Veri Akışı

```
(port 3004):
  ├── Binance Futures WS: trade/bookTicker
  │   └── user_data stream → ws_pump (main.rs:315)
  ├── premiumIndex poll (main.rs:323)
  └── JSON yazıcı (main.rs:341)

Ring: /cycle_finance_pricefeed (20_000 slot)
  └── → all consumers (alert-service, breakout-strategy)
```

### Thread/Task Yapısı

```
(tokio + std::thread):
  ├── tokio task: ws_pump (main.rs:315)
  ├── tokio task: premiumIndex poll (main.rs:323)
  └── std thread: ingest (parser+ring+state) (main.rs:335)
  └── flume bounded 262_144 (main.rs:311)
```

---

## Süreç 8: stream-ohlcv (Çoklu Stream OHLCV)

### Amaç
Sembol + başlangıç zamanı + interval ile canlı OHLCV mum akışı üretir. Geçmişi ohlcv-engine'den çeker, canlı fiyatı :3004'ten poll eder, kapanan mumları /cycle_finance_stream_ohlcv ring'ine yayınlar.

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/lib.rs` | 321 | Modül tanımı (stream_ohlcv) |
| `src/main.rs` | 537 | Ana döngü (tokio) |
| `src/stream_ring.rs` | 28 | Ring write kilidi (stream_ring.rs:28) |

### Veri Akışı

```
stream-ohlcv (port 3008):
  ├── tokio task: start_stream → tokio::spawn(run_stream) (main.rs:340)
  ├── stream durumu: tokio::sync::RwLock
  ├── ring write kilidi: Mutex (main.rs:54)
  ├── ring: /cycle_finance_stream_ohlcv (8192 slot)
  └── stream_ohlcv client: lib.rs:14,233
```

---

## Süreç 9: breakout-strategy (Event-Driven Kırılım)

### Amaç
Event-driven kırılım stratejisi (sinyal üretici, emir açmaz). ring'inden tick okur, bekleme aralığında detect-ms analizini çağırır, direnç/destek kırılımında BUY/SELL sinyali üretir.

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/lib.rs` | 3 | Core module (struct definitions) |
| `src/main.rs` | 308 | Ana strateji (Event-Driven Kırılım) |
| `src/metrics.rs` | 580 | Mikro-yapı metrik çekirdek |
| `src/bin/listener.rs` | 282 | Mikro-yapı metrik paneli |
| `src/bin/alerts.rs` | 223 | alerts.toml CLI |
| `src/bin/risk_analysis.rs` | 112 | SQL risk raporu |

### Veri Akışı

```
breakout-strategy:
  ├── main.rs (308): Event-Driven Kırılım
  │   ├── ring: /cycle_finance_pricefeed
  │   ├── detect-ms API: GET /api/ms?symbol=VELVETUSDT
  │   ├── fiyat ring: /cycle_finance_pricefeed
  │   ├── evaluate: detect-ms + → sinyal üretir
  │   └── emir açmaz (sinyal üretici — emir açmaz)
  ├── listener.rs:38: price corr thread (200ms)
  ├── alerts.rs:223: alerts.toml CLI
  └── risk_analysis.rs:112: Risk dağıtım raporu
```

---

## Süreç 10: trade-ohlcv (Trade → 1s OHLCV)

### Amaç
Trade flow ring'ini (`/cycle_finance_trades`) okuyup sembol başına **1 saniyelik OHLCV** barları üretir. Kapanan barları `/cycle_finance_trade_ohlcv` ring'ine binary yayınlar, stdout'a canlı stream eder (tmux pencere 20) ve HTTP API ile sunar.

### Giriş Noktaları

| Dosya | Satır | Sorumlu |
|:---|:---|:---|
| `src/lib.rs` | 431 | TradeCandle, SecondAggregator, codec, client, testler |
| `src/main.rs` | 336 | Daemon (ring okuyucu + aggregator + axum API) |
| `examples/read_ring.rs` | 29 | Tüketici örneği (ring okuma) |

### Veri Akışı

```
DATA terminali / flows (trade akışı)
  └── /dev/shm/cycle_finance_trades (Trade event'leri)
        └── trade-ohlcv kaynak thread (spin-loop, retry pencereli)
             └── flume bounded (262_144)
                  └── aggregator task → SecondAggregator → 1s bar
                       ├── kapalı bar → codec::encode → /cycle_finance_trade_ohlcv (StreamRingBuffer)
                       ├── stdout canlı stream: [HH:MM:SS] SYM 1s O=.. H=.. L=.. C=.. V=.. TB=.. n=..
                       └── HTTP API: /api/health · /api/symbols · /api/candles/{symbol}
```

### Thread / Task Yapısı

- std thread: trade flow ring okuyucu — cursor'dan head'e; slot yoksa 40×100µs retry penceresi, sonra overwrite kontrolü (ilave event kaçırma).
- tokio task: aggregator — `SecondAggregator::on_trade` ile 1s dilimi takibi, kapalı mum → ring + stream + state (VecDeque cache, CACHE_MAX=500).
- axum server (:3009, env `TRADE_OHLCV_PORT`): 3 route, `infra::util::single_instance("trade-ohlcv")`.

---

## Thread / Task Yapısı (Giriş Noktaları)

| Servis | Model | Detay |
|:---|:---|:---|
| alert-service | tokio + std::thread | tokio task: Binance WS reconnect; std thread: ring okuyucular (source.rs:14,108), ses sink (engine.rs:183), stdin CLI (main.rs:93) |
| calc-ind | tek axum server (async) | Handler başına request; ek thread yok |
| detect-ms | tek axum server (async) | Handler'da 3 sıralı Binance fetch (main.rs:85-96); ek thread yok |
| exec-console | tek thread, blocking | reqwest blocking + rustyline REPL; thread spawn yok |
| ohlcv-engine | cli: tek async; server: axum | Ek thread yok |
| | tokio + std::thread | tokio task: ws_pump (main.rs:315); std thread: ingest (main.rs:335); flume bounded 262_144 (main.rs:311) |
| stream-ohlcv | tokio | Her stream bir task: start_stream → tokio::spawn(run_stream) (main.rs:340); stream durumu: tokio::sync::RwLock; ring write kilidi Mutex (main.rs:54) |
| trade-ohlcv | tokio + std::thread | std thread: trade flow ring okuyucu (retry pencereli) → flume 262_144 → tokio aggregator task (SecondAggregator) → ring + stdout stream + axum (:3009) |
| breakout-strategy | tokio + std::thread | main.rs:168 std thread ring okuyucu → mpsc::unbounded_channel (main.rs:276) → tokio actor döngüsü (main.rs:283); listener.rs:38 std thread price corr |

---

## Satır Sayıları (toplam 7055)

| Servis | Toplam |
|:---|:---|
| alert-service | 746 |
| calc-ind | 397 (+30 örnek) |
| detect-ms | 1534 |
| exec-console | 664 |
| ohlcv-engine | 215 |
| | 366 |
| stream-ohlcv | 858 |
| trade-ohlcv | 767 (+29 örnek) |
| breakout-strategy | 1508 |
| **TOPLAM** | **7055** (+59 örnek) |

---

## Sonuç

services-engine, 9 paket ve 30+ binary'i içerir. Her birinde kendi thread modeli ve altyapısı vardır. Ring buffer (POSIX shm) ile veri paylaşımı: `/cycle_finance_ring` en çok bağlanır; `/cycle_finance_pricefeed` ve `/cycle_finance_calc` ise ring buffer üretici/tüketici çiftidir.

`trade-ohlcv` (:3009), `/cycle_finance_trades` flow ring'inden trade verisini okuyup sembol başına 1s OHLCV barı üretir; kapanan barları `/cycle_finance_trade_ohlcv` ring'ine binary yayınlar ve stdout'a canlı stream eder.

Breakout-strategy, 1508 satırda; alert-service, 746 satırda. Veri akışı ring buffer+HTTP+WS'lerle gerçekleştirilir.