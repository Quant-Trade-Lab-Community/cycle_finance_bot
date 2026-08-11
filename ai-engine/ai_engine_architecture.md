# 🧠 AI-Engine Mimari Dokümanı

## Genel Bakış

**AI-Engine**, Cycle Finance sisteminin **LLM tabanlı yapay zeka agent katmanıdır**. Sembol başına piyasa bağlamı toplar, üç agent (sinyal / risk / duygu) çıktı üretir, koordinatör tek karara indirger, kararı risk kapısından geçirir ve **paper** (shared-memory order ring) veya **canlı** (executiond REST) icra eder. Bir daemon'dur: periyodik döngü + HTTP durum API'si sunar.

- Tanım: `src/main.rs:1-8` — "Periyodik döngü: sembol bağlamını toplar → agent'lar çalışır → koordinatör karar verir → risk gate → icra"
- Kütüphane: `src/lib.rs:1-5` — "LLM agent'ları ... strateji/risk/duygu analizi yapar, koordinatör kararı sentezler ve emri risk kapısından geçirip paper veya canlı icra eder"

**Tek binary:** `src/main.rs` (`#[tokio::main]`, `main.rs:59-60`). CLI argümanı yok; `RUN_MODE` kavramı yok.

---

## Katmanlar ve Modül Sorumlulukları

| Katman | Modül | Sorumluluk |
|:---|:---|:---|
| **Config** | `config.rs` | `ai.toml` / `AI_CONFIG` yükleme; 5 alt yapılandırma (providers, schedule, execution, risk, context); API anahtarları env'den. Varsayılanlar + testler (`config.rs:152-167`) |
| **Context** | `context.rs` | Sembol başına birleşik `MarketContext` üretimi: fiyat (price-feed :3004), indikatör (calc-ind :3007), piyasa yapısı (detect-ms MSMP 2.0 :3002), hesap (paper JWT :8080), opsiyonel haber kaynağı |
| **Risk Gate** | `gates.rs` | `risk-engine` RiskEngine (risk.toml politikası) + deterministik boyut kırpma + agent veto kuralları. `GateOutcome` (Executed/Held/Rejected) denetim izi üretir |
| **Agents** | `agents/` | `signal.rs` (strateji/yon), `risk.rs` (risk skoru + veto), `sentiment.rs` (haber duyarlılığı), `coordinator.rs` (nihai karar sentezi). Ortak `Agent` trait + `AgentOutput` enum (`agents/mod.rs:32-45`) |
| **Executor** | `executor/` | İcra: `paper.rs` (order ring'e yazma), `live.rs` (executiond REST), `mod.rs` (mode/approval politikası + HITL onay) |
| **LLM** | `llm/` | OpenAI ve Anthropic provider soyutlaması (`LlmProvider` trait, `complete_json`); `make_provider` fabrikası; `LlmError` |
| **Domain** | `lib.rs` | Ortak veri tipleri: `Action`, `PriceSnapshot`, `MarketContext`, `SignalOutput`, `RiskOutput`, `SentimentOutput`, `FinalDecision`, `now_ms` |

---

## Veri Akışı

```
price-feed (:3004) ─┐
calc-ind   (:3007) ─┤   context.rs (tokio::join! paralel)
detect-ms  (:3002) ─┼─► MarketContext ─► Agent'lar (signal/risk/sentiment)
paper hesap (:8080)─┤        │
haber (ops.)        ─┘        ▼
                        Coordinator.decide → FinalDecision
                                │
                        RiskGate.process (gates.rs)
                        │ HOLD/veto → Held/Rejected (denetim)
                        │ onay → quantity.min(max_notional/mark)
                                ▼
                        Executor
                        ├─ paper → /cycle_finance_orders ring
                        └─ live  → executiond POST /api/v1/orders (:3010)
                                │
                        HTTP status API (:3110) — /api/health, /api/status
```

### Girdi Kaynakları

| Kaynak | Yöntem | Adres | Kod |
|:---|:---|:---|:---|
| price-feed | `GET /api/lastprice/{symbol}` | `:3004` | `context.rs:70-71` |
| calc-ind | `POST /api/calc` + `/cycle_finance_orders` ring okuma | `:3007` | `context.rs:92-128` |
| detect-ms | `GET /api/ms?symbol=&interval=&limit=` (MSMP 2.0) | `:3002` | `context.rs:131-135` |
| paper hesap | JWT login → balance + positions | `:8080` | `context.rs:166-217` |
| risk politikası | `risk.toml` dosyası | kök dizin | `gates.rs:39` |

### Çıktılar

- **Paper icra → shared memory:** `/cycle_finance_orders` POSIX `shm_open` + `mmap` ring (10.000 slot) — `paper.rs:8,44-50`
- **Canlı icra → REST:** executiond `POST /api/v1/orders` (JWT), `:3010` — `live.rs:43-85`
- **HITL ara dosyalar:** bekleyen emir `/tmp/ai_pending.json`, onay `/tmp/ai_approve.txt` — `executor/mod.rs:103-131`
- **HTTP durum API'si:** `:3110` — `main.rs:216-237`
- **Log:** stdout tabanlı (kalıcı DB yok; durum yalnızca bellekte)

---

## Thread / Task Yapısı

```
#[tokio::main] main.rs
├── Task 1: HTTP server (axum, :3110) — tokio::spawn
│       └── parking_lot::RwLock<Option<RunSummary>> üzerinden durum paylaşımı
└── Task 2: Ana döngü — tokio::time::sleep(interval_secs) polling (spin-loop YOK)
        └── her sembol için run_cycle():
            ├── context fetch'leri tokio::join! ile paralel (context.rs:52,176)
            ├── indikatör okumaları tokio::spawn + 64MB stack'li std::thread (context.rs:100-117)
            └── 3 agent SIRALI await (main.rs:161-164)
```

> ⚠️ **Not:** `main.rs:3` yorumu agent'ların "paralel çalıştığını" söyler; gerçekte üç agent ardışık `await` edilir. Paralellik yalnızca context fetch'lerinde ve HTTP/ana-döngü task ayrımındadır.

---

## Güvenlik Katmanları (fail-safe prensibi)

1. **LLM fail-safe default'lar** — LLM yoksa: SignalAgent → HOLD, RiskAgent → nötr 0.5, SentimentAgent → nötr 0.0; "asla kör emir üretilmez" (`llm/mod.rs:5`)
2. **Risk agent veto** — `risk.veto` → HOLD + qty=0 (`coordinator.rs:42-46`)
3. **Anomali kuralı** — `anomaly_veto && risk_score >= 0.8` → otomatik Rejected (`gates.rs:85-90`)
4. **Deterministik boyut kırpma** — `quantity.min(max_notional / mark)` (`gates.rs:94-97`)
5. **RiskEngine politikası** — `evaluate(intent)` reddederse → Rejected (`gates.rs:128-135`)
6. **Opsiyonel HITL onayı** — `/tmp` dosya tabanlı bekleme (`executor/mod.rs:114-131`)

---

## Dış Bağımlılıklar

**Workspace crates:** `axum 0.8`, `tokio 1.0` (full), `serde`, `serde_json`, `rust_decimal 1.34`, `reqwest 0.11` (rustls), `toml 0.8`, `chrono 0.4`, `dotenvy 0.15`, `async-trait 0.1`, `parking_lot 0.12`

**Path bağımlılıkları:**
- `transport` → `../cycle-engine/transport` (shared-memory order ring)
- `risk-engine` → `../risk-engine` (RiskEngine politika motoru)
- `calc-ind` → `../services-engine/calc-ind` (indikatör REST client + ring okuma)

---

## Satır Sayıları (toplam 2.116 .rs satır)

| Dosya | Satır | Dosya | Satır |
|:---|:---|:---|:---|
| `src/main.rs` | 237 | `src/agents/signal.rs` | 108 |
| `src/lib.rs` | 219 | `src/agents/risk.rs` | 90 |
| `src/config.rs` | 231 | `src/agents/sentiment.rs` | 71 |
| `src/gates.rs` | 145 | `src/executor/mod.rs` | 133 |
| `src/context.rs` | 297 | `src/executor/paper.rs` | 62 |
| `src/agents/mod.rs` | 62 | `src/executor/live.rs` | 86 |
| `src/agents/coordinator.rs` | 143 | `src/llm/*.rs` | 232 |
