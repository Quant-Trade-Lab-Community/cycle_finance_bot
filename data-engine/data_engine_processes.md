# 📊 DATA-Engine Fonksiyonel Süreçler

## Giriş

`cold-starter` binary (soğuk başlatma rutini), `cold-storage` library (mmap disk tamponu). Her ikisi de workspace üyesidir. `cargo run -p cold-starter` ile çalıştırılır. `cold-storage` sadece bir lib crate'dir (binary değil).

**Önemli not:** Gerçek zaman serisi I/O TimescaleDB'ye (`cycle-engine/persistence`) yazılır; cold-starter TimescaleDB'den okur.

---

## Süreç 1: Cold Starter (Soğuk Başlatma)

`cold-starter/src/main.rs` — 2 fazlı kurtarma:

| Adım | Fonksiyon | Kod |
|:---|:---|:---|
| 1 | `CatchupRoutines` örneği oluşturulur (`main.rs`) | 2. |
| 2 | `fetch_200_ema()` — son 200 trade fiyatı TimescaleDB `trades` hypertable'ından çekilir, 200 EMA hesaplanır | 3. |
| 3 | `transition_to_live()` — buffer temizle, canlı moda geç | 4. |

> Bağlantı `TIMESCALEDB_URL` (varsayılan `postgres://cycle:cycle@localhost:5432/market_data`) üzerinden `sqlx` ile kurulur.

---

## Süreç 2: Cold Storage (mmap Disk Tamponu)

`cold-storage/src/lib.rs:1-35` — `DiskBuffer` struct'ı:

```
DiskBuffer::new(path, size) → dosyayı read/write/create modunda açar → set_len ile boyutlandırır → map_mut ile belleğe eşler
write_slice(offset, data) → bounds kontrolü (offset + data.len() <= mmap.len()) → copy_from_slice ile yazar
```

`read_slice(offset, len)` — mmap'dan veriyi okur; bounds kontrolü yapar.

---

## Süreç 3: Veri Akışı

TimescaleDB erişimi şu şekilde sağlanır:

1. **`cycle-engine/persistence/src/timescaledb.rs`** — `start_tsdb_writer` (thread spawn + `flume::bounded` kanalı), hypertable'lar (trades, orderbooks, liquidations, funding_rates, markprices, indexprices, lastprices, open_interests)
2. **`strategies-engine/src/bin/risk_analysis.rs`** — TimescaleDB `trades` tablosunu SQL ile özetler
3. **`data-engine/cold-starter/src/catchup.rs`** — TimescaleDB `trades`'ten 200 EMA hesaplar

---

## Thread / Task Yapısı

**cold-starter ve cold-storage içinde yok.**

- `cold-storage/src/lib.rs` `MmapMut` tutar; thread-safe değil (Send/Sync derive edilmemiş)
- `cold-starter` `#[tokio::main]` async giriş kullanır (sqlx bağlantısı için)

---

## Dış Bağımlılıklar

| Bağımlılık | Kaynak | Kullanım |
|:---|:---|:---|
| `memmap2 = { workspace = true }` | kök workspace'te | cold-storage/Cargo.toml |
| `sqlx = { workspace = true }` | kök workspace'te (postgres, runtime-tokio) | cold-starter/Cargo.toml — TimescaleDB sorguları |
| `tokio = { workspace = true }` | kök workspace'te | cold-starter async giriş |
| `cargo` | N/A | build |

---

## Satır Kodu

| Dosya | Satır |
|:---|:---|
| `cold-starter/src/main.rs` | 18 |
| `cold-starter/src/catchup.rs` | 42 |
| `cold-starter/Cargo.toml` | 8 |
| **cold-starter toplam** | **68** |
| `cold-storage/src/lib.rs` | 35 |
| `cold-storage/Cargo.toml` | 6 |
| **cold-storage toplam** | **41** |

---

## Sonuç

data-engine; cold-starter (TimescaleDB'den 200 EMA) ve cold-storage (mmap) yapılarını sunar. Gerçek zaman serisi kalıcılığı `cycle-engine/persistence/src/timescaledb.rs` (TimescaleDB) tarafından sağlanır.
