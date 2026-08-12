# Kurumsal Düzeyde Kuantitatif Risk Yönetimi Çerçevesi (Institutional Quant Risk Framework)

> [!IMPORTANT]
> Kurumsal düzeyde (Hedge Fund, Prop House, Family Office) risk yönetimi, bireysel trader yaklaşımlarından tamamen farklıdır. Milyonlarca dolarlık teminatların, milisaniyelerle yarışan algoritmaların ve katı regülasyonların olduğu bu ekosistemde risk yönetimi; gecikme bütçesi (latency budget), likidite kısıtları, karşı taraf riskleri ve regülatif uyumun (compliance) milimetrik bir dengesidir.

---

## 1. Gecikme Bütçesi (Latency Budget) vs. Risk Hassasiyeti

Kuantitatif stratejilerde milisaniyeler (hatta mikrosaniyeler) kârlılığı belirler. Risk kontrollerinin getirdiği işlem gecikmesi (latency overhead) ile risk denetiminin hassasiyeti arasında kusursuz bir optimizasyon yapılmalıdır.

- **Sorun:** Yazılım tabanlı (CPU seviyesinde) yapılan detaylı pre-trade risk kontrolleri, işlem döngüsüne 10-50 mikrosaniye gecikme ekleyerek alfa üretimini (özellikle HFT ve Arbitraj stratejilerinde) öldürebilir.
- **Kurumsal Çözüm:** 
  - **Donanım Seviyesinde Filtreleme:** En kritik ve basit kontroller (Fat-finger, maksimum emir limiti, price collars) ağ kartı (**FPGA / SmartNIC**) seviyesinde, borsa hattı üzerinde doğrudan (in-line) ve nanosaniyeler içinde gerçekleştirilir.
  - **Asenkron Risk Hesaplama:** Portföy VaR, Expected Shortfall ve korelasyon matrisi güncellemeleri gibi yoğun matematiksel hesaplamalar ana işlem hattının (hot path) dışına çıkarılarak asenkron çalışan bir risk sunucusunda (risk daemon) yürütülür.

---

## 2. Likidite Riski ve Piyasa Etkisi (Market Impact / Slippage)

Büyük ölçekli fonlarda portföyün anlık teorik değeri (NAV), likidasyon anındaki gerçek değere eşit değildir. 

- **Sorun:** Panik veya acil durum anlarında büyük pozisyonları piyasa emirleriyle kapatmaya çalışmak, emir defterindeki (order book) derinliği tüketerek ciddi fiyat kaymalarına (**slippage**) yol açar ve zararı katlar.
- **Kurumsal Çözüm:**
  - **Almgren-Chriss Likidasyon Modeli:** Risk motoru, pozisyon büyüklüğünü piyasanın Ortalama Günlük Hacmi (ADV - Average Daily Volume) ile sürekli kıyaslamalıdır.
  - **Dinamik Yürütme (Optimal Execution):** Risk motoru, acil çıkış durumlarında piyasayı yıkmamak adına portföyü parçalara bölerek en optimum zaman dilimlerinde (VWAP, TWAP veya katılım oranlı algoritmalarla) çıkış stratejileri hesaplamalıdır.

---

## 3. Ekstrem Senaryolar ve Tail Risk (Stres Testleri)

Tarihsel verilere dayanan standart risk modelleri (tarihsel VaR vb.), piyasaların normal dağılıma yakın hareket ettiği dönemlerde çalışır. Ancak kriz anlarında bu modeller tamamen geçersiz kalır.

- **Sorun:** Finansal krizlerde normalde korelasyonu sıfıra yakın olan varlıklar hızla aynı yönde (genellikle aşağı) hareket etmeye başlar (Correlation Convergence).
- **Kurumsal Çözüm:**
  - **Sentetik Kriz Simülasyonları:** Risk motoru, portföyü her gün geçmişteki büyük kriz senaryolarına (2008 Lehman Çöküşü, 2020 COVID Paneli, 2022 LUNA/UST çöküşü) tabi tutmalıdır.
  - **Uç Değer Teorisi (Extreme Value Theory - EVT):** Kuyruk risklerini (tail risk) modellemek için standart normal dağılım yerine kalın kuyruklu (fat-tailed) dağılımlar (Gumbel, Frechet, Weibull) ve Copula modelleri kullanılmalıdır.

---

## 4. Çoklu Karşı Taraf ve Prime Brokerage Yönetimi

Kurumsal fonlar risklerini dağıtmak ve likiditeyi artırmak için birden fazla borsa, likidite sağlayıcı (LP) ve Prime Broker (PB) ile çalışır.

- **Sorun:** Farklı platformlardaki pozisyonlar birbirini risk açısından dengeliyor görünse de (örneğin A borsasında Long, B borsasında Short), borsalardan birinin çökmesi veya marjin çağrısı (margin call) yapması durumunda portföy zincirleme olarak likide olabilir.
- **Kurumsal Çözüm:**
  - **Merkezi Teminat Yönetimi (Cross-Margining):** Tüm borsalardaki marjin durumları, teminat oranları ve serbest nakit akışı merkezi risk motorunda tek bir portföy olarak konsolide edilmeli ve borsalar arası dinamik fon transferleri (rebalancing) otomatize edilmelidir.

---

## 5. Regülatif Güvenlik Duvarları ve Algoritmik Denetim (Compliance)

Küresel regülasyonlar (SEC Rule 15c3-5, MiFID II RTS 6 vb.), aracı kurumlara ve fonlara sıkı algoritmik denetim yükümlülükleri getirir.

- **Sorun:** Algoritmik bir hatanın (bug) döngüye girerek milisaniyeler içinde binlerce hatalı veya manipülatif emir (spoofing, layering) göndermesi, firmayı doğrudan iflasa veya lisans kaybına götürebilir (Knight Capital vakası).
- **Kurumsal Çözüm:**
  - **Emir Hızı ve Hacim Limitleri (Rate Limiting):** Risk motoru, milisaniye başına maksimum mesaj/emir sayısını denetlemeli ve bu eşik aşıldığında sistemi otomatik olarak askıya almalıdır.
  - **Fiyat Sınırları (Price Collars):** Piyasa fiyatından çok uzak limit emirleri veya anormal büyüklükteki piyasa emirleri borsa kapısına gitmeden önce risk motorunda bloke edilmelidir.
