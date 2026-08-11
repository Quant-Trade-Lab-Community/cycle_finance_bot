#!/usr/bin/env bash
# PAPER sistemini kapatır (veri akışları + paper-service).
set -euo pipefail

echo "=== PAPER sistemi kapatılıyor ==="
pkill -x paper-service 2>/dev/null && echo "  paper-service durduruldu" || echo "  paper-service zaten kapalı"

FLOWS="flow-trade flow-depth flow-liquidation flow-oi flow-funding flow-markprice flow-lastprice flow-indexprice"
for f in $FLOWS; do
  if pkill -x "$f" 2>/dev/null || pkill -f "$f" 2>/dev/null; then
    echo "  $f durduruldu"
  fi
done
echo "  Veri akışları durduruldu"

# Paylaşımlı hafıza temizliği (akış ring'leri + rate kapısı)
rm -f /dev/shm/cycle_finance_trades /dev/shm/cycle_finance_depth /dev/shm/cycle_finance_liquidations /dev/shm/cycle_finance_open_interest /dev/shm/cycle_finance_funding /dev/shm/cycle_finance_markprice /dev/shm/cycle_finance_lastprice /dev/shm/cycle_finance_indexprice /dev/shm/cycle_finance_api_gate

echo "Done."
