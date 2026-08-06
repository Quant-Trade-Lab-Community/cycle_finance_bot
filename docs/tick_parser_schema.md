# Algoritmik Şema: `tick.rs` (SIMD-JSON Parser)

Bu dosya, Adapter'dan gelen ham (raw) Byte yığınlarını (JSON) saniyenin milyonda biri (Nanosaniye/Mikrosaniye) hızında CPU üzerinde donanımsal olarak parse edip `OwnedEvent` isimli C-Struct yapımıza çevirir. Yavaş olan standart kütüphaneler yerine Intel/AMD işlemcilerdeki "SIMD (Single Instruction, Multiple Data)" komut setini kullanır.

## Akış Şeması (Flowchart)

```mermaid
graph TD
    Start([Raw Bytes Gelir: Vec_u8])
    ParseSIMD[simd_json::to_borrowed_value<br/>CPU SIMD Komutları ile Parse]
    
    Start --> ParseSIMD
    
    ParseSIMD -- Hatalı JSON --> Drop([Veriyi At: None])
    ParseSIMD -- Başarılı --> ReadEvent{JSON İçindeki<br/>'e' alanını (Event Type) oku}
    
    ReadEvent --> |trade / aggTrade| ParseTrade[Sembol, Fiyat, Miktar Oku<br/>new_trade()]
    ReadEvent --> |depthUpdate| ParseDepth[20 Kademe Bids & Asks Oku<br/>new_orderbook()]
    ReadEvent --> |forceOrder| ParseForce[o objesi içindeki<br/>Fiyat, Miktar Oku<br/>new_liquidation()]
    ReadEvent --> |markPriceUpdate| ParseMark[Mark Price, Funding Rate Oku<br/>new_funding_rate()]
    ReadEvent --> |bookTicker| ParseBook[Bid, Ask Fiyat ve Miktarları Oku<br/>new_bookticker()]
    
    ParseTrade --> BuildStruct
    ParseDepth --> BuildStruct
    ParseForce --> BuildStruct
    ParseMark --> BuildStruct
    ParseBook --> BuildStruct
    
    BuildStruct([OwnedEvent Oluştur<br/>Return Some])
```

## Algoritmik Adımlar

1. **Donanımsal Hızlandırma:** Veri geldiği anda klasik karakter-karakter okuma yerine, SIMD sayesinde işlemci tek bir saat döngüsünde (clock cycle) birden fazla karakteri okuyup parse eder.
2. **Dalga Yönlendirme (Routing):** Ayrıştırılan JSON ağacı içerisindeki `e` (Event Type) alanına bakılır. Bu, verinin bir Trade mi, Orderbook mu yoksa Likidasyon mu olduğunu belirler.
3. **Seçici Kopyalama:** Devasa JSON metninin tamamı RAM'de tutulmaz. Sadece ihtiyacımız olan (fiyat, miktar, zaman) matematiksel alanlar (f64) olarak kopyalanır ve geri kalanı atılır.
4. **Sıfır Memory Leak:** Veri, Rust'ın Heap (Dinamik Bellek) yönetimine bırakılmaz. Boyutu statik olarak derleme zamanında belli olan (Fixed Size) `OwnedEvent` nesnesine çevrilir. Bu da Garbage Collection (Çöp Toplayıcı) gecikmelerini (Pause) tarihe gömer.
