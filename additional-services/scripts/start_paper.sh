#!/usr/bin/env bash
# PAPER sistemi tek komutla başlatma.
#   DATA terminal (Binance Futures → tick ring) + paper-service (API + actor)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# ── Binary dizini: varsayılan release; debug için BIN_DIR=./target/debug ver ──
BIN="${BIN_DIR:-$ROOT/target/release}"
BUILD_ARGS=""
case "$BIN" in
  *release*) BUILD_ARGS="--release" ;;
esac

API_ADDR="${PAPER_API_ADDR:-127.0.0.1:8080}"
ADMIN_USER="${PAPER_ADMIN_USER:-admin}"
ADMIN_PASS="${PAPER_ADMIN_PASS:-changeme123}"
INITIAL_USDT="${PAPER_INITIAL_USDT:-10000}"

echo "=== Derleniyor... ==="
cargo build $BUILD_ARGS -p core -p paper-service

echo "=== Eski süreçler kapatılıyor (varsa) ==="
pkill -x core 2>/dev/null || true
pkill -x paper-service 2>/dev/null || true
pkill -x paper_cli 2>/dev/null || true
sleep 1

# Tick ring'i temizle (farklı kapasiteyle başlatılırsa)
rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders

echo "=== DATA terminali başlatılıyor (Binance Futures WS) ==="
setsid env RUN_MODE=DATA "$BIN/core" > /tmp/data_terminal.log 2>&1 < /dev/null &
disown

echo "=== paper-service başlatılıyor (REST API + Actor) ==="
rm -rf data-engine/data/paper_wal
setsid env \
  PAPER_ADMIN_USER="$ADMIN_USER" \
  PAPER_ADMIN_PASS="$ADMIN_PASS" \
  PAPER_API_ADDR="$API_ADDR" \
  PAPER_INITIAL_USDT="$INITIAL_USDT" \
  PAPER_DB_PATH=./data-engine/data/paper_live.db \
  PAPER_SLED_PATH=./data-engine/data/paper_wal \
  "$BIN/paper-service" > /tmp/paper_service.log 2>&1 < /dev/null &
disown

echo "=== Süreçler başlatılıyor... ==="
sleep 4

echo ""
echo "✅ PAPER SİSTEMİ ÇALIŞIYOR"
echo "=============================================="
echo "REST API      : http://$API_ADDR/api/v1/system/health"
echo "Metrikler     : http://$API_ADDR/metrics"
echo "Giriş         : user=$ADMIN_USER pass=$ADMIN_PASS"
echo ""
echo "Kontrol (fiyat geliyor mu):"
echo "  curl -s http://$API_ADDR/api/v1/system/health"
echo ""
echo "CLI örnekleri:"
echo "  $BIN/paper_cli --api http://$API_ADDR --user $ADMIN_USER --password $ADMIN_PASS status"
echo "  $BIN/paper_cli --api http://$API_ADDR --user $ADMIN_USER --password $ADMIN_PASS order --symbol BTCUSDT --side BUY --order-type MARKET --qty 0.001"
echo ""
echo "Loglar: /tmp/data_terminal.log , /tmp/paper_service.log"
echo "Kapatmak için: ./scripts/stop_paper.sh"
