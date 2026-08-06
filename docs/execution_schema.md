# Algoritmik Şema: `lib.rs` ve `signer.rs` (Execution Engine)

Bu dosyalar sistemin "Silahı"dır. `core` katmanında Strateji motoru karar alıp tetiği çektiğinde, mermiyi (Emri) en hızlı ve en güvenli şekilde borsaya (Binance Order API) fırlatır. Klasik sistemlerin (Python/REST) aksine, emirler saniyeler süren HTTP/SSL el sıkışmalarını beklemek yerine önceden açık tutulan WebSocket tüneli üzerinden şifrelenip fırlatılır.

## Akış Şeması (Flowchart)

```mermaid
graph TD
    Start([Strateji Motoru Emir Verir<br/>OrderRequest])
    
    Start --> SendFlume[flume order_tx üzerinden<br/>Mesaj Yolla]
    
    SendFlume --> RecvEngine((Execution Motoru<br/>order_rx.recv))
    
    RecvEngine --> FetchKeys[Çevre Değişkenlerinden<br/>API_KEY ve SECRET_KEY Al]
    
    FetchKeys --> PreparePayload[Emir Parametrelerini<br/>JSON Formatında Hazırla<br/>symbol, side, type, qty, vb.]
    
    PreparePayload --> Signer[signer.rs Çağrılır]
    
    Signer --> HMAC[HMAC-SHA256 Algoritması ile<br/>Gizli Anahtar (SECRET_KEY) kullanılarak<br/>Payload İmzalanır]
    
    HMAC --> AttachSignature[İmza (Signature) Payload'a Eklenir]
    
    AttachSignature --> WSSend[Önceden Açık Tutulan<br/>ws-api.binance.com Tüneline<br/>Anında Fırlat]
    
    WSSend --> WaitResponse[Borsadan Yanıt Bekle]
    
    WaitResponse --> Success([Emir Gerçekleşti])
    WaitResponse --> Error([Hata: Likidasyon / Bakiye Yetersiz])
```

## Algoritmik Adımlar

1. **Bağlantı Kurulumu:** Sistem ilk açıldığında `execution-engine` hemen `ws-api.binance.com` adresine 443 portundan güvenli bir WebSocket açar ve bu tüneli kopartmadan sonsuza dek açık tutar (Keep-Alive).
2. **Kuyruk Dinleme:** Motor, stratejiden gelecek emirleri beklemek üzere tamamen bloke olmayan (non-blocking) bir kuyruğa (`order_rx`) kulak kabartır.
3. **Kriptografik İmza (Signing):** Emir geldiği mikrosaniyede `signer.rs` devreye girer. Binance'in güvenlik şartı olan HMAC-SHA256 imzası, sunucunun işlemcisi üzerinde `sha2` kütüphanesiyle oluşturulup paketin altına mühürlenir.
4. **Fırlatma (Dispatch):** İmzalanmış emir paketi, daha önce açık bırakılan WebSocket tünelinden 0 gecikmeyle (Zero TCP Handshake) borsaya iletilir. Bu sayede klasik botların yaşadığı "Emir geç iletildi, piyasa fiyatı kaçtı" sorunu %99 oranında engellenir.
