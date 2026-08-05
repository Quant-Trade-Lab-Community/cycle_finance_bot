import sqlite3
import pandas as pd

def analyze_risk():
    print("Veritabanı taranıyor... İstatistiksel risk hesaplanıyor...\n")
    conn = sqlite3.connect('market_data.db')
    
    # Tüm trade verilerini SQL üzerinden gruplayarak istatistiksel özet çıkar
    query = """
    SELECT 
        symbol as Parite,
        COUNT(*) as Islem_Adedi,
        SUM(price * quantity) as Toplam_Hacim_USDT,
        MIN(price) as Min_Fiyat,
        MAX(price) as Max_Fiyat
    FROM trades
    GROUP BY symbol
    HAVING Islem_Adedi > 50
    ORDER BY Toplam_Hacim_USDT DESC
    """
    
    df = pd.read_sql(query, conn)
    
    if df.empty:
        print("Yeterli veri bulunamadı.")
        return

    # Fiyat dalgalanma riskini (Volatility) yüzde olarak hesapla
    df['Volatilite_Riski_%'] = ((df['Max_Fiyat'] - df['Min_Fiyat']) / df['Min_Fiyat']) * 100
    
    print("=== 📊 PİYASA HACİM VE RİSK DAĞILIMI (EN ÇOK İŞLEM GÖREN 15 PARİTE) ===")
    print(df.head(15).to_string(index=False, float_format="%.2f"))
    
    print("\n=== ⚠️ EN YÜKSEK RİSK / VOLATİLİTE İÇEREN 10 PARİTE ===")
    risk_df = df.sort_values(by='Volatilite_Riski_%', ascending=False).head(10)
    print(risk_df[['Parite', 'Islem_Adedi', 'Volatilite_Riski_%', 'Toplam_Hacim_USDT']].to_string(index=False, float_format="%.2f"))

if __name__ == "__main__":
    analyze_risk()
