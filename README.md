# 🏛️ Cycle Finance — Yüksek Frekanslı Alım-Satım Sistemi

> **High-Frequency Trading / Market Structure Analysis Platform**
> Rust monorepo · event-driven mimari · düşük gecikme hedefli

---

## 📌 İçindekiler

- [Genel Bakış](#-genel-bakış)
- [Klasör Yapısı](#-klasör-yapısı)
- [Katmanlı Mimari](#-katmanlı-mimari)
- [Aktif Servisler](#-aktif-servisler)
- [Deaktive Edilmiş Servisler](#-deaktive-edilmiş-servisler)
- [Kırılım Stratejisi (Breakout)](#-kırılım-stratejisi-breakout)
- [Execution Engine (Canlı)](#-execution-engine-canlı)
- [Veri Kaydı](#-veri-kaydı)
- [Kurulum](#-kurulum)
- [Çalıştırma](#-çalıştırma)
- [tmux Terminal Düzeni](#-tmux-terminal-düzeni)
- [Testler](#-testler)
- [CI / K8s](#-ci--k8s)
- [Teknik Detaylar](#-teknik-detaylar)

---

## 📖 Genel Bakış

**Cycle Finance**, Binance Futures verisi üzerinde gerçek zamanlı piyasa yapısı analizi yapan ve **kırılım (breakout) sinyalleri** üreten yüksek frekanslı bir alım-satım platformudur.

Ana akış:

```
Binance Futures WS
   → adapter (binance WS client)
   → core (parser + validator + SQLite yazıcı)
   → /dev/shm ring buffer (IPC)
   → detect-ms (Market Structure Multi-Protocol, 7 katman)
   → breakout-strategy (kırılım sinyali: sembol + yön)
```

**Not:** Bu sistem şu an **paper-trading** modundadır. `breakout-strategy` yalnızca **sembol + yön** sinyali üretir, emir açmaz. `paper-service` sanal emir yürütme sağlar.

> 🛡️ **`execution-engine` artık canlı (LIVE) Binance Futures emir altyapısına sahiptir** (REST + user-data WS, idempotency, preflight, kill switch). Varsayılan `EXEC_DRY_RUN=true`'dır — gerçek emir için **bilinçli onay** gerekir. Detaylar: [Execution Engine (Canlı)](#-execution-engine-canlı).

---

## 🗂️ Klasör Yapısı

```
PROJE/
├── cycle-engine/          # Çekirdek kütüphaneler (Katman 0-2)
│   ├── contracts/         #   Veri sözleşmeleri (events, wire binary codec)
│   ├── transport/         #   IPC: /dev/shm ring buffer'lar
│   ├── core/              #   Parser, validator, orkestratör, LOB sim, timer
│   └── adapter/           #   Dış entegrasyonlar (Binance WS, Redis, Vault...)
│
├── additional-services/   # Operasyonel destek
│   ├── os-utils/          #   RT zamanlayıcı (SCHED_FIFO), lock-free config
│   ├── scripts/           #   tmux başlatıcılar, izleme, ortam betikleri
│   ├── config/            #   TOML yapılandırmaları
│   ├── k8s/               #   Kubernetes manifestleri + chaos senaryoları
│   └── formal_verification/ # TLA+ spesifikasyonları
│
├── data-engine/           # Veri katmanı
│   ├── cold-storage/      #   mmap tabanlı disk tamponu
│   ├── cold-starter/      #   Soğuk başlatma / geri yakalama
│   └── data/              #   MERKEZİ VERİ KAYDI (DB, WAL)
│
├── services-engine/       # Aktif servisler
│   ├── alert-service/     #   Sesli koşul uyarıları
│   ├── detect-ms/         #   Market Structure Multi-Protocol (:3002)
│   ├── ohlcv-engine/      #   Klines istemcisi (kütüphane + API)
│   ├── paper-service/     #   Paper trading REST API (:8080)
│   └── price-feed/        #   Fiyat akışı daemon'u (:3004)
│
├── strategies-engine/     # Stratejiler
│   ├── breakout-strategy/ #   Kırılım stratejisi (sinyal üretici)
│   └── trait_def.rs       #   Strateji trait tanımları (Signal, FillReport)
│
├── execution-engine/      # Emir yürütme (PAPER / LIVE) — kütüphane
├── risk-engine/           # Risk çekirdeği (pre-trade kapısı, muhasebe, VaR) + risk-worker daemon
├── risk.toml              # Risk limitleri (hot-reload)
├── unused_services/       # Deaktive edilmiş servisler (arşiv)
├── tests/                 # Kök test klasörü (tüm testler burada yapılır)
├── docs/                  # Dokümantasyon + akış diyagramları
└── target/                # Build çıktısı
```

---

## 🧱 Katmanlı Mimari

| Katman | Klasör | Görev |
|---|---|---|
| **0 — Sözleşmeler** | `cycle-engine/contracts` | `events.rs`, `wire.rs` (tipli binary codec) |
| **1 — Transport (IPC)** | `cycle-engine/transport` | `/dev/shm` GenerationalRingBuffer + OrderRingBuffer (torn-read korumalı) |
| **2 — Çekirdek Motor** | `cycle-engine/core` | simdjson parser, validator, TitaniumOrchestrator (spin-loop), RiskEngine, LOB sim, TscTimer (RDTSC) |
| **2 — Açılış Ekranı** | `cycle-engine/splash` | FIGlet ASCII animasyonu (CYCLE FINANCE, harf harf) |
| **3 — Veri** | `data-engine` | SQLite yazımı, soğuk depolama, WAL |
| **4 — Analiz** | `services-engine/detect-ms` | 7 katmanlı piyasa yapısı analizi |
| **5 — Strateji** | `strategies-engine` | Kırılım sinyali üretimi |
| **6 — Yürütme** | `execution-engine` + `paper-service` | Emir yürütme (paper) |
| **Ops** | `additional-services` | Ortam, betikler, k8s, TLA+ |

### Veri Akışı

```
Binance WS (adapter) → flume queue → EventParser (simdjson)
  → DataValidator (stale ≤ 200ms, crossed book, circuit breaker)
  → wire::encode → GenerationalRingBuffer (/dev/shm/cycle_finance_ring)
  → SQLite batch writer (data-engine/data/market_data.db)

price-feed (:3004) → /dev/shm/cycle_finance_pricefeed → breakout-strategy
detect-ms (:3002)  ← BinanceClient (ohlcv-engine)
breakout-strategy  → SINYAL (sembol + yön)
```

---

## 🚀 Aktif Servisler

| Servis | Port | Görev |
|---|---|---|
| **core** (DATA modu) | — | Binance WS → parse → ring → DB |
| **detect-ms** | `:3002` | 7 katmanlı piyasa yapısı analizi (pivot, trend, seviye, likidite, FVG, naratif) |
| **price-feed** | `:3004` | WS → EventParser → kendi ring'i + REST `/api/lastprice/{symbol}` |
| **paper-service** | `:8080` | REST API, JWT auth, actor + event store (sled WAL), pozisyon/PnL |
| **alert-service** | — | `alerts.toml` koşullarına göre sesli uyarı |
| **breakout-strategy** | — | Kırılım sinyali üretici (emir açmaz) |
| **ohlcv-engine** | `:3000` | Klines istemcisi (kütüphane + `cli`/`server` bin) |
| **calc-ind** | `:3007` | İndikatör hesaplama motoru (ferro_ta_core) + `/dev/shm` ring yayını |
| **executiond** | `:3010` | Canlı Binance Futures emir daemon'u (DRY_RUN varsayılan) |
| **exec-cli** | — | Execution yönetim CLI (emir/kaldıraç/margin/hedge) |
| **risk-worker** | `:3011` | Soğuk yol risk parametre üretici (korelasyon, VaR, konsantrasyon, 60s) |

### Aktif Workspace Üyeleri (19)

`contracts, transport, core, adapter, os-utils, cold-storage, cold-starter, execution-engine, risk-engine, strategies-engine, breakout-strategy, ohlcv-engine, calc-ind, detect-ms, paper-service, alert-service, price-feed, splash`

---

## 🗄️ Deaktive Edilmiş Servisler

Aşağıdaki servisler `unused_services/` klasörüne taşınmış ve workspace'ten çıkarılmıştır (derlenmez):

| Servis | Eski Görevi |
|---|---|
| `detect-liquidity` | EQH/EQL, FVG, sweep tespiti |
| `detect-pattern` | Formasyon taraması |
| `detect-trb` | Navier-Stokes / kavitasyon çözücü |
| `detect-wyckoff` | Wyckoff faz analizi |
| `scout-service` | Fırsat tarayıcı |

*Bu servisleri yeniden etkinleştirmek için: klasörü `services-engine/`'e geri taşıyın, `Cargo.toml`'a `members` olarak ekleyin ve path bağımlılıklarını güncelleyin.*

---

## 🎯 Kırılım Stratejisi (Breakout)

`strategies-engine/breakout-strategy` — **sinyal üretici** (emir açmaz).

### Algoritma

```
detect-ms raporundan:
  ats > 0 (yukarı trend) → en yüksek skorlu SH (direnç) seviyesini seç
                           └─ fiyat > SH  →  BUY sinyali
  ats < 0 (aşağı trend) → en yüksek skorlu SL (destek) seviyesini seç
                           └─ fiyat < SL  →  SELL sinyali
  ats = 0 → nötr, sinyal yok
```

### Fiyat kaynağı (öncelik sırası)

1. `price-feed` ring'i (`/dev/shm/cycle_finance_pricefeed`) — event-by-event
2. `price-feed` REST `:3004`
3. `detect-ms` `current_price`

### Çıktı

```
📡 SİNYAL → Sembol: HEIUSDT | Yön: BUY (fiyat: ring) | Fiyat=0.2135 ATS=2.1 Trend=... Confluence=%
```

### Konfigürasyon (env)

| Değişken | Varsayılan | Açıklama |
|---|---|---|
| `BREAKOUT_SYMBOL` | `HEIUSDT` | Analiz edilecek sembol |
| `BREAKOUT_INTERVAL` | `1m` | Kline intervali |
| `BREAKOUT_LIMIT` | `100` | Analizdeki mum sayısı |
| `BREAKOUT_WAIT_SEC` | `check_every×60` | Değerlendirme aralığı (sn) |
| `BREAKOUT_CHECK_EVERY` | `20` | Bekleme = pencere×60 sn |

Dinamik bekleme: `/tmp/breakout_wait_sec.txt` dosyasına saniye yazarak çalışan stratejinin beklemesini değiştirebilirsiniz.

---

## 🛡️ Execution Engine (Canlı)

Kurumsal Binance USDT-M Futures emir yürütme katmanı (`execution-engine` + `executiond` daemon + `exec-cli`).

### Mimari Özeti

```
strateji / REST API ──► ExecutionActor (tek-yazıcı) ──► BinanceClient (REST, imzalı)
      ▲                          │                              ▲
      │                          │ UserDataEvent (gzip WS)      │ periyodik uzlaştırma
UserDataStream ◄── listenKey ────┴───────────────────────────────┘
      │
      ▼
AccountSnapshot (Arc<RwLock>) ──► REST API (:3010) / stratejiler (okuma)
```

- **REST + WS hibrit**: emir/kontrol REST, anlık hesap/pozisyon/emir güncellemeleri user-data stream.
- **Tek-yazıcı**: tüm yazma işlemleri `ExecutionActor` task'ından geçer.
- **Borsa doğrudur**: state, user-data deltalarının snapshot'a işlenmesidir; periyodik uzlaştırma sapmayı yakalar.
- **Idempotency**: her emir benzersiz `newClientOrderId` taşır; tekrar istek aynı yanıtı döner.
- **Pre-trade doğrulama**: sembol filtreleri (PRICE_FILTER, LOT_SIZE, MIN_NOTIONAL, MAX_POSITION), precizyon, pozisyon modu tutarlılığı, notional limit.
- **8 emir tipi**: `LIMIT, MARKET, STOP, STOP_MARKET, TAKE_PROFIT, TAKE_PROFIT_MARKET, TRAILING_STOP_MARKET, LIMIT_MAKER` + batch (≤5) + modify (PUT /fapi/v1/order).

### Kontrol Edilen Hesap Özellikleri

| Alan | Endpoint / İşlem |
|---|---|
| Emir | place, batchOrders, query, cancel, cancelAll, modify, workingType (MARK/CONTRACT), priceProtect, reduceOnly, closePosition |
| Pozisyon | positionRisk, positionMargin (+history), ADL quantile, forceOrders, leverageBracket |
| Hesap | /fapi/v3/account, balance, income (FUNDING_FEE dahil), commissionRate, apiTradingStatus |
| Yapılandırma | leverage set, marginType (ISOLATED/CROSSED), positionSide/dual (hedge modu), multiAssetsMargin, listenKey |
| Anlık | User Data Stream: ACCOUNT_UPDATE, ORDER_TRADE_UPDATE, ACCOUNT_CONFIG_UPDATE, MARGIN_CALL, listenKeyExpired |

### Güvenlik (Canlı Mod)

| Önlem | Davranış |
|---|---|
| `EXEC_DRY_RUN=true` (varsayılan) | Emir doğrulanır, imzalanır, loglanır ama **gönderilmez** |
| Kill switch | `/tmp/exec_kill_switch` dosyası + `PUT /api/v1/risk/kill-switch`; açıkken tüm yazma reddedilir |
| İlk eşitleme | Borsa ile eşitlenmeden **hiçbir emir kabul edilmez** |
| `EXEC_MAX_NOTIONAL` | Tek emir USDT üst sınırı (varsayılan 1000 USDT) |
| `EXEC_MAX_ORDERS_PER_MIN` | Kayan pencere emir limiti |
| `EXEC_SYMBOL_BLOCKLIST` | Sembol bazlı engelleme |
| Cevap okunmadan emir | Asla "başarılı" sayılmaz; ACK beklenir, zaman aşımında sorgu ile uzlaştırılır |

### Çalıştırma

```bash
# DRY_RUN ile güvenli başlangıç
EXEC_MODE=LIVE EXEC_DRY_RUN=true ./target/debug/executiond

# Gerçek emir gönderimi (AÇIK ONAY — dikkat!)
EXEC_DRY_RUN=false ./target/debug/executiond --host 127.0.0.1 --port 3010
```

### REST API (:3010, JWT)

```
POST   /api/v1/auth/login · /api/v1/auth/refresh
POST   /api/v1/orders          → emir gönder (idempotent)
POST   /api/v1/orders/batch    → toplu emir (≤5)
GET    /api/v1/orders?symbol=  → açık emirler (snapshot)
GET    /api/v1/orders/query    → REST sorgu (orderId / clientOrderId)
POST   /api/v1/orders/cancel   → iptal
DELETE /api/v1/orders/{cid}    → clientOrderId ile iptal
DELETE /api/v1/orders/open?symbol= → sembolün tümünü iptal
PUT    /api/v1/orders/{cid}    → modify (fiyat/miktar/stop)
GET    /api/v1/account | /positions | /balances | /income | /funding
GET    /api/v1/force-orders | /commission-rate/{s} | /adl/{s} | /trading-status
GET    /api/v1/exchange-info/{s}
PUT    /api/v1/symbols/{s}/leverage · /margin-type · POST /symbols/{s}/margin
PUT    /api/v1/position-mode · /api/v1/multi-assets
GET    /api/v1/risk · PUT /api/v1/risk/kill-switch · GET /api/v1/mode · /healthz
GET    /metrics            → Prometheus metrikleri
```

Örnek emir:

```bash
TOKEN=$(curl -s -X POST http://127.0.0.1:3010/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"changeme123"}' | jq -r .access_token)

curl -s -X POST http://127.0.0.1:3010/api/v1/orders \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"client_order_id":"strat-1","symbol":"BTCUSDT","side":"BUY","type":"MARKET",
       "quantity":"0.001","position_side":"BOTH"}'
```

### CLI (`exec-cli`)

```bash
./target/debug/exec-cli account                    # bakiye + pozisyon + açık emir
./target/debug/exec-cli order BTCUSDT BUY MARKET 0.001 --reduce-only
./target/debug/exec-cli orders BTCUSDT
./target/debug/exec-cli leverage BTCUSDT 10
./target/debug/exec-cli margin-type BTCUSDT ISOLATED
./target/debug/exec-cli hedge true
./target/debug/exec-cli funding BTCUSDT
./target/debug/exec-cli exchange-info BTCUSDT
```

### Ortam Değişkenleri (`EXEC_`)

| Değişken | Varsayılan | Açıklama |
|---|---|---|
| `EXEC_MODE` | `LIVE` | `PAPER` ise `paper-service` kullanılır |
| `EXEC_DRY_RUN` | `true` | `false` = gerçek emir (açık onay) |
| `EXEC_BASE_URL` | `https://fapi.binance.com` | Testnet: `https://testnet.binancefuture.com` |
| `EXEC_WS_URL` | `wss://fstream.binance.com` | Testnet: `wss://stream.binancefuture.com` |
| `EXEC_MAX_NOTIONAL` | `1000` | Tek emir USDT üst sınırı (0 = sınırsız) |
| `EXEC_MAX_ORDERS_PER_MIN` | `60` | Dakikada emir limiti (0 = sınırsız) |
| `EXEC_SYMBOL_BLOCKLIST` | — | Virgülle ayrılmış semboller |
| `EXEC_KILL_SWITCH_PATH` | `/tmp/exec_kill_switch` | Kill switch dosyası |
| `EXEC_API_ADDR` | `127.0.0.1:3010` | REST bind adresi |
| `EXEC_JWT_SECRET` | dev değeri | JWT secret (üretimde değiştirin) |
| `EXEC_ADMIN_USER` / `EXEC_ADMIN_PASS` | `admin` / `changeme123` | REST giriş |

### Testler

```bash
cargo test -p execution-engine            # birim + sahte-Binance entegrasyon
cargo test -p execution-engine --test mock_binance
```

---

## 💾 Veri Kaydı

Tüm veriler merkezi olarak `data-engine/data/` altında tutulur:

```
data-engine/data/
├── market_data.db     # Ana tick/OHLCV SQLite (WAL modu)
├── paper_live.db      # Paper-service çalışma veritabanı
└── paper_wal/         # Paper event store (sled)
```

Herhangi bir servis veri yazarsa buraya yazar. `.gitignore` ile git dışı tutulur.

---

## 🔧 Kurulum

**Gereksinimler:** Rust (stable), tmux, curl, jq

```bash
# Tüm workspace'i derle
cargo build --workspace

# Kurulum paketi oluştur
./install.sh

# Sadece derle
./install.sh --only-build
```

---

## ▶️ Çalıştırma

### 1. Veri terminali (DATA)

```bash
cd PROJE && RUN_MODE=DATA ./target/debug/core
```

> Açılış ekranı (`cycle-engine/splash`) `cycle_tmux.sh` başlatıcısında gösterilir; `core`'u doğrudan çalıştırırken splash oynamaz.

### 2. Servisleri ayrı ayrı başlat

```bash
./target/debug/detect-ms        # :3002  analiz motoru
./target/debug/price-feed       # :3004  fiyat akışı
./target/debug/paper-service     # :8080  paper API
./target/debug/alert-service     # sesli uyarı
./target/debug/breakout-strategy # kırılım sinyali
./target/debug/calc-ind          # :3007  indikatör hesaplama motoru
```

### 2b. calc-ind — indikatör hesaplama (ferro_ta_core)

`calc-ind` servisi istek üzerine OHLCV'yi `ohlcv-engine`'den çeker, `ferro_ta_core` ile indikatör hesaplar ve sonucu **binary olarak** `/dev/shm/cycle_finance_calc` ring'ine yayınlar. İstek atan servis `calc_ind::client` ile sonucu ring'den okur.

```bash
# İstek (HTTP): symbol + interval + start/end + indikatör + parametreler
curl -X POST http://127.0.0.1:3007/api/calc \
  -H "Content-Type: application/json" \
  -d '{"symbol":"BTCUSDT","interval":"1h","start_ms":null,"end_ms":null,
       "indicator":"rsi","params":{"period":14}}'
# → {"count":1000,"request_id":1,"series":["rsi"],"status":"success"}
```

**Rust tüketici API'si:**

```rust
use calc_ind::{IndRequest, client};
use std::collections::HashMap;

let mut params = HashMap::new();
params.insert("period".to_string(), 14.0);
let req = IndRequest::new("BTCUSDT", "1h", None, None, "rsi").with_params(params);
let id = client::request_default(&req).await?;          // request_id
let res = client::read_result(id, 5, 200);               // ring'den oku (retry)
// res.series["rsi"] → Vec<Option<f64>> (None = warm-up NaN)
```

Örnek: `cargo run -p calc-ind --example read_ring`

**Desteklenen indikatörler:** `sma, ema, wma, macd, bbands, rsi, stoch, momentum, roc, stddev, atr, vwap, volume` — parametreler istekte `params` haritasıyla verilir.

### 3. tmux ile tüm ortamı başlat (önerilen)

```bash
./additional-services/scripts/cycle_tmux.sh          # başlat + bağlan
./additional-services/scripts/cycle_tmux.sh kill     # durdur
./additional-services/scripts/cycle_tmux.sh status   # durum
```

> `cycle_tmux.sh` çalıştırıldığında önce **CYCLE FINANCE** FIGlet açılış ekranı tek terminalde (harf harf animasyon) gösterilir, ardından her servis tek bir sekmede olacak şekilde tmux session'ı açılır. Açılış ekranı `cycle-engine/splash` crate'i (`cycle-splash` binary) ile sağlanır; hız `show_splash_with(text, ms)` ile özelleştirilebilir.

### 4. Ortam fonksiyonları

```bash
source additional-services/scripts/cycle_env.sh
help-cycle   # komut listesi
```

Yaygın komutlar: `data-live`, `detect-ms-start`, `calc-ind-start`, `breakout-start`, `paper-start`, `alert-start`, `listener-start`, `risk-start`, `risk-worker-start`, `monitor-start`.

---

## 🖥️ tmux Terminal Düzeni

| Pencere | İçerik |
|---|---|
| 0 — STRATEGY | Strateji terminali (PyO3) |
| 1 — LISTENER | Anlık metrik analizi |
| 2 — RISK | Risk analizi (--watch) |
| 3 — SHELL | Ortam fonksiyonları (help-cycle) |
| 4 — DATA | Binance WS veri terminali |
| 5 — ALERT | Sesli uyarı servisi |
| 6 — PAPER | Paper REST API |
| 7 — Monitor | CPU/RAM/GPU izleme |
| 8 — DETECT-MS | Analiz motoru |
| 9 — BREAKOUT | Kırılım stratejisi |
| 10 — STREAM-OHLCV | Canlı OHLCV mum akışı |
| 11 — CALC-IND | İndikatör hesaplama motoru |

---

## 🧪 Testler

Tüm testler kök `tests/` klasöründe yapılır (`Cargo.toml` workspace `exclude` listesindedir).

```bash
cargo test --workspace
```

> Not: Birim testler crate içlerinde; entegrasyon/e2e testleri `tests/` altına eklenmelidir.

---

## 🚦 CI / K8s

- **CI** (`.github/workflows/test-suite.yml`): `cargo test --release`, `cargo check --all-features`, `cargo bench` (WCET 750µs), kaos testleri (Chaos Mesh, erişilebilirse)
- **K8s** (`additional-services/k8s/`): deployment + chaos senaryoları (DNS/network partition/NTP drift)
- **TLA+** (`additional-services/formal_verification/`): `CycleFinance.tla` + `.cfg`

---

## ⚙️ Teknik Detaylar

### Gecikme optimizasyonları

- **`/dev/shm` ring buffer**: prosesler arası sıfır-kopya IPC, torn-read korumalı (generational check)
- **RT zamanlayıcı**: `os-utils::set_rt_thread_priority` → `SCHED_FIFO` (prio 99)
- **CPU pin**: `hal::cpu::pin_to_core` — tick thread'i belirli çekirdeğe sabitlenir
- **Pre-fault bellek**: `hal::memory::allocate_huge_buffer` — çalışma anında page fault yok
- **TSC timer**: `timer/tsc.rs` — `RDTSC` tabanlı yüksek çözünürlüklü zamanlama
- **simdjson**: zero-copy JSON parsing
- **SQLite WAL + batch**: 10k yazma/1sn batching

### Risk Yönetimi

- `risk-engine` — ortak risk çekirdeği (tek doğruluk kaynağı):
  - `RiskEngine::evaluate` — 13 adımlı pre-trade kural zinciri (kill switch,
    circuit breaker, blocklist, rate-limit, notional, leverage, pozisyon,
    exposure, HHI konsantrasyon, marj, günlük kayıp, drawdown, fail-closed mark)
  - `Portfolio`/`Position` — coin-bazlı muhasebe, fill/PnL, likidasyon fiyatı
  - `RiskWorker` daemon (`risk-worker`, :3011) — korelasyon → Tikhonov → EWMA vol
    → parametrik VaR → önerilen limitler (60s)
  - `Seqlock` `RiskCache`, JSONL `AuditLog`, `KillSwitch` (dosya + bayrak)
- `risk.toml` — tüm limitler; hot-reload (mtime izleme)
- Execution entegrasyonu: `RiskChecks` bağdaştırıcısı → `RiskEngine`, fill
  geri beslemesi (`ORDER_TRADE_UPDATE` → `on_fill`), resync senkronizasyonu
- Circuit breaker: 100+ hatalı/sn → akışı durdur; 3+ ardışık risk reddi → otomatik kill switch

### Güvenlik

- `execution-engine/src/signer.rs` — Binance imzalama (HMAC-SHA256) altyapısı
- `paper-service` — JWT auth (argon2 şifre hash)
- `adapter/vault` — HashiCorp Vault entegrasyon taslağı (anahtar rotasyonu)
- `.env` repodan hariç tutulur (`BINANCE_API_KEY`, `BINANCE_SECRET_KEY`)

---

## 📜 Lisans / Not

Bu proje özel bir araştırma/geliştirme projesidir. Canlı para ile kullanım risk içerir; sistem yalnızca paper-trading için yapılandırılmıştır.

---

*Doküman oluşturma tarihi: 2026-08-08 · Ayrıntılı mimari doküman: `docs/PROJE_DOKUMANTASYONU.md` · Risk motoru mimarisi: `docs/RISK_ENGINE_MIMARISI.md` · Akış diyagramları: `docs/flowcharts/`*
