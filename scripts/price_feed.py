#!/usr/bin/env python3
"""
Cycle Finance — PRICE FEED Servisi (arka plan daemon)
=====================================================
Sistemde tanımlı sembollerin anlık last price'ını çeker ve
tüm katmanlara sunar:

  - HTTP API  : http://127.0.0.1:3004/api/lastprice
                http://127.0.0.1:3004/api/lastprice/BTCUSDT
  - JSON dosya: /tmp/price_feed.json  (her güncellemede yazılır)

Tüketiciler:
  - ALERT katmanı   → fiyat karşılaştırması için
  - PAPER katmanı   → mark price / fiyat beslemesi için
  - STRATEJİ katmanı → kırılım/seviye kontrolü için

Sembol kaynağı: $PRICE_FEED_SYMBOLS (virgülle ayrılmış) veya
alerts.toml içindeki semboller + HEIUSDT.

Kullanım:  python3 scripts/price_feed.py   (arka planda çalışır)
"""

import json
import os
import sys
import time
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

BINANCE_URL = "https://api.binance.com/api/v3/ticker/price"
PORT = int(os.environ.get("PRICE_FEED_PORT", "3004"))
REFRESH_SEC = float(os.environ.get("PRICE_FEED_REFRESH", "1.0"))
OUT_FILE = "/tmp/price_feed.json"

# ── Sembol listesi ───────────────────────────────────────────
def default_symbols():
    syms = set()
    # alerts.toml'dan sembolleri topla
    alerts_toml = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "alerts.toml")
    try:
        for line in open(alerts_toml):
            line = line.strip()
            if line.startswith("symbol") and "=" in line:
                s = line.split("=", 1)[1].strip().strip('"').strip("'")
                if s:
                    syms.add(s)
    except Exception:
        pass
    syms.add("HEIUSDT")
    return sorted(syms)


SYMBOLS = [s for s in os.environ.get("PRICE_FEED_SYMBOLS", "").split(",") if s] or default_symbols()

# ── Paylaşılan durum ─────────────────────────────────────────
_lock = None
PRICES = {}          # symbol -> {"price": float, "ts": int, "ok": bool}
LAST_ERROR = None


# ── Binance'ten fiyat çek ────────────────────────────────────
def fetch_prices():
    global LAST_ERROR
    out = {}
    try:
        url = BINANCE_URL + "?symbols=" + urllib.parse.quote(json.dumps(SYMBOLS))
        req = urllib.request.Request(url, headers={"User-Agent": "cycle-price-feed"})
        with urllib.request.urlopen(req, timeout=5) as r:
            rows = json.loads(r.read().decode())
        for row in rows:
            sym = row.get("symbol")
            if sym:
                out[sym] = float(row.get("price", 0))
        LAST_ERROR = None
    except Exception as e:
        LAST_ERROR = str(e)
    return out


def refresh_loop():
    global PRICES
    while True:
        fetched = fetch_prices()
        ts = int(time.time())
        now = {s: {"price": fetched.get(s), "ts": ts,
                   "ok": s in fetched} for s in SYMBOLS}
        if fetched:
            PRICES = now
            try:
                with open(OUT_FILE, "w") as f:
                    json.dump({"updated": ts, "prices": now,
                               "symbols": SYMBOLS}, f, indent=2)
            except Exception:
                pass
        time.sleep(REFRESH_SEC)


# ── HTTP API ─────────────────────────────────────────────────
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def _send(self, code, obj):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        path = self.path.strip("/")
        if path == "api/lastprice":
            self._send(200, {"updated": PRICES.get("__ts", int(time.time())),
                             "prices": {k: v["price"] for k, v in PRICES.items()},
                             "symbols": SYMBOLS,
                             "last_error": LAST_ERROR})
            return
        if path.startswith("api/lastprice/"):
            sym = path.split("/")[-1].upper()
            if sym in PRICES:
                self._send(200, {"symbol": sym,
                                 "price": PRICES[sym]["price"],
                                 "ts": PRICES[sym]["ts"]})
            else:
                self._send(404, {"error": f"bilinmeyen sembol: {sym}",
                                 "available": SYMBOLS})
            return
        if path == "health":
            self._send(200, {"status": "ok", "symbols": SYMBOLS,
                             "prices": {k: v["price"] for k, v in PRICES.items()}})
            return
        self._send(404, {"error": "not found"})


def main():
    import threading
    global _lock
    _lock = threading.Lock()

    print("=" * 55)
    print("  💹  PRICE FEED — Anlık LastPrice Servisi")
    print(f"  Semboller : {', '.join(SYMBOLS)}")
    print(f"  Yenileme  : {REFRESH_SEC}s")
    print(f"  HTTP API  : http://127.0.0.1:{PORT}/api/lastprice")
    print(f"  JSON çıktı: {OUT_FILE}")
    print("=" * 55)

    threading.Thread(target=refresh_loop, daemon=True).start()

    try:
        srv = ThreadingHTTPServer(("127.0.0.1", PORT), Handler)
        srv.serve_forever()
    except KeyboardInterrupt:
        print("\nPrice feed kapatıldı.")
        sys.exit(0)


if __name__ == "__main__":
    main()
