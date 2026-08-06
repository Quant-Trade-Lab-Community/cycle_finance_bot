# Iron Fist 2.0 - Kapsamlı Kod Referansı (Code Reference)

Bu doküman, Demir Yumruk projesindeki her bir dosyanın içindeki yapıları (Struct/Sınıf), verileri (Enum) ve fonksiyonları (Function/Metod) satır satır, sınıf sınıf açıklamaktadır.

---

## 1. `core` Katmanı (Ana Motor)

### Dosya: `core/src/ring_buffer.rs`
HFT sisteminin kalbi olan **Sıfır-Tahsisli** bellek (memory) yönetim dosyası.

#### Veriler (Enums & Structs)
- **`enum EventType`**: Borsadan gelen verilerin tipini belirler. Alt veri tipleri şunlardır:
  - `Trade`: Fiyat (`price`), miktar (`quantity`) ve zaman (`timestamp`).
  - `Orderbook`: 20 kademeli alış (`bids`) ve satış (`asks`) defteri.
  - `Liquidation`: Patlayan işlemler. Yön (`side`), fiyat, miktar ve zaman tutar.
  - `FundingRate`: Fonlama oranı ve Mark Price verilerini tutar.
  - `BookTicker`: BBO (En iyi Alış/Satış) fiyatlarını anlık tutar.
  - `OpenInterest`: Açık pozisyon (OI) miktarlarını tutar.
- **`struct OwnedEvent`**: Sabit boyutlu (Fixed-Size) C-Struct'ı. İçerisinde hangi coine (`symbol`) ait olduğunu ve veriyi (`payload: EventType`) barındırır.
- **`struct RingBuffer`**: Önceden tahsis edilmiş diziyi (`buffer`), şu an yazılan dizini (`write_index`), toplam kapasiteyi (`capacity`) ve halkanın tam tur atıp atmadığını (`is_full`) tutar.

#### Metodlar (Fonksiyonlar)
- **`OwnedEvent::new_trade(..)`**: Hızlıca yeni bir Trade (İşlem) nesnesi yaratır. Sembolü 16 byte'lık diziye sıkıştırır. (Aynı mantıkla `new_orderbook`, `new_liquidation`, `new_funding_rate`, `new_bookticker`, `new_open_interest` metodları vardır).
- **`RingBuffer::new(capacity)`**: Sisteme ~100MB RAM ayırarak boş nesnelerle dolu devasa bir dizi (Array) oluşturur. HFT hızı için malloc (tahsis) işlemini en başta 1 kez yapar.
- **`RingBuffer::push(event)`**: `O(1)` hızında çalışır. Gelen yeni olayı (`event`), dizideki mevcut indexe doğrudan ezer. Eğer tur tamamlanmışsa, ezilen çok eski veriyi (`evicted`) geri döndürür.

---

### Dosya: `core/src/tick.rs`
JSON verilerini, işlemcinin SIMD (Tek Komut Çoklu Veri) birimlerini kullanarak işleyen donanım hızlandırmalı parser dosyası.

#### Sınıflar
- **`struct EventParser`**: Sadece statik metodlar barındıran yardımcı bir ayrıştırıcı sınıfıdır.

#### Fonksiyonlar
- **`EventParser::parse(bytes)`**: Ağdan gelen ham JSON byte dizisini alır. `simd_json` kullanarak ağacı oluşturur. JSON'ın içindeki `e` (Olay tipi) değerine bakar.
  - Eğer `trade` veya `aggTrade` ise -> `OwnedEvent::new_trade()` döndürür.
  - Eğer `depthUpdate` ise -> Bids ve Asks listelerini tarayıp `OwnedEvent::new_orderbook()` döndürür.
  - Eğer `forceOrder` ise -> Tasfiye edilen pozisyonu `new_liquidation()` olarak döndürür.
  - Eğer `markPriceUpdate` veya `bookTicker` ise ilgili constructor'ları çağırır.

---

### Dosya: `core/src/validator.rs`
Kurumsal veri güvenliğini (Data Validation & Circuit Breaker) sağlayan koruyucu dosya.

#### Veriler (Structs)
- **`struct DataValidator`**: Doğrulama işlemini yapan sınıftır.
  - `circuit_breaker`: Sistem genelindeki küresel şalter (AtomicBool).
  - `bad_tick_count`: Ardışık gelen hatalı veri sayısı (AtomicUsize).
  - `max_latency_ms`: Tolere edilebilir maksimum gecikme (Örn: 200 milisaniye).
  - `last_reset_time`: Hata sayacının en son ne zaman sıfırlandığı (Timestamp).

#### Fonksiyonlar
- **`DataValidator::new()`**: Şalteri kapalı, sayacı sıfır olarak başlatır.
- **`is_valid(event)`**: RingBuffer'a veri yazılmadan önce çağrılır. 
  - Gecikme kontrolü yapar. Eğer `event`'in zaman damgası şu anki saatten 200ms eskiyse veriyi "Bayat (Stale)" olarak işaretler.
  - Fiyat/Miktar anomalisi arar (Negatif fiyatları reddeder).
  - Tahtanın çaprazlaşıp çaprazlaşmadığını (Bid >= Ask) kontrol eder.
- **`flag_invalid(reason)`**: Veri hatalıysa sayacı (`bad_tick_count`) 1 artırır. Eğer 1 saniyede 100'den fazla bozuk veri gelirse Şalteri (`circuit_breaker = true`) havaya kaldırır.

---

### Dosya: `core/src/db.rs`
Gecikme (Latency) yaratmadan veritabanına veri yazan asenkron SQLite I/O modülü.

#### Fonksiyonlar
- **`start_db_writer(rx)`**: Ayrı bir iş parçacığında sonsuz bir döngüde çalışır.
  - `CREATE TABLE IF NOT EXISTS`: Başlangıçta Trades, Orderbooks, Liquidations, Funding Rates, Booktickers ve Open Interests tablolarını yaratır.
  - Veritabanını `WAL` (Write-Ahead Logging) modunda açarak okuyucuların kilitlenmesini engeller.
  - **Batching Döngüsü**: `rx.recv()` ile Lock-Free kanaldan (RingBuffer'dan ezilen) verileri toplar. Sayacı (`batch_count`) artırır. Eğer sayaç 10.000'i bulursa veya 1 saniye geçerse `BEGIN TRANSACTION` tetikler ve toplanan on binlerce SQL kaydını milisaniyeler içinde tek seferde yazar (`COMMIT`).

---

### Dosya: `core/src/main.rs`
Uygulamanın başlangıç noktası (Orkestratör).

#### Fonksiyonlar
- **`main()`**: 
  - `LockFreeDispatcher` ve `flume` kanallarını (tx, rx) başlatır.
  - `db_writer`'ı ayrı bir `thread::spawn` ile arka plana atar.
  - Open Interest (Açık Pozisyon) verilerini çeken `tokio::spawn` arka plan işçisini REST API'ye bağlar.
  - `execution-engine`'i arka plan thread'inde çalıştırır.
  - En büyük önceliğe (Priority 99) sahip olan ana Parser Thread'ini başlatır. Burada sonsuz döngüde gelen veriler `validator`'dan geçer, `ring_buffer`'a gömülür ve taşan veriler veritabanına gönderilir.
  - Son olarak `adapter` katmanını çağırarak Binance Websocket tünellerini açar.

---

## 2. `adapter` Katmanı (Veri Emişi)

### Dosya: `adapter/src/binance.rs`
Binance'den tüm verileri içeri taşıyan multiplexing dosyası.

#### Fonksiyonlar
- **`fetch_usdt_pairs()`**: Binance'in REST API'sine gidip işlem gören tüm USDT-M vadeli işlem sembollerinin isimlerini (Örn: BTCUSDT) liste olarak çeker.
- **`start_binance_ws_client(tx)`**: 
  - `fetch_usdt_pairs`'ı çağırıp tüm pariteleri alır.
  - Her bir parite için `@trade`, `@depth20@100ms`, `@forceOrder`, `@markPrice` ve `@bookTicker` stream dizilerini yaratır.
  - Binance'in 200 stream/bağlantı sınırını aşmak için, tüm bu streamleri `chunks(200)` mantığıyla paketlere böler.
  - Her paket için `tokio::spawn` açıp paralel bir `connect_async` başlatır.
  - Veri aktığında veriyi asla okumaz veya String'e çevirmez; anında `tx.send(bytes)` ile Lock-Free olarak ana motora yollar.

---

## 3. `execution-engine` Katmanı (Emir & Şifreleme)

### Dosya: `execution-engine/src/lib.rs`
Emirlerin borsaya fırlatıldığı WebSocket bağlantısı.

#### Veriler (Structs)
- **`struct OrderRequest`**: Stratejiden gelen emrin detaylarını tutar (`symbol`, `side` [Buy/Sell], `type` [Limit/Market], `quantity`, `price`).

#### Fonksiyonlar
- **`start_execution_engine(rx, api_key, secret_key)`**:
  - `ws-api.binance.com:443` adresine (Order API) asenkron bir websocket açar.
  - Strateji motorundan (veya şimdilik `main.rs`'ten) bir `OrderRequest` mesajı bekler.
  - Mesaj geldiğinde JSON formatında Binance emir formatını (`"method": "order.place"`) hazırlar.
  - Şifreleme modülüne (`signer`) JSON parametrelerini gönderir.
  - İmzalı (Signed) paketi WebSocket'ten borsaya ateşler ve cevabı bekler.

### Dosya: `execution-engine/src/signer.rs`
Binance'in güvenlik duvarı olan imzalamayı (Cryptography) üstlenen dosya.

#### Fonksiyonlar
- **`sign_payload(secret, payload_string)`**: 
  - Sistemin ortam değişkenlerinden (`.env`) aldığı `BINANCE_SECRET_KEY`'i kullanarak, gönderilecek parametre string'ini `HMAC-SHA256` algoritması ile şifreler.
  - Milisaniye altı (mikrosaniye) hızda çalıştığı için emrin gecikmesini önler.
  - Şifrelenmiş HEX dizgisini (signature) geri döndürür.
