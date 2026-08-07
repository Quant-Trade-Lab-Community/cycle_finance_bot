#!/usr/bin/env python3
"""
HEIUSDT Kırılım Stratejisi
==========================
detect-ms (MSMP 2.0) seviye + yapı analizini kullanır.

Mantık:
  1. detect-ms'den HEIUSDT 1m, 100 pencere analizi al.
  2. Yapı trendi YUKARI (ats > 0) ise:
       - En yüksek skorlu direnç (SH) seviyesini bul.
       - Fiyat bu seviyenin ÜZERİNDE kapattıysa → BUY.
  3. Yapı trendi AŞAĞI (ats < 0) ise:
       - En yüksek skorlu destek (SL) seviyesini bul.
       - Fiyat bu seviyenin ALTINDA kapattıysa → SELL.
  4. Şart sağlanırsa paper-service'e ilgili yönde market emri aç.

Periyot: her 20 pencere (1m → 20 dakika) bir analiz.
"""

import json
import os
import sys
import time
import urllib.request
import urllib.error

DETECT_MS_URL = os.environ.get("DETECT_MS_URL", "http://127.0.0.1:3002")
PRICE_FEED_URL = os.environ.get("PRICE_FEED_URL", "http://127.0.0.1:3004")
PAPER_API = os.environ.get("PAPER_API", "http://127.0.0.1:8080")
PAPER_USER = os.environ.get("PAPER_ADMIN_USER", "admin")
PAPER_PASS = os.environ.get("PAPER_ADMIN_PASS", "changeme123")

SYMBOL = os.environ.get("HEIUSDT_SYMBOL", "HEIUSDT")
INTERVAL = os.environ.get("HEIUSDT_INTERVAL", "1m")
LIMIT = int(os.environ.get("HEIUSDT_LIMIT", "100"))
CHECK_EVERY_WINDOWS = int(os.environ.get("HEIUSDT_CHECK_EVERY", "20"))
# Bekleme süresi saniye cinsinden (varsayılan: 20 pencere x 60sn = 1200sn = 20dk).
# Shell'den ayarlanabilir: heiusdt-wait <saniye>
WAIT_SEC = int(os.environ.get("HEIUSDT_WAIT_SEC", str(CHECK_EVERY_WINDOWS * 60)))
WAIT_FILE = "/tmp/heiusdt_wait_sec.txt"
QTY = os.environ.get("HEIUSDT_QTY", "1000")
DRY_RUN = "--dry-run" in sys.argv
ONCE = "--once" in sys.argv


def http_json(url, method="GET", body=None, token=None):
    req = urllib.request.Request(url, method=method)
    if body is not None:
        req.add_header("Content-Type", "application/json")
        req.data = json.dumps(body).encode()
    if token:
        req.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        return {"error": f"HTTP {e.code}: {e.read().decode()[:200]}"}
    except Exception as e:
        return {"error": str(e)}


def login():
    return http_json(f"{PAPER_API}/api/v1/auth/login", method="POST",
                     body={"username": PAPER_USER, "password": PAPER_PASS})


def get_positions(token):
    return http_json(f"{PAPER_API}/api/v1/account/positions", token=token)


def place_order(token, side, qty):
    oid = f"heiusdt-{int(time.time()*1000)}"
    return http_json(f"{PAPER_API}/api/v1/order", method="POST",
                     token=token,
                     body={
                         "client_order_id": oid,
                         "symbol": SYMBOL,
                         "side": side,
                         "order_type": "MARKET",
                         "quantity": qty,
                     })


def fetch_analysis():
    url = f"{DETECT_MS_URL}/api/ms?symbol={SYMBOL}&interval={INTERVAL}&limit={LIMIT}"
    return http_json(url)


def fetch_price_feed(symbol):
    """Price-feed'ten anlık fiyat alır (last → mark → index → ask fallback)."""
    d = http_json(f"{PRICE_FEED_URL}/api/lastprice/{symbol}")
    if "error" in d:
        return None, d["error"]
    p = d.get("price", {})
    for k in ("last", "mark", "index", "ask"):
        if p.get(k):
            return float(p[k]), None
    return None, "price-feed'te fiyat yok"


def best_level(levels, level_type):
    """En yüksek priority_score'a sahip SH (direnç) veya SL (destek) seviyesi."""
    cands = [l for l in levels if l.get("level_type") == level_type]
    if not cands:
        return None
    return max(cands, key=lambda l: float(l.get("priority_score", 0)))


def evaluate(data, price=None):
    """Kırılım koşulu sağlanırsa ('BUY'|'SELL'), değilse None döndürür.

    price: price-feed'ten gelen anlık fiyat; yoksa detect-ms current_price.
    """
    if "error" in data:
        return None, f"detect-ms hatası: {data['error']}"
    if not data.get("levels"):
        return None, "Seviye yok"

    if price is None:
        price = float(data.get("current_price", 0))
    ats = float(data.get("ats", 0))
    trend = data.get("trend_label", "")
    log = (f"Fiyat={price}  ATS={ats:.4f}  Trend={trend}  "
           f"Confluence=%{data.get('confluence_index', '')}")

    if ats > 0:
        level = best_level(data["levels"], "SH")
        if not level:
            return None, log + " | Direnç yok"
        lv = float(level["price"])
        score = level["priority_score"]
        if price > lv:
            return "BUY", (log + f" | 🎯 DİRENÇ KIRILDI SH={lv} (skor:{score})"
                                f" → BUY")
        return None, log + f" | Direnç yukarı kırılmadı SH={lv}"
    elif ats < 0:
        level = best_level(data["levels"], "SL")
        if not level:
            return None, log + " | Destek yok"
        lv = float(level["price"])
        score = level["priority_score"]
        if price < lv:
            return "SELL", (log + f" | 🎯 DESTEK KIRILDI SL={lv} (skor:{score})"
                                 f" → SELL")
        return None, log + f" | Destek aşağı kırılmadı SL={lv}"
    else:
        return None, log + " | Nötr trend"


def current_wait_sec():
    """Bekleme süresini alır: önce /tmp/heiusdt_wait_sec.txt, sonra env, sonra varsayılan."""
    try:
        with open(WAIT_FILE) as f:
            v = int(f.read().strip())
            if v > 0:
                return v
    except Exception:
        pass
    return WAIT_SEC


def analyze_once():
    """Bir analiz döngüsü. detect-ms ulaşılabilirse True, değilse False döndürür."""
    auth = login()
    if "access_token" not in auth:
        print(f"❌ Paper giriş başarısız: {auth}")
        return False
    token = auth["access_token"]

    data = fetch_analysis()
    if "error" in data:
        # detect-ms'e ulaşılamadı (connection refused) — kısa aralıkla yeniden dene
        print(f"[{time.strftime('%H:%M:%S')}] ⚠️  detect-ms erişilemiyor: {data['error']}")
        print("   → 10 sn sonra yeniden denenecek...")
        return False

    pf_price, pf_err = fetch_price_feed(SYMBOL)
    signal, msg = evaluate(data, price=pf_price)
    print(f"[{time.strftime('%H:%M:%S')}] {SYMBOL} {INTERVAL} {LIMIT} pencere")
    if pf_price is not None:
        print(f"  💹 price-feed: {pf_price}")
    elif pf_err:
        print(f"  ⚠️  price-feed: {pf_err} (detect-ms fiyatı kullanıldı)")
    print(f"  {msg}")

    if signal is None:
        return True

    # Aynı sembolde zaten pozisyon varsa tekrar açma
    pos = get_positions(token)
    for p in pos.get("positions", []):
        if p.get("symbol") == SYMBOL and float(p.get("quantity", 0)) != 0:
            print(f"  ⏭️  {SYMBOL} pozisyonu zaten var ({p.get('side')} "
                  f"{p.get('quantity')}). Yeni emir açılmadı.")
            return True

    if DRY_RUN:
        print(f"  🧪 [DRY-RUN] {signal} emri gönderilmedi (QTY={QTY})")
        return True

    resp = place_order(token, signal, QTY)
    if "order_id" in resp:
        print(f"  ✅ {signal} emri açıldı → id={resp['order_id']} "
              f"avg={resp.get('avg_price')}")
    else:
        print(f"  ❌ Emir reddedildi: {resp}")
    return True


def main():
    print("══════════════════════════════════════════════════")
    print(f"  🎯 HEIUSDT KIRILIM STRATEJİSİ  ({SYMBOL} {INTERVAL})")
    print(f"  Pencere: {LIMIT} | Kontrol: her {CHECK_EVERY_WINDOWS} pencere")
    print(f"  Paper: {PAPER_API} | detect-ms: {DETECT_MS_URL}")
    if DRY_RUN:
        print("  🧪 MOD: DRY-RUN (emir gönderilmez)")
    print("══════════════════════════════════════════════════")

    if ONCE:
        analyze_once()
        return

    while True:
        try:
            ok = analyze_once()
        except Exception as e:
            print(f"⚠️  Hata: {e}")
            ok = False
        if not ok:
            # detect-ms'e ulaşılamadı — 10 sn sonra yeniden dene
            print(f"  🔄 10 sn sonra yeniden deneniyor... {time.strftime('%H:%M:%S')}\n")
            time.sleep(10)
            continue
        sleep_s = current_wait_sec()
        print(f"  😴 {sleep_s} saniye ({sleep_s/60:.1f} dk) bekleniyor... (heiusdt-wait ile değiştir)"
              f" {time.strftime('%H:%M:%S')}\n")
        time.sleep(sleep_s)


if __name__ == "__main__":
    main()
