# STRATEGIES-Engine Fonksiyonel Süreçler

## Giriş

strategies-engine bir binary değil, kütüphane. cycle-engine/engine/src/engine/strategy_console.rs tarafından barındırılır.

**Başlatma:**
- `cargo run -p strategies-engine` (kök workspace) — bu, binary değil, library çalıştırır.
- `strategy-console` binary'i kullanılır: `strategy-console run breakout momentum` komutu ile strateji kullanılır.

---

## Süreç 1: Orkestrasyon Kontrol

`src/orchestrator.rs:12-22` — Komut ayrıştırma:

```
cycle-engine shell (tmux 0)
  └─ strat run breakout momentum          # komut dosyasına yazar
       └─ /tmp/strategy_cmd.d/cmd_*.cmd   # maildir benzeri kuyruk
            └─ StrategyOrchestrator::process_command(line)
```

- `process_command` (orchestrator.rs:309-389) — `run/start`, `stop`, `restart`, `list/ls`, `status`, `help` işleyir.
- Komut dosyası: `/tmp/strategy_cmd.d/*.cmd` (STRATEGY_CMD_DIR env ile değiştirilebilir).

---

## Süreç 2: Strateji Durumları (StrategyState)

| Durum | Anlamı |
|:---|:---|
| Running | Alt-sürece yaşıyor. |
| Stopped | Durmuş / hiç başlatılmış. |
| Failed(String) | Çıktı ama hata (kod ≠ 0) veya başlatılamadı. |

### Durum Geçişleri

| Geçiş | Nerede |
|:---|:---|
| Stopped → Running | run() spawn başarılı (orchestrator.rs:161) |
| Running → Stopped | stop() SIGTERM sonrası (orchestrator.rs:198) |
| Running → Stopped | tick() exit code == 0 (orchestrator.rs:227-228) |
| Running → Failed("çıktı kodu N") | tick() exit code ≠ 0 (orchestrator.rs:229-230) |
| Running → Failed("poll hatası") | tick() try_wait hata dönerse (orchestrator.rs:235-237) |
| Stopped → Failed | stop() calismayan stratejiyi durdurmaya çalışırsa hata mesajı döner ama durum değişmez (orchestrator.rs:187-189) |

---

## Süreç 3: Takma Ad Çözümlüğü

resolve_strategy (orchestrator.rs:123-133):

1. Birebir eşleşme: breakout-strategy → breakout-strategy.
2. -strategy soneki ekleyerek: breakout → breakout-strategy.

not_found (orchestrator.rs:135-142): Klasör boşsa farklı, değilse mevcut listeyi gösteren hata mesajı.

Test kanıtı: run breakout → "başlatılamadı" (takma ad çözüldü), "bulunamadı" değil.

run ayrıca zaten çalışanan bir stratejiyi reddeder: "'{name}' zaten çalışıyor (pid: N)" (orchestrator.rs:152-153).

---

## Süreç 4: Alt-Sürece Spawn / Stop / Reap

### Alt-Sürece Başlatma (run)

`run()` (orchestrator.rs:145-173):

1. `available()` — strateji klasörü kontrolü.
2. `binary_path()` — alt-sürece ait binary yol.
3. `spawn()` — bir sonraki process oluşturur.
4. `State::Running` olarak kaydeder.
5. `resolve_strategy()` — takma ad çözerek strateji adını normalleştirir.

### Alt-Sürece Durdurma (stop)

`stop()` (orchestrator.rs:181-201):

1. SIGTERM ile alt-sürece gönderir.
2. `child.wait()` bekler.
3. `exit code` kontrol edilir.
4. Status dosyasına yazılır.

### Alt-Sürece Tekrarlama (tick)

`tick()` (orchestrator.rs:216-248):

1. `try_wait()` — non-blocking, ölen süreçleri alır.
2. `exit code == 0` → Stopped (başarılı).
3. `exit code ≠ 0` → Failed("çıktı kodu N").
4. `stderr` çıktısı: ölen süreçtan output alınır (246).

---

## Süreç 5: Komut İşleme (process_command)

orchestrator.rs:309-389

### Komut Anahtar Kelimeleri

| Komut | Kullanım |
|:---|:---|
| `run <isim> [...]` | Alt-sürece başlatır |
| `stop <isim> [...]` | SIGTERM ile durdurur |
| `restart <isim>` | stop + run |
| `list` / `ls` | Mevcut stratejileri gösterir |
| `status` | Ayrıntılı orkestrasyon raporu |
| `help` | Kullanım |

### Komut Dosyası

Komut dosyası: `/tmp/strategy_cmd.d/cmd_<n>.cmd` (STRATEGY_CMD_DIR). Her bir satır bir komutdur.

Dosyaya yazılan komutlar:
- `strat run breakout momentum`
- `strat stop breakout`
- `strat restart breakout`

Komut dosyası okunur `spawn_cmd_file_poller` (orchestrator.rs:36-62):
- 250 ms'de bir okur.
- Satırları `Input::Command` olarak kanala basar.
- Dosyayı siler.

### Giriş Noktası

`spawn_stdin_reader` (orchestrator.rs:65-91):
- rustyline REPL: `strategy> ` prompt.
- Yalnızca stdin girişi işler, file kuyruğu kullanmaz.

---

## Süreç 6: Status ve Rapor

`status()` (orchestrator.rs:251-298):

- Unicode emoji li: 🟢 Running, 🔴 Stopped, 🟡 Failed, 🟠 Stopped (çalışıyor).
- Alt-sürece birden fazla (matching): listelenir.
- `pid` ve `exit code` için bilgiler sunulur.

---

## Satır Sayıları

| Dosya | Satır |
|:---|:---|
| strategies-engine/Cargo.toml | 9 |
| strategies-engine/src/lib.rs | 5 |
| strategies-engine/src/orchestrator.rs | 442 |
| strategies-engine/src/trait_def.rs | 27 |
| **strategies-engine toplam** | **483** |

---

## Sonuç

strategies-engine, strateji orkestrasyon kütüphanesi:
- Ortak strateji sözleşmesi (trait_def.rs::Strategy) — Signal enum, FillReport struct, Strategy trait'i.
- Orkestrasyon çekirdeği (orchestrator.rs:442 satır) — durum makinesi, alt-sürece spawn/kill, reap, komut ayrıştırma, durum raporu.
- Takma ad çözümleme (resolve_strategy) — `breakout` → `breakout-strategy`.
- Komut kuyruğu (spawn_cmd_file_poller, 250 ms) — /tmp/strategy_cmd.d/*.cmd.
- Alt-sürece yaşam döngüsü: run → spawn, stop → SIGTERM, tick → reap.

strategy-console binary'i:
- strategy-console.rs:11-13 → engine::engine::strategy_console::run_strategy_console() çağırır.
- 1 thread + 2 yardımcı thread (spawn_cmd_file_poller, spawn_stdin_reader).
- 500 ms tick döngüsü: orch.tick() → reap.