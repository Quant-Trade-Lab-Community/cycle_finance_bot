#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — Shell Yardımcı Komutları
#  Bu dosya cycle_tmux.sh tarafından otomatik source edilir.
#  Elle de kullanılabilir: source ~/Desktop/PROJE/scripts/cycle_env.sh
# ============================================================

# ── Kök dizini otomatik bul ──────────────────────────────────
CYCLE_ROOT="${CYCLE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CYCLE_API="${CYCLE_API:-http://127.0.0.1:8080}"
CYCLE_USER="${CYCLE_USER:-admin}"
CYCLE_PASS="${CYCLE_PASS:-changeme123}"

# ── Renk kodları ─────────────────────────────────────────────
_G='\033[0;32m'; _Y='\033[1;33m'; _C='\033[0;36m'
_B='\033[1;34m'; _W='\033[1;37m'; _R='\033[0;31m'
_D='\033[2m';    _N='\033[0m'

# ============================================================
#  KOMUT REHBERİ
# ============================================================
help-cycle() {
  echo ""
  echo -e "${_W}╔══════════════════════════════════════════════════════════════════╗${_N}"
  echo -e "${_W}║        🏛️  CYCLE FINANCE — KOMUT REHBERİ                        ║${_N}"
  echo -e "${_W}╚══════════════════════════════════════════════════════════════════╝${_N}"

  echo -e "\n${_Y}━━━  🔧 SİSTEM YÖNETİMİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_G}cycle-start${_N}          Tüm terminalleri yeniden başlat"
  echo -e "  ${_G}cycle-kill${_N}           Tüm terminalleri ve servisleri kapat"
  echo -e "  ${_G}cycle-status${_N}         Çalışan servislerin CPU/RAM durumu"
  echo -e "  ${_G}cycle-build${_N}          Projeyi derle (cargo build)"
  echo -e "  ${_G}cycle-build-full${_N}     Tam set derle (--features full)"

  echo -e "\n${_Y}━━━  📡 DATA TERMİNALİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}data-live${_N}            Canlı Binance WS başlat (RUN_MODE=DATA)"
  echo -e "  ${_C}data-backtest${_N}        CSV backtest başlat"
  echo -e "  ${_C}data-log${_N}             Data terminal logunu izle"

  echo -e "\n${_Y}━━━  🛡️  PAPER SERVICE (REST API)  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}paper-health${_N}         Sistem sağlık kontrolü"
  echo -e "  ${_C}paper-balance${_N}        Bakiye ve equity bilgisi"
  echo -e "  ${_C}paper-positions${_N}      Açık pozisyonlar"
  echo -e "  ${_C}paper-orders${_N}         Açık emirler"
  echo -e "  ${_C}paper-history${_N}        İşlem geçmişi"
  echo -e "  ${_C}paper-metrics${_N}        Prometheus metrikleri (ham)"
  echo -e "  ${_C}paper-log${_N}            Paper service logunu izle"

  echo -e "\n${_Y}━━━  📋 EMİR İŞLEMLERİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}paper-buy  BTCUSDT 0.001${_N}   Market BUY emri"
  echo -e "  ${_C}paper-sell BTCUSDT 0.001${_N}   Market SELL emri"
  echo -e "  ${_C}paper-cli  [arglar]${_N}         Paper CLI (tüm seçenekler)"

  echo -e "\n${_Y}━━━  🧠 STRATEGY / CORRELATION  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}strategy-start${_N}       Strategy terminalini başlat"
  echo -e "  ${_C}correlation-start${_N}    Korelasyon analizini başlat"

  echo -e "\n${_Y}━━━  🔔 ALERT SERVİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}alert-list${_N}           Aktif uyarıları listele"
  echo -e "  ${_C}alert-reload${_N}         Alert servisini yeniden başlat"

  echo -e "\n${_Y}━━━  📊 İZLEME  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}monitor-start${_N}        İzleme paneline geç (Ctrl+B → 1)"

  echo -e "\n${_Y}━━━  🗄️  VERİTABANI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}db-trades${_N}            Son 20 işlemi göster"
  echo -e "  ${_C}db-size${_N}              Veritabanı boyutu"

  echo -e "\n${_Y}━━━  🌐 TMUX KISAYOLLARI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_B}Ctrl+B → ok tuşu${_N}     Panel değiştir"
  echo -e "  ${_B}Ctrl+B → z${_N}           Paneli tam ekran yap / küçült"
  echo -e "  ${_B}Ctrl+B → d${_N}           Session'ı arka plana al"
  echo -e "  ${_B}Ctrl+B → 0${_N}           Terminal sekmesi"
  echo -e "  ${_B}Ctrl+B → 1${_N}           Monitor sekmesi"
  echo -e "  ${_B}Fare tıklama/scroll${_N}  Panel seç / scroll"

  echo -e "\n${_W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_D}help-cycle yazarak bu listeye tekrar ulaşabilirsin.${_N}"
  echo ""
}

# ============================================================
#  SİSTEM YÖNETİMİ
# ============================================================
cycle-start() {
  "$CYCLE_ROOT/scripts/cycle_tmux.sh"
}
cycle-kill() {
  "$CYCLE_ROOT/scripts/cycle_tmux.sh" kill
}
cycle-status() {
  "$CYCLE_ROOT/scripts/cycle_tmux.sh" status
}
cycle-build() {
  cd "$CYCLE_ROOT" && cargo build -p core -p paper-service -p alert-service
}
cycle-build-full() {
  cd "$CYCLE_ROOT" && cargo build -p paper-service --features full
}

# ============================================================
#  DATA TERMİNALİ
# ============================================================
data-live() {
  cd "$CYCLE_ROOT" && RUN_MODE=DATA ./target/debug/core
}
data-backtest() {
  cd "$CYCLE_ROOT" && RUN_MODE=BACKTEST CSV_PATH="./test_data.csv" ./target/debug/core
}
data-log() {
  tail -f /tmp/data_terminal.log
}

# ============================================================
#  PAPER SERVICE — JWT otomatik alınır
# ============================================================
_cycle_token() {
  curl -s -X POST "$CYCLE_API/api/v1/auth/login" \
    -H 'Content-Type: application/json' \
    -d "{\"username\":\"$CYCLE_USER\",\"password\":\"$CYCLE_PASS\"}" \
    2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('access_token',''))" 2>/dev/null
}

paper-health() {
  curl -s "$CYCLE_API/api/v1/system/health" | python3 -m json.tool
}
paper-balance() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/balance" | python3 -m json.tool
}
paper-positions() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/positions" | python3 -m json.tool
}
paper-orders() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/orders" | python3 -m json.tool
}
paper-history() {
  local tok; tok=$(_cycle_token)
  curl -s -H "Authorization: Bearer $tok" "$CYCLE_API/api/v1/account/trade-history" | python3 -m json.tool
}
paper-metrics() {
  curl -s "$CYCLE_API/metrics"
}
paper-log() {
  tail -f /tmp/paper_service.log
}

paper-buy() {
  local sym="${1:-BTCUSDT}" qty="${2:-0.001}"
  local tok; tok=$(_cycle_token)
  local oid="cli-$(date +%s)"
  curl -s -X POST \
    -H "Authorization: Bearer $tok" \
    -H 'Content-Type: application/json' \
    -d "{\"symbol\":\"$sym\",\"side\":\"BUY\",\"order_type\":\"MARKET\",\"quantity\":$qty,\"client_order_id\":\"$oid\"}" \
    "$CYCLE_API/api/v1/order" | python3 -m json.tool
}
paper-sell() {
  local sym="${1:-BTCUSDT}" qty="${2:-0.001}"
  local tok; tok=$(_cycle_token)
  local oid="cli-$(date +%s)"
  curl -s -X POST \
    -H "Authorization: Bearer $tok" \
    -H 'Content-Type: application/json' \
    -d "{\"symbol\":\"$sym\",\"side\":\"SELL\",\"order_type\":\"MARKET\",\"quantity\":$qty,\"client_order_id\":\"$oid\"}" \
    "$CYCLE_API/api/v1/order" | python3 -m json.tool
}
paper-cli() {
  "$CYCLE_ROOT/target/debug/paper_cli" \
    --api "$CYCLE_API" --user "$CYCLE_USER" --password "$CYCLE_PASS" "$@"
}

# ============================================================
#  STRATEGY / CORRELATION
# ============================================================
strategy-start() {
  cd "$CYCLE_ROOT" && RUN_MODE=STRATEGY ./target/debug/core
}
correlation-start() {
  cd "$CYCLE_ROOT" && RUN_MODE=CORRELATION ./target/debug/core
}

# ============================================================
#  ALERT SERVİSİ
# ============================================================
alert-list() {
  echo "=== alerts.toml ==="
  grep -E '^\s*(symbol|condition|price|voice)\s*=' "$CYCLE_ROOT/alerts.toml" | sed 's/^/  /'
}
alert-reload() {
  pkill -x alert-service 2>/dev/null || true
  sleep 1
  cd "$CYCLE_ROOT" && nohup ./target/debug/alert-service --config ./alerts.toml > /tmp/alert_service.log 2>&1 &
  echo "✅ Alert servisi yeniden başlatıldı (pid: $!)"
}

# ============================================================
#  İZLEME
# ============================================================
monitor-start() {
  if tmux has-session -t cycle 2>/dev/null; then
    tmux select-window -t cycle:1
  else
    "$CYCLE_ROOT/scripts/monitor.sh"
  fi
}

# ============================================================
#  VERİTABANI
# ============================================================
db-trades() {
  sqlite3 "$CYCLE_ROOT/market_data.db" \
    "SELECT id,symbol,side,entry_price,exit_price,pnl FROM trades ORDER BY id DESC LIMIT 20;" \
    2>/dev/null || echo "DB boş veya bulunamadı."
}
db-size() {
  du -sh "$CYCLE_ROOT/market_data.db" 2>/dev/null
}

# ── Yüklendiğini bildir ──────────────────────────────────────
echo -e "${_D}[cycle_env] Yüklendi — ROOT: $CYCLE_ROOT | API: $CYCLE_API${_N}"
