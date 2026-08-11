# 📊 DATA-Engine Mimari Dokümanı

## Genel Bakış

**data-engine**, Cycle Finance'in **data katmanını** (disk tamponu, Ring Buffer, SQLite) oluşturur.

- **Data Engine** (veri akışı): 2 tablo ve veri kaynağında (Market data, paper orders) okuyan, yazar ve yöneten bir kütüphane.
- **Disk Buffer** (`cold-storage`): `memmap2` ile mmap disk tamponu; low-latency veri okuma/yazma.
- **Cold Starter** (`cold-starter`): Soğuk başlatma rutini — indikatör geçmişi yüklemek, disk tamponu paper mode'da replay, canlı moda geçiş.
- **Veri Formatları:** SQLite (WAL modu), sled 0.34 segment store (paper_wal/), POSIX shm ring (data/).

---

## Katmanlar ve Modül Sorumlulukları

| Katman | Modül | Sorumluluk |
|:---|:---|:---|
| **Cold Starter** | `cold-starter/src/main.rs` | Soğuk başlatma orkestrasyonu; `CatchupRoutines` (3 fazlı kurtarma) |
| **Cold Starter** | `cold-starter/src/catchup.rs` | 3 aşama: fetch_200_ema → replay_buffer_in_paper_mode → transition_to_live |
| **Cold Storage** | `cold-storage/src/lib.rs` | `DiskBuffer` struct (mmap disk tamponu; `write_slice`, `set_len`) |
| **Data Store** | `data/market_data.db` | SQLite WAL modu — 709M, 8 tablo, 1.7M kayıt (trades tablosu) |
| **Data Store** | `data/paper_live.db` | SQLite WAL modu — 4KB (paper işlem logları) |
| **Data Store** | `data/paper_wal/` | sled 0.34 segment store (paper order data) |

---

## Veri Akışı

### Cold Starter (soğuk başlatma)

| Adım | İşlem | Kod |
|:---|:---|:---|
| 1 | `cold-starter` → `CatchupRoutines` (kurtarma orkestrasyonu) (main.rs:3-9) | 2. |
| 2 | `fetch_200_ema()` — 200 EMA geçmişi `market_data.db`'den çekme (mock: 50000.0) (catchup.rs:6-10) | 3. |
| 3 | `replay_buffer_in_paper_mode()` — mmap buffer'ı paper mode'nda replay (mock) (catchup.rs:14-17) | 4. |
| 4 | `transition_to_live()` — buffer temizle, canlı moda geç (mock) (catchup.rs:20-22) | 5. |

> ⚠️ Tüm adımlar iskelet seviyesindedir; gerçek I/O yapılmaz.

### Cold Storage (mmap disk tamponu)

```
write_slice(offset, data)
    └─ bounds kontrolü (offset + data.len() <= mmap.len())
    └─ copy_from_slice ile yazma (cold-storage/src/lib.rs:28-34)
```

`read_slice(offset, len)` — mmap'dan veriyi okur, bounds kontrolü yapar.

---

## Veri Formatları ve Şema

### Market Data DB (`market_data.db` — SQLite WAL)

**Şema (8 tablo):**

| Tablo | Kolonlar | Kaynak satır | Mevcut kayıt |
|:---|:---|:---|:---|
| `trades` | id, symbol, price, quantity, timestamp | `db.rs:21-29` | 1.740.244 |
| `orderbooks` | id, symbol, bids TEXT, asks TEXT | `db.rs:31-39` | 939.303 |
| `liquidations` | id, symbol, side, price, quantity, timestamp | `db.rs:41-51` | 0 |
| `funding_rates` | id, symbol, mark_price, index_price, funding_rate, next_funding_time | `db.rs:53-63` | 0 |
| `booktickers` | id, symbol, best_bid_price/qty, best_ask_price/qty | `db.rs:65-75` | 0 |
| `open_interests` | id, symbol, open_interest, timestamp | `db.rs:77-85` | 0 |
| `opportunities` | id, symbol, score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, verdict, timestamp | `db.rs:87-101` | 0 |
| `symbol_metrics` | id, symbol, score, efficiency, price_bps_per_s, price_ticks_per_s, ob_changes_per_s, spread_bps, timestamp | `db.rs:103-116` | 0 |

- PRAGMA: `journal_mode=WAL; synchronous=NORMAL; cache_size=-64000` (`db.rs:13-17`)
- Batch commit: 10.000 kayıt veya 1 sn (`db.rs:118-121`)

### Paper DB (`paper_live.db` — SQLite WAL)

**Şema (2 tablo):**

| Tablo | Kolonlar | Kaynak satır |
|:---|:---|:---|
| `paper_open_orders` | order_id TEXT PK, symbol, side, price, open_quantity, original_quantity, locked_balances_json | `sqlite_projection.rs:36` (INSERT OR REPLACE) |
| `paper_trades` | id, order_id, symbol, side, price, quantity, fee, timestamp | `sqlite_projection.rs:26` (INSERT) |

Her iki tablo da şu an **boş**. DB yolu `PAPER_DB_PATH` env var ile seçilir (`execution-engine/src/paper/config.rs:58-59`).

### Sled 0.34 Segment Store (`paper_wal/`)

- `conf` dosyası: `segment_size: 524288 / use_compression: false / version: 0.34`
- `paper_service` (SledEventStore) — `sled::open(path)` (`events.rs:50`), `append` (`:65-71`), `replay` (`:73-91`)
- Default yol: `PAPER_SLED_PATH` env var (`:101-103` → `./data-engine/data/paper_wal`)
- Dosya: 524.287 byte; `snap.0000000000000060` sled anlık görüntüsü; `blobs/` boş

---

## Thread / Task Yapısı

**Data engine içinde hiç yok.**

- `main.rs` tek thread, senkron, async yok; `tokio::main` yok.
- `DiskBuffer` `MmapMut` tutar; thread-safe değil (`Send`/`Sync` derive edilmemiş).

### İlgili (dış) thread yapısı

- `cycle-engine/engine/src/main.rs:16-18` — `thread::spawn` ile `start_db_writer` ayrı thread'de; `flume::bounded(1_000_000)` kanalıyla event alır.
- `cycle-engine/engine/src/main.rs:24-27` — RT priority (99) ile ayrı bir tüketim thread'i (LockFreeDispatcher consumer).

---

## Dış Bağımlılıklar

| Bağımlılık | Kaynak | Kullanım |
|:---|:---|:---|
| `memmap2` | workspace (`memmap2 = "0.9"`) | Cold storage disk tamponu (`cold-storage/src/lib.rs:7`) |
| `sqlite3` | Cargo built-in | `db.rs` tablo oluşturma ve okuma |
| `crate` (sadece) | workspace | `cold-starter` ve `cold-storage` bağımlılıkları boş veya workspace içi |
| `rust_decimal` | workspace | Para aritmetik, serde |

---

## Satır Sayıları

| Dosya | Satır |
|:---|:---|
| `cold-starter/src/main.rs` | 9 |
| `cold-starter/src/catchup.rs` | 23 |
| `cold-starter/Cargo.toml` | 5 |
| `cold-starter toplam` | **37** |
| `cold-storage/src/lib.rs` | 35 |
| `cold-storage/Cargo.toml` | 6 |
| `cold-storage toplam` | **41** |
| **data-engine Rust + manifest toplamı** | **78** |

---

## Veri Akış

```
Market Data DB (market_data.db) ──► start_db_writer (thread) ──► flume kanalı ──► (sanal) data akışı
                                                        │
                                                        ▼
paper_live.db ──► paper_service ──► event-sourcing projection ──► SQLite
                                    │
paper_wal (sled 0.34) ──► paper_service ──► event-sourcing
```

---

## Sonuç

data-engine şu an sadece iskelet aşamasında. Cold-starter (mock) ve cold-storage (mmap) bu yapıyı sunar. Gerçek veri akışı `db.rs` (SQLite) ve `paper_service` (Sled WAL + SQLite) tarafından sağlanır.
