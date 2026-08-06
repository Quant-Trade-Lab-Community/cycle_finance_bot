# Algoritmik Şema: `binance.rs` (Adapter Layer)

Bu dosya sistemin dış dünyaya (Binance Vadeli İşlemler) açılan ağzıdır. Milyonlarca veriyi darboğaz (bottleneck) oluşturmadan sisteme çeker. Binance'in bir WebSocket bağlantısı için koyduğu 200 abonelik (subscription) sınırını "Chunking" algoritmasıyla aşarak 1.250 aboneliği 7 kanaldan eşzamanlı çeker.

## Akış Şeması (Flowchart)

```mermaid
graph TD
    Start([Adapter Başlatılır])
    GetPairs[Binance REST API'den<br/>Tüm USDT Paritelerini Çek (Örn: 250 adet)]
    
    Start --> GetPairs
    
    GetPairs --> BuildStreams[Her Parite İçin 5 Stream Metni Oluştur<br/>trade, depth20, forceOrder, vs.]
    
    BuildStreams --> Chunking[Toplam 1250 Stream'i<br/>200'lük Paketlere (Chunk) Böl]
    
    Chunking --> SpawnLoops{Her Paket İçin<br/>Ayrı Asenkron Görev<br/>Tokio Spawn}
    
    SpawnLoops --> WS1[WebSocket 1<br/>1-200 Stream]
    SpawnLoops --> WS2[WebSocket 2<br/>201-400 Stream]
    SpawnLoops --> WSX[WebSocket 7<br/>1201-1250 Stream]
    
    WS1 --> RecvLoop((Sonsuz Okuma Döngüsü))
    WS2 --> RecvLoop
    WSX --> RecvLoop
    
    RecvLoop --> Receive[Ağdan Ham Byte Array Gelir]
    Receive --> SendFlume[String'e Çevirmeden Doğrudan<br/>Flume Kuyruğuna tx.send Fırlat]
    
    SendFlume --> Return((Döngüye Geri Dön))
```

## Algoritmik Adımlar

1. **Keşif (Discovery):** Önce HTTP üzerinden `fapi/v1/exchangeInfo` uç noktasına gidilir ve piyasada işlem gören tüm `USDT` çiftleri bulunur.
2. **Kombinasyon (Combination):** Bulunan her sembol için (Örn: `btcusdt`) 5 farklı veri kanalı (`@trade`, `@depth20@100ms`, `@forceOrder`, `@markPrice`, `@bookTicker`) birleştirilerek dev bir istek (Request) listesi oluşturulur.
3. **Chunking (Böl-Parçala):** Binance'in katı limitlerine (200 stream/connection) takılmamak için bu dev liste 200'erli paketlere bölünür.
4. **Çoklu Bağlantı (Multiplexing):** Elde edilen her paket (Chunk) için izole bir `tokio::spawn` iş parçacığı yaratılır ve aynı anda borsaya 7 farklı tünel (WebSocket) açılır.
5. **Veri İletimi (Zero-Copy):** Ağdan okunan veri asla Rust `String` tipine çevrilmez, JSON parse edilmez. Doğrudan ham byte array (`Vec<u8>`) olarak `Lock-Free` flume kuyruğuna atılarak ana parse motoruna (Core) bırakılır. Böylece ağ bağlantısı hiçbir zaman veri yoğunluğundan şişip kopmaz.
