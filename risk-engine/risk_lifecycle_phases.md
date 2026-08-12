# Kuantitatif Risk Yönetiminin 3 Temel Safhası (Pre-Trade, In-Trade, Scaling)

Kuantitatif sistemlerde risk yönetimi tek seferlik statik bir kontrol değildir; işlemin yaşam döngüsüne yayılmış üç aşamalı bir savunma hattıdır.

---

## 1. Pozisyon Açmadan Önce (Pre-Trade Risk Control)

**Amaç:** Portföyü kuyruk riskinden korumak ve sisteme toksik pozisyon almamak.

Emir borsaya gönderilmeden milisaniyeler önce risk motorunun yapması gereken temel kontroller:

- **Marjin ve Likidite Kontrolü:** Serbest marjinin yeterliliği doğrulanır. Hedef enstrümanın tahtasındaki (order book) anlık likidite analiz edilerek planlanan emir büyüklüğünün kaymaya (slippage) yol açıp açmayacağı hesaplanır.
- **Portföy Limitleri:** Bu pozisyon açıldığında portföyün toplam riski (VaR) ve kaldıraç oranı sistemsel limitleri aşıyor mu?
- **Korelasyon Analizi:** Yeni pozisyonun mevcut pozisyonlarla korelasyon matrisi üzerindeki etkisi incelenir. Yüksek korelasyon durumunda pozisyon büyüklüğü otomatik olarak traşlanır.
- **Pozisyon Büyüklüğü (Sizing):** Stop-loss mesafesine (ATR veya volatilite bazlı) göre kaç birim alınacağı bu aşamada milimetrik hesaplanır.

---

## 2. Pozisyon Anında (In-Trade / Active Risk Control)

**Amaç:** Pozisyon aktif olduktan sonra piyasa koşullarındaki dalgalanmaları ve sistem sağlığını gerçek zamanlı izlemek.

İşlem gerçekleştikten sonra risk motoru gerçek zamanlı (Real-Time Mark-to-Market) izlemeye geçer:

- **Dinamik Stop-Loss & Take-Profit:** Fiyat stop seviyesine geldiğinde emir borsaya anında iletilmelidir. Volatilite değiştikçe stop seviyeleri dinamik olarak güncellenir (Trailing Stop).
- **Maksimum Çekilme (Drawdown) İzleme:** Pozisyonun veya genel portföyün gerçekleşmemiş zararı (unrealized PnL) kritik seviyeye ulaştığında, **Acil Durum Kilidi (Kill Switch)** tetiklenir ve tüm açık pozisyonlar kapatılır.
- **Zaman Bazlı Durdurma (Time Stop):** Pozisyon beklenen sürede hedefine gitmediyse ve piyasa yatay seyrediyorsa, sermaye maliyeti (opportunity cost) ve taşıma maliyeti (funding rate) sebebiyle pozisyonun otomatik kapatılması gerekir.
- **Sistem Sağlığı ve Bağlantı Kontrolü:** Borsa API'leri veya veri akışındaki kesintilerde risk motoru "yetim pozisyonları" (orphan positions) güvenli bir şekilde kapatabilmelidir.

---

## 3. Pozisyona Ek İşlem Yaparken (Scaling-In / Modification Risk Control)

**Amaç:** Mevcut bir risk profilini modifiye ederken ortalama maliyet yanılgısına düşmemek ve riski kontrolsüzce büyütmemek.

Mevcut bir pozisyona ekleme yaparken veya azaltırken uygulanan katı kurallar:

- **Zarardaki Pozisyona Ekleme Yasağı (No Martingale):** Strateji özel olarak bir ızgara (grid) stratejisi değilse, zarardaki pozisyona ortalama düşürmek amacıyla ekleme yapılamaz. Ekleme (Pyramiding) sadece pozisyon kârdayken yapılmalıdır.
- **Ortalama Giriş Fiyatı ve Stop Yenileme:** Yeni ekleme yapıldığında ortalama giriş fiyatı (Average Entry Price) değişir. Risk motoru, yeni maliyete göre tüm pozisyonun ortak stop noktasını yukarı çekmelidir (breakeven veya kâra).
- **Marjinal Risk Katkısı (Marginal VaR):** Eklenen her birim, portföyün toplam riskini doğrusal olmayan (non-linear) bir şekilde artırabilir. Ekleme sonrası yeni VaR değeri limitleri aşmamalıdır.
