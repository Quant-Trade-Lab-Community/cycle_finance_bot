# 🔧 EXECUTION-Engine Fonksiyonel Süreçler

## Giriş

Tek binary (`executiond`), CLI (`exec-cli`). `EXEC_MODE=LIVE` kullanılır. `EXEC_DRY_RUN=true` varsayılan (güvenlik önlemi).

**Başlatma:**
- `cargo run -p execution-engine` (kök workspace üyesi)
- `./target/debug/executiond --host 127.0.0.1 --port 3010`
- `./target/debug/exec-cli`

---

## Süreç 1: Emir Giriş Noktası (REST / WS)

`execution-engine/src/client/mod.rs:99-360`

### REST API (HTTPS)
```
POST /fapi/v1/order
Headers: X-MBX-APIKEY (HMAC-SHA256), Content-Type: application/json
Body: symbol, side, type, quantity, price (optional), timeInForce (GTC/IOC/FOK/GTX)
```

### WebSocket (Binance USDT-M Futures)
```
user-data stream: wss://fstream.binance.com/ws/<listenKey>
Header: X-MBX-APIKEY, Content-Type: application/json
Body: gzip binary frame (decoder.rs:11-25) → user_event_type() (decoder.rs:45-51)
```

---

## Süreç 2: İmzalama

`execution-engine/src/signer.rs:24-31`

```
Hmac<Sha256>::new_with_key(hmac_key)
  .update(request_payload)
  .finalize() → hex_string (64 karakter)
```

Kanal: `X-MBX-APIKEY` header (`http.rs:112-114`), `request_body` → `HMAC-SHA256 hex` (`signer.rs:24-31`).

---

## Süreç 3: Emir İletişimi

```
user (REST/WS)
  ├── put_order (POST /fapi/v1/order)
  │   ├── BinanceClient.place_order
  │   │   ├── build_url: params + timestamp + recvWindow
  │   │   ├── sort → query string
  │   │   ├── BinanceSigner.sign HMAC-SHA256 (signer.rs:24-31)
  │   │   ├── X-MBX-APIKEY header (http.rs:112-114)
  │   │   ├── POST /fapi/v1/order (fapi.binance.com)
  │   │   │
  │   │   └── Binance REST POST /fapi/v1/order
  │   │       ├── order_id → idempotency cache kontrolü (actor.rs:298-301)
  │   │       ├── order_id = UUID v4 (preflight.rs:231)
  │   │       └── 200 ok → in_flight insert (actor.rs:326)
  │   │
  │   └── Binance WS: user-data stream (listenKey) → executiond: order_id → emir
  │
  └── executiond (executiond.rs:16) — kanala basar (exec-cli veya tokio)
```

---

## Süreç 4: Emir Yürütme (Actor)

`execution-engine/src/execution/actor.rs:166-251`

```
ExecutionActor.handle_command(cmd)
  ├── snapshot.ready kontrol (actor.rs:257)
  │   → ready: emir gönderilebilir
  │   → not ready: emir bekle (güvenlik)
  ├── kill switch kontrol (actor.rs:260)
  ├── quoteOrderQty → quantity (mark fiyatıyla) (actor.rs:266-280)
  │   → quoteOrderQty: mark fiyatına göre order_quantity hesaplama
  ├── risk.check → risk-engine (actor.rs:291)
  ├── idempotency cache kontrol (actor.rs:298-301)
  │   → aynı newClientOrderId tekrarı önbellekten yanıtlanır
  ├── preflight normalize+doğrula (actor.rs:304)
  │   → sembol kuralları, precizyon/step yuvarlama, hedge/one-way tutarlılığı, MIN_NOTIONAL
  ├── DRY_RUN şubesi (actor.rs:307-323)
  │   → emir gönderilmez, doğrulanır ama borsaya GİMTER
  ├── in_flight insert (actor.rs:326)
  │
  └── order_ack (Binance tarafında) → order_id kontrolü
```

### Idempotency Cache (actor.rs:140)
- Boyutu: 10_000 (`actor.rs:140`)
- aynı `newClientOrderId` tekrarı önbellekten yanıtlanır
- cache taraması: `(orderId, side, type, quantity) → bool`

---

## Süreç 5: Risk kontrolü

`execution-engine/src/risk/`

- `RiskPolicy::check(order_request) → redderse → emir reddedilir` (`risk/checks.rs:38-58`)
- `risk.check()` → `RiskEngine::evaluate()` (execution-engine/src/risk/checks.rs:38-58)
- **kill switch:** `config.rs:107` dosya-tabanlı. Dosyaya yazıldığında `kill_switch.engage()` (orchestrator.rs:33).

### Rate Limit
- Kayan pencere: `VecDeque<Instant>`, 60s'den eski kayıtlar `prune` ile düşer (`limits.rs:52-57`)
- Breaker: `record_rejection()` eşiği aşarsa `true` (kill switch arm'ı tetikler, orchestrator.rs:75-86)

---

## Süreç 6: WebSocket Veri akışı (stream_task)

`execution-engine/src/user_data/stream.rs:41-78`

```
listenKey → keepalive → user_data stream (listenKey)
  ├── gzip binary frame decode (decoder.rs:11-25)
  ├── user_event_type() (decoder.rs:45-51)
  └── user_tx (mpsc::unbounded_channel) → ExecutionActor.handle_user_event
```

**reconnect:** `1→60 sn` (listenKey önbellekten geri çekilir).

---

## Süreç 7: Öne Bakılan Durumlar

| Durum | Kod | Açıklama |
|:---|:---|:---|
| `is_retryable()` | `error.rs:43-58` | retry edilebilir hatalar (timeout, connect, 429) |
| `retry_after` | `http.rs:143-155` | 429 hatasındaki `retry-after` başlığıyla |
| `Max_ATTEMPTS=3` | `http.rs:17-18` | üstel backoff: `250ms * 2^(attempt-1)` |
| `EXEC_DRY_RUN` | `config.rs:74-78` | `true` → emir borsaya GİMTER, doğrulama yapılsa bile |
| `in_flight` zaman aşımı | `actor.rs:139` | 5000ms |
| `max_in_flight` | `config.rs:115` | default 64 |
| `EXEC_RESYNC_ON_RECONNECT` | `config.rs:109-112` | her bağlantıda tam resync (stream.rs:63) |
| `EXEC_RECONCILE_INTERVAL_SEC` | `config.rs:113` | default 300 sn |

---

## Süreç 8: Emir Dönüşümleri

| Başlangıç | Sonuç |
|:---|:---|
| `newClientOrderId` (UUID v4) | `client_order_id` (preflight.rs:231) |
| `orderId` | `orderId` (Binance tarafında) (order.rs:334) |
| `order_status` | `OrderStatus` (N/A/Binary) → `is_terminal` / `is_open` (order.rs:236-246) |
| **HMAC-SHA256 hex** | **Signer: hex_encode(HmacSha256(data))** (signer.rs:45) |

---

## Süreç 9: Snapshot & Projector

`execution-engine/src/state/snapshot.rs:14-25`:
- `AccountSnapshot` — `Arc<RwLock>` paylaşılan okuma görünümü
- `ready` bayrağı (eşitleme tamamlanmadan emir yok)

`execution-engine/src/state/projector.rs:12-72` — WS olaylarını snapshot'a uygulayan saf fonksiyonlar:
- bakiye/pozisyon upsert (projector.rs:74-139)
- açık emir senkronu (projector.rs:146-188)
- işaretli fill hesabı hedge/one-way (projector.rs:191-255)

---

## Süreç 10: API Uygulaması

`execution-engine/src/service/api.rs:100-141`:
- JWT login/refresh (api.rs:173-204)
- emir/pozisyon/control uçları
- Tüm yazma işlemleri actor'e komut (service/api.rs)

---

## Thread / Task Haritası Özeti

| # | Süreç | Tip | Bloklanır mı? |
|:---:|:---|:---|:---|
| 1 | HTTP API (axum): emir gönderme | Tokio Task | Async I/O |
| 2 | WebSocket stream_task | Tokio Task | Async I/O |
| 3 | Emir yürütme (actor): tek-yazıcı döngüsü | Tokio Task | async select! |
| 4 | Snapshot güncelleme | Tokio Task | async |
| 5 | REST API görevleri | Tokio Task | async |
| 6 | Flume köprüsü (eski API) | Tokio Task | async |
| 7 | User Data WS keepalive | tokio::spawn | async |
| 8 | Risk check | Tokio Task | async |
| 9 | In-flight kontrol | tokio::spawn | async |
