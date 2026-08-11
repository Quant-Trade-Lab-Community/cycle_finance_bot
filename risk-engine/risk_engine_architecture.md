# 🔒 RISK-Engine Mimari Dokümanı

## Genel Bakış

**risk-engine**, Cycle Finance sisteminin **ortak risk çekirdeği** (single source of truth). Tüm risk kuralları burada yaşar.

- **Hot path (pre-trade):** execution-engine emir göndermeden önce `RiskEngine::evaluate()` çağırır → emir onaylanır veya reddedilir.
- **Cold path (risk parametre üretimi):** risk-worker daemon'ı 60 saniyede bir korelasyon/VaR hesaplar, önerilen limitleri üretir; hot path bu parametreleri okur.

Temel ilkeler:
- **Fail-closed:** Durum bilinmiyorsa emir reddedilir (bayat mark, model yok → red).
- **Para Decimal, asla f64:** PnL/limit/pozisyon/marj rust_decimal; f64 yalnızca istatistiksel modellerde.
- **Hot path allocation-free, sıralı kural zinciri:** evaluate tek metod, maliyet sırasına göre fail-fast çalışır.
- **Her karar denetlenebilir:** AuditLog onay/red/fill kaydeder.
- **Kill switch otomatik + manuel:** Günlük kayıp/drawdown aşımı, likidasyon veya 3+ ardışık red → otomatik kapan; sadece manuel açılır.

---

## Katmanlar ve Modül Sorumlulukları

| Modül | Sorumluluk | Önemli semboller |
|:---|:---|:---|
| **var.rs** | Value-at-Risk hesabı. Parametrik (varyans-kovaryans), tarihsel, tek sembol VaR; HHI ağırlık üst sınırı; safe_notional (kayıp bütçesi / VaR) | parametric_var_99_1d, z_score, historical_var, safe_notional |
| **correlation.rs** | Pearson korelasyon matrisi, Ledoit–Wolf shrink, Tikhonov ridge, koşul sayısı, Jacobi özdeğer çözücü (N≤64), EWMA volatilite (λ=0.94) | correlation_matrix, shrink, tikhonov, regularize_correlation_matrix, ewma_volatility, jacobi_eigenvalues |
| **worker.rs** | Cold path döngüsü: fiyat geçmişi tutar, log getirileri çıkarır, korelasyon → Tikhonov → EWMA vol → parametrik VaR → HHI → önerilen limitler üretir ve RiskCache'e yazar | RiskWorker::run_cycle, compute_params, PriceHistory, apply_suggestions |
| **cache.rs** | Seqlock tabanlı parametre önbelleği. Worker 60s'de yazar, hot path lock-free okur (torn-read koruması) | Seqlock<T>, RiskCache, RiskParameters |
| **engine.rs** | Pre-trade kural zinciri (hot path). 13 adımlı evaluate, ret nedenleri, breaker arttırma, kill switch arm, audit | RiskEngine::evaluate, reject, on_fill, apply_worker_params |
| **limits.rs** | Emir akışı limitleri: kayan pencere (60s) rate limit + circuit breaker (ardışık red sayacı) | RateLimit, CircuitBreaker |
| **kill_switch.rs** | Acil durdurma: AtomicBool bayrağı + dosya (/tmp/exec_kill_switch). İkisinden biri varsa açık. Otomatik arm, sadece manuel release | is_open, engage, release |
| **accounting.rs** | Portföy muhasebesi: işaretli pozisyonlar, ONE_WAY netleştirme, ağırlıklı ortalama giriş, gerçekleşen/gerçekleşmemiş PnL, likidasyon fiyatı, drawdown, günlük kayıp | Portfolio::apply_fill, apply_signed, liquidation_price, daily_loss |
| **audit.rs** | Denetim izi: JSONL dosyasına flume kanalı + arka plan iş parçacığıyla yazar. Hot path asla diske yazım bekletmez | RiskDecisionEvent, JsonLinesAudit::open, AuditLog |
| **state.rs** | Paylaşılan risk state: parking_lot::RwLock altında portföy, mark fiyatları, status, bekleyen emir rezervi. Fill/mark işleme, status değerlendirme, otomatik kill switch arm, snapshot | RiskState::process_fill, evaluate_status, snapshot |
| **policy.rs** | Konfigüre edilebilir limit seti + sembol bazlı override. risk.toml'dan deserialize edilir | RiskPolicy, effective, PerSymbolLimits |
| **config.rs** | risk.toml yükleme (yoksa varsayılan) + mtime tabanlı hot-reload izleyici | load_risk_config_from, ConfigWatcher, ReloadablePolicy |
| **types.rs** | Ortak tipler: OrderIntent, RiskDecision, RejectReason (16 kural), RiskStatus, Fill, MarkPrice | RejectReason::rule_name, RiskStatus::halts_trading |
| **exposure.rs** | Brüt/net exposure, HHI konsantrasyon, pre-trade için projeksiyon | exposure, projected_gross_exposure |
| **liquidity.rs** | LOB simülörü: sabit nokta (×100k fiyat, ×1k miktar) integer aritmetik, piyasa emir ortalama dolum, slippage bps | LobSimulator::simulate_buy, estimate_slippage_bps |
| **bin/risk-worker.rs** | Daemon binary: 60s döngü + axum REST (/healthz, /api/risk/snapshot, /api/risk/kill-switch), POSIX shm ring'e yayın | main, publish, read_marks |

**Not:** liquidity.rs ve LiquidityLimitExceeded/enable_liquidity_gate **şu anda RiskEngine::evaluate zincirine bağlı değil**. LOB simülörü ve slippage fonksiyonu hazır ama kural zincirinde kullanılmıyor (engine.rs'de 13 adımda likidite kontrolü yok; yalnızca types.rs:115, policy.rs:58, liquidity.rs:121 ve config/risk.toml:25'te tanımlı). Bu, risk_engine_processes.md'de "bekleyen/henüz bağlanmamış kapı" olarak not edilmeli.

---

## Veri Akışı

### Pre-trade (Hot Path)
```
execution-engine emir → RiskEngine::evaluate()
  └── 13 adımlı evaluate (engine.rs:92-244)
  ├─ 1. Kill switch açık mı?
  ├─ 2. Circuit breaker: ardışık red ≥ eşik?
  ├─ 3. Kalıcı status (halts_trading)?
  ├─ 4. Blocklist
  ├─ 5. Rate limit (60s pencere)
  ├─ 6. Fiyat kaynağı: limit emrinde emir fiyat, market emrinde mark; bayatlık > stale_mark_ms (200ms) → fail-closed
  ├─ 7. Notional limit
  ├─ 8. Kaldıraç limiti (intent veya politika)
  ├─ 9. Pozisyon limiti (projeksiyon): (mevcut + emir) * fiyat > limit
  ├─ 10. Brüt exposure (projeksiyon)
  ├─ 11. Konsantrasyon HHI
  ├─ 12. Marj: notional/leverage > cash - open_orders_notional
  ├─ 13. Parametrik risk kapısı (opsiyonel): cache'te available && gate_ready yoksa red
  └─ Onay: rate_limit.record() + breaker.record_approval() + audit.record_approved → RiskDecision::Approved
```

### Cold Path (Risk Worker Daemon)
```
60s döngü → worker.run_cycle() → compute_params()
  ├─ log getirileri → korelasyon → Tikhonov → EWMA vol → parametrik VaR → HHI → önerilen limitler
  └─ RiskCache'ea (seqlock) yazar → gate_ready=true
```

---

## Giriş Noktaları

### Binary

| Binary | Dosya | Giriş Noktasi |
|:---|:---|:---|
| `risk-worker` | `src/bin/risk-worker.rs` | risk-worker.rs:211-305 — risk parametre üretici daemon. **CLI argümanı yok**; yapılandırma tamamen env var + dosya (risk.toml). |

### REST API (risk-worker)

| Uç | Açıklama |
|:---|:---|
| GET /healthz | çevrim sayısı + son parametreler + politika yolu |
| GET /api/risk/snapshot | model parametreleri + politika + kill switch durumu |
| PUT /api/risk/kill-switch {enabled} | manuel arm/release |

### CLI Argümanları

| Argüman | Varsayılan | Konum |
|:---|:---|:---|
| RISK_WORKER_PORT | 3011 | risk-worker.rs:212 |
| RISK_WORKER_INTERVAL_SEC | 60 | risk-worker.rs:216 |
| RISK_WORKER_MAX_SAMPLES | 120 | risk-worker.rs:220 |
| RISK_KILL_SWITCH_PATH | /tmp/exec_kill_switch | risk-worker.rs:231 |
| RISK_CONFIG | ./risk.toml | src/config.rs:27 |

### Girdi Kaynakları
- /tmp/price_feed.json (price-feed çıktısı, {"prices": {SYM: {"mark":..., "last":...}}})
- risk.toml yolu (varsayılan ./risk.toml)
- /tmp/risk_params.json (risk worker parametreleri)

### Çıktılar
- POSIX shm ring /cycle_finance_risk_params (transport::ring_buffer::GenerationalRingBuffer)
- /tmp/risk_params.json

---

## Thread / Task Yapısı

### risk-worker Daemon (risk-worker.rs)

```
risk-worker main thread
├── std::thread::spawn → döngü iş parçacığı başlatır (risk-worker.rs:252)
│   └── her cycle_sec'te: hot-reload kontrolü → read_marks() → worker.run_cycle() → publish() → tracing::info!
├── tokio::runtime::Runtime::new() + block_on
│   └── axum REST sunucusu (tokio task)
│       └── REST handler'ları read().await ile okur
│
├── RiskState: parking_lot::RwLock<RiskStateInner> (state.rs:30)
│   → Fill/mark yazma kısa, pre-trade okuma tek lock
├── RiskCache: seqlock (spin + seq doğrulama) (cache.rs:46-88)
│   → yazar seq'i tek yapıp yazar, çift yapar; okuyucu s1==s2 ise güvenli kopya
├── RiskEngine: parking_lot::Mutex<RateLimit>, Mutex<CircuitBreaker> (engine.rs:25-26)
│   → kısa kilitli sayaçlar
├── flume kanalı (audit) → JsonLinesAudit::open (audit.rs:69-86)
│   → unbounded kanal + yazıcı thread (JSONL append)
├── POSIX shm ring (cross-process): /cycle_finance_risk_params, 1024 slot, 702B veri
│   → worker parametreleri diğer proseslere yayınlar
│   → Sığmıyorsa 700B'den büyük JSON ring'e yazılır, dosyaya yazılır (risk-worker.rs:81-85)
```

### RiskCache (seqlock)

Seqlock<T> (cache.rs:46-88):
- Worker 60s'de yazar → write() → seq++
- Hot path okur → read() → s1 == s2 kontrolü
- S1 == s2 → güvenli kopya
- S1 != s2 → yeni seq'ye geç → tekrar okuma

---

## Kritik Algoritmalar

### Parametrik VaR (var.rs:26-41)
- sigma_p^2 = Σ_i Σ_j w_i w_j σ_i σ_j ρ_ij (w'Σw)
- VaR = z(0.99) * sqrt(sigma_p^2)
- Matris boyutları tutarsız → None (fail-closed)
- z-skoru tablosu: %95→1.6449, %97→1.8808, %98→2.0537, %99→2.3263, %99.5→2.5758 (var.rs:8-18)

### Korelasyon Matrisi (correlation.rs:11-49)
- Pearson: getiri ortalamaları → kovaryans (s/(t-1)) → cov[i][j] / (σ_i σ_j)
- Sıfır varyanslı sembol: kendisiyle 1, başkalarıyla 0 (sağlam davranış)
- Girdi formatı: satır=sembol, sütun=zaman

### Regularizasyon Zinciri (correlation.rs:54-113)
- **Shrink (Ledoit–Wolf):** (1-s)C + sI, s∈[0,1]
- **Tikhonov (ridge):** C + αI
- **Koşul sayısı:** |λmax/λmin|; λmin≈0 → None (tekil)
- **Jacobi özdeğer çözücü:** N≤64 simetrik matrisler, en büyük off-diagonal elemanı hedef alan döndürmeler; max_off < 1e-12 veya 100*n*n iterasyonda durur
- **EWMA volatilite:** λ=0.94, sondan başa var = λ·var + (1-λ)r²

### HHI Konsantrasyon (exposure.rs:38-47)
- share_i = notional_i / gross
- HHI = Σ share_i²
- Kapalı durum: max_hhi == 0 → concentration_breached

### Pozisyon Limiti Projekeksiyonu (engine.rs:181-200)
- projected = |existing_quantity + emir_signed_quantity| * fiyat
- projected > max_position_usdt → red
- Brüt exposure projeksiyonu diğer sembolleri dahil eder (exposure.rs:59-78)

### Likidasyon Fiyatı (accounting.rs:49-56)
- long: entry * (1 - 1/lev + maintenance_margin_rate)
- short: entry * (1 + 1/lev - maintenance_margin_rate)
- liquidation_breached: long'da mark ≤ liq, short'ta mark ≥ liq

### Muhasebe (accounting.rs:163-227)
- ONE_WAY netleştirme: realized = (fill - entry) * close_qty (long) / tersi (short)
- Yön dönüşü: kapatılan kısım realize, kalan yeni giriş fiyatıyla short'a döner
- Aynı yön: ağırlıklı ortalama total_cost/total_qty
- Roll daya: UTC gün sayacı (ts/86400s) → realized_today sıfırlanır

### Rate Limit + Circuit Breaker (limits.rs:52-86)
- Kayan pencere: VecDeque<Instant>, 60s'den eski kayıtlar prune ile düşer
- Breaker: record_rejection() eşiği aşarsa true (kill switch arm'ı tetikler)

### LOB Slippage (liquidity.rs:11-13)
- Sabit nokta: fiyat×100k, miktar×1k, u64/u128 taşmasız
- simulate_buy/sell: ilk 10 seviye kademe kademeli dolum, ortalama fiyat
- estimate_slippage_bps: |avg/mid - 1| * 10_000, yalnızca tam dolum durumunda

### VaR Tabanlı Limit Önerisi (worker.rs:177-191)
- suggested_max_position = daily_loss_budget / var_pct (var::safe_notional)
- suggested_leverage = (budget / var / 1000).clamp(1.0, 3.0)
- apply_suggestions/apply_worker_params: öneriler yalnızca mevcut limiti daraltırsa uygulanır

### Kill Switch Tetikleme Mantığı

**Açık kabul:** flag || dosya var (kill_switch.rs:28-30). İki bağımsız işaretin OR'u.

**Otomatik tetikleyiciler:**
1. **Status ihlali** (evaluate_status, state.rs:146-179)
   - drawdown > max_drawdown_pct → MaxDrawdownBreached (state.rs:160)
   - daily_loss <= -max_daily_loss_usdt → MaxDailyLossBreached (state.rs:162)
   - herhangi bir pozisyonda liquidation_breached → Liquidation (state.rs:169)
   - Durum halts_trading() döndürürse → kill_switch.engage() (state.rs:176-178)
2. **Ardışık red eşiği:** her reject() breaker.record_rejection() çağırır; >= consecutive_rejection_auto_stop (varsayılan 3) ise kill_switch.engage() (engine.rs:308-313). Onayda sayaç sıfırlanır (engine.rs:241).

**Manuel:**
- touch /tmp/exec_kill_switch (dosya var → açık) — kill_switch.rs:6
- REST PUT /api/risk/kill-switch {enabled=true/false} — risk-worker.rs:190

**Açma (release):** yalnızca manuel — flag=false + dosya sil (kill_switch.rs:40-44). Sıcak yolda hiçbir kod kill switch'i otomatik açmaz.

---

## Dış Bağımlılıklar (Cargo.toml)

risk-engine/Cargo.toml:7-21 — hepsi workspace tanımlı (Cargo.toml:31-80):

| Bağımlılık | Kaynak | Kullanım |
|:---|:---|:---|
| rust_decimal | workspace (1.34) | tüm para/limit/pozisyon matematiği |
| parking_lot | workspace (0.12) | RwLock (state), Mutex (rate/breaker) |
| serde / serde_json | workspace (1.0) | RiskPolicy deserialize, audit JSONL, REST JSON |
| toml | workspace (0.8) | risk.toml parse |
| chrono | workspace (0.4) | tanımlı ama kullanılmıyor |
| flume | workspace (0.11) | audit arka plan kanalı |
| transport | path: ../cycle-engine/transport | POSIX shm GenerationalRingBuffer (kendi deps: libc, memmap2) |
| tokio | workspace (1.0 full) | worker REST runtime |
| axum | workspace (0.8) | REST API |
| tracing | workspace (0.1) | çevrim logları |
| proptest (dev) | workspace | tanımlı ama testlerde kullanılmıyor |