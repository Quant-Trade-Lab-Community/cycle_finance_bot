# Çoklu Borsa Finans Merkezi — Genişletme Planı

> Mevcut Cycle Finance sisteminin çoklu borsa / çoklu hesap / portföy yönetim merkezine dönüşüm notları.

---

## 1. Mevcut Durum Analizi

**Şu an sistem uçtan uca Binance'e sabitlenmiştir:**

| Katman | Binance Bağımlılığı |
|---|---|
| `execution-engine` | `signer.rs` (HMAC-SHA256), `/fapi/v1` REST endpoint'leri, user-data WS, sembol filtreleri/precizyon Binance'e özel |
| `cycle-engine/adapter` | Veri WS yalnızca Binance |
| `config.rs` | Tek `base_url` / `ws_url`, tek hesap varsayımı |

**Ancak dönüşüm için hazır iskelet mevcut:**

- `Gateway` trait'i zaten var (`LiveGateway` / `PaperGateway` → `execution-engine/src/gateway.rs:284`)
- `OrderRequest`, `AccountSnapshot`, `risk-engine` muhasebesi büyük oranda borsa-nötr
- `executiond`'nin preflight / idempotency / kill-switch katmanı borsadan bağımsız mantık

**Sonuç:** Yapılabilir — bu bir mimari genişletme, yeniden yazım değil. Mevcut kodun ~%80'i korunur.

---

## 2. Forex Tarafında API Desteği Olan Sağlayıcılar

Forex tarafında "borsa" kavramı yoktur — işlemler **broker/aracı kurum** üzerinden yapılır (çoğu CFD).

### Tam API (REST + Streaming)

| Sağlayıcı | API | Özellik |
|---|---|---|
| **OANDA** | REST v20 + WS | En olgun kamu API'si, forex/CFD, sandbox ortamı |
| **IG Group** | REST + Lightstreamer WS | FX, CFD, spread bet, demo hesap |
| **Saxo Bank** | OpenAPI (REST+WS) | Kurumsal seviye, margin hesaplama dahil |
| **FXCM** | REST Trading API | FIX de var, api-demo |
| **Dukascopy** | JForex API (Java) | Kurumsal likidite, FIX |
| **Deriv** | REST/WS (API v2) | FX + binary, kolay başlangıç |
| **Alpaca** | REST (forex dahil) | ABD regüle, kripto + hisse de var |

### cTrader / MT4-MT5 tabanlı (platform API'si)

- **Pepperstone, IC Markets, FxPro** → **cTrader Open API** (REST 2.0, .NET/Rust/Python istemcileri)
- MT4/MT5'te **resmi açık REST API yok** — genelde üçüncü parti köprüler gerekir (MetaApi, MQL→REST). FIX arayüzü kurumsal hesaplara verilir.

### Kurumsal / FIX

- **LMAX Global** — FIX API (ESMA regüle, netleştirme) — en "borsa benzeri" yapı
- **PrimeXM / Currenex / EBS** — kurumsal likidite havuzları, FIX

### Kendi sistemimiz için pratik sıralama

1. **OANDA** — REST netliği + demo + `Gateway` trait'ine en az sürtünmeyle oturur
2. **IG / Saxo** — hesap/sembol/pozisyon modeli zengin
3. **cTrader** — zaten REST, Rust istemcisi mevcut
4. **MT4/MT5** — yalnızca son çare (köprü katmanı gerekir)

> Not: `ExecutionActor` tek-borsa tekliğini, per-borsa `Exchange` trait'i + router ile çözersek bu broker'ların REST'leri birbirine çok benzer (emir/pozisyon/denge). Asıl fark **sembol sözleşmesi ve imzalama** şemasındadır.

---

## 3. Dönüşümde Yapılacaklar (Özet)

1. **`Exchange` trait'i + per-borsa implementasyonları** — asıl iş bu (~%40)
2. **Signer soyutlaması** — HMAC → trait, per-borsa (Ed25519 vb.) (~%15)
3. **Veri adapter'ları** — her borsa için ayrı WS client (~%20)
4. **`exchange` alanının** emir/pozisyon/snapshot'a eklenmesi + router (~%15)
5. **Çapraz-borsa risk agregasyonu** — marj/pozisyon tek havuzda birleşmeli (~%10)

> **Dikkat noktası:** Risk motoru şu an tek hesap varsayıyor. Çoklu borsada aynı coin iki yerde açıksa toplam exposure hesabı değişir — bu en hassas kısımdır.

---

## 4. Finans Merkezi Mimari Genişletmesi

Finans merkezi = hesap kimliği (identity) + borsa nötr emir + birleşik portföy + çapraz-hesap risk.

### 4.1 Hesap & Kimlik Katmanı (yeni)

```
Account { id, exchange, name, credentials, leverage_budget, risk_profile }
AccountRegistry → Vault (API anahtarları merkezi, rotasyon destekli)
```

- `account_id` tüm emir/pozisyon/snapshot'lara girer
- Config: `accounts.toml` (Vault'taki anahtar referanslarıyla)
- `Exchange` trait'i: `BinanceExchange`, `OandaExchange`, `PaperExchange` → `Router`

### 4.2 Order Router / Akıllı Yönlendirme (yeni)

- Stratejiler `submit_order`'a hesap bilmeden `HEDEF: (sembol, büyüklük, risk_bütçesi)` verir
- Router kuralları: en iyi fiyat → likidite → komisyon → bakiye → risk limiti
- Tek mantıksal emir birden çok hesaba **parça parça** dağıtılabilir (allocation engine)

### 4.3 Portföy Yönetim Motoru (yeni, en değerli parça)

- Tüm hesapları ortak baza normalize et (USD; TRY opsiyonel)
- **Net exposure**: aynı enstrüman 3 borsada açık → birleşik pozisyon
- Allocation hedefleri: `%40 Binance / %30 OANDA / %30 rezerv`
- Rebalans: hedeften sapma > eşik → otomatik düzeltme emri (risk gate'li)
- Birleşik PnL / drawdown / ücret raporu

### 4.4 Çapraz-Borsa Risk Motoru (risk-engine genişlemesi)

- Risk artık hesap değil **portföy bazında**: toplam marj, toplam exposure, korelasyon
- Risk bütçesi dağıtımı: her hesaba üst limit (portföyün %X'inden fazla kaybetmesin)
- **Dikkat**: aynı coin 2 borsada açık → HHI konsantrasyon şu an tek hesap varsayıyor; toplam üzerinden hesaba geçmeli

### 4.5 Veri & Operasyon

- Veri adapter'ları: her borsa için ayrı WS client (şu an sadece Binance)
- `data-engine`'e `exchange` kolonu (veri kaydı zaten merkezi — avantaj)
- Vault: anahtar rotasyonu + per-account izolasyon
- Audit: her emir hangi hesaba/hangi borsaya gitti — mevcut JSONL AuditLog'a `account_id` eklenir

---

## 5. Faz Planı (Önerilen Sıra)

| Faz | Kapsam | Çıktı |
|---|---|---|
| **Faz 1** | `Account` + `Exchange` trait + Router + mevcut Binance'i yeniden sarmalama | İki Binance hesabı bile çalışsın (risk-engine'e dokunmadan) |
| **Faz 2** | Portföy agregasyonu (net exposure + PnL) + `Portfolio` modülü | Birleşik portföy görünümü |
| **Faz 3** | Çapraz-borsa risk + allocation/rebalans + OANDA gibi 2. borsa | Gerçek çoklu borsa |
| **Faz 4** | Akıllı yönlendirme (en iyi fiyat/likidite) + raporlama/vergi | Kurumsal seviye |

**Kritik karar noktası:** Risk motoru hesap-bazlı mı tutulacak (bağımsız hesaplar) yoksa portföy-bazlı birleşik mi (tek sermaye havuzu)? İkincisi çok daha güçlü ama risk-engine'de gerçek yeniden yazım gerektirir.

---

## 6. İş Dağılımı Tahmini

| İş | Oran |
|---|---|
| `Exchange` trait + per-borsa impl | %40 |
| Signer soyutlaması | %15 |
| Veri WS adapter'ları | %20 |
| `exchange` alanı + router | %15 |
| Çapraz-borsa risk | %10 |
