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

# Panel 4: Shell (genel komut satırı)
tmux send-keys -t "$SESSION:0.4" "
cd $ROOT
echo ''
echo '💻  GENEL KOMUT SATIRI'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo \"  REST API  : http://$PAPER_API_ADDR/api/v1/system/health\"
echo \"  Metrikler : http://$PAPER_API_ADDR/metrics\"
echo \"  Giriş     : user=$PAPER_ADMIN_USER  pass=$PAPER_ADMIN_PASS\"
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo ''
echo 'Yararlı komutlar:'
echo \"  curl -s http://$PAPER_API_ADDR/api/v1/system/health | python3 -m json.tool\"
echo \"  ./target/debug/paper_cli --api http://$PAPER_API_ADDR --user $PAPER_ADMIN_USER --password $PAPER_ADMIN_PASS status\"
echo \"  ./scripts/cycle_tmux.sh status   # Panel durumları\"
echo \"  ./scripts/cycle_tmux.sh kill     # Tümünü kapat\"
echo ''
" Enter

# ── tmux mouse ve görsel ayarları ────────────────────────
tmux set-option -t "$SESSION" mouse on
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format " #{pane_title} "
tmux set-option -t "$SESSION" status-style "bg=colour235,fg=colour250"
tmux set-option -t "$SESSION" pane-active-border-style "fg=colour39"
tmux set-option -t "$SESSION" pane-border-style "fg=colour238"
tmux set-option -t "$SESSION" status-left " 🏛️  #[bold]Cycle Finance#[nobold] | "
tmux set-option -t "$SESSION" status-right " %H:%M:%S | %d.%m.%Y "
tmux set-option -t "$SESSION" status-interval 1

# ── Shell paneline odaklan ve bağlan ─────────────────────
tmux select-pane -t "$SESSION:0.4"
tmux attach-session -t "$SESSION"
