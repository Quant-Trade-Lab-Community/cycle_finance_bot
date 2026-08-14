# Risk-Engine Mimarisi — Cycle Finance

## 1. Amaç ve Kapsam

Risk-Engine, tüm emir öncesi (pre-trade) ve anlık (mark-to-market) risk kurallarının
**tek doğruluk kaynağıdır** (single source of truth). Kurallar üç ayrı yerde
dağınıktı (`execution-engine/src/risk/checks.rs`, `paper/risk.rs`, eski `engine.rs`);
bu plan hepsini tek çekirdekte toplar.

- **Hot path** (in-process): `RiskEngine::evaluate` pre-trade kural zinciri —
  emrin geçtiği süreçte (execution-engine) çalışır, IPC gecikmesi sıfır.
- **Cold path** (bağımsız daemon): `risk-worker` 60s'de korelasyon → Tikhonov →
  EWMA vol → parametrik VaR → konsantrasyon → önerilen limitleri üretir ve
  `RiskCache` + `/cycle_finance_risk_params` ring'e yazar.

## 2. Tasarım İlkeleri

1. **Tek doğruluk kaynağı.** Risk kuralları yalnızca `risk-engine` crate'inde yaşar.
2. **Fail-closed.** Durum bilinmiyorsa emir reddedilir: mark stale (>200ms) →
   market emri red; parametrik model yoksa ve kapı açıksa → red.
3. **Para `Decimal`'dir, asla `f64`.** PnL/limit/pozisyon/marj `rust_decimal`.
   `f64` yalnızca istatistiksel modellerde (korelasyon, VaR).
4. **Her karar denetlenebilir.** `AuditLog` onay/red nedenleriyle JSONL'e yazar
   (arka plan iş parçacığı, sıcak yolu bloklamaz).
5. **Kill switch otomatik + manuel.** Günlük kayıp/drawdown aşımı veya 3+ ardışık
   red → otomatik kapan; yalnızca manuel açılır (`release`).

## 3. Katman Modeli

```
┌─────────────────────────────────────────────────────────────────────────┐
│  HOT PATH (in-process — emrin geçtiği süreç: execution-engine)          │
│                                                                         │
│  OrderIntent ─► RiskEngine::evaluate ─► RiskDecision(Approved/Rejected) │
│      ▲                     │                    │                        │
│      │              RiskState (RwLock)          ▼ batched               │
│      │      positions/mark/cash/peak_equity  AuditLog (JSONL)           │
│      │            ▲ fill ingress ◄─ ORDER_TRADE_UPDATE                  │
│      │            │        ▲ mark price ◄─ AccountSnapshot (resync)     │
│      └────────────┴────────┴────────────────────────────────────────────┘
│              RiskCache (seqlock)  ◄───────── parametre push              │
└─────────────────────────────────────────────────────────────────────────┘
                                  ▲ /cycle_finance_risk_params ring
┌─────────────────────────────────────────────────────────────────────────┐
│  COLD PATH (risk-worker daemon :3011, 60s)                              │
│  /tmp/price_feed.json → getiri serileri → korelasyon → Tikhonov →       │
│  EWMA vol → VaR → HHI → önerilen limitler → RiskCache + ring + REST     │
└─────────────────────────────────────────────────────────────────────────┘
```

**Karar ayrımı:** Risk *kararları* her zaman emrin geçtiği süreçte verilir.
Worker yalnızca *model çıktısı* üretir ve bunları parametre olarak sunar.

## 4. Modül Yapısı (`risk-engine/src/`)

| Modül | Sorumluluk |
|---|---|
| `types.rs` | `Side`, `OrderIntent`, `RiskDecision`, `RejectReason`, `RiskStatus`, `Fill`, `MarkPrice` |
| `policy.rs` | `RiskPolicy` + `PerSymbolLimits` override + `EffectiveLimits` (TOML) |
| `config.rs` | `risk.toml` yükleme + mtime `ConfigWatcher` (hot-reload) |
| `engine.rs` | `RiskEngine::evaluate` — sıralı kural zinciri (fail-fast) |
| `state.rs` | `RiskState` + `RiskStateInner` + serileştirilebilir `RiskSnapshot` |
| `accounting.rs` | `Portfolio`/`Position` — coin-bazlı muhasebe, fill, PnL, drawdown |
| `exposure.rs` | Brüt/net exposure, HHI konsantrasyon, projeksiyon |
| `liquidity.rs` | Sabit-nokta LOB simülasyonu, slippage (bps) tahmini |
| `correlation.rs` | Pearson korelasyon, shrinkage, Tikhonov, koşul sayısı, Jacobi |
| `var.rs` | Parametrik VaR (σₚ²=wᵀΣw) + historical VaR + `safe_notional` |
| `limits.rs` | Kayan-pencere `RateLimit`, `CircuitBreaker` |
| `kill_switch.rs` | Dosya + bayrak; otomatik tetikleyiciler; manuel açılış |
| `cache.rs` | `Seqlock<T>` + `RiskCache` (sıcak yol lock-free okuma) |
| `audit.rs` | `RiskDecisionEvent` → JSONL arka plan yazıcı |
| `worker.rs` | `RiskWorker` soğuk yol çevrimi + `PriceHistory` |
| `bin/risk-worker.rs` | Daemon: döngü + REST (`/healthz`, `/api/risk/snapshot`, kill-switch) |

## 5. Pre-trade Kural Zinciri (`RiskEngine::evaluate`)

Sıralama maliyet sırasına göredir (fail-fast):

1. `KillSwitch` — açıksa red
2. `CircuitBreaker` — ardışık red eşiği
3. `RiskStatus.halts_trading()` — günlük kayıp / drawdown / likidasyon / leverage
4. `Blocklist`
5. `RateLimit` — 60 sn kayan pencere
6. Mark kaynağı: limit emri → emir fiyatı; market emri → mark (stale → fail-closed)
7. `NotionalLimit` — tek emir
8. `LeverageLimit`
9. `PositionLimit` — sembol projeksiyonu
10. `ExposureLimit` — portföy brüt projeksiyon
11. `ConcentrationLimit` — HHI (worker çıktısı / politikadan)
12. `MarginCheck` — notional/leverage ≤ mevcut nakit − açık emir rezervi
13. `ParametricRiskUnavailable` (opsiyonel, `gate_on_parametric_risk=true`)

Ret → `RejectReason` (kural adı + metrik + eşik) → `AuditLog` + ardışık red
sayacı → eşik aşılınca otomatik kill switch.

## 6. Veri Akışları (kapalı döngüler)

1. **Emir:** `execution-engine` `submit_order` → `RiskChecks` bağdaştırıcısı
   (`OrderRequest → OrderIntent`) → `RiskEngine::evaluate` → red/onay.
2. **Fill geri beslemesi:** `ORDER_TRADE_UPDATE` (TRADE) → `RiskChecks::on_fill`
   → `RiskEngine::on_fill` → `Portfolio::apply_fill` → pozisyon/realized/daily loss.
3. **Fiyat:** `AccountSnapshot` (resync/uzlaştırma) → `RiskChecks::sync_from_snapshot`
   → mark fiyatlar + pozisyonlar + nakit → unrealized PnL / drawdown / likidasyon.

**Bilinen sınır (fail-closed):** Canlıda yeni sembole mark bilgisi olmadan
market emri `StaleMark` ile reddedilir. Çözüm: daemon mark fiyatlarını
`RiskChecks::push_mark` ile besler (worker `/tmp/price_feed.json`'dan zaten okur).

## 7. Risk-Worker Çevrimi (cold path)

1. `/tmp/price_feed.json` mark fiyatları → `PriceHistory` (sembol başına ~120 örnek)
2. Log-getiri serileri
3. Pearson korelasyon → shrink → Tikhonov (hedef koşul sayısı)
4. EWMA vol (λ=0.94)
5. Parametrik portföy VaR (%99, 1g) + HHI
6. Öneri: `safe_notional(daily_loss_budget, VaR)` ve kaldıraç
7. `RiskParameters` → `RiskCache` (seqlock) + `/cycle_finance_risk_params` ring
   + `/tmp/risk_params.json`
8. Volatilite yoksa (var≈0) öneri konservatif: günlük bütçe kadar

REST: `GET /healthz`, `GET /api/risk/snapshot`, `PUT /api/risk/kill-switch`.

## 8. Yapılandırma (`risk.toml`)

Tüm limitler tek dosyada; `RISK_CONFIG` env veya `./risk.toml`. Hot-reload
(worker mtime izler). Sembol bazlı override: `[symbol.SEMBOL]`.

## 9. Entegrasyon Noktaları

| Taraf | Değişiklik |
|---|---|
| `execution-engine/Cargo.toml` | `risk-engine` bağımlılığı eklendi |
| `execution-engine/src/risk/checks.rs` | `RiskChecks` → `RiskEngine` bağdaştırıcısı |
| `execution-engine/src/risk/kill_switch.rs` | `risk_engine::kill_switch::KillSwitch` re-export |
| `execution-engine/src/execution/actor.rs` | fill → `on_fill`, resync → `sync_from_snapshot`, market emri mark besleme |
| `execution-engine/src/state/snapshot.rs` | `open_orders_notional()` eklendi |
| `cycle-engine/core/engine/orchestrator.rs` | `Signal → OrderIntent`, yeni `RiskEngine` API'si |
| `cycle-engine/core/cli/paper_cli.rs` | `risk_engine::portfolio` → `risk_engine::accounting` |
| `install.sh`, `cycle_env.sh` | `risk-worker` binary/servis kaydı, `risk.toml` kopyalama |

## 10. Test Stratejisi

- **Unit (accounting):** qty korunumu, avg-entry, kısmi/tam kapanış, yön değişimi,
  komisyon, drawdown, günlük kayıp.
- **Unit (engine):** kural zinciri, fail-closed mark, blocklist, notional, pozisyon
  projeksiyonu, rate limit, otomatik kill switch, per-symbol override.
- **Unit (matris):** korelasyon, tekil matris regularizasyonu, Jacobi iz korunumu,
  VaR monotonluğu, worker `available`/`unavailable`.
- **Integration:** `mock_binance` (execution-engine) — actor üzerinden emir akışı.
- **Önerilen:** TLA+ spec — "günlük kayıp aşımındayken emir onaylanmaz".

## 11. Faz Durumu

| Faz | Durum |
|---|---|
| 0 — Ortak çekirdek (types/policy/state/accounting/limits/kill_switch) | ✅ |
| 1 — Execution entegrasyonu (fill/resync/mark) | ✅ |
| 2 — Tam kural zinciri + audit | ✅ |
| 3 — Cold path (correlation/VaR/worker/ring/REST) | ✅ |
| 4 — Config hot-reload + testler + doküman | ✅ |
