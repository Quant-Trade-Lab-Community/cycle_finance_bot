# Demir Yumruk v3.0 - Hibrid (Nautilus x Demir Yumruk) HFT Motoru

Demir Yumruk v3.0, dünyanın en acımasız ve yüksek frekanslı kripto ticaret sistemlerine kafa tutmak için sıfırdan "Bare-Metal" (Donanıma yakın) felsefesiyle yazılmış kurumsal bir HFT (High-Frequency Trading) motorudur.

Yeni sürümle birlikte, sistemin çekirdek hızı korunurken **NautilusTrader**'ın muazzam analitik yetenekleri (Python Strateji Desteği, Risk ve Portföy Yönetimi, Geriye Dönük Simülasyon) altyapıya entegre edilmiştir.

## 🚀 Temel Felsefe: "Zero-Copy Hız + Python Zekası"
- **Zero-Copy IPC (Lock-Free):** Terminal modülleri arasındaki iletişim `/dev/shm` (Generational Ring Buffer) kullanılarak, serileştirme ve kopyalama olmadan nanosaniyeler içinde sağlanır.
- **Python Strateji Köprüsü (PyO3):** Quant araştırmacıları stratejilerini Python diliyle yazarken, arka planda C/Rust hızında bir altyapı çalışır. Hızdan ödün vermeden maksimum strateji geliştirme esnekliği sunulur.
- **Kesintisiz Risk Yönetimi:** Cüzdan bakiyesi (Margin), Gerçekleşmiş / Gerçekleşmemiş Kâr-Zarar (PnL) ve Max Drawdown limitleri Rust'ın hızında atomik olarak hesaplanır.

---

## 🏛️ Mimari Katmanlar (Terminal Modları)

Sistem `RUN_MODE` ortam değişkeniyle farklı terminallere (bağımsız süreçlere) bölünerek çalışır.

### 1. DATA Terminali (`RUN_MODE=DATA`)
Piyasa verilerini çeken, parse eden ve kilitsiz (Lock-free) paylaşımlı hafıza kuyruğuna (Generational Ring Buffer) yazan ana motordur.

### 2. STRATEGY Terminali (`RUN_MODE=STRATEGY`)
Veri kuyruğundan sıfır-kopya (Zero-Copy) prensibiyle verileri sömürür.
- İçerisinde **PyO3** ile gömülü bir Python yorumlayıcısı barındırır.
- `strategies/` klasöründeki Python scriptlerini (örn: `test_strategy.py`) okur ve piyasa verisini anında Python'daki `on_event(tick)` fonksiyonuna fırlatır.
- Stratejilerin oluşturduğu emirleri (Order) doğrudan Emir Kuyruğuna (Order Ring Buffer) basar.

### 3. PAPER Terminali (`RUN_MODE=PAPER`)
NautilusTrader'dan esinlenilerek geliştirilen **Risk ve Portföy Yönetimi** katmanıdır.
- Canlı piyasaya emir göndermek yerine simüle edilmiş (Paper Trading) cüzdanı yönetir.
- Komisyon oranlarını, kaldıraçları (Leverage) ve gerçekleşmemiş PnL miktarlarını canlı hesaplar.
- Kâr-Zarar limitleri aşıldığında sistemdeki alım-satımı durduran güvenlik duvarıdır.

### 4. BACKTEST Terminali (`RUN_MODE=BACKTEST`)
Geriye dönük testler (Simülasyon) için NautilusTrader'ın esnekliğini sağlayan modülümüzdür.
- Canlı WebSocket verisi yerine diskteki bir `.csv` dosyasını (`test_data.csv`) okur.
- Okuduğu geçmiş veriyi anında JSON baytlarına dönüştürerek paylaşımlı hafızaya basar.
- **STRATEGY** terminali, verinin geçmişten mi yoksa canlıdan mı geldiğini bilmez. Gerçek piyasa kodu ile simülasyon kodu %100 aynı kalır.

---

## 🛠️ Nasıl Derlenir ve Çalıştırılır?

Bu projeyi derleyebilmek için sisteminizde Python kütüphaneleri bulunmalıdır.

### Önkoşullar (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install python3.14-dev
```

### Derleme ve Başlatma
Proje farklı terminaller halinde paralel çalıştırılmalıdır:

```bash
# 1. Veri Okuma Modu (Canlı Piyasa veya Backtest'ten sadece biri çalıştırılmalı)
# Canlı Piyasa:
RUN_MODE=DATA cargo run --release -p core
# VEYA Simülasyon:
RUN_MODE=BACKTEST CSV_PATH="./test_data.csv" cargo run --release -p core

# 2. Risk ve Portföy Yöneticisi (Paper Trading Terminali)
RUN_MODE=PAPER cargo run --release -p core

# 3. Python Strateji Motoru (Strategy Terminali)
RUN_MODE=STRATEGY cargo run --release -p core
```

Sistemde web arayüzü çöpe atılmış, tamamen hız odaklı, efsanevi safkan **HFT (High Frequency Trading) Rustyline CLI** mimarisi benimsenmiştir. Terminal üzerinden anlık durumunuzu takip edebilirsiniz.
