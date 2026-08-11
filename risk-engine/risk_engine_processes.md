# 🔒 RISK-Engine Fonksiyonel Süreçler

## Giriş

`risk-worker` binary (risk parametre üretici daemon), 60 saniyede bir döngü ile çalışır. CLI argümanı yok; yapılandırma tamamen env var + dosya (risk.toml). RiskControl (kill switch) dosyaya yazılır, `risk.toml` içindeki parametreler.

**Başlatma:**
- `cargo run -p risk-engine` (kök workspace)
- `RISK_WORKER_PORT=3011 cargo run -p risk-engine`
- `./target/debug/risk-worker`

---

## Süreç 1: Risk Engine (Pre-trade Zincir)

`src/engine.rs:92-244` — 13 adımlı pre-trade kural zinciri:

| # | Adım | Konum | Ret Nedeni |
|:---|:---|:---|:---|
| 1 | **Kill switch** açık mı? | engine.rs:96 | KillSwitch |
| 2 | **Circuit breaker**: ardışık red ≥ eşik? | engine.rs:110 | CircuitBreaker |
| 3 | **Kalıcı status** (halts_trading)? | engine.rs:117 | DailyLossExceeded / DrawdownExceeded / LiquidationProximity / LeverageExceeded |
| 4 | **Blocklist** | engine.rs:138 | BlockedSymbol |
| 5 | **Rate limit** (60s pencere) | engine.rs:143 | RateLimit |
| 6 | **Fiyat kaynağı**: limit emrinde emir fiyat, market emrinde mark; bayatlık > stale_mark_ms (200ms) → fail-closed | engine.rs:148-157 | StaleMark |
| 7 | **Notional limit** | engine.rs:170 | NotionalExceeded |
| 8 | **Kaldıraç limiti** (intent veya politika) | engine.rs:175 | LeverageExceeded |
| 9 | **Pozisyon limiti** (projeksiyon): (mevcut + emir) * fiyat > limit | engine.rs:181-200 | PositionLimitExceeded |
| 10 | **Brüt exposure** (projeksiyon) | engine.rs:205 | ExposureLimitExceeded |
| 11 | **Konsantrasyon HHI** | engine.rs:214 | ConcentrationExceeded |
| 12 | **Marj**: notional/leverage > cash - open_orders_notional | engine.rs:223 | InsufficientMargin |
| 13 | **Parametrik risk kapısı** (opsiyonel): cache'te available && gate_ready yoksa red | engine.rs:230 | ParametricRiskUnavailable |

**Onay:** rate_limit.record() + breaker.record_approval() + audit.record_approved → RiskDecision::Approved (engine.rs:240-243).

---

## Süreç 2: Cold Path (Risk Worker Daemon)

`src/worker.rs:118-205` — 60 saniyede bir:

1. **compute_params()** → log getirileri → korelasyon → Tikhonov → EWMA vol → parametrik VaR → HHI → önerilen limitler
2. **apply_suggestions()** → öneriler yalnızca mevcut limiti **daraltırsa** uygulanır
3. **RiskCache'ea (seqlock) yazar** → gate_ready=true

---

## Süreç 3: Kill Switch Tetikleme Mantığı

### Açık kabul
`flag || dosya var` (kill_switch.rs:28-30). İki bağımsız işaretin OR'u.

### Otomatik tetikleyiciler
1. **Status ihlali** (evaluate_status, state.rs:146-179):
   - drawdown > max_drawdown_pct → MaxDrawdownBreached (state.rs:160)
   - daily_loss <= -max_daily_loss_usdt → MaxDailyLossBreached (state.rs:162)
   - herhangi bir pozisyonda liquidation_breached → Liquidation (state.rs:169)
   - durum halts_trading() döndürürse → kill_switch.engage() (state.rs:176-178)
2. **Ardışık red eşiği**: her reject() breaker.record_rejection() çağırır; >= consecutive_rejection_auto_stop (varsayılan 3) ise kill_switch.engage() (engine.rs:308-313). Onayda sayaç sıfırlanır (engine.rs:241).

### Manuel tetikleyiciler
- **touch /tmp/exec_kill_switch** (dosya var → açık) — kill_switch.rs:6
- **REST PUT /api/risk/kill-switch {enabled:true/false}** — risk-worker.rs:190

### Açma (release)
Yalnızca manuel — flag=false + dosya sil (kill_switch.rs:40-44). Sıcak yolda hiçbir kod kill switch'i otomatik açmaz.

---

## Süreç 4: VaR Akışı (Cold → Hot)

Worker her çevrimde compute_params() çalıştırır:

1. **log getirileri** → log getirileri → history
2. **korelasyon** → korelasyon matrisi (correlation.rs)
3. **Tikhonov** → regularize_correlation_matrix
4. **EWMA volatilite** → ewma_volatility (λ=0.94)
5. **parametrik VaR** → parametric_var_99_1d
6. **HHI** → HHI konsantrasyon
7. **önerilen limitler** → apply_worker_params

---

## Süreç 5: Audit (JsonLinesAudit)

`src/audit.rs:69-86` — flume kanalı:
- unbounded kanal + yazıcı thread (JSONL append)
- record() → try_send (asla bloklamaz)
- disabled() → kayıtlar düşer (audit.rs:93-97)

---

## Süreç 6: RiskCache (Seqlock)

`src/cache.rs:46-88` — seqlock tabanlı parametre önbelleği:
- Worker 60s'de yazar → write() → seq++
- Hot path okur → read() → s1 == s2 kontrolü
- S1 == s2 → güvenli kopya
- S1 != s2 → yeni seq'ye geç → tekrar okuma

---

## Süreç 7: Risk State (state.rs)

`src/state.rs:30` — `parking_lot::RwLock<RiskStateInner>`:
- Portföy, mark fiyatları, status, bekleyen emir rezervi
- Fill/mark yazma kısa, pre-trade okuma tek lock
- Snapshot (state.rs:182)

---

## Satır Sayıları

| Dosya | Satır |
|:---|:---|
| src/engine.rs | 348 |
| src/accounting.rs | 314 |
| src/bin/risk-worker.rs | 306 |
| src/state.rs | 260 |
| src/types.rs | 251 |
| src/correlation.rs | 232 |
| src/worker.rs | 224 |
| src/liquidity.rs | 159 |
| src/audit.rs | 157 |
| src/config.rs | 148 |
| src/cache.rs | 132 |
| src/policy.rs | 124 |
| src/limits.rs | 87 |
| src/exposure.rs | 85 |
| src/var.rs | 83 |
| src/kill_switch.rs | 49 |
| src/lib.rs | 42 |

**Test (`tests/`, 3 dosya): 521 satır** (engine_tests 289, accounting_tests 149, matrix_tests 83)

**Cargo.toml:** 29 satır

**Toplam: 3.551 satır** (examples/ boş)

---

## Sonuç

risk-engine, pre-trade kural zinciri (13 adımlı evaluate), cold path risk worker daemonı (60s döngü), kill switch (otomatik + manuel), limit sistemi (rate limit + circuit breaker), ve audit (JSONL flume). Fail-closed, sadece risk varsa red; risk engine'den değil risk worker'dan geçer.