# 🎯 Strategies Engine — Tam Kaynak Kodu + Detaylı Analiz

> `strategies-engine/`. Bu doküman dizin ağacını, klasör/dosya sözlüğünü, her dosyanın **tam kaynak kodunu** ve **detaylı analizini** (mermaid akış diyagramlarıyla) içerir. Tarih: 2026-08-09

---

## 📂 İçindekiler

- [Dizin Ağacı](#dizin-agac)
- [Klasör ve Dosya Sözlüğü](#klasor-ve-dosya-sozlugu)
- [Detaylı Analiz (mermaid)](#detayl-analiz-mermaid)
- [Tam Kaynak Kodu](#tam-kaynak-kodu)

---

## 🌳 Dizin Ağacı

```
strategies-engine/
├── Cargo.toml
├── mod.rs
├── trait_def.rs
    ├── breakout-strategy/Cargo.toml
        ├── breakout-strategy/src/lib.rs
        ├── breakout-strategy/src/main.rs
        ├── breakout-strategy/src/metrics.rs
            ├── breakout-strategy/src/bin/alerts.rs
            ├── breakout-strategy/src/bin/listener.rs
            ├── breakout-strategy/src/bin/risk_analysis.rs
```

---

## 📖 Klasör ve Dosya Sözlüğü

> `strategies-engine/` — **Genel amaç:** Strateji katmanı. `trait_def.rs`'te strateji sözleşmesi, `breakout-strategy` ise ilk canlı strateji: ring'inden fiyat okur, detect-ms'ten seviye alır, kırılım kontrolü yapar ve sinyal üretir.
| Klasör / Dosya | Anlamı |
|---|---|
| `strategies-engine/` | Strateji katmanının kök workspace lib kutusu; `Strategy` trait arayüzünü tanımlar |
| `strategies-engine/Cargo.toml` | Lib manifesti; kütüphaneyi `mod.rs` üzerinden derler, `transport` ve workspace `rust_decimal` bağımlılıklarını ilan eder |
| `strategies-engine/mod.rs` | Lib kök modülü; `trait_def` modülünü dışa açar (tek satırlık modül ağacı) |
| `strategies-engine/trait_def.rs` | `Signal`, `FillReport` veri tipleri ile tüm stratejilerin uygulaması gereken `Strategy` trait'ini tanımlar |
| `breakout-strategy/` | Kırılım stratejisinin bağımsız binary kutusu; sinyal üretici + metrik/korelasyon/alarm/risk yardımcı araçları |
| `breakout-strategy/Cargo.toml` | Binary manifesti; main, listener, alerts, risk_analysis hedeflerini tanımlar ve çalışma zamanı bağımlılıklarını (tokio, reqwest, serde, sqlx, contracts, transport) ilan eder |
| `breakout-strategy/src/lib.rs` | Kutunun lib kökü; `metrics` mikro-yapı metrik çekirdeğini modül olarak açar |
| `breakout-strategy/src/main.rs` | Event-driven kırılım stratejisi: ring'den fiyat okur, detect-ms'ten seviye alır, kırılımı değerlendirir ve BUY/SELL sinyali üretir |
| `breakout-strategy/src/metrics.rs` | Kurumsal tick-by-tick mikro-yapı metrik çekirdeği: Lee-Ready, WLOBI, EffDelta, Absorption, aVPIN, Hasbrouck VAR, EfP ve Alpha Basket sinyali |
| `breakout-strategy/src/bin/listener.rs` | Veri merkezi izleyicisi: mikro-yapı metrik tablosu + fiyat korelasyonu + DATA trade hacim korelasyonu, JSON çıktılı |
| `breakout-strategy/src/bin/alerts.rs` | `alerts.toml` yönetim CLI'ı: alarm bloklarını listele/ekle/güncelle/sil (toml el-ayrıştırıcılı) |
| `breakout-strategy/src/bin/risk_analysis.rs` | `TimescaleDB`'deki trades tablosunu SQL ile özetleyen risk/hacim/volatilite raporlayıcı (watch modlu) |

---

## 🔬 Detaylı Analiz (mermaid)

> Her dosyanın **detaylı açıklaması**, **algoritmik akış diyagramı** (mermaid) ve **"neden kullandık"** gerekçesi. Mermaid diyagramları HTML'de otomatik çizilir. Tarih: 2026-08-09

### `strategies-engine/Cargo.toml`
**Detaylı açıklama:** Lib kutusunu `[lib] path = "mod.rs"` ile tanımlar; yani workspace kökünün bu dizini bir kütüphane olarak derlediği paket manifestidir. `transport` bağımlılığı path ile `../cycle-engine/transport`'tan alınır (paylaşımlı bellek ring buffer'ları için), `rust_decimal` ise workspace'te tek sürümde tutulur (HFT'de float yerine ondalık hassasiyeti korumak için). Versiyon 0.1.0 / edition 2021'dir; mantıksal olarak strateji tanımı yalnızca bu iki dış bağımlılığa dayanır — emir/veri katmanlarıyla bağı transport üzerinden kurulur.
**Neden kullandık:** Workspace düzeninde ortak `rust_decimal` sürümünü merkezileştirmek; transport ring buffer arayüzünü tek path ile içe almak; kutuyu lib olarak derleyip tüm stratejilerin aynı trait'e bağlanmasını sağlamak.

```mermaid
flowchart LR
    A["strategies-engine<br>(lib: mod.rs)"] --> B["transport<br>../cycle-engine/transport"]
    A --> C["rust_decimal<br>(workspace)"]
    B --> D["ring_buffer<br>MarketDataSlot,<br>GenerationalRingBuffer"]
```

### `strategies-engine/mod.rs`
**Detaylı açıklama:** Tek satırlık kök modül dosyasıdır; `pub mod trait_def;` ile strateji arayüzünü kutu dışına açar. Bir `Strategy` arayüzü dışında strateji çekirdeğinde başka modül yoktur — alt stratejiler (ör. breakout) kendi kutularında yaşar ve bu lib'i bağımlılık olarak kullanır.
**Neden kullandık:** Lib kökünü mod.rs olarak ayırmak (Cargo.toml path eşleşmesi); trait tanımını tek modülde tutarak alt stratejilerin `use strategies_engine::trait_def::Strategy` ile tüketmesini sağlamak.

```mermaid
flowchart LR
    A["mod.rs<br>pub mod trait_def"] --> B["trait_def.rs<br>Strategy trait + Signal"]
```

### `strategies-engine/trait_def.rs`
**Detaylı açıklama:** Strateji sözleşmesini kurar. `Signal` enum'u bir stratejinin karar çıktısını modeller: `None` (işlem yok), `BuyMarket`/`SellMarket` (miktar bazlı piyasa emri), `BuyLimit`/`SellLimit` (fiyat+miktar bazlı limit emri) ve `CancelAll`. `FillReport` ise gerçekleşen emir bilgisini (order_id, executed_qty, avg_price) taşır. `Strategy` trait'i `Send + Sync` olup dört olay güdümlü geri çağrı sunar: `on_market_data` (ring slot'undan veri geldiğinde), `on_timer` (zamanlayıcı tick'i, `frame_id` + süre farkı ile), `on_fill` (fill raporu geldiğinde) ve `reset` (durum sıfırlama). Motor bu çağrıları sırayla tetikler, strateji her çağrıda bir `Signal` döndürür.
**Neden kullandık:** Tüm stratejilerin tek tip veri/karar arayüzüne bağlanması; olay güdümlü HFT döngüsüne uygun, kilit/engelleme barındırmayan saf `Send + Sync` sözleşme; emir çeşitlerini (market/limit) tek enum ile taşıyabilme.

```mermaid
flowchart TD
    A["Motor / Ring"] -->|"MarketDataSlot + frame_id"| B["on_market_data"]
    B --> C{"Karar?"}
    C -->|"işlem yok"| D["Signal::None"]
    C -->|"piyasa al"| E["Signal::BuyMarket<br>qty"]
    C -->|"piyasa sat"| F["Signal::SellMarket<br>qty"]
    C -->|"limit emir"| G["Signal::BuyLimit /<br>SellLimit<br>price + qty"]
    C -->|"tüm emirleri iptal"| H["Signal::CancelAll"]
    A -->|"delta_ns + frame_id"| I["on_timer"]
    I --> C
    J["FillReport<br>order_id, qty, avg"] --> K["on_fill"]
    K --> C
```

### `breakout-strategy/Cargo.toml`
**Detaylı açıklama:** `breakout-strategy` binary kutusunun manifestidir; `src/main.rs` ana binary (`breakout-strategy`), diğer yardımcı araçlar (`listener.rs`, `alerts.rs`, `risk_analysis.rs`) `src/bin/` altında otomatik binary hedefleri olarak derlenir. Bağımlılık seti iki gruba ayrılır: çalışma zamanı servis katmanı (`tokio`, `reqwest`, `serde`, `serde_json`, `chrono`, `sqlx`) ve çekirdek sözleşmeler (`contracts` = event/wire kodlama, `transport` = ring buffer okuma, `rust_decimal` = hassas fiyat). `contracts` ve `transport` workspace path'leriyle (`../../cycle-engine/`) katmanlar arası bağ kurulur.
**Neden kullandık:** Çekirdek event tiplerini (`contracts::events`) ve wire decode'u transport'tan ayrı bir sözleşme katmanına taşımak; tokio async döngü + reqwest REST + sqlx raporlama ihtiyacını tek kutuda karşılamak; bin'lerin ortak kütüphane (`metrics.rs`) paylaşımını sağlamak.

```mermaid
flowchart LR
    A["breakout-strategy"] --> B["main.rs<br>kırılım sinyali"]
    A --> C["listener.rs<br>metrik + korelasyon"]
    A --> D["alerts.rs<br>toml yönetimi"]
    A --> E["risk_analysis.rs<br>SQL risk"]
    A --> F["contracts<br>events + wire"]
    A --> G["transport<br>GenerationalRingBuffer"]
    A --> H["tokio / reqwest / serde / sqlx / chrono / rust_decimal"]
```

### `breakout-strategy/src/main.rs`
**Detaylı açıklama:** Event-driven kırılım stratejisinin ana binary'sidir. Bir std thread (`spawn_price_reader`) paylaşımlı bellekteki ring'ini (`/cycle_finance_pricefeed`) sürekli okur, `wire::decode` ile event'i çözüp sembol eşleşmesi yapar, `event_price` ile tek fiyatı (Trade→ask öncelikli BookTicker→mark) çıkarır ve mpsc unbounded kanala basar. Tokio actor döngüsü fiyatları 500 ms zaman aşımıyla alır (fiyat daima anlık), beklenen `wait_sec` (varsayılan 20 dk, `/tmp/breakout_wait_sec.txt` ile dinamik) dolduğunda `analyze_once` çağırır: detect-ms (`:3002/api/ms`) seviyelerini alır, fiyat kaynağını ring→REST→detect-ms `current_price` önceliğiyle seçer, `evaluate` ile kırılımı değerlendirir. `evaluate` ATS işaretine bakar: ATS>0 iken en yüksek skorlu `SH` (direnç) seviyesi `price > SH` ise BUY, ATS<0 iken `SL` (destek) seviyesi `price < SL` ise SELL sinyali üretir. `--once` modu tek değerlendirme yapar; normal modda döngü sonsuzdur. Kod şu an **sinyal üreticidir**: emir açmaz, sadece sembol + yön bilgisini konsola basar; kararın emire dönüşmesi execution/paper-service katmanının işidir.
**Neden kullandık:** Polling yerine ring'den event-by-event fiyat alarak gecikmeyi düşürmek; fiyat kaynağını üçlü öncelikle (ring→REST→detect-ms) dayanıklı hale getirmek; kırılım mantığını saf test edilebilir `evaluate` fonksiyonuna ayırarak seviye seçimini (`best_level`) skorla yapmak; bekleme süresini dosya üzerinden çalışırken değiştirilebilir kılmak.

```mermaid
flowchart TD
    subgraph K2["Katman 2: Ring Okuyucu (std thread)"]
        A["ring<br>/cycle_finance_pricefeed"] --> B["read_slot + get_head"]
        B --> C["wire::decode"]
        C --> D{"sembol eşleşti mi?"}
        D -->|"hayır"| B
        D -->|"evet"| E["event_price<br>Trade/BookTicker/FundingRate"]
        E --> F["mpsc unbounded<br>tx.send fiyat"]
    end
    subgraph ACT["Actor Döngüsü (tokio)"]
        G["rx.recv<br>500ms zaman aşımı"] --> H["latest_price güncelle"]
        H --> I{"wait_sec doldu mu?"}
        I -->|"hayır"| G
        I -->|"evet"| J["analyze_once"]
    end
    subgraph SERV["Servis Katmanı"]
        K["detect-ms :3002<br>/api/ms?symbol+interval"] --> L["seviyeler + ATS + trend"]
        M[":3004<br>/api/lastprice (REST)"] --> N["fiyat seçimi<br>ring öncelikli"]
        L --> O["evaluate"]
        N --> O
    end
    J --> K
    J --> M
    O --> P{"ATS işareti?"}
    P -->|"ATS > 0"| Q["best_level SH<br>direnç"]
    P -->|"ATS < 0"| R["best_level SL<br>destek"]
    P -->|"ATS = 0"| S["Nötr, sinyal yok"]
    Q --> T{"price > SH?"}
    T -->|"evet"| U["📡 SİNYAL: BUY<br>sembol + yön"]
    T -->|"hayır"| V["kırılım yok"]
    R --> W{"price < SL?"}
    W -->|"evet"| X["📡 SİNYAL: SELL<br>sembol + yön"]
    W -->|"hayır"| V
    U --> Y["konsola bas<br>emir açılmaz (sinyal üretici)"]
    X --> Y
    Y -.->|"sonraki katman<br>execution / paper-service"| Z["emir oluşturma"]
```

### `breakout-strategy/src/lib.rs`
**Detaylı açıklama:** Kutunun kütüphane köküdür; tek `pub mod metrics;` ifadesiyle mikro-yapı metrik çekirdeğini binary'lere (özellikle `listener.rs`) açık hale getirir. main.rs doğrudan binary olarak çalıştığı için bu lib yalnızca paylaşılan metrik modülünü barındırır.
**Neden kullandık:** Metrik çekirdeğinin hem `listener` hem potansiyel diğer tüketicilerce tek yerden kullanılmasını sağlamak; lib/bin ayrımıyla test ve kütüphane dışa aktarımını kolaylaştırmak.

```mermaid
flowchart LR
    A["lib.rs"] --> B["metrics.rs<br>SymbolMetrics + CorrSeries + normalized_corr"]
    B --> C["listener.rs (bin)"]
```

### `breakout-strategy/src/metrics.rs`
**Detaylı açıklama:** Kurumsal tick-by-tick mikro-yapı analiz çekirdeğidir. Her `process_tick` çağrısı 7 aşamayı sırayla işler: (0) Lee-Ready işaretleme mid'e göre trade yönünü bulur; (1) WLOBI (üstel ağırlıklı emir dengesizliği) ve quote slope likidite mimarisini ölçer; (2) EffDelta + saniyelik delta velocity agresif akışı ölçer; (3) Absorption Ratio pasif alım / agresif satış oranını verir; (4) aVPIN dinamik Parkinson volatilite bazlı hacim bucket'larıyla akış toksisitesini hesaplar; (5) Hasbrouck VAR (OLS: r = α1·x + α2·r_prev) kalıcı/geçici etkiyi ve EfP agresif trade / L2 derinlik oranını verir; (6) Alpha Basket bu metrikleri z-skor benzeri normalleştirip `γ` ağırlıklarıyla birleştirir, lojistik `p_long` üretir ve kesin karar kuralıyla (aVPIN≥0.6 → nötr, p_long>0.65 → +1, <0.35 → −1) `signal` döndürür. Tüm parametreler (`MetricsConfig`) `/tmp/listener_metrics.conf` dosyasından hot-reload edilir.
**Neden kullandık:** Kurumsal yayın metodolojilerini tek sembol durumunda akümüle edip tek `refresh` ile tazelemek; çekirdek ve korelasyon ayrımı (CorrSeries + normalized Pearson) ile çok sembollü panel kurmak; tüm metrikleri karar sinyaliyle (Long/Short/Nötr) tek bir sayıya indirmek.

```mermaid
flowchart TD
    subgraph A["Aşama 0-1: Likidite"]
        B["process_tick<br>price, qty, is_buyer_maker"] --> C["Lee-Ready<br>mid'e göre işaret"]
        C --> D["WLOBI +<br>quote slope"]
    end
    subgraph E["Aşama 2-3: Akış"]
        C --> F["EffDelta +<br>ΔV (velocity)"]
        C --> G["Absorption +<br>IDM"]
    end
    subgraph H["Aşama 4-5: Toksisite"]
        C --> I["aVPIN<br>Parkinson vol + bucket"]
        C --> J["Hasbrouck VAR<br>kalıcı/geçici etki"]
        J --> K["EfP<br>qty / L2 derinlik"]
    end
    D --> L["Alpha Basket<br>γ ağırlıkları"]
    F --> L
    G --> L
    I --> L
    K --> L
    L --> M["p_long lojistik"]
    M --> N{"Karar kuralı?"}
    N -->|"aVPIN >= 0.6"| O["signal = 0<br>toksik, pasif"]
    N -->|"p_long > 0.65"| P["signal = +1<br>▲ LONG"]
    N -->|"p_long < 0.35"| Q["signal = -1<br>▼ SHORT"]
    N -->|"diğer"| R["signal = 0<br>Nötr"]
```

### `breakout-strategy/src/bin/listener.rs`
**Detaylı açıklama:** Veri merkezi izleyici binary'sidir. Ana thread DATA ring'ini (`/dev/shm/cycle_finance_ring`) 160k kapasiteyle okur, `wire::decode` sonrası `alerts.toml` sembol listesinden (VELVETUSDT her zaman eklenir) geçenler için `SymbolMetrics` tutar; `Trade` → `process_tick` + hacim korelasyon serisi, `Orderbook` → ilk 5 kademe derinliği (`update_depth` + `refresh`) işler. Ayrı thread (`spawn_price_corr_thread`) `:3004/api/lastprice`'tan 200 ms'de bir fiyat çekip fiyat korelasyon serilerine yazar. Her 2 sn'de bir `render` iki korelasyon matrisini (fiyat ve hacim, normalize Pearson) + mikro-yapı metrik tablosunu çizer ve `/tmp/listener_metrics.json`'a yazar. Pencere süreleri conf dosyasından hot-reload edilir.
**Neden kullandık:** Trade + derinlik + REST fiyatını tek ekranda birleştirip semboller arası fiyat/hacim korelasyonunu normalize 0-1 çizdirmek; izleyiciyi ayrı binary yaparak ana stratejiyi etkilemeden çalıştırmak; konsol + JSON çift çıktı ile otomasyona uygunluk sağlamak.

```mermaid
flowchart TD
    subgraph A["Veri Kaynakları"]
        B["DATA ring<br>/dev/shm/cycle_finance_ring"] --> C["read_slot + wire::decode"]
        D[":3004<br>lastprice"] --> E["spawn_price_corr_thread<br>200ms periyot"]
    end
    C --> F{"Event türü?"}
    F -->|"Trade"| G["process_tick<br>Lee-Ready → tüm metrikler"]
    G --> H["vol_series<br>hacim CorrSeries"]
    F -->|"Orderbook"| I["ilk 5 kademe<br>update_depth + refresh"]
    E --> J["price_series<br>fiyat CorrSeries"]
    G --> K["SymbolMetrics<br>TPS/WLOBI/EFFΔ/aVPIN/..."]
    I --> K
    K --> L{"2 sn doldu mu?"}
    L -->|"evet"| M["render<br>metrik tablosu"]
    M --> N["fiyat korelasyon matrisi<br>normalized_corr 0-1"]
    M --> O["hacim korelasyon matrisi"]
    M --> P["/tmp/listener_metrics.json<br>yaz"]
    L -->|"hayır"| C
```

### `breakout-strategy/src/bin/alerts.rs`
**Detaylı açıklama:** `alerts.toml`'u yöneten CLI aracıdır. `parse_blocks` dosyayı satır satır okuyup `[[alerts]]` bloklarını `AlertBlock` yapısına ayrıştırır (başlık satırlarını korur). Komutlar: `list` mevcut alarmları tablo halinde gösterir; `add` --symbol/--condition/--price (opsiyonel --tolerance/--voice/--cooldown) ile yeni blok ekler; `update` --old-price eşleşen bloğu günceller; `remove` eşleşen bloğu siler. `write_config` başlığı koruyup blokları `render_block` ile geri yazar; semboller büyük, koşullar küçük harfe normalleştirilir.
**Neden kullandık:** Alarm yönetimini toml parse kütüphanesine bağımlı olmadan küçük bir CLI ile otomasyona açmak; Python muadili (`scripts/alerts_cli.py`) ile işlev eşitliği sağlarken ek binary derlemesi sunmak; yanlış eşleşmede hata kodu (exit 1) ile güvenli işlem yapmak.

```mermaid
flowchart TD
    A["main<br>argv komut"] --> B{"Komut?"}
    B -->|"list"| C["parse_blocks + listele"]
    B -->|"add"| D["parametre doğrula<br>--symbol/--condition/--price"]
    B -->|"update"| E["--old-price eşle<br>blok bul"]
    B -->|"remove"| F["filtrele<br>eşleşen bloğu sil"]
    D --> G["normalize<br>sym büyük / cond küçük"]
    G --> H["blok ekle"]
    H --> I["write_config<br>header korunur"]
    E --> I
    F --> I
    C --> J["konsol tablosu"]
    I --> J
    E --> K{"eşleşme yok?"}
    K -->|"evet"| L["exit 1"]
    F --> K
```

### `breakout-strategy/src/bin/risk_analysis.rs`
**Detaylı açıklama:** TimescaleDB (`sqlx`) içindeki `trades` tablosunu tek SQL ile özetler: sembol bazında işlem sayısı, hacim (Σ price×qty), min/max fiyat; `COUNT(*) > 50` filtresiyle en çok işlem gören 15 pariteyi hacme göre sıralı gösterir. Volatilite yüzdesi `(max−min)/min×100` formülüyle hesaplanır ve en yüksek riskli 10 parite ayrı tabloda listelenir. `--watch` modunda ekran temizlenmeden imleç başa alınarak `WATCH_SEC` (varsayılan 5 sn) periyodla yenilenir (tmux RISK paneli için titreşimsiz). Bağlantı `TIMESCALEDB_URL` (varsayılan `postgres://cycle:cycle@localhost:5432/market_data`) üzerinden `sqlx::postgres::PgPool` ile yapılır.
**Neden kullandık:** Ham trade verisini SQL toplamıyla hızlı özetleyip risk dağılımını tablolaştırmak; `--watch` ile tmux'ta canlı panel olarak kullanmak; veri yetersizse sessizce uyarı verip uygulamayı çökertmemek.

```mermaid
flowchart TD
    A["main"] --> B{"--watch mı?"}
    B -->|"hayır"| C["render"]
    B -->|"evet"| D["temizle + render<br>imleç başta"]
    C --> E["SQL GROUP BY symbol<br>count, volume, min, max"]
    E --> F["cnt > 50 filtre<br>ORDER BY volume DESC"]
    F --> G["volatilite %<br>max-min üzerinden"]
    G --> H["en çok işlem gören<br>15 parite"]
    G --> I["en yüksek risk<br>10 parite"]
    D --> J["sleep WATCH_SEC<br>varsayılan 5 sn"]
    J --> D
```

---

## Özet
- **Analiz edilen dosya sayısı:** 10 (2 × Cargo.toml, mod.rs, trait_def.rs, lib.rs, main.rs, metrics.rs, listener.rs, alerts.rs, risk_analysis.rs)
- **Mermaid diyagramı sayısı:** 10 (her dosya için 1)
- **Ayrıca incelenen referanslar:** `transport::ring_buffer` (MarketDataSlot, GenerationalRingBuffer), `contracts::events` (EventType, wire::decode) — motorun akışını doğrulamak için.

---

## 📄 Tam Kaynak Kodu

### `strategies-engine/Cargo.toml`

```toml
[package]
name = "strategies-engine"
version = "0.1.0"
edition = "2021"

[lib]
path = "mod.rs"

[dependencies]
transport = { path = "../cycle-engine/transport" }
rust_decimal = { workspace = true }
```

### `strategies-engine/mod.rs`

```rust
pub mod trait_def;
```

### `strategies-engine/trait_def.rs`

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

### `strategies-engine/breakout-strategy/Cargo.toml`

```toml
[package]
name = "breakout-strategy"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "breakout-strategy"
path = "src/main.rs"

[dependencies]
tokio = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
sqlx = { workspace = true }
contracts = { path = "../../cycle-engine/contracts" }
transport = { path = "../../cycle-engine/transport" }
rust_decimal = { workspace = true }
```

### `strategies-engine/breakout-strategy/src/lib.rs`

```rust
//! velvetusdt — VELVETUSDT stratejisi + mikro-yapı metrik çekirdeği.

pub mod metrics;
```

### `strategies-engine/breakout-strategy/src/main.rs`

```rust
//! BREAKOUT Kırılım Stratejisi (Rust) — Event-Driven Sürüm
//!
//! Mimari (Katman 5: Strateji): **Actor + olay güdümlü**. Eski sürüm 20 dakikada
//! bir REST polling ile uyanıyordu; bu sürüm fiyatı ring'inden
//! **event-by-event** alır, değerlendirmeyi bekleme aralığında otomatik daya
//! (varsayılan 20 dakika, `/tmp/breakout_wait_sec.txt` ile dinamik).
//!
//! **Sinyal üretici mod**: Emir AÇMAZ. Sadece kırılım algılandığında
//! sembol + yön (BUY/SELL) bilgisini üretir.
//!
//! Akış:
//! ```text
//! ring (/cycle_finance_pricefeed)
//!   → ring okuyucu std thread (fiyat event'leri)
//!   → mpsc UnboundedChannel → [actor döngüsü]
//!                                ├─ fiyat anlık güncel (bekleme aralığında bile)
//!                                └─ bekleme aralığı dolmuşsa değerlendirme:
//!                                   detect-ms (:3002) → kırılım → sinyal (sembol+yön)
//! ```

use contracts::events::{EventType, OwnedEvent};
use contracts::wire;
use rust_decimal::prelude::*;
use serde_json::Value;
use std::env;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use transport::ring_buffer::GenerationalRingBuffer;

const DETECT_MS_URL: &str = "http://127.0.0.1:3002";
const PRICE_FEED_URL: &str = "http://127.0.0.1:3004";
const WAIT_FILE: &str = "/tmp/breakout_wait_sec.txt";
/// Ring'de yeni event yoksa uyanma sınırı — döngü asla tamamen uykuda kalmaz.
const WAKE_INTERVAL: Duration = Duration::from_millis(500);

struct Config {
    symbol: String,
    interval: String,
    limit: usize,
    wait_sec: u64,
    once: bool,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn load_config() -> Config {
    let check_every: usize = env_or("BREAKOUT_CHECK_EVERY", "20").parse().unwrap_or(20);
    let wait_sec: u64 = env_or("BREAKOUT_WAIT_SEC", &(check_every * 60).to_string())
        .parse()
        .unwrap_or((check_every * 60) as u64);
    let args: Vec<String> = env::args().collect();
    Config {
        symbol: env_or("BREAKOUT_SYMBOL", "VELVETUSDT"),
        interval: env_or("BREAKOUT_INTERVAL", "1m"),
        limit: env_or("BREAKOUT_LIMIT", "100").parse().unwrap_or(100),
        wait_sec,
        once: args.iter().any(|a| a == "--once"),
    }
}

// ── HTTP yardımcıları ────────────────────────────────────────
async fn http_get(client: &reqwest::Client, url: &str) -> Value {
    match client.get(url).send().await {
        Ok(r) => r.json::<Value>().await.unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

async fn fetch_analysis(client: &reqwest::Client, cfg: &Config) -> Value {
    let url = format!(
        "{DETECT_MS_URL}/api/ms?symbol={}&interval={}&limit={}",
        cfg.symbol, cfg.interval, cfg.limit
    );
    http_get(client, &url).await
}

async fn fetch_price_feed(client: &reqwest::Client, cfg: &Config) -> (Option<f64>, Option<String>) {
    let url = format!("{PRICE_FEED_URL}/api/lastprice/{}", cfg.symbol);
    let v = http_get(client, &url).await;
    if v.get("error").is_some() {
        return (None, v.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()));
    }
    if let Some(p) = v.pointer("/price") {
        for key in ["last", "mark", "index", "ask"] {
            if let Some(f) = p.get(key).and_then(|x| x.as_f64()) {
                if f > 0.0 {
                    return (Some(f), None);
                }
            }
        }
    }
    (None, Some("'te fiyat yok".to_string()))
}

// ── Seviye seçimi ────────────────────────────────────────────
fn best_level(levels: &[Value], level_type: &str) -> Option<(f64, f64)> {
    levels
        .iter()
        .filter(|l| l.get("level_type").and_then(|x| x.as_str()) == Some(level_type))
        .filter_map(|l| {
            let price = l.get("price").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok())?;
            let score = l.get("priority_score").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            Some((price, score))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}

// ── Kırılım değerlendirme (saf fonksiyon — test edilebilir) ──
fn evaluate(data: &Value, price: f64) -> (Option<String>, String) {
    if data.get("error").is_some() {
        return (None, format!("detect-ms hatası: {}", data.get("error").unwrap()));
    }
    let levels = match data.get("levels").and_then(|l| l.as_array()) {
        Some(l) if !l.is_empty() => l,
        _ => return (None, "Seviye yok".to_string()),
    };

    let ats: f64 = data.get("ats").and_then(|a| a.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let trend = data.get("trend_label").and_then(|t| t.as_str()).unwrap_or("");
    let confluence = data.get("confluence_index").and_then(|c| c.as_str()).unwrap_or("");
    let log = format!("Fiyat={price:.6}  ATS={ats:.4}  Trend={trend}  Confluence=%{confluence}");

    if ats > 0.0 {
        match best_level(levels, "SH") {
            Some((lv, score)) => {
                if price > lv {
                    (Some("BUY".into()), format!("{log} | 🎯 DİRENC KIRILDI SH={lv} (skor:{score}) → BUY"))
                } else {
                    (None, format!("{log} | Direnc yukarı kırılmadı SH={lv}"))
                }
            }
            None => (None, format!("{log} | Direnc yok")),
        }
    } else if ats < 0.0 {
        match best_level(levels, "SL") {
            Some((lv, score)) => {
                if price < lv {
                    (Some("SELL".into()), format!("{log} | 🎯 DESTEK KIRILDI SL={lv} (skor:{score}) → SELL"))
                } else {
                    (None, format!("{log} | Destek aşağı kırılmadı SL={lv}"))
                }
            }
            None => (None, format!("{log} | Destek yok")),
        }
    } else {
        (None, format!("{log} | Nötr trend"))
    }
}

// ── Bekleme süresi (dinamik) ─────────────────────────────────
fn current_wait_sec(default: u64) -> u64 {
    if let Ok(content) = std::fs::read_to_string(WAIT_FILE) {
        if let Ok(v) = content.trim().parse::<u64>() {
            if v > 0 {
                return v;
            }
        }
    }
    default
}

// ── Ring okuyucu (Katman 2 trans sözleşmesi) ─────────────────
/// Price-feed ring'indeki ilgili sembolün fiyat event'lerini kanala basar.
fn spawn_price_reader(symbol: &str, tx: mpsc::UnboundedSender<f64>) {
    let symbol = symbol.to_ascii_uppercase();
    std::thread::spawn(move || {
        let gen_ring = GenerationalRingBuffer::with_name("/cycle_finance_pricefeed", 20_000);
        let mut cursor = gen_ring.get_head();
        let mut symbol_buf = [0u8; 16];
        let bytes = symbol.as_bytes();
        let len = bytes.len().min(16);
        symbol_buf[..len].copy_from_slice(&bytes[..len]);

        loop {
            match gen_ring.read_slot(cursor) {
                Some(slot) => {
                    if let Some(ev) = wire::decode(&slot.data[..slot.len as usize]) {
                        if ev.symbol == symbol_buf {
                            if let Some(price) = event_price(&ev) {
                                if tx.send(price).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    cursor += 1;
                }
                None => {
                    let head = gen_ring.get_head();
                    if head > cursor {
                        cursor = head; // üretici arayı kapattı
                    } else {
                        std::thread::sleep(std::time::Duration::from_micros(500));
                    }
                }
            }
        }
    });
}

/// Event'ten stratejinin kullanacağı tek fiyatı çıkarır (bridge ile aynı öncelik).
fn event_price(ev: &OwnedEvent) -> Option<f64> {
    match &ev.payload {
        EventType::Trade { price, .. } => price.to_f64(),
        EventType::BookTicker { best_ask_price, best_bid_price, .. } => {
            let ask = best_ask_price.to_f64()?;
            if ask > 0.0 {
                Some(ask)
            } else {
                let bid = best_bid_price.to_f64()?;
                (bid > 0.0).then_some(bid)
            }
        }
        EventType::FundingRate { mark_price, .. } => mark_price.to_f64(),
        _ => None,
    }
}

// ── Tek değerlendirme ────────────────────────────────────────
struct EvalOutcome {
    ok: bool,
    msg: String,
}

async fn analyze_once(client: &reqwest::Client, cfg: &Config, price_override: Option<f64>) -> EvalOutcome {
    let data = fetch_analysis(client, cfg).await;
    if data.get("error").is_some() {
        let e = data.get("error").unwrap();
        return EvalOutcome { ok: false, msg: format!("⚠️ detect-ms erişilemiyor: {e}") };
    }

    let (pf_price, pf_err) = fetch_price_feed(client, cfg).await;
    let price = price_override
        .filter(|p| *p > 0.0)
        .or(pf_price)
        .unwrap_or_else(|| {
            data.get("current_price").and_then(|c| c.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)
        });
    let (signal, msg) = evaluate(&data, price);
    let feed_tag = if price_override.is_some() { "ring" } else if pf_err.is_none() { "REST" } else { "detect-ms" };

    let Some(side) = signal else {
        return EvalOutcome { ok: true, msg: format!("{msg}") };
    };

    EvalOutcome {
        ok: true,
        msg: format!("📡 SİNYAL → Sembol: {} | Yön: {} (fiyat: {feed_tag}) | {msg}", cfg.symbol, side),
    }
}

#[tokio::main]
async fn main() {
    let cfg = load_config();
    println!("══════════════════════════════════════════════════");
    println!("  🎯 BREAKOUT KIRILIM STRATEJİSİ — EVENT-DRIVEN  ({} {})", cfg.symbol, cfg.interval);
    println!("  Pencere: {} | Bekleme: {} sn | Kaynak: ring", cfg.limit, cfg.wait_sec);
    println!("  detect-ms: {DETECT_MS_URL}");
    println!("  📡 MOD: Sinyal üretici (sembol + yön, emir AÇILMAZ)");
    println!("══════════════════════════════════════════════════");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client");

    if cfg.once {
        let r = analyze_once(&client, &cfg, None).await;
        println!("[{}] {}", timestamp(), r.msg);
        return;
    }

    // Event-driven döngü: fiyat anlık (ring), değerlendirme bekleme aralığında.
    let (tx, mut rx) = mpsc::unbounded_channel::<f64>();
    spawn_price_reader(&cfg.symbol, tx);

    let mut latest_price: Option<f64> = None;
    let mut last_eval = Instant::now() - Duration::from_secs(cfg.wait_sec);
    let mut startup = true;

    loop {
        let evt = tokio::time::timeout(WAKE_INTERVAL, rx.recv()).await;
        if let Ok(Some(p)) = evt {
            latest_price = Some(p);
        }

        let sec = current_wait_sec(cfg.wait_sec);
        if startup || last_eval.elapsed().as_secs() >= sec {
            last_eval = Instant::now();
            startup = false;

            let r = analyze_once(&client, &cfg, latest_price).await;
            println!("[{}] {}", timestamp(), r.msg);
            if !r.ok {
                println!("  🔄 10 sn sonra yeniden deneniyor...");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
            println!("  😴 {sec} sn ({:.1} dk) bekleniyor... (breakout-wait ile değişir)\n", sec as f64 / 60.0);
        }
    }
}

fn timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}
```

### `strategies-engine/breakout-strategy/src/metrics.rs`

```rust
//! Microstructure Metrics — kurumsal tick-by-tick metrik çekirdeği.
//!
//! Veri kaynağı: DATA MERKEZİ (`/dev/shm/cycle_finance_ring`). KULLANILMAZ.
//!
//! Aşamalar:
//!   0. Lee-Ready Signing (trade yönü)
//!   1. WLOBI + Quote Slope (likidite mimarisi)
//!   2. EffDelta + Delta Velocity (saldırgan akış)
//!   3. Absorption Ratio + Iceberg (pasif emilim)
//!   4. aVPIN (mikro-yapı toksisitesi)
//!   5. Hasbrouck VAR + EfP (kalıcı/geçici etki)
//!   6. Alpha Basket (lojistik sinyal)

use std::collections::VecDeque;

// ── Metrik parametreleri (Θ) — shell'den değiştirilebilir ─────
// /tmp/listener_metrics.conf dosyasından okunur (listenconfig komutu).
pub const CONFIG_FILE: &str = "/tmp/listener_metrics.conf";

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub lambda: f64,           // WLOBI decay
    pub theta_vol: f64,        // Delta velocity eşiği
    pub alpha_bucket: f64,     // aVPIN bucket sabiti
    pub k_abs: usize,          // absorption penceresi (trade)
    pub n_bucket: usize,       // aVPIN bucket sayısı
    pub ice_threshold: f64,    // IDM eşiği
    pub efp_threshold: f64,    // execution footprint eşiği
    pub noise_corr: f64,       // Lee-Ready gürültü filtresi
    pub delta_window_sec: usize, // ΔV penceresi (saniye)
    pub tps_window_sec: usize,   // TPS pencere (saniye)
    pub corr_price_window_sec: usize, // fiyat korelasyon penceresi (saniye)
    pub corr_vol_window_sec: usize,   // hacim korelasyon penceresi (saniye)
    pub gamma: [f64; 6],       // Alpha Basket ağırlıkları
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            lambda: 0.015,
            theta_vol: 2.5,
            alpha_bucket: 0.75,
            k_abs: 100,
            n_bucket: 50,
            ice_threshold: 1.2,
            efp_threshold: 0.05,
            noise_corr: 0.85,
            delta_window_sec: 60,
            tps_window_sec: 10,
            corr_price_window_sec: 5,
            corr_vol_window_sec: 5,
            gamma: [0.0, 0.4, -0.3, 0.5, 0.6, -0.35],
        }
    }
}

impl MetricsConfig {
    /// /tmp/listener_metrics.conf dosyasından parametreleri yükler.
    /// Format: key = value  (bir satırda bir parametre)
    pub fn load() -> Self {
        let mut cfg = Self::default();
        let content = match std::fs::read_to_string(CONFIG_FILE) {
            Ok(c) => c,
            Err(_) => return cfg,
        };
        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            let (k, v) = match t.split_once('=') {
                Some(x) => x,
                None => continue,
            };
            let k = k.trim();
            let v = v.trim();
            let f = |d: f64| v.parse::<f64>().unwrap_or(d);
            match k {
                "lambda" => cfg.lambda = f(cfg.lambda),
                "theta_vol" => cfg.theta_vol = f(cfg.theta_vol),
                "alpha_bucket" => cfg.alpha_bucket = f(cfg.alpha_bucket),
                "k_abs" => cfg.k_abs = v.parse::<usize>().unwrap_or(cfg.k_abs),
                "n_bucket" => cfg.n_bucket = v.parse::<usize>().unwrap_or(cfg.n_bucket),
                "ice_threshold" => cfg.ice_threshold = f(cfg.ice_threshold),
                "efp_threshold" => cfg.efp_threshold = f(cfg.efp_threshold),
                "noise_corr" => cfg.noise_corr = f(cfg.noise_corr),
                "delta_window_sec" => cfg.delta_window_sec = v.parse::<usize>().unwrap_or(cfg.delta_window_sec),
                "tps_window_sec" => cfg.tps_window_sec = v.parse::<usize>().unwrap_or(cfg.tps_window_sec),
                "corr_price_window_sec" => cfg.corr_price_window_sec = v.parse::<usize>().unwrap_or(cfg.corr_price_window_sec),
                "corr_vol_window_sec" => cfg.corr_vol_window_sec = v.parse::<usize>().unwrap_or(cfg.corr_vol_window_sec),
                "gamma0" => cfg.gamma[0] = f(cfg.gamma[0]),
                "gamma1" => cfg.gamma[1] = f(cfg.gamma[1]),
                "gamma2" => cfg.gamma[2] = f(cfg.gamma[2]),
                "gamma3" => cfg.gamma[3] = f(cfg.gamma[3]),
                "gamma4" => cfg.gamma[4] = f(cfg.gamma[4]),
                "gamma5" => cfg.gamma[5] = f(cfg.gamma[5]),
                _ => {}
            }
        }
        cfg
    }
}

// ── Derinlik kademesi ────────────────────────────────────────
#[derive(Debug, Clone, Copy, Default)]
pub struct DepthLevel {
    pub price: f64,
    pub qty: f64,
}

// ── Sembol başına metrik durumu ──────────────────────────────
pub struct SymbolMetrics {
    // Lee-Ready
    prev_price: f64,
    prev_prev_price: f64,
    prev_sign: i8,
    prev_delta: f64,
    // mid / spread
    mid: f64,
    avg_spread: f64,
    spread_count: u64,
    // order book (ilk 5 kademe)
    bids: [DepthLevel; 5],
    asks: [DepthLevel; 5],
    // EffDelta
    pub eff_delta: f64,
    eff_delta_hist: VecDeque<f64>, // saniyelik
    last_delta_time: u64,
    // Absorption
    trade_signs: VecDeque<(f64, i8)>, // (qty, sign)
    // aVPIN
    bucket_volume: f64,
    bucket_vbuy: VecDeque<f64>,
    bucket_vsell: VecDeque<f64>,
    last_park_high: f64,
    last_park_low: f64,
    // Hasbrouck VAR (son 200 örnek)
    var_r: VecDeque<f64>,
    var_x: VecDeque<f64>,
    // TPS (trade/saniye) — son tps_window_sec saniyedeki trade sayısı
    trade_times: VecDeque<u64>,
    pub tps: f64,
    // EfP
    last_depth_total: f64,
    // sonuçlar
    pub cfg: MetricsConfig,
    pub wlobi: f64,
    pub slope_ask: f64,
    pub slope_bid: f64,
    pub delta_velocity: f64,
    pub absorption: f64,
    pub idm: f64,
    pub avpin: f64,
    pub permanent_impact: f64,
    pub temporary_impact: f64,
    pub efp: f64,
    pub alpha_score: f64,
    pub p_long: f64,
    pub signal: i8, // +1 Long, -1 Short, 0 Nötr
}

impl Default for SymbolMetrics {
    fn default() -> Self {
        Self {
            prev_price: 0.0,
            prev_prev_price: 0.0,
            prev_sign: 0,
            prev_delta: 0.0,
            mid: 0.0,
            avg_spread: 0.0,
            spread_count: 0,
            bids: [DepthLevel::default(); 5],
            asks: [DepthLevel::default(); 5],
            eff_delta: 0.0,
            eff_delta_hist: VecDeque::new(),
            last_delta_time: 0,
            trade_signs: VecDeque::new(),
            bucket_volume: 0.0,
            bucket_vbuy: VecDeque::new(),
            bucket_vsell: VecDeque::new(),
            last_park_high: 0.0,
            last_park_low: f64::MAX,
            var_r: VecDeque::new(),
            var_x: VecDeque::new(),
            trade_times: VecDeque::new(),
            tps: 0.0,
            last_depth_total: 0.0,
            cfg: MetricsConfig::load(),
            wlobi: 0.0,
            slope_ask: 0.0,
            slope_bid: 0.0,
            delta_velocity: 0.0,
            absorption: 0.0,
            idm: 0.0,
            avpin: 0.0,
            permanent_impact: 0.0,
            temporary_impact: 0.0,
            efp: 0.0,
            alpha_score: 0.0,
            p_long: 0.5,
            signal: 0,
        }
    }
}

impl SymbolMetrics {
    /// Config dosyasını yeniden yükler (shell'den değiştirilen parametreleri uygular)
    pub fn reload_config(&mut self) {
        self.cfg = MetricsConfig::load();
        // Pencere sınırlarını yeni değerlere kırp
        while self.eff_delta_hist.len() > self.cfg.delta_window_sec {
            self.eff_delta_hist.pop_front();
        }
        while self.trade_signs.len() > self.cfg.k_abs {
            self.trade_signs.pop_front();
        }
        while self.bucket_vbuy.len() > self.cfg.n_bucket {
            self.bucket_vbuy.pop_front();
        }
        while self.bucket_vsell.len() > self.cfg.n_bucket {
            self.bucket_vsell.pop_front();
        }
    }
    // ══ AŞAMA 0: Lee-Ready Signing ═══════════════════════════
    pub fn lee_ready_sign(&mut self, price: f64) -> i8 {
        let mid = self.mid;
        let sign = if price > mid {
            1
        } else if price < mid {
            -1
        } else if self.prev_delta != 0.0 {
            self.prev_sign
        } else {
            // Tick rule: sign(P_t - P_{t-2})
            if price > self.prev_prev_price { 1 } else if price < self.prev_prev_price { -1 } else { 0 }
        } as i8;

        self.prev_delta = price - self.prev_price;
        self.prev_prev_price = self.prev_price;
        self.prev_price = price;
        self.prev_sign = sign;
        sign
    }

    // ══ Order book güncelleme (ilk 5 kademe) ═════════════════
    pub fn update_depth(&mut self, bids: &[DepthLevel], asks: &[DepthLevel]) {
        for i in 0..5 {
            self.bids[i] = bids.get(i).copied().unwrap_or_default();
            self.asks[i] = asks.get(i).copied().unwrap_or_default();
        }
        // Top of book → mid + spread
        let b0 = self.bids[0].price;
        let a0 = self.asks[0].price;
        if b0 > 0.0 && a0 > 0.0 {
            self.mid = (b0 + a0) / 2.0;
            let spread = a0 - b0;
            self.avg_spread = (self.avg_spread * self.spread_count as f64 + spread) / (self.spread_count + 1) as f64;
            self.spread_count += 1;
        }
        // EfP paydası: ilk 5 kademe toplam derinlik
        self.last_depth_total = self.bids.iter().map(|l| l.qty).sum::<f64>()
            + self.asks.iter().map(|l| l.qty).sum::<f64>();
    }

    // ══ AŞAMA 1: WLOBI ═══════════════════════════════════════
    pub fn compute_wlobi(&mut self) -> f64 {
        // ω_i = e^(-λ·i) — kademe derinliği yaşam süresi vekili
        let mut w_bid = 0.0;
        let mut w_ask = 0.0;
        for i in 0..5 {
            let w = (-self.cfg.lambda * (i as f64 + 1.0)).exp();
            w_bid += w * self.bids[i].qty;
            w_ask += w * self.asks[i].qty;
        }
        let denom = w_ask + w_bid;
        self.wlobi = if denom > 0.0 { (w_ask - w_bid) / denom } else { 0.0 };
        self.wlobi
    }

    // Quote Slope: (ln V1 - ln V5) / (P5 - P1)
    pub fn compute_slopes(&mut self) {
        let (v1a, v5a, p1a, p5a) = (
            self.asks[0].qty.max(1e-12),
            self.asks[4].qty.max(1e-12),
            self.asks[0].price,
            self.asks[4].price,
        );
        let (v1b, v5b, p1b, p5b) = (
            self.bids[0].qty.max(1e-12),
            self.bids[4].qty.max(1e-12),
            self.bids[0].price,
            self.bids[4].price,
        );
        self.slope_ask = if (p5a - p1a).abs() > 1e-12 { (v1a.ln() - v5a.ln()) / (p5a - p1a) } else { 0.0 };
        self.slope_bid = if (p5b - p1b).abs() > 1e-12 { (v1b.ln() - v5b.ln()) / (p5b - p1b) } else { 0.0 };
    }

    // ══ AŞAMA 2: EffDelta + Delta Velocity ═══════════════════
    pub fn update_eff_delta(&mut self, price: f64, qty: f64, sign: i8, ts_ms: u64) {
        let s_eff = 2.0 * (price - self.mid).abs();
        let s_bar = if self.avg_spread > 0.0 { self.avg_spread } else { s_eff.max(1e-12) };
        let delta_contribution = (sign as f64) * qty * (s_eff / s_bar);
        self.eff_delta += delta_contribution;

        // Saniyelik velocity
        let sec = ts_ms / 1000;
        if sec != self.last_delta_time {
            if self.eff_delta_hist.len() >= self.cfg.delta_window_sec {
                self.eff_delta_hist.pop_front();
            }
            self.eff_delta_hist.push_back(self.eff_delta);
            self.last_delta_time = sec;
        }
        if self.eff_delta_hist.len() >= 2 {
            let prev = *self.eff_delta_hist.get(self.eff_delta_hist.len() - 2).unwrap();
            let cur = *self.eff_delta_hist.back().unwrap();
            self.delta_velocity = cur - prev; // Δt = 1 sn
        }
    }

    // ══ AŞAMA 3: Absorption Ratio ════════════════════════════
    pub fn update_absorption(&mut self, qty: f64, sign: i8) {
        self.trade_signs.push_back((qty, sign));
        if self.trade_signs.len() > self.cfg.k_abs {
            self.trade_signs.pop_front();
        }
        let mut buy = 0.0;
        let mut sell = 0.0;
        for &(q, s) in &self.trade_signs {
            if s > 0 { buy += q; } else { sell += q; }
        }
        // Abs = pasif alım hacmi / agresif satış hacmi
        self.absorption = if sell > 0.0 { buy / sell } else { 0.0 };
    }

    // ══ AŞAMA 4: aVPIN ═══════════════════════════════════════
    pub fn update_avpin(&mut self, price: f64, qty: f64, sign: i8, ts_ms: u64) {
        // Parkinson H/L (son saniye içindeki max/min)
        let sec = ts_ms / 1000;
        if self.last_park_high == 0.0 {
            self.last_park_high = price;
            self.last_park_low = price;
        }
        if sec != self.last_delta_time {
            self.last_park_high = price;
            self.last_park_low = price;
        } else {
            self.last_park_high = self.last_park_high.max(price);
            self.last_park_low = self.last_park_low.min(price);
        }

        let h = self.last_park_high.max(price);
        let l = self.last_park_low.min(price);
        // Parkinson volatilitesi: sqrt(1/(4·ln2)) · sqrt(avg ln²(H/L))
        let parkinson = if h > 0.0 && l > 0.0 && h > l {
            let r = (h / l).ln();
            (1.0 / (4.0 * std::f64::consts::LN_2)).sqrt() * r.abs()
        } else {
            0.0
        };

        if sign > 0 {
            self.bucket_vbuy.push_back(qty);
        } else {
            self.bucket_vsell.push_back(qty);
        }
        if self.bucket_vbuy.len() > self.cfg.n_bucket {
            self.bucket_vbuy.pop_front();
        }
        if self.bucket_vsell.len() > self.cfg.n_bucket {
            self.bucket_vsell.pop_front();
        }

        // Ortalama trade hacmi (son 1000 trade, bucket listelerinden)
        let n_trades = (self.bucket_vbuy.len() + self.bucket_vsell.len()).max(1) as f64;
        let total_vol: f64 = self.bucket_vbuy.iter().sum::<f64>() + self.bucket_vsell.iter().sum::<f64>();
        let avg_vol = total_vol / n_trades;

        // Dinamik hacim bucket'ı: B_vol = α · σ_parkinson · V̄
        let b_vol = self.cfg.alpha_bucket * parkinson.max(1e-9) * avg_vol.max(1e-9);

        let sum_buy: f64 = self.bucket_vbuy.iter().sum();
        let sum_sell: f64 = self.bucket_vsell.iter().sum();
        let n = self.bucket_vbuy.len().max(self.bucket_vsell.len()).max(1) as f64;
        self.avpin = (sum_buy - sum_sell).abs() / (n * b_vol.max(1e-9));
    }

    // ══ AŞAMA 5: Hasbrouck VAR ═══════════════════════════════
    pub fn update_hasbrouck(&mut self, price: f64, qty: f64, sign: i8) {
        let r = price.ln() - self.prev_prev_price.ln().max(1e-12).ln();
        // Basitleştirme: r_t = ln(P_t) - ln(P_{t-1}); prev_price saklanır
        let r_prev = self.var_r.back().copied().unwrap_or(0.0);
        let x = (sign as f64) * qty;
        self.var_r.push_back(r);
        self.var_x.push_back(x);
        if self.var_r.len() > 200 {
            self.var_r.pop_front();
            self.var_x.pop_front();
        }

        if self.var_r.len() < 30 {
            return;
        }
        // OLS: r_t = α1·x_t + α2·r_{t-1} + ε
        let n = self.var_r.len();
        let (mut s_xx, mut s_xr, mut s_rr, mut s_yr, mut s_yx, mut s_yy) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for i in 1..n {
            let xi = self.var_x[i];
            let r_prev_i = self.var_r[i - 1];
            let yi = self.var_r[i];
            s_xx += xi * xi;
            s_xr += xi * r_prev_i;
            s_rr += r_prev_i * r_prev_i;
            s_yr += yi * r_prev_i;
            s_yx += yi * xi;
            s_yy += yi * yi;
        }
        let denom = s_xx * s_rr - s_xr * s_xr;
        if denom.abs() < 1e-15 {
            return;
        }
        let alpha1 = (s_yx * s_rr - s_yr * s_xr) / denom;
        let alpha2 = (s_yy * s_xx - s_yx * s_xr) / denom;
        // α2'yi regresyon katsayısı olarak düzelt (proxy)
        let _ = r_prev;
        self.permanent_impact = alpha1 / (1.0 - alpha2.max(-0.99).min(0.99)).max(1e-9);
        self.temporary_impact = self.var_r[n - 1] - alpha1 * self.var_x[n - 1] - alpha2 * self.var_r[n - 2];
    }

    // EfP: agresif trade / toplam L2 derinlik
    pub fn update_efp(&mut self, qty: f64) {
        self.efp = if self.last_depth_total > 0.0 { qty / self.last_depth_total } else { 0.0 };
    }

    // ══ AŞAMA 6: Alpha Basket ════════════════════════════════
    pub fn compute_signal(&mut self) -> i8 {
        // Z-skor standardizasyonu (ham değerler → normalize)
        let z_wlobi = (self.wlobi).tanh();
        let z_avpin = (self.avpin - 0.5) * 2.0;
        let z_abs = (self.absorption - 1.0).tanh();
        let z_effdelta = (self.eff_delta / 1000.0).tanh();
        let z_perm = (self.permanent_impact / 1e-6).tanh();

        // A_t = γ0 + γ1·(Abs-1) + γ2·(-WLOBI) + γ3·(0.7-aVPIN)
        //        + γ4·sign(-EffDelta)·1{|ΔV|<θ} - γ5·Perm
        let not_exhausted = (self.delta_velocity.abs() < self.cfg.theta_vol) as i32 as f64;
        let a = self.cfg.gamma[0]
            + self.cfg.gamma[1] * z_abs
            + self.cfg.gamma[2] * (-z_wlobi)
            + self.cfg.gamma[3] * (0.7 - z_avpin)
            + self.cfg.gamma[4] * (-z_effdelta).signum() * not_exhausted
            - self.cfg.gamma[5] * z_perm;

        self.alpha_score = a;
        self.p_long = 1.0 / (1.0 + (-a).exp());

        // Kesin karar kuralı
        if self.avpin >= 0.6 {
            self.signal = 0; // toksik akışta pasif kal
        } else if self.p_long > 0.65 {
            self.signal = 1;
        } else if self.p_long < 0.35 {
            self.signal = -1;
        } else {
            self.signal = 0;
        }
        self.signal
    }

    // Tüm metrikleri tek adımda tazele
    pub fn refresh(&mut self) {
        self.compute_wlobi();
        self.compute_slopes();
        self.compute_signal();
    }

    pub fn process_tick(&mut self, price: f64, qty: f64, is_buyer_maker: bool, ts_ms: u64) {
        let sign = self.lee_ready_sign(price);
        self.update_eff_delta(price, qty, sign, ts_ms);
        self.update_absorption(qty, sign);
        self.update_avpin(price, qty, sign, ts_ms);
        self.update_hasbrouck(price, qty, sign);
        self.update_efp(qty);
        self.update_tps(ts_ms);
        let _ = is_buyer_maker; // Lee-Ready yönü is_buyer_maker'ı aşar (mid'e göre)
        self.refresh();
    }

    // ══ TPS — saniyedeki trade sayısı ═════════════════════════
    fn update_tps(&mut self, ts_ms: u64) {
        self.trade_times.push_back(ts_ms);
        let window_ms = (self.cfg.tps_window_sec.max(1)) as u64 * 1000;
        while let Some(&t) = self.trade_times.front() {
            if ts_ms.saturating_sub(t) > window_ms {
                self.trade_times.pop_front();
            } else {
                break;
            }
        }
        let win = self.cfg.tps_window_sec.max(1) as f64;
        self.tps = self.trade_times.len() as f64 / win;
    }
}

// ══ Korelasyon serisi — pencere bazlı zaman serisi ═════════════
/// (ts_ms, value) çiftlerini pencere içinde tutar, normalize korelasyon (0-1) hesaplar.
#[derive(Debug, Clone)]
pub struct CorrSeries {
    pub points: VecDeque<(u64, f64)>,
    window_ms: u64,
}

impl CorrSeries {
    pub fn new(window_sec: usize) -> Self {
        Self {
            points: VecDeque::new(),
            window_ms: (window_sec.max(1) as u64) * 1000,
        }
    }

    /// pencere süresini güncelle ve eski noktaları kırp
    pub fn set_window(&mut self, window_sec: usize) {
        self.window_ms = (window_sec.max(1) as u64) * 1000;
        self.trim(now_ms());
    }

    pub fn push(&mut self, ts_ms: u64, value: f64) {
        self.points.push_back((ts_ms, value));
        self.trim(ts_ms);
    }

    fn trim(&mut self, ref_ts: u64) {
        while let Some(&(t, _)) = self.points.front() {
            if ref_ts.saturating_sub(t) > self.window_ms {
                self.points.pop_front();
            } else {
                break;
            }
        }
    }

    /// Değerleri (korelasyon için) pencere içindeki sırayla döndürür
    pub fn values(&self) -> Vec<f64> {
        self.points.iter().map(|&(_, v)| v).collect()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// İki seri arasında Pearson korelasyonu. Sonuç [0,1]'e normalize edilir: (r+1)/2.
/// Yetersiz veri (n<3 veya sabit seri) durumunda 0.0 (ilişkisiz) döndürülür.
pub fn normalized_corr(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n < 3 {
        return 0.0;
    }
    let (a, b) = (&a[..n], &b[..n]);
    let ma = a.iter().sum::<f64>() / n as f64;
    let mb = b.iter().sum::<f64>() / n as f64;
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    for i in 0..n {
        let dx = a[i] - ma;
        let dy = b[i] - mb;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx < 1e-12 || syy < 1e-12 {
        return 0.0;
    }
    let r = sxy / (sxx.sqrt() * syy.sqrt());
    (r + 1.0) / 2.0
}
```

### `strategies-engine/breakout-strategy/src/bin/alerts.rs`

```rust
//! alerts.toml yönetim aracı (Rust) — Python karşılığı: scripts/alerts_cli.py
//!
//! Kullanım:
//!   alerts list
//!   alerts add --symbol VELVETUSDT --condition above --price 0.22 [--voice "..."] [--cooldown 30] [--tolerance 0.0005]
//!   alerts update --symbol VELVETUSDT --condition above --old-price 0.21628 [--price 0.22] [--voice "..."] [--cooldown 30]
//!   alerts remove --symbol VELVETUSDT --condition above --price 0.21628

use std::process::exit;

const CONFIG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../alerts.toml");

// ── Basit blok ayrıştırma ────────────────────────────────────
#[derive(Debug, Clone)]
struct AlertBlock {
    symbol: String,
    condition: String,
    price: String,
    tolerance: Option<String>,
    voice: Option<String>,
    cooldown: Option<String>,
}

fn norm_price(v: &str) -> String {
    match v.trim().parse::<f64>() {
        Ok(f) => format!("{}", f),
        Err(_) => v.trim().to_string(),
    }
}

fn parse_blocks(content: &str) -> (Vec<String>, Vec<AlertBlock>) {
    let mut header = Vec::new();
    let mut blocks = Vec::new();
    let mut cur: Option<AlertBlock> = None;

    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("[[alerts]]") {
            if let Some(b) = cur.take() {
                blocks.push(b);
            }
            cur = Some(AlertBlock {
                symbol: String::new(),
                condition: String::new(),
                price: String::new(),
                tolerance: None,
                voice: None,
                cooldown: None,
            });
            continue;
        }
        match &mut cur {
            Some(b) => {
                if t.starts_with("symbol") {
                    b.symbol = val_of(t);
                } else if t.starts_with("condition") {
                    b.condition = val_of(t);
                } else if t.starts_with("price") {
                    b.price = val_of(t);
                } else if t.starts_with("tolerance_pct") {
                    b.tolerance = Some(val_of(t));
                } else if t.starts_with("voice") {
                    b.voice = Some(val_of(t));
                } else if t.starts_with("cooldown_sec") {
                    b.cooldown = Some(val_of(t));
                }
            }
            None => header.push(line.to_string()),
        }
    }
    if let Some(b) = cur.take() {
        blocks.push(b);
    }
    (header, blocks)
}

fn val_of(line: &str) -> String {
    let (_, v) = line.split_once('=').unwrap_or(("", ""));
    v.trim().trim_matches('"').trim_matches('\'').trim().to_string()
}

fn render_block(b: &AlertBlock) -> String {
    let mut out = String::from("[[alerts]]\n");
    out.push_str(&format!("symbol = \"{}\"\n", b.symbol));
    out.push_str(&format!("condition = \"{}\"\n", b.condition));
    out.push_str(&format!("price = {}\n", norm_price(&b.price)));
    if let Some(t) = &b.tolerance {
        out.push_str(&format!("tolerance_pct = {}\n", norm_price(t)));
    }
    if let Some(v) = &b.voice {
        out.push_str(&format!("voice = \"{}\"\n", v));
    }
    if let Some(c) = &b.cooldown {
        out.push_str(&format!("cooldown_sec = {}\n", c));
    }
    out
}

fn write_config(header: &[String], blocks: &[AlertBlock]) {
    let mut out = header.join("\n");
    if !out.is_empty() && !blocks.is_empty() {
        out.push('\n');
    }
    if !blocks.is_empty() {
        let rendered: Vec<String> = blocks.iter().map(render_block).collect();
        out.push_str(&rendered.join("\n"));
        out.push('\n');
    }
    std::fs::write(CONFIG, out).expect("alerts.toml yazılamadı");
}

// ── Komutlar ────────────────────────────────────────────────
fn cmd_list() {
    let content = std::fs::read_to_string(CONFIG).unwrap_or_default();
    let (_, blocks) = parse_blocks(&content);
    if blocks.is_empty() {
        println!("  📭 Alarmsız");
        return;
    }
    for (i, b) in blocks.iter().enumerate() {
        let voice = b.voice.clone().unwrap_or_default();
        let vdesc = if voice.is_empty() { "🔊 beep".to_string() } else { format!("🗣️ {voice}") };
        let tol = b.tolerance.clone().unwrap_or_else(|| "-".into());
        let cd = b.cooldown.clone().unwrap_or_else(|| "-".into());
        println!(
            "  [{}] {:<9} {:<6} fiyat={:<10} tol={} cooldown={}s {}",
            i + 1, b.symbol, b.condition, b.price, tol, cd, vdesc
        );
    }
}

fn cmd_add(args: &[String]) {
    let sym = arg(&args, "--symbol").unwrap_or_else(|| die("--symbol gerekli"));
    let cond = arg(&args, "--condition").unwrap_or_else(|| die("--condition gerekli"));
    let price = arg(&args, "--price").unwrap_or_else(|| die("--price gerekli"));
    let content = std::fs::read_to_string(CONFIG).unwrap_or_default();
    let (header, mut blocks) = parse_blocks(&content);
    blocks.push(AlertBlock {
        symbol: sym.to_uppercase(),
        condition: cond.to_lowercase(),
        price: price.to_string(),
        tolerance: arg(&args, "--tolerance"),
        voice: arg(&args, "--voice"),
        cooldown: Some(arg(&args, "--cooldown").unwrap_or_else(|| "30".to_string())),
    });
    write_config(&header, &blocks);
    println!("✅ Eklendi: {} {} {}", sym.to_uppercase(), cond, price);
}

fn cmd_update(args: &[String]) {
    let sym = arg(&args, "--symbol").unwrap_or_else(|| die("--symbol gerekli")).to_uppercase();
    let cond = arg(&args, "--condition").unwrap_or_else(|| die("--condition gerekli")).to_lowercase();
    let old = arg(&args, "--old-price").unwrap_or_else(|| die("--old-price gerekli"));
    let content = std::fs::read_to_string(CONFIG).unwrap_or_default();
    let (header, mut blocks) = parse_blocks(&content);
    let target = norm_price(&old);

    for b in blocks.iter_mut() {
        if b.symbol == sym && b.condition == cond && norm_price(&b.price) == target {
            if let Some(p) = arg(&args, "--price") {
                b.price = p.to_string();
            }
            if let Some(v) = arg(&args, "--voice") {
                b.voice = Some(v.to_string());
            }
            if let Some(c) = arg(&args, "--cooldown") {
                b.cooldown = Some(c.to_string());
            }
            if let Some(t) = arg(&args, "--tolerance") {
                b.tolerance = Some(t.to_string());
            }
            write_config(&header, &blocks);
            println!("✅ Güncellendi: {sym} {cond}");
            return;
        }
    }
    eprintln!("❌ Alarm bulunamadı: {sym} {cond} {old}");
    exit(1);
}

fn cmd_remove(args: &[String]) {
    let sym = arg(&args, "--symbol").unwrap_or_else(|| die("--symbol gerekli")).to_uppercase();
    let cond = arg(&args, "--condition").unwrap_or_else(|| die("--condition gerekli")).to_lowercase();
    let price = arg(&args, "--price").unwrap_or_else(|| die("--price gerekli"));
    let content = std::fs::read_to_string(CONFIG).unwrap_or_default();
    let (header, blocks) = parse_blocks(&content);
    let target = norm_price(&price);
    let before = blocks.len();
    let kept: Vec<AlertBlock> = blocks
        .into_iter()
        .filter(|b| !(b.symbol == sym && b.condition == cond && norm_price(&b.price) == target))
        .collect();
    if kept.len() == before {
        eprintln!("❌ Alarm bulunamadı: {sym} {cond} {target}");
        exit(1);
    }
    write_config(&header, &kept);
    println!("✅ Silindi: {sym} {cond} {target}");
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("list");
    match cmd {
        "list" => cmd_list(),
        "add" => cmd_add(&args[1..]),
        "update" => cmd_update(&args[1..]),
        "remove" => cmd_remove(&args[1..]),
        _ => {
            eprintln!("Kullanım: alerts list|add|update|remove");
            exit(1);
        }
    }
}
```

### `strategies-engine/breakout-strategy/src/bin/listener.rs`

```rust
//! LISTENER — DATA MERKEZİ mikro-yapı metrikleri + korelasyon tabloları (Rust).
//!
//! Veri kaynakları:
//!   - DATA MERKEZİ (core RUN_MODE=DATA → `/dev/shm/cycle_finance_ring`): trade/depth + hacim
//!   - PRICE-FEED (:3004): lastprice (fiyat korelasyonu için)
//!
//! Ekran:
//!   1. Mikro-yapı metrik tablosu (TPS, WLOBI, EffΔ, aVPIN, Hasbrouck, EfP, sinyal)
//!   2. Fiyat korelasyon tablosu (lastprice, N sn pencere, normalize 0-1)
//!   3. Hacim korelasyon tablosu (DATA trade hacmi, N sn pencere, normalize 0-1)
//!
//! Pencere süreleri shell'den ayarlanabilir (listenconfig-set):
//!   corr_price_window_sec, corr_vol_window_sec
//!
//! Çıktılar: konsol + /tmp/listener_metrics.json

use breakout_strategy::metrics::{normalized_corr, CorrSeries, DepthLevel, SymbolMetrics};
use rust_decimal::prelude::ToPrimitive;

use transport::ring_buffer::GenerationalRingBuffer;
use contracts::events::EventType;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const OUT_FILE: &str = "/tmp/listener_metrics.json";
const REFRESH_MS: u64 = 2000;
const PRICE_FEED_URL: &str = "http://127.0.0.1:3004";

fn decode_symbol(buf: &[u8; 16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(16);
    String::from_utf8_lossy(&buf[..len]).to_string().to_uppercase()
}

///'ten periyodik lastprice çeker ve CorrSeries'e yazar.
fn spawn_price_corr_thread(symbols: Vec<String>, series: Arc<Mutex<HashMap<String, CorrSeries>>>) {
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .expect("reqwest client");
        loop {
            let url = format!("{PRICE_FEED_URL}/api/lastprice");
            if let Ok(resp) = client.get(&url).send() {
                if let Ok(v) = resp.json::<serde_json::Value>() {
                    if let Some(prices) = v.get("prices").and_then(|p| p.as_object()) {
                        let now = now_ms();
                        let mut s = series.lock().unwrap();
                        for sym in &symbols {
                            if let Some(p) = prices.get(sym).and_then(|x| x.get("last")).and_then(|x| x.as_f64()) {
                                let e = s.entry(sym.clone()).or_insert_with(|| CorrSeries::new(5));
                                e.push(now, p);
                            }
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    });
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn main() {
    println!("{}", "═".repeat(96));
    println!("  🛰️  LISTENER — MİKRO-YAPI METRİKLERİ + KORELASYON");
    println!("  Kaynak: DATA (/dev/shm/cycle_finance_ring) + PRICE-FEED (:3004)");
    println!("{}", "═".repeat(96));

    let ring = Arc::new(GenerationalRingBuffer::new(160_000));
    let mut cursor = ring.get_head();
    let mut symbols: HashMap<String, SymbolMetrics> = HashMap::new();

    let known: Vec<String> = load_symbols();

    // Fiyat korelasyon serileri ()
    let price_series: Arc<Mutex<HashMap<String, CorrSeries>>> = Arc::new(Mutex::new(HashMap::new()));
    spawn_price_corr_thread(known.clone(), price_series.clone());

    // Hacim korelasyon serileri (DATA trade) — sembol → (pencere, değer)
    let vol_series: Arc<Mutex<HashMap<String, CorrSeries>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut last_render = std::time::Instant::now();
    let mut tick_count: u64 = 0;
    let mut depth_count: u64 = 0;

    loop {
        if let Some(slot) = ring.read_slot(cursor) {
            if let Some(event) = contracts::wire::decode(&slot.data[..slot.len as usize]) {
                let sym = decode_symbol(&event.symbol);
                if !known.iter().any(|k| k == &sym) {
                    cursor += 1;
                    continue;
                }
                let m = symbols.entry(sym.clone()).or_default();

                match event.payload {
                    EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
                        let p = price.to_f64().unwrap_or(0.0);
                        let q = quantity.to_f64().unwrap_or(0.0);
                        m.process_tick(p, q, is_buyer_maker, timestamp);
                        // Hacim korelasyonu: trade hacmini pencereye ekle (biriken değer)
                        {
                            let mut vs = vol_series.lock().unwrap();
                            let e = vs.entry(sym.clone()).or_insert_with(|| CorrSeries::new(5));
                            e.push(now_ms(), q);
                        }
                        tick_count += 1;
                    }
                    EventType::Orderbook { bids, asks } => {
                        let bids_l: Vec<DepthLevel> = bids.iter().take(5)
                            .map(|(p, q)| DepthLevel { price: p.to_f64().unwrap_or(0.0), qty: q.to_f64().unwrap_or(0.0) })
                            .collect();
                        let asks_l: Vec<DepthLevel> = asks.iter().take(5)
                            .map(|(p, q)| DepthLevel { price: p.to_f64().unwrap_or(0.0), qty: q.to_f64().unwrap_or(0.0) })
                            .collect();
                        depth_count += 1;
                        m.update_depth(&bids_l, &asks_l);
                        m.refresh();
                    }
                    _ => {}
                }
            }
            cursor += 1;
        } else {
            std::thread::sleep(Duration::from_micros(50));
        }

        if last_render.elapsed().as_millis() as u64 >= REFRESH_MS {
            for m in symbols.values_mut() {
                m.reload_config();
                // korelasyon pencere sürelerini uygula
                let (pw, vw) = (m.cfg.corr_price_window_sec, m.cfg.corr_vol_window_sec);
                {
                    let mut ps = price_series.lock().unwrap();
                    for e in ps.values_mut() {
                        e.set_window(pw);
                    }
                }
                {
                    let mut vs = vol_series.lock().unwrap();
                    for e in vs.values_mut() {
                        e.set_window(vw);
                    }
                }
            }
            render(&symbols, &price_series, &vol_series, tick_count, depth_count);
            tick_count = 0;
            depth_count = 0;
            last_render = std::time::Instant::now();
        }
    }
}

fn load_symbols() -> Vec<String> {
    let mut syms: Vec<String> = Vec::new();
    if let Ok(content) = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../alerts.toml")) {
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

/// Fiyat/hacim korelasyon matrisini çizer (normalize 0-1).
fn render_corr(title: &str, symbols: &[String], series: &Arc<Mutex<HashMap<String, CorrSeries>>>) {
    let s = series.lock().unwrap();
    println!("  {title}");
    println!("  {:<9}", "");
    for sym in symbols {
        print!("{:>10}", short(sym));
    }
    println!();
    for a in symbols {
        print!("  {:<9}", short(a));
        let av = s.get(a).map(|x| x.values()).unwrap_or_default();
        for b in symbols {
            let bv = s.get(b).map(|x| x.values()).unwrap_or_default();
            let c = normalized_corr(&av, &bv);
            print!("{:>10.2}", c);
        }
        println!();
    }
    println!();
}

fn short(s: &str) -> String {
    s.trim_end_matches("USDT").to_string()
}

fn render(symbols: &HashMap<String, SymbolMetrics>,
          price_series: &Arc<Mutex<HashMap<String, CorrSeries>>>,
          vol_series: &Arc<Mutex<HashMap<String, CorrSeries>>>,
          ticks: u64, depth: u64) {
    print!("\x1b[2J\x1b[H");
    println!("{}", "═".repeat(96));
    println!("  🛰️  LISTENER — MİKRO-YAPI METRİKLERİ + KORELASYON");
    println!("  DATA tick/s: {ticks} | depth/s: {depth} |: :3004");
    println!("{}", "═".repeat(96));

    if symbols.is_empty() {
        println!("  📭 VERİ BEKLENİYOR — DATA terminali çalışıyor mu?");
        return;
    }

    // ── Mikro-yapı metrik tablosu ──
    println!("  {:<9}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>7}{:>8}{:>8}{:>8}",
        "SEMBOL", "TPS", "WLOBI", "SLP", "EFFΔ", "ΔV", "ABS", "aVPIN", "PERM", "EfP", "P(LONG)", "SİNYAL");
    println!("  {}", "-".repeat(96));
    let mut rows: Vec<(&String, &SymbolMetrics)> = symbols.iter().collect();
    rows.sort_by_key(|(k, _)| k.clone());
    for (sym, m) in rows {
        let signal = match m.signal {
            1 => "▲ LONG",
            -1 => "▼ SHORT",
            _ => "· NÖTR",
        };
        println!(
            "  {:<9}{:>8.1}{:>8.3}{:>8.2}{:>8.2}{:>8.2}{:>8.2}{:>8.3}{:>8.1e}{:>7.3}{:>8.3}{:>8}",
            sym, m.tps, m.wlobi, m.slope_ask, m.eff_delta, m.delta_velocity,
            m.absorption, m.avpin, m.permanent_impact, m.efp, m.p_long, signal
        );
    }
    println!();

    // ── Fiyat korelasyonu (lastprice) ──
    let sym_list: Vec<String> = {
        let mut v: Vec<String> = symbols.keys().cloned().collect();
        v.sort();
        v
    };
    render_corr(&format!("📈 FİYAT KORELASYONU (lastprice)"),
                &sym_list, price_series);
    render_corr(&format!("📊 HACİM KORELASYONU (DATA trade hacmi)"),
                &sym_list, vol_series);

    println!("{}", "-".repeat(96));
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    println!("  Son güncelleme: {now} | listenconfig-set corr_price_window_sec / corr_vol_window_sec ile pencere değiştir");

    // ── JSON çıktısı ──
    let mut out = serde_json::Map::new();
    for (sym, m) in &*symbols {
        out.insert(sym.clone(), json!({
            "tps": m.tps,
            "wlobi": m.wlobi,
            "slope_ask": m.slope_ask,
            "slope_bid": m.slope_bid,
            "eff_delta": m.eff_delta,
            "delta_velocity": m.delta_velocity,
            "absorption": m.absorption,
            "idm": m.idm,
            "avpin": m.avpin,
            "permanent_impact": m.permanent_impact,
            "temporary_impact": m.temporary_impact,
            "efp": m.efp,
            "alpha_score": m.alpha_score,
            "p_long": m.p_long,
            "signal": m.signal,
        }));
    }
    let doc = json!({ "timestamp": now, "metrics": out });
    let _ = std::fs::write(OUT_FILE, serde_json::to_string_pretty(&doc).unwrap_or_default());
}
```

### `strategies-engine/breakout-strategy/src/bin/risk_analysis.rs`

```rust
//! Risk analizi (Rust) — TimescaleDB'deki trades tablosunu SQL ile özetler.
//!
//! --watch  : sabit ekranda her N sn'de yenilenir (tmux RISK paneli için).
//!           clear YAPILMAZ; imleç başa alınıp üzerine yazılır (titreşimsiz).
//! WATCH_SEC: yenileme süresi (varsayılan 5 sn).
//!
//! Bağlantı: `TIMESCALEDB_URL` (varsayılan postgres://cycle:cycle@localhost:5432/market_data)

use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::time::Duration;

#[derive(Debug)]
struct SymbolRow {
    symbol: String,
    count: i64,
    volume: f64,
    min: f64,
    max: f64,
}

fn db_url() -> String {
    std::env::var("TIMESCALEDB_URL")
        .unwrap_or_else(|_| "postgres://cycle:cycle@localhost:5432/market_data".into())
}

async fn render() {
    let pool = match PgPoolOptions::new().max_connections(2).connect(&db_url()).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Veritabanı açılamadı: {e}");
            return;
        }
    };

    let query = "
        SELECT symbol, COUNT(*) as cnt,
               SUM(price * quantity) as volume,
               MIN(price) as min_p,
               MAX(price) as max_p
        FROM trades
        GROUP BY symbol
        HAVING COUNT(*) > 50
        ORDER BY volume DESC
    ";

    let rows: Vec<SymbolRow> = match sqlx::query(query)
        .fetch_all(&pool)
        .await
    {
        Ok(rows) => rows
            .iter()
            .map(|r| SymbolRow {
                symbol: r.get("symbol"),
                count: r.get("cnt"),
                volume: r.get("volume"),
                min: r.get("min_p"),
                max: r.get("max_p"),
            })
            .collect(),
        Err(_) => {
            println!("Yeterli veri bulunamadı.");
            return;
        }
    };

    if rows.is_empty() {
        println!("Yeterli veri bulunamadı.");
        return;
    }

    let rows: Vec<(SymbolRow, f64)> = rows
        .into_iter()
        .map(|r| {
            let vol = if r.min > 0.0 { ((r.max - r.min) / r.min) * 100.0 } else { 0.0 };
            (r, vol)
        })
        .collect();

    println!("=== 📊 PİYASA HACİM VE RİSK DAĞILIMI (EN ÇOK İŞLEM GÖREN 15 PARİTE) ===");
    println!("  {:<10}{:<12}{:<16}{:<14}{:<14}{:<18}", "PARİTE", "İŞLEM", "HACİM_USDT", "MİN", "MAKS", "VOLATİLİTE_%");
    for (r, vol) in rows.iter().take(15) {
        println!(
            "  {:<10}{:<12}{:<16.2}{:<14.2}{:<14.2}{:<18.2}",
            r.symbol, r.count, r.volume, r.min, r.max, vol
        );
    }

    let mut sorted: Vec<&(SymbolRow, f64)> = rows.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\n=== ⚠️ EN YÜKSEK RİSK / VOLATİLİTE İÇEREN 10 PARİTE ===");
    println!("  {:<10}{:<12}{:<18}{:<16}", "PARİTE", "İŞLEM", "VOLATİLİTE_%", "HACİM_USDT");
    for (r, vol) in sorted.iter().take(10) {
        println!("  {:<10}{:<12}{:<18.2}{:<16.2}", r.symbol, r.count, vol, r.volume);
    }
}

#[tokio::main]
async fn main() {
    let watch = std::env::args().any(|a| a == "--watch");
    let watch_sec: u64 = std::env::var("WATCH_SEC")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    if !watch {
        render().await;
        return;
    }

    // Sabit ekran: ilk render tam boyutla çizilir; sonrakiler imleç başa alınır.
    print!("\x1b[2J\x1b[H"); // başta bir kez temizle
    render().await;
    loop {
        tokio::time::sleep(Duration::from_secs(watch_sec)).await;
        print!("\x1b[H"); // imleç en üste
        render().await;
    }
}
```
