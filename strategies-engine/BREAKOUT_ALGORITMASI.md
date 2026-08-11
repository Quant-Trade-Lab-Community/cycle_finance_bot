# 🔥 KRİPTO FUTURES TEK ZAMAN DİLİMİ KIRILIM TESPİT ALGORİTMASI

**Sürüm 1.0 — Acımasız & Mükemmeliyetçi**

`breakout.rs` — spesifikasyon birebir uygulanmıştır.

---

## Veri Hattı (Her 1 saniye)

| Kaynak | Sağladığı Girdiler | Tazelenme |
|---|---|---|
| **Binance klines** (N=200 mum) | P_high, P_low, P_open, P_close, hacim → ATR(14), SMA(20), High14/Low14 | 10 sn |
| **detect-ms** (`:3002`) | R (en iyi SH), S (en iyi SL) + priority_score | 10 sn |
| **Flow ring'leri** (RAM — her saniye) | CVD (taker delta), OI/OI_prev, funding μ/σ(20), mark, last, Liq_current, Liq_avg | 1 sn |
| **Mum geçmişi** (otomatik) | T_cnt (0.5·ATR bantta seviye dokunuşu), V_touch_avg | 10 sn |

## Değişken Tanımları

| Sembol | Açıklama | Kaynak |
|---|---|---|
| P_high, P_low, P_open, P_close | Mevcut mum değerleri | Klines |
| σ | ATR(14) — volatilite | Klines → `indicators.rs` |
| V_avg | SMA(Volume, 20) | Klines → `indicators.rs` |
| R, S | En yakın direnç / destek | detect-ms SH/SL |
| T_cnt | Seviyeye geçmiş dokunuş sayısı (0.5·σ bant) | Mumlar → `compute_touches()` |
| V_touch_avg | Dokunuş anlarındaki ortalama hacim | Mumlar → `compute_touches()` |
| OI, OI_prev | Açık pozisyon (şimdi / önceki) | Open interest ring |
| F_rate | Funding rate | Funding ring |
| CVD_now, CVD_prev_10, CVD_sigma | Kümülatif hacim deltası (taker alım − satım) | Trade ring → `feed.rs` |
| Liq_current, Liq_avg | Mevcut / ortalama likidasyon | Liquidations ring |
| Mark, Last | Mark / son işlem fiyatı | Markprice / lastprice ring |

---

## Algoritma Adımları

### 1. Seviye Sağlamlık Skoru — S_level [0.0 – 1.0]

```
S_level = min(1, T_cnt/15) · 0.40
        + min(1, V_touch_avg / V_avg) · 0.40
        + min(1, 2σ / |R−S|+ε) · 0.20
```

- Çok dokunulmuş seviye → güvenilir
- Dokunuşta yüksek hacim → güçlü
- |R−S| dar (sıkışma) → kırılım potansiyeli yüksek
- ε = 1e-9 (sıfır bölme koruması)

### 2. Kırılım Tetikleyici (Acımasız Filtre)

```
P_close ≥ R + 0.25·σ  →  UP      (direnç kırıldı)
P_close ≤ S − 0.25·σ  →  DOWN    (destek kırıldı)
diğer                  →  NONE
```

Fiyat seviyeyi **0.25 ATR** net geçmedikçe kırılım **sayılmaz** — piyasa gürültüsü elenir.

### 3. Kırılım Kalitesi — Q [0% – 100%]

```
V_score  = min(1, Volume_current / V_avg)               // Hacim skoru (0.40)
M_score  = (UP)   (P_close − Low14) / (High14 − Low14)  // Momentum (0.35)
           (DOWN) (High14 − P_close) / (High14 − Low14)
Body     = |P_close − P_open| / (P_high − P_low)        // Gövde oranı (0.25)

Q = (V_score · 0.40 + M_score · 0.35 + Body · 0.25) × 100
```

Yüksek hacim + güçlü momentum + dolgun gövde = kaliteli kırılım.

### 4. Sahte Olasılığı — F [0% – 100%] *(düşük = iyi)*

```
W_score  = (UP)   (P_high − max(P_close,P_open)) / (P_high−P_low) · 2
           (DOWN) (min(P_close,P_open) − P_low) / (P_high−P_low) · 2
           // Uzun fitil = tuzak (0.30)

ΔOI_norm = (OI − OI_prev) / (OI_prev + ε)
OI_score = max(0, −ΔOI_norm)
           // Fiyat↑ ama OI↓ = short kapatma → sahte (0.30)

Z_funding = (F_rate − μ_20) / σ_20
FZ_score  = min(1, max(0, Z_funding / 3))
           // Aşırı funding = sahte riski (0.20)

Liq_score = min(1, Liq_current / Liq_avg)
           // Likidasyon spreyi (0.20)

F = (W · 0.30 + OI · 0.30 + FZ · 0.20 + Liq · 0.20) × 100
```

### 5. Kırılım Kesinliği — C [0% – 100%]

```
CVD_score = min(1, max(0, (CVD_now − CVD_prev_10) / (σ_cvd · 10)))
            // Gerçek taker akışı (0.40)

MP_score  = 1.0  (UP ∧ Mark > Last → Contango)
            1.0  (DOWN ∧ Mark < Last → Backwardation)
            0.5  (diğer)
            // Mark-fiyat uyumu (0.20)

C = (S_level · 0.40 + CVD_score · 0.40 + MP_score · 0.20) × 100
```

---

## Acımasız Kurallar *(override — Bölüm 5–6)*

| Kural | Koşul | Sonuç |
|---|---|---|
| **Likidasyon avı** | Liq_current > 5 × Liq_avg | direction → **NONE** (stop-hunt, trend değil) |
| **Aşırı funding** | Z_funding > 3 | certainty **≤ %30** (pozisyon alınmaz) |
| **Fitil tuzağı** | Wick seviyeyi deldi ama kapanış eşik altında | Fake'ye **+%15** sabit ceza |

---

## Çıktı (Her saniye — JSON)

```json
{
  "direction":           "UP" / "DOWN" / "NONE",
  "broken_level":        0.7156,
  "breakout_quality":    72.7,
  "fake_percentage":     22.1,
  "certainty_percentage": 40.7
}
```

- `UP` → 📈 **BUY** sinyali
- `DOWN` → 📉 **SELL** sinyali
- `NONE` → nötr (fiyat aralıkta veya likidasyon avı)

**Not:** Emir AÇMAZ — sadece sembol + yön + Q/F/C bilgisini üretir.

---

## Dosya Yapısı

```
services-engine/strategies/breakout-strategy/src/
├── breakout.rs      ← Bu algoritmanın çekirdek motoru (pure math)
├── indicators.rs    ← ATR(14), SMA(20), High/Low(14) hesaplayıcıları
├── feed.rs          ← Flow ring'lerinden türev veri toplayıcı (CVD, OI, funding, liq, mark, last)
├── main.rs          ← Veri hattı: klines + detect-ms + flow → compute → JSON çıktı
├── metrics.rs       ← Mikro-yapı metrikleri (TPS, aVPIN, EfP, ...)
├── lib.rs           ← Modül ihracı
└── BREAKOUT_ALGORITMASI.md  ← Bu doküman
```

## Çalıştırma

```bash
cd ~/Desktop/PROJE
# Release (önerilen):
./target/release/breakout-strategy

# Tek seferlik test:
./target/release/breakout-strategy --once

# Farklı sembol:
BREAKOUT_SYMBOL=VELVETUSDT ./target/release/breakout-strategy

# Canlı izleme:
tail -f /tmp/breakout-strategy.log
```
