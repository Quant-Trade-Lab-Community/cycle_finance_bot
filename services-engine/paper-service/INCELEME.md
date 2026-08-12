# Paper Service — Geniş Çaplı İnceleme

Kapsam: `execution-engine/src/paper/` (çekirdek domain) + `services-engine/paper-service/` (servis katmanı).
Tarih: 2026-08-12

## Mimari Genel Bakış

Paper service iki katmandan oluşur:

### Katman 1 — Çekirdek domain (`execution-engine/src/paper/`)

| Dosya | Görev |
|---|---|
| `actor.rs` (737 satır) | Event-sourcing + actor model. Tüm emir/marj/likidasyon mantığı tek task'ta sıralı işlenir. |
| `position.rs` | ONE_WAY (netleştirme) + HEDGE (ayrı LONG/SHORT) pozisyon yönetimi. **Boyut birimi USDT notional, kontrat değil.** |
| `risk.rs` | Drawdown / günlük kayıp / kaldıraç / likidasyon kontrolü, `on_mark_tick`. |
| `account.rs` | Free/locked bakiyeler (USDT + BTC). |
| `snapshot.rs` | API okumaları için paylaşılan `RwLock<PaperSnapshot>`. |
| `config.rs` | Ortam değişkenlerinden yapılandırma. |
| `domain_event.rs` | Event sourcing domain event'leri. |

### Katman 2 — Servis (`services-engine/paper-service/`)

| Dosya | Görev |
|---|---|
| `api.rs` | Axum REST :8080, JWT (argon2), idempotency, HTTPS opsiyonu. |
| `bridge.rs` | Flow ring → `MarkPriceUpdate`, order ring → `SubmitOrder` (spin-loop, zero-copy). |
| `events.rs` | Sled WAL event store + replay. |
| `sqlite_projection.rs` | DomainEvent akışından `paper_trades` / `paper_open_orders` (batch flush). |
| `postgres_store.rs` | `--features full` ile PG event store. |
| `metrics.rs` | Prometheus metrikleri (atomic sayaçlar). |
| `idempotency.rs` | `client_order_id -> CachedResponse` önbelleği. |
| `bin/paper_cli.rs` | REST üzerinden CLI (status/positions/history/order). |

## Veri Akışı

```
flow ring ──► MarkPriceUpdate ──► on_mark_tick (likidasyon/funding) + check_limit_orders
order ring ─► SubmitOrder ─────► process_price_only (mark price'a göre PRICE_ONLY dolum)
REST API ───► ActorCommand ────► actor task ──► DomainEvent kanalı ──► Sled + SQLite + PG
API okuma ──► snapshot (her komuttan sonra publish_snapshot)
```

- Tek olay kanalı: actor yalnızca `DomainEvent` üretir; Sled WAL, PostgreSQL ve SQLite projection bu tek akıştan beslenir.
- Yazma işlemleri actor task'ında sıralıdır; okuma istekleri paylaşılan snapshot'tan yapılır.
- `--features full` ile PostgreSQL + Redis etkinleşir, aksi halde Sled WAL + in-memory idempotency.

## Önemli Davranışlar

- **PRICE_ONLY**: order book yok; dolum mark price ile yapılır. Market emir anında, limit emir mark'ı geçince dolar.
- **Hedge modda** BUY→LONG, SELL→SHORT; one-way modda BOTH beklenir (BOTH verilirse hedge'te reddedilir).
- **Boyut birimi USDT notional** (`position.rs`): `quantity` bir pozisyonun USDT değeridir (Long pozitif, Short negatif).
- **Replay**: crash sonrası event'ler tekrar oynatılarak son duruma ulaşılır.
- **Idempotency**: aynı `client_order_id` iki kez borsaya gönderilmez; önbellekteki yanıt döner.

## Bulgular (Riskler / Eksikler)

### Yüksek

1. **Reddedilen emir öncesi event yazımı** — `actor.rs:442`'de `OrderCreated` emit edildikten *sonra* `:453`'te yetersiz fon reddi döner. Reddedilen emir WAL'a event olarak işlenir → replay'de tutarsızlık (`OrderCreated` var, `OrderFilled` yok).

2. **Eksik replay** — `rebuild_from_events` (`actor.rs:210`) yalnızca `OrderFilled` + `FundingRateApplied` işler. `OrderCreated` (bekleyen limit emirleri), `Liquidation` ve kilitli marjlar replay'de yeniden kurulmaz. Çökme sonrası açık limit emirleri ve kilitli marj kaybolur.

### Orta

3. **`OrderCancelled` hiç emit edilmiyor** — actor'da iptal mekanizması yok; CLI'da cancel komutu yok. `DomainEvent::OrderCancelled` varyantı ölü kod.

4. **Metrikler eksik** — `record_liquidation` / `record_funding` (`metrics.rs`) hiç çağrılmıyor. `record_fill` yalnızca REST yolunda (`api.rs:254`) artıyor; order-ring'den gelen emirlerde sayaç güncellenmiyor.

5. **Bekleyen limit emirde order_id kayboluyor** — `actor.rs:519` `OrderAck { order_id: "PENDING" }` döner; istemci gerçek emir kimliğini alamaz, emri izleyemez/iptal edemez.

6. **Market/limit BTC dengesizliği** — `fill_limit` BTC free bakiyesini güncellerken (`actor.rs:629-635`), market fill hiç güncellemez. USDT-notional modelde tutarsız davranış.

7. **Auth kalıcılığı yok** — `main.rs:118`'de admin şifre hash'i her başlatmada taze salt ile hesaplanır; tek kullanıcı, kalıcı kimlik yok. JWT secret default: `paper-dev-secret-change-me`.

8. **Repo'ya çalışma verisi commit edilmiş** — `paper_wal/` (sled) ve `market_data.db` (SQLite WAL) git tarafından izleniyor; `.gitignore`'da değil.

### Düşük

9. **Bridge'de `blocking_recv`** — `bridge.rs:122`'de std thread actor yanıtını beklerken order-ring okuyucu tıkanır; latansı yüksekse ring dolabilir.

10. **`order_id` üretimi `now_ms()`** — `actor.rs:416` aynı milisaniyede iki emir üretilirse çakışabilir; `persist_trade` farklı bir id üretir.

11. **Hedge'de marj hesabı ile pozisyon değişimi uyumsuz** — `apply_fill_hedge` sıfıra kıstırırken (`position.rs:232-237`) marj hesabı tam `signed` değer üzerinden yapılır.

12. **CORS permissive** — `api.rs:408` tüm origin'lere açık; localhost için makul ama dışarıya açılırsa kısıtlanmalı.

## Not

- Proje `cargo build -p paper-service` ile temiz derlenir.
- Paper modüllerinde test yok (`#[cfg(test)]` bulunamadı).
- Likidasyon/funding mantığı çalışır durumda olmasına rağmen bu olayların metrik ve replay entegrasyonu eksiktir.
