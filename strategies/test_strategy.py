def on_start():
    print("Python Strategy: Başlatıldı (on_start)")

def on_tick(tick):
    # tick bir sözlük (dictionary) olarak Rust'tan geliyor
    symbol = tick.get("symbol", "UNKNOWN")
    bid_price = tick.get("bid_price", 0.0)
    ask_price = tick.get("ask_price", 0.0)
    
    # Basit bir örnek strateji: Spread (Makas) analizi
    spread = ask_price - bid_price
    if spread > 5.0:
        print(f"[{symbol}] ⚠️ Yüksek Spread Tespit Edildi: {spread:.2f} USD")
    elif ask_price > 0.0:
        # Sadece sessizce veriyi izle
        pass
