# 🔧 Additional Services — Tam Kaynak Kodu + Detaylı Analiz

> `additional-services/`. Bu doküman dizin ağacını, klasör/dosya sözlüğünü, her dosyanın **tam kaynak kodunu** ve **detaylı analizini** (mermaid akış diyagramlarıyla) içerir. Tarih: 2026-08-09

---

## 📂 İçindekiler

- [Dizin Ağacı](#dizin-agac)
- [Klasör ve Dosya Sözlüğü](#klasor-ve-dosya-sozlugu)
- [Detaylı Analiz (mermaid)](#detayl-analiz-mermaid)
- [Tam Kaynak Kodu](#tam-kaynak-kodu)

---

## 🌳 Dizin Ağacı

```
additional-services/
    ├── config/config_v5.toml
    ├── config/config_v6.toml
    ├── formal_verification/CycleFinance.cfg
    ├── formal_verification/CycleFinance.tla
    ├── k8s/chaos_dns_failure.yaml
    ├── k8s/chaos_network_partition.yaml
    ├── k8s/chaos_ntp_drift.yaml
    ├── k8s/deployment.yaml
    ├── os-utils/Cargo.toml
        ├── os-utils/src/config.rs
        ├── os-utils/src/lib.rs
    ├── scripts/cycle_env.sh
    ├── scripts/cycle_tmux.sh
    ├── scripts/exec_setup.sh
    ├── scripts/gdpr_erasure_test.sh
    ├── scripts/monitor.sh
    ├── scripts/start_paper.sh
    ├── scripts/stop_paper.sh
    ├── scripts/tmux_clipboard_paste.sh
```

---

## 📖 Klasör ve Dosya Sözlüğü

> `additional-services/` — **Genel amaç:** Yardımcı hizmetler: OS seviyesinde config (os-utils), çoklu servis orchestration (scripts/cycle_tmux.sh), canlı izleme (monitor.sh), paper trade başlat/durdur, KVKK/GDPR silme testi ve ortam kurulum betikleri.
| Klasör / Dosya | Anlamı |
|---|---|
| `additional-services/` | Cycle Finance HFT motorunun operasyon katmanı: yapılandırma, shell orkestrasyonu, OS seviyesi gerçek zamanlı yardımcılar, izleme, kaos/uyum testleri ve biçimsel doğrulama |
| `config/` | API uç noktası ve trading parametrelerini tutan TOML yapılandırma dizini |
| `config/config_v5.toml` | Binance WS v5 uç noktası (`wss://stream.binance.com:9443/ws`) ve `max_positions=100` tanımı |
| `config/config_v6.toml` | Blue/Green dağıtım için Binance WS v6 uç noktası (`.../ws/v6`) ve aynı `max_positions=100` |
| `os-utils/` | İşletim sistemi seviyesi Rust yardımcı kütüphanesi (gerçek zamanlı öncelik + kilitsiz config) |
| `os-utils/Cargo.toml` | os-utils kutusunun workspace tabanlı `libc` ve `crossbeam` bağımlılıkları |
| `os-utils/src/config.rs` | Epoch-tabanlı kilitsiz (lock-free) `GlobalConfig` ve `ConfigManager` (okuma + atomik değiştirme) |
| `os-utils/src/lib.rs` | Geçerli iş parçacığını Linux'ta `SCHED_FIFO` gerçek zamanlı planlayıcıya yükselten yardımcı |
| `scripts/` | Tmux tabanlı servis yönetimi, izleme ve uyum testleri için shell betikleri dizini |
| `scripts/cycle_env.sh` | 1200+ satırlık shell komut kütüphanesi (tüm servislerin start/stop/status, REST sorguları, listener parametreleri) |
| `scripts/cycle_tmux.sh` | 14 pencereli tek-sekme tmux başlatıcı; tüm motoru tek komutla derler, açar ve bağlanır |
| `scripts/exec_setup.sh` | Binance Futures API anahtarlarını ekrana yazdırmadan `.env`'e (izin 600) güvenle kaydeden betik |
| `scripts/gdpr_erasure_test.sh` | KVKK/GDPR "silme hakkı" protokolünü hash + ClickHouse mutation simülasyonuyla test eden betik |
| `scripts/monitor.sh` | CPU/RAM/GPU ve 6 servisin durumunu bar grafikleriyle saniyede bir çizen izleme paneli |
| `scripts/start_paper.sh` | DATA terminali + paper-service'i arka planda tek komutla derleyip başlatan betik |
| `scripts/stop_paper.sh` | DATA terminali + paper-service'i durdurup paylaşımlı ring buffer'ları temizleyen betik |
| `scripts/tmux_clipboard_paste.sh` | Wayland/X11 panosundaki metni tmux paste buffer'a yükleyip yapıştıran betik (Ctrl+V) |
| `k8s/` | Kubernetes + Chaos Mesh dağıtım ve dayanıklılık testi manifestleri |
| `k8s/deployment.yaml` | `SYS_NICE` (SCHED_FIFO için) yetkili, 4 CPU/4Gi limitli core Deployment'ı |
| `k8s/chaos_dns_failure.yaml` | Binance DNS uç noktalarına 5 dk arıza enjekte eden DNSChaos deneyi (30 dk'da bir) |
| `k8s/chaos_network_partition.yaml` | core ↔ redis-cluster arasında 10 sn ağ bölünmesi enjekte eden NetworkChaos deneyi |
| `k8s/chaos_ntp_drift.yaml` | Sistem saatine +10 sn kayma enjekte eden TimeChaos (NTP kayması) deneyi |
| `formal_verification/` | TLA+ biçimsel doğrulama dosyaları |
| `formal_verification/CycleFinance.cfg` | TLC model checker: `Spec`, `Safety` invarianti ve `Liveness` özelliği |
| `formal_verification/CycleFinance.tla` | Lock-free tick kuyruğunun Safety (tick düşmemesi) ve Liveness (ölü kilitlenme yok) kanıtı modeli |

---

## 🔬 Detaylı Analiz (mermaid)

> Her dosyanın **detaylı açıklaması**, **algoritmik akış diyagramı** (mermaid) ve **"neden kullandık"** gerekçesi. Mermaid diyagramları HTML'de otomatik çizilir. Tarih: 2026-08-09

### `scripts/cycle_tmux.sh`
**Detaylı açıklama:** `cycle`, `kill`, `status`, `attach` alt komutlarıyla çok amaçlı bir tmux başlatıcıdır. Session zaten varsa doğrudan bağlanır; yoksa `cargo build` ile splash, core, paper, alert, breakout, stream-ohlcv, ai-engine ve exec-console paketlerini derler, eski süreçleri ve `/dev/shm` ring buffer'larını temizler. Ardından her servisi ayrı bir pencerede (STRATEGY→LISTENER→RISK→SHELL→DATA→ALERT→PAPER→Monitor→DETECT-MS→BREAKOUT→STREAM-OHLCV→CALC-IND→AI→CONSOLE) sıralı gecikmelerle `tmux send-keys` ile başlatır. Son adımda Ctrl+V pano yapıştırma bağını, fare desteğini ve Matrix yeşili status barı yapılandırıp session'a bağlanır.
**Neden kullandık:**
- Tüm motor tek komutla (`cycle-start`) deterministik sırada ayağa kalkar; her servis kendi penceresinde log izlenebilir.
- `set -euo pipefail` ile hata güvenliği; `full_cleanup` kill'de kalıntı süreç ve paylaşımlı bellek bırakmaz.
- `BIN_DIR`/`CYCLE_ROOT` değişkenleriyle hem kaynak ağacı hem kurulu paket dağıtımı desteklenir.
- Window 0'daki STRATEGY ile Window 4'teki DATA aynı `core` binary'sinin farklı `RUN_MODE` modlarıdır; aynı paylaşımlı hafıza üzerinden haberleşir.

```mermaid
flowchart TD
    A["cycle_tmux.sh çağrısı"] --> B{"alt komut?"}
    B -->|"kill"| C["full_cleanup: tmux kill + core/paper/alert pkill + ring buffer sil"]
    B -->|"status"| D["tmux pencerelerini listele + servis CPU/RAM"]
    B -->|"attach"| E["tmux attach-session"]
    B -->|"varsayılan başlat"| F{"session 'cycle' var mı?"}
    F -->|"evet"| E
    F -->|"hayır"| G["cargo build release 8 paket + splash"]
    G --> H["eski süreçler pkill + /dev/shm temizliği"]
    H --> I["/tmp/cycle_init.sh üret: env + source cycle_env + help"]
    I --> J["tmux new-session -x 220 -y 50"]
    J --> K["Pencere 0 STRATEGY (RUN_MODE=STRATEGY core)"]
    K --> L["Pencere 1 LISTENER"]
    L --> M["Pencere 2 RISK (--watch)"]
    M --> N["Pencere 3 SHELL: source cycle_init.sh"]
    N --> O["Pencere 4 DATA (RUN_MODE=DATA core)"]
    O --> P["Pencere 5 ALERT + Pencere 6 PAPER"]
    P --> Q["Pencere 7 Monitor: bash monitor.sh"]
    Q --> R["Pencere 8-11: DETECT-MS, BREAKOUT, STREAM-OHLCV, CALC-IND"]
    R --> S["Pencere 12 AI + Pencere 13 EXEC CONSOLE"]
    S --> T["mouse aç, C-v paste bind, matrix status bar"]
    T --> U["select-window 0 + attach-session"]
```

### `scripts/monitor.sh`
**Detaylı açıklama:** Saniyede bir yenilenen, cursor'u taşımadan (`tput cup 0 0`) ekranı yeniden çizen bir izleme panosudur. Önce AMD GPU kartını sysfs'ten bulur, ardından sistem geneli CPU/RAM/GPU kullanımını `/proc/meminfo`, `top` ve sysfs'ten okuyup `draw_bar` ile renkli bar grafiklerine çevirir. Servis satırlarında `core` süreci `/proc/<pid>/environ` içindeki `RUN_MODE` değişkenine göre DATA/STRATEGY/BACKTEST/CORRELATION olarak ayrıştırılır; paper-service ve alert-service `pgrep` ile bulunup `ps`'ten CPU/RSS/VSZ okunur. Altta `/dev/shm` ring buffer'larının varlığı ve boyutu gösterilir.
**Neden kullandık:**
- HFT'de gecikme kalıcılığı kritiktir; CPU/RAM artışı ve ring buffer varlığı tek ekranda sürekli izlenir.
- `find_pid_env` ile aynı binary'nin hangi modda çalıştığı güvenilir şekilde ayırt edilir (ps çıktısında görünmeyen env).
- `trap` ile çıkışta cursor geri getirilir; `MONITOR_INTERVAL` ile yenileme hızı ayarlanabilir.
- Yüksek kullanımda renk kırmızıya döner; sorun anında fark edilir.

```mermaid
flowchart TD
    A["monitor.sh başlat"] --> B["tput civis + trap cursor"]
    B --> C["GPU kartını sysfs'te bul"]
    C --> D["sonsuz döngü: tput cup 0 0"]
    D --> E["Sistem özeti: CPU idle + /proc/meminfo + gpu_busy_percent"]
    E --> F["find_pid_env: core'u RUN_MODE'a göre ayır"]
    F --> G["find_pid: paper-service, alert-service"]
    G --> H{"servis PID canlı mı?"}
    H -->|"hayır"| I["KAPALI + kırmızı boş bar"]
    H -->|"evet"| J["ps: pid, pcpu, rss, vsz"]
    J --> K["draw_bar: %80+ kırmızı, %50+ sarı"]
    I --> L["ring buffer /dev/shm durumu + çalışan servis sayısı"]
    K --> L
    L --> M["sleep INTERVAL"]
    M --> D
```

### `os-utils/src/config.rs`
**Detaylı açıklama:** Tick döngüsünde mutex yerine `crossbeam::epoch` kullanan kilitsiz yapılandırma deposudur. `ConfigManager` bir `Atomic<GlobalConfig>` tutar; `read_config` epoch guard ile (Acquire) yükleyip `as_ref` ile güvenli referans döner, böylece config okurken belleği korur. `swap_config` yeni yapılandırmayı `Owned` ile atomik `swap` (Release) eder ve eski işaretçiyi `guard.defer_destroy` ile, o epoch'u kimse tutmayınca güvenle çöp toplatır. Böylece use-after-free riski, kilit olmadan ortadan kalkar.
**Neden kullandık:**
- Gerçek zamanlı HFT tick döngüsünde mutex alınması jitter üretir; epoch tabanlı reclamation kilit gerektirmez.
- Konfig değişimi (örn. v5→v6 API geçişi) çalışan sistemde durmadan atomik yapılabilir.
- `Ordering::Acquire/Release` ile okuma-yazma tutarlılığı garanti altına alınır.

```mermaid
flowchart TD
    A["ConfigManager::new(initial)"] --> B["Atomic GlobalConfig'e ilk config yaz"]
    C["read_config guard"] --> D["load Ordering::Acquire"]
    D --> E["unsafe ptr.as_ref → &GlobalConfig"]
    F["swap_config yeni config"] --> G["epoch::pin + Owned::new"]
    G --> H["current_config.swap Release"]
    H --> I{"eski ptr null mu?"}
    I -->|"hayır"| J["guard.defer_destroy(eski ptr)"]
    I -->|"evet"| K["eski yok, GC gerekmez"]
    J --> L["println: config kilitsiz değişti"]
    E --> M["guard scope dışında config otomatik korunur"]
```

### `os-utils/src/lib.rs`
**Detaylı açıklama:** `set_rt_thread_priority(priority)` fonksiyonu, yalnızca Linux'ta mevcut iş parçacığını `sched_setscheduler(0, SCHED_FIFO, ...)` ile gerçek zamanlı planlayıcıya yükseltir; `0` hedefi çağıran iş parçacığını temsil eder. Başarısızlık durumunda (CAP_SYS_NICE/root gerekmez) hata mesajı bastırılır, sistem kilitlenmez. Linux dışı platformlarda güvenli bir no-op uyarısı verir.
**Neden kullandık:**
- İcra ve veri işleme iş parçacıklarına deterministik zamanlama kazandırmak için SCHED_FIFO gerekir.
- Hata yutularak sistemin geri kalanı asla engellenmez; k8s Deployment'ı `SYS_NICE` ile bu yetkiyi sağlar.

```mermaid
flowchart TD
    A["set_rt_thread_priority p"] --> B{"target_os == linux?"}
    B -->|"hayır"| C["no-op uyarısı yazdır"]
    B -->|"evet"| D["sched_param {sched_priority: p}"]
    D --> E["sched_setscheduler 0, SCHED_FIFO"]
    E --> F{"sonuç 0 mı?"}
    F -->|"evet"| G["başarılı: SCHED_FIFO p"]
    F -->|"hayır"| H["err: CAP_SYS_NICE veya root gerekli"]
```

### `scripts/cycle_env.sh`
**Detaylı açıklama:** Motorun kalbi olan 1200 satırlık shell kütüphanesidir; tmux panellerinde elle servis yönetimi, REST sorguları ve parametre değişikliği için 100+ fonksiyon tanımlar. `CYCLE_ROOT`/`CYCLE_API`/kimlik değişkenlerini tanımlar, `help-cycle` ile renkli komut rehberi basar. Her start fonksiyonu önce `_start_guard` ile kendini yeniden source eder (eski tanım kullanılmaz), `_core_mode_pid` ile core'un `/proc/<pid>/environ` içindeki `RUN_MODE`'unu okur, ardından `_tmux_pane` ile hedef pencere/pane'e `C-c`, `C-u` ve komutu gönderir. Stop fonksiyonları TERM→KILL ikilisini, sorgular curl+`python3 -m json.tool` kullanır; `exec-*` serisi kill switch dosyasıyla acil durdurma sağlar.
**Neden kullandık:**
- HFT motorunda her servisi elle ayrı terminalde başlatmak hata üretir; tek kaynaktan standart start/stop/status tanımları verir.
- `_start_guard` sayesinde tmux paneli eski sürümü source etse bile daima güncel fonksiyonlar çalışır.
- JWT tabanlı paper REST erişimi, listener metrik parametreleri ve AI HITL onayı gibi operasyon ihtiyaçlarını tek yerden karşılar.

```mermaid
flowchart TD
    A["source cycle_env.sh"] --> B["CYCLE_ROOT + API + kimlik varsayılanları"]
    B --> C["renk kodları + help-cycle"]
    D{"kullanıcı komutu"} --> E{"start mı?"}
    E -->|"evet"| F["_start_guard → kendini yeniden source"]
    F --> G["_core_mode_pid / pgrep: zaten çalışıyor mu?"]
    G -->|"evet"| H["uyarı + return 1"]
    G -->|"hayır"| I["cargo build -p paket + ring temizliği"]
    I --> J["_tmux_pane: C-c + C-u + komut gönder"]
    J --> K["pgrep/curl doğrula + başarı mesajı"]
    E -->|"hayır"| L{"stop mu?"}
    L -->|"evet"| M["pgrep bul → TERM → sleep → KILL"]
    L -->|"hayır"| N{"sorgu mu?"}
    N -->|"evet"| O["_cycle_token JWT al → curl REST → json.tool"]
    N -->|"hayır"| P["parametre/exec-*/alert-* yardımcıları"]
```

### `scripts/exec_setup.sh`
**Detaylı açıklama:** Binance Futures API anahtarlarını güvenli şekilde `.env`'e yazan betiktir; `read -s` ile anahtar ekrana yazılmaz ve dosya `chmod 600` ile yalnızca sahibine açılır. Pano yapıştırmasında oluşan bracketed-paste artıkları (`\e[200~`/`\e[201~`), CR/LF kalıntıları temizlenir, değerler kırpılır ve maskeli doğrulama sonrası "EVET" onayı alınır. `--show` maskeli görüntü, `--testnet` testnet URL'lerini yazıp `EXEC_DRY_RUN=true` yapar. Güvenlik varsayılanı olarak anahtar girişinde de `EXEC_DRY_RUN=true` set edilir; gerçek emir için ayrıca onay gerekir.
**Neden kullandık:**
- HFT icra motorunun canlı borsa anahtarları ekranda görünmemeli ve dosya izniyle korunmalıdır.
- Yapıştırma kaynaklı CRLF/escape karakterleri `.env`'i bozabilir; tampon boşaltma + temizlik bunu engeller.
- DRY_RUN varsayılanı, kazara gerçek emir gönderimini iki adımlı onaya bağlar.

```mermaid
flowchart TD
    A["exec_setup.sh"] --> B{"argüman?"}
    B -->|"--show"| C[".env'deki anahtarları maskeli listele"]
    B -->|"--testnet"| D["EXEC_BASE_URL/WS_URL testnet + DRY_RUN=true"]
    B -->|"yok"| E["drain_input + read -s BINANCE_API_KEY"]
    E --> F["drain_input + read -s BINANCE_SECRET_KEY"]
    F --> G["escape/CRLF temizle + trim"]
    G --> H["maskeli doğrulama göster"]
    H --> I{"'EVET' onayı?"}
    I -->|"hayır"| J["iptal, hiçbir şey yazılmaz"]
    I -->|"evet"| K["write_env: KEY + EXEC_MODE=LIVE + DRY_RUN=true"]
    K --> L["chmod 600 .env"]
    L --> M["özet: gerçek emir için ayrıca onay gerek"]
```

### `scripts/start_paper.sh`
**Detaylı açıklama:** Paper sistemini tek komutla ayağa kaldırır: önce `core` ve `paper-service` derlenir, eski süreçler `pkill` ile kapatılır ve tick ring'leri silinir. Ardından `setsid ... &` ile DATA terminali (`RUN_MODE=DATA core`) arka planda `/tmp/data_terminal.log`'a, paper-service de JWT kimlik/başlangıç bakiyesi/veritabanı yollarıyla `/tmp/paper_service.log`'a yazılarak başlatılır. Son adımda REST API, metrik ve CLI kullanım örnekleri ekrana dökülür.
**Neden kullandık:**
- Hızlı manuel test için tmux yerine arka plan (`setsid`/`disown`) çalıştırma seçeneği sunar.
- Ring buffer temizliği, farklı kapasitede başlatmada tutarsızlığı önler.
- Log dosyalarına yönlendirme, tmux olmadan da tanılama imkânı verir.

```mermaid
flowchart TD
    A["start_paper.sh"] --> B["cargo build core + paper-service"]
    B --> C["pkill core, paper-service, paper_cli"]
    C --> D["rm /dev/shm ring buffer'ları"]
    D --> E["setsid RUN_MODE=DATA core → /tmp/data_terminal.log"]
    E --> F["setsid paper-service env'leri → /tmp/paper_service.log"]
    F --> G["sleep 4"]
    G --> H["sağlık/API/CLI kullanım raporu"]
```

### `scripts/stop_paper.sh`
**Detaylı açıklama:** `paper-service` ve `core` süreçlerini `pkill` ile kapatıp paylaşımlı hafıza ring'lerini siler. Her adımın sonucu kullanıcıya bildirilir.
**Neden kullandık:**
- `start_paper.sh`'in simetriğidir; kalıntı süreç ve bellek bırakmadan temiz kapanış sağlar.

```mermaid
flowchart TD
    A["stop_paper.sh"] --> B["pkill paper-service"]
    B --> C["pkill core"]
    C --> D["rm /dev/shm/cycle_finance_ring + orders"]
    D --> E["Done"]
```

### `scripts/gdpr_erasure_test.sh`
**Detaylı açıklama:** KVKK/GDPR "unutulma hakkı" sürecini simüle eder: kullanıcı ID'si tuzla birleştirilip SHA3-256 ile hash'lenir, hash ClickHouse mutation'ı (`ALTER TABLE ticks DELETE`) için hedef alınır, ardından Merkle ağacı ile fiziksel silme doğrulanır ve olay 3 yıllık uyum süresi için `deletion_registry_mock.log`'a yazılır.
**Neden kullandık:**
- HFT motorunun kişisel veri işleme sürecinde silme protokolünün çalıştığını otomatik doğrular.
- Hash tabanlı kimlikleme sayesinde gerçek kimlik kullanılmadan süreç test edilir.

```mermaid
flowchart TD
    A["GDPR simülasyonu"] --> B["USER_ID + SALT → sha3sum 256"]
    B --> C["User Hash üret"]
    C --> D["ClickHouse ALTER DELETE mutation gönder"]
    D --> E["Merkle Tree ile silme doğrula"]
    E --> F["deletion_registry'ye 3 yıllık kayıt"]
    F --> G["protokol tamam"]
```

### `scripts/tmux_clipboard_paste.sh`
**Detaylı açıklama:** tmux'ta Ctrl+V/Ctrl+Shift+V bağıyla tetiklenen, OS panosundaki metni yapıştıran betiktir. Wayland'de `wl-paste`, yoksa X11'de `xclip`/`xsel` sırasıyla denenir; panodaki son satır sonları (CR/LF) temizlenir ki sonraki `read` boş yutmasın. Metin `tmux load-buffer` ile yüklenip `tmux paste-buffer` ile yapıştırılır.
**Neden kullandık:**
- `exec_setup.sh`'teki anahtar girişi sırasında pano yapıştırmayı hem Wayland hem X11'de çalışır hale getirir.
- Artık satır sonlarını temizleyerek güvenli `read -s` girişini korur.

```mermaid
flowchart TD
    A["C-v bağı → betik"] --> B{"wl-paste + WAYLAND_DISPLAY?"}
    B -->|"evet"| C["wl-paste -p > tmp"]
    B -->|"hayır"| D{"xclip var?"}
    D -->|"evet"| E["xclip clipboard > tmp"]
    D -->|"hayır"| F{"xsel var?"}
    F -->|"evet"| G["xsel -b -o > tmp"]
    C --> H["CR/LF temizle (perl/sed)"]
    E --> H
    G --> H
    H --> I["tmux load-buffer + paste-buffer"]
    I --> J["rm tmp"]
```

### `config/config_v5.toml`
**Detaylı açıklama:** Cycle Finance 2.0'ın v5 API yapılandırmasıdır; Binance stream WebSocket uç noktasını (`wss://stream.binance.com:9443/ws`) ve trading için `max_positions=100` sınırını tanımlar.
**Neden kullandık:**
- Motorun `GlobalConfig`'ine örnek ilk değer sağlar ve API sürümünün kilitsiz şekilde değiştirilebilmesini test eder.

```mermaid
flowchart TD
    A["config_v5.toml"] --> B["[api] version v5"]
    B --> C["endpoint binance.com:9443/ws"]
    C --> D["[trading] max_positions 100"]
```

### `config/config_v6.toml`
**Detaylı açıklama:** Blue/Green dağıtım senaryosu için v6 uç noktasına (`wss://stream.binance.com:9443/ws/v6`) işaret eden yapılandırma; aynı `max_positions=100` ile `swap_config` üzerinden çalışırken geçiş yapılabilir.
**Neden kullandık:**
- v5→v6 geçişinin `ConfigManager` kilitsiz swap mekanizmasıyla sıfır kesintiyle test edilebilmesini sağlar.

```mermaid
flowchart TD
    A["config_v6.toml"] --> B["[api] version v6"]
    B --> C["endpoint binance.com:9443/ws/v6"]
    C --> D["[trading] max_positions 100"]
```

### `os-utils/Cargo.toml`
**Detaylı açıklama:** `os-utils` crate'ini tanımlar; `libc` (SCHED_FIFO syscall'ları için) ve `crossbeam` (epoch tabanlı kilitsiz yapılandırma için) workspace bağımlılıklarını kullanır.
**Neden kullandık:**
- Workspace'ten bağımlılık çekerek sürüm tutarlılığı sağlar; HFT'ye özgü iki düşük seviye kütüphaneyi paketler.

```mermaid
flowchart TD
    A["os-utils Cargo.toml"] --> B["libc: sched_setscheduler"]
    A --> C["crossbeam: epoch reclamation"]
```

### `k8s/deployment.yaml`
**Detaylı açıklama:** `cycle-finance-core` Deployment'ı tek replika ile çalışır; `SYS_NICE` capability'si ekleyerek SCHED_FIFO gerçek zamanlı planlamayı, cgroup v2 annotation'ı ile kaynak yönetimini zorunlu kılar. 4 CPU/4Gi limit ve `RUST_LOG=info` tanımlıdır.
**Neden kullandık:**
- Gerçek zamanlı önceliğin konteyner içinde de çalışması için `os-utils`'ın ihtiyaç duyduğu yetki manifest ile verilir.
- Kaos deneylerinin hedeflediği `app: cycle-finance` etiketini sağlar.

```mermaid
flowchart TD
    A["deployment.yaml"] --> B["Deployment cycle-finance-core, 1 replika"]
    B --> C["SYS_NICE capability (SCHED_FIFO için)"]
    C --> D["limits: 4 CPU, 4Gi"]
    D --> E["cgroup v2 annotation + RUST_LOG info"]
```

### `k8s/chaos_dns_failure.yaml`
**Detaylı açıklama:** Chaos Mesh `DNSChaos` deneyidir; `api.binance.com` ve `stream.binance.com` için 5 dakikalık DNS hatası enjekte eder, cron ile 30 dakikada bir tekrarlar.
**Neden kullandık:**
- Borsa DNS'inin çökmesi durumunda motorun yeniden bağlanma davranışını test eder.

```mermaid
flowchart TD
    A["DNSChaos"] --> B["action error, mode all"]
    B --> C["pattern: api/stream.binance.com"]
    C --> D["duration 5m, cron @every 30m"]
```

### `k8s/chaos_network_partition.yaml`
**Detaylı açıklama:** `NetworkChaos` ile core ile redis-cluster arasında çift yönlü (both) 10 saniyelik ağ bölünmesi oluşturur, cron ile 5 dakikada bir tekrarlar.
**Neden kullandık:**
- Veri katmanıyla bağlantı koptuğunda servislerin (özellikle kuyruklama) davranışını sınar.

```mermaid
flowchart TD
    A["NetworkChaos"] --> B["action partition, direction both"]
    B --> C["hedef: redis-cluster"]
    C --> D["duration 10s, cron @every 5m"]
```

### `k8s/chaos_ntp_drift.yaml`
**Detaylı açıklama:** `TimeChaos` ile tüm `cycle-finance` podlarının saatine +10 saniye kayma (NTP drift) uygular; 5 dakika sürer, 15 dakikada bir tekrarlar.
**Neden kullandık:**
- HFT'de zaman damgaları ve kandil/tick sıralaması kritik olduğundan saat kaymasına dayanıklılığı test eder.

```mermaid
flowchart TD
    A["TimeChaos"] --> B["timeOffset +10s (NTP drift)"]
    B --> C["duration 5m, cron @every 15m"]
```

### `formal_verification/CycleFinance.tla`
**Detaylı açıklama:** Lock-free MPMC tick kuyruğunun TLA+ modelidir: üretici `Produce` sınırsız olmaması için 1000 tick sınırıyla kuyruğa tick ekler, tüketici `Consume` kuyruktan kilit olmadan çeker. `Safety` (işlenen tick, üretilen tick'i asla aşamaz) ve `Liveness` (her üretilen tick sonunda tüketilir) özellikleri TLC ile kanıtlanır; `WF_vars(Consume)` ile zayıf adalet garanti edilir.
**Neden kullandık:**
- Gerçek zamanlı döngüde ölü kilit (liveness) ve tick kaybı (safety) hatasının tasarım aşamasında kanıtlanmasını sağlar.

```mermaid
flowchart TD
    A["Init: kuyruk boş, ticks 0"] --> B{"Next = Produce veya Consume"}
    B -->|"Produce"| C["Append tick, ticks_in+1 (bounded 1000)"]
    B -->|"Consume"| D["Tail, ticks_out+1"]
    C --> E["Safety: ticks_out ≤ ticks_in"]
    D --> E
    E --> F["Liveness: ticks_in=n ~> ticks_out=n"]
    F --> G["Spec = Init ∧ [Next]_vars ∧ WF_vars(Consume)"]
```

### `formal_verification/CycleFinance.cfg`
**Detaylı açıklama:** TLC model checker için yapılandırmadır; `SPECIFICATION Spec`, `INVARIANT Safety` ve `PROPERTY Liveness` tanımlayarak modelin tam kapsamda doğrulanmasını sağlar.
**Neden kullandık:**
- Kanıtlanacak özellikleri tek dosyada sabitler; TLC çalıştırması deterministik hale gelir.

```mermaid
flowchart TD
    A["CycleFinance.cfg"] --> B["SPECIFICATION Spec"]
    B --> C["INVARIANT Safety"]
    C --> D["PROPERTY Liveness"]
    D --> E["TLC ile doğrula"]
```

**Özet:** 19 dosya analiz edildi, 19 mermaid diyagramı üretildi (4 kritik: `cycle_tmux.sh`, `monitor.sh`, `os-utils/config.rs`, `cycle_env.sh`).

---

## 📄 Tam Kaynak Kodu

### `additional-services/config/config_v5.toml`

```toml
# API v5 Configuration for Cycle Finance 2.0
[api]
version = "v5"
endpoint = "wss://stream.binance.com:9443/ws"

[trading]
max_positions = 100
```

### `additional-services/config/config_v6.toml`

```toml
# API v6 Configuration for Cycle Finance 2.0 (Blue/Green Deployment)
[api]
version = "v6"
endpoint = "wss://stream.binance.com:9443/ws/v6"

[trading]
max_positions = 100
```

### `additional-services/formal_verification/CycleFinance.cfg`

```ini
SPECIFICATION Spec
INVARIANT Safety
PROPERTY Liveness
```

### `additional-services/formal_verification/CycleFinance.tla`

```
--------------------------- MODULE CycleFinance ---------------------------
EXTENDS Naturals, Sequences, TLC

(* 
  TLA+ Model for Cycle Finance Lock-Free Tick Processing.
  Proves that ticks produced by the network adapter are eventually consumed
  by the core without deadlocks (Liveness) and without dropping (Safety).
*)

VARIABLES 
    queue,       \* Lock-free MPMC queue (flume)
    ticks_in,    \* Total ticks generated
    ticks_out    \* Total ticks processed

vars == <<queue, ticks_in, ticks_out>>

Init == 
    /\ queue = <<>>
    /\ ticks_in = 0
    /\ ticks_out = 0

(* Producer adds a tick to the queue *)
Produce == 
    /\ ticks_in < 1000  \* Bounded model checking
    /\ queue' = Append(queue, "tick")
    /\ ticks_in' = ticks_in + 1
    /\ UNCHANGED <<ticks_out>>

(* Consumer processes a tick from the queue lock-free *)
Consume == 
    /\ queue # <<>>
    /\ queue' = Tail(queue)
    /\ ticks_out' = ticks_out + 1
    /\ UNCHANGED <<ticks_in>>

Next == Produce \/ Consume

(* Safety: Processed ticks never exceed produced ticks *)
Safety == ticks_out <= ticks_in

(* Liveness: Every produced tick is eventually consumed *)
Liveness == \A n \in Nat : (ticks_in = n) ~> (ticks_out = n)

Spec == Init /\ [][Next]_vars /\ WF_vars(Consume)

=============================================================================
```

### `additional-services/k8s/chaos_dns_failure.yaml`

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: DNSChaos
metadata:
  name: cycle-finance-dns-failure
  namespace: default
spec:
  action: error
  mode: all
  selector:
    labelSelectors:
      app: cycle-finance
  patterns:
    - api.binance.com
    - stream.binance.com
  duration: '5m'
  scheduler:
    cron: '@every 30m'
```

### `additional-services/k8s/chaos_network_partition.yaml`

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: cycle-finance-network-partition
  namespace: default
spec:
  action: partition
  mode: all
  selector:
    labelSelectors:
      app: cycle-finance
  direction: both
  target:
    selector:
      labelSelectors:
        app: redis-cluster
    mode: all
  duration: '10s'
  scheduler:
    cron: '@every 5m'
```

### `additional-services/k8s/chaos_ntp_drift.yaml`

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: TimeChaos
metadata:
  name: cycle-finance-ntp-drift
  namespace: default
spec:
  mode: all
  selector:
    labelSelectors:
      app: cycle-finance
  timeOffset: '10s' # Simulate NTP drift of 10 seconds ahead
  duration: '5m'
  scheduler:
    cron: '@every 15m'
```

### `additional-services/k8s/deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cycle-finance-core
  labels:
    app: cycle-finance
spec:
  replicas: 1
  selector:
    matchLabels:
      app: cycle-finance
  template:
    metadata:
      labels:
        app: cycle-finance
      annotations:
        # Require cgroups v2 resource management
        kubernetes.io/cgroup-version: "v2"
    spec:
      containers:
      - name: core
        image: cycle-finance/core:latest
        resources:
          limits:
            cpu: "4"
            memory: "4Gi"
          requests:
            cpu: "4"
            memory: "4Gi"
        securityContext:
          capabilities:
            add:
              - SYS_NICE # Required for SCHED_FIFO real-time thread scheduling
        env:
          - name: RUST_LOG
            value: "info"
```

### `additional-services/os-utils/Cargo.toml`

```toml
[package]
name = "os-utils"
version = "0.1.0"
edition = "2021"

[dependencies]
libc = { workspace = true }
crossbeam = { workspace = true }
```

### `additional-services/os-utils/src/config.rs`

```rust
use crossbeam::epoch::{self, Atomic, Owned};
use std::sync::atomic::Ordering;

/// System configuration using lock-free epoch-based reclamation.
/// Prevents use-after-free without using Mutex/RwLock in the tick loop.
pub struct GlobalConfig {
    pub max_positions: usize,
    pub active_api_version: &'static str,
}

pub struct ConfigManager {
    // crossbeam_epoch::Atomic provides safe, lock-free memory reclamation
    current_config: Atomic<GlobalConfig>,
}

impl ConfigManager {
    pub fn new(initial: GlobalConfig) -> Self {
        Self {
            current_config: Atomic::new(initial),
        }
    }

    /// Read configuration. The returned guard ensures the config is not dropped
    /// while the current thread is holding it (epoch pinning).
    pub fn read_config<'a>(&'a self, guard: &'a epoch::Guard) -> &'a GlobalConfig {
        let ptr = self.current_config.load(Ordering::Acquire, guard);
        unsafe { ptr.as_ref().unwrap() }
    }

    /// Swap configuration globally. Old config is queued for garbage collection
    /// once no threads are pinning the epoch.
    pub fn swap_config(&self, new_config: GlobalConfig) {
        let guard = epoch::pin();
        let new_ptr = Owned::new(new_config);
        
        let old_ptr = self.current_config.swap(new_ptr, Ordering::Release, &guard);
        
        if !old_ptr.is_null() {
            unsafe {
                // Queue the old configuration for deletion safely.
                guard.defer_destroy(old_ptr);
            }
        }
        println!("Config: Successfully swapped lock-free configuration.");
    }
}
```

### `additional-services/os-utils/src/lib.rs`

```rust
#![allow(unsafe_code)]
pub mod config;

#[cfg(target_os = "linux")]
use libc::{sched_param, sched_setscheduler, SCHED_FIFO};

/// Safely sets the current thread to the SCHED_FIFO real-time scheduler.
/// On non-Linux platforms or if permissions are lacking, it logs a warning.
pub fn set_rt_thread_priority(priority: i32) {
    #[cfg(target_os = "linux")]
    {
        let param = sched_param {
            sched_priority: priority,
        };
        
        let result = unsafe {
            // 0 means the calling thread
            sched_setscheduler(0, SCHED_FIFO, &param)
        };
        
        if result != 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("Failed to set SCHED_FIFO (requires CAP_SYS_NICE or root): {}", err);
        } else {
            println!("Thread successfully elevated to SCHED_FIFO with priority {}", priority);
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("set_rt_thread_priority is a no-op on non-Linux platforms.");
    }
}
```

### `additional-services/scripts/cycle_env.sh`

```bash
#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — Shell Yardımcı Komutları
#  Bu dosya cycle_tmux.sh tarafından otomatik source edilir.
#  Elle de kullanılabilir: source <proje-koku>/additional-services/scripts/cycle_env.sh
# ============================================================

# ── Kök dizini otomatik bul ──────────────────────────────────
CYCLE_ROOT="${CYCLE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
CYCLE_API="${CYCLE_API:-http://127.0.0.1:8080}"
CYCLE_USER="${CYCLE_USER:-admin}"
CYCLE_PASS="${CYCLE_PASS:-changeme123}"

# ── Renk kodları ─────────────────────────────────────────────
_G='\033[0;32m'; _Y='\033[1;33m'; _C='\033[0;36m'
_B='\033[1;34m'; _W='\033[1;37m'; _R='\033[0;31m'
_D='\033[2m';    _N='\033[0m'

# ============================================================
#  KOMUT REHBERİ
# ============================================================
help-cycle() {
  echo ""
  echo -e "${_W}╔══════════════════════════════════════════════════════════════════╗${_N}"
  echo -e "${_W}║        🏛️  CYCLE FINANCE — KOMUT REHBERİ                        ║${_N}"
  echo -e "${_W}╚══════════════════════════════════════════════════════════════════╝${_N}"

  echo -e "\n${_Y}━━━  🔧 SİSTEM YÖNETİMİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_G}cycle-start${_N}          Tüm terminalleri yeniden başlat"
  echo -e "  ${_G}cycle-kill${_N}           Tüm terminalleri ve servisleri kapat"
  echo -e "  ${_G}cycle-status${_N}         Çalışan servislerin CPU/RAM durumu"
  echo -e "  ${_G}cycle-build${_N}          Projeyi derle (cargo build)"
  echo -e "  ${_G}cycle-build-full${_N}     Tam set derle (--features full)"

  echo -e "\n${_Y}━━━  ⚙️  SİSTEMLERİ TEK TEK AÇ / KAPAT  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_G}data-start${_N} / ${_R}data-stop${_N}          DATA terminali (Binance WS)"
  echo -e "  ${_G}strategy-start${_N} / ${_R}strategy-stop${_N}  STRATEGY terminali (PyO3)"
  echo -e "  ${_G}paper-start${_N} / ${_R}paper-stop${_N}        Paper-service (REST :8080)"
  echo -e "  ${_G}alert-start${_N} / ${_R}alert-stop${_N}        Alert-service"
  echo -e "  ${_G}listener-start${_N} / ${_R}listener-stop${_N}  Listener (anlık metrik analizi)"
  echo -e "  ${_G}detect-ms-start${_N} / ${_R}detect-ms-stop${_N}  MSMP analiz motoru (:3002)"
  echo -e "  ${_G}calc-ind-start${_N} / ${_R}calc-ind-stop${_N}    İndikatör hesaplama motoru (:3007)"
  echo -e "  ${_G}breakout-start${_N} / ${_R}breakout-stop${_N}    VELVETUSDT kırılım stratejisi"
  echo -e "  ${_G}stream-ohlcv-start${_N} / ${_R}stream-ohlcv-stop${_N}  Canlı OHLCV mum akışı (:3008)"

  echo -e "\n${_Y}━━━  🤖 AI ENGINE (LLM Agent Katmanı)  ━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}ai-start${_N}           AI Engine'i başlat (ai.toml + OpenAI/Anthropic)"
  echo -e "  ${_R}ai-stop${_N}            Durdur"
  echo -e "  ${_C}ai-status${_N}          Çalışıyor mu? CPU/RAM + son döngü"
  echo -e "  ${_C}ai-approve${_N}         HITL modunda bekleyen emri onayla (echo approve)"
  echo -e "  ${_C}ai-reject${_N}          HITL modunda bekleyen emri reddet"
  echo -e "  ${_C}ai-log${_N}             Canlı log izle"

  echo -e "\n${_Y}━━━  🖥️  EXEC CONSOLE (Execution Engine elle komut)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}exec-console-start${_N}   Konsolu tmux sekmesinde başlat (executiond :3010)"
  echo -e "  ${_R}exec-console-stop${_N}    Durdur"
  echo -e "  ${_C}exec-console-status${_N}  Çalışıyor mu? CPU/RAM"
  echo -e "  ${_C}exec-console-log${_N}     Konsol penceresine geç (Ctrl+B → 13)"

  echo -e "\n${_Y}━━━  🛰️  LISTENER  (Anlık Metrik Analizi)  ━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}listener-start${_N}      Pane 0.2'de başlat"
  echo -e "  ${_C}listener-stop${_N}       Durdur"
  echo -e "  ${_C}listener-status${_N}     Çalışıyor mu? CPU/RAM"
  echo -e "  ${_C}listenconfig-list${_N}   Metrik parametrelerini göster"
  echo -e "  ${_C}listenconfig-set KEY VAL${_N}  Parametre değiştir (lambda, k_abs, gamma...) "
  echo -e "  ${_C}listenconfig-reset${_N}  Varsayılanlara dön"
  echo -e "  ${_C}listener-log${_N}        Metrik çıktısını izle (/tmp/listener_metrics.json)"

  echo -e "\n${_Y}━━━  ⚠️  RİSK ANALİZİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}risk-start${_N}           Pane 0.1'de başlat (5 sn yenileme)"
  echo -e "  ${_C}risk-worker-start${_N}    risk-worker daemon'ı başlat (korelasyon/VaR)"
  echo -e "  ${_C}risk-stop${_N}            Durdur"
  echo -e "  ${_C}risk-query${_N}           Tek seferlik analiz çalıştır"

  echo -e "\n${_Y}━━━  💹 PRICE-FEED  (WS→Ring, Anlık Last/Mark/Index)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}pricefeed-start${_N}     Arka planda başlat (:3004)"
  echo -e "  ${_C}pricefeed-stop${_N}      Durdur"
  echo -e "  ${_C}pricefeed-status${_N}    Çalışıyor mu? CPU/RAM + health"
  echo -e "  ${_C}pricefeed-query SYM${_N} Tek sembol sorgula (örn. pricefeed-query VELVETUSDT)"
  echo -e "  ${_C}pricefeed-log${_N}       Canlı log izle"

  echo -e "\n${_Y}━━━  📡 DATA TERMİNALİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}data-live${_N}            Canlı Binance WS başlat (RUN_MODE=DATA)"
  echo -e "  ${_C}data-backtest${_N}        CSV backtest başlat"
  echo -e "  ${_C}data-log${_N}             Data terminal logunu izle"

  echo -e "\n${_Y}━━━  🛡️  PAPER SERVICE (REST API)  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}paper-health${_N}         Sistem sağlık kontrolü"
  echo -e "  ${_C}paper-balance${_N}        Bakiye ve equity bilgisi"
  echo -e "  ${_C}paper-positions${_N}      Açık pozisyonlar"
  echo -e "  ${_C}paper-orders${_N}         Açık emirler"
  echo -e "  ${_C}paper-history${_N}        İşlem geçmişi"
  echo -e "  ${_C}paper-metrics${_N}        Prometheus metrikleri (ham)"
  echo -e "  ${_C}paper-log${_N}            Paper service logunu izle"

  echo -e "\n${_Y}━━━  📋 EMİR İŞLEMLERİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}paper-buy  BTCUSDT 0.001${_N}   Market BUY emri"
  echo -e "  ${_C}paper-sell BTCUSDT 0.001${_N}   Market SELL emri"
  echo -e "  ${_C}paper-cli  [arglar]${_N}         Paper CLI (tüm seçenekler)"

  echo -e "\n${_Y}━━━  🛡️ EXECUTION ENGINE (Canlı Binance :3010)  ━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}exec-setup${_N}          Anahtar gir (ekrana yazılmaz, .env 600)"
  echo -e "  ${_C}exec-show${_N}           Yapılandırma göster (anahtarlar maskeli)"
  echo -e "  ${_C}exec-testnet${_N}        Testnet URL'leri yaz"
  echo -e "  ${_C}exec-dry${_N}            executiond DRY_RUN'da başlat (emir gitmez)"
  echo -e "  ${_R}exec-live${_N}           Gerçek emir modu ('GO' onayı ister)"
  echo -e "  ${_C}exec-stop${_N}           executiond durdur"
  echo -e "  ${_C}exec-status${_N}         Mod + risk durumu"
  echo -e "  ${_C}exec-account / exec-positions / exec-balance / exec-orders${_N}"
  echo -e "  ${_R}exec-kill / exec-unkill${_N}  Kill switch aç/kapat (acil durum)"

  echo -e "\n${_Y}━━━  🧠 STRATEGY / CORRELATION  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}strategy-start${_N}       Strategy terminalini başlat (arka plan)"
  echo -e "  ${_C}strategy-stop${_N}        Strategy terminalini durdur"
  echo -e "  ${_C}correlation-start${_N}    Korelasyon analizini başlat"

  echo -e "\n${_Y}━━━  🔔 ALERT SERVİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}alert-list${_N}           Aktif uyarıları listele"
  echo -e "  ${_C}alert-add VELVETUSDT above 0.22 \"ses\"${_N}   Yeni alarm ekle"
  echo -e "  ${_C}alert-update SYM cond OLD NEW${_N}   Alarmı güncelle"
  echo -e "  ${_C}alert-remove SYM cond PRICE${_N}     Alarmı sil"
  echo -e "  ${_C}alert-reload${_N}         Alert servisini yeniden başlat"

  echo -e "\n${_Y}━━━  📈 DETECT-MS  (Market Structure Engine :3002)  ━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}detect-ms-start${_N}      Servisi arka planda başlat (port 3002)"
  echo -e "  ${_C}detect-ms-stop${_N}       Servisi durdur"
  echo -e "  ${_C}detect-ms-status${_N}     Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}detect-ms-query${_N}      BTCUSDT 15m analiz (JSON çıktı)"
  echo -e "  ${_C}detect-ms-query ETHUSDT 1h 500${_N}   Özel sorgu"
  echo -e "  ${_C}detect-ms-log${_N}        Canlı log izle"

  echo -e "\n${_Y}━━━  🎯 VELVETUSDT KIRILIM STRATEJİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}breakout-start${_N}        Stratejiyi başlat (VELVETUSDT 1m, 100 pencere)"
  echo -e "  ${_C}breakout-stop${_N}         Stratejiyi durdur"
  echo -e "  ${_C}breakout-status${_N}       Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}breakout-query${_N}        Tek seferlik analiz (emir açmaz)"
  echo -e "  ${_C}breakout-query --dry-run${_N}  Analiz + kırılım simülasyonu"
  echo -e "  ${_C}breakout-wait 600${_N}     Bekleme süresini ayarla (saniye)"
  echo -e "  ${_C}breakout-log${_N}          Canlı strateji logu izle"

  echo -e "\n${_Y}━━━  📡 STREAM-OHLCV  (Canlı OHLCV Mum Akışı :3008)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}stream-ohlcv-start${_N}    Servisi başlat (ring: /dev/shm/cycle_finance_stream_ohlcv)"
  echo -e "  ${_C}stream-ohlcv-stop${_N}     Servisi durdur"
  echo -e "  ${_C}stream-ohlcv-status${_N}   Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}stream-ohlcv-start-stream SYM ITV START_MS${_N}   Stream aç (örn. BTCUSDT 60 0)"
  echo -e "  ${_C}stream-ohlcv-streams${_N}  Aktif stream'leri listele"
  echo -e "  ${_C}stream-ohlcv-query SYM ITV START_MS${_N}   Stream aç + durum göster"

  echo -e "\n${_Y}━━━  📊 İZLEME  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}monitor-start${_N}        İzleme paneline geç (Ctrl+B → 4)"

  echo -e "\n${_Y}━━━  🗄️  VERİTABANI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}db-trades${_N}            Son 20 işlemi göster"
  echo -e "  ${_C}db-size${_N}              Veritabanı boyutu"

  echo -e "\n${_Y}━━━  🌐 TMUX KISAYOLLARI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_B}Ctrl+B → ok tuşu${_N}     Panel değiştir"
  echo -e "  ${_B}Ctrl+B → z${_N}           Paneli tam ekran yap / küçült"
  echo -e "  ${_B}Ctrl+B → d${_N}           Session'ı arka plana al"
  echo -e "  ${_B}Ctrl+B → 0${_N}           Trading sekmesi (4 panel)"
  echo -e "  ${_B}Ctrl+B → 1${_N}           📡 DATA sekmesi"
  echo -e "  ${_B}Ctrl+B → 2${_N}           🔔 ALERT sekmesi"
  echo -e "  ${_B}Ctrl+B → 3${_N}           🛡️ PAPER sekmesi"
  echo -e "  ${_B}Ctrl+B → 4${_N}           Monitor sekmesi"
  echo -e "  ${_B}Ctrl+B → 5${_N}           DETECT-MS sekmesi"
  echo -e "  ${_B}Ctrl+B → 6${_N}           VELVETUSDT sekmesi"
  echo -e "  ${_B}Ctrl+B → 7${_N}           STREAM-OHLCV sekmesi"
  echo -e "  ${_B}Fare tıklama/scroll${_N}  Panel seç / scroll"

  echo -e "\n${_W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_D}help-cycle yazarak bu listeye tekrar ulaşabilirsin.${_N}"
  echo ""
}

# ============================================================
#  SİSTEM YÖNETİMİ
# ============================================================
# Bu dosya değiştiğinde fonksiyonları güncellemek için:
reload-cycle() {
  source "$CYCLE_ROOT/additional-services/scripts/cycle_env.sh" >/dev/null 2>&1
  echo "✅ cycle_env.sh yeniden yüklendi"
}

# Her start/stop fonksiyonunun güncel sürümü kullanması için otomatik yenileme
# (tmux SHELL paneli eski sürümü yüklemiş olsa bile sorun yaşanmaz)
_start_guard() {
  source "$CYCLE_ROOT/additional-services/scripts/cycle_env.sh" >/dev/null 2>&1
}
cycle-start() {
  "$CYCLE_ROOT/additional-services/scripts/cycle_tmux.sh"
}
cycle-kill() {
  "$CYCLE_ROOT/additional-services/scripts/cycle_tmux.sh" kill
}
cycle-status() {
  "$CYCLE_ROOT/additional-services/scripts/cycle_tmux.sh" status
}
cycle-build() {
  cd "$CYCLE_ROOT" && cargo build -p core -p paper-service -p alert-service -p breakout-strategy
}
cycle-build-full() {
  cd "$CYCLE_ROOT" && cargo build -p paper-service --features full
}

# ============================================================
#  SİSTEMLERİ TEK TEK AÇ / KAPAT  (4 panelli Trading penceresi)
#  DATA, ALERT ve PAPER ayrı sekme (pencere) olarak açılır.
#  Her servis kendi pane'inde başlar.
# ============================================================
# Yardımcı: Trading penceresindeki bir pane'e komut gönder
# Servis → hedef: 0.0=STRATEGY 0.2=LISTENER 0.1=RISK 0.3=SHELL
#                1=DATA sekmesi  2=ALERT sekmesi  3=PAPER sekmesi
_tmux_pane() {
  local name="$1"; shift
  local session="cycle"
  local pane
  case "$name" in
    "📡DATA")   pane="1" ;;
    "🛡️PAPER")  pane="3" ;;
    "🧠STRATEGY") pane="0.0" ;;
    "🔔ALERT")  pane="2" ;;
    "🛰️LISTENER") pane="0.2" ;;
    "⚠️RISK")  pane="0.1" ;;
    "💻SHELL")  pane="0.3" ;;
    "📡STREAM-OHLCV") pane="7" ;;
    *)
      # Tanınmayan → yeni pencere (ör. DETECT-MS, VELVETUSDT)
      if ! tmux has-session -t "$session" 2>/dev/null; then
        tmux new-session -d -s "$session" -x 220 -y 50
        tmux rename-window -t "$session:0" "Trading"
      fi
      local idx
      idx=$(tmux list-windows -t "$session" -F "#{window_name} #{window_index}" 2>/dev/null | awk -v n="$name" '$1==n{print $2}')
      if [ -z "$idx" ]; then
        tmux new-window -t "$session" -n "$name"
        idx=$(tmux list-windows -t "$session" -F "#{window_name} #{window_index}" 2>/dev/null | awk -v n="$name" '$1==n{print $2}')
      fi
      tmux send-keys -t "$session:$idx" "$@"
      return 0
      ;;
  esac
  tmux send-keys -t "$session:$pane" C-c
  tmux send-keys -t "$session:$pane" C-u
  tmux send-keys -t "$session:$pane" "$@"
}

# ── DATA terminali (Binance WS → ring) ──────────────────────
# RUN_MODE env değişkeni ps'de görünmez → /proc/*/environ ile kontrol et
_core_mode_pid() {
  local mode="$1"
  for p in $(pgrep -x core 2>/dev/null); do
    if tr '\0' '\n' < "/proc/$p/environ" 2>/dev/null | grep -q "^RUN_MODE=$mode$"; then
      echo "$p"
      return 0
    fi
  done
  return 1
}

data-start() {
  _start_guard
  if _core_mode_pid DATA &>/dev/null; then echo "⚠️  DATA zaten çalışıyor (pid: $(_core_mode_pid DATA))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p core 2>&1 | tail -1
  rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders
  _tmux_pane "📡DATA" "cd $CYCLE_ROOT && RUN_MODE=DATA ./target/debug/core" Enter
  echo "✅ DATA başlatıldı (sekme 1 — 📡 DATA)"
}
data-stop() {
  _start_guard
  local p; p=$(_core_mode_pid DATA)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; echo "✅ DATA durduruldu [pid:$p]"; else echo "ℹ️  DATA çalışmıyor"; fi
}

# ── STRATEGY terminali (core) ────────────────────────────────
strategy-start() {
  _start_guard
  if _core_mode_pid STRATEGY &>/dev/null; then echo "⚠️  STRATEGY zaten çalışıyor (pid: $(_core_mode_pid STRATEGY))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p core 2>&1 | tail -1
  _tmux_pane "🧠STRATEGY" "cd $CYCLE_ROOT && RUN_MODE=STRATEGY ./target/debug/core" Enter
  echo "✅ STRATEGY başlatıldı (pane 0.0)"
}
strategy-stop() {
  _start_guard
  local p; p=$(_core_mode_pid STRATEGY)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; echo "✅ STRATEGY durduruldu [pid:$p]"; else echo "ℹ️  STRATEGY çalışmıyor"; fi
}

# ── PAPER-SERVICE (REST API :8080) ───────────────────────────
paper-start() {
  _start_guard
  if pgrep -x "paper-service" &>/dev/null; then echo "⚠️  paper-service zaten çalışıyor"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p paper-service 2>&1 | tail -1
  rm -rf "$CYCLE_ROOT/data-engine/data/paper_wal"
  _tmux_pane "🛡️PAPER" \
    "cd $CYCLE_ROOT && PAPER_ADMIN_USER=${PAPER_ADMIN_USER:-admin} PAPER_ADMIN_PASS=${PAPER_ADMIN_PASS:-changeme123} PAPER_API_ADDR=${PAPER_API_ADDR:-127.0.0.1:8080} PAPER_INITIAL_USDT=${PAPER_INITIAL_USDT:-100000} PAPER_DB_PATH=$CYCLE_ROOT/data-engine/data/paper_live.db PAPER_SLED_PATH=$CYCLE_ROOT/data-engine/data/paper_wal ./target/debug/paper-service" \
    Enter
  echo "✅ PAPER-SERVICE başlatıldı (sekme 3 — 🛡️ PAPER, http://127.0.0.1:8080)"
}
paper-stop() {
  _start_guard
  local p; p=$(pgrep -x paper-service 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ paper-service durduruldu [pid:$p]"; else echo "ℹ️  paper-service çalışmıyor"; fi
}

# ── ALERT-SERVICE ────────────────────────────────────────────
alert-start() {
  _start_guard
  if pgrep -x "alert-service" &>/dev/null; then echo "⚠️  alert-service zaten çalışıyor"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p alert-service 2>&1 | tail -1
  _tmux_pane "🔔ALERT" "cd $CYCLE_ROOT && ./target/debug/alert-service --config $CYCLE_ROOT/alerts.toml" Enter
  echo "✅ ALERT-SERVICE başlatıldı (sekme 2 — 🔔 ALERT)"
}
alert-stop() {
  _start_guard
  local p; p=$(pgrep -x alert-service 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ alert-service durduruldu [pid:$p]"; else echo "ℹ️  alert-service çalışmıyor"; fi
}

# ── RISK-WORKER (Soğuk yol parametre üretici — korelasyon/VaR) ──
risk-worker-start() {
  _start_guard
  if pgrep -x risk-worker &>/dev/null; then echo "⚠️  risk-worker zaten çalışıyor (pid: $(pgrep -x risk-worker | head -1))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p risk-engine 2>&1 | tail -1
  _tmux_pane "🧮RISK-WORKER" "cd $CYCLE_ROOT && ./target/debug/risk-worker" Enter
  sleep 2
  if pgrep -x risk-worker &>/dev/null; then
    echo "✅ RISK-WORKER başlatıldı (http://127.0.0.1:3011/healthz)"
  else
    echo "❌ RISK-WORKER başlatılamadı"
  fi
}
risk-worker-stop() {
  _start_guard
  local p; p=$(pgrep -x risk-worker 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ risk-worker durduruldu [pid:$p]"; else echo "ℹ️  risk-worker çalışmıyor"; fi
}
risk-worker-status() {
  _start_guard
  local p; p=$(pgrep -x risk-worker 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then echo "✅ RISK-WORKER ÇALIŞIYOR [pid:$p]"; else echo "✘  risk-worker durdurulmuş"; fi
}

# ── LISTENER (Anlık Metrik Analizi, pane 0.1) ──────────
listener-start() {
  _start_guard
  if pgrep -x listener &>/dev/null; then
    echo "⚠️  listener zaten çalışıyor (pid: $(pgrep -x listener | head -1))"
    return 1
  fi
  # Bağımlılık: paper-service gerekli
  if ! pgrep -x paper-service &>/dev/null; then
    echo "⚠️  paper-service çalışmıyor — önce paper-start ile başlatın"
    return 1
  fi
  _tmux_pane "🛰️LISTENER" "cd $CYCLE_ROOT && $CYCLE_ROOT/target/release/listener" Enter
  sleep 2
  if pgrep -x listener &>/dev/null; then
    echo "✅ LISTENER başlatıldı (pane 0.2)"
  else
    echo "❌ LISTENER başlatılamadı"
  fi
}
listener-stop() {
  _start_guard
  local p; p=$(pgrep -x listener 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    pkill -TERM -x listener 2>/dev/null
    sleep 1
    pkill -KILL -x listener 2>/dev/null || true
    echo "✅ LISTENER durduruldu [pid:$p]"
  else
    echo "ℹ️  LISTENER çalışmıyor"
  fi
}
listener-status() {
  _start_guard
  local p; p=$(pgrep -x listener 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    local cpu mem
    cpu=$(ps -p "$p" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$p" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ LISTENER ÇALIŞIYOR  [pid:$p  CPU:${cpu}%  RAM:${mem}]"
  else
    echo "✘  LISTENER durdurulmuş"
  fi
}
listener-log() {
  tail -f /tmp/listener_metrics.json 2>/dev/null || echo "metrik dosyası yok"
}

# ── RISK (Anlık risk analizi, pane 0.3) ──────────────────────
risk-start() {
  _start_guard
  if pgrep -x risk_analysis &>/dev/null; then
    echo "⚠️  RISK zaten çalışıyor (pid: $(pgrep -x risk_analysis | head -1))"
    return 1
  fi
  _tmux_pane "⚠️RISK" "cd $CYCLE_ROOT && ./target/release/risk_analysis --watch" Enter
  sleep 2
  echo "✅ RISK başlatıldı (pane 0.1)"
}
risk-stop() {
  _start_guard
  local p; p=$(pgrep -x risk_analysis 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    pkill -TERM -x risk_analysis 2>/dev/null; sleep 1
    pkill -KILL -x risk_analysis 2>/dev/null || true
    echo "✅ RISK durduruldu [pid:$p]"
  else
    echo "ℹ️  RISK çalışmıyor"
  fi
}
risk-status() {
  _start_guard
  local p; p=$(pgrep -x risk_analysis 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    echo "✅ RISK ÇALIŞIYOR [pid:$p]"
  else
    echo "✘  RISK durdurulmuş"
  fi
}
risk-query() {
  _start_guard
  cd "$CYCLE_ROOT" && ./target/release/risk_analysis
 }

# ── Listener metrik parametreleri (shell'den ayarlanabilir) ──
# Config dosyası: /tmp/listener_metrics.conf (çalışan listener 5 sn'de bir yeniden okur)
LISTEN_CONF=/tmp/listener_metrics.conf

# listenconfig-list  → tüm parametreleri göster
# listenconfig-set lambda 0.02   → parametre değiştir
# listenconfig-reset          → varsayılanlara dön
listenconfig-list() {
  _start_guard
  local conf="$LISTEN_CONF"
  if [ -f "$conf" ]; then
    echo "=== Listener metrik parametreleri ($conf) ==="
    cat "$conf"
  else
    echo "ℹ️  Config dosyası yok — varsayılanlar kullanılıyor:"
    echo "  lambda = 0.015        (WLOBI decay)"
    echo "  theta_vol = 2.5       (Delta velocity eşiği)"
    echo "  alpha_bucket = 0.75   (aVPIN bucket sabiti)"
    echo "  k_abs = 100           (absorption penceresi, trade)"
    echo "  n_bucket = 50         (aVPIN bucket sayısı)"
    echo "  ice_threshold = 1.2   (Iceberg eşiği)"
    echo "  efp_threshold = 0.05  (Execution footprint eşiği)"
    echo "  noise_corr = 0.85     (Lee-Ready gürültü filtresi)"
    echo "  delta_window_sec = 60 (ΔV penceresi, saniye)"
    echo "  tps_window_sec = 10  (TPS penceresi, saniye)"
    echo "  corr_price_window_sec = 5 (fiyat korelasyonu penceresi, saniye)"
    echo "  corr_vol_window_sec = 5   (hacim korelasyonu penceresi, saniye)"
    echo "  gamma0..gamma5        (Alpha Basket ağırlıkları)"
  fi
}

listenconfig-set() {
  _start_guard
  local key="${1:-}" val="${2:-}"
  if [ -z "$key" ] || [ -z "$val" ]; then
    echo "Kullanım: listenconfig-set <key> <value>"
    echo "Örn: listenconfig-set lambda 0.02 | listenconfig-set k_abs 200"
    echo "     listenconfig-set gamma1 0.5 | listenconfig-set delta_window_sec 120"
    return 1
  fi
  local valid_keys="lambda theta_vol alpha_bucket k_abs n_bucket ice_threshold efp_threshold noise_corr delta_window_sec tps_window_sec corr_price_window_sec corr_vol_window_sec gamma0 gamma1 gamma2 gamma3 gamma4 gamma5"
  if ! echo "$valid_keys" | grep -qw "$key"; then
    echo "❌ Geçersiz parametre: $key"
    echo "Geçerli: $valid_keys"
    return 1
  fi
  # k_abs, n_bucket, delta_window_sec tam sayı olmalı
  if echo "k_abs n_bucket delta_window_sec tps_window_sec corr_price_window_sec corr_vol_window_sec" | grep -qw "$key"; then
    if ! echo "$val" | grep -qE '^[0-9]+$'; then
      echo "❌ $key tam sayı olmalı"; return 1
    fi
  else
    if ! echo "$val" | grep -qE '^-?[0-9]+(\.[0-9]+)?$'; then
      echo "❌ $key sayı olmalı"; return 1
    fi
  fi
  # Eski değeri değiştir veya ekle
  if grep -q "^${key} *=" "$LISTEN_CONF" 2>/dev/null; then
    sed -i "s|^${key} *=.*|${key} = ${val}|" "$LISTEN_CONF"
  else
    echo "${key} = ${val}" >> "$LISTEN_CONF"
  fi
  echo "✅ $key = $val kaydedildi ($LISTEN_CONF)"
  echo "   Çalışan listener 5 sn'de bir yeniden okur. list-restart ile hemen uygula."
}

listenconfig-reset() {
  _start_guard
  rm -f "$LISTEN_CONF"
  echo "✅ Varsayılan parametrelere dönüldü (config dosyası silindi)"
}

# Kısayollar
listener-config() { listenconfig-list; }
listener-set() { listenconfig-set "$@"; }

# ── PRICE-FEED (WS → ring buffer, anlık last/mark/index price) ──
pricefeed-start() {
  _start_guard
  if pgrep -x "" &>/dev/null; then
    echo "⚠️  zaten çalışıyor (pid: $(pgrep -x | head -1))"
    return 1
  fi
  cd "$CYCLE_ROOT" && cargo build -p 2>&1 | tail -1
  setsid nohup "$CYCLE_ROOT/target/debug/" > /tmp/price_feed.log 2>&1 < /dev/null &
  sleep 3
  if curl -s -m 2 http://127.0.0.1:3004/health >/dev/null 2>&1; then
    echo "✅ PRICE-FEED başlatıldı → http://127.0.0.1:3004/api/lastprice"
  else
    echo "❌ PRICE-FEED başlatılamadı:"; tail -5 /tmp/price_feed.log
  fi
}
pricefeed-stop() {
  _start_guard
  local p; p=$(pgrep -x "" 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ durduruldu [pid:$p]"; else echo "ℹ️  çalışmıyor"; fi
}
pricefeed-status() {
  _start_guard
  local p; p=$(pgrep -x "" 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    local cpu mem
    cpu=$(ps -p "$p" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$p" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ PRICE-FEED ÇALIŞIYOR  [pid:$p  CPU:${cpu}%  RAM:${mem}]"
    curl -s -m 2 http://127.0.0.1:3004/health
    echo
  else
    echo "✘  PRICE-FEED durdurulmuş"
  fi
}
pricefeed-query() {
  _start_guard
  local sym="${1:-BTCUSDT}"
  curl -s -m 3 "http://127.0.0.1:3004/api/lastprice/$sym" | python3 -m json.tool 2>/dev/null \
    || echo "❌ Servis yanıt vermiyor — pricefeed-start ile başlat."
}
pricefeed-log() {
  tail -f /tmp/price_feed.log
}

# ============================================================
#  DATA TERMİNALİ
# ============================================================
data-live() {
  cd "$CYCLE_ROOT" && RUN_MODE=DATA ./target/debug/core
}
data-backtest() {
  cd "$CYCLE_ROOT" && RUN_MODE=BACKTEST CSV_PATH="./test_data.csv" ./target/debug/core
}
data-log() {
  tail -f /tmp/data_terminal.log
}

# ============================================================
#  PAPER SERVICE — JWT otomatik alınır
# ============================================================
_cycle_token() {
  curl -s -X POST "$CYCLE_API/api/v1/auth/login" \
    -H 'Content-Type: application/json' \
    -d "{\"username\":\"$CYCLE_USER\",\"password\":\"$CYCLE_PASS\"}" \
    2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('access_token',''))" 2>/dev/null
}

paper-health() {
  curl -s "$CYCLE_API/api/v1/system/health" | python3 -m json.tool
}
paper-balance() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/balance" | python3 -m json.tool
}
paper-positions() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/positions" | python3 -m json.tool
}
paper-orders() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/orders" | python3 -m json.tool
}
paper-history() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/trade-history" | python3 -m json.tool
}
paper-metrics() {
  curl -s "$CYCLE_API/metrics"
}
paper-log() {
  tail -f /tmp/paper_service.log
}

paper-buy() {
  local sym="${1:-BTCUSDT}" qty="${2:-0.001}"
  local tok; tok=$(_cycle_token)
  local oid="cli-$(date +%s)"
  curl -s -X POST \
    -H "Authorization: Bearer $tok" \
    -H 'Content-Type: application/json' \
    -d "{\"symbol\":\"$sym\",\"side\":\"BUY\",\"order_type\":\"MARKET\",\"quantity\":$qty,\"client_order_id\":\"$oid\"}" \
    "$CYCLE_API/api/v1/order" | python3 -m json.tool
}
paper-sell() {
  local sym="${1:-BTCUSDT}" qty="${2:-0.001}"
  local tok; tok=$(_cycle_token)
  local oid="cli-$(date +%s)"
  curl -s -X POST \
    -H "Authorization: Bearer $tok" \
    -H 'Content-Type: application/json' \
    -d "{\"symbol\":\"$sym\",\"side\":\"SELL\",\"order_type\":\"MARKET\",\"quantity\":$qty,\"client_order_id\":\"$oid\"}" \
    "$CYCLE_API/api/v1/order" | python3 -m json.tool
}
paper-cli() {
  "$CYCLE_ROOT/target/debug/paper_cli" \
    --api "$CYCLE_API" --user "$CYCLE_USER" --password "$CYCLE_PASS" "$@"
}

# ============================================================
#  STRATEGY / CORRELATION
# ============================================================
# Not: strategy-start/stop artık "SİSTEMLERİ TEK TEK AÇ/KAPAT" bölümünde
# (arka planda, pid dosyalı). correlation-start foreground çalıştırır.
correlation-start() {
  cd "$CYCLE_ROOT" && RUN_MODE=CORRELATION ./target/debug/core
}

# ============================================================
#  ALERT SERVİSİ
# ============================================================
alert-list() {
  echo "=== alerts.toml — aktif uyarılar ==="
  "$CYCLE_ROOT/target/debug/alerts" list
  echo ""
  echo "Kullanım:"
  echo "  alert-add VELVETUSDT above 0.22 [voice metni] [cooldown]"
  echo "  alert-update VELVETUSDT above 0.21628 0.22 [voice] [cooldown]"
  echo "  alert-remove VELVETUSDT above 0.21628"
}
alert-reload() {
  pkill -x alert-service 2>/dev/null || true
  sleep 1
  cd "$CYCLE_ROOT" && nohup ./target/debug/alert-service --config ./alerts.toml > /tmp/alert_service.log 2>&1 &
  echo "✅ Alert servisi yeniden başlatıldı (pid: $!)"
}

# ── Alarm yönetimi (shell'den) — değişiklik sonrası otomatik reload ──
_alert_apply() {
  local msg="$1"
  echo "$msg"
  echo "🔄 Alert servisi yeniden yükleniyor..."
  # Eski süreci durdur, tmux pane'inde yeniden başlat
  pkill -x alert-service 2>/dev/null || true
  sleep 1
  tmux send-keys -t "cycle:5" C-c 2>/dev/null
  tmux send-keys -t "cycle:5" "cd $CYCLE_ROOT && ./target/debug/alert-service --config $CYCLE_ROOT/alerts.toml" Enter 2>/dev/null
  sleep 1
  echo "✅ Tamamlandı. alert-list ile görüntüleyin."
}

# Yeni alarm ekle
# Kullanım: alert-add <SYMBOL> <above|below|cross|touch> <PRICE> [voice] [cooldown]
alert-add() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" price="${3:-}" voice="${4:-}" cooldown="${5:-30}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$price" ]; then
    echo "Kullanım: alert-add <SYMBOL> <above|below|cross|touch> <PRICE> [voice metni] [cooldown]"
    return 1
  fi
  local voice_arg=()
  [ -n "$voice" ] && voice_arg=(--voice "$voice")
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" add \
    --symbol "$sym" --condition "$cond" --price "$price" \
    "${voice_arg[@]}" --cooldown "$cooldown")"
}

# Mevcut alarmı güncelle (eski fiyata göre bulur)
# Kullanım: alert-update <SYMBOL> <cond> <OLD_PRICE> <NEW_PRICE> [voice] [cooldown]
alert-update() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" old="${3:-}" new="${4:-}" voice="${5:-}" cooldown="${6:-}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$old" ]; then
    echo "Kullanım: alert-update <SYMBOL> <cond> <OLD_PRICE> [NEW_PRICE] [voice] [cooldown]"
    return 1
  fi
  local args=(--symbol "$sym" --condition "$cond" --old-price "$old")
  [ -n "$new" ] && args+=(--price "$new")
  [ -n "$voice" ] && args+=(--voice "$voice")
  [ -n "$cooldown" ] && args+=(--cooldown "$cooldown")
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" update "${args[@]}")"
}

# Alarm sil
# Kullanım: alert-remove <SYMBOL> <cond> <PRICE>
alert-remove() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" price="${3:-}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$price" ]; then
    echo "Kullanım: alert-remove <SYMBOL> <cond> <PRICE>"
    return 1
  fi
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" remove \
    --symbol "$sym" --condition "$cond" --price "$price")"
}

# ============================================================
#  İZLEME
# ============================================================
monitor-start() {
  if tmux has-session -t cycle 2>/dev/null; then
    tmux select-window -t cycle:7
  else
    "$CYCLE_ROOT/additional-services/scripts/monitor.sh"
  fi
}

# ============================================================
#  VERİTABANI
# ============================================================
db-trades() {
  local url="${TIMESCALEDB_URL:-postgres://cycle:cycle@localhost:5432/market_data}"
  psql "$url" -c \
    "SELECT symbol, price, quantity, timestamp FROM trades ORDER BY timestamp DESC LIMIT 20;" \
    2>/dev/null || echo "DB boş veya bulunamadı."
}
db-size() {
  local url="${TIMESCALEDB_URL:-postgres://cycle:cycle@localhost:5432/market_data}"
  psql "$url" -c \
    "SELECT pg_size_pretty(pg_database_size(current_database())) AS db_size;" \
    2>/dev/null || echo "DB bağlanılamadı."
}

# ============================================================
#  DETECT-MS  —  Market Structure Multi-Protocol Engine
#  REST API: http://127.0.0.1:3002/api/ms?symbol=BTCUSDT&interval=15m
# ============================================================
DETECT_MS_ADDR="${DETECT_MS_ADDR:-127.0.0.1:3002}"

detect-ms-start() {
  _start_guard
  if pgrep -x "detect-ms" &>/dev/null; then
    echo "⚠️  detect-ms zaten çalışıyor (pid: $(pgrep -x detect-ms))"
    echo "   → detect-ms-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/debug/detect-ms" ]; then
    echo "🔨 detect-ms derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p detect-ms 2>&1 | tail -5
  fi

  echo "🚀 detect-ms başlatılıyor → http://$DETECT_MS_ADDR"
  _tmux_pane "📈DETECT-MS" "cd $CYCLE_ROOT && ./target/debug/detect-ms" Enter
  sleep 1
  if pgrep -x detect-ms &>/dev/null; then
    echo "✅ detect-ms başladı [pid: $(pgrep -x detect-ms)]"
    echo "   API: http://$DETECT_MS_ADDR/api/ms?symbol=BTCUSDT&interval=15m"
  else
    echo "❌ detect-ms başlatılamadı."
  fi
}

detect-ms-stop() {
  _start_guard
  if pgrep -x "detect-ms" &>/dev/null; then
    pkill -TERM -x "detect-ms" && echo "✅ detect-ms durduruldu"
  else
    echo "⚠️  detect-ms zaten çalışmıyor"
  fi
}

detect-ms-status() {
  local pid
  pid=$(pgrep -x "detect-ms" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ detect-ms ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$DETECT_MS_ADDR/api/ms?symbol=BTCUSDT&interval=15m"
  else
    echo "✘  detect-ms durdurulmuş"
  fi
}

# ── calc-ind (İndikatör Hesaplama Motoru :3007) ─────────────
calc-ind-start() {
  _start_guard
  if pgrep -x "calc-ind" &>/dev/null; then
    echo "⚠️  calc-ind zaten çalışıyor (pid: $(pgrep -x calc-ind))"
    echo "   → calc-ind-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/debug/calc-ind" ]; then
    echo "🔨 calc-ind derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p calc-ind 2>&1 | tail -5
  fi

  echo "🚀 calc-ind başlatılıyor → http://127.0.0.1:3007"
  _tmux_pane "🧮CALC-IND" "cd $CYCLE_ROOT && ./target/debug/calc-ind" Enter
  sleep 1
  if pgrep -x calc-ind &>/dev/null; then
    echo "✅ calc-ind başladı [pid: $(pgrep -x calc-ind)]"
    echo "   API: http://127.0.0.1:3007/api/calc"
  else
    echo "❌ calc-ind başlatılamadı."
  fi
}

calc-ind-stop() {
  _start_guard
  if pgrep -x "calc-ind" &>/dev/null; then
    pkill -TERM -x "calc-ind" && echo "✅ calc-ind durduruldu"
  else
    echo "⚠️  calc-ind zaten çalışmıyor"
  fi
}

calc-ind-status() {
  local pid
  pid=$(pgrep -x "calc-ind" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ calc-ind ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://127.0.0.1:3007/api/calc"
  else
    echo "✘  calc-ind durdurulmuş"
  fi
}

# ============================================================
#  AI ENGINE (LLM Agent Katmanı — ai.toml + OpenAI/Anthropic)
#  Bağımlılık: (:3004), detect-ms (:3002), calc-ind (:3007), paper (:8080)
# ============================================================
AI_ADDR="${AI_ADDR:-127.0.0.1:3110}"

ai-start() {
  _start_guard
  if pgrep -x "ai-engine" &>/dev/null; then
    echo "⚠️  ai-engine zaten çalışıyor (pid: $(pgrep -x ai-engine | head -1))"
    echo "   → ai-stop ile önce durdur"
    return 1
  fi
  if [ ! -f "$CYCLE_ROOT/target/debug/ai-engine" ]; then
    echo "🔨 ai-engine derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p ai-engine 2>&1 | tail -5
  fi
  echo "🚀 ai-engine başlatılıyor → http://$AI_ADDR"
  _tmux_pane "🤖AI" "cd $CYCLE_ROOT && ./target/debug/ai-engine" Enter
  sleep 1
  if pgrep -x ai-engine &>/dev/null; then
    echo "✅ ai-engine başladı [pid: $(pgrep -x ai-engine | head -1)]"
    echo "   Status: http://$AI_ADDR/api/status"
  else
    echo "❌ ai-engine başlatılamadı. (OPENAI_API_KEY / ANTHROPIC_API_KEY gerekli olabilir)"
  fi
}

ai-stop() {
  _start_guard
  if pgrep -x "ai-engine" &>/dev/null; then
    pkill -TERM -x "ai-engine" && echo "✅ ai-engine durduruldu"
  else
    echo "⚠️  ai-engine zaten çalışmıyor"
  fi
}

ai-status() {
  local pid
  pid=$(pgrep -x "ai-engine" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ ai-engine ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    curl -s -m 2 "http://$AI_ADDR/api/status" | python3 -m json.tool 2>/dev/null \
      || echo "   (status API yanıt vermiyor)"
  else
    echo "✘  ai-engine durdurulmuş"
  fi
}

# HITL onayı — /tmp/ai_approve.txt üzerinden
ai-approve() {
  echo "approve" > /tmp/ai_approve.txt
  echo "✅ Onay verildi — bekleyen emir icra edilecek."
}

ai-reject() {
  echo "reject" > /tmp/ai_approve.txt
  echo "❌ Onay reddedildi."
}

ai-log() {
  # ai-engine tmux içinde çalıştığında log'u tmux penceresinden izlemek daha iyidir.
  echo "ℹ️  ai-engine tmux penceresinde çalışıyor; log için pencereye geçin:"
  echo "   tmux select-window -t cycle:12   (veya Ctrl-b + 12)"
}

# ============================================================
#  EXEC CONSOLE (executiond :3010 elle komut konsolu)
# ============================================================
exec-console-start() {
  _start_guard
  if pgrep -x "exec-console" &>/dev/null; then
    echo "⚠️  exec-console zaten çalışıyor (pid: $(pgrep -x exec-console | head -1))"
    return 1
  fi
  if [ ! -f "$CYCLE_ROOT/target/debug/exec-console" ]; then
    echo "🔨 exec-console derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p exec-console 2>&1 | tail -5
  fi
  echo "🚀 exec-console başlatılıyor (executiond :3010 bağlantılı)..."
  _tmux_pane "🖥️CONSOLE" "cd $CYCLE_ROOT && ./target/debug/exec-console" Enter
  sleep 1
  if pgrep -x exec-console &>/dev/null; then
    echo "✅ exec-console başladı [pid: $(pgrep -x exec-console | head -1)]"
    echo "   Sekme: Ctrl+B → 13  |  Komutlar: help"
  else
    echo "❌ exec-console başlatılamadı. (executiond çalışıyor mu? EXEC_ADMIN_PASS doğru mu?)"
  fi
}

exec-console-stop() {
  _start_guard
  if pgrep -x "exec-console" &>/dev/null; then
    pkill -TERM -x "exec-console" && echo "✅ exec-console durduruldu"
  else
    echo "⚠️  exec-console zaten çalışmıyor"
  fi
}

exec-console-status() {
  local pid
  pid=$(pgrep -x "exec-console" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ exec-console ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   Sekme: Ctrl+B → 13"
  else
    echo "✘  exec-console durdurulmuş"
  fi
}

exec-console-log() {
  echo "ℹ️  Konsol tmux penceresinde çalışıyor; geçmek için:"
  echo "   tmux select-window -t cycle:13   (veya Ctrl-b + 13)"
}

# Sorgu kısayolları
detect-ms-query() {
  # Kullanım: detect-ms-query [SYMBOL] [INTERVAL] [LIMIT]
  local sym="${1:-BTCUSDT}" itv="${2:-15m}" lim="${3:-200}"
  echo "📡 Sorgu: $sym $itv (limit: $lim) → http://$DETECT_MS_ADDR"
  curl -s "http://$DETECT_MS_ADDR/api/ms?symbol=${sym}&interval=${itv}&limit=${lim}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. detect-ms-start ile başlat."
}

detect-ms-log() {
  tail -f /tmp/detect_ms.log
}

# ============================================================
#  STREAM-OHLCV  (stream-ohlcv — canlı OHLCV mum akışı :3008)
#  istek: {symbol, start_ms, interval_secs} → POST /api/stream
#  mumlar binary olarak /dev/shm/cycle_finance_stream_ohlcv ring'ine yazılır.
# ============================================================
STREAM_OHLCV_ADDR="${STREAM_OHLCV_ADDR:-127.0.0.1:3008}"

stream-ohlcv-start() {
  _start_guard
  if pgrep -x stream-ohlcv &>/dev/null; then
    echo "⚠️  stream-ohlcv zaten çalışıyor (pid: $(pgrep -x stream-ohlcv | head -1))"
    return 1
  fi
  if [ ! -f "$CYCLE_ROOT/target/debug/stream-ohlcv" ]; then
    echo "🔨 stream-ohlcv derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p stream-ohlcv 2>&1 | tail -5
  fi
  echo "🚀 stream-ohlcv başlatılıyor → http://$STREAM_OHLCV_ADDR"
  _tmux_pane "📡STREAM-OHLCV" "cd $CYCLE_ROOT && ./target/debug/stream-ohlcv" Enter
  sleep 1
  if pgrep -x stream-ohlcv &>/dev/null; then
    echo "✅ stream-ohlcv başladı [pid: $(pgrep -x stream-ohlcv | head -1)]"
    echo "   POST http://$STREAM_OHLCV_ADDR/api/stream  {symbol, start_ms, interval_secs}"
  else
    echo "❌ stream-ohlcv başlatılamadı."
  fi
}

stream-ohlcv-stop() {
  _start_guard
  if pgrep -x stream-ohlcv &>/dev/null; then
    pkill -TERM -x stream-ohlcv && echo "✅ stream-ohlcv durduruldu"
  else
    echo "⚠️  stream-ohlcv zaten çalışmıyor"
  fi
}

stream-ohlcv-status() {
  local pid
  pid=$(pgrep -x stream-ohlcv 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ stream-ohlcv ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$STREAM_OHLCV_ADDR/api/health"
  else
    echo "✘  stream-ohlcv durdurulmuş"
  fi
}

# Kullanım: stream-ohlcv-start-stream [SYMBOL] [INTERVAL_SN] [START_MS]
stream-ohlcv-start-stream() {
  local sym="${1:-BTCUSDT}" itv="${2:-60}" start="${3:-0}"
  echo "📡 Stream açılıyor: $sym interval=${itv}s start_ms=${start}"
  curl -s -X POST "http://$STREAM_OHLCV_ADDR/api/stream" \
    -H "Content-Type: application/json" \
    -d "{\"symbol\":\"$sym\",\"start_ms\":$start,\"interval_secs\":$itv}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. stream-ohlcv-start ile başlat."
}

# Kullanım: stream-ohlcv-query [SYMBOL] [INTERVAL_SN] [START_MS]
stream-ohlcv-query() {
  local sym="${1:-BTCUSDT}" itv="${2:-60}" start="${3:-0}"
  echo "📡 Sorgu: $sym ${itv}s → http://$STREAM_OHLCV_ADDR"
  curl -s -X POST "http://$STREAM_OHLCV_ADDR/api/stream" \
    -H "Content-Type: application/json" \
    -d "{\"symbol\":\"$sym\",\"start_ms\":$start,\"interval_secs\":$itv}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor."
}

# Kullanım: stream-ohlcv-streams
stream-ohlcv-streams() {
  echo "📡 Aktif stream'ler: http://$STREAM_OHLCV_ADDR/api/streams"
  curl -s "http://$STREAM_OHLCV_ADDR/api/streams" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor."
}

# ============================================================
#  BREAKOUT STRATEJİSİ  (breakout-strategy)
#  detect-ms + paper-service kullanır. VELVETUSDT 1m, 100 pencere,
#  her 20 pencerede bir analiz.
# ============================================================
breakout-start() {
  _start_guard
  if pgrep -x breakout-strategy &>/dev/null; then
    echo "⚠️  VELVETUSDT stratejisi zaten çalışıyor (pid: $(pgrep -x breakout-strategy | head -1))"
    return 1
  fi
  # Bağımlılık kontrolü
  if ! curl -s -o /dev/null -w "%{http_code}" "http://$DETECT_MS_ADDR/api/ms?symbol=VELVETUSDT&interval=1m&limit=5" 2>/dev/null | grep -q 200; then
    echo "⚠️  detect-ms yanıt vermiyor → breakout-start ile başlatın"
    return 1
  fi
  echo "🎯 VELVETUSDT stratejisi başlatılıyor (VELVETUSDT 1m, 100 pencere, 20 pencere/kontrol)..."
  _tmux_pane "🎯BREAKOUT" "cd $CYCLE_ROOT && $CYCLE_ROOT/target/debug/breakout-strategy" Enter
  sleep 2
  if pgrep -x breakout-strategy &>/dev/null; then
    echo "✅ VELVETUSDT stratejisi başladı [pid: $(pgrep -x breakout-strategy | head -1)]"
    echo "   Pencere: cycle → 🎯BREAKOUT"
  else
    echo "❌ VELVETUSDT stratejisi başlatılamadı."
  fi
}

breakout-stop() {
  _start_guard
  local pid
  pid=$(pgrep -x breakout-strategy 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    pkill -TERM -x breakout-strategy 2>/dev/null
    sleep 1
    pkill -KILL -x breakout-strategy 2>/dev/null || true
    echo "✅ VELVETUSDT stratejisi durduruldu [pid:$pid]"
  else
    echo "⚠️  VELVETUSDT stratejisi zaten çalışmıyor"
  fi
}

breakout-status() {
  local pid
  pid=$(pgrep -x breakout-strategy 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ VELVETUSDT stratejisi ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
  else
    echo "✘  VELVETUSDT stratejisi durdurulmuş"
  fi
}

breakout-log() {
  tail -f /tmp/breakout.log
}

# Bekleme süresini saniye cinsinden ayarla (çalışan strateji bir sonraki döngüde uygular)
# Kullanım: breakout-wait 600   (10 dakika)  |  breakout-wait 1200  (20 dakika)
breakout-wait() {
  _start_guard
  local sec="${1:-}"
  if [ -z "$sec" ]; then
    local cur; cur=$(cat /tmp/breakout_wait_sec.txt 2>/dev/null || echo "1200")
    echo "ℹ️  Mevcut bekleme: $cur sn"
    echo "Kullanım: breakout-wait <saniye>   (örn. breakout-wait 600 → 10dk)"
    return 0
  fi
  if ! echo "$sec" | grep -qE '^[0-9]+$' || [ "$sec" -lt 10 ]; then
    echo "❌ Saniye değeri geçerli değil (min 10): $sec"
    return 1
  fi
  echo "$sec" > /tmp/breakout_wait_sec.txt
  echo "✅ Bekleme süresi ayarlandı: $sec sn ($((sec/60)) dk)"
  echo "   Çalışan strateji bir sonraki döngüde bu değeri kullanır."
  if pgrep -x breakout-strategy >/dev/null 2>&1; then
    echo "   ℹ️  Strateji çalışıyor — yeni süre otomatik uygulanacak."
  fi
}

breakout-query() {
  # Kullanım: breakout-query [--dry-run]
  if [ "${1:-}" = "--dry-run" ]; then
    cd "$CYCLE_ROOT" && $CYCLE_ROOT/target/debug/breakout-strategy --once --dry-run
  else
    cd "$CYCLE_ROOT" && $CYCLE_ROOT/target/debug/breakout-strategy --once
  fi
}

# ============================================================
#  EXECUTION ENGINE  (Canlı Binance Futures — executiond :3010)
#  Anahtarları SHELL panelinden güvenli girmek için:
#    exec-setup     → anahtar gir (ekrana yazılmaz, .env 600)
#    exec-show      → mevcut yapılandırma (anahtarlar maskeli)
# ============================================================
exec-setup() {
  _start_guard
  "$CYCLE_ROOT/additional-services/scripts/exec_setup.sh"
}
exec-show() {
  _start_guard
  "$CYCLE_ROOT/additional-services/scripts/exec_setup.sh" --show
}
exec-testnet() {
  _start_guard
  "$CYCLE_ROOT/additional-services/scripts/exec_setup.sh" --testnet
  echo "✅ Testnet ayarlandı. exec-dry ile başlatın."
}
exec-dry() {
  _start_guard
  if pgrep -x executiond &>/dev/null; then echo "⚠️  executiond zaten çalışıyor → exec-stop"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p execution-engine --bins 2>&1 | tail -1
  _tmux_pane "🛡️EXEC" "cd $CYCLE_ROOT && EXEC_MODE=LIVE EXEC_DRY_RUN=true ./target/debug/executiond" Enter
  sleep 2
  exec-status
}
exec-live() {
  _start_guard
  if pgrep -x executiond &>/dev/null; then echo "⚠️  executiond zaten çalışıyor → exec-stop"; return 1; fi
  echo ""
  echo -e "${_R}⚠️  GERÇEK EMİR MODU (EXEC_DRY_RUN=false)${_N}"
  echo -e "${_R}    Emirler gerçek Binance hesabına gidecek.${_N}"
  echo "    Devam etmek için 'GO' yazın:"
  local onay
  read -r onay
  if [ "$onay" != "GO" ]; then echo "İptal."; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p execution-engine --bins 2>&1 | tail -1
  _tmux_pane "🛡️EXEC" "cd $CYCLE_ROOT && EXEC_DRY_RUN=false ./target/debug/executiond" Enter
  sleep 2
  exec-status
}
exec-stop() {
  _start_guard
  local p; p=$(pgrep -x executiond 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    kill -TERM "$p" 2>/dev/null; sleep 1
    kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null
    echo "✅ executiond durduruldu [pid:$p]"
  else
    echo "ℹ️  executiond çalışmıyor"
  fi
}
exec-status() {
  _start_guard
  local pid; pid=$(pgrep -x executiond 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    echo "✅ executiond ÇALIŞIYOR [pid:$pid]"
    curl -s -m 2 http://127.0.0.1:3010/api/v1/mode  | python3 -m json.tool 2>/dev/null
    curl -s -m 2 http://127.0.0.1:3010/api/v1/risk   | python3 -m json.tool 2>/dev/null
  else
    echo "✘ executiond durdurulmuş"
  fi
}
exec-account()   { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" account; }
exec-positions() { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" positions "${1:-}"; }
exec-balance()   { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" balance; }
exec-orders()    { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" orders "${1:-}"; }
exec-order()     { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" order "$@"; }
exec-cancel-all(){ _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" cancel-all "${1:-}"; }
exec-kill()      { _start_guard; touch "${EXEC_KILL_SWITCH_PATH:-/tmp/exec_kill_switch}"; echo "⚠️  Kill switch AÇIK — yeni emirler reddedilir"; }
exec-unkill()    { _start_guard; rm -f "${EXEC_KILL_SWITCH_PATH:-/tmp/exec_kill_switch}"; echo "Kill switch kapatıldı."; }

# ── Yüklendiğini bildir ──────────────────────────────────────
echo -e "${_D}[cycle_env] Yüklendi — ROOT: $CYCLE_ROOT | API: $CYCLE_API${_N}"
```

### `additional-services/scripts/cycle_tmux.sh`

```bash
#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — tmux tek-sekme başlatıcı
#  Kullanım: ./scripts/cycle_tmux.sh [attach|kill|status]
#
#  Her servis tek sekmede (window) çalışır:
#  Pencere 0 — 🧠 STRATEGY
#  Pencere 1 — 🛰️  LISTENER
#  Pencere 2 — ⚠️  RISK
#  Pencere 3 — 💻 SHELL
#  Pencere 4 — 📡 DATA
#  Pencere 5 — 🔔 ALERT
#  Pencere 6 — 🛡️ PAPER
#  Pencere 7 — Monitor  (CPU/RAM/GPU izleme)
#  Pencere 8 — DETECT-MS (MSMP :3002)
#  Pencere 9 — BREAKOUT (Kırılım stratejisi)
#  Pencere 10 — STREAM-OHLCV (canlı OHLCV mum akışı :3008)
#  Pencere 11 — CALC-IND (indikatör hesaplama motoru :3007)
#  Pencere 12 — 🤖 AI (LLM agent katmanı, ai.toml + OpenAI/Anthropic)
#  Pencere 13 — 🖥️ CONSOLE (executiond elle komut konsolu)
# ============================================================
set -euo pipefail

SESSION="cycle"
# Kurulu pakette CYCLE_ROOT, kaynak ağacında varsayılan olarak betiğin konumundan bulunur.
ROOT="${CYCLE_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"

# ── Binary dizini: varsayılan release; debug için BIN_DIR=./target/debug ver ──
BIN="${BIN_DIR:-$ROOT/target/release}"
BUILD_ARGS=""
case "$BIN" in
  *release*) BUILD_ARGS="--release" ;;
esac

# ── Kurulu paket dizinleri (kaynak ağacına göre varsayılan) ──
CONFIG_DIR="${CYCLE_CONFIG_DIR:-$ROOT}"
SCRIPTS_DIR="${CYCLE_SCRIPTS_DIR:-$ROOT/additional-services/scripts}"

# ── Env varsayılanları ───────────────────────────────────────
PAPER_API_ADDR="${PAPER_API_ADDR:-127.0.0.1:8080}"
PAPER_ADMIN_USER="${PAPER_ADMIN_USER:-admin}"
PAPER_ADMIN_PASS="${PAPER_ADMIN_PASS:-changeme123}"
PAPER_INITIAL_USDT="${PAPER_INITIAL_USDT:-100000}"
ALERT_CONFIG="${ALERT_CONFIG:-$CONFIG_DIR/alerts.toml}"

# ── Tam temizlik fonksiyonu ──────────────────────────────────
full_cleanup() {
  echo "🧹 Temizleniyor..."
  tmux kill-session -t "$SESSION" 2>/dev/null && echo "  ✔ tmux session kapatıldı" || echo "  - tmux session yoktu"
  for proc in core paper-service alert-service; do
    if pgrep -x "$proc" &>/dev/null; then
      pkill -TERM -x "$proc" 2>/dev/null || true
      sleep 0.5
      pkill -KILL -x "$proc" 2>/dev/null || true
      echo "  ✔ $proc durduruldu"
    fi
  done
  for f in /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders; do
    [ -f "$f" ] && rm -f "$f" && echo "  ✔ $f silindi" || true
  done
  echo "✅ Temizlik tamamlandı."
}

# ── Alt komutlar ─────────────────────────────────────────────
case "${1:-}" in
  kill)
    full_cleanup
    exit 0
    ;;
  status)
    echo "=== tmux Pencereleri ==="
    tmux list-windows -t "$SESSION" -F "  #{window_index}: #{window_name}" 2>/dev/null \
      || echo "  ⚠️  '$SESSION' session'ı çalışmıyor."
    echo ""
    echo "=== Çalışan Servisler ==="
for proc in core paper-service alert-service; do
      pid=$(pgrep -x "$proc" 2>/dev/null | head -1 || true)
      if [ -n "$pid" ]; then
        mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
        cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
        echo "  ✔ $proc  [pid:$pid]  CPU:${cpu}%  RAM:${mem}"
      else
        echo "  ✘ $proc  (durdurulmuş)"
      fi
    done
    exit 0
    ;;
  attach)
    tmux attach-session -t "$SESSION" 2>/dev/null || { echo "⚠️  Session yok."; exit 1; }
    exit 0
    ;;
esac

# ── Zaten çalışıyorsa bağlan ─────────────────────────────────
if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "⚡ '$SESSION' zaten çalışıyor. Bağlanılıyor..."
  tmux attach-session -t "$SESSION"
  exit 0
fi

# ── Derleme (yalnızca kaynak ağacında) ────────────────────────
if [ -f "$ROOT/Cargo.toml" ]; then
  echo "🔨 Derleniyor..."
  cd "$ROOT"
  cargo build $BUILD_ARGS -p cycle-splash -p core -p paper-service -p alert-service -p breakout-strategy -p stream-ohlcv -p ai-engine -p exec-console 2>&1 | tail -5
else
  echo "ℹ️  Kurulu paket — önceden derlenmiş binary'ler kullanılıyor ($BIN)"
fi

# ── Eski süreçleri ve ring buffer'ları temizle ───────────────
echo "🧹 Eski süreçler temizleniyor..."
for proc in core paper-service alert-service; do
  if pgrep -x "$proc" &>/dev/null; then
    pkill -TERM -x "$proc" 2>/dev/null || true
    sleep 0.3
    pkill -KILL -x "$proc" 2>/dev/null || true
    echo "  ✔ $proc durduruldu"
  fi
done
rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders
echo "  ✔ Ring buffer'lar temizlendi"
sleep 1

# ── Açılış ekranı (tek terminal) ─────────────────────────────
echo "🎬 Açılış ekranı..."
cd "$ROOT"
"$BIN/cycle-splash" 2>/dev/null || "$ROOT/target/debug/cycle-splash" 2>/dev/null || echo "  (cycle-splash bulunamadı)"

# ── Shell init dosyasını oluştur ────────────────────────────
cat > /tmp/cycle_init.sh << INITEOF
#!/usr/bin/env bash
export CYCLE_ROOT='$ROOT'
export CYCLE_API='http://$PAPER_API_ADDR'
export CYCLE_USER='$PAPER_ADMIN_USER'
export CYCLE_PASS='$PAPER_ADMIN_PASS'
source '$SCRIPTS_DIR/cycle_env.sh'
help-cycle
INITEOF
chmod +x /tmp/cycle_init.sh

# ── Session oluştur ──────────────────────────────────────────
tmux new-session -d -s "$SESSION" -x 220 -y 50
tmux rename-window -t "$SESSION:0" "🧠 STRATEGY"

# ── Pencere 0: STRATEGY ─────────────────────────────────────
tmux send-keys -t "$SESSION:0" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧠  STRATEGY TERMİNALİ  (PyO3)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $ROOT && RUN_MODE=STRATEGY $BIN/core
" Enter

# ── Pencere 1: LISTENER ─────────────────────────────────────
tmux new-window -t "$SESSION:1" -n "🛰️  LISTENER"
tmux send-keys -t "$SESSION:1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛰️   LISTENER  (Anlık Metrik Analizi)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/listener
" Enter

# ── Pencere 2: RISK ─────────────────────────────────────────
tmux new-window -t "$SESSION:2" -n "⚠️  RISK"
tmux send-keys -t "$SESSION:2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '⚠️   RİSK ANALİZİ  (TimescaleDB)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/risk_analysis --watch
" Enter

# ── Pencere 3: SHELL ────────────────────────────────────────
tmux new-window -t "$SESSION:3" -n "💻 SHELL"
tmux send-keys -t "$SESSION:3" "source /tmp/cycle_init.sh" Enter

# ── Pencere 4: DATA ─────────────────────────────────────────
tmux new-window -t "$SESSION:4" -n "📡 DATA"
tmux send-keys -t "$SESSION:4" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  DATA TERMİNALİ  (Binance WS)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
cd $ROOT && RUN_MODE=DATA $BIN/core
" Enter

# ── Pencere 5: ALERT ────────────────────────────────────────
tmux new-window -t "$SESSION:5" -n "🔔 ALERT"
tmux send-keys -t "$SESSION:5" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔔  ALERT SERVİSİ  (Sesli Uyarı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/alert-service --config $ALERT_CONFIG
" Enter

# ── Pencere 6: PAPER ────────────────────────────────────────
tmux new-window -t "$SESSION:6" -n "🛡️ PAPER"
tmux send-keys -t "$SESSION:6" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛡️   PAPER SERVICE  (REST API :8080)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && \
  PAPER_ADMIN_USER=$PAPER_ADMIN_USER \
  PAPER_ADMIN_PASS=$PAPER_ADMIN_PASS \
  PAPER_API_ADDR=$PAPER_API_ADDR \
  PAPER_INITIAL_USDT=$PAPER_INITIAL_USDT \
  PAPER_SLED_PATH=./data-engine/data/paper_wal \
  PAPER_DB_PATH=./data-engine/data/paper_live.db \
  $BIN/paper-service
" Enter

# ── Pencere 7: MONITOR ──────────────────────────────────────
tmux new-window -t "$SESSION:7" -n "Monitor"
tmux send-keys -t "$SESSION:7" "bash '$SCRIPTS_DIR/monitor.sh'" Enter

# ── Pencere 8: DETECT-MS ────────────────────────────────────
tmux new-window -t "$SESSION:8" -n "DETECT-MS"
tmux send-keys -t "$SESSION:8" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📈  DETECT-MS  (MSMP 2.0 :3002)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-ms
" Enter

# ── Pencere 9: BREAKOUT STRATEJİ ────────────────────────────
tmux new-window -t "$SESSION:9" -n "BREAKOUT"
tmux send-keys -t "$SESSION:9" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🎯  BREAKOUT  (Kırılım Stratejisi)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/breakout-strategy
" Enter

# ── Pencere 10: STREAM-OHLCV ────────────────────────────────
tmux new-window -t "$SESSION:10" -n "STREAM-OHLCV"
tmux send-keys -t "$SESSION:10" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  STREAM-OHLCV  (Canlı OHLCV :3008)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/stream-ohlcv
" Enter

# ── Pencere 11: CALC-IND ────────────────────────────────────
tmux new-window -t "$SESSION:11" -n "CALC-IND"
tmux send-keys -t "$SESSION:11" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧮  CALC-IND  (İndikatör Motoru :3007)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/calc-ind
" Enter

# ── Pencere 12: AI ENGINE ───────────────────────────────────
tmux new-window -t "$SESSION:12" -n "🤖 AI"
tmux send-keys -t "$SESSION:12" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🤖  AI ENGINE  (LLM Agent Katmanı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $CONFIG_DIR && $BIN/ai-engine
" Enter

# ── Pencere 13: EXEC CONSOLE ────────────────────────────────
tmux new-window -t "$SESSION:13" -n "🖥️ CONSOLE"
tmux send-keys -t "$SESSION:13" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🖥️  EXEC CONSOLE  (executiond :3010)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $ROOT && $BIN/exec-console
" Enter

# ── Görsel ayarlar (global) ──────────────────────────────────
tmux set-option -t "$SESSION" mouse on
tmux set-option -t "$SESSION" status-interval 1

# ── Pano yapıştırma: Ctrl+V / Ctrl+Shift+V → OS panosunu yapıştır ──
tmux bind -n C-v run-shell "$SCRIPTS_DIR/tmux_clipboard_paste.sh" 2>/dev/null || true
tmux bind -n C-S-v run-shell "$SCRIPTS_DIR/tmux_clipboard_paste.sh" 2>/dev/null || true
tmux set-option -g set-clipboard on 2>/dev/null || true

# Status bar — Matrix yeşili / siyah
tmux set-option -t "$SESSION" status-style          "bg=#000000,fg=#00ff41"
tmux set-option -t "$SESSION" status-left           "#[bg=#003300,fg=#00ff41,bold]  🏛️  Cycle Finance  #[bg=#000000,fg=#00ff41] "
tmux set-option -t "$SESSION" status-left-length    30
tmux set-option -t "$SESSION" status-right          "#[fg=#00ff41]0#[fg=#00cc33]:STRAT #[fg=#00ff41]1#[fg=#00cc33]:LISTEN #[fg=#00ff41]2#[fg=#00cc33]:RISK #[fg=#00ff41]4#[fg=#00cc33]:DATA #[fg=#00ff41]5#[fg=#00cc33]:ALERT #[fg=#00ff41]6#[fg=#00cc33]:PAPER #[fg=#00ff41]7#[fg=#00cc33]:Mon #[fg=#00ff41]10#[fg=#00cc33]:STREAM #[fg=#00ff41]11#[fg=#00cc33]:CALC #[fg=#00ff41]12#[fg=#00cc33]:AI #[fg=#00ff41]13#[fg=#00cc33]:CONSOLE #[fg=#00ff41]%H:%M:%S"
tmux set-option -t "$SESSION" status-right-length   80

# Window sekme renkleri — matrix
tmux set-option -t "$SESSION" window-status-format          "#[fg=#008a2e] #{window_index}:#{window_name} "
tmux set-option -t "$SESSION" window-status-current-format  "#[bg=#003300,fg=#00ff41,bold] #{window_index}:#{window_name} "

# ── Terminal penceresine dön ve bağlan ───────────────────────
tmux select-window -t "$SESSION:0"
tmux attach-session -t "$SESSION"
```

### `additional-services/scripts/exec_setup.sh`

```bash
#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Binance Futures API anahtarlarını güvenli şekilde gir (shell üzerinden).
#
# - Anahtar girilirken ekrana YAZILMAZ (read -s).
# - .env dosyası 600 (yalnızca sahip) iznine alınır.
# - Varsayılan EXEC_DRY_RUN=true (güvenlik): gerçek emir için ayrıca
#   `EXEC_DRY_RUN=false` onayı gerekir.
#
# Kullanım:
#   ./additional-services/scripts/exec_setup.sh          # anahtar gir
#   ./additional-services/scripts/exec_setup.sh --show   # hangi değişkenler set?
#   ./additional-services/scripts/exec_setup.sh --testnet # testnet URL'leri yaz
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

ENV_FILE="${EXEC_ENV_FILE:-$(cd "$(dirname "$0")/../.." && pwd)/.env}"
TESTNET_BASE="https://testnet.binancefuture.com"
TESTNET_WS="wss://stream.binancefuture.com"

# Önceki yapıştırmadan artık satır başı kalıntılarını terminal tamponundan atar.
# CRLF pano yapıştırınca ilk read '\r'de biter, kalan '\n' bir sonraki read'i
# boş yutardı — bu yüzden her okumadan önce tamponu temizle.
drain_input() {
    local ch
    while read -r -t 0 -s -n 1 ch 2>/dev/null; do :; done
}

trim() {
    local v="$1"
    v="${v#"${v%%[![:space:]]*}"}"   # öndeki boşlukları kırp
    v="${v%"${v##*[![:space:]]}"}"   # arkadaki boşlukları kırp
    printf '%s' "$v"
}

prompt_key() {
    local label="$1"
    local value
    # İpuçları stderr'e gider; sadece girilen değer stdout'a döner ($( ) yakalamasın).
    echo "  İpucu: yapıştırmak için Ctrl+Shift+V (Linux) veya Ctrl+V (tmux bağı)." >&2
    echo "  Girdiğiniz karakterler EKRANA YAZILMAZ (güvenlik) — yapıştırın, Enter." >&2
    while :; do
        drain_input
        read -rsp "$label: " value
        echo >&2
        # Bracketed-paste artıkları + satır sonlarını temizle (pano yapıştırınca)
        value="${value//$'\e[200~'/}"
        value="${value//$'\e[201~'/}"
        value="${value//$'\r'/}"
        value="${value//$'\n'/}"
        value="$(trim "$value")"
        if [[ -n "$value" ]]; then
            break
        fi
        echo "  Boş olamaz, tekrar dene." >&2
    done
    printf '%s' "$value"
}

write_env() {
    local key="$1" val="$2"
    local tmp="${ENV_FILE}.tmp"
    if grep -qE "^${key}=" "$ENV_FILE" 2>/dev/null; then
        # awk ile sadece ilgili satırı değiştir; dosyadaki \r kalıntılarını da temizle.
        awk -v k="$key" -v v="$val" '
            { gsub(/\r$/, ""); }
            $0 ~ "^"k"=" { print k "=" v; found=1; next }
            { print }
            END{ if (!found) print k "=" v }
        ' "$ENV_FILE" > "$tmp" && mv "$tmp" "$ENV_FILE"
    else
        # Satır sonu kalıntılarını temizle, sonra ekle.
        sed -i 's/\r$//' "$ENV_FILE" 2>/dev/null || true
        printf '%s=%s\n' "$key" "$val" >> "$ENV_FILE"
    fi
}

case "${1:-}" in
    --show)
        echo "ENV_FILE: $ENV_FILE"
        for v in BINANCE_API_KEY BINANCE_SECRET_KEY EXEC_MODE EXEC_DRY_RUN EXEC_BASE_URL EXEC_WS_URL; do
            if grep -qE "^${v}=" "$ENV_FILE" 2>/dev/null; then
                val=$(grep -E "^${v}=" "$ENV_FILE" | head -1 | cut -d= -f2-)
                if [[ "$v" == *KEY* ]]; then
                    masked="${val:0:6}****${val: -4}"
                    [[ -z "$val" ]] && masked="(boş)"
                    echo "  $v = $masked"
                else
                    echo "  $v = ${val:-}"
                fi
            else
                echo "  $v = (tanımsız)"
            fi
        done
        ;;
    --testnet)
        write_env EXEC_BASE_URL "$TESTNET_BASE"
        write_env EXEC_WS_URL "$TESTNET_WS"
        write_env EXEC_DRY_RUN "true"
        echo "Testnet yapılandırması yazıldı:"
        echo "  EXEC_BASE_URL=$TESTNET_BASE"
        echo "  EXEC_WS_URL=$TESTNET_WS"
        ;;
    *)
        if [[ ! -f "$ENV_FILE" ]]; then
            echo "  .env yok — oluşturuluyor: $ENV_FILE"
            : > "$ENV_FILE"
        fi

        echo "Binance Futures API anahtarları — ekrana yazılmaz."
        api_key=$(prompt_key "BINANCE_API_KEY")
        secret=$(prompt_key "BINANCE_SECRET_KEY")

        if [[ -z "$api_key" || -z "$secret" ]]; then
            echo "  ✘ Anahtar(lar) boş — hiçbir şey kaydedilmedi. Tekrar dene." >&2
            exit 1
        fi

        # Maskeli doğrulama — yanlış yapıştırma burada yakalanır.
        mask() { local v="$1"; [[ -z "$v" ]] && echo "(boş)" || echo "${v:0:4}****${v: -4}"; }
        echo
        echo "  Girilen anahtarlar:"
        echo "    BINANCE_API_KEY    = $(mask "$api_key")  (uzunluk: ${#api_key})"
        echo "    BINANCE_SECRET_KEY = $(mask "$secret")  (uzunluk: ${#secret})"
        drain_input
        read -rsp "  Kaydetmek için 'EVET' yazın, iptal için Enter: " onay
        echo
        if [[ "$onay" != "EVET" ]]; then
            echo "İptal edildi — hiçbir şey yazılmadı."
            exit 0
        fi

        write_env "BINANCE_API_KEY" "$api_key"
        write_env "BINANCE_SECRET_KEY" "$secret"
        write_env "EXEC_MODE" "LIVE"
        # Güvenlik: DRY_RUN varsayılan açık. Gerçek emir için kullanıcı ayrıca
        # `EXEC_DRY_RUN=false` ayarlamalıdır.
        write_env "EXEC_DRY_RUN" "true"

        chmod 600 "$ENV_FILE"
        echo
        echo "Anahtarlar kaydedildi: $ENV_FILE (izin 600)"
        echo "  EXEC_DRY_RUN=true  → emirler gönderilmez."
        echo "  Gerçek emir için:  EXEC_DRY_RUN=false ./target/debug/executiond"
        echo "  Testnet için:      $0 --testnet"
        ;;
esac
```

### `additional-services/scripts/gdpr_erasure_test.sh`

```bash
#!/bin/bash

# GDPR/KVKK Right to Erasure Simulation Script for Cycle Finance 2.0
# Simulates physically wiping a user's data from ClickHouse.

set -e

USER_ID="client_9942"
SALT="sUp3rS3cr3tS4lt"

echo "=========================================="
echo " Starting GDPR Erasure Protocol Simulation"
echo "=========================================="

# 1. Masking the User ID to create the hash
USER_HASH=$(echo -n "${USER_ID}${SALT}" | sha3sum -a 256 | awk '{print $1}' || echo "mocked_sha3_hash_8a2b3c")
echo "[+] Target User Hash: $USER_HASH"

# 2. Simulate ClickHouse Mutation
echo "[+] Triggering ALTER TABLE ticks DELETE WHERE symbol_hash = '$USER_HASH'..."
sleep 1
echo "[+] ClickHouse mutation submitted."

# 3. Simulate verifying the physical erasure
echo "[+] Verifying erasure via Merkle Tree..."
sleep 1
echo "[+] Data physically scrubbed from disks and EC-12/4 replicas."

# 4. Log to Deletion Registry
echo "[+] Appending event to deletion_registry for 3-year compliance hold..."
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) | $USER_HASH | GDPR Erasure Processed" >> deletion_registry_mock.log

echo "=========================================="
echo " Erasure Protocol Complete."
echo "=========================================="
```

### `additional-services/scripts/monitor.sh`

```bash
#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — Servis İzleme Paneli
#  Her saniye güncellenir. Ctrl+C ile çıkılır.
#
#  İzlenen servisler:
#    core (DATA / STRATEGY / BACKTEST / CORRELATION)
#    paper-service
#    alert-service
# ============================================================

# ── Renkler ──────────────────────────────────────────────────
R='\033[0;31m'    # kırmızı
G='\033[0;32m'    # yeşil
Y='\033[1;33m'    # sarı
C='\033[0;36m'    # camgöbeği
B='\033[1;34m'    # mavi
M='\033[0;35m'    # mor
W='\033[1;37m'    # beyaz kalın
DIM='\033[2m'     # soluk
N='\033[0m'       # reset
BG='\033[40m'     # siyah arka plan

# ── GPU sysfs yolu (AMD RX 5500) ─────────────────────────────
GPU_CARD=""
for card in /sys/class/drm/card*/device/gpu_busy_percent; do
    if [ -r "$card" ]; then
        GPU_CARD="$(dirname "$card")"
        break
    fi
done

# ── Bar çizici ───────────────────────────────────────────────
# Kullanım: draw_bar <yüzde(0-100)> <genişlik> <renk>
draw_bar() {
    local pct="${1:-0}"
    local width="${2:-20}"
    local color="${3:-$G}"
    # Yüzde'yi tam sayıya dönüştür
    pct=$(echo "$pct" | awk '{printf "%d", $1}')
    [ "$pct" -gt 100 ] 2>/dev/null && pct=100
    [ "$pct" -lt 0 ]   2>/dev/null && pct=0
    local filled=$(( pct * width / 100 ))
    local empty=$(( width - filled ))
    # Yüksek kullanımda renk değiştir
    if [ "$pct" -ge 80 ]; then color="$R"
    elif [ "$pct" -ge 50 ]; then color="$Y"
    fi
    local bar=""
    for ((i=0; i<filled; i++)); do bar+="█"; done
    for ((i=0; i<empty;  i++)); do bar+="░"; done
    echo -ne "${color}${bar}${N}"
}

# ── RAM okunabilir format ─────────────────────────────────────
human_mb() {
    local kb="${1:-0}"
    local mb=$(( kb / 1024 ))
    if [ "$mb" -ge 1024 ]; then
        echo "$(awk "BEGIN{printf \"%.1f\", $mb/1024}")G"
    else
        echo "${mb}M"
    fi
}

# ── Servis satırı çiz ─────────────────────────────────────────
# Kullanım: print_service <etiket> <pid> <renk>
print_service() {
    local label="$1"
    local pid="$2"
    local color="${3:-$C}"
    local bar_width=18

    if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
        printf "${color}%-22s${N} ${DIM}%-8s${N} ${R}%-6s${N}   %-${bar_width}s   ${R}%-8s${N}   %-${bar_width}s\n" \
            "$label" "-" "KAPALI" "$(draw_bar 0 $bar_width $R)" "-" "$(draw_bar 0 $bar_width $R)"
        return
    fi

    # CPU ve bellek bilgisi
    local stat
    stat=$(ps -p "$pid" -o pid,pcpu,rss,vsz --no-headers 2>/dev/null | head -1)
    [ -z "$stat" ] && return

    local cpu  rss vsz
    cpu=$(echo "$stat" | awk '{printf "%.1f", $2}')
    rss=$(echo "$stat" | awk '{print $3}')   # KB
    vsz=$(echo "$stat" | awk '{print $4}')   # KB (sanal)

    local rss_str vsz_str cpu_int
    rss_str=$(human_mb "$rss")
    vsz_str=$(human_mb "$vsz")
    cpu_int=$(echo "$cpu" | awk '{printf "%d", $1}')

    # Çok çekirdekli sistemlerde CPU > 100 olabilir, bar için sıkıştır
    local cpu_bar_pct=$(( cpu_int > 100 ? 100 : cpu_int ))

    printf "${color}%-22s${N} ${W}%-8s${N} ${Y}%5s%%${N}  %s  ${C}%-8s${N}  %s\n" \
        "$label" "[$pid]" "$cpu" \
        "$(draw_bar "$cpu_bar_pct" "$bar_width")" \
        "$rss_str" \
        "$(draw_bar "$(( rss / 1024 > 100 ? 100 : rss / 1024 ))" "$bar_width")"
}

# ── Sistem toplamı ───────────────────────────────────────────
system_summary() {
    # CPU toplam kullanımı
    local cpu_idle cpu_use
    cpu_idle=$(top -bn1 | grep "Cpu(s)" | awk '{print $8}' | tr -d '%' | tr ',' '.')
    [ -z "$cpu_idle" ] && cpu_idle=$(vmstat 1 1 | tail -1 | awk '{print $15}')
    cpu_use=$(awk "BEGIN{printf \"%.1f\", 100 - ${cpu_idle:-0}}")

    # RAM
    local mem_total mem_avail mem_used mem_pct
    mem_total=$(awk '/MemTotal/{print $2}' /proc/meminfo)
    mem_avail=$(awk '/MemAvailable/{print $2}' /proc/meminfo)
    mem_used=$(( mem_total - mem_avail ))
    mem_pct=$(awk "BEGIN{printf \"%d\", $mem_used * 100 / $mem_total}")

    # GPU (AMD sysfs)
    local gpu_use="N/A" gpu_vram_pct=0 gpu_vram_str="N/A"
    if [ -n "$GPU_CARD" ]; then
        gpu_use=$(cat "${GPU_CARD}/gpu_busy_percent" 2>/dev/null || echo "0")
        local vram_used vram_total
        vram_used=$(cat "${GPU_CARD}/mem_info_vram_used"  2>/dev/null || echo "0")
        vram_total=$(cat "${GPU_CARD}/mem_info_vram_total" 2>/dev/null || echo "1")
        gpu_vram_pct=$(awk "BEGIN{printf \"%d\", $vram_used * 100 / $vram_total}")
        local vram_used_mb=$(( vram_used / 1024 / 1024 ))
        local vram_total_mb=$(( vram_total / 1024 / 1024 ))
        gpu_vram_str="${vram_used_mb}M / ${vram_total_mb}M"
    fi

    local cpu_int=${cpu_use%.*}
    printf "${DIM}Sistem Geneli:${N}\n"
    printf "  ${W}CPU  ${N}%5s%%  %s\n" "$cpu_use"  "$(draw_bar "$cpu_int" 30)"
    printf "  ${W}RAM  ${N}%5s%%  %s  ${DIM}(%s / %s)${N}\n" \
        "$mem_pct" "$(draw_bar "$mem_pct" 30)" \
        "$(human_mb "$mem_used")" "$(human_mb "$mem_total")"
    if [ -n "$GPU_CARD" ]; then
        printf "  ${W}GPU  ${N}%5s%%  %s\n" "$gpu_use" "$(draw_bar "$gpu_use" 30)"
        printf "  ${W}VRAM ${N}%5s%%  %s  ${DIM}(%s)${N}\n" \
            "$gpu_vram_pct" "$(draw_bar "$gpu_vram_pct" 30)" "$gpu_vram_str"
    else
        printf "  ${W}GPU  ${N}${DIM}  AMD sysfs okunamadı${N}\n"
    fi
}

# ── PID bul ─────────────────────────────────────────────────
find_pid() {
    local name="$1"
    pgrep -x "$name" 2>/dev/null | head -1
}

find_pid_env() {
    # RUN_MODE=X olan core process'ini bul
    local mode="$1"
    pgrep -x "core" 2>/dev/null | while read -r pid; do
        if grep -qa "RUN_MODE=$mode" /proc/"$pid"/environ 2>/dev/null; then
            echo "$pid"
            return
        fi
    done
}

# ── Ana döngü ────────────────────────────────────────────────
INTERVAL="${MONITOR_INTERVAL:-1}"

# Cursor'ı gizle, çıkışta geri getir
tput civis
trap 'tput cnorm; echo' EXIT INT TERM

# İlk açılışta bir kez temizle
clear

while true; do
    # Ekranı silmeden cursor'ı sol-üst köşeye taşı (titreme yok)
    tput cup 0 0

    local_time=$(date '+%H:%M:%S')
    local_date=$(date '+%d.%m.%Y')

    echo -e "${W}╔══════════════════════════════════════════════════════════════════════════════════╗${N}"
    printf "${W}║${N}  ${M}📊 CYCLE FINANCE — SERVİS İZLEME PANELİ${N}%$((39 - ${#local_time}))s${Y}%s${N}  ${W}║${N}\n" "" "$local_time  $local_date"
    echo -e "${W}╚══════════════════════════════════════════════════════════════════════════════════╝${N}"
    echo ""

    # ── Sistem özeti ─────────────────────────────────────────
    system_summary
    echo ""

    # ── Servis başlıkları ─────────────────────────────────────
    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"
    printf "${DIM}%-22s  %-8s  %-7s  %-18s  %-8s  %-18s${N}\n" \
        "SERVİS" "PID" "CPU%" "CPU KULLANIMI" "RAM" "RAM KULLANIMI"
    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"

    # ── Servisler ────────────────────────────────────────────
    PID_DATA=$(find_pid_env "DATA")
    PID_STRATEGY=$(find_pid_env "STRATEGY")
    PID_BACKTEST=$(find_pid_env "BACKTEST")
    PID_CORRELATION=$(find_pid_env "CORRELATION")
    PID_PAPER=$(find_pid "paper-service")
    PID_ALERT=$(find_pid "alert-service")

    # core binary tek isimle görünüyorsa genel bul
    [ -z "$PID_DATA" ] && [ -z "$PID_STRATEGY" ] && [ -z "$PID_BACKTEST" ] && [ -z "$PID_CORRELATION" ] && {
        ALL_CORE=$(pgrep -x "core" 2>/dev/null | head -1)
    }

    print_service "📡 DATA"          "${PID_DATA:-$ALL_CORE}" "$C"
    print_service "🧠 STRATEGY"      "$PID_STRATEGY"          "$B"
    print_service "🔄 BACKTEST"      "$PID_BACKTEST"          "$M"
    print_service "📈 CORRELATION"   "$PID_CORRELATION"       "$Y"
    echo -e "${DIM}──────────────────────────────────────────────────────────────────────────────────${N}"
    print_service "🛡️  PAPER-SERVICE" "$PID_PAPER"             "$G"
    print_service "🔔 ALERT-SERVICE" "$PID_ALERT"             "$Y"

    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"

    # ── Çalışan servis sayısı ─────────────────────────────────
    running=0
    for p in "$PID_DATA" "$PID_STRATEGY" "$PID_BACKTEST" "$PID_CORRELATION" "$PID_PAPER" "$PID_ALERT"; do
        [ -n "$p" ] && kill -0 "$p" 2>/dev/null && (( running++ )) || true
    done

    echo ""
    printf "  ${DIM}Çalışan servis: ${W}%d/6${N}${DIM}   |   Yenileme: her %ss   |   Çıkış: Ctrl+C${N}\n" \
        "$running" "$INTERVAL"

    # ── Ring buffer bilgisi ───────────────────────────────────
    echo ""
    echo -e "  ${DIM}Ring Buffer Durumu:${N}"
    for ring in cycle_finance_ring cycle_finance_orders; do
        if [ -f "/dev/shm/$ring" ]; then
            ring_size=$(du -sh "/dev/shm/$ring" 2>/dev/null | cut -f1)
            printf "    ${G}✔${N} /dev/shm/%-28s %s\n" "$ring" "$ring_size"
        else
            printf "    ${R}✘${N} /dev/shm/%-28s ${DIM}(yok)${N}\n" "$ring"
        fi
    done

    sleep "$INTERVAL"
done
```

### `additional-services/scripts/start_paper.sh`

```bash
#!/usr/bin/env bash
# PAPER sistemi tek komutla başlatma.
#   DATA terminal (Binance Futures → tick ring) + paper-service (API + actor)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# ── Binary dizini: varsayılan release; debug için BIN_DIR=./target/debug ver ──
BIN="${BIN_DIR:-$ROOT/target/release}"
BUILD_ARGS=""
case "$BIN" in
  *release*) BUILD_ARGS="--release" ;;
esac

API_ADDR="${PAPER_API_ADDR:-127.0.0.1:8080}"
ADMIN_USER="${PAPER_ADMIN_USER:-admin}"
ADMIN_PASS="${PAPER_ADMIN_PASS:-changeme123}"
INITIAL_USDT="${PAPER_INITIAL_USDT:-10000}"

echo "=== Derleniyor... ==="
cargo build $BUILD_ARGS -p core -p paper-service

echo "=== Eski süreçler kapatılıyor (varsa) ==="
pkill -x core 2>/dev/null || true
pkill -x paper-service 2>/dev/null || true
pkill -x paper_cli 2>/dev/null || true
sleep 1

# Tick ring'i temizle (farklı kapasiteyle başlatılırsa)
rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders

echo "=== DATA terminali başlatılıyor (Binance Futures WS) ==="
setsid env RUN_MODE=DATA "$BIN/core" > /tmp/data_terminal.log 2>&1 < /dev/null &
disown

echo "=== paper-service başlatılıyor (REST API + Actor) ==="
rm -rf data-engine/data/paper_wal
setsid env \
  PAPER_ADMIN_USER="$ADMIN_USER" \
  PAPER_ADMIN_PASS="$ADMIN_PASS" \
  PAPER_API_ADDR="$API_ADDR" \
  PAPER_INITIAL_USDT="$INITIAL_USDT" \
  PAPER_DB_PATH=./data-engine/data/paper_live.db \
  PAPER_SLED_PATH=./data-engine/data/paper_wal \
  "$BIN/paper-service" > /tmp/paper_service.log 2>&1 < /dev/null &
disown

echo "=== Süreçler başlatılıyor... ==="
sleep 4

echo ""
echo "✅ PAPER SİSTEMİ ÇALIŞIYOR"
echo "=============================================="
echo "REST API      : http://$API_ADDR/api/v1/system/health"
echo "Metrikler     : http://$API_ADDR/metrics"
echo "Giriş         : user=$ADMIN_USER pass=$ADMIN_PASS"
echo ""
echo "Kontrol (fiyat geliyor mu):"
echo "  curl -s http://$API_ADDR/api/v1/system/health"
echo ""
echo "CLI örnekleri:"
echo "  $BIN/paper_cli --api http://$API_ADDR --user $ADMIN_USER --password $ADMIN_PASS status"
echo "  $BIN/paper_cli --api http://$API_ADDR --user $ADMIN_USER --password $ADMIN_PASS order --symbol BTCUSDT --side BUY --order-type MARKET --qty 0.001"
echo ""
echo "Loglar: /tmp/data_terminal.log , /tmp/paper_service.log"
echo "Kapatmak için: ./scripts/stop_paper.sh"
```

### `additional-services/scripts/stop_paper.sh`

```bash
#!/usr/bin/env bash
# PAPER sistemini kapatır (DATA + paper-service).
set -euo pipefail

echo "=== PAPER sistemi kapatılıyor ==="
pkill -x paper-service 2>/dev/null && echo "  paper-service durduruldu" || echo "  paper-service zaten kapalı"
pkill -x core 2>/dev/null && echo "  DATA terminal durduruldu" || echo "  DATA terminal zaten kapalı"

# Paylaşımlı hafıza temizliği
rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders

echo "Done."
```

### `additional-services/scripts/tmux_clipboard_paste.sh`

```bash
#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# OS panosunu tmux paste buffer'a yükleyip yapıştırır.
# Wayland (wl-paste) → X11 (xclip/xsel) sırasıyla dener.
#
# tmux'ta Ctrl+V (veya Ctrl+Shift+V) bu betiği çalıştırır:
#   bind -n C-v   run-shell "~/.cycle_tmux_paste.sh"
# ─────────────────────────────────────────────────────────────────────────────
set -u

tmp="$(mktemp /tmp/tmux_paste.XXXXXX)"

if command -v wl-paste >/dev/null 2>&1 && [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
    wl-paste -p 2>/dev/null > "$tmp"
elif command -v xclip >/dev/null 2>&1; then
    xclip -o -selection clipboard 2>/dev/null > "$tmp"
elif command -v xsel >/dev/null 2>&1; then
    xsel -b -o 2>/dev/null > "$tmp"
fi

if [[ -s "$tmp" ]]; then
    # Panodaki sondaki satır sonlarını (CR/LF) temizle: paste sırasında fazladan
    # Enter/\r yutulup bir sonraki "read"i boş tetiklemesin.
    perl -0777 -pi -e 's/[\r\n]+\z//' "$tmp" 2>/dev/null || sed -i 's/\r$//' "$tmp"
    tmux load-buffer "$tmp" 2>/dev/null
    tmux paste-buffer 2>/dev/null
fi

rm -f "$tmp"
```
