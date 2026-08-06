#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — tmux çok-terminal başlatıcı
#  Kullanım: ./scripts/cycle_tmux.sh [attach|kill|status]
#
#  Pencere 0 — Trading (5 panel):
#    ┌──────────────────────┬──────────────────────┐
#    │  📡 DATA             │  🛡️  PAPER-SERVICE    │
#    ├──────────────────────┼──────────────────────┤
#    │  🧠 STRATEGY         │  🔔 ALERT-SERVICE    │
#    ├──────────────────────┴──────────────────────┤
#    │  💻 SHELL  (help-cycle, paper-buy, ...)     │
#    └─────────────────────────────────────────────┘
#  Pencere 1 — Monitor (CPU/RAM/GPU izleme)
# ============================================================
set -euo pipefail

SESSION="cycle"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

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
    for proc in core paper-service alert-service; do
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
cargo build -p core -p paper-service -p alert-service 2>&1 | tail -5

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
rm -f /dev/shm/demir_yumruk_ring /dev/shm/demir_yumruk_orders
echo "  ✔ Ring buffer'lar temizlendi"
sleep 1

# ── Session oluştur ──────────────────────────────────────────
tmux new-session -d -s "$SESSION" -x 220 -y 50
tmux rename-window -t "$SESSION:0" "Trading"

# ── Panel düzeni ─────────────────────────────────────────────
# 0=sol-üst  1=sağ-üst  2=sol-orta  3=sağ-orta  4=alt-tam
tmux split-window -t "$SESSION:0"    -h
tmux split-window -t "$SESSION:0.0"  -v
tmux split-window -t "$SESSION:0.1"  -v
tmux split-window -t "$SESSION:0"    -v -p 20

# ── Panel başlıkları ─────────────────────────────────────────
tmux select-pane -t "$SESSION:0.0" -T "📡 DATA"
tmux select-pane -t "$SESSION:0.1" -T "🛡️  PAPER"
tmux select-pane -t "$SESSION:0.2" -T "🧠 STRATEGY"
tmux select-pane -t "$SESSION:0.3" -T "🔔 ALERT"
tmux select-pane -t "$SESSION:0.4" -T "💻 SHELL"

# ── Panel 0: DATA ────────────────────────────────────────────
tmux send-keys -t "$SESSION:0.0" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  DATA TERMİNALİ  (Binance WS)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
cd $ROOT && RUN_MODE=DATA ./target/debug/core
" Enter

# ── Panel 1: PAPER-SERVICE ───────────────────────────────────
tmux send-keys -t "$SESSION:0.1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛡️   PAPER SERVICE  (REST API :8080)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && \
  PAPER_ADMIN_USER=$PAPER_ADMIN_USER \
  PAPER_ADMIN_PASS=$PAPER_ADMIN_PASS \
  PAPER_API_ADDR=$PAPER_API_ADDR \
  PAPER_INITIAL_USDT=$PAPER_INITIAL_USDT \
  PAPER_SLED_PATH=./paper_wal \
  PAPER_DB_PATH=/tmp/paper_live.db \
  ./target/debug/paper-service
" Enter

# ── Panel 2: STRATEGY ────────────────────────────────────────
tmux send-keys -t "$SESSION:0.2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧠  STRATEGY TERMİNALİ  (PyO3)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $ROOT && RUN_MODE=STRATEGY ./target/debug/core
" Enter

# ── Panel 3: ALERT-SERVICE ───────────────────────────────────
tmux send-keys -t "$SESSION:0.3" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔔  ALERT SERVİSİ  (Sesli Uyarı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && ./target/debug/alert-service --config $ALERT_CONFIG
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
source '$ROOT/scripts/cycle_env.sh'
help-cycle
INITEOF
chmod +x /tmp/cycle_init.sh

# ── Panel 4: SHELL ───────────────────────────────────────────
tmux send-keys -t "$SESSION:0.4" "source /tmp/cycle_init.sh" Enter

# ── Pencere 1: MONITOR ───────────────────────────────────────
tmux new-window -t "$SESSION:1" -n "Monitor"
tmux send-keys -t "$SESSION:1" "bash '$ROOT/scripts/monitor.sh'" Enter
tmux select-pane -t "$SESSION:1" -T "Monitor"

# ── Görsel ayarlar ───────────────────────────────────────────
tmux set-option -t "$SESSION" mouse on
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format " #{pane_title} "
tmux set-option -t "$SESSION" status-style "bg=colour235,fg=colour250"
tmux set-option -t "$SESSION" pane-active-border-style "fg=colour39"
tmux set-option -t "$SESSION" pane-border-style "fg=colour238"
tmux set-option -t "$SESSION" status-left " 🏛️  #[bold]Cycle Finance#[nobold] | "
tmux set-option -t "$SESSION" status-right " Ctrl+B→0:Terminal  1:Monitor | %H:%M:%S "
tmux set-option -t "$SESSION" status-interval 1

# ── Terminal penceresine dön ve bağlan ───────────────────────
tmux select-window -t "$SESSION:0"
tmux select-pane  -t "$SESSION:0.4"
tmux attach-session -t "$SESSION"
