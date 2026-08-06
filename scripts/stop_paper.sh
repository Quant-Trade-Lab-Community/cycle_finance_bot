#!/usr/bin/env bash
# PAPER sistemini kapatır (DATA + paper-service).
set -euo pipefail

echo "=== PAPER sistemi kapatılıyor ==="
pkill -x paper-service 2>/dev/null && echo "  paper-service durduruldu" || echo "  paper-service zaten kapalı"
pkill -x core 2>/dev/null && echo "  DATA terminal durduruldu" || echo "  DATA terminal zaten kapalı"

# Paylaşımlı hafıza temizliği
rm -f /dev/shm/demir_yumruk_ring /dev/shm/demir_yumruk_orders

echo "Done."
