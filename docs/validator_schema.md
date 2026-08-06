# Algoritmik Şema: `validator.rs`

Bu dosya, HFT sistemine dışarıdan gelen (Borsa kaynaklı) tehlikeli, bozuk veya gecikmeli verilerin "Strateji Motoru"na ve "Veritabanına" sızmasını engelleyen kurumsal risk yönetimi (Data Validation & Circuit Breaker) katmanıdır.

## Akış Şeması (Flowchart)

```mermaid
graph TD
    Start([Veri Gelir: OwnedEvent])
    GetTime[Sistemin Anlık Zamanını Al: 'now']
    
    CheckReset{Son Şalter<br/>Sıfırlamasından<br/>1 Saniye Geçti mi?}
    Start --> GetTime --> CheckReset
    
    CheckReset -- Evet --> ResetTimer[bad_tick_count = 0<br/>Şalteri İndir (Safe)]
    CheckReset -- Hayır --> Route[Veri Tipini Oku]
    ResetTimer --> Route
    
    Route --> |Trade / Liquidation| CheckPrice{Fiyat <= 0 veya<br/>Miktar <= 0 mı?}
    Route --> |Orderbook / BookTicker| CheckCross{Alış Fiyatı >=<br/>Satış Fiyatı mı?}
    
    CheckPrice -- Evet --> FlagInvalid[flag_invalid çağır]
    CheckPrice -- Hayır --> CheckLatency{now - timestamp<br/>> 200ms mi?}
    
    CheckCross -- Evet --> FlagInvalid
    CheckCross -- Hayır --> CheckDrift
    
    CheckLatency -- Evet (Stale Data) --> FlagInvalid
    CheckLatency -- Hayır --> CheckDrift{timestamp > now<br/>(Gelecekten mi Geldi?)}
    
    CheckDrift -- Evet (NTP Drift) --> FlagInvalid
    CheckDrift -- Hayır --> Valid([Geçerli! RingBuffer'a İlet])
    
    FlagInvalid --> IncCount[bad_tick_count += 1]
    IncCount --> CheckCB{bad_tick_count > 100?}
    
    CheckCB -- Evet --> TriggerCB[CIRCUIT_BREAKER = TRUE<br/>Alım Satımı Durdur]
    CheckCB -- Hayır --> Drop([Veriyi Çöpe At (Continue)])
    TriggerCB --> Drop
```

## Algoritmik Adımlar

1. **Zaman Sıfırlama (Reset):** Her milisaniye çalıştığı için sürekli saate bakar. Eğer önceki hatalardan üzerinden 1 tam saniye geçmişse, sistem kendini "Güvenli (Safe)" ilan eder ve `bad_tick_count` sayacını sıfırlar. Şalter inikse kaldırır.
2. **Mantık Kontrolleri (Sanity Checks):**
   - Borsanın anlık gönderdiği veride fiyat eksi (negatif) veya sıfır olamaz.
   - Borsa Tahtasında (Orderbook) En İyi Alış (Bid), En İyi Satıştan (Ask) büyük olamaz. Olursa borsa çaprazlaşmıştır (Crossed Book), veri tehlikelidir.
3. **Gecikme (Latency) Kontrolleri:**
   - Verinin içindeki Zaman Damgası (Timestamp), bizim sunucumuzun o anki saatinden 200ms daha eskiyse veri "Bayat (Stale)" ilan edilir. HFT'de 200ms önceki veriyle ticaret yapılmaz.
4. **Şalter (Circuit Breaker):**
   - Reddedilen her veri için `bad_tick_count` 1 artar.
   - Eğer borsa çıldırır ve 1 saniye içinde 100'den fazla saçma sapan veri gönderirse, küresel Şalter (`circuit_breaker = true`) atomik olarak aktif edilir. Bu değişkeni okuyan Emir Motoru (Execution Engine) ateşi keser.
