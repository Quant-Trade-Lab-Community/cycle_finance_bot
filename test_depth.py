import asyncio
import websockets
import json

async def main():
    async with websockets.connect("wss://stream.binance.com:9443/ws/btcusdt@depth5@100ms") as ws:
        print(await ws.recv())

asyncio.run(main())
