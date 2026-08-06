# Algoritmik Şema: `db.rs` (SQLite & Persistence Layer)

Bu dosya sistemin veri ambarıdır. Yüksek Frekanslı sistemlerde en büyük sorun "Disk Gecikmesi" (I/O Latency)'dir. Ana HFT motoru saniyede on binlerce kez diske yazma emri verirse sistem kilitlenir. Bu yüzden `db.rs`, ana motoru 1 milisaniye bile bekletmeyen (Asenkron) ve disk I/O limitlerini `WAL` (Write-Ahead Logging) ve `Batch Insert` (Toplu Yazma) teknikleriyle aşan bir yapıya sahiptir.

## Akış Şeması (Flowchart)

```mermaid
graph TD
    Start([Ana Motor: db_tx.send_evicted])
    
    Start --> FlumeQ((Flume db_rx Kuyruğu))
    
    FlumeQ --> Recv[Arka Plan DB İşçisi<br/>rx.recv]
    
    Recv --> Route[Veri Tipi Nedir?]
    
    Route --> |Trade| BufTrade[Trades Tablosu İçin RAM'de Beklet]
    Route --> |Orderbook| BufOB[Orderbooks Tablosu İçin RAM'de Beklet]
    Route --> |Liquidation| BufLiq[Liquidations Tablosu İçin RAM'de Beklet]
    Route --> |Funding Rate| BufFund[Funding_Rates Tablosu İçin RAM'de Beklet]
    Route --> |BookTicker| BufTick[Booktickers Tablosu İçin RAM'de Beklet]
    Route --> |Open Interest| BufOI[Open_Interests Tablosu İçin RAM'de Beklet]
    
    BufTrade --> IncCount[batch_count += 1]
    BufOB --> IncCount
    BufLiq --> IncCount
    BufFund --> IncCount
    BufTick --> IncCount
    BufOI --> IncCount
    
    IncCount --> CheckBatch{batch_count >= 10.000<br/>VEYA<br/>Zaman > 1 Saniye mi?}
    
    CheckBatch -- Hayır --> Recv
    CheckBatch -- Evet --> BeginTx[BEGIN TRANSACTION]
    
    BeginTx --> Execute[Tüm SQL INSERT Sorgularını<br/>Tek Seferde (Bulk) Çalıştır]
    
    Execute --> CommitTx[COMMIT]
    
    CommitTx --> Reset[batch_count = 0]
    
    Reset --> Recv
```

## Algoritmik Adımlar

1. **Bağımsız Yaşam Alanı:** Veritabanı yöneticisi ana ticari motorun (Core) ipliğinde (thread) çalışmaz. Tamamen ayrı bir thread üzerinde, kilit-siz bir mesaj kutusundan (Flume) eski/ezilmiş (Evicted) verilerin düşmesini bekler. Ana motor I/O beklemesi yaşamaz.
2. **Batching (Yığınlama):** Gelen her yeni mesaj için `INSERT` komutu ile diske gidilmez. Veriler RAM üzerinde toplanır (`batch_count`). 
3. **Commit Tetikleyici:** İşçi, sayacın 10.000'e ulaşmasını VEYA son yazmanın üzerinden 1 saniye geçmesini bekler. Hangisi önce gerçekleşirse tetiği çeker.
4. **Transaction (Toplu Aktarım):** Tetik çekildiğinde SQLite bağlantısına `BEGIN TRANSACTION` komutu yollanır, on binlerce satır veri milisaniyeler içinde SQLite motoruna basılır ve `COMMIT` ile disk mühürlenir. WAL (Write-Ahead Logging) modunda olduğu için SQLite okuyucuları da kilitlenmeden veriye erişebilir.
