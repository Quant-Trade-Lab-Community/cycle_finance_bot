# 🔧 BREAKOUT ALGORİTMASI — İYİLEŞTİRME ÖNERİLERİ

**İnceleme tarihi:** 2026-08-12
**Kapsam:** `strategies-engine/src/breakout.rs`, `main.rs`, `feed.rs`, `indicators.rs`

Öneriler en yüksek etkiden düşüğe doğru sıralanmıştır.

---

## 1. Gerçek Doğruluk Hataları (Öncelikli)

### 1.1 OI skoru yön-kör (bug)

`breakout.rs:172-174` — `oi_score = max(0, −ΔOI_norm)` her yön için aynı formülü kullanıyor.

- **UP kırılım:** OI düşüşü = short kapatma → sahte (doğru).
- **DOWN kırılım:** OI düşüşü = long'ların çıkışı → **destekleyici**, sahte DEĞİL.

Oysa mevcut formül DOWN'da da OI düşüşünü sahte olarak cezalandırıyor. OI skoru yöne göre asimetrik işlenmeli:

```
UP   : OI_score = max(0, −ΔOI_norm)   // short kapatma → sahte
DOWN : OI_score = max(0,  ΔOI_norm)   // long girişi devam ediyorsa sahte riski
```

### 1.2 T_cnt iki seviyeyi birden sayıyor

`main.rs:92-109` — `compute_touches` R **ve** S dokunuşlarını tek `count` içinde topluyor. `level_strength` (`breakout.rs:111-121`) her iki seviye için aynı şişkin sayıyı kullanıyor.

**Çözüm:** R ve S için ayrı `T_cnt` ve `V_touch_avg` hesaplanmalı; `BreakoutInput`'a ayrı alanlar eklenmeli.

### 1.3 Fitil dokunuşu T_cnt'yi şişiriyor

`main.rs:100-101` — kapanış seviyeye uzakken uzun fitil seviyeye değse bile dokunuş sayılıyor.

**Çözüm:** Dokunuş koşulu "mum **kapanışı** seviyeye 0.5σ bant içinde" olarak sıkılaştırılmalı. Fitil dokunuşu ayrı bir "wick touch" sayaçı olarak tutulmalı (fitil tuzağı skoruna ham madde).

### 1.4 Momentum skoru seviye yerine 14'lük high/low kullanıyor

`breakout.rs:155-157` — kırılan seviye `high_14`'e yakınsa marjinal bir kırılım bile `m_score ≈ 1` alır.

**Çözüm:** Momentum seviye-mesafesine göre hesaplanmalı:

```
M_score = (UP)   (P_close − R) / ATR      // seviyeden ne kadar kopuldu
          (DOWN) (S − P_close) / ATR
```

### 1.5 `liq_run` `broken_level`'i sıfırlıyor

`breakout.rs:229-230` — likidasyon avında yön NONE'a çevrilirken `broken_level`, `quality`, `fake`, `certainty` bilgisi kayboluyor.

**Çözüm:** Sadece `direction` NONE olsun; `broken_level` ve skorlar korunmalı. Böylece üst katmanlar "hangi seviye avlandı" bilgisini izleyebilir.

---

## 2. Algoritmik İyileştirmeler

### 2.1 Tek mum onayı yok

Şu an tek kapanış eşiği aştı diye sinyal veriliyor (`breakout.rs:125-133`). Tek mum gürültüye açık.

**Çözüm:** Onaylama:
- İkinci mum kapanışı da aynı tarafta kalsın, **ve**
- İki mumun birleşik hacmi `V_avg` üzerinde olsun.
- 1–2 mum gecikme kabul edilerek sahte sinyal oranı ciddi düşer.

### 2.2 Adaptif eşik

`0.25 × ATR` sabit (`breakout.rs:126`). Yüksek volatilite rejiminde çok küçük kalır (gürültüyü elemeyi başaramaz), düşük volde aşırı büyür (sinyal gecikir).

**Çözüm:** Eşiği ATR'nin yüzdelik dilimiyle ölçekle:

```
threshold = 0.25 × ATR × (1 + regime_factor)
regime_factor = (ATR − ATR_median) / ATR_median   // 0.5 katsayıyla sınırla
```

### 2.3 Yön-koşullu funding cap

`breakout.rs:203-207` — `Z_funding > 3` cap'i yalnızca pozitif (aşırı uzun) tarafı yakalar. DOWN kırılımda tehlikeli uç `Z < −3`'tür (aşırı short sıkışıklığı).

**Çözüm:**

```
UP   : Z > +3 → certainty ≤ %30
DOWN : Z < −3 → certainty ≤ %30
```

### 2.4 Kırılım sonrası retest / geri çekilme

Kırılan seviye yeni destek/direnç olur. Pullback onayı (geri çekilip seviyeden dönme) giriş kalitesini artırır.

`main.rs:234`'teki `last_signal_direction` guard'ı aynı yönde yeniden girişe izin vermiyor.

**Çözüm:** Retest paterni izleyen bir durum makinesi: `BO → Pullback → Confirm → signal`. Pullback'te seviye üzerinde tutunma (close ≥ R) ile yeniden sinyal.

### 2.5 Multi-timeframe trend filtresi

UP kırılım yalnızca daha yüksek TF trendi yukarıyken alınmalı (ve tersi). Örn. `1m` sinyali için `15m` EMA(50) yönü filtre olarak kullanılabilir.

---

## 3. Risk / Bağlam Filtreleri

### 3.1 Volatilite rejimi + seans filtresi

- Düşük hacimli dilimlerde (gece seansı, düşük likidite saatleri) sinyalleri bastır veya kesinliği aşağı çek.
- `volume_current` ile seansın tipik hacmi karşılaştırılabilir.

### 3.2 Kesinlik eşiği (sinyal filtresi)

Şu an her kırılımda sinyal yazılıyor; `certainty` ve `fake` skorları **filtre olarak kullanılmıyor** (`main.rs:234`).

**Çözüm:**

```
sinyal koşulu: direction ≠ NONE
               ∧ certainty ≥ 60
               ∧ fake ≤ 40
```

Eşikler `ai.toml` / env üzerinden ayarlanabilir hale getirilmeli.

---

## 4. Altyapı (En Büyük Kazanç)

### 4.1 Backtest / parametre optimizasyonu yok

Tüm ağırlıklar hard-coded:
- `Q` ağırlıkları: 0.40 / 0.35 / 0.25
- `F` ağırlıkları: 0.30 / 0.30 / 0.20 / 0.20
- `C` ağırlıkları: 0.40 / 0.40 / 0.20
- Kırılım eşiği: 0.25σ

`data-engine/data/market_data.db`'de geçmiş veri mevcut. **Objektif bir ölçü olmadan** algoritmayı "iyileştirmek" mümkün değil.

**Öneri:**
1. `market_data.db` üzerinden walk-forward backtest harness'ı kur.
2. Metrikler: kazanma oranı, ortalama kâr/kayıp, maksimum düşüş, sahte sinyal oranı.
3. Ağırlık ve eşikleri grid/random search ile optimize et.
4. Out-of-sample (görülmemiş dilim) doğrulaması zorunlu.

### 4.2 Birim testler zayıf

- `wick_fake_penalty` testi hiçbir şeyi assert etmiyor (`breakout.rs:308-316`) — sadece yönün NONE olduğunu kontrol ediyor, `+15` cezasını doğrulamıyor.
- DOWN senaryosu için test yok.

**Öneri:** Eklenmesi gereken testler:
- DOWN kırılım algılama
- OI asimetrisi (DOWN'da OI düşüşü sahte puanını **artırmamalı**)
- R/S dokunuşlarının ayrı sayılması
- Adaptif eşik davranışı
- Kesinlik eşiği filtresi

---

## Uygulama Önceliği

| Adım | İçerik | Etki | Efor |
|---|---|---|---|
| 1 | Bug düzeltmeleri (§1) | Yüksek | Düşük |
| 2 | Tek-mum onayı + adaptif eşik (§2.1, §2.2) | Yüksek | Orta |
| 3 | Kesinlik filtresi + yön-koşullu funding (§3.2, §2.3) | Orta | Düşük |
| 4 | Retest durum makinesi (§2.4) | Orta | Orta |
| 5 | Backtest altyapısı (§4.1) | En yüksek | Yüksek |
