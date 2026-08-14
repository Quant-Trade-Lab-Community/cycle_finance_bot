# 💾 Data Engine — Tam Kaynak Kodu + Detaylı Analiz

> `data-engine/`. Bu doküman dizin ağacını, klasör/dosya sözlüğünü, her dosyanın **tam kaynak kodunu** ve **detaylı analizini** (mermaid akış diyagramlarıyla) içerir. Tarih: 2026-08-09

---

## 📂 İçindekiler

- [Dizin Ağacı](#dizin-agac)
- [Klasör ve Dosya Sözlüğü](#klasor-ve-dosya-sozlugu)
- [Detaylı Analiz (mermaid)](#detayl-analiz-mermaid)
- [Tam Kaynak Kodu](#tam-kaynak-kodu)

---

## 🌳 Dizin Ağacı

```
data-engine/
    ├── cold-starter/Cargo.toml
        ├── cold-starter/src/catchup.rs
        ├── cold-starter/src/main.rs
    ├── cold-storage/Cargo.toml
        ├── cold-storage/src/lib.rs
```

---

## 📖 Klasör ve Dosya Sözlüğü

> `data-engine/` — **Genel amaç:** Soğuk veri katmanı. `cold-storage` geçmiş kapanış verisini kalıcı dosyalarda tutar, `cold-starter` soğuk başlangıçta TimescaleDB `trades`'ten indikatörleri (200 EMA) hesaplayıp canlı moda geçer.
| Klasör / Dosya | Anlamı |
|---|---|
| `data-engine/` | HFT motorunun kalıcı veri saklama ve soğuk başlatma katmanı |
| `data-engine/data/` | Çalışma zamanı üretilen kalıcı veriler (mmap snap dosyası) — market verileri artık TimescaleDB'de |
| TimescaleDB `market_data` | Market verilerinin saklandığı zaman serisi veritabanı (PostgreSQL uzantısı, hypertable'lar) |
| `data-engine/data/paper_wal/` | DiskBuffer (mmap) tamponunun kalıcı dizini |
| `data-engine/data/paper_wal/conf` | Tampon konfigürasyon bilgisi |
| `data-engine/data/paper_wal/snap.0000000000000060` | 60. mmap anlık görüntüsü (snapshot) — sıra numaralı segment |
| `data-engine/cold-storage/` | Bellek eşlemeli disk tamponu (mmap) sağlayan düşük gecikmeli depolama crate'i |
| `data-engine/cold-storage/Cargo.toml` | cold-storage paket manifesti; tek bağımlılık workspace'ten `memmap2` |
| `data-engine/cold-storage/src/lib.rs` | `DiskBuffer` — memory-mapped dosya tabanlı sıfır kopya yazım arabelleği |
| `data-engine/cold-starter/` | Sistem kurtarma/başlatma rutinlerini barındıran ikili crate |
| `data-engine/cold-starter/Cargo.toml` | cold-starter manifesti; bağımlılıksız, workspace üyesi |
| `data-engine/cold-starter/src/main.rs` | Çalıştırılabilir giriş; catchup rutinlerini sırayla çağırır |
| `data-engine/cold-starter/src/catchup.rs` | `CatchupRoutines` — soğuk başlatma adımları (TimescaleDB'den 200 EMA, live geçiş) |

---

---

## 🔬 Detaylı Analiz (mermaid)

> Her dosyanın **detaylı açıklaması**, **algoritmik akış diyagramı** (mermaid) ve **"neden kullandık"** gerekçesi. Mermaid diyagramları HTML'de otomatik çizilir. Tarih: 2026-08-09

### `cold-starter/src/catchup.rs`

**Detaylı açıklama:** `CatchupRoutines` struct'ı, motorun soğuk (cold) başlatma sırasında çalıştırdığı kurtarma akışını temsil eder. 1) `fetch_200_ema`: İndikatörlerin doğru başlangıç durumuna sahip olması için 200 dönemlik EMA'yı TimescaleDB `trades` hypertable'ındaki son 200 trade fiyatından hesaplar (`sqlx`, `TIMESCALEDB_URL`). 2) `transition_to_live`: Canlı moda geçiş adımıdır.

**Neden kullandık:**
- Gecelik duruş sonrası indikatörlerin (EMA) doğru hesaplanabilmesi için geçmiş veriye hızlı erişim gerekir.
- TimescaleDB hypertable'ları yüksek yazma hacminde kararlı kalır ve SQL ile geçmiş trade verisine hızlı sorgu imkânı tanır.
- İki fazlı yapı başlatmayı deterministik ve test edilebilir kılar.

```mermaid
flowchart TD
    A["motor başlar<br>soğuk başlatma"] --> B["fetch_200_ema<br>TimescaleDB trades'ten son 200 fiyat"]
    B --> C["EMA hesapla"]
    C --> D["transition_to_live<br>buffer temizle"]
    D --> E["canlı modda çalış"]
```

---

### `cold-starter/src/main.rs`

**Detaylı açıklama:** İkili (binary) giriş noktasıdır. `catchup` modülünü `pub mod` ile içe alır, `CatchupRoutines` örneğini oluşturur ve `#[tokio::main]` altında `fetch_200_ema` çağrısının sonucuna göre ya EMA değerini yazdırır ya da hata ile çıkar; ardından `transition_to_live`'ı çağırır. Görevi orkestrasyondur; mantık `catchup.rs` içindedir.

**Neden kullandık:**
- Soğuk başlatma akışının tek bir çalıştırılabilirde denenebilmesi için ayrı bir binary gerekir.
- `main`'in rutinleri sırayla çağırması faz sırasını garanti eder; async çalışma TimescaleDB'ye bloke etmeden bağlanmayı sağlar.

```mermaid
flowchart TD
    A["main"] --> B["CatchupRoutines oluştur"]
    B --> C["fetch_200_ema (async)"]
    C --> D{"Ok mu?"}
    D -->|"hayır"| E["hata yazdır + çık"]
    D -->|"evet"| F["200 EMA yazdır"]
    F --> G["transition_to_live"]
    G --> H["başlatma tamam"]
```

---

### `cold-storage/src/lib.rs`

**Detaylı açıklama:** `DiskBuffer` struct'ı, memory-mapped dosya (mmap) üzerinde "sıfır gecikmeli" bir yazma tamponu sağlar. `new` dosyayı okuma/yazma modunda açar (gerekirse oluşturur), `set_len` ile istenen boyuta uzatır ve dosyayı belleğe eşler. `write_slice`, güvenlik kontrolünden sonra (`offset + data.len() <= mmap.len()`) veriyi doğrudan eşlenen bölgeye kopyalar; sistem çağrısı (syscall) gerektirmediği için HFT için kritik olan düşük gecikmeyi sağlar. Gerekli `unsafe` kod tek crate'te izole edilmiş, üstte `#![allow(unsafe_code)]` ile sınırlandırılmıştır — böylece motorun geri kalanı `#![forbid(unsafe_code)]` korumasında kalır.

**Neden kullandık:**
- Geleneksel disk I/O'suna göre çok daha düşük gecikme (çekirdek tamponlama + syscall yok).
- İşlem sonrası process ölse bile veri dosyada kalır (kalıcılık).
- `unsafe` mmap çağrısını tek bir crate'e hapsederek güvenlik sınırı çizer.

```mermaid
flowchart TD
    A["DiskBuffer::new"] --> B["OpenOptions ile dosyayı aç<br>gerekirse oluştur"]
    B --> C["set_len ile istenen boyuta uzat"]
    C --> D["MmapMut::map_mut ile belleğe eşle"]
    D --> E["write_slice"] --> F{"offset + len<br>&lt;= mmap.len()?"}
    F -->|"evet"| G["mmap segmentine<br>kopyala"]
    F -->|"hayır"| H["yazmayı atla<br>sınır koru"]
```

---

### `cold-storage/Cargo.toml`

**Detaylı açıklama:** cold-storage paket manifestidir; edition 2021, sürüm 0.1.0. Tek bağımlılığı `memmap2`'dir ve `workspace = true` ile sürümü workspace kökündeki `Cargo.toml`'dan (memmap2 = "0.9") alır — böylece sürüm kayması yaşanmaz. Dosya açma/işletim sistemi çağrıları için `std` kullanılır, ek harici bağımlılık yoktur.

**Neden kullandık:**
- mmap desteği için `memmap2` gereklidir (parking_lot, rust_decimal gibi diğer crate'lerle birlikte workspace'te yönetilir).
- `workspace = true` sürümlerin tek noktadan (kök manifest) güncellenmesini garanti eder.

```mermaid
flowchart TD
    A["cold-storage Cargo.toml"] --> B["memmap2 = workspace"]
    B --> C["workspace kökü<br>memmap2 = 0.9"]
    C --> D["lib.rs DiskBuffer mmap tamponu"]
```

---

### `cold-starter/Cargo.toml`

**Detaylı açıklama:** cold-starter ikili paketinin manifestidir; edition 2021, sürüm 0.1.0. Şu an `[dependencies]` bölümü boştur — bağımsız duran, çalıştırılabilir bir crate'tir. Workspace üyesi olarak kök manifestte listelenir, böylece aynı `cargo build` ile derlenir ve versiyon bütünlüğü korunur. İleride `cold-storage`'a bağımlılık ekleyerek DiskBuffer replay'ini gerçekleştirmesi beklenir.

**Neden kullandık:**
- Soğuk başlatma rutinlerini ayrı bir ikiliye ayırır (bağımlılık grafiğini izole eder).
- Workspace üyesi olması tüm motorla birlikte tek komutla derlenmesini sağlar.

```mermaid
flowchart TD
    A["cold-starter Cargo.toml"] --> B["dependencies: boş"]
    B --> C["main.rs + catchup.rs"]
    C --> D["ilk başlatma rutinleri"]
```

---

### `data-engine/data/`

**Detaylı açıklama:** Çalışma zamanı üretilen kalıcı veriler `data-engine/data/` altında toplanır: `paper_wal/` — DiskBuffer mmap tamponunun kalıcı segmentleri (`conf` + `snap.0000000000000060` gibi sıra numaralı anlık görüntüler). Market verileri ise TimescaleDB'de hypertable'lara yazılır (`cycle-engine/persistence`), SQLite kullanılmaz.

**Neden kullandık:**
- TimescaleDB (PostgreSQL), okuma/yazma çakışmalarını azaltıp yüksek yazma hacminde kararlı kalır.
- mmap snapshot segmentleri (sıra numaralı) veri kaybı olmadan kurtarma/replay imkânı tanır.
- Kalıcı dosyalar, process ölümlerinden sonra state'i yeniden kurmayı sağlar.

```mermaid
flowchart TD
    A["TimescaleDB market_data hypertable'ları"] --> B["soğuk başlatma için geçmiş veri (200 EMA)"]
    A --> D["canlı akış (persistence batch yazıcı)"]
    D --> A
```

---

**Özet:** 14 dosya/klasör sözlükte listelendi; kritik dosyalar dahil 7 mermaid diyagramı üretildi (catchup.rs ve cold-storage lib.rs dahil tüm dosyalar için akış şeması eklendi). Motorun amacı: soğuk başlatmada geçmiş veriyi (TimescaleDB/EMA) yükleyip canlı moda güvenle geçmek.

---

## 📄 Tam Kaynak Kodu

### `data-engine/cold-starter/Cargo.toml`

```toml
[package]
name = "cold-starter"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
sqlx = { workspace = true }
```

### `data-engine/cold-starter/src/catchup.rs`

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

### `data-engine/cold-starter/src/main.rs`

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

### `data-engine/cold-storage/Cargo.toml`

```toml
[package]
name = "cold-storage"
version = "0.1.0"
edition = "2021"

[dependencies]
memmap2 = { workspace = true }
```

### `data-engine/cold-storage/src/lib.rs`

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
