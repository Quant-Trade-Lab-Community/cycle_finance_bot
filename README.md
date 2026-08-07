# 🏛️ Cycle Finance — Kurumsal HFT Sistemi

**Proje Türü:** Kurumsal Düzey Yüksek Frekanslı Kripto Ticaret (HFT) Motoru  
**Dil:** Rust (çekirdek) + Python (strateji/listener katmanları)  
**Hedef Borsa:** Binance (Spot & Futures)  
**Mimari:** Bare-Metal, Zero-Copy IPC, Multi-Terminal, Actor Model, Event Sourcing

---

## 📐 Genel Mimari

Sistem, bağımsız süreçler (terminaller) halinde çalışan bir **mikro-çekirdek mimarisi** kullanır. Terminaller arasındaki iletişim `/dev/shm` üzerinde memory-mapped, lock-free **Generational Ring Buffer** ile sağlanır — serileştirme veya kopyalama yoktur.

```mermaid
graph TB
    subgraph "DATA Terminali (core)"
        WS["Binance WebSocket<br/>(Futures & Spot)"] --> TP["Tick Parser<br/>(simd-json)"]
        TP --> VAL["DataValidator"]
        VAL --> TR["Tick Ring Buffer<br/>/dev/shm/demir_yumruk_ring<br/>160,000 byte"]
        VAL --> DB_RAW["SQLite DB Writer<br/>(flume channel)"]
    end

    subgraph "BACKTEST Terminali (core)"
        CSV["CSV Dosyası<br/>(timestamp,price,qty)"] --> BT["Backtester<br/>(10ms aralık)"]
        BT --> TR
    end

    subgraph "STRATEGY Terminali (core)"
        TR --> |"Zero-Copy Read"| OHLCV["OHLCV Candle Builder<br/>(1dk mumlar)"]
        OHLCV --> DET["5 Algılayıcı Motoru"]
        DET --> PY["Python Strateji Süreci<br/>(heiusdt_breakout.py)"]
        PY --> OR["Order Ring Buffer<br/>/dev/shm/demir_yumruk_orders<br/>10,000 byte"]
    end

    subgraph "PAPER-SERVICE (bağımsız binary)"
        OR --> |"Zero-Copy Read"| BRIDGE["Ring Bridge<br/>(spawn_ring_bridge)"]
        TR --> |"Fiyat Beslemesi"| BRIDGE
        BRIDGE --> ACTOR["PaperEngineActor<br/>(Actor Model)"]
        ACTOR --> WAL["Sled WAL<br/>(paper_wal/)"]
        ACTOR --> PG["PostgreSQL<br/>(--features full)"]
        ACTOR --> SNAP["Snapshot<br/>(Arc<RwLock>)"]
        SNAP --> API["REST API<br/>(axum 0.8)"]
        SNAP --> METRICS["Prometheus /metrics"]
    end

    subgraph "ALERT-SERVICE (bağımsız binary)"
        TR --> |"Ring Source"| AE["AlertEngine"]
        WS2["Binance WS<br/>(bağımsız)"] --> AE
        AE --> AUDIO["Sesli Uyarı<br/>(spd-say / paplay)"]
    end

    subgraph "CORRELATION Terminali (core)"
        TR --> CA["Korelasyon Analizi<br/>(Anomali + Kümeleme)"]
    end

    subgraph "PRICE-FEED (bağımsız binary :3004)"
        WS3["Binance WS<br/>markPrice + bookTicker"] --> PF["PriceFeed<br/>HTTP API /api/lastprice"]
    end

    subgraph "DETECT-MS (bağımsız binary :3002)"
        PF --> |"fiyat beslemesi"| MS["MSMP 2.0 REST API<br/>7 katmanlı analiz"]
    end
```

---

## 🧩 Modül Haritası (16 Crate)

| # | Crate | Rol | Önemli Dosyalar |
|---|-------|-----|-----------------|
| 1 | [core](./core) | Ana orkestratör — 5 terminal modu | `main.rs`, `engine/`, `cli/`, `memory/` |
| 2 | [adapter](./adapter) | Binance WS/REST bağdaştırıcısı | `src/binance.rs` |
| 3 | [os-utils](./os-utils) | Lock-free Ring Buffer (`/dev/shm`) | `src/lib.rs` |
| 4 | [ohlcv-engine](./ohlcv-engine) | Gerçek zamanlı OHLCV mum oluşturucu | `src/lib.rs`, `src/client.rs` |
| 5 | [execution-engine](./execution-engine) | Emir yönetimi + Paper Engine | `src/lib.rs`, `src/paper/` |
| 6 | [paper-service](./paper-service) | Bağımsız Paper Trading Servisi | `src/api.rs`, `src/events.rs`, `src/bridge.rs` |
| 7 | [risk-worker](./risk-worker) | Risk matrisi & FinOps optimizasyonu | `src/matrix.rs`, `src/finops.rs` |
| 8 | [cold-starter](./cold-starter) | Soğuk başlangıç (geçmiş kline verisi) | `src/lib.rs` |
| 9 | [cold-storage](./cold-storage) | SQLite işlem geçmişi | `src/lib.rs` |
| 10 | [detect-sr](./detect-sr) | Destek/Direnç + FVG algılama | `src/lib.rs` |
| 11 | [detect-trend](./detect-trend) | Trend algılama (EMA + ADX) | `src/lib.rs` |
| 12 | [detect-ms](./detect-ms) | MSMP 2.0 REST API (7 katmanlı analiz, :3002) | `src/main.rs`, `src/narrative.rs` |
| 13 | [detect-liquidity](./detect-liquidity) | Likidite havuzu tespiti | `src/lib.rs` |
| 14 | [detect-pattern](./detect-pattern) | Mum çubuğu patern algılama | `src/lib.rs` |
| 15 | [alert-service](./alert-service) | Sesli fiyat uyarı servisi | `src/engine.rs`, `src/audio.rs`, `src/source.rs` |
| 16 | [price-feed](./price-feed) | Anlık last/mark/index price daemon (:3004) | `src/main.rs` |

---

## 🖥️ Terminal Modları (`RUN_MODE`)

Tüm terminaller tek `core` binary'si üzerinden çalışır; `RUN_MODE` ortam değişkeniyle seçilir.

### 1. DATA Terminali (`RUN_MODE=DATA`)

> Canlı Binance piyasa verisini ring buffer'a besleyen ana motor.

- **Binance WebSocket** bağlantısı (Futures & Spot stream desteği)
- **simd-json** ile ultra-hızlı sıfır-kopya JSON ayrıştırma
- Abone olunan stream'ler: `@trade` (işlemler) + `@depth20@100ms` (20 seviyeli emir defteri)
- Parser desteklenen tipler: `Trade`, `OrderBook`, `Liquidation`, `FundingRate`, `BookTicker`, `OpenInterest`
- **DataValidator:** Fiyat > 0, Miktar > 0, Zaman damgası ± 60 sn kontrolü
- Ring buffer'a yazma + paralel SQLite kayıt (`flume` kanalı ile)
- RT thread önceliği: `set_rt_thread_priority(99)`

**CLI Komutları:** `status`, `quit`

---

### 2. STRATEGY Terminali (`RUN_MODE=STRATEGY`)

> Sistemin beyni — HEIUSDT kırılım stratejisini Python süreci olarak çalıştırır.

- `detect-ms` REST API'sinden (:3002) seviye/yapı analizi alır
- `paper-service` API'sine (:8080) kırılım sinyaline göre market emri açar
- Rust tarafı (`strategy_cli.rs`) Python sürecini spawn eder ve izler
- Komutlar: `status`, `restart`, `exit`

> **Not:** Eski PyO3 köprüsü kaldırıldı — strateji mantığı artık bağımsız Python sürecinde çalışır (bkz. §HEIUSDT Kırılım Stratejisi).

**CLI Komutları:** `status`, `restart`, `quit`

---

### 3. PAPER Terminali (`RUN_MODE=PAPER`)

> `core` içindeki yerleşik hafif paper trading CLI'ı.

- Order ring buffer'dan emir okuma
- Tick ring buffer'dan canlı fiyat beslemesi
- Temel portföy ve risk yönetimi

> **Not:** Üretim ortamı için ayrı `paper-service` binary'si kullanın (bkz. §Paper Service).

**CLI Komutları:** `status` / `balance`, `positions`, `risk`, `history`, `quit`

---

### 4. BACKTEST Terminali (`RUN_MODE=BACKTEST`)

> Geriye dönük simülasyon motoru.

- CSV dosyasından (`timestamp,price,qty`) okuma
- JSON'a dönüştürüp ring buffer'a yazma (10 ms aralıkla)
- Strategy terminali verinin kaynağını bilmez — **%100 aynı kod çalışır**

```bash
RUN_MODE=BACKTEST CSV_PATH="./test_data.csv" cargo run --release -p core
```

---

### 5. CORRELATION Terminali (`RUN_MODE=CORRELATION`)

> Anomali tespiti ve piyasa korelasyon analizi.

- Ring buffer'dan canlı tick okuma (HEIUSDT odaklı)
- Kayan pencere (varsayılan: `WINDOW_SEC=10`) ile istatistiksel analiz
- **Anomali Tespiti:** Flat (düz seyir), Breakout (kırılım) senaryoları
- **Kümeleme & Öz-doğrulama:** Anomali sonuç doğrulaması (`TRACK_SEC`)
- Asenkron kuyruk tabanlı mimari (v5.0)

```bash
RUN_MODE=CORRELATION WINDOW_SEC=15 TRACK_SEC=15 cargo run --release -p core
```

---

## 🛡️ Paper Service (`paper-service`)

Bağımsız bir binary olarak çalışan üretim kalitesi paper trading servisi.

### Mimari

```
PaperEngineActor (Actor Model)
    ├── HybridOrderBook    (PRICE_ONLY / L2_SWEEP / LINEAR_IMPACT)
    ├── AccountState       (bakiye, equity, realized PnL)
    ├── PositionManager    (long/short pozisyonlar, likidasyon fiyatı)
    ├── RiskManager        (drawdown, günlük kayıp, kaldıraç limitleri)
    └── DomainEvent → Sled WAL → PostgreSQL (--features full)
```

### Event Sourcing Katmanı

| Katman | Açıklama |
|--------|----------|
| **Sled WAL** | Her event önce embedded disk store'a yazılır (`paper_wal/`) |
| **PostgreSQL** | `--features full` ile opsiyonel event store senkronizasyonu |
| **Replay** | Çökme sonrasında event'ler yeniden oynatılarak state restore edilir |
| **Idempotency** | `client_order_id` → önbellek, çift emir gönderimini önler |

### REST API Endpoint'leri (`http://127.0.0.1:8080`)

| Metod | Endpoint | Açıklama |
|-------|----------|----------|
| `POST` | `/api/v1/auth/login` | JWT token al |
| `POST` | `/api/v1/auth/refresh` | Token yenile |
| `GET` | `/api/v1/system/health` | Sağlık kontrolü |
| `POST` | `/api/v1/order` | Emir gönder (idempotent) |
| `GET` | `/api/v1/orders` | Açık emirleri listele |
| `GET` | `/api/v1/account/balance` | Bakiye ve equity |
| `GET` | `/api/v1/account/positions` | Açık pozisyonlar |
| `GET` | `/api/v1/account/trade-history` | İşlem geçmişi |
| `GET` | `/api/v1/risk/liquidation-price/{symbol}` | Likidasyon fiyatı |
| `GET` | `/metrics` | Prometheus metrikleri |

### Prometheus Metrikleri (`GET /metrics`)

| Metrik | Tip | Açıklama |
|--------|-----|----------|
| `paper_order_place_total` | counter | Toplam emir gönderimi |
| `paper_order_place_failure_total` | counter | Reddedilen emirler |
| `paper_liquidation_events_total` | counter | Likidasyon sayısı |
| `paper_funding_events_total` | counter | Funding uygulama sayısı |
| `paper_fills_total` | counter | Gerçekleşen dolum sayısı |
| `paper_account_balance_usdt` | gauge | Anlık hesap bakiyesi (USDT) |

### Matching Modları

| Mod | Açıklama |
|-----|----------|
| `PRICE_ONLY` | Order book'suz, gerçek fiyat verisiyle dolum (varsayılan) |
| `L2_SWEEP` | L2 derinlik simülasyonu |
| `LINEAR_IMPACT` | Lineer piyasa etkisi modeli |

### Paper Engine Risk Parametreleri

| Parametre | Env Değişkeni | Varsayılan |
|-----------|---------------|------------|
| Başlangıç bakiyesi | `PAPER_INITIAL_USDT` | 100,000 USDT |
| Maker komisyon | `PAPER_MAKER_FEE` | %0.02 (2 bps) |
| Taker komisyon | `PAPER_TAKER_FEE` | %0.05 (5 bps) |
| Baz gecikme | `PAPER_BASE_LATENCY_MS` | 5 ms |
| Gecikme jitter | `PAPER_LATENCY_JITTER_MS` | 2 ms |
| Maks. pozisyon | `PAPER_MAX_POSITION_QTY` | 10.0 |
| Maks. kaldıraç | `PAPER_MAX_LEVERAGE` | 20x |
| Maks. drawdown | `PAPER_MAX_DRAWDOWN_PCT` | %5 |
| Maks. günlük kayıp | `PAPER_MAX_DAILY_LOSS` | 1,000 USDT |

### Paper CLI (`paper-cli`)

```bash
# Durum
./target/debug/paper_cli --api http://127.0.0.1:8080 --user admin --password changeme123 status

# Emir gönder
./target/debug/paper_cli --api http://127.0.0.1:8080 --user admin --password changeme123 \
    order --symbol BTCUSDT --side BUY --order-type MARKET --qty 0.001
```

---

## 💹 Price Feed (`price-feed` / `scripts/price_feed.py`)

> Tüm katmanlara anlık last/mark/index fiyatı sunan daemon. Rust (WS) ve Python (REST polling) iki gerçeklemesi mevcuttur; ikisi de aynı HTTP API ve JSON çıktısını paylaşır.

**Rust gerçeklemesi (`cargo run -p price-feed`):** Binance Futures WS'ten `markPrice@1s` (mark + index) ve `bookTicker@1s` (best bid/ask) aboneliği yapar, `simd_json` ile ayrıştırıp kendi ring buffer'ına yazar (`/dev/shm/demir_yumruk_pricefeed`).

**Python gerçeklemesi (`python3 scripts/price_feed.py`):** Binance REST `ticker/price`'i periyodik çeker (`PRICE_FEED_REFRESH` sn), sembolleri `alerts.toml` + `HEIUSDT`'tan türetir.

| Uç Nokta | Açıklama |
|----------|----------|
| `GET /api/lastprice` | Tüm semboller `{last, mark, index, bid, ask}` |
| `GET /api/lastprice/{SYMBOL}` | Tek sembol |
| `GET /health` | Sağlık + anlık fiyatlar |
| `/tmp/price_feed.json` | Her güncellemede yazılan JSON dosyası |

**Sembol kaynağı:** `PRICE_FEED_SYMBOLS` env'i (virgülle ayrılmış) veya `alerts.toml` sembolleri + `HEIUSDT`. Port: `PRICE_FEED_PORT` (varsayılan 3004).

---

## 📈 Detect-MS — MSMP 2.0 REST API (`:3002`)

> Kurumsal matematiksel çerçeve: 7 katmanlı piyasa yapısı analiz motoru. `BinanceClient` ile geçmiş kline çeker, üç zaman penceresi (Core/Amplified/Acute) üzerinde analiz yapar ve birleşik rapor döndürür.

| Katman | Analiz |
|--------|--------|
| 1 | Session-Based Zaman Pencereleri |
| 2 | Dinamik Pivot (ATR × 0.25, Tip A/B, Likidite Bölgeleri) |
| 3 | Trend Yapısı (Log-Regresyon, R², Hurst Üssü) |
| 4 | Stratejik Seviye Envanteri (Üssel Çürüme, BO Onayı) |
| 5 | Likidite Pool (VWAP, Volume Profile, BSL/SSL) |
| 6 | Dengesizlik (FVG + Cumulative Delta) |
| 7 | Bütünsel Naratif (ATS, Vakum Bölgesi, Confluence Index) |

```bash
cargo run -p detect-ms
curl "http://127.0.0.1:3002/api/ms?symbol=BTCUSDT&interval=15m&limit=200"
```

**Rapor çıktısı:** `ats` (Ağırlıklı Trend Skoru), `hurst`, `r_squared`, `trend_label`, `confluence_index`, `levels` (priority_score'lu SH/SL seviyeleri), `vacuum_zone`.

---

## 🔔 Alert Service (`alert-service`)

Yapılandırma dosyasından okunan kurallara göre sesli fiyat uyarısı üreten bağımsız servis.

### Veri Kaynakları

| Kaynak | Açıklama |
|--------|----------|
| `ring` | DATA terminali tick ring buffer'ından okur |
| `binance` | Bağımsız Binance WebSocket bağlantısı açar |

### Uyarı Koşulları

| Koşul | Tetiklenme |
|-------|------------|
| `above` | Fiyat eşiği yukarı geçince |
| `below` | Fiyat eşiği aşağı geçince |
| `cross` | Fiyat her geçişte (her iki yön) |
| `touch` | Fiyat tolerans aralığına girince |

### Ses Çıktısı
- **Konuşma:** `spd-say` (text-to-speech, özel metin desteği)
- **Beep:** `paplay` (ses dosyası)

### Örnek Yapılandırma ([alerts.toml](./alerts.toml))

```toml
data_source = "ring"

[[alerts]]
symbol = "BTCUSDT"
condition = "above"
price = 64500
voice = "Bitcoin 64 bin 500 üzerine çıktı"
cooldown_sec = 30

[[alerts]]
symbol = "ETHUSDT"
condition = "cross"
price = 3200
cooldown_sec = 60
```

### CLI Komutları (stdin)
- `list` — Aktif uyarıları listele
- `add <SYMBOL> <above|below|cross|touch> <price> [metin]` — Uyarı ekle
- `quit` — Servisi kapat

---

## 🔍 Algılayıcı (Detector) Motoru — 5 Modül

### 1. Destek/Direnç Algılama (`detect-sr`)

| Özellik | Detay |
|---------|-------|
| **Pivot S/R** | Kayan pencere ile yerel en yüksek/düşük noktalar |
| **Fair Value Gap (FVG)** | 3 mumlu dengesizlik tespiti (Bullish/Bearish) |
| **Filtreleme** | %0.05 eşik ile benzer seviyeleri birleştirme |

**Çıktı:** `Vec<SrLevel>` + `Vec<Fvg>`

---

### 2. Trend Algılama (`detect-trend`)

| Gösterge | Parametre |
|----------|-----------|
| **Hızlı EMA** | Periyot: 9 |
| **Yavaş EMA** | Periyot: 21 |
| **ADX** | Periyot: 14, Eşik: 25.0 |

**Karar Mantığı:**
- `Bullish` → fast_ema > slow_ema VE adx ≥ 25
- `Bearish` → fast_ema < slow_ema VE adx ≥ 25
- `Sideways` → diğer tüm durumlar

---

### 3. Piyasa Yapısı Algılama (`detect-ms`)

- **Break of Structure (BOS):** Mevcut trendin yönünde kırılma
- **Change of Character (CHoCH):** Trend yönüne karşı kırılma (dönüş sinyali)
- **Order Block:** BOS öncesindeki son ters yönlü mum

**Çıktı:** `(Vec<MsBreak>, Vec<OrderBlock>)`

---

### 4. Likidite Algılama (`detect-liquidity`)

- **Eşit Yüksekler/Düşükler:** %0.02 toleransta eşleşen fiyat seviyeleri
- **Swing Likidite:** Swing high/low noktalarındaki likidite havuzları
- **Deduplikasyon:** %0.05 eşik ile benzer seviyeleri birleştirme

**Çıktı:** `Vec<LiquidityPool>` (BuySide / SellSide)

---

### 5. Mum Çubuğu Patern Algılama (`detect-pattern`)

| Patern | Koşul |
|--------|-------|
| **Bullish Engulfing** | Gövde önceki mumu tamamen yutuyor (yükseliş) |
| **Bearish Engulfing** | Gövde önceki mumu tamamen yutuyor (düşüş) |
| **Bullish Pin Bar** | Alt fitil ≥ 2× gövde, üst fitil ≤ 0.5× gövde |
| **Bearish Pin Bar** | Üst fitil ≥ 2× gövde, alt fitil ≤ 0.5× gövde |
| **Inside Bar** | Mum tamamen önceki mumun içinde |

---

## 🎯 HEIUSDT Kırılım Stratejisi ([heiusdt_breakout.py](./strategies/heiusdt_breakout.py))

> Kullanıcı türevli HEIUSDT odaklı alım-satım stratejisi. `detect-ms` analizi + `paper-service` emir yürütme.

```mermaid
sequenceDiagram
    participant MS as detect-ms REST (:3002)
    participant PY as heiusdt_breakout.py
    participant PS as paper-service (:8080)

    loop Her 20 pencere (1m)
        PY->>MS: GET /api/ms?symbol=HEIUSDT&interval=1m&limit=100
        MS-->>PY: MSMPReport (levels, ats, trend_label)
        PY->>PY: Kırılım koşulu kontrolü (SH/SL seviyesi)
        alt ATS > 0 ve fiyat > SH direnci
            PY->>PS: POST /api/v1/order → BUY
        else ATS < 0 ve fiyat < SL desteği
            PY->>PS: POST /api/v1/order → SELL
        end
    end
```

**Karar Mantığı:**
- Yapı trendi **yukarı** (`ats > 0`) ise en yüksek skorlu direnç (SH) bulunur; fiyat üzerinde kapatırsa → **BUY**
- Yapı trendi **aşağı** (`ats < 0`) ise en yüksek skorlu destek (SL) bulunur; fiyat altında kapatırsa → **SELL**
- Aynı sembolde açık pozisyon varsa yeni emir açılmaz
- `--dry-run` ile emirsiz analiz, `--once` ile tek seferlik çalıştırma

```bash
# Tek seferlik analiz
python3 strategies/heiusdt_breakout.py --once

# Analiz + kırılım simülasyonu (emir açmaz)
python3 strategies/heiusdt_breakout.py --once --dry-run

# Sürekli strateji (her 20 pencerede kontrol)
python3 strategies/heiusdt_breakout.py
```

**Env değişkenleri:** `HEIUSDT_SYMBOL`, `HEIUSDT_INTERVAL`, `HEIUSDT_LIMIT`, `HEIUSDT_CHECK_EVERY`, `HEIUSDT_QTY`

---

## 💾 Veri Katmanı

### Paylaşımlı Hafıza (Zero-Copy IPC)

| Ring Buffer | Yol | Kapasite | Amaç |
|------------|------|----------|------|
| Tick Ring | `/dev/shm/demir_yumruk_ring` | 160,000 byte | Piyasa verisi (Trade/BookTicker/FundingRate) |
| Order Ring | `/dev/shm/demir_yumruk_orders` | 10,000 byte | Strateji emirleri |
| PriceFeed Ring | `/dev/shm/demir_yumruk_pricefeed` | 20,000 slot | Price-feed daemon fiyatları |

**Slot yapısı:** `[8 byte generation] [4 byte data_len] [data bytes...]`

**Ring Buffer özellikleri:**
- Lock-free, atomic generation counter
- Generational (stale data tespiti)
- Kani Model Checker ile formel olarak doğrulanmış

### SQLite Veritabanı (cold-storage + paper engine)

```sql
-- İşlem geçmişi (cold-storage)
CREATE TABLE trades (
    id INTEGER PRIMARY KEY,
    symbol TEXT,
    side TEXT,
    entry_price REAL,
    exit_price REAL,
    qty REAL,
    pnl REAL,
    commission REAL,
    timestamp INTEGER
);

-- Funding rate kayıtları (mark + index fiyat)
CREATE TABLE funding_rates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    mark_price REAL NOT NULL,
    index_price REAL NOT NULL DEFAULT 0,
    funding_rate REAL NOT NULL,
    next_funding_time INTEGER NOT NULL
);
```

### Sled WAL (paper-service)

- Event sourcing için embedded key-value store
- Her `DomainEvent` sıralı olarak binary'e yazılır
- Süreç yeniden başladığında otomatik replay ile state restore

---

## ⚙️ Risk Yönetimi

### TitaniumOrchestrator (Strateji Yöneticisi)

- Birden fazla strateji aynı anda çalışabilir (`Vec<Box<dyn Strategy>>`)
- **Panic koruması:** `catch_unwind` ile stratejilerdeki hatalar yakalanır
- Hatalı strateji `Poisoned` durumuna geçer, diğerleri çalışmaya devam eder
- Timer callback desteği (her 1 ms'de `on_timer`)
- CPU optimize edilmiş spin-loop (`std::hint::spin_loop()`)

### Risk Durumları (PaperEngine)

| Durum | Açıklama |
|-------|----------|
| `Ok` | Normal çalışma |
| `MaxDrawdownBreached` | Drawdown limiti aşıldı |
| `MaxDailyLossBreached` | Günlük kayıp limiti aşıldı |
| `MaxLeverageBreached` | Kaldıraç limiti aşıldı |

### Risk Worker Modülleri

| Modül | İşlev |
|-------|-------|
| `matrix.rs` | Tikhonov (Ridge) regularizasyonu, Dinamik VWAP |
| `finops.rs` | Bulut maliyeti optimizasyonu (FinOps), ClickHouse cold data repack |
| `cache.rs` | Risk hesaplama önbelleği |

---

## 🔒 Formel Doğrulama (Kani Model Checker)

**Matematiksel olarak kanıtlanan özellikler:**

| Bileşen | Kanıtlanan Özellik |
|---------|--------------------|
| Ring Buffer | `head % slot_count` her zaman sınırlar içinde |
| Ring Buffer | Veri uzunluğu slot kapasitesini asla aşmaz |
| Ring Buffer | Generation counter sıfıra asla dönmez |
| Risk Engine | Bakiye asla negatife düşmez |
| Risk Engine | Drawdown %0–100 aralığında kalır |
| Risk Engine | Pozisyon büyüklüğü limitlere uyar |
| Risk Engine | Komisyon her zaman ≥ 0 |
| Risk Engine | Kaldıraç maksimumu asla aşmaz |
| Tick Validator | Fiyat/miktar/zaman doğrulaması matematiksel olarak sağlam |

---

## 🚀 Emir Yürütme Motoru (`execution-engine`)

- **Desteklenen Emir Türleri:** `Market`, `Limit`
- **Süre Politikaları:** `GTC` (Good-Til-Cancel), `IOC` (Immediate-or-Cancel), `FOK` (Fill-or-Kill)
- **Kimlik Doğrulama:** HMAC-SHA256 imzalama (Binance WebSocket API v3)
- **Canlı mod:** WebSocket Order API (`wss://ws-api.binance.com:443/ws-api/v3`)
- **Paper mod:** `TRADING_MODE=PAPER` ile PaperEngineActor'e yönlendirme
- **Yeniden bağlanma:** Bağlantı koptuğunda 3 sn bekleyerek otomatik reconnect

---

## ⚙️ Operasyon ve Dağıtım

### 🖥️ tmux Çok-Terminal Başlatıcı (`scripts/cycle_tmux.sh`)

Tüm sistem tek komutla 6 panelli bir tmux oturumu olarak başlar (`cycle`):

```
Pencere 0 — Trading:
  ┌──────────────────────┬──────────────────────┐
  │  📡 DATA             │  🛡️  PAPER-SERVICE    │
  ├──────────────────────┼──────────────────────┤
  │  🧠 STRATEGY         │  🔔 ALERT-SERVICE    │
  ├──────────────────────┴──────────────────────┤
  │  🛰️  LISTENER        │  💻 SHELL            │
  └─────────────────────────────────────────────┘
Pencere 1 — Monitor (CPU/RAM/GPU izleme)
```

```bash
./scripts/cycle_tmux.sh          # derle + başlat + bağlan
./scripts/cycle_tmux.sh kill     # tüm servisleri + ring buffer'ları temizle
./scripts/cycle_tmux.sh status   # çalışan servislerin CPU/RAM durumu
./scripts/cycle_tmux.sh attach   # mevcut oturuma bağlan
```

**Shell paneli yardımcı komutları** (`source scripts/cycle_env.sh`):
- `cycle-start` / `cycle-kill` / `cycle-status` / `cycle-build`
- `data-start|stop`, `strategy-start|stop`, `paper-start|stop`, `alert-start|stop`
- `listener-start|stop|status` — açık pozisyon metrikleri
- `paper-buy BTCUSDT 0.001`, `paper-sell`, `paper-balance`, `paper-positions`, `paper-health`
- `detect-ms-start|stop|query BTCUSDT 15m`, `heiusdt-start|stop|query`
- `monitor-start`, `db-trades`, `db-size`
- `help-cycle` — tam komut listesi

### 🛰️ Listener Katmanı (`scripts/listener.py`)

> Açık pozisyonları paper-service'ten çeker, anlık metrik analizi yapar ve `/tmp/listener_metrics.json`'a yazar. Şu an placeholder metrikler içerir (gerçek metrikler sonra eklenecek).

### 📦 Kurulum Paketi (`install.sh`)

```bash
./install.sh                # derle + ~/.cycle dizinine kur
./install.sh --prefix /opt  # özel kurulum dizini
./install.sh --only-build   # sadece derle
./install.sh --package      # kurulum + .tar.gz paketi
./install.sh --uninstall    # kaldır

# Kurulan paketi kullan
~/.cycle/bin/cycle start|stop|status
source ~/.cycle/cycle-env.sh
```

### 🚀 Hızlı Başlatma (Paper Sistemi)

```bash
# Tek komutla DATA + paper-service başlat
./scripts/start_paper.sh

# Parametreli başlatma
PAPER_INITIAL_USDT=50000 PAPER_ADMIN_PASS=güvenli123 ./scripts/start_paper.sh

# Durdurma
./scripts/stop_paper.sh
```

### Manuel Başlatma

```bash
# 1. DATA terminali (Canlı Binance Futures)
RUN_MODE=DATA cargo run --release -p core

# VEYA Backtest modu
RUN_MODE=BACKTEST CSV_PATH="./test_data.csv" cargo run --release -p core

# 2. Paper Service (REST API + Actor)
PAPER_ADMIN_USER=admin PAPER_ADMIN_PASS=changeme123 \
PAPER_API_ADDR=127.0.0.1:8080 PAPER_INITIAL_USDT=100000 \
cargo run --release -p paper-service

# 3. Strategy terminali (HEIUSDT kırılım stratejisi)
RUN_MODE=STRATEGY cargo run --release -p core

# 4. Korelasyon terminali
RUN_MODE=CORRELATION WINDOW_SEC=10 TRACK_SEC=10 cargo run --release -p core

# 5. Alert servisi
cargo run --release -p alert-service -- --config alerts.toml

# 6. Price feed daemon (Rust WS)
cargo run --release -p price-feed

# VEYA Python price feed (REST polling)
python3 scripts/price_feed.py

# 7. Detect-MS — MSMP 2.0 analiz API'si (:3002)
cargo run --release -p detect-ms
```

### Soğuk Başlangıç (`cold-starter`)

- Sistem açılışında Binance REST API'den geçmiş kline verisi çeker
- Algılayıcıların yeterli mum verisine (≥ 42) hemen sahip olmasını sağlar

### Kubernetes Dağıtımı (`k8s/`)

| Terminal | Bellek | CPU | Özellik |
|----------|--------|-----|---------|
| DATA | 256 Mi | 500m | `/dev/shm` emptyDir mount |
| STRATEGY | 512 Mi | 1 | `/dev/shm` emptyDir mount |
| PAPER-SERVICE | 256 Mi | 500m | `/dev/shm` emptyDir mount |

**Chaos Engineering Senaryoları:**
- [`chaos_dns_failure.yaml`](./k8s/chaos_dns_failure.yaml) — DNS arızası simülasyonu
- [`chaos_network_partition.yaml`](./k8s/chaos_network_partition.yaml) — Ağ bölünmesi
- [`chaos_ntp_drift.yaml`](./k8s/chaos_ntp_drift.yaml) — NTP saat kayması

---

## 🛠️ Derleme ve Önkoşullar

### Önkoşullar (Ubuntu/Debian)

```bash
# Python 3 (strateji/listener/price_feed scriptleri için)
sudo apt-get update
sudo apt-get install python3 python3-pip

# Ses uyarıları için (alert-service)
sudo apt-get install speech-dispatcher pulseaudio-utils

# tmux (çok-terminal başlatıcı)
sudo apt-get install tmux

# Rust araç zinciri
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Ortam Değişkenleri (`.env`)

```bash
# Borsa kimlik bilgileri
BINANCE_API_KEY=your_api_key
BINANCE_SECRET_KEY=your_secret_key

# Paper Service kimlik doğrulama
PAPER_ADMIN_USER=admin
PAPER_ADMIN_PASS=changeme123
PAPER_JWT_SECRET=paper-dev-secret-change-me
PAPER_API_ADDR=127.0.0.1:8080

# Paper Engine parametreleri
PAPER_INITIAL_USDT=100000
PAPER_MATCHING_MODE=PRICE_ONLY
PAPER_SLED_PATH=./paper_wal
PAPER_DB_PATH=./market_data.db

# Opsiyonel: PostgreSQL (--features full)
DATABASE_URL=postgresql://user:pass@localhost/paper_db

# Price Feed
PRICE_FEED_SYMBOLS=BTCUSDT,ETHUSDT,SOLUSDT,HEIUSDT
PRICE_FEED_PORT=3004

# Detect-MS
DETECT_MS_ADDR=127.0.0.1:3002
```

### Derleme

```bash
# Temel derleme
cargo build --release

# Tam set (PostgreSQL + Redis desteği)
cargo build --release --features full -p paper-service

# Formel doğrulama
cargo kani --package formal_verification
```

---

## 📊 Konfigürasyon

### `config/config_v6.toml` (ve `config_v5.toml`)

```toml
[api]
version = "v6"
endpoint = "wss://stream.binance.com:9443/ws/v6"

[trading]
max_positions = 100
```

---

## 🧪 Test ve Benchmark

- **Birim testleri:** `core/tests/`, `adapter/tests/`
- **Benchmark:** `core/benches/ring_bench.rs`, `core/benches/tick_benchmark.rs` — Ring buffer + tick parse throughput
- **Test verileri:** [`test_data.csv`](./test_data.csv) — Backtest simülasyonu için
- **WebSocket testi:** [`test_ws.py`](./test_ws.py), [`test_depth.py`](./test_depth.py)
- **Risk analizi:** [`scripts/risk_analysis.py`](./scripts/risk_analysis.py)
- **GDPR silme testi:** [`scripts/gdpr_erasure_test.sh`](./scripts/gdpr_erasure_test.sh)
---

## 📚 Dokümantasyon

| Dosya | İçerik |
|-------|--------|
| [complete_system_documentation.md](./docs/complete_system_documentation.md) | 30 KB — Tüm sistem mimarisi |
| [bare_metal_plan.md](./docs/bare_metal_plan.md) | Düşük gecikme yol haritası (Faz 0–4) |
| [code_reference.md](./docs/code_reference.md) | API referansı |
| [ring_buffer_schema.md](./docs/ring_buffer_schema.md) | Ring buffer hafıza düzeni |
| [adapter_schema.md](./docs/adapter_schema.md) | Borsa bağdaştırıcı API |
| [execution_schema.md](./docs/execution_schema.md) | Emir yürütme API |
| [tick_parser_schema.md](./docs/tick_parser_schema.md) | Tick ayrıştırma ve stream tipleri |
| [validator_schema.md](./docs/validator_schema.md) | Doğrulama kuralları |
| [db_schema.md](./docs/db_schema.md) | Veritabanı şeması |

---

## 🗂️ Proje Yapısı

```
PROJE/
├── core/                    # Ana orkestratör (5 terminal modu)
│   └── src/
│       ├── main.rs          # Terminal yönlendirici
│       ├── cli/             # paper_cli, strategy_cli, correlation_cli
│       ├── engine/          # orchestrator, backtester
│       ├── memory/          # ring_buffer, order_ring
│       └── (tick, validator, db, pii, risk, strategy, timer)
├── adapter/                 # Binance WS/REST bağdaştırıcısı
├── execution-engine/        # Emir motoru + PaperEngineActor
│   └── src/paper/           # actor, config, risk, position, snapshot...
├── paper-service/           # Bağımsız Paper Trading REST servisi
│   └── src/                 # api, bridge, events, metrics, idempotency
├── alert-service/           # Sesli fiyat uyarı servisi
│   └── src/                 # engine, audio, source, config
├── price-feed/              # Anlık last/mark/index price daemon (:3004)
│   └── src/main.rs          # WS → EventParser → ring → HTTP API
├── risk-worker/             # Risk matrisi & FinOps
├── ohlcv-engine/            # OHLCV mum oluşturucu + Binance kline client
├── os-utils/                # Düşük seviye OS yardımcıları
├── cold-starter/            # Soğuk başlangıç (kline verisi)
├── cold-storage/            # SQLite işlem kaydı
├── detect-sr/               # Destek/Direnç + FVG
├── detect-trend/            # Trend (EMA + ADX)
├── detect-ms/               # MSMP 2.0 REST API (:3002)
├── detect-liquidity/        # Likidite havuzu tespiti
├── detect-pattern/          # Mum çubuğu paternleri
├── formal_verification/     # Kani Model Checker testleri
├── strategies/              # Python strateji dosyaları
│   └── heiusdt_breakout.py  # HEIUSDT kırılım stratejisi
├── scripts/                 # Başlatma/durdurma & analiz scriptleri
│   ├── cycle_tmux.sh        # 6 panelli tmux başlatıcı
│   ├── cycle_env.sh         # Shell yardımcı komutları
│   ├── listener.py          # Anlık pozisyon metrikleri
│   ├── monitor.sh           # Servis izleme paneli
│   ├── price_feed.py        # Python price feed daemon
│   ├── start_paper.sh
│   ├── stop_paper.sh
│   ├── risk_analysis.py
│   └── gdpr_erasure_test.sh
├── install.sh               # Kurulum paketi (~/.cycle)
├── k8s/                     # Kubernetes manifests + Chaos senaryoları
├── config/                  # Konfigürasyon dosyaları
│   ├── config_v5.toml
│   └── config_v6.toml
├── docs/                    # Teknik dokümantasyon
├── alerts.toml              # Alert servisi yapılandırması
└── Cargo.toml               # Workspace tanımı (16 crate)
```
