# Demir Yumruk v3.0 - Detaylı Dosya ve Modül Akış Şemaları

Bu belge, projenin her bir kritik kod dosyasının (module) iç işleyişini gösteren Mermaid akış şemalarını barındırır.

## 1. Yönlendirici (Orchestrator)
**Dosya:** `core/src/main.rs`
```mermaid
flowchart TD
    Start([Başlangıç - cargo run]) --> ReadEnv{RUN_MODE\nNedir?}
    
    ReadEnv -->|DATA| DataMode[Veri Terminalini Başlat\nBinance WS Bağlan]
    ReadEnv -->|BACKTEST| BacktestMode[Simülasyon Modunu Başlat\nCSV Oku]
    ReadEnv -->|STRATEGY| StrategyMode[Strateji Motorunu Başlat\nPython Entegrasyonu]
    ReadEnv -->|PAPER| PaperMode[Risk & Cüzdan Motorunu Başlat\nKâr/Zarar Hesapla]
    
    DataMode --> |WebSocket'ten okunan byte'ları| RingB[(Generational Ring Buffer)]
    BacktestMode --> |CSV satırlarını JSON'a çevirip| RingB
```

## 2. Sıfır-Kopya Hafıza (Generational Ring Buffer)
**Dosya:** `core/src/memory/ring_buffer.rs`
```mermaid
flowchart LR
    subgraph Yazan ["Yazan (Producer)"]
        W1[Veri Gelir] --> W2[Atomik Head Pointer'ı Oku]
        W2 --> W3[Slot'a Byte'ları Kopyala]
        W3 --> W4[Head Pointer'ı +1 Artır]
    end
    
    subgraph PaylasimliHafiza ["Paylaşımlı Hafıza (mmap /dev/shm)"]
        R[(Generational\nRing Buffer)]
    end
    
    subgraph Okuyan ["Okuyan (Consumer)"]
        C1[Spin Loop\nBekle] --> C2{Cursor < Head ?}
        C2 -- Evet --> C3[Slot Verisini Oku]
        C2 -- Hayır --> C1
        C3 --> C4[Cursor +1 Artır]
    end
    
    W4 -.-> R
    R -.-> C2
```

## 3. Emir Hafıza Kuyruğu (Order Ring Buffer)
**Dosya:** `core/src/memory/order_ring.rs`
```mermaid
flowchart TD
    Strategy[Strateji Karar Verdi] --> PushOrder[Order::new() Oluştur]
    PushOrder --> RingWrite[OrderRingBuffer.push()]
    RingWrite --> CheckCap{Kapasite\nDoldu mu?}
    CheckCap -- Evet --> Overwrite[En Eski Verinin\nÜstüne Yaz]
    CheckCap -- Hayır --> WriteData[Hafızaya Yaz & Pointer Artır]
    
    WriteData -.-> Paper[PAPER/LIVE Terminali Okur]
```

## 4. Python Strateji Köprüsü
**Dosya:** `core/src/strategy/python_bridge.rs`
```mermaid
flowchart TD
    RustEnv[Rust Ortamı] --> PyGIL[PyO3 GIL Al]
    PyGIL --> PyInit[test_strategy.py Dosyasını\nPyModule Olarak Yükle]
    
    RustEvent[OwnedEvent Geldi] --> Decode[Sembol ve Verileri\nRust Struct'ından Oku]
    Decode --> ToDict[PyDict::new_bound ile\nSözlüğe Çevir]
    ToDict --> CallPy[Python on_event() Fonksiyonunu\nSözlük ile Çağır]
```

## 5. Strateji CLI Motoru
**Dosya:** `core/src/cli/strategy_cli.rs`
```mermaid
flowchart TD
    Start[CLI Başlar] --> Setup[Ring Buffer & PythonBridge Başlat]
    
    subgraph ArkaPlanDongusu ["Arka Plan Döngüsü"]
        L1[Sonsuz Loop] --> L2{read_slot(cursor)}
        L2 -- Yok --> Spin[spin_loop]
        Spin --> L1
        L2 -- Var --> Parse[EventParser::parse]
        Parse --> Exec[PythonBridge.on_event]
        Exec --> Inc[cursor += 1]
        Inc --> L1
    end
```

## 6. Risk ve Portföy Yönetimi
**Dosya:** `core/src/risk/portfolio.rs`
```mermaid
flowchart TD
    Init[Portfolio Yarat\n(10K USD, %20 Drawdown)] --> Wait[Emir Bekle]
    
    Wait --> Fill[process_fill Çağrılır]
    Fill --> CalcComm[Bakiye -= Komisyon]
    
    CalcComm --> CheckDir{Pozisyonu\nKapatıyor mu?}
    CheckDir -- Evet --> Realized[Gerçekleşen PnL Hesapla\nBakiyeye Ekle]
    CheckDir -- Hayır --> Avg[Ortalama Giriş Fiyatını\nGüncelle]
    
    Realized --> CheckDD[is_drawdown_exceeded()]
    Avg --> CheckDD
```

## 7. CSV Simülatörü (Backtester)
**Dosya:** `core/src/engine/backtester.rs`
```mermaid
flowchart TD
    Start[BACKTEST Modu] --> Open[test_data.csv Dosyasını Aç]
    Open --> Loop[Satır Satır Oku]
    
    Loop --> ParseCSV[CSV'yi Parçala]
    ParseCSV --> BuildJSON[Binance WS Formatında\nSahte JSON Metni Yarat]
    
    BuildJSON --> Push[Ring Buffer'a Bas]
    Push --> Limit{Mod 100.000\nSıfır mı?}
    
    Limit -- Evet --> Yield[thread::yield_now]
    Limit -- Hayır --> Loop
    Yield --> Loop
```

## 8. Binance WebSocket Adaptörü
**Dosya:** `adapter/src/binance.rs`
```mermaid
flowchart TD
    Start[tokio_tungstenite Başlar] --> Sub[Binance WSS Bağlan]
    Sub --> Listen[Mesaj Bekle]
    
    Listen --> Recv[Text/Binary Frame Geldi]
    Recv --> Chunk{Veri Parçalanmış mı?}
    Chunk -- Evet --> Concat[Veriyi Birleştir]
    Chunk -- Hayır --> SendFlume[Flume Channel ile Core'a Yolla]
    SendFlume --> Listen
```

## 9. Veri Ayrıştırıcı (Event Parser)
**Dosya:** `core/src/tick.rs`
```mermaid
flowchart TD
    Input[WS'den Ham Byte Array Gelir] --> Parse[simd_json::to_borrowed_value]
    Parse --> CheckStream{stream\n@trade mi?}
    
    CheckStream -- Evet --> Trade[Sembol, Fiyat, Miktar Oku]
    Trade --> CreateTrade[OwnedEvent::new_trade()]
    
    CheckStream -- Hayır --> CheckBook{stream\n@bookTicker mı?}
    CheckBook -- Evet --> Book[Bids/Asks Oku]
    Book --> CreateBook[OwnedEvent::new_book()]
    
    CreateTrade --> Output[OwnedEvent Döndür]
    CreateBook --> Output
```
