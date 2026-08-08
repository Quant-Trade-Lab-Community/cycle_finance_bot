#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — tmux çok-terminal başlatıcı
#  Kullanım: ./scripts/cycle_tmux.sh [attach|kill|status]
#
#  Pencere 0 — Trading (4 panel):
#    ┌──────────────────────┬──────────────────────┐
#    │  🧠 STRATEGY          │  🛰️  LISTENER        │
#    ├──────────────────────┼──────────────────────┤
#    │  ⚠️  RISK             │  💻 SHELL            │
#    └──────────────────────┴──────────────────────┘
#  Pencere 1 — 📡 DATA   (sekme terminal)
#  Pencere 2 — 🔔 ALERT  (sekme terminal)
#  Pencere 3 — 🛡️ PAPER (sekme terminal)
#  Pencere 4 — Monitor  (CPU/RAM/GPU izleme)
#  Pencere 5 — DETECT-MS (MSMP :3002)
#  Pencere 6 — HEIUSDT (Kırılım stratejisi)
#  Pencere 7 — WYCKOFF (:3005)
#  Pencere 8 — TURBULANS/DETECT-TRB (:3006)
#  Pencere 9 — SCOUT (Binance USDT tarayıcı → /dev/shm/demir_yumruk_scout)
# ============================================================
set -euo pipefail

SESSION="cycle"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

# ── Binary dizini: varsayılan release; debug için BIN_DIR=./target/debug ver ──
BIN="${BIN_DIR:-$ROOT/target/release}"
BUILD_ARGS=""
case "$BIN" in
  *release*) BUILD_ARGS="--release" ;;
esac

# ── Env varsayılanları ───────────────────────────────────────
PAPER_API_ADDR="${PAPER_API_ADDR:-127.0.0.1:8080}"
PAPER_ADMIN_USER="${PAPER_ADMIN_USER:-admin}"
PAPER_ADMIN_PASS="${PAPER_ADMIN_PASS:-changeme123}"
PAPER_INITIAL_USDT="${PAPER_INITIAL_USDT:-100000}"
ALERT_CONFIG="${ALERT_CONFIG:-$ROOT/alerts.toml}"

# ── Tam temizlik fonksiyonu ──────────────────────────────────
full_cleanup() {
  echo "🧹 Temizleniyor..."
  tmux kill-session -t "$SESSION" 2>/dev/null && echo "  ✔ tmux session kapatıldı" || echo "  - tmux session yoktu"
  for proc in core paper-service alert-service; do
    if pgrep -x "$proc" &>/dev/null; then
      pkill -TERM -x "$proc" 2>/dev/null || true
      sleep 0.5
      pkill -KILL -x "$proc" 2>/dev/null || true
      echo "  ✔ $proc durduruldu"
    fi
  done
  for f in /dev/shm/demir_yumruk_ring /dev/shm/demir_yumruk_orders; do
    [ -f "$f" ] && rm -f "$f" && echo "  ✔ $f silindi" || true
  done
  echo "✅ Temizlik tamamlandı."
}

# ── Alt komutlar ─────────────────────────────────────────────
case "${1:-}" in
  kill)
    full_cleanup
    exit 0
    ;;
  status)
    echo "=== tmux Panelleri ==="
    tmux list-panes -t "$SESSION" -F "  #{pane_index}: #{pane_title} [pid:#{pane_pid}] #{pane_current_command}" 2>/dev/null \
      || echo "  ⚠️  '$SESSION' session'ı çalışmıyor."
    echo ""
    echo "=== Çalışan Servisler ==="
for proc in core paper-service alert-service scout-service; do
      pid=$(pgrep -x "$proc" 2>/dev/null | head -1 || true)
      if [ -n "$pid" ]; then
        mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
        cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
        echo "  ✔ $proc  [pid:$pid]  CPU:${cpu}%  RAM:${mem}"
      else
        echo "  ✘ $proc  (durdurulmuş)"
      fi
    done
    exit 0
    ;;
  attach)
    tmux attach-session -t "$SESSION" 2>/dev/null || { echo "⚠️  Session yok."; exit 1; }
    exit 0
    ;;
esac

# ── Zaten çalışıyorsa bağlan ─────────────────────────────────
if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "⚡ '$SESSION' zaten çalışıyor. Bağlanılıyor..."
  tmux attach-session -t "$SESSION"
  exit 0
fi

# ── Derleme ──────────────────────────────────────────────────
echo "🔨 Derleniyor..."
cd "$ROOT"
cargo build $BUILD_ARGS -p core -p paper-service -p alert-service -p scout-service 2>&1 | tail -5

# ── Eski süreçleri ve ring buffer'ları temizle ───────────────
echo "🧹 Eski süreçler temizleniyor..."
for proc in core paper-service alert-service; do
  if pgrep -x "$proc" &>/dev/null; then
    pkill -TERM -x "$proc" 2>/dev/null || true
    sleep 0.3
    pkill -KILL -x "$proc" 2>/dev/null || true
    echo "  ✔ $proc durduruldu"
  fi
done
rm -f /dev/shm/demir_yumruk_ring /dev/shm/demir_yumruk_orders /dev/shm/demir_yumruk_scout
echo "  ✔ Ring buffer'lar temizlendi"
sleep 1

# ── Session oluştur ──────────────────────────────────────────
tmux new-session -d -s "$SESSION" -x 220 -y 50
tmux rename-window -t "$SESSION:0" "Trading"

# ── Panel düzeni ─────────────────────────────────────────────
# 0=sol-üst(STRATEGY)  2=sağ-üst(LISTENER)
# 1=sol-alt(RISK)      3=sağ-alt(SHELL)
tmux split-window -t "$SESSION:0"    -h
tmux split-window -t "$SESSION:0.0"  -v
tmux split-window -t "$SESSION:0.2"  -v

# ── Panel başlıkları ─────────────────────────────────────────
tmux select-pane -t "$SESSION:0.0" -T "🧠 STRATEGY"
tmux select-pane -t "$SESSION:0.2" -T "🛰️  LISTENER"
tmux select-pane -t "$SESSION:0.1" -T "⚠️  RISK"
tmux select-pane -t "$SESSION:0.3" -T "💻 SHELL"

# ── Panel 0: STRATEGY ────────────────────────────────────────
tmux send-keys -t "$SESSION:0.0" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧠  STRATEGY TERMİNALİ  (PyO3)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $ROOT && RUN_MODE=STRATEGY $BIN/core
" Enter

# ── Panel 2: LISTENER ─────────────────────────────────────────
tmux send-keys -t "$SESSION:0.2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛰️   LISTENER  (Anlık Metrik Analizi)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/listener
" Enter

# ── Panel 1: RISK ─────────────────────────────────────────────
tmux send-keys -t "$SESSION:0.1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '⚠️   RİSK ANALİZİ  (market_data.db)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/risk_analysis --watch
" Enter

# ── Panel 3: SHELL ───────────────────────────────────────────
tmux send-keys -t "$SESSION:0.3" "source /tmp/cycle_init.sh" Enter

# ── Pencere 1: DATA (sekme terminal) ─────────────────────────
tmux new-window -t "$SESSION:1" -n "📡 DATA"
tmux send-keys -t "$SESSION:1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  DATA TERMİNALİ  (Binance WS)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
cd $ROOT && RUN_MODE=DATA $BIN/core
" Enter

# ── Pencere 2: ALERT (sekme terminal) ────────────────────────
tmux new-window -t "$SESSION:2" -n "🔔 ALERT"
tmux send-keys -t "$SESSION:2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔔  ALERT SERVİSİ  (Sesli Uyarı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/alert-service --config $ALERT_CONFIG
" Enter

# ── Pencere 3: PAPER (sekme terminal) ────────────────────────
tmux new-window -t "$SESSION:3" -n "🛡️ PAPER"
tmux send-keys -t "$SESSION:3" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛡️   PAPER SERVICE  (REST API :8080)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && \
  PAPER_ADMIN_USER=$PAPER_ADMIN_USER \
  PAPER_ADMIN_PASS=$PAPER_ADMIN_PASS \
  PAPER_API_ADDR=$PAPER_API_ADDR \
  PAPER_INITIAL_USDT=$PAPER_INITIAL_USDT \
  PAPER_SLED_PATH=./data-engine/data/paper_wal \
  PAPER_DB_PATH=./data-engine/data/paper_live.db \
  $BIN/paper-service
" Enter

# ── Shell init dosyasını oluştur ────────────────────────────
# (tmux send-keys ile çok satırlı komut göndermek güvensiz;
#  bunun yerine önce dosyaya yaz, shell paneli source eder)
cat > /tmp/cycle_init.sh << INITEOF
#!/usr/bin/env bash
export CYCLE_ROOT='$ROOT'
export CYCLE_API='http://$PAPER_API_ADDR'
export CYCLE_USER='$PAPER_ADMIN_USER'
export CYCLE_PASS='$PAPER_ADMIN_PASS'
source '$ROOT/additional-services/scripts/cycle_env.sh'
help-cycle
INITEOF
chmod +x /tmp/cycle_init.sh

# ── Pencere 4: MONITOR ───────────────────────────────────────
tmux new-window -t "$SESSION:4" -n "Monitor"
tmux send-keys -t "$SESSION:4" "bash '$ROOT/additional-services/scripts/monitor.sh'" Enter
tmux select-pane -t "$SESSION:4" -T "Monitor"

# ── Pencere 5: DETECT-MS ─────────────────────────────────────
tmux new-window -t "$SESSION:5" -n "DETECT-MS"
tmux send-keys -t "$SESSION:5" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📈  DETECT-MS  (MSMP 2.0 :3002)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-ms
" Enter

# ── Pencere 6: HEIUSDT STRATEJİ ─────────────────────────────
tmux new-window -t "$SESSION:6" -n "HEIUSDT"
tmux send-keys -t "$SESSION:6" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🎯  HEIUSDT  (Kırılım Stratejisi)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/heiusdt
" Enter

# ── Pencere 7: WYCKOFF ANALİZ ───────────────────────────────
tmux new-window -t "$SESSION:7" -n "WYCKOFF"
tmux send-keys -t "$SESSION:7" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🏛️  DETECT-WYCKOFF  (Wyckoff :3005)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-wyckoff
" Enter

# ── Pencere 8: DETECT-TRB (Navier-Stokes) ────────────────────
tmux new-window -t "$SESSION:8" -n "TURBULANS"
tmux send-keys -t "$SESSION:8" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🌊  DETECT-TRB  (Navier-Stokes :3006)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-trb
" Enter

# ── Pencere 9: SCOUT (Binance USDT tarayıcı) ────────────────
tmux new-window -t "$SESSION:9" -n "SCOUT"
tmux send-keys -t "$SESSION:9" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔭  SCOUT  (Binance USDT tarayıcı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo 'Fırsat + sembol metrikleri → /dev/shm/demir_yumruk_scout'
echo 'Tüketici: ./target/debug/probe --once'
sleep 2
cd $ROOT && $BIN/scout-service
" Enter

# ── Görsel ayarlar (global) ──────────────────────────────────
tmux set-option -t "$SESSION" mouse on
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format " #{pane_title} "
tmux set-option -t "$SESSION" status-interval 1

# Status bar — koyu tema
tmux set-option -t "$SESSION" status-style          "bg=colour232,fg=colour245"
tmux set-option -t "$SESSION" status-left           "#[bg=colour25,fg=colour255,bold]  🏛️  Cycle Finance  #[bg=colour232,fg=colour245] "
tmux set-option -t "$SESSION" status-left-length    30
tmux set-option -t "$SESSION" status-right          "#[fg=colour39]0#[fg=colour244]:Trading #[fg=colour45]1#[fg=colour244]:DATA #[fg=colour214]2#[fg=colour244]:ALERT #[fg=colour82]3#[fg=colour244]:PAPER #[fg=colour196]4#[fg=colour244]:Mon #[fg=colour171]7#[fg=colour244]:WYCKOFF #[fg=colour51]8#[fg=colour244]:TRB #[fg=colour250]%H:%M:%S"
tmux set-option -t "$SESSION" status-right-length   80

# Window sekme renkleri
tmux set-option -t "$SESSION" window-status-format          "#[fg=colour240] #{window_index}:#{window_name} "
tmux set-option -t "$SESSION" window-status-current-format  "#[bg=colour25,fg=colour255,bold] #{window_index}:#{window_name} "

# ── Per-pane renk temaları ───────────────────────────────────
# 🧠 STRATEGY  → Mor tema     (bg: koyu mor   | kenarlık: parlak magenta)
tmux select-pane -t "$SESSION:0.0" -P "bg=colour53,fg=colour255"
tmux set-option -t "$SESSION:0.0" -p pane-active-border-style "fg=colour171,bold"
tmux set-option -t "$SESSION:0.0" -p pane-border-style        "fg=colour55"

# 🛰️  LISTENER   → Camgöbeği tema (bg: koyu turkuaz | kenarlık: cyan)
tmux select-pane -t "$SESSION:0.2" -P "bg=colour23,fg=colour255"
tmux set-option -t "$SESSION:0.2" -p pane-active-border-style "fg=colour45,bold"
tmux set-option -t "$SESSION:0.2" -p pane-border-style        "fg=colour36"

# ⚠️  RISK       → Kırmızı tema  (bg: koyu bordo | kenarlık: kırmızı)
tmux select-pane -t "$SESSION:0.1" -P "bg=colour52,fg=colour255"
tmux set-option -t "$SESSION:0.1" -p pane-active-border-style "fg=colour196,bold"
tmux set-option -t "$SESSION:0.1" -p pane-border-style        "fg=colour124"

# 💻 SHELL     → Antrasit tema (bg: çok koyu  | kenarlık: açık gri)
tmux select-pane -t "$SESSION:0.3" -P "bg=colour233,fg=colour252"
tmux set-option -t "$SESSION:0.3" -p pane-active-border-style "fg=colour244,bold"
tmux set-option -t "$SESSION:0.3" -p pane-border-style        "fg=colour238"

# Pane başlık formatı — renk kodlu
tmux set-option -t "$SESSION:0" pane-border-format \
  "#{?#{==:#{pane_index},0},#[fg=colour171 bold],#{?#{==:#{pane_index},1},#[fg=colour196 bold],#{?#{==:#{pane_index},2},#[fg=colour45 bold],#{?#{==:#{pane_index},3},#[fg=colour244 bold],#[fg=colour244 bold]}}}}} #{pane_title} #[default]"

# ── Terminal penceresine dön ve bağlan ───────────────────────
tmux select-window -t "$SESSION:0"
tmux select-pane  -t "$SESSION:0.3"
tmux attach-session -t "$SESSION"
