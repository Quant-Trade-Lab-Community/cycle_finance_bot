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

**Not:** Bu sistem şu an **paper-trading** modundadır. `breakout-strategy` yalnızca **sembol + yön** sinyali üretir, emir açmaz. `paper-service` sanal emir yürütme sağlar. `execution-engine` canlı (LIVE) emir altyapısı için hazırlanmıştır ancak aktif değildir.

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
├── risk-engine/           # Risk motoru (matris, LOB sim, portfolio)
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

### Aktif Workspace Üyeleri (16)

`contracts, transport, core, adapter, os-utils, cold-storage, cold-starter, execution-engine, risk-engine, strategies-engine, breakout-strategy, ohlcv-engine, detect-ms, paper-service, alert-service, price-feed`

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
```

### 3. tmux ile tüm ortamı başlat (önerilen)

```bash
./additional-services/scripts/cycle_tmux.sh          # başlat + bağlan
./additional-services/scripts/cycle_tmux.sh kill     # durdur
./additional-services/scripts/cycle_tmux.sh status   # durum
```

> `cycle_tmux.sh` çalıştırıldığında önce **CYCLE FINANCE** FIGlet açılış ekranı tek terminalde (harf harf animasyon) gösterilir, ardından tmux session'ı ve 4'lü Trading ekranı açılır. Açılış ekranı `cycle-engine/splash` crate'i (`cycle-splash` binary) ile sağlanır; hız `show_splash_with(text, ms)` ile özelleştirilebilir.

### 4. Ortam fonksiyonları

```bash
source additional-services/scripts/cycle_env.sh
help-cycle   # komut listesi
```

Yaygın komutlar: `data-live`, `detect-ms-start`, `breakout-start`, `paper-start`, `alert-start`, `listener-start`, `risk-start`, `monitor-start`.

---

## 🖥️ tmux Terminal Düzeni

| Pencere | İçerik |
|---|---|
| 0 — Trading | STRATEGY / LISTENER / RISK / SHELL (4 panel) |
| 1 — DATA | Binance WS veri terminali |
| 2 — ALERT | Sesli uyarı servisi |
| 3 — PAPER | Paper REST API |
| 4 — Monitor | CPU/RAM/GPU izleme |
| 5 — DETECT-MS | Analiz motoru |
| 6 — BREAKOUT | Kırılım stratejisi |

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

- `risk-engine` — RiskEngine (pozisyon limiti, günlük kayıp limiti), LOB simülasyonu, portfolio PnL/drawdown
- `core/src/risk` — orkestratör içi sinyal risk filtresi (strateji → risk → gateway)
- Circuit breaker: 100+ hatalı/sn → akışı durdur

### Güvenlik

- `execution-engine/src/signer.rs` — Binance imzalama (HMAC-SHA256) altyapısı
- `paper-service` — JWT auth (argon2 şifre hash)
- `adapter/vault` — HashiCorp Vault entegrasyon taslağı (anahtar rotasyonu)
- `.env` repodan hariç tutulur (`BINANCE_API_KEY`, `BINANCE_SECRET_KEY`)

---

## 📜 Lisans / Not

Bu proje özel bir araştırma/geliştirme projesidir. Canlı para ile kullanım risk içerir; sistem yalnızca paper-trading için yapılandırılmıştır.

---

*Doküman oluşturma tarihi: 2026-08-08 · Ayrıntılı mimari doküman: `docs/PROJE_DOKUMANTASYONU.md` · Akış diyagramları: `docs/flowcharts/`*
