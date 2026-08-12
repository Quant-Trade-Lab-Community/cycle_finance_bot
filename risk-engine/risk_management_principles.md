# Kuantitatif Risk Yönetimi İlkeleri (Quantitative Risk Management Principles)

> [!IMPORTANT]
> Finans piyasaları, ne kadar akıllı olduğunuzu umursamayan, duyguları olmayan ve en ufak zayıflığınızda sizi yutmaya hazır birer canavardır. Çoğu amatör "Nasıl kazanırım?" diye düşünürken, profesyonel kuantlar **"Nasıl hayatta kalırım?"** sorusuyla başlar. En mükemmel algoritma bile oyunda kalamazsa değeri sıfırdır. Risk yönetimi bir tercih değil; uzun vadeli matematiksel beklentinin (expected value) tek teminatıdır.

---

## 1. Pozisyon Büyüklüğü (Position Sizing) & Kelly Kriteri

Giriş sinyalinin (entry signal) kalitesi ne olursa olsun, yanlış pozisyon büyüklüğü iflasa yol açar. Risk motoru, her işlem için dinamik olarak en uygun pozisyon büyüklüğünü hesaplamalıdır.

### Kelly Kriteri
Teorik olarak optimal büyüklüğü belirlemek için aşağıdaki formül temel alınır:

\[f^* = \frac{bp - q}{b}\]

Burada:
- \(f^*\): Portföyün riske edilecek oranı
- \(b\): İşlemdeki kazanç/kayıp oranı (Odds)
- \(p\): Kazanma olasılığı (Win rate)
- \(q\): Kaybetme olasılığı (\(1 - p\))

> [!WARNING]
> Gerçek dünyada parametreler durağan (stationary) olmadığı için **asla tam Kelly kullanılmaz**. Risk motoru, aşırı kaldıraç riskini önlemek için **Fractional Kelly (Yarım veya Çeyrek Kelly)** uygulamalıdır.

---

## 2. Volatilite Tabanlı Risk Ayarlama (Volatility Targeting)

Sabit bir kontrat veya sabit bir fiat para miktarı ile risk yönetimi yapılamaz. Piyasa koşulları sürekli değişir.

- **Dinamik Stop Mesafesi:** Stop-loss seviyeleri, enstrümanın tarihsel veya anlık volatilitesine (**ATR - Average True Range** veya **GARCH modelleri**) göre belirlenmelidir.
- **Risk Katkısının Eşitlenmesi:** Yüksek volatilitede pozisyon küçültülür, düşük volatilitede pozisyon büyütülür. Amaç, her bir işlemin portföye getirdiği anlık risk katkısının (risk contribution) sabitlenmesidir.

---

## 3. Kovaryans ve Korelasyon Yönetimi (Correlation Matrix)

Portföydeki varlıkların çeşitlendirilmiş (diversified) olması bir illüzyon olabilir. 

- Korelasyon katsayıları kriz anlarında hızla \(1.0\)'e yakınsar. 
- Risk motoru, açık tüm pozisyonların anlık **Kovaryans Matrisini (Covariance Matrix)** hesaplamalıdır.
- Toplam portföy riski hesaplanırken **Value at Risk (VaR)** ve **Expected Shortfall (ES)** metrikleri, korelasyon katsayıları gözetilerek dinamik olarak güncellenmelidir.

---

## 4. Drawdown Kontrolü ve Acil Durum Kilitleri (Circuit Breakers)

Duygusal kararlara yer yoktur. Önceden belirlenmiş limitlere ulaşıldığında sistem otomatik olarak müdahale etmelidir.

- **Günlük/Haftalık Maksimum Kayıp Limitleri:** Portföy genelinde günlük veya haftalık belirlenen maksimum kayıp yüzdesine (örneğin %3) ulaşıldığında, risk motoru tüm açık pozisyonları piyasa emirleriyle kapatmalı ve sistem yeni işlem girişlerine kendisini kilitlemelidir.
- **Kademeli Risk Azaltma (Drawdown-Based Sizing):** Portföy tepe noktasından (Peak-to-Trough) aşağı indikçe (drawdown arttıkça), maksimum kaldıraç ve risk limitleri matematiksel bir fonksiyon olarak azaltılmalıdır.

---

## 5. Kaldıraç (Leverage) Kuralları

Kaldıraç, hataları katlayarak büyütür. Yönsel (directional) stratejilerde yüksek kaldıraç kullanımı matematiksel olarak iflas olasılığını (probability of ruin) artırır. Kaldıraç yalnızca yüksek Sharpe oranına sahip, düşük korelasyonlu arbitraj veya piyasa nötr (market-neutral) stratejilerde sıkı kontrollerle kullanılmalıdır.
