#!/usr/bin/env python3
"""
Cycle Finance — LISTENER Katmanı
================================
Açık pozisyonları paper-service'ten çeker ve data merkezinden gelen
verilerle ANLIK METRİK ANALİZİ yapar.

Metrikler şu an BOŞ (placeholder). Gerçek metrikler sonra eklenecek.
Veri kaynağı: paper-service REST API (:8080)
  - /api/v1/account/positions  → açık pozisyonlar
  - /api/v1/system/health      → son fiyat (data merkezinden)

Kullanım:  python3 scripts/listener.py
"""

import json
import os
import sys
import time
import urllib.request

PAPER_API = os.environ.get("PAPER_API", "http://127.0.0.1:8080")
PAPER_USER = os.environ.get("PAPER_ADMIN_USER", "admin")
PAPER_PASS = os.environ.get("PAPER_ADMIN_PASS", "changeme123")
REFRESH_SEC = float(os.environ.get("LISTENER_REFRESH_SEC", "2"))


def http_json(url, method="GET", body=None, token=None):
    req = urllib.request.Request(url, method=method)
    if body is not None:
        req.add_header("Content-Type", "application/json")
        req.data = json.dumps(body).encode()
    if token:
        req.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read().decode())
    except Exception as e:
        return {"error": str(e)}


def login():
    return http_json(f"{PAPER_API}/api/v1/auth/login", method="POST",
                     body={"username": PAPER_USER, "password": PAPER_PASS})


def fetch():
    auth = login()
    token = auth.get("access_token", "")
    if not token:
        return None, auth.get("error", "giriş başarısız")

    health = http_json(f"{PAPER_API}/api/v1/system/health", token=token)
    pos = http_json(f"{PAPER_API}/api/v1/account/positions", token=token)
    return (health, pos), None


def compute_metrics(positions):
    """AÇIK POZİSYONLAR İÇİN ANLIK METRİK ANALİZİ.

    ŞU AN BOŞ — metrikler sonra eklenecek.
    Her pozisyon için döndürülecek metrik örnekleri (boş şablon):
      - mark_price, unrealized_pnl, equity_risk, ...
    """
    metrics = {}
    for p in positions:
        sym = p.get("symbol", "?")
        # ── METRİK ŞABLONU (doldurulacak) ─────────────────────
        metrics[sym] = {
            "placeholder": True,
            "unrealized_pnl": p.get("unrealized_pnl"),
            "mark_price": p.get("mark_price"),
        }
    return metrics


def render(health, pos, metrics):
    """Terminal ekranına tablo çizer."""
    os.system("clear")
    print("═" * 60)
    print("  🛰️  LISTENER — ANLIK POZİSYON METRİKLERİ")
    print(f"  Paper: {PAPER_API}  |  Yenileme: {REFRESH_SEC}s")
    print("═" * 60)

    if health:
        last = health.get("last_price")
        status = health.get("status", "?")
        print(f"  Veri Merkezi: {status}  |  Son Fiyat: {last}")
    else:
        print(f"  ⚠️  Veri Merkezi: {health}")

    print("-" * 60)
    positions = pos.get("positions", []) if pos else []
    if not positions:
        print("  📭 AÇIK POZİSYON YOK")
    else:
        print(f"  {'SEMBOL':<12}{'YÖN':<8}{'MİKTAR':<10}{'GİRİŞ':<14}{'MARK':<14}{'METRİK'}")
        print("  " + "-" * 56)
        for p in positions:
            sym = p.get("symbol", "?")
            side = p.get("side", "?")
            qty = p.get("quantity", 0)
            entry = p.get("avg_entry_price", 0)
            mark = p.get("mark_price")
            metric = metrics.get(sym, {})
            # ── METRİK GÖSTERİMİ (boş) ───────────────────────
            mtext = "⏳ analiz bekliyor" if metric.get("placeholder") else "—"
            print(f"  {sym:<12}{side:<8}{qty:<10}{entry:<14}{str(mark):<14}{mtext}")
    print("-" * 60)
    now = time.strftime("%H:%M:%S")
    print(f"  Son güncelleme: {now}  (Ctrl+C ile çık)")

    # ── DIŞA AKTARILACAK METRİKLER (şimdilik sadece konsol) ──
    # Gelecekte: JSON dosyaya yaz / herkese açık porttan yayınla.
    with open("/tmp/listener_metrics.json", "w") as f:
        json.dump({"timestamp": now, "metrics": metrics,
                   "positions": positions}, f, default=str)


def main():
    print("═" * 60)
    print("  🛰️  LISTENER KATMANI BAŞLATILIYOR")
    print("═" * 60)
    time.sleep(1)

    while True:
        try:
            data, err = fetch()
            if err:
                print(f"⚠️  {err} — yeniden deneniyor...")
                time.sleep(REFRESH_SEC)
                continue
            health, pos = data
            positions = pos.get("positions", [])
            metrics = compute_metrics(positions)
            render(health, pos, metrics)
        except KeyboardInterrupt:
            print("\nListener kapatılıyor.")
            break
        except Exception as e:
            print(f"⚠️  Hata: {e} — yeniden deneniyor...")
        time.sleep(REFRESH_SEC)


if __name__ == "__main__":
    main()
