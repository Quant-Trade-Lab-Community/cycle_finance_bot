# Bare-Metal Düşük Gecikme Planı (Faz 0–3)

**Hedef:** Standart x86 sunucu/VM'de, özel NIC/FPGA olmadan, yazılımda minimum gecikme (~100µs–1ms kuyruk).
**Hedef Gecikme:** Kesin hedef yok — Faz 0 ölçümüyle veriye göre belirlenecek.
**Ortam:** VM mi bare-metal mi uygulama başında `systemd-detect-virt` ile tespit edilip Faz 3 ayarları ona göre yapılacak.

## Temel Teşhis (bugünkü durum)

Sıcak yol "bare-metal" değil. Her tick için:
`WS frame → String (alloc) → Vec<u8> (alloc) → flume → bytes.clone() (alloc) → simd_json (parse) → Decimal (100ns+) → ring slot + SQLite`

Bare-metal iskelesi ölü kod: `hal/cpu.rs`, `engine/orchestrator.rs`, `timer/tsc.rs` hiçbir yerden çağrılmıyor. `clock_gettime`/`fence`/hugepage/mlock yok.

## Faz 0 — Ölçüm Altyapısı (hedefi veriyle belirleme)

1. `core/src/timer/tsc.rs` — TSC frekansını `_rdtsc` ile kalibre et (sabit 3.0 GHz'i kaldır), `SystemTime`/`clock_gettime` karşı kalibrasyon
2. Ring slot'una TSC timestamp alanı ekle; uçtan uca tick→algılayıcı→karar latens ölçümü + p50/p99/p99.9 loglama
3. `perf record`/flamegraph ile profille — gerçek darboğazları doğrula
4. **Karar noktası:** ilk ölçümden sonra hedef gecikme rakamı birlikte belirlenir

## Faz 1 — Sıcak Yolu Bare-Metal Yapma

| # | İş | Detay |
|---|-----|-------|
| 1.1 | CPU pin'i bağla | `hal/cpu.rs:3` `pin_to_core()` — parse thread (`core/src/main.rs:27`), bridge spin reader'lar (`paper-service/src/bridge.rs:68,108`), correlation (`cli/correlation_cli.rs:218`), orchestrator (`engine/orchestrator.rs:44`) |
| 1.2 | RT önceliği yaygınlaştır | `SCHED_FIFO 99` şu an sadece parse thread'de (`core/src/main.rs:28`). Sıcak yoldaki tüm thread'lere uygula |
| 1.3 | Allocation'ları kaldır | `adapter/src/binance.rs:45-46` (String→Vec kopyası) → önceden ayrılmış buffer havuzu; `core/src/main.rs:38` `bytes.clone()` kaldır |
| 1.4 | Clock düzeltmesi | `SystemTime` (validator, actor) → kalibre edilmiş TSC monotonic saat |
| 1.5 | Bellek yerleşimi | Ring'i `MAP_HUGETLB` ile 2MB hugepage'e koy; `mlockall` ile sayfa swap dışına; `SharedHeader`'a cacheline padding (`core/src/memory/ring_buffer.rs:15-20` false-sharing) |
| 1.6 | Ring buffer doğruluğu | Init race (`ring_buffer.rs:76-83`) → CAS+magic; slot okumaya release/acquire fence; `_mm_pause` ile spin_loop |
| 1.7 | Orchestrator'ı canlandır | `engine/orchestrator.rs:44` spin-loop (1ms `TscTimer` + `_rdtsc`) STRATEGY terminali için ana yol — tokio yığını sıcak yoldan çıkar |

## Faz 2 — Senkronizasyon ve Veri Yolu Temizliği

1. Sled "WAL" sıcak yoldan çıkar: `paper-service/src/main.rs:60-76`'daki async executor'daki std Mutex + sled append'i ayrı std thread'e taşı
2. Fixed-point aritmetik: matching'te `rust_decimal` (≈100ns+) → `i64` sabit nokta (≈5ns) — `execution-engine/src/paper/actor.rs`
3. Lock-free SPSC: parse→aktör `tokio mpsc` → lock-free queue
4. Order ring'e gerçek producer: `OrderRingBuffer::push` (`order_ring.rs:110`) tanımlı ama asla çağrılmıyor — strateji→emir akışı şu an HTTP'den geçiyor, doğrudan order ring'e yaz
5. Bridge/source okuyucularındaki per-slot `.to_vec()` kopyalarını kaldır (`bridge.rs:35`, `source.rs:20`)

## Faz 3 — OS ve Dağıtım (Bare-Metal)

1. systemd birimleri (tmux yerine): `CPUAffinity=`, `LimitRTPRIO=`, `Nice=-20`, `Restart=on-failure`
2. Kernel cmdline (bare-metal ise): `isolcpus=` `nohz_full=` `rcu_nocbs=` `nmi_watchdog=0`
3. `tuned-adm profile latency-performance`, CPU governor `performance`, THP kapat, IRQ affinity (NIC kesmeleri sıcak çekirdeklerden uzaklaştır)
4. K8s manifestleri düzelt veya kaldır (bugünkü hali `/dev/shm` mountsuz, çalışmaz)

## Faz 4 (opsiyonel, donanım gerektirir)

Standart VM'de kernel bypass mümkün değil. Gelecekte:
- DPDK uyumlu NIC alınırsa → WebSocket yerine Binance raw UDP/multicast + AF_XDP/DPDK (Faz 1'deki arayüz bozulmadan yapıya hazır bırakılacak)
- FPGA yolu (Appendix — ayrı yol haritası, şimdi kapsam dışı)

## Beklenen Sonuç

- Şimdi: ~5ms+ (kasıtlı simülasyon dahil) → Hedef: yazılımda ~100µs–500µs (network RTT hariç) — kesin rakam Faz 0 ölçümüyle belirlenir
- Her faz KPI'larla doğrulanır, geriye dönüş imkânı korunur

## Doğrulama

- Her faz sonunda `cargo build --release --workspace` + `cargo test` + `cargo clippy`
- `cargo kani --package formal_verification` (ring buffer/risk değişikliklerinde)
- Faz 0'ın latens ölçümü her fazdan sonra tekrarlanır → ilerleme sayısal takip
