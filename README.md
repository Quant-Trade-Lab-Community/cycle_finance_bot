# Demir Yumruk v3.0 - Titanyum Çekirdek (Titanium Core) & Sıfır Kopya Gözlem Mimarisi

Demir Yumruk v3.0, dünyanın en acımasız ve yüksek frekanslı kripto ticaret sistemlerine kafa tutmak için sıfırdan "Bare-Metal" (Donanıma yakın) felsefesiyle yazılmış kurumsal bir HFT (High-Frequency Trading) motorudur.

Sistem, işletim sisteminin hantallıklarını, kilitleme (lock) mekanizmalarının darboğazlarını ve çöp toplayıcı (GC) / bellek tahsisi (heap allocation) gecikmelerini tamamen ortadan kaldıran kusursuz bir mimariye sahiptir.

## 🚀 Temel Felsefe: "Zero-Copy, Zero-Block, Zero-Compromise"
- **Hot Path Dokunulmazdır:** Emir defterini okuyan ve karar alan strateji döngüsü asla bloke edilemez (Non-Blocking). Veri çekme veya arayüze veri gönderme işlemleri için dahi 1 nanosaniye feda edilmez.
- **Kilitsiz İletişim (Lock-Free):** Thread'ler arası iletişim sadece `std::sync::atomic` ve Memory Fences (Acquire/Release) üzerinden sağlanır. `Arc<Mutex<T>>` gibi ilkel kilitlemeler sistemden tamamen temizlenmiştir.
- **Bellek Tahsisi Yasaktır:** Sıcak döngü (hot loop) içinde `Box`, `Vec` veya `String` tahsisi (Heap allocation) yapılamaz. Bellek önceden devasa bir blok halinde İşletim Sisteminden mmap / pre-fault ile tahsis edilir.

---

## 🏛️ Mimari Katmanlar

### 1. Titanyum Çekirdek (Titanium Orchestrator)
Sistemin kalbidir. Gelen piyasa verilerini stratejilere besler ve üretilen emirleri dış dünyaya (Execution Engine) iletir.
- **Core Pinning:** Çekirdek, `core_affinity` kullanılarak işletim sisteminin context-switch (bağlam değiştirme) hantallığından kaçmak için **CPU Çekirdek 1**'e fiziksel olarak çivilenmiştir (Pinning).
- **Spin-Loop Dispatcher:** İşlemci asla `thread::sleep` durumuna geçmez. Gelen veri olmadığı anlarda `std::hint::spin_loop` ile L1 Cache'de pusuya yatar. Veri geldiği mikrosaniyede işler.
- **Donanımsal Saat (RDTSC):** Zaman ölçümleri için hantal syscall olan `SystemTime::now()` yerine, işlemcinin kendi döngü sayacı olan Time Stamp Counter (RDTSC) asmbly komutu kullanılarak nanosaniye hassasiyetinde mutlak zaman elde edilir.

### 2. Generational Ring Buffer
Veri taşıma omurgasıdır. Sıfır kopya (Zero-Copy) prensibiyle çalışır.
- **Pre-faulted Memory:** Başlangıçta 100.000 adet olay kaydı alabilecek devasa, ardışık bir RAM bloğu ayrılır.
- **Generational Indexing:** Okuma/Yazma göstergeleri atomik olarak güncellenir. Tüketici (Consumer) ve Üretici (Producer) hiçbir zaman birbirini kilitler üzerinden beklemez.
- **Veri Boyutu:** Her bir emir defteri (Orderbook) olayı belleğe tam oturması için L1 Cache line'a hizalanacak şekilde tasarlanmıştır.

### 3. Katı (Ruthless) Risk Simülatörü
Risk yönetimi "en kötü senaryoya" göre çalışır.
- **Float Yasaklı Matematik:** Gerçek donanım hızında çalışabilmek ve kayan nokta hatalarından kurtulmak için hesaplamalar tamamen Tamsayı (Fixed-Point Integer) kullanılarak yapılır.
- **Gerçek L2 VWAP (Hacim Ağırlıklı Fiyat):** Yüzeysel bir "Sabit Kayma" (Fixed Slippage) yerine, emrin piyasa derinlik tablosunda (LOB) hangi kademelere çarpacağını tek tek simüle ederek milimetrik zarar/kar tahmini yapar.

### 4. Sıfır Kopya Gözlem (Zero-Copy Observability)
HFT motoru uçarken onun hızından faydalanarak canlı verileri arayüze aktaran devrimsel bir yönetim panelidir.
- **Soğuk Yol (Cold Path) İzole Tokio:** Gözlem ve HTTP istekleri, Titanium Core'dan tamamen izole biçimde **CPU Çekirdek 0** üzerinde bir Tokio Runtime'ında yaşar.
- **Atomik Metrikler:** Orchestrator kendi hızını kesmeden verileri `AtomicU64` ile hafızaya yazar. İzlem thread'i bu verileri `Ordering::Acquire` ile okur.
- **Postcard (Binary Delta Akışı):** Veri serileştirme için şişkin JSON formatı çöpe atılmış, tamamen ikili (binary) ve mikroskobik boyutlu `Postcard` formatı kullanılmıştır.
- **Gömülü Dashboard (rust-embed):** Saf TypeScript, Vite ve uPlot ile geliştirilen, GPU hızlandırmalı modern karanlık arayüz (Admin UI) derlenerek doğrudan Rust binary'si içine gömülmüştür. Dış bir web sunucusuna ihtiyaç yoktur.

---

## 🛠️ Nasıl Derlenir ve Çalıştırılır?

Proje `unsafe_code` kullanılarak performansın sınırlarını zorlar (Ring Buffer ve RDTSC için ham bellek/assembly erişimi).

```bash
# Sadece core projesini çalıştırın. Web arayüzü içine gömülü (rust-embed) olarak ayağa kalkacaktır.
cargo run --release -p core
```

### Arayüze Erişim
Tarayıcınızdan şu adrese gidin:
**`http://localhost:8080/`**

- **Kill Switch:** Olası bir felaket durumunda web arayüzündeki Kırmızı Acil Durum butonuna **3 saniye basılı tutarak** sistemi anında 'Draining' (Güvenli Kapatma) moduna alabilirsiniz. Bu komut, Titanium Core'a Lock-Free kanal üzerinden 1 nanosaniyede iletilir.
