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
cargo build $BUILD_ARGS -p flows -p paper-service

echo "=== Eski süreçler kapatılıyor (varsa) ==="
for p in flow-trade flow-depth flow-liquidation flow-oi flow-funding flow-markprice flow-lastprice flow-indexprice paper-service paper_cli; do
  pkill -x "$p" 2>/dev/null || true
done
sleep 1

# Akış ring'lerini ve rate kapısını temizle
rm -f /dev/shm/cycle_finance_trades /dev/shm/cycle_finance_depth /dev/shm/cycle_finance_liquidations /dev/shm/cycle_finance_open_interest /dev/shm/cycle_finance_funding /dev/shm/cycle_finance_markprice /dev/shm/cycle_finance_lastprice /dev/shm/cycle_finance_indexprice /dev/shm/cycle_finance_api_gate

echo "=== Veri akışları başlatılıyor (WS → parse → ring → TimescaleDB) ==="
FLOWS="flow-trade flow-depth flow-liquidation flow-oi flow-funding flow-markprice flow-lastprice flow-indexprice"
for f in $FLOWS; do
  setsid env "$BIN/$f" > "/tmp/${f}.log" 2>&1 < /dev/null &
  disown
done

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
echo "Loglar: /tmp/flow-*.log , /tmp/paper_service.log"
echo "Kapatmak için: ./scripts/stop_paper.sh"
