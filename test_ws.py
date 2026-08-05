import asyncio
import websockets

async def main():
    print("Testing Spot...")
    try:
        async with websockets.connect("wss://stream.binance.com:9443/ws/btcusdt@trade", open_timeout=5) as ws:
            print(await ws.recv())
    except Exception as e:
        print("Spot failed:", e)

    print("Testing Futures...")
    try:
        async with websockets.connect("wss://fstream.binance.com/ws/btcusdt@aggTrade", open_timeout=5) as ws:
            print(await ws.recv())
    except Exception as e:
        print("Futures failed:", e)

asyncio.run(main())
