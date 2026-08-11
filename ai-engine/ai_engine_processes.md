# 🧠 AI-Engine Fonksiyonel Süreçler

## Giriş

Tek binary (`ai-engine`), `RUN_MODE` kavramı yok. Daemon: periyodik LLM karar döngüsü + HTTP durum API'si. Yapılandırma `ai.toml` (veya `AI_CONFIG`), API anahtarları `.env` / ortam.

**Başlatma:** `cargo run -p ai-engine` (kök workspace üyesi). Port `:3110`.

---

## Süreç 1: Başlatma ve Bileşen Kurulumu

```
main.rs:59-103
├── dotenvy::dotenv() → .env yükle (main.rs:61)
├── AiConfig::load() → ai.toml (yoksa varsayılan) (main.rs:62)
├── LLM provider üretimi (OpenAI/Anthropic/none) (main.rs:71)
├── Bileşenler: ContextBuilder, RiskGate, Executor, Coordinator,
│              RiskAgent, SentimentAgent (main.rs:79-84)
└── HTTP durum API'si tokio::spawn (127.0.0.1:3110) (main.rs:94-103)
```

---

## Süreç 2: Ana Karar Döngüsü (`run_cycle`)

`main.rs:106-195` — sonsuz döngü: `run_cycle()` → `last_run` yaz → `interval_secs` uyku.

Her sembol için (BTCUSDT, ETHUSDT, SOLUSDT, HEIUSDT — `ai.toml`):

| Adım | İşlem | Kod |
|:---:|:---|:---|
| 1 | `context_builder.build(symbol)` → `MarketContext` | `main.rs:151` |
| 2 | `is_healthy()` — sağlıksız fiyat kaynağı varsa sembol atlanır | `main.rs:153-156` |
| 3 | `risk_gate.on_mark(symbol, mark)` — canlı mark fiyatı risk engine'ine beslenir | `main.rs:158-159` |
| 4 | Sinyal agent → `SignalOutput` | `main.rs:161` |
| 5 | Risk agent → `RiskOutput` | `main.rs:162` |
| 6 | Duygu agent → `SentimentOutput` | `main.rs:163` |
| 7 | `coordinator.decide(...)` → `FinalDecision` | `main.rs:181` |
| 8 | `risk_gate.process(decision, mark, executor)` → `GateOutcome` | `main.rs:183` |
| 9 | Sonuç `DecisionView` → `RunSummary` | `main.rs:186` |

> 4-6 adımları kodda **sıralı** çalışır (yorum paralel dese de).

---

## Süreç 3: Context Toplama (Paralel Fetch)

`context.rs:52-245`

```
tokio::join!  ──►  indikatör + yapı fetch'i paralel  (context.rs:52)
                    ├─ calc-ind POST /api/calc + ring okuma (context.rs:92-128)
                    │     └─ her indikatör tokio::spawn; içinde 64MB stack'li
                    │        std::thread ile ring okuma (context.rs:100-117)
                    └─ detect-ms GET /api/ms (context.rs:131-135)
tokio::join!  ──►  balance + positions paralel (context.rs:176)
haber (ops.)  ──►  news_feed_url REST (context.rs:220-245)
```

5 indikatör: rsi, macd, bbands, vwap, atr (`context.rs:16`) → `fill_indicators` ile birleştirme (`context.rs:248-274`).

---

## Süreç 4: Risk Kapısı (`RiskGate`)

`gates.rs:77-144` — pipeline sırası:

```
1. HOLD kararı        → anında Held
2. decision.veto      → Rejected
3. anomaly_veto && risk_score >= 0.8 → otomatik Rejected  (gates.rs:85-90)
4. Boyut kırpma       → quantity.min(max_notional / mark) (gates.rs:94-97)
5. Kırpma sonrası qty <= 0 → Held
6. OrderIntent kur    → strategy_id: 900 (gates.rs:111-125)
7. RiskEngine.evaluate(intent)  → reddederse Rejected     (gates.rs:128-135)
8. Executor'a ilet    → GateOutcome (Executed/Held/Rejected)
```

`on_mark` bayat-mark reddini önler (`gates.rs:55-68`).

---

## Süreç 5: Koordinatör Karar Sentezi

`agents/coordinator.rs`

- Risk vetosu her zaman öncelikli: `risk.veto` → HOLD + qty=0 (`coordinator.rs:42-46`)
- Deterministik fallback: trade için `confidence >= 0.5` ve qty ≠ 0; sentiment `< -0.3` ise qty `× 0.5` (`coordinator.rs:78-112`)
- LLM JSON şeması sıkı ("BİREBİR uy", `coordinator.rs:12-20`); parse hatası → fallback

---

## Süreç 6: İcra (Paper / Live / HITL)

`executor/mod.rs:22-29` — mode `paper | live | both | none`.

- **Paper:** `/cycle_finance_orders` shm ring'e `OrderRingBuffer::push` (`paper.rs:44-50`)
- **Live:** executiond'e JWT login → `POST /api/v1/orders` (`live.rs:43-85`)
- **HITL (ops.):** emir `/tmp/ai_pending.json`, onay `/tmp/ai_approve.txt` — 1 sn poll (`executor/mod.rs:114-131`)

---

## Süreç 7: HTTP Durum API'si

`main.rs:216-237` — axum, `127.0.0.1:3110`

| Uç | Açıklama |
|:---|:---|
| `GET /api/health` | Sağlık kontrolü (`main.rs:216-230`) |
| `GET /api/status` | Son döngü özeti (`RunSummary`, `main.rs:232-237`) |

Durum `parking_lot::RwLock<Option<RunSummary>>` içinde; kalıcı depo yok (`main.rs:53-57`).

---

## Süreç 8: Fail-safe Davranışları

| Koşul | Sonuç | Kod |
|:---|:---|:---|
| LLM provider yok | SignalAgent → HOLD, Risk → 0.5, Sentiment → 0.0 | `llm/mod.rs:5` |
| Sağlıksız fiyat kaynağı | Sembol atlanır | `main.rs:153-156` |
| Anomali veto + risk ≥ 0.8 | Otomatik red | `gates.rs:85-90` |
| RiskEngine reddi | Rejected | `gates.rs:128-135` |
| HITL onayı yok | Emir bekletilir | `executor/mod.rs:114-131` |

---

## Thread / Task Haritası Özeti

| # | Süreç | Tip | Bloklanır mı? |
|:---:|:---|:---|:---|
| 1 | HTTP durum API'si (:3110) | Tokio Task | Async I/O |
| 2 | Ana karar döngüsü | Tokio Task (main) | `interval_secs` uyku |
| 3 | Context fetch'leri | Tokio `join!` | Async I/O |
| 4 | Ring okuma (indikatör) | std::thread (64MB stack) | Evet |
| 5 | HITL onay poll | dosya poll | 1 sn sleep |
