# STRATEGIES-Engine Mimari Dokümanı

## Genel Bakış

strategies-engine, Cycle Finance sisteminin strateji orkestrasyon kütüphanesidir. Bir binary değil, lib; cycle-engine/engine/src/engine/strategy_console.rs tarafından barındırılır.

Ana işlevi: services-engine/strategies/<isim>/ klasörünü tarar, her stratejiyi kendi derlenmiş binary'si olarak ayra bir alt-sürece başlatır, durumu izler ve komut kanallarından gelen istekleri işler.

- Strateji kaydi: HashMap<String, ManagedStrategy> (orchestrator.rs:55-59).
- Klasör tarama: available() her seferinde strategies_dir içindeki Cargo.toml'u olan alt klasörleri okur (orchestrator.rs:76-90).
- Komut işleme: process_command(line) — run/start, stop, restart, list/ls, status, help (orchestrator.rs:309-389).
- Alt-sürece yaşam döngüsü: run() → Command::spawn; stop() → SIGTERM; tick() → try_wait ile olen süreçleri toplama (reap).

Kritik nokta: trait_def.rs'deki Strategy trait'i sürci (in-process) orkestrator TitaniumOrchestrator (cycle-engine/engine/src/engine/orchestrator.rs:3, 97) tarafından kullanılır; StrategyOrchestrator ise sürci-dışı alt-sürece yönetir. İki ayrı mekanizmadır.

## Katmanlar ve Modül Sorumlulukları

| Dosya | Sorumlu |
|:---|:---|
| strategies-engine/src/lib.rs (5 satır) | Modül tanidlari + re-export: StrategyOrchestrator, ManagedStrategy, StrategyState, Strategy, Signal, FillReport |
| strategies-engine/src/orchestrator.rs (442 satır) | Orkestrasyon cekirdegi: durum makinesi, alt-surece spawn/kill, reap, komut ayistirma, durum raporu, 4 unit test |
| strategies-engine/src/trait_def.rs (27 satır) | Ortak strateji sözleşmesi: Signal enum'u, FillReport struct'u, Strategy trait'i |
| strategies-engine/Cargo.toml (9 satır) | Paket tanimi + 3 bağımlılık |

### orchestrator.rs detay
- StrategyState (31-39): strateji durumu.
- ManagedStrategy (42-49): tek stratejinin kaydi (ad, durum, başlangıç zamanı, son exit kodu, Child handle).
- StrategyOrchestrator (55-59): strategies_dir, bin_dir, strategies HashMap.
- new (66-73), available (76-90), state (93-98), ensure_registered (100-113), binary_path (115-117).
- resolve_strategy (123-133), not_found (135-142) — takma ad cöşeleme.
- run (145-173), run_many (176-178), stop (181-201), stop_many (204-206), restart (209-212).
- tick (216-248) — reap/poll.
- status (251-298) — Unicode emoji li rapor.
- process_command (309-389) — komut yorumlayici.
- Testler (392-442).

### trait_def.rs detay
- Signal (4-12): None, BuyMarket{quantity}, SellMarket{quantity}, BuyLimit{price,quantity}, SellLimit{price,quantity}, CancelAll.
- FillReport (14-19): order_id, executed_qty, avg_price.
- Strategy trait (21-27): id(), on_market_data(frame_id, &MarketDataSlot), on_timer(frame_id, delta_ns), on_fill(&FillReport), reset(). Send + Sync sınırlı.

## Strateji Yaşam Döngüsü

### StrategyState Durumlar (orchestrator.rs:31-39)
- Running — alt-sürece yaşıyor.
- Stopped — durmuş / hiç başlatılmış.
- Failed(String) — çıktı ama hata (kod ≠ 0) veya başlatılamadı.

### Durum Geçişleri

Stopped → Running: run() spawn başarılı (orchestrator.rs:161)

Running → Stopped: stop() SIGTERM sonrası (orchestrator.rs:198)

Running → Stopped: tick() exit code == 0 (orchestrator.rs:227-228)

Running → Failed("çıktı kodu N"): tick() exit code ≠ 0 (orchestrator.rs:229-230)

Running → Failed("poll hatası"): tick() try_wait hata dönerse (orchestrator.rs:235-237)

Stopped → Failed: stop() calsmayan stratejiyi durdurmaya çalışırsa hata mesajı döner ama durum değişmez (orchestrator.rs:187-189)

### Takma Ad Çözümlüğü

resolve_strategy (orchestrator.rs:123-133):

1. Birebir eşleşme: breakout-strategy → breakout-strategy.
2. -strategy soneki ekleyerek: breakout → breakout-strategy.

not_found (orchestrator.rs:135-142): Klasör boşsa farklı, değilse mevcut listeyi gösteren hata mesajı.

Test kanıtı: run breakout → "başlatılamadı" (takma ad çözüldü), "bulunamadı" değil.

run ayrıca zaten çalışanan bir stratejiyi reddeder: "'{name}' zaten çalışıyor (pid: N)" (orchestrator.rs:152-153).

## Giriş Noktaları

### Binary'ler

1. strategy-console (cycle-engine/engine/src/bin/strategy-console.rs:11-13) → engine::engine::strategy_console::run_strategy_console() (strategy_console.rs:94).

   strategies_dir = $CYCLE_ROOT/services-engine/strategies
   bin_dir = $CYCLE_ROOT/target/debug
   Komut kuyruğu: /tmp/strategy_cmd.d (STRATEGY_CMD_DIR env ile değiştirilebilir)
   Durum dosyası: /tmp/strategy_status.txt (STRATEGY_STATUS_FILE)

2. strategies-engine kendisi binary tutmeyip lib yapısı; cycle-engine/engine/src/engine/strategy_console.rs bağımsız orkestrasyon binary'dir.

3. breakout-strategy binary'si (breakout-strategy/Cargo.toml:6-8): ana strateji süreci, orkestratörün run breakout ile spawn ettüğü süreçtir.

4. Cargo auto-bin'ler (src/bin/): alerts, listener, risk_analysis — orkestratör bunları spawn etmez; bağımsız yardımcı araçlardır.

### Komutlar (process_command, orchestrator.rs:316-387)

| Komut | Alias | İşlem |
|:---|:---|:---|
| run <isim> [...] | start | bir veya daha fazla stratejiyi alt-sürece başlatır |
| stop <isim> [...] | — | SIGTERM ile durdurur |
| restart <isim> | — | stop + run (orchestrator.rs:209-212) |
| list | ls | mevcut stratejiler + durumu |
| status | — | ayrıntılı orkestrasyon raporu |
| help | — | kullanım |

### Çevre Komut Girişleri (strategy_console.rs:36-91)

- Çevrimli stdin (rustyline, strategy> prompt) — spawn_stdin_reader (65-91).
- Dosya kuyruğu poller — spawn_cmd_file_poller (36-62): /tmp/strategy_cmd.d/*.cmd dosyalarını 250 ms'de bir okur, satırları Input::Command olarak kanala basar, dosyayı siler. Shell'deki strat run breakout komutları buraya yazılır.

## Alt-Sürece Dağılım

Orkestratör bunları spawn etmez; bağımsız alt-süreçler her birinde kendi process'ı:

| Alt Sürece | Dosya | İşlem |
|:---|:---|:---|
| breakout-strategy | breakout-strategy/Cargo.toml | Ana strateji (Event-Driven Kırılım) |
| breakout-strategy/src/bin/listener.rs | | Mikro-yapı metrikleri |
| breakout-strategy/src/bin/alerts.rs | | alerts.toml CLI |
| breakout-strategy/src/bin/risk_analysis.rs | | Risk dağıtım raporu |
| breakout-strategy/src/lib.rs | | Core module |
| breakout-strategy/src/metrics.rs | | Mikro-yapı metrik |
| src/main.rs (308 satır) | | Ana strateji |

## Thread / Task Yapısı

### strategy-console / StrategyOrchestrator

Tek thread + 2 yardımcı thread:

```
strategy-console main thread
  ├── spawn_cmd_file_poller thread (strategy_console.rs:36-62) — 250 ms poll /tmp/strategy_cmd.d
  ├── spawn_stdin_reader thread (strategy_console.rs:65-91) — rustyline, sonsuz readline
  └── ana döngü (strategy_console.rs:128-151):
      ├── rx.try_recv() → process_command(line) → yanıtı yaz + /tmp/strategy_status.txt
      ├── 500 ms'de bir orch.tick() → ölen alt-süreçleri reap (orchestrator.rs:216)
      └── 100 ms sleep
```

stop() içinde: SIGTERM gönderdikten sonra child.wait() bir geçici thread'e alınır (orchestrator.rs:195-197). Analiz: SIGKILL fallback yorumda belirtilir (orchestrator.rs:180), ama kodda sadece SIGTERM var.

tick() (orchestrator.rs:216-248): try_wait() (non-blocking poll), olen süreçler stderr'e raporlanır (246).

### breakout-strategy main.rs (async + 1 std thread)

```
spawn_price_reader std thread (main.rs:166-201) — /cycle_finance_pricefeed ring'ini okur
  └── mpsc::unbounded_channel (main.rs:276) → tokio actor döngüsü (main.rs:283)
```

- WAKE_INTERVAL=500ms sayesinde ring'de event yoksa da döngü uyanır (main.rs:33-34).

### listener.rs (1 std thread + 1 yardımcı thread)

```
spawn_price_corr_thread (listener.rs:37-62) — 200 ms'de bir :3004/api/lastprice çeker, fiyat CorrSeries'lerine yazar.
ana thread: ring okuma döngüsü (94-159) — 50 µs sleep; 2 sn'de bir render + JSON
```

- Fiyat/hacim korelasyon serileri Arc<Mutex<HashMap<...>>> ile paylaşılır.

## Dış Bağımlılıklar

### strategies-engine (Cargo.toml:6-9)

| Bağımlılık | Kaynak | Kullanım |
|:---|:---|:---|
| transport | path: ../cycle-engine/transport | MarketDataSlot (trait_def.rs:1), ring/event/wire tipleri |
| rust_decimal | workspace (1.34, maths+serde) | Decimal (trait_def.rs:2, 7) |
| libc | workspace (0.2) | kill(child.id(), SIGTERM) (orchestrator.rs:193) |

### breakout-strategy (Cargo.toml:10-18)

| Bağımlılık | Kaynak | Kullanım |
|:---|:---|:---|
| tokio | workspace (1.0 full) | async main, mpsc, timeout |
| reqwest | workspace (0.11, blocking özellikli) | detect-ms/REST çağrıları |
| serde / serde_json | workspace | JSON parse/serialize |
| chrono | workspace (0.4) | zaman damgası |
| sqlx | workspace (postgres, runtime-tokio) | TimescaleDB trades sorguları (risk_analysis) |
| transport | path: ../../../cycle-engine/transport | ring_buffer, events, wire |
| rust_decimal | workspace | fiyat dönüşümleri |

### transport modülleri (dolayılı)

- ring_buffer.rs: GenerationalRingBuffer::with_name("/cycle_finance_pricefeed", …) (ring_buffer.rs:50), get_head (157), read_slot (164), MarketDataSlot (12).
- events.rs: EventType::{Trade, Orderbook, BookTicker, FundingRate, …} (events.rs:46-98).
- wire.rs: decode(buf) → Option<OwnedEvent> (wire.rs:203).

## Satır Sayıları

| Dosya | Satır |
|:---|:---|
| strategies-engine/Cargo.toml | 9 |
| strategies-engine/src/lib.rs | 5 |
| strategies-engine/src/orchestrator.rs | 442 |
| strategies-engine/src/trait_def.rs | 27 |
| **strategies-engine toplam** | **483** |
| breakout-strategy/Cargo.toml | 18 |
| breakout-strategy/src/lib.rs | 3 |
| breakout-strategy/src/main.rs | 308 |
| breakout-strategy/src/metrics.rs | 580 |
| breakout-strategy/src/bin/listener.rs | 282 |
| breakout-strategy/src/bin/alerts.rs | 223 |
| breakout-strategy/src/bin/risk_analysis.rs | 112 |
| **breakout-strategy toplam** | **1526** |

İlişkili (isteğe bağlı, dokümanlar için):
| cycle-engine/engine/src/engine/strategy_console.rs | 152 |
| cycle-engine/engine/src/bin/strategy-console.rs | 13 |
| cycle-engine/engine/src/engine/orchestrator.rs (TitaniumOrchestrator, trait kullanıcısı) | 183 |

## Sonuç

Mimari: İki katmanlı tasarım var — (1) trait_def.rs::Strategy sözleşmesi, TitaniumOrchestrator tarafından aynı süreçte (shard'lı, ring'den on_market_data ile) çalıştırılır ve Signal → OrderIntent → RiskEngine → gateway akar (orchestrator.rs:32-51, 97); (2) StrategyOrchestrator ise her stratejiyi bağımsız bir OS süreci olarak yönetir (run/stop/status/reap). StrategyState makinesi + resolve_strategy takma ad çözümlemesi + maildir-benzedi /tmp/strategy_cmd.d kuyruğu, cycle-engine shell'inin strat komutlarına bağlanır.

Süreç: Orkestratör target binary'sini spawn eder; strateji süreci ring'den event-by-event fiyat alır, bekleme aralığında detect-ms'i sorgular ve sinyal üretir (emir açmaz). Yardımcı süreçler (listener, alerts, risk_analysis) orkestratör tarafından değil, manuel/tmux panellerinde başlatılır; sırasıyla mikro-yapı metrikleri, alerts.toml yönetimi ve SQL risk raporu üretir.