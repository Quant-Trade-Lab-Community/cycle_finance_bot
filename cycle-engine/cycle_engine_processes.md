# ⚙️ Cycle-Engine Fonksiyonel Süreçler

Veri toplama artık **8 bağımsız akış süreci** (`flows` crate) ile yapılır; `engine` yalnızca strateji orkestrasyonu sağlar:

- `flow-*` (8 binary) → **bağımsız veri akışı süreçleri**: `WS → parse → validate → ring → TimescaleDB`
- `strategy-console` → **strateji orkestrasyon merkezi**: stratejileri alt-süreç olarak yönetir (Bölüm 3)

```
./target/debug/flow-trade         → Trade akışı (WS→ring→TimescaleDB)
./target/debug/flow-depth         → Depth20 akışı
./target/debug/flow-liquidation   → Likidasyon akışı
./target/debug/flow-oi            → Open Interest akışı
./target/debug/flow-funding       → Funding Rate akışı
./target/debug/flow-markprice     → Mark Price akışı
./target/debug/flow-lastprice     → Last Price akışı
./target/debug/flow-indexprice    → Index Price akışı
./target/debug/strategy-console   → Strateji orkestrasyon merkezi
```

---

## 1. Veri Akışı Süreçleri — Ana Veri Hattı

Her akış **ayrı bir OS sürecidir** ve 3 eşzamanlı parça çalıştırır:

```mermaid
flowchart TD
    subgraph "AKIŞ SÜRECİ (ör. flow-trade)"
        subgraph "TOKIO ASYNC RUNTIME"
            A["start_ws_client(tx, streams, use_gate=true)<br/>RateGate'ten token al → WS bağlan → SUBSCRIBE"]
        end

        subgraph "THREAD — İŞLEMCİ (RT Priority 99)"
            B1["rx.recv() — Bounded Kanal Okuma"]
            B2["parse_for()<br/>simd_json / EventParser"]
            B3["DataValidator::is_valid()<br/>Circuit Breaker"]
            B4["wire::encode()<br/>Compact Binary"]
            B5["ring.push()<br/>kendi akış ring'i (/dev/shm)"]
            B6["db_tx.try_send()<br/>DB kanalına gönder"]
            B1 --> B2 --> B3 --> B4 --> B5
            B3 --> B6
        end

        subgraph "THREAD — DB WRITER"
            C1["db_rx.recv()"]
            C2["TimescaleDB INSERT<br/>(akışın hypertable'ı)"]
            C3{"1000 kayıt VEYA<br/>1sn geçti mi?"}
            C4["tx.commit()"]
            C1 --> C2 --> C3
            C3 -->|Evet| C4
            C4 --> C1
        end
    end

    RG["RateGate<br/>/dev/shm/cycle_finance_api_gate"] -.->|"token"| A
    A -->|"flume bounded<br/>Vec&lt;u8&gt;"| B1
    B6 -->|"flume bounded(1M)<br/>OwnedEvent"| C1

    style A fill:#1a1a2e,stroke:#e94560,color:#fff
    style B1 fill:#0f3460,stroke:#fff,color:#fff
    style B5 fill:#533483,stroke:#fff,color:#fff
    style C4 fill:#e94560,stroke:#fff,color:#fff
```

**Akış → kaynak → ring → hypertable eşlemesi:**

| Akış | Binary | Kaynak | Stream / REST | Ring | Bellek | Hypertable |
|:---|:---|:---|:---|:---|:---:|:---|
| 1. Trade | `flow-trade` | **WS** | `{sym}@trade` | `/cycle_finance_trades` | 50 MB | `trades` |
| 2. Depth20 | `flow-depth` | **WS** | `{sym}@depth20@100ms` | `/cycle_finance_depth` | 100 MB | `orderbooks` |
| 3. Likidasyon | `flow-liquidation` | **WS** | `{sym}@forceOrder` | `/cycle_finance_liquidations` | 20 MB | `liquidations` |
| 4. Open Interest | `flow-oi` | **REST** | `GET /fapi/v1/openInterest` | `/cycle_finance_open_interest` | 20 MB | `open_interests` |
| 5. Funding Rate | `flow-funding` | **REST** | `GET /fapi/v1/premiumIndex` | `/cycle_finance_funding` | 10 MB | `funding_rates` |
| 6. Mark Price | `flow-markprice` | **REST** | `GET /fapi/v1/premiumIndex` | `/cycle_finance_markprice` | 50 MB | `markprices` |
| 7. Last Price | `flow-lastprice` | **REST** | `GET /fapi/v1/ticker/price` | `/cycle_finance_lastprice` | 50 MB | `lastprices` |
| 8. Index Price | `flow-indexprice` | **REST** | `GET /fapi/v1/premiumIndex` | `/cycle_finance_indexprice` | 50 MB | `indexprices` |

> **REST fallback:** Bu ağdan Binance markPrice/indexPrice/lastPrice/openInterest stream'leri WS ile
> iletilmediği için o akışlar REST ile beslenir (`flows/src/rest.rs`); REST yanıtı WS-format frame'e
> çevrilir ve aynı `parse → validate → ring → TimescaleDB` hattından geçer. Semboller
> `CYCLE_FLOW_SYMBOLS` ile değiştirilebilir (varsayılan `BTCUSDT,ETHUSDT,SOLUSDT,HEIUSDT`).
>
> **Likidasyon:** Binance'te piyasa geneli likidasyon için REST endpoint yoktur (`allForceOrders` 404);
> `flow-liquidation` WS aboneliğinde kalır — bu ağda stream iletilmediği için tablo boş, stream çalışan ağda anında dolar.

### Süreç 1: Veri Kaynağı — WebSocket + Rate Kapısı / REST Fallback

**Dosya:** [gateway/src/binance.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/gateway/src/binance.rs) · [gateway/src/rate_gate.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/gateway/src/rate_gate.rs)

```
start_ws_client(tx, streams, use_gate=true)
  ├── chunks(200) → Her chunk için tokio::spawn
  │     └── start_ws_chunk(tx, chunk, id, use_gate)
  │           ├── [use_gate] RateGate::acquire(30s)   ← Binance limit koruması
  │           ├── connect_async("wss://fstream.binance.com/stream")
  │           ├── SUBSCRIBE JSON mesajı gönder
  │           └── tokio::select! döngüsü:
  │                 ├── ping_interval.tick() → Ping gönder (30sn)
  │                 └── read.next() → Text mesajı → into_bytes() → tx.send_async()
  │
  └── Chunk'lar arası 600ms gecikme (WAF koruması)
```

**Rate kapısı (API rate limit koruması):** `RateGate` `/dev/shm/cycle_finance_api_gate` üzerinde prosesler arası token bucket'tır. Bağımsız 8 akış aynı bütçeyi paylaşır; kapasite/dolum `CYCLE_GATE_CAPACITY` / `CYCLE_GATE_RATE` ile ayarlanır.

**REST fallback ([rest.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/flows/src/rest.rs)):** WS ile gelmeyen akışlar (funding, markprice, indexprice, lastprice, oi) her poll döngüsünde (`CYCLE_REST_POLL_MS`, varsayılan 2s) `RateGate` token'ı alır, REST yanıtını WS-format frame'e çevirir ve aynı `raw_tx` kanalına yazar — parse/validate/ring/TSDB hattı değişmez.

**Rate güvenliği (REST):**
- HTTP **429** → 60 sn, HTTP **418** (teapot/IP banı) → 5 dk geri çekilme (asla aynı hızda vurmaya devam etmez)
- Her akış dakikalık ağırlığını (request × endpoint weight: premiumIndex=1, ticker/price=2, openInterest=1) `/tmp/cycle_flow_weights/<flow>.weight` dosyasına yazar → **monitor sekmesi** toplamı gösterir (limit 2400/dk)

**Hata yönetimi:**
- Bağlantı koparsa → Exponential backoff (1s → 2s → 4s → ... → 60s tavan)
- Ping başarısız → Derhal reconnect döngüsüne gir
- Tüketici kanal kapandıysa → Task'ı temiz sonlandır

---

### Süreç 2: Veri İşleme Hattı (RT Priority Thread)

**Dosya:** [flows/src/lib.rs:76-119](file:///home/smhvz/Desktop/PROJE/cycle-engine/flows/src/lib.rs#L76-L119) · [flows/src/parse.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/flows/src/parse.rs)

Bu thread `SCHED_FIFO` öncelik 99 ile çalışır (Linux RT scheduler). İşlem adımları:

```
set_rt_thread_priority(99)  ← OS-level realtime öncelik

while let Ok(mut bytes) = raw_rx.recv() {
    ┌─ ADIM 1: PARSE ────────────────────────────────────────┐
    │  parse_for(kind, &mut bytes)                            │
    │  • Tanınan stream'ler → EventParser (simd_json)         │
    │  • @lastPrice/@indexPrice/!openInterest → ek ayrıştırıcı│
    │  • Mevcut EventType varyantlarına eşlenir (yeni eklenmez)│
    └─────────────────────────────────────────────────────────┘
           │
           ▼
    ┌─ ADIM 2: DOĞRULAMA ────────────────────────────────────┐
    │  DataValidator::is_valid(&owned_event)                   │
    │  • Fiyat/Miktar <= 0 → REJECT                           │
    │  • Gecikme > 200ms (Stale) → REJECT                     │
    │  • NTP Drift > 5000ms → REJECT                          │
    │  • Crossed Book (Bid >= Ask) → REJECT                   │
    │  • 1sn'de 100+ reject → CIRCUIT BREAKER!                │
    └─────────────────────────────────────────────────────────┘
           │
           ▼
    ┌─ ADIM 3: KODLAMA + DAĞITIM ────────────────────────────┐
    │  wire::encode(&owned_event, &mut frame_buf)             │
    │  • OwnedEvent → Compact Binary Frame (max 659B)        │
    │                                                          │
    │  ring.push(&frame_buf[..len])                           │
    │  • Bu akışın ring'ine atom yazma (örn. /cycle_finance_trades)│
    │  • Torn-read korumalı: data → fence → seq → head        │
    └─────────────────────────────────────────────────────────┘
           │
           ▼
    ┌─ ADIM 4: DB KANALINA GÖNDERİM ────────────────────────┐
    │  db_tx.try_send(owned_event)                             │
    │  • Non-blocking: Kuyruk doluysa drop (db_drops++)        │
    │  • Hot path'i asla bloke etmez                           │
    └─────────────────────────────────────────────────────────┘
           │
           ▼
    ┌─ ADIM 5: PERFORMANS RAPORU (1 saniyede bir) ───────────┐
    │  "[<akış>] evt/s: X | invalid: Y | db_drops: Z"          │
    └─────────────────────────────────────────────────────────┘
}
```

---

### Süreç 3: TimescaleDB Yazıcı (Ayrı Thread)

**Dosya:** [persistence/src/timescaledb.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/persistence/src/timescaledb.rs)

> **Kurulum (PC'ye native):** PostgreSQL 18 + TimescaleDB 2.x (`timescaledb-tune` ile `shared_preload_libraries='timescaledb'`, `CREATE EXTENSION timescaledb`). Kullanıcı `cycle` / şifre `cycle`, DB `market_data`; bağlantı `TIMESCALEDB_URL` (varsayılan `postgres://cycle:cycle@localhost:5432/market_data`).

```
start_tsdb_writer(db_rx, kind)
  ├── sqlx PgPool bağlantısı (TIMESCALEDB_URL, 2s retry — akışı asla durdurmaz)
  ├── Akışın hypertable'ını oluştur (CREATE TABLE + create_hypertable)
  └── Batch döngüsü:
        while let Ok(event) = rx.recv_timeout(250ms) {
            match (kind, event.payload) {
                Trade    → INSERT INTO trades
                Orderbook→ INSERT INTO orderbooks (bids/asks JSONB)
                Liquidation → INSERT INTO liquidations
                OpenInterest → INSERT INTO open_interests
                FundingRate (funding)  → INSERT INTO funding_rates
                FundingRate (markprice)→ INSERT INTO markprices (mark_price)
                FundingRate (indexprice)→ INSERT INTO indexprices (index_price)
                FundingRate (lastprice)→ INSERT INTO lastprices (mark_price)
            }
            batch_count++

            if batch_count >= 1000 || 1sn geçtiyse {
                tx.commit()     ← Toplu yazma
            }
        }
```

**Optimizasyon:**
- TimescaleDB hypertable'ları `timestamp` üzerinde bölümlenir (zaman serisi sorguları)
- 1000 batch: Disk I/O'yu minimize eder
- `try_send`: Ana hat doluysa DB yerine performansı tercih eder
- Bağlantı yoksa bekleme ile yeniden dener — veri akışı kuralları bozulmaz

---

### Süreç 4: Ring Buffer Tüketicileri (Harici Süreçler)

Her akış kendi ring'ine yazar; diğer bağımsız OS süreçleri o ring'leri **RAM'den** (paylaşımlı bellek) okur:

| Tüketici | Ring Buffer | Açıklama |
|:---|:---|:---|
| `breakout-strategy` | `/cycle_finance_trades` | Kırılım stratejisi fiyatı (event-driven) |
| `alert-service` | `/cycle_finance_trades` | Sesli fiyat uyarısı |
| `paper-service` | `/cycle_finance_trades` | Mark price güncellemesi (dolum/likidasyon) |
| `stream-ohlcv` | `/cycle_finance_lastprice` | Canlı mum güncellemesi |
| `risk-worker` | `/cycle_finance_markprice` | Risk parametreleri (VaR/korelasyon) |
| `calc-ind` | `/cycle_finance_calc` | İndikatör hesaplama (:3007) |
| `stream-ohlcv` | `/cycle_finance_stream_ohlcv` | OHLCV mum üretimi |

---

## 2. TitaniumOrchestrator (İç-Süreç Strateji Motoru)

Aynı süreç içinde çalışan, ring buffer'dan veri okuyup stratejileri `catch_unwind` ile değerlendiren spin-loop motorudur. Şu anda hiçbir binary doğrudan başlatmaz; canlı strateji yürütme, ayrı `strategy-console` sürecinin alt-süreç yönetimiyle yapılır (Bölüm 3).

**Dosya:** [engine/src/engine/orchestrator.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/engine/src/engine/orchestrator.rs)

```mermaid
flowchart TD
    A["run_spin_loop() başlat"] --> B["TscTimer başlat<br/>RDTSC nanosaniye sayacı"]
    
    B --> C{"Ring Buffer'da<br/>yeni veri var mı?<br/>(head > cursor)"}
    C -->|Hayır| D["std::hint::spin_loop()<br/>CPU PAUSE instruction"]
    D --> C
    
    C -->|Evet| E["read_slot(head)<br/>wire::decode()"]
    E --> F["Her strateji için:"]
    
    F --> G{"state == Active?"}
    G -->|Hayır| F
    G -->|Evet| H["catch_unwind {<br/>  strategy.on_market_data(frame_id, slot)<br/>}"]
    
    H --> I{"Panic oldu mu?"}
    I -->|Evet| J["state = Poisoned<br/>Strateji devre dışı"]
    I -->|Hayır| K["Signal döndü"]
    
    K --> L["signal_to_intent()<br/>Signal → OrderIntent"]
    L --> M{"Signal::None<br/>veya CancelAll?"}
    M -->|Evet| F
    M -->|Hayır| N["RiskEngine.evaluate(intent)"]
    
    N --> O{"Risk kararı?"}
    O -->|Approved| P["gateway_tx.send(Signal)<br/>Emri yürütme katmanına gönder"]
    O -->|Rejected| Q["Log: rule_name + describe()"]
    
    P --> F
    Q --> F

    subgraph "Timer Tick (Her 1ms)"
        R["current_time - last_tick > 1_000_000 ns?"]
        R -->|Evet| S["strategy.on_timer(frame_id, delta)"]
        S --> T["Aynı gate_and_dispatch akışı"]
    end

    style A fill:#1a1a2e,stroke:#e94560,color:#fff
    style H fill:#533483,stroke:#fff,color:#fff
    style N fill:#e94560,stroke:#fff,color:#fff
    style P fill:#0f3460,stroke:#fff,color:#fff
```

**İki tetikleyici:**
1. **Market Data:** Ring buffer'a yeni veri yazıldığında stratejiler değerlendirilir
2. **Timer Tick:** Her ~1ms'de stratejilere `on_timer()` çağrısı yapılır (zaman bazlı kararlar)

**Risk kapısı (gate_and_dispatch):**
```
Signal → signal_to_intent() → OrderIntent
  ├── BuyMarket{qty}  → OrderIntent{side:Buy, kind:Market}
  ├── SellMarket{qty}  → OrderIntent{side:Sell, kind:Market}
  ├── BuyLimit{p,q}    → OrderIntent{side:Buy, kind:Limit}
  ├── SellLimit{p,q}   → OrderIntent{side:Sell, kind:Limit}
  └── None/CancelAll   → (atla)

RiskEngine.evaluate(intent) →
  ├── Approved → gateway_tx.send(signal)
  └── Rejected{reason} → log(rule_name, describe)
```

---

## 3. STRATEJİ Orkestrasyon Konsolu (strategy-console)

DATA konsolundan tamamen bağımsız, ayrı bir OS sürecidir. `engine` crate'inden ayrı binary olarak derlenir.

**Dosyalar:** [engine/src/bin/strategy-console.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/engine/src/bin/strategy-console.rs) · [engine/src/engine/strategy_console.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/engine/src/engine/strategy_console.rs) · [strategies-engine/src/orchestrator.rs](file:///home/smhvz/Desktop/PROJE/strategies-engine/src/orchestrator.rs)

```mermaid
flowchart LR
    SH["cycle-engine Shell<br/>(tmux pencere 0)"] -->|"strat run breakout"| CMD["/tmp/strategy_cmd.d/cmd_*.cmd<br/>maildir benzeri kuyruk"]
    CMD --> SC["strategy-console<br/>StrategyOrchestrator"]
    SC -->|"spawn / SIGTERM"| ST["breakout-strategy<br/>(alt-süreç)"]
    SC -->|"durum yazma"| STAT["/tmp/strategy_status.txt"]
    ST -->|"okur"| PF["/cycle_finance_trades (flow ring — RAM)"]
```

- `StrategyOrchestrator` (strategies-engine) `services-engine/strategies/` klasörünü tarar; her strateji dizini ayrı alt-süreç olarak yönetilir
- Strateji adları dizin adından gelir; kısa ad takma adı desteklenir (`breakout` → `breakout-strategy`)
- Shell komutları kuyruk dosyasına yazılır, konsol 250ms'de poll eder
- `tick()` ölen alt-süreçleri toplar (reap); `status()` çalışan/mevcut stratejileri raporlar
- Çıkışta tüm yönetilen stratejiler SIGTERM ile durdurulur

| Komut | Açıklama |
|:---|:---|
| `strat run breakout` | Stratejiyi başlat |
| `strat stop breakout` | Stratejiyi durdur |
| `strat restart breakout` | Yeniden başlat |
| `strat list` / `strat status` | Mevcut / çalışan stratejiler |
| `strat attach` | tmux pencere 1'e (🧠 STRATEGY) geç |

> tmux eşlemesi: `1 — 🧠 STRATEGY` → `strategy-console` · `12-19` → 8 veri akışı (`flow-*`). Strateji çıktısı akış ekranlarına karışmaz.

---

## 4. Detektör Köprüsü (Scout Ring Okuyucu)

**Dosya:** [engine/src/bridge/detector_bridge.rs](file:///home/smhvz/Desktop/PROJE/cycle-engine/engine/src/bridge/detector_bridge.rs)

Harici detektörler (detect-ms, candle-classifier vb.) `/cycle_finance_scout` ring buffer'ına `Opportunity` frame'leri yazar. Bu köprü onları okur.

```
spawn_watcher(handler) → Tokio arka plan task'ı:
  loop {
      bridge.poll(handler)  ← Scout ring'deki yeni Opportunity'leri oku
        ├── read_slot(cursor) → wire::decode()
        ├── EventType::Opportunity → OpportunityHit oluştur
        │     ├── symbol, score, efficiency
        │     ├── price_bps_per_s, price_ticks_per_s
        │     ├── spread_bps, verdict (0=GÜÇLÜ → 4=ZAYIF)
        │     └── is_actionable(max_verdict) filtresi
        └── handler(&hit) çağır

      tokio::time::sleep(100ms)  ← Polling aralığı
  }
```

---

## 5. Thread/Task Haritası Özeti

**Her veri akışı süreci (8 adet):**

| # | Parça | Tip | Öncelik | Bloklanır mı? |
|:---:|:---|:---|:---:|:---:|
| 1 | WebSocket görevi | Tokio Task | Normal | Async I/O |
| 2 | Veri İşleme Hattı | OS Thread | **RT 99** | Hayır (bounded queue) |
| 3 | TimescaleDB Writer | OS Thread | Normal | Evet (DB I/O, 2s retry) |

**Genel sistem:**

| # | Süreç | Tip | Öncelik | Bloklanır mı? |
|:---:|:---|:---|:---:|:---:|
| 1 | `flow-*` (8 adet — bağımsız veri akışı) | Ayrı OS Süreci | RT 99 (işlemci thread) | Hayır |
| 2 | Rate kapısı (acquire) | Akış içi bekleme | Normal | 25ms poll / 30s tavan |
| 3 | Orchestrator Spin-Loop | OS Thread | RT | **Asla** (spin_loop) |
| 4 | Scout Watcher | Tokio Task | Normal | 100ms sleep |
| 5 | Timer Tick | Orkestratör içi | RT | Hayır |
| 6 | strategy-console (orkestrasyon) | Ayrı OS Süreci | Normal | 250ms poll |
| 7 | breakout-strategy (yönetilen alt-süreç) | Ayrı OS Süreci | Normal | Evet (döngü bekleme) |

> 6–7 satırları veri akışlarından bağımsız çalışır: `strategy-console`, strateji süreçlerini spawn eder/yönetir (Bölüm 3).

```mermaid
graph LR
    subgraph "8 × Akış Süreci (flow-*)"
        A1["flow-trade"]
        A2["flow-depth"]
        A3["...diğer 6 akış"]
    end

    subgraph "Akış içi Thread'ler"
        T1["Tokio Runtime<br/>(WS task)"]
        T2["İşlemci Thread<br/>(RT Priority 99)"]
        T3["TSDB Writer Thread"]
    end

    subgraph "Shared Memory IPC (/dev/shm)"
        R1["cycle_finance_trades"]
        R2["cycle_finance_depth"]
        RG["cycle_finance_api_gate"]
    end

    A1 --> T1
    A2 --> T1
    A3 --> T1
    T1 -->|"flume"| T2
    T2 -->|"push"| R1
    T2 -->|"push"| R2
    T2 -->|"flume"| T3
    T1 -.->|"token"| RG

    style T2 fill:#e94560,stroke:#fff,color:#fff
    style R1 fill:#533483,stroke:#fff,color:#fff
    style RG fill:#b7950b,stroke:#fff,color:#fff
```
