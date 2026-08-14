# 🔧 RISK-Engine Geliştirme Önerileri

Risk-engine kaynak kodu, mimari dokümanlar, testler ve execution-engine entegrasyonu incelendi.
Derleme + 32 test geçiyor (engine_tests 13, accounting_tests 12, matrix_tests 7). Öneriler öncelik sırasına göre gruplandırılmıştır.

---

## 1. Kritik (doğruluk)

### 1.1 Likidite kapısı hâlâ bağlı değil
`enable_liquidity_gate`, `max_slippage_bps`, `LobSimulator`, `LiquidityLimitExceeded` hazır ama
`engine.rs` 13 adımlı `evaluate` zincirinde kullanılmıyor.

- Mevcut: `types.rs:115` (RejectReason), `policy.rs:58` (limitler), `liquidity.rs:121` (`estimate_slippage_bps`), `risk.toml:25` (config)
- Eksik: `estimate_slippage_bps` → `LiquidityLimitExceeded` adımı olarak 14. kural eklenmeli.
- Bu, `risk_engine_architecture.md`'de "bekleyen/henüz bağlanmamış kapı" olarak da not edilmiştir.

### 1.2 Politika çifti drift (ciddi)
`RiskEngine` kendi `policy: RwLock<RiskPolicy>`'sini (engine.rs:21) ve `RiskState` kendi kopyasını
(state.rs:31, `with_parts` içinde `policy.clone()`) tutuyor.

- `set_policy` (engine.rs:83) ve `apply_worker_params` (engine.rs:294) yalnızca engine'in kopyasını günceller.
- `evaluate_status` (state.rs:146) kendi kopyasını okur → **bayat** `max_drawdown_pct` / `max_daily_loss_usdt` ile çalışır.
- Execution tarafı `set_policy`'yi sık kullanıyor (checks.rs:124-152).
- Çözüm: `Arc<RwLock<RiskPolicy>>` tek kopya olarak paylaşılmalı.

### 1.3 `reduce_only` / `close_position` hiç kontrol edilmiyor
- Alanlar tanımlı ve set ediliyor (types.rs:53-54, checks.rs:173-174) ama `evaluate`'de hiç okunmuyor.
- Reduce-only emir bile pozisyon projeksiyonuna tabi; "artış emri" olarak yanlış yönde kullanım engellenmiyor.
- Execution tarafında `preflight.rs:127` kendi kontrolünü yapıyor ama risk-engine (tek doğruluk kaynağı) doğrulamıyor.

### 1.4 Parametrik kapı tazeliği yok
- `gate_ready`/`available` bir kez `true` olunca worker ölse bile kalıcı kalır.
- `computed_at_ms` mevcut (cache.rs:28) ama `evaluate` kontrolünde (engine.rs:230-235) kullanılmıyor.
- Öneri: `max_param_age_ms` eşiği ekleyip aşımda fail-closed davranmak.

### 1.5 HHI konsantrasyon projekte edilmiyor
- Step 11 (engine.rs:214-220) mevcut portföy HHI'sını kontrol eder; emir sonrası (`signed_delta` ile) projekte HHI değil.
- Pozisyon (9) ve exposure (10) projekte ediliyor, konsantrasyon (11) edilmiyor — tutarsız.

### 1.6 Rate limit TOCTOU + hot-reload eksikliği
- `check()` + `record()` (engine.rs:143, 240) ayrı çağrılar; eşzamanlı çağrılar limiti aşabilir. Tek mutex altında atomik yapılmalı.
- `set_max_per_min` (limits.rs:25) hiçbir yerde çağrılmıyor → `set_policy` sonrası rate limit güncellenmiyor.

---

## 2. Worker / Model

### 2.1 Eşit ağırlık varsayımı
- worker.rs:148 `vec![1.0 / n]` — gerçek portföy exposure ağırlıkları yerine eşit ağırlık kullanılıyor.
- VaR ve önerilen limitler gerçekçi değil. Brüt exposure payları parametre olarak geçilmeli.

### 2.2 Öneriler uygulanmıyor
- `apply_suggestions` (worker.rs:211) / `apply_worker_params` (engine.rs:294) tanımlı ama risk-worker binary'sinde hiç çağrılmıyor.
- Worker hesaplayıp yayınlıyor ama hot path daraltmıyor.
- Yalnız daraltma yapılıyor; vol düşünce gevşetme ve worker durunca restore yok.

---

## 3. Güvenlik / Operasyon

### 3.1 REST kill-switch auth'suz
- PUT `/api/risk/kill-switch` (risk-worker.rs:195) kimlik doğrulamasız.
- Repo'da başka servislerde JWT deseni mevcut. En azından loopback dışına kapalı olduğu garanti edilmeli + basit token.

### 3.2 `is_open()` her evaluate'de `stat` syscall
- kill_switch.rs:29 — hot path'te her emir için dosya kontrolü (disk erişimi).
- mtime/inotify veya kısa TTL cache önerilir.

### 3.3 Bozuk `risk.toml` sessizce yutuluyor
- config.rs:74-77 `load_risk_config_from(...).ok()` — parse hatası hiç loglanmıyor.

---

## 4. Test Boşlukları

### 4.1 Liquidity.rs hiç test edilmemiş
- LOB simülasyonu ve slippage hesabı için test yok (test dizininde yalnızca engine/accounting/matrix testleri var).

### 4.2 proptest tanımlı ama kullanılmıyor
- Property testler eklenebilir: accounting (negatif qty, sıfır fiyat), rate-limit penceresi, seqlock eşzamanlılık, kill-switch dosya+flag.

### 4.3 REST endpoint testi yok
- healthz / snapshot / kill-switch için axum testleri eklenebilir.

### 4.4 Dead risk durumları
- `RiskStatus::MaxLeverageBreached` / `ParametricRiskUnavailable` hiçbir yerde üretilmiyor (state.rs yalnızca drawdown/daily/liquidation set eder).

### 4.5 `examples/` boş
- Embed API örneği (RiskEngine + worker + cache akışı) eklenebilir.

---

## 5. Küçük Notlar

- `notional()` `None` döndüğünde yanlış `StaleMark` reason üretiliyor (engine.rs:161-164 — aslında fiyat eksik).
- `chrono` ve `proptest` bağımlılıkları kullanılmıyor (Cargo.toml:14,22).

---

## Öncelik Önerisi

Uygulanacaksa öncelik sırası (doğruluk açısından en değerli olanlar):

1. **1.2 Politika drifti** — tek `Arc<RwLock<RiskPolicy>>` paylaşımı
2. **1.1 Likidite kapısı** — 14. kural olarak bağlama
3. **1.3 reduce_only / close_position** — zincire doğrulama ekleme
4. **1.4 Parametrik kapı tazeliği** — `max_param_age_ms`
5. **1.5 HHI projeksiyonu** — emir sonrası konsantrasyon
6. **1.6 Rate limit atomik + hot-reload** — tek mutex + `set_max_per_min`
