# 🔧 EXECUTION-Engine Mimari Dokümanı

## Genel Bakış

**execution-engine**, Cycle Finance sisteminin **emir yürütme katmanıdır**. Binance USDT-M Futures için kurumsal emir yürütme, order lifecycle management, HFT için tokio + axum-based.

- Tanım: `src/lib.rs:1` — "Binance USDT-M Futures için kurumsal emir yürütme katmanı"
- Tek binary: `executiond` (`src/bin/executiond.rs`), CLI: `exec-cli` (`src/bin/exec-cli.rs`)
- **EXEC_MODE=LIVE** kullanılır (`config.rs:68-70`); yalnızca LIVE mod vardır.
- `EXEC_DRY_RUN=true` varsayılan (güvenlik önlemi; canlı emir borsa geçmez) (`executiond.rs:20-22`)

---

## Katmanlar ve Modül Sorumlulukları

| Katman | Modül | Sorumluluk |
|:---|:---|:---|
| **Client & HTTP** | `execution-engine/src/client/mod.rs`, `client/http.rs` | Binance REST yüzeyi, HTTP bağlantısı, retry, throttle |
| **İmzala** | `execution-engine/src/signer.rs` | HMAC-SHA256 imza (`signer.rs:24-31`); HMAC-SHA256 + hex |
| **Emir Domain** | `execution-engine/src/order.rs` | Order side/type/quantity/time-in-force, status, BinanceOrderResponse |
| **Sign & Gateway** | `execution-engine/src/signer.rs`, `gateway.rs` | HMAC-SHA256 imza, EngineHandle (actor komut köprüsü), LiveGateway |
| **Executor** | `execution-engine/src/executor/` | emir gönderimi (REST API), batch ≤5, iptal, idempotency, retry |
| **Snapshot & Monitor** | `execution-engine/src/state/projector.rs`, `execution-engine/src/state/snapshot.rs` | AccountSnapshot (Arc<RwLock>), order snapshot, bid/ask sync |
| **Risk** | `execution-engine/src/risk/` | limit sistemi, kill switch, rate limit, circuit breaker |
| **User Data Stream** | `execution-engine/src/user_data/stream.rs` | WS istemci: listenKey üret, keepalive, reconnect |
| **Idempotency** | `execution-engine/src/execution/idempotency.rs` | client_order_id ikinci kez borsaya gitmez |
| **Lifecycle** | `execution-engine/src/execution/lifecycle.rs` | InFlightRegistry, havada emir takibi, zaman aşımı |
| **Metrics** | `execution-engine/src/metrics.rs` | HDR histogram latency |
| **Event Sourcing** | `execution-engine/src/state/projector.rs` | WS olayları snapshot'a uygulayan saf fonksiyonlar |
| **Config** | `execution-engine/src/config.rs` | `ExecConfig`, TradingMode, `EXEC_` önekli env tabanlı config |
| **Error** | `execution-engine/src/error.rs` | ExecError hata modeli (Binance code alanı ayrıştırma) |

---

## Veri Akışı

```
user request (REST/WS)
  │
  ├── BinanceClient.place_order
  │     ├── build_url: params + timestamp + recvWindow
  │     ├── sort → query string
  │     ├── BinanceSigner.sign HMAC-SHA256 hex (signer.rs:24-31)
  │     ├── X-MBX-APIKEY header (http.rs:112-114)
  │     ├── POST /fapi/v1/order (fapi.binance.com)
  │     │
  │     ├── Binance REST (REST API)
  │     │     └── Order status → idempotency cache
  │     │
  │     └── Binance WS: user-data stream (listenKey)
  │           ├── gzip binary frame decode (decoder.rs:11-25)
  │           ├── user_event_type() (decoder.rs:45-51)
  │           └── user_tx (mpsc::unbounded_channel) → ExecutionActor
  │
  ├── ExecutionActor.handle_command (actor.rs:186-251)
  │     ├── snapshot.ready kontrol (actor.rs:257)
  │     ├── kill switch (actor.rs:260)
  │     ├── quoteOrderQty → quantity (mark fiyatıyla) (actor.rs:266-280)
  │     ├── risk.check → risk-engine (actor.rs:291)
  │     ├── idempotency cache (actor.rs:298-301)
  │     ├── preflight normalize+doğrula (actor.rs:304)
  │     ├── DRY_RUN şubesi (actor.rs:307-323)
  │     └── in_flight insert (actor.rs:326)
  │
  ├── BinanceClient.place_order (client/mod.rs:99) → HttpClient.request (http.rs:93)
  │     ├── retry (http.rs:101-196)
  │     ├── timeout/connect error → retry (http.rs:122-130)
  │     └── 429/418 retry-after → retry-after başlığıyla (http.rs:143-155)
  │
  └── ExecutionActor.handle_user_event (actor.rs:539-581)
        ├── WS olayları → snapshot → REST API → order/pozisyon control (service/api.rs)
        └── projeksiyon → AccountSnapshot (Arc<RwLock>)
```

---

## Giriş Noktaları

### Binary'ler

| Binary | Dosya | Giriş Noktasi |
|:---|:---|:---|
| `executiond` | `src/bin/executiond.rs` | `executiond.rs:25-66` — CLI argümanları: `--host`, `--port`, `--no-dry-run` |
| `exec-cli` | `src/bin/exec-cli.rs` | `exec-cli.rs:15-86` — REST API uçları |

### RUN_MODE

- `execution-engine` içinde **yok**
- `EXEC_MODE=LIVE` kullanılır (`config.rs:68-70`); `TradingMode` yalnızca `Live` içerir

### Komutlar

| Komut | Kullanım |
|:---|:---|
| `--host` | `127.0.0.1` varsayılan |
| `--port` | `3010` varsayılan |
| `--no-dry-run` | EXEC_DRY_RUN varsayılan'tan geçersiz kılar |

---

## Thread / Task Yapısı

- **Runtime:** tokio (`#[tokio::main]`, `executiond.rs:25`, `exec-cli.rs:98`)
- **Task'lar:**
  1. `actor_task` — ExecutionActor::run (tek-yazıcı komut/olay döngüsü)
  2. `stream_task` — UserDataStream::run (WS + keepalive + reconnect)
  3. `spawn_rest` — axum REST API ayrı görev
- **Kanallar:**
  - `cmd_tx`/`cmd_rx`: `mpsc::unbounded_channel<Command>` (`lib.rs:81`)
  - `user_tx`/`user_rx`: `mpsc::unbounded_channel<UserEvent>` (`lib.rs:82`)
  - oneshot reply kanalları: `gateway.rs:37,48,64`
  - flume köprüsü (eski API): `lib.rs:134-162`
- **Paylaşılan state:** `Arc<RwLock<AccountSnapshot>>` (parking_lot), `Arc<Metrics>`, `Arc<KillSwitch>`
- **Timer'lar:** `tokio::select!` + interval: reconcile (actor.rs:159-161), in-flight kontrol 1 sn (`actor.rs:163-164`)

---

## Kritik Algoritmalar

### İmzalama (HMAC)
`Hmac<Sha256>` + hex (`signer.rs:24-31`); `new_from_slice` + `update` + `finalize` → 64 hex karakter.

### Order ID
- İstemci tarafı: UUID v4 (`preflight.rs:231`), ≤36 karakter kontrolü
- Borsa tarafı: `orderId` (`order.rs:334`)
- **Idempotency:** aynı `newClientOrderId` tekrarı önbellekten yanıtlanır (`actor.rs:298-301`)

### Time-in-Force (TIF)
`GTC/IOC/FOK/GTX` (`order.rs:93-111`); LIMIT tipi TIF **zorunlu** (`preflight.rs:122-124`). LIMIT_MAKER TIF taşır (POST_ONLY).

### Retry
`MAX_ATTEMPTS=3`, üstel backoff `250ms * 2^(attempt-1)` (`http.rs:17-18`); timeout/connect error (`http.rs:122-130`), 429/418 `retry-after` başlığıyla (`http.rs:143-155`).

### Preflight Normalizasyonu
- Miktar step/floor precizyon (`preflight.rs:166-201`)
- fiyat tick yarım-yukarı (`preflight.rs:204-221`)
- hedge modda `BOTH` reddi / one-way'de `LONG/SHORT` reddi (`preflight.rs:57-71`)
- MIN_NOTIONAL (reduce hariç) (`preflight.rs:127-140`)
- MAX_NUM_ALGO_ORDERS limit 0 ise koşullu emir yasağı (`preflight.rs:143-148`)
- `quoteOrderQty` → mark fiyatından quantity'e çevirir (Binance USDT-M futures desteği yok) (`actor.rs:263-280`)

### In-Flight Lifecycle
`in_flight` havada emir takibi, 5000ms zaman aşımı (`actor.rs:139`), `max_in_flight` default 64 (`config.rs:115`). Terminal durumda confirm, hâlâ açıksa timeout sıfırlama (`actor.rs:649-678`).

### Kill Switch
`config.rs:107` (dosya-tabanlı). `risk_engine::kill_switch::KillSwitch` re-export (`risk/kill_switch.rs:5`).

---

## Dış Bağımlılıklar

| Bağımlılık | Kaynak | Kullanım |
|:---|:---|:---|
| `hmac 0.12`, `sha2 0.10`, `hex 0.4` | İmzalama | signer.rs:1-3 |
| `tokio 1.0` (full), `tokio-tungstenite 0.20` (rustls) | Runtime + WS | `stream.rs:11` |
| `futures-util` | WS split/stream | `stream.rs:11` |
| `serde`, `serde_json` | (de)serialization | `order.rs`, `types` |
| `dotenvy` | `.env` yükleme | `executiond.rs:27` |
| `flume 0.11` | Eski API köprüsü | `lib.rs:135` |
| `rust_decimal 1.34` | Parasal aritmetik | `order.rs` |
| `parking_lot 0.12` | `RwLock`/`Mutex` | `lib.rs:77` |
| `reqwest 0.11` (rustls) | REST istemci | `http.rs:21` |
| `axum 0.8`, `tower`, `tower-http` | REST API | `service/api.rs` |
| `tracing`, `tracing-subscriber` | Loglama | `executiond.rs:28-33` |
| `uuid 1.6` (v4) | client_order_id | `preflight.rs:231` |
| `clap 4.6` | CLI | `executiond.rs:13-23`, `exec-cli.rs:15-86` |
| `flate2 1.0` | gzip decode | `decoder.rs:7` |
| `async-trait` | Gateway trait | `gateway.rs:284` |
| `hdrhistogram 7.6` | Latency histogram | `metrics.rs:6` |
| `rand 0.8` | Argon2 salt | `service/mod.rs:18` |
| `argon2 0.5` | Admin auth | `service/mod.rs:16-17` |
| `jsonwebtoken 9` | JWT | `api.rs:18` |
| `risk-engine` (yerel path) | Ortak risk çekirdeği | `risk/checks.rs` |
| `execution-engine` (yerel path) | Ortak risk çekirdeği | `risk/checks.rs` |
| `ai-engine` (yerel path) | RiskEngine politikası | `gates.rs:31` |

---

## Satır Sayıları

| Dosya | Satır |
|:---|:---|
| `src/lib.rs` | 11 |
| `src/order.rs` | 48 |
| `src/signer.rs` | 31 |
| `src/client/mod.rs` | 496 |
| `src/client/http.rs` | 138 |
| `src/execution/actor.rs` | 701 |
| `src/execution/preflight.rs` | 456 |
| `src/execution/lifecycle.rs` | 90 |
| `src/execution/idempotency.rs` | 36 |
| `src/risk/*.rs` | 86 |
| `src/state/*.rs` | 35 |
| `src/types/*.rs` | 55 |
| `src/bin/executiond.rs` | 66 |
| `src/bin/exec-cli.rs` | 86 |
| `src/bin/executiond.rs` | 5 |
| **Toplam (src + tests)** | **6.756** |
| `Cargo.toml` | 46 |

---

## Dönüşümler

| Başlangıç | Sonuç |
|:---|:---|
| `client_order_id` (UUID v4) | `newClientOrderId` = UUID v4 (`preflight.rs:230-232`) |
| `orderId` | `orderId` (Binance tarafında) |
| `status` | `OrderStatus` (N/A/Binary) → `is_terminal` / `is_open` |
| **HMAC-SHA256 hex** | Signer: `hex_encode(HmacSha256(data))` (signer.rs:45) |

---

## Sonuç

execution-engine, Binance USDT-M Futures için kurumsal emir yürütme. Tokio + axum basitleştirilmiş, HMAC-SHA256 imza, idempotency, rate limit, circuit breaker, kill switch ile güvenlik katmanı. `EXEC_DRY_RUN=true` varsayılan (güvenlik önlemi). `EXEC_MODE=LIVE` komutla ayarlanır.