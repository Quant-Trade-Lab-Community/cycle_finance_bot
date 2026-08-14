from websocket import WebSocketApp
import json
import time

SYMBOL = "btcusdt"
LAST_PRICE = None

def on_message(ws, message):
    global LAST_PRICE
    try:
        data = json.loads(message)
        # @ticker stream'inde son fiyat "c" alanında gelir
        if "c" in data:
            LAST_PRICE = float(data["c"])
            print(f"[{time.strftime('%H:%M:%S')}] BTCUSDT Last Price: {LAST_PRICE:.2f}")
    except Exception as e:
        print(f"Hata: {e}")

def on_error(ws, error):
    print(f"Bağlantı hatası: {error}")

def on_close(ws, close_status_code, close_msg):
    print("Bağlantı kapandı. Yeniden bağlanılıyor...")
    # Otomatik yeniden bağlanma için tekrar çağır
    time.sleep(2)
    start_websocket()

def on_open(ws):
    print(f"{SYMBOL} Last Price akışı başlatıldı (Ticker).")

def start_websocket():
    # @ticker stream'i son işlem fiyatını (lastPrice) getirir
    url = f"wss://fstream.binance.com/ws/{SYMBOL}@ticker"
    
    ws = WebSocketApp(
        url,
        on_open=on_open,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close
    )
    ws.run_forever()

if __name__ == "__main__":
    print("Binance Futures BTCUSDT - Son İşlem Fiyatı (Last Price) Akışı")
    start_websocket()