#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — tmux çok-terminal başlatıcı
#  Kullanım: ./scripts/cycle_tmux.sh [attach|kill|status]
#
#  Layout (5 panel):
#  ┌─────────────────────┬─────────────────────┐
#  │  1: DATA            │  2: PAPER-SERVICE   │
#  ├─────────────────────┼─────────────────────┤
#  │  3: STRATEGY        │  4: ALERT-SERVICE   │
#  ├─────────────────────┴─────────────────────┤
#  │  5: SHELL (genel komut satırı)            │
#  └───────────────────────────────────────────┘
# ============================================================

# ── help-cycle fonksiyonunu geçici dosyaya yaz ─────────────
# (tmux her panel için yeni bash açar, bu dosyayı source eder)
generate_cycle_env() {
  local ROOT="$1"
  local API_ADDR="$2"
  local ADMIN_USER="$3"
  local ADMIN_PASS="$4"

  cat > /tmp/cycle_env.sh << ENVEOF
#!/usr/bin/env bash
# Cycle Finance — shell yardımcıları (otomatik yüklendi)

CYCLE_ROOT="$ROOT"
CYCLE_API="http://$API_ADDR"
CYCLE_USER="$ADMIN_USER"
CYCLE_PASS="$ADMIN_PASS"

# ─────────────────────────────────────────────────────────────
help-cycle() {
  local G="\\033[0;32m"  # yeşil
  local Y="\\033[1;33m"  # sarı
  local C="\\033[0;36m"  # camgöbeği
  local B="\\033[1;34m"  # mavi
  local W="\\033[1;37m"  # beyaz kalın
  local R="\\033[0;31m"  # kırmızı
  local N="\\033[0m"     # reset

  echo ""
  echo -e "\${W}╔══════════════════════════════════════════════════════════════════╗\${N}"
  echo -e "\${W}║           🏛️  CYCLE FINANCE — KOMUT REHBERİ                     ║\${N}"
  echo -e "\${W}╚══════════════════════════════════════════════════════════════════╝\${N}"

  echo -e "\n\${Y}━━━  🔧 SİSTEM YÖNETİMİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${G}cycle-start\${N}          Tüm terminalleri yeniden başlat (tmux)"
  echo -e "  \${G}cycle-kill\${N}           Tüm terminalleri kapat"
  echo -e "  \${G}cycle-status\${N}         Panel durumlarını göster"
  echo -e "  \${G}cycle-build\${N}          Projeyi derle (cargo build)"
  echo -e "  \${G}cycle-build-full\${N}     Tam set derle (paper-service --features full)"

  echo -e "\n\${Y}━━━  📡 DATA TERMİNALİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${C}data-live\${N}            Canlı Binance WS başlat (RUN_MODE=DATA)"
  echo -e "  \${C}data-backtest\${N}        CSV backtest başlat"
  echo -e "  \${C}data-log\${N}             Data terminal logunu izle"

  echo -e "\n\${Y}━━━  🛡️  PAPER SERVICE (REST API)  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${C}paper-health\${N}         Sistem sağlık kontrolü"
  echo -e "  \${C}paper-balance\${N}        Bakiye ve equity bilgisi"
  echo -e "  \${C}paper-positions\${N}      Açık pozisyonlar"
  echo -e "  \${C}paper-orders\${N}         Açık emirler"
  echo -e "  \${C}paper-history\${N}        İşlem geçmişi"
  echo -e "  \${C}paper-metrics\${N}        Prometheus metrikleri"
  echo -e "  \${C}paper-log\${N}            Paper service logunu izle"

  echo -e "\n\${Y}━━━  📋 EMİR İŞLEMLERİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${C}paper-buy  <SYM> <QTY>\${N}   Market BUY emri"
  echo -e "  \${C}paper-sell <SYM> <QTY>\${N}   Market SELL emri"
  echo -e "  \${C}paper-cli  [ARGLAR]\${N}       Paper CLI (tüm seçenekler)"

  echo -e "\n\${Y}━━━  🧠 STRATEGY TERMİNALİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${C}strategy-start\${N}       Strategy terminalini başlat"
  echo -e "  \${C}strategy-log\${N}         Strategy logunu izle (gelecekte)"

  echo -e "\n\${Y}━━━  🔔 ALERT SERVİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${C}alert-list\${N}           Aktif uyarıları listele (alerts.toml)"
  echo -e "  \${C}alert-reload\${N}         Alert servisini yeniden başlat"

  echo -e "\n\${Y}━━━  📈 KORElASYON TERMİNALİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${C}correlation-start\${N}    HEIUSDT korelasyon analizini başlat"

  echo -e "\n\${Y}━━━  📊 İZLEME SERVİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${C}monitor-start\${N}        Servis izleme panelini başlat (Ctrl+B → 1 ile git)"
  echo -e "  \${DIM}  → CPU%, RAM, GPU%, VRAM%, Ring Buffer durumu gösterir\${N}"

  echo -e "\n\${Y}━━━  🗄️  VERİTABANI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${C}db-trades\${N}            Son 20 işlemi göster"
  echo -e "  \${C}db-size\${N}              Veritabanı boyutu"

  echo -e "\n\${Y}━━━  🌐 tmux KISAYOLLARI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${B}Ctrl+B → ok tuşu\${N}     Panel değiştir"
  echo -e "  \${B}Ctrl+B → z\${N}           Paneli tam ekran yap / küçült"
  echo -e "  \${B}Ctrl+B → d\${N}           Session'ı arka plana al"
  echo -e "  \${B}Ctrl+B → c\${N}           Yeni sekme (window) aç"
  echo -e "  \${B}Ctrl+B → 0-9\${N}         Sekmeye geç"
  echo -e "  \${B}Fare tıklama\${N}          Panel seç"
  echo -e "  \${B}Fare kaydırma\${N}         Scroll"

  echo -e "\n\${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\${N}"
  echo -e "  \${R}help-cycle\${N}  yazarak bu listeye tekrar ulaşabilirsin."
  echo ""
}

# ── Sistem yönetimi ────────────────────────────────────────
cycle-start()  { \"\$CYCLE_ROOT/scripts/cycle_tmux.sh\"; }
cycle-kill()   { \"\$CYCLE_ROOT/scripts/cycle_tmux.sh\" kill; }
cycle-status() { \"\$CYCLE_ROOT/scripts/cycle_tmux.sh\" status; }
cycle-build()  { cd \"\$CYCLE_ROOT\" && cargo build -p core -p paper-service -p alert-service; }
cycle-build-full() { cd \"\$CYCLE_ROOT\" && cargo build -p paper-service --features full; }

# ── Data terminali ─────────────────────────────────────────
data-live()     { cd \"\$CYCLE_ROOT\" && RUN_MODE=DATA ./target/debug/core; }
data-backtest() { cd \"\$CYCLE_ROOT\" && RUN_MODE=BACKTEST CSV_PATH=\"./test_data.csv\" ./target/debug/core; }
data-log()      { tail -f /tmp/data_terminal.log; }

# ── Paper Service ──────────────────────────────────────────
_jwt_header() {
  TOKEN=\$(curl -s -X POST \"\$CYCLE_API/api/v1/auth/login\" \\
    -H 'Content-Type: application/json' \\
    -d \"{\\\"username\\\":\\\"\$CYCLE_USER\\\",\\\"password\\\":\\\"\$CYCLE_PASS\\\"}\" \\
    | python3 -c \"import sys,json; print(json.load(sys.stdin)['access_token'])\" 2>/dev/null)
  echo \"Authorization: Bearer \$TOKEN\"
}
paper-health()    { curl -s \"\$CYCLE_API/api/v1/system/health\" | python3 -m json.tool; }
paper-balance()   { curl -s -H \"\$(_jwt_header)\" \"\$CYCLE_API/api/v1/account/balance\" | python3 -m json.tool; }
paper-positions() { curl -s -H \"\$(_jwt_header)\" \"\$CYCLE_API/api/v1/account/positions\" | python3 -m json.tool; }
paper-orders()    { curl -s -H \"\$(_jwt_header)\" \"\$CYCLE_API/api/v1/orders\" | python3 -m json.tool; }
paper-history()   { curl -s -H \"\$(_jwt_header)\" \"\$CYCLE_API/api/v1/account/trade-history\" | python3 -m json.tool; }
paper-metrics()   { curl -s \"\$CYCLE_API/metrics\"; }
paper-log()       { tail -f /tmp/paper_service.log; }

paper-buy() {
  local SYM=\"\${1:-BTCUSDT}\" QTY=\"\${2:-0.001}\"
  local HDR=\"\$(_jwt_header)\"
  curl -s -X POST -H \"\$HDR\" -H 'Content-Type: application/json' \\
    -d \"{\\\"symbol\\\":\\\"\$SYM\\\",\\\"side\\\":\\\"BUY\\\",\\\"order_type\\\":\\\"MARKET\\\",\\\"quantity\\\":\$QTY,\\\"client_order_id\\\":\\\"cli-\$(date +%s)\\\"}\" \\
    \"\$CYCLE_API/api/v1/order\" | python3 -m json.tool
}
paper-sell() {
  local SYM=\"\${1:-BTCUSDT}\" QTY=\"\${2:-0.001}\"
  local HDR=\"\$(_jwt_header)\"
  curl -s -X POST -H \"\$HDR\" -H 'Content-Type: application/json' \\
    -d \"{\\\"symbol\\\":\\\"\$SYM\\\",\\\"side\\\":\\\"SELL\\\",\\\"order_type\\\":\\\"MARKET\\\",\\\"quantity\\\":\$QTY,\\\"client_order_id\\\":\\\"cli-\$(date +%s)\\\"}\" \\
    \"\$CYCLE_API/api/v1/order\" | python3 -m json.tool
}
paper-cli() { \"\$CYCLE_ROOT/target/debug/paper_cli\" --api \"\$CYCLE_API\" --user \"\$CYCLE_USER\" --password \"\$CYCLE_PASS\" \"\$@\"; }

# ── Strategy / Correlation ─────────────────────────────────
strategy-start()    { cd \"\$CYCLE_ROOT\" && RUN_MODE=STRATEGY ./target/debug/core; }
correlation-start() { cd \"\$CYCLE_ROOT\" && RUN_MODE=CORRELATION ./target/debug/core; }

# ── Alert servisi ──────────────────────────────────────────
alert-list()   { grep -A5 '\[\[alerts\]\]' \"\$CYCLE_ROOT/alerts.toml\" | grep -E 'symbol|condition|price'; }
alert-reload() { pkill -x alert-service 2>/dev/null; sleep 1; cd \"\$CYCLE_ROOT\" && ./target/debug/alert-service --config ./alerts.toml &; }

# ── Veritabanı ─────────────────────────────────────────────
db-trades() { sqlite3 \"\$CYCLE_ROOT/market_data.db\" \"SELECT id,symbol,side,entry_price,pnl FROM trades ORDER BY id DESC LIMIT 20\" 2>/dev/null || echo 'DB boş veya bulunamadı.'; }
db-size()   { du -sh \"\$CYCLE_ROOT/market_data.db\" 2>/dev/null; }

# ── İzleme servisi ─────────────────────────────────────────
monitor-start() {
  if tmux has-session -t cycle 2>/dev/null; then
    tmux select-window -t cycle:1 2>/dev/null || {
      tmux new-window -t cycle:1 -n '📊 Monitor'
      tmux send-keys -t cycle:1 \"\$CYCLE_ROOT/scripts/monitor.sh\" Enter
    }
  else
    \"\$CYCLE_ROOT/scripts/monitor.sh\"
  fi
}

export -f help-cycle cycle-start cycle-kill cycle-status cycle-build cycle-build-full
export -f data-live data-backtest data-log
export -f paper-health paper-balance paper-positions paper-orders paper-history paper-metrics paper-log paper-buy paper-sell paper-cli
export -f strategy-start correlation-start
export -f alert-list alert-reload
export -f db-trades db-size monitor-start
ENVEOF

  chmod +x /tmp/cycle_env.sh
}
set -euo pipefail

SESSION="cycle"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# ── Env varsayılanları ─────────────────────────────────────
PAPER_API_ADDR="${PAPER_API_ADDR:-127.0.0.1:8080}"
PAPER_ADMIN_USER="${PAPER_ADMIN_USER:-admin}"
PAPER_ADMIN_PASS="${PAPER_ADMIN_PASS:-changeme123}"
PAPER_INITIAL_USDT="${PAPER_INITIAL_USDT:-100000}"
ALERT_CONFIG="${ALERT_CONFIG:-$ROOT/alerts.toml}"

# ── Alt komutlar ───────────────────────────────────────────
case "${1:-}" in
  kill)
    tmux kill-session -t "$SESSION" 2>/dev/null && echo "✅ Session '$SESSION' kapatıldı." || echo "⚠️  Session bulunamadı."
    exit 0
    ;;
  status)
    tmux list-panes -t "$SESSION" -F "#{pane_index}: #{pane_title} [#{pane_pid}] #{pane_current_command}" 2>/dev/null \
      || echo "⚠️  '$SESSION' session'ı çalışmıyor."
    exit 0
    ;;
  attach)
    tmux attach-session -t "$SESSION" 2>/dev/null || { echo "⚠️  Session yok. Önce scripti başlatın."; exit 1; }
    exit 0
    ;;
esac

# ── Zaten çalışıyorsa bağlan ───────────────────────────────
if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "⚡ '$SESSION' session'ı zaten çalışıyor. Bağlanılıyor..."
  tmux attach-session -t "$SESSION"
  exit 0
fi

# ── Binary'leri derle ─────────────────────────────────────
echo "🔨 Derleniyor..."
cd "$ROOT"
cargo build -p core -p paper-service -p alert-service 2>&1 | tail -5

# ── Eski ring buffer'ları temizle ─────────────────────────
rm -f /dev/shm/demir_yumruk_ring /dev/shm/demir_yumruk_orders

# ── Session oluştur ───────────────────────────────────────
tmux new-session -d -s "$SESSION" -x 220 -y 50

# ── Pencere adı ───────────────────────────────────────────
tmux rename-window -t "$SESSION:0" "Cycle Finance"

# ── Layout: 5 panel ───────────────────────────────────────
# Panel 0 → sol üst  (DATA)
# Panel 1 → sağ üst  (PAPER-SERVICE)
# Panel 2 → sol orta (STRATEGY)
# Panel 3 → sağ orta (ALERT-SERVICE)
# Panel 4 → alt tam  (SHELL)

tmux split-window  -t "$SESSION:0" -h          # 0 sol | 1 sağ
tmux split-window  -t "$SESSION:0.0" -v        # 0 üst | 2 alt (sol)
tmux split-window  -t "$SESSION:0.1" -v        # 1 üst | 3 alt (sağ)
tmux split-window  -t "$SESSION:0" -v -p 20    # alt tam genişlik (shell)

# ── Panel başlıkları ──────────────────────────────────────
tmux select-pane -t "$SESSION:0.0" -T "📡 DATA"
tmux select-pane -t "$SESSION:0.1" -T "🛡️  PAPER"
tmux select-pane -t "$SESSION:0.2" -T "🧠 STRATEGY"
tmux select-pane -t "$SESSION:0.3" -T "🔔 ALERT"
tmux select-pane -t "$SESSION:0.4" -T "💻 SHELL"

# ── Komutları çalıştır ────────────────────────────────────

# Panel 0: DATA terminali
tmux send-keys -t "$SESSION:0.0" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  DATA TERMİNALİ  (Binance WS)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
cd $ROOT && RUN_MODE=DATA ./target/debug/core
" Enter

sleep 1

# Panel 1: PAPER SERVICE
tmux send-keys -t "$SESSION:0.1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛡️   PAPER SERVICE  (REST API)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && \\
  PAPER_ADMIN_USER=$PAPER_ADMIN_USER \\
  PAPER_ADMIN_PASS=$PAPER_ADMIN_PASS \\
  PAPER_API_ADDR=$PAPER_API_ADDR \\
  PAPER_INITIAL_USDT=$PAPER_INITIAL_USDT \\
  PAPER_SLED_PATH=./paper_wal \\
  PAPER_DB_PATH=/tmp/paper_live.db \\
  ./target/debug/paper-service
" Enter

# Panel 2: STRATEGY terminali
tmux send-keys -t "$SESSION:0.2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧠  STRATEGY TERMİNALİ  (PyO3)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $ROOT && RUN_MODE=STRATEGY ./target/debug/core
" Enter

# Panel 3: ALERT SERVICE
tmux send-keys -t "$SESSION:0.3" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔔  ALERT SERVİSİ  (Sesli Uyarı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && ./target/debug/alert-service --config $ALERT_CONFIG
" Enter

# ── cycle_env.sh dosyasını oluştur ───────────────────────
generate_cycle_env "$ROOT" "$PAPER_API_ADDR" "$PAPER_ADMIN_USER" "$PAPER_ADMIN_PASS"

# Panel 4: Shell (genel komut satırı)
tmux send-keys -t "$SESSION:0.4" "source /tmp/cycle_env.sh && cd $ROOT && help-cycle" Enter

# ── Window 1: MONITOR (ayrı sekme) ───────────────────────
tmux new-window -t "$SESSION:1" -n "📊 Monitor"
tmux send-keys -t "$SESSION:1" "
echo ''
echo '📊  SERVİS İZLEME PANELİ BAŞLATIYOR...'
sleep 2
$ROOT/scripts/monitor.sh
" Enter
tmux select-pane -t "$SESSION:1" -T "📊 Monitor"

# ── tmux mouse ve görsel ayarları ────────────────────────
tmux set-option -t "$SESSION" mouse on
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format " #{pane_title} "
tmux set-option -t "$SESSION" status-style "bg=colour235,fg=colour250"
tmux set-option -t "$SESSION" pane-active-border-style "fg=colour39"
tmux set-option -t "$SESSION" pane-border-style "fg=colour238"
tmux set-option -t "$SESSION" status-left " 🏛️  #[bold]Cycle Finance#[nobold] | "
tmux set-option -t "$SESSION" status-right " Ctrl+B → 0:Terminal  1:Monitor | %H:%M:%S "
tmux set-option -t "$SESSION" status-interval 1

# ── Ana pencereye (terminal) dön ve bağlan ───────────────
tmux select-window -t "$SESSION:0"
tmux select-pane -t "$SESSION:0.4"
tmux attach-session -t "$SESSION"

