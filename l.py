import asyncio
import json
import aiohttp
import websockets

async def get_active_symbols():
    """Binance Vadeli İşlemler üzerindeki aktif USDT paritelerini çeker."""
    url = "https://binance.com"
    async with aiohttp.ClientSession() as session:
        try:
            async with session.get(url) as response:
                if response.status == 200:
                    data = await response.json()
                    symbols = [
                        item['symbol'].lower() 
                        for item in data['symbols'] 
                        if item['status'] == 'TRADING' and item['quoteAsset'] == 'USDT'
                    ]
                    return symbols
        except Exception as e:
            print(f"⚠️ API'den parite listesi alınamadı, yerel liste kullanılacak. Hata: {e}")
            
    # API hatası durumunda elle güvenli liste
    return ["btcusdt", "ethusdt", "solusdt", "xrpusdt", "bnbusdt", "linkusdt", "adausdt"]

async def start_stream_group(group_id, symbols_chunk):
    """Belirli bir parite grubu için WebSocket bağlantısı başlatır."""
    # Streams dizilimini hazırlıyoruz
    streams = [f"{sym}@forceOrder" for sym in symbols_chunk]
    stream_string = "/".join(streams)
    
    # URL'i elle en temiz haliyle birleştiriyoruz
    base_url = f"wss://://binance.com{stream_string}"
    
    print(f"🔄 [Grup {group_id}] {len(symbols_chunk)} parite için bağlantı kuruluyor...")
    
    while True:
        try:
            # Sıkı URI kontrolü için trust_env=False ekliyoruz (Sistemdeki hatalı proxy ayarlarını ezer)
            async with websockets.connect(base_url, trust_env=False) as websocket:
                print(f"✅ [Grup {group_id}] Bağlantı başarılı! Canlı veriler bekleniyor...")
                while True:
                    response = await websocket.recv()
                    raw_data = json.loads(response)
                    
                    data = raw_data.get('data', {})
                    order_info = data.get('o', {})
                    
                    symbol = order_info.get('s')
                    side = order_info.get('S')
                    quantity = order_info.get('q')
                    price = order_info.get('p')
                    
                    if not symbol:
                        continue
                        
                    usd_value = float(quantity) * float(price)
                    
                    # 500$ ve üzerindeki likidasyonları ekrana bas
                    if usd_value >= 500:
                        direction = "🔴 LONG LİKİDASYONU" if side == "SELL" else "🟢 SHORT LİKİDASYONU"
                        print(f"[{symbol}] {direction} | Fiyat: ${float(price):,.2f} | Değer: ${usd_value:,.2f}")
                        
        except websockets.exceptions.ConnectionClosed:
            print(f"⚠️ [Grup {group_id}] Bağlantı koptu! 5 saniye içinde yeniden bağlanılıyor...")
            await asyncio.sleep(5)
        except Exception as e:
            # Hatalı URL'in tam olarak nereden geldiğini görmek için base_url'i de hataya yazdırıyoruz
            print(f"❌ [Grup {group_id}] Hata oluştu: {e}")
            print(f"   ↳ Gönderilmeye çalışılan URL: {base_url}")
            await asyncio.sleep(3)

async def main():
    symbols = await get_active_symbols()
    print(f"📊 Toplam aktif parite sayısı: {len(symbols)}")
    
    # Binance sınırı nedeniyle pariteleri 100'erli daha küçük ve güvenli gruplara bölüyoruz
    chunk_size = 100
    chunks = [symbols[i:i + chunk_size] for i in range(0, len(symbols), chunk_size)]
    
    tasks = []
    for index, chunk in enumerate(chunks, 1):
        tasks.append(start_stream_group(index, chunk))
        
    await asyncio.gather(*tasks)

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n👋 Program kullanıcı tarafından sonlandırıldı.")
