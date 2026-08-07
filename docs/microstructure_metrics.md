# 📈 Microstructure Metrics — Anlam ve Kullanım Rehberi

Bu doküman, Listener katmanının ürettiği mikro-yapı (market microstructure) metriklerinin **ne anlama geldiğini** ve **nasıl yorumlanacağını** açıklar.

Veri kaynağı: **DATA merkezi** (`/dev/shm/demir_yumruk_ring` → core `RUN_MODE=DATA`)
Üretim yeri: `heiusdt/src/metrics.rs` → `./target/debug/listener`
Çıktılar: konsol tablosu + `/tmp/listener_metrics.json`

---

## 🎯 Metrikler ne için?

Bu metrikler, bir piyasanın **mikro-yapısını** (emir akışı, likidite mimarisi, agresif/pasif davranış, toksisite) ölçer. Amaç: ham fiyat akışındaki **"parayı bilen" (informed)** trader'ların izini sürmek ve fiyat keşfinin yönünü tahmin etmek.

> ⚠️ **Kalibrasyon uyarısı:** Tüm eşikler (`λ`, `θ_vol`, `α`, `γ`, `K`) sembol bazında yeniden kalibre edilmelidir. Sabit parametrelerle uzun süre canlı çalıştırmak kayıp üretir.

---

## 📋 Metrik Tablosu

### SEMBOL
İşlem gören parite (BTCUSDT, ETHUSDT, SOLUSDT, HEIUSDT — `alerts.toml`'dan otomatik).

### WLOBI — Weighted Limit Order Book Imbalance
```
WLOBI = (Σωᵢ·Vᵢ_ask − Σωᵢ·Vᵢ_bid) / (Σωᵢ·Vᵢ_ask + Σωᵢ·Vᵢ_bid)
ωᵢ = e^(−λ·i)   (i = kademe derinliği)
```
**Anlamı:** Emir defterindeki likidite dengesizliği. İlk 5 kademe, derinliğe göre üstel ağırlıklandırılır (yüzeye yakın kademeler daha önemli).

| Değer | Yorum |
|-------|-------|
| **+0.5 … +1.0** | Ask tarafı (satış) baskın → fiyat üzerinde aşağı baskı olabilir |
| **−0.5 … −1.0** | Bid tarafı (alım) baskın → fiyat üzerinde yukarı baskı olabilir |
| **≈ 0** | Dengeli defter, yönsel sinyal zayıf |

### SLP_ASK / SLP_BID — Quote Slope (Likidite Eğimi)
```
Slope_ask = (ln V1_ask − ln V5_ask) / (P5_ask − P1_ask)
```
**Anlamı:** En iyi fiyattan derinliğe gidildikçe hacmin logaritmik olarak ne kadar hızlı azaldığı. Defterin "dikliği".

| Değer | Yorum |
|-------|-------|
| **Büyük pozitif** | Hacim yüzeye yoğunlaşmış → likidite yüzeysel, kırılmaya açık |
| **Küçük/negatif** | Hacim derine yayılmış → güçlü destek/direnç |
| **1.5× artış (önceki slop'a göre)** | Piyasa yapıcılar riskten kaçıyor → volatilite leading göstergesi |

### EFFΔ — Effective Delta
```
EffDelta = Σ Sₖ · Vₖ_agg · (sₖ_eff / s̄)
sₖ_eff = 2·|Pₖ − midₖ|   (efektif spread)
```
**Anlamı:** Agresif (market) emirlerin **net yönlü basıncı**, efektif spread ile normalize edilmiş. Pozitif → alım baskısı, negatif → satış baskısı.

| Değer | Yorum |
|-------|-------|
| **Büyük pozitif** | Agresif alımlar baskın |
| **Büyük negatif** | Agresif satışlar baskın |
| **≈ 0** | Denge, sinyal yok |

### ΔV — Delta Velocity
```
ΔV = (EffDelta_t − EffDelta_{t−Δt}) / Δt   (Δt = 1 sn)
```
**Anlamı:** EffDelta'nın değişim hızı. Akışın ne kadar hızlı ivmelendiği.

| Değer | Yorum |
|-------|-------|
| **|ΔV| > θ_vol · σ(EffDelta_60s)** | Aşırı ivmelenme → tükenme (mean-reversion) sinyali |
| **Sakin |ΔV|** | Akış istikrarlı, trend devam edebilir |

### ABS — Absorption Ratio
```
Abs = Σ pasif alım hacmi / Σ agresif satış hacmi   (son K=100 trade)
```
**Anlamı:** Agresif akışın pasif emirler tarafından ne kadar **emildiği**. Güçlü emilim = büyük oyuncuların gizlice pozisyon biriktirmesi.

| Değer | Yorum |
|-------|-------|
| **> 1.0** | Alım tarafı emilim baskın → gizli birikim olasılığı |
| **< 1.0** | Satış tarafı agresif → pasif emilim zayıf |
| **Sürekli yüksek** | Kurumsal (iceberg) pozisyon alma sinyali |

### IDM — Iceberg Detection Metric
```
IDM = E[C(θ)] / (Var(∂C/∂θ) + ε)
```
**Anlamı:** Bir fiyat seviyesinde kümülatif dolumun, dolum hızı **sabit** kalırken büyük olması = gizli (iceberg) emir varlığı.

| Değer | Yorum |
|-------|-------|
| **> Eşik (1.2)** | O seviyede gizli pasif emir var; yönü trade işaretiyle (S_t) belirlenir |

### aVPIN — Adaptive Volume-Synchronized Probability of Informed Trading
```
σ_parkinson = √(1/(4·ln2)) · √(Σ ln²(H/L)/N)
B_vol(t) = α · σ_parkinson · V̄(son 1000 trade)
aVPIN = Σ|V_buy − V_sell| / (n · B_vol)
```
**Anlamı:** Piyasada **bilgili (informed)** trader'ların baskınlığı. Hacim-bucket'lı toksisite ölçüsü. Bucket boyutu Parkinson volatilitesine göre **dinamik** ayarlanır.

| Değer | Yorum |
|-------|-------|
| **> 0.7** | Informed trader baskın → toksik akış, trend güçlü |
| **0.5 – 0.7** | Orta toksisite, dikkatli ol |
| **< 0.4** | Nispeten "temiz" akış, mean-reversion daha olası |
| **≥ 0.6 (karar kuralı)** | Sinyal **nötrleştirilir** — toksik akışta pozisyon açma |

### PERM — Permanent Impact (Kalıcı Etki)
```
Perm = α₁ / (1 − α₂)   (Hasbrouck VAR: r_t = α₁·x_t + α₂·r_{t−1} + ε)
```
**Anlamı:** Agresif akışın fiyat üzerindeki **kalıcı** etkisi. Yüksek kalıcı etki = emir akışı gerçek bilgi taşıyor (fiyat keşfi).

| Değer | Yorum |
|-------|-------|
| **Perm > 2 × Temp** | Fiyat keşfi (trend) — akış bilgilendirici |
| **Perm < 2 × Temp** | Geçici envanter dengesizliği (mean-reversion) |

### EfP — Execution Footprint
```
EfP = agresif trade hacmi / (ilk 5 kademe toplam derinlik)
```
**Anlamı:** Tek bir agresif emrin piyasa derinliğine oranı. "Büyük oyuncu zorlaması" tespiti.

| Değer | Yorum |
|-------|-------|
| **> 0.05 (%5)** | Trade "büyük oyuncu zorlaması" — modellerden **çıkarılır** (outlier) |
| **< 0.05** | Normal büyüklükte emir |

### P(LONG) — Long Olasılığı
```
P(Long) = 1 / (1 + e^(−A))
A = γ₀ + γ₁(Abs−1) + γ₂(−WLOBI) + γ₃(0.7−aVPIN) + γ₄·sign(−EffDelta)·1{|ΔV|<θ} − γ₅·Perm
```
**Anlamı:** Alpha Basket'in lojistik çıktısı — yukarı yönlü hareket olasılığı (0–1).

### SİNYAL — Nihai Karar
| P(Long) | aVPIN | Sinyal |
|---------|-------|--------|
| > 0.65 | < 0.6 | **▲ LONG** |
| < 0.35 | < 0.6 | **▼ SHORT** |
| herhangi | ≥ 0.6 | **· NÖTR** (toksik akışta pasif kal) |

---

## 🧠 Birlikte Yorumlama (Örnek Senaryolar)

### Güçlü LONG senaryosu
- WLOBI negatif (bid tarafı derin)
- ABS > 1.0 (pasif emilim)
- EffDelta pozitif ve ΔV sakin
- aVPIN < 0.4
- PERM > Temp (fiyat keşfi)

### Tükenme (mean-reversion) senaryosu
- |ΔV| çok yüksek (aşırı ivmelenme)
- EfP > 0.05 (büyük oyuncu zorlaması)
- PERM < Temp

### Toksik akış (kaçınma)
- aVPIN > 0.6 → sinyal nötr, pozisyon açılmaz

---

## 🔧 Parametreler (Θ — sembol bazında kalibre edilmeli)

| Parametre | Varsayılan | Anlamı |
|-----------|-----------|--------|
| `λ` (LAMBDA) | 0.015 | WLOBI decay — kademe derinliği ağırlığı |
| `θ_vol` (THETA_VOL) | 2.5 | Delta velocity tükenme eşiği |
| `α` (ALPHA_BUCKET) | 0.75 | aVPIN bucket boyutu sabiti |
| `K` (K_ABS) | 100 | Absorption penceresi (trade) |
| `n` (N_BUCKET) | 50 | aVPIN bucket sayısı |
| `γ₀..γ₅` | [0, 0.4, −0.3, 0.5, 0.6, −0.35] | Alpha Basket ağırlıkları |

> **Kurumsal not:** Θ her sabah `min Σ(signal−P&L)², Sharpe > 2.0` optimizasyonuyla çözülür; `B_vol` her 5 dk'da Bayesian Online Changepoint Detection (BOCD) ile güncellenir.

---

## 📊 JSON Çıktısı (`/tmp/listener_metrics.json`)

```json
{
  "timestamp": "06:17:50",
  "metrics": {
    "BTCUSDT": {
      "wlobi": 0.848,
      "slope_ask": 10.27,
      "eff_delta": -2.89,
      "delta_velocity": -0.07,
      "absorption": 1.246,
      "avpin": 505.56,
      "permanent_impact": 0.686,
      "temporary_impact": 0.0,
      "efp": 0.0,
      "alpha_score": 0.0,
      "p_long": 0.5,
      "signal": 0
    }
  }
}
```

---

## 🔗 İlgili Dokümanlar
- [ring_buffer_schema.md](./ring_buffer_schema.md) — veri akışı (DATA merkezi)
- [tick_parser_schema.md](./tick_parser_schema.md) — stream tipleri (trade/depth/bookTicker)
- [complete_system_documentation.md](./complete_system_documentation.md) — genel mimari
