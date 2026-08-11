# 📊 DATA-Engine Fonksiyonel Süreçler

## Giriş

`cold-starter` binary (soğuk başlatma rutini), `cold-storage` library (mmap disk tamponu). Her ikisi de workspace üyesidir. `cargo run -p cold-starter` ile çalıştırılır. `cold-storage` sadece bir lib crate'dir (binary değil).

**Önemli not:** data-engine şu an iskelet aşamasında. Gerçek I/O (SQLite, sled) `data/` klasöründeki DB'lere bağlıdır.

---

## Süreç 1: Cold Starter (Soğuk Başlatma)

`cold-starter/src/main.rs:3-9` — 3 fazlı kurtarma:

| Adım | Fonksiyon | Kod |
|:---|:---|:---|
| 1 | `CatchupRoutines` örneği oluşturulur (`main.rs:5`) | 2. |
| 2 | `fetch_200_ema()` — 200 EMA geçmişini ClickHouse Data Lake'ten çekme (mock: 50000.0) | 3. |
| 3 | `replay_buffer_in_paper_mode()` — mmap buffer'ı paper modda replay (mock) | 4. |
| 4 | `transition_to_live()` — buffer temizle, canlı moda geç (mock) | 5. |

> ⚠️ Tüm adımlar iskelet seviyesindedir; gerçek I/O yapılmaz. `catchup.rs:9` ve `catchup.rs:15`'teki `// Mock` yorumları bunu doğrular.

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

### Kritik tespit: `data-engine` içindeki crate'lerin hiçbiri DB okumaz/yazmaz.

DB'lere erişim **dışarıdaki crate'lerden** yapılır:

1. **`cycle-engine/persistence/src/db.rs`** — `start_db_writer` (thread spawn + `flume::bounded(1_000_000)` kanalı), 8 tablo (trades, orderbooks, liquidations, funding_rates, booktickers, open_interests, opportunities, symbol_metrics)
2. **`services-engine/strategies/breakout-strategy/src/bin/risk_analysis.rs`** — `trades` tablosunu SQL ile özetler
3. **`unused_services/detect-trb/src/main.rs`** — varsayılan db yolu

---

## Thread / Task Yapısı

**cold-starter ve cold-storage içinde hiç yok.**
- `main.rs` tek thread, senkron, async yok, `#[tokio::main]` yok, `thread::spawn` yok
- `DiskBuffer` `MmapMut` tutar; thread-safe değil (Send/Sync derive edilmemiş)

---

## Dış Bağımlılıklar

| Bağımlılık | Kaynak | Kullanım |
|:---|:---|:---|
| `memmap2 = { workspace = true }` | kök workspace'te (Cargo.toml:55) | cold-storage/Cargo.toml:7 |
| `cargo` | N/A | build |

---

## Satır Kodu

| Dosya | Satır |
|:---|:---|
| `cold-starter/src/main.rs` | 9 |
| `cold-starter/src/catchup.rs` | 23 |
| `cold-starter/Cargo.toml` | 5 |
| **cold-starter toplam** | **37** |
| `cold-storage/src/lib.rs` | 35 |
| `cold-storage/Cargo.toml` | 6 |
| **cold-storage toplam** | **41** |

---

## Sonuç

data-engine şu anda iskelet aşamasında; cold-starter (mock) ve cold-storage (mmap) bu yapıyı sunar. Gerçek veri akışı `cycle-engine/persistence/src/db.rs` (SQLite WAL) ve `services-engine/strategies/breakout-strategy/src/bin/risk_analysis.rs` tarafından sağlanır.