# 📊 DATA-Engine Mimari Dokümanı

## Genel Bakış

**data-engine**, Cycle Finance'in **data katmanını** (disk tamponu, Ring Buffer, TimescaleDB) oluşturur.

- **Data Engine** (veri akışı): Zaman serisi kalıcılığını (TimescaleDB) ve Ring Buffer veri kaynağını okuyan, yazar ve yöneten kütüphaneler.
- **Disk Buffer** (`cold-storage`): `memmap2` ile mmap disk tamponu; low-latency veri okuma/yazma.
- **Cold Starter** (`cold-starter`): Soğuk başlatma rutini — TimescaleDB'den indikatör geçmişi (200 EMA) hesaplamak, canlı moda geçiş.
- **Veri Formatları:** TimescaleDB (PostgreSQL uzantısı), POSIX shm ring (data/).

---

## Katmanlar ve Modül Sorumlulukları

| Katman | Modül | Sorumluluk |
|:---|:---|:---|
| **Cold Starter** | `cold-starter/src/main.rs` | Soğuk başlatma orkestrasyonu; `CatchupRoutines` (kurtarma) |
| **Cold Starter** | `cold-starter/src/catchup.rs` | 2 aşama: fetch_200_ema → transition_to_live |
| **Cold Storage** | `cold-storage/src/lib.rs` | `DiskBuffer` struct (mmap disk tamponu; `write_slice`, `set_len`) |
| **Data Store** | TimescaleDB `market_data` DB | Hypertable'lar: trades, orderbooks, liquidations, funding_rates, markprices, lastprices, indexprices, open_interests |

---

## Veri Akışı

### Cold Starter (soğuk başlatma)

| Adım | İşlem | Kod |
|:---|:---|:---|
| 1 | `cold-starter` → `CatchupRoutines` (kurtarma orkestrasyonu) (main.rs) | 2. |
| 2 | `fetch_200_ema()` — 200 EMA geçmişi TimescaleDB `trades` hypertable'ından çekilir ve hesaplanır (catchup.rs) | 3. |
| 3 | `transition_to_live()` — buffer temizle, canlı moda geç (catchup.rs) | 4. |

### Cold Storage (mmap disk tamponu)

```
write_slice(offset, data)
    └─ bounds kontrolü (offset + data.len() <= mmap.len())
    └─ copy_from_slice ile yazma (cold-storage/src/lib.rs:28-34)
```

`read_slice(offset, len)` — mmap'dan veriyi okur, bounds kontrolü yapar.

---

## Veri Formatları ve Şema

### TimescaleDB `market_data` (zaman serisi kalıcılığı)

Kalıcılık, `cycle-engine/persistence` tarafından TimescaleDB (PostgreSQL) hypertable'larına yazılır. Şema:

| Tablo | Kolonlar | Kaynak |
|:---|:---|:---|
| `trades` | symbol, price, quantity, is_buyer_maker, timestamp | `persistence/src/timescaledb.rs:60` |
| `orderbooks` | symbol, bids JSONB, asks JSONB, timestamp | `persistence/src/timescaledb.rs:61` |
| `liquidations` | symbol, side, price, quantity, timestamp | `persistence/src/timescaledb.rs:62` |
| `funding_rates` | symbol, mark_price, index_price, funding_rate, next_funding_time, timestamp | `persistence/src/timescaledb.rs:63` |
| `open_interests` | symbol, open_interest, timestamp | `persistence/src/timescaledb.rs:64` |
| `markprices` | symbol, price, timestamp | `persistence/src/timescaledb.rs:65` |
| `indexprices` | symbol, price, timestamp | `persistence/src/timescaledb.rs:66` |
| `lastprices` | symbol, price, timestamp | `persistence/src/timescaledb.rs:67` |

- Bağlantı: `TIMESCALEDB_URL` (varsayılan `postgres://cycle:cycle@localhost:5432/market_data`)
- Batch commit: 1000 kayıt veya 1 sn (`persistence/src/timescaledb.rs:109`)

---

## Thread / Task Yapısı

**Data engine içinde hiç yok.**

- `main.rs` tek thread, senkron, async yok; `tokio::main` yok.
- `DiskBuffer` `MmapMut` tutar; thread-safe değil (`Send`/`Sync` derive edilmemiş).

### İlgili (dış) thread yapısı

- `cycle-engine/engine/src/main.rs` — `thread::spawn` ile `start_db_writer` ayrı thread'de; `flume::bounded(1_000_000)` kanalıyla event alır.
- `cycle-engine/engine/src/main.rs` — RT priority (99) ile ayrı bir tüketim thread'i (LockFreeDispatcher consumer).

---

## Dış Bağımlılıklar

| Bağımlılık | Kaynak | Kullanım |
|:---|:---|:---|
| `memmap2` | workspace (`memmap2 = "0.9"`) | Cold storage disk tamponu (`cold-storage/src/lib.rs:7`) |
| `sqlx` | workspace (postgres, runtime-tokio) | Cold starter — TimescaleDB sorguları |
| `crate` (sadece) | workspace | `cold-starter` ve `cold-storage` bağımlılıkları workspace içi |
| `rust_decimal` | workspace | Para aritmetik, serde |

---

## Satır Sayıları

| Dosya | Satır |
|:---|:---|
| `cold-starter/src/main.rs` | 18 |
| `cold-starter/src/catchup.rs` | 42 |
| `cold-starter/Cargo.toml` | 8 |
| `cold-starter toplam` | **68** |
| `cold-storage/src/lib.rs` | 35 |
| `cold-storage/Cargo.toml` | 6 |
| `cold-storage toplam` | **41** |
| **data-engine Rust + manifest toplamı** | **109** |

---

## Veri Akış

```
TimescaleDB (market_data hypertable'ları) ──► start_tsdb_writer (thread) ──► flume kanalı ──► flow ring'leri
```

---

## Sonuç

data-engine; cold-starter (TimescaleDB'den 200 EMA) ve cold-storage (mmap) yapılarını sunar. Gerçek zaman serisi kalıcılığı TimescaleDB (`persistence`) tarafından sağlanır.
