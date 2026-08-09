#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — tmux tek-sekme başlatıcı
#  Kullanım: ./scripts/cycle_tmux.sh [attach|kill|status]
#
#  Her servis tek sekmede (window) çalışır:
#  Pencere 0 — 🧠 STRATEGY
#  Pencere 1 — 🛰️  LISTENER
#  Pencere 2 — ⚠️  RISK
#  Pencere 3 — 💻 SHELL
#  Pencere 4 — 📡 DATA
#  Pencere 5 — 🔔 ALERT
#  Pencere 6 — 🛡️ PAPER
#  Pencere 7 — Monitor  (CPU/RAM/GPU izleme)
#  Pencere 8 — DETECT-MS (MSMP :3002)
#  Pencere 9 — BREAKOUT (Kırılım stratejisi)
#  Pencere 10 — STREAM-OHLCV (canlı OHLCV mum akışı :3008)
#  Pencere 11 — CALC-IND (indikatör hesaplama motoru :3007)
#  Pencere 12 — 🤖 AI (LLM agent katmanı, ai.toml + OpenAI/Anthropic)
#  Pencere 13 — 🖥️ CONSOLE (executiond elle komut konsolu)
# ============================================================
set -euo pipefail

SESSION="cycle"
# Kurulu pakette CYCLE_ROOT, kaynak ağacında varsayılan olarak betiğin konumundan bulunur.
ROOT="${CYCLE_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"

# ── Binary dizini: varsayılan release; debug için BIN_DIR=./target/debug ver ──
BIN="${BIN_DIR:-$ROOT/target/release}"
BUILD_ARGS=""
case "$BIN" in
  *release*) BUILD_ARGS="--release" ;;
esac

# ── Kurulu paket dizinleri (kaynak ağacına göre varsayılan) ──
CONFIG_DIR="${CYCLE_CONFIG_DIR:-$ROOT}"
SCRIPTS_DIR="${CYCLE_SCRIPTS_DIR:-$ROOT/additional-services/scripts}"

# ── Env varsayılanları ───────────────────────────────────────
PAPER_API_ADDR="${PAPER_API_ADDR:-127.0.0.1:8080}"
PAPER_ADMIN_USER="${PAPER_ADMIN_USER:-admin}"
PAPER_ADMIN_PASS="${PAPER_ADMIN_PASS:-changeme123}"
PAPER_INITIAL_USDT="${PAPER_INITIAL_USDT:-100000}"
ALERT_CONFIG="${ALERT_CONFIG:-$CONFIG_DIR/alerts.toml}"

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
  for f in /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders; do
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
    echo "=== tmux Pencereleri ==="
    tmux list-windows -t "$SESSION" -F "  #{window_index}: #{window_name}" 2>/dev/null \
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

# ── Derleme (yalnızca kaynak ağacında) ────────────────────────
if [ -f "$ROOT/Cargo.toml" ]; then
  echo "🔨 Derleniyor..."
  cd "$ROOT"
  cargo build $BUILD_ARGS -p cycle-splash -p engine -p paper-service -p alert-service -p breakout-strategy -p stream-ohlcv -p ai-engine -p exec-console 2>&1 | tail -5
else
  echo "ℹ️  Kurulu paket — önceden derlenmiş binary'ler kullanılıyor ($BIN)"
fi

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
rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders
echo "  ✔ Ring buffer'lar temizlendi"
sleep 1

# ── Açılış ekranı (tek terminal) ─────────────────────────────
echo "🎬 Açılış ekranı..."
cd "$ROOT"
"$BIN/cycle-splash" 2>/dev/null || "$ROOT/target/debug/cycle-splash" 2>/dev/null || echo "  (cycle-splash bulunamadı)"

# ── Shell init dosyasını oluştur ────────────────────────────
cat > /tmp/cycle_init.sh << INITEOF
#!/usr/bin/env bash
export CYCLE_ROOT='$ROOT'
export CYCLE_API='http://$PAPER_API_ADDR'
export CYCLE_USER='$PAPER_ADMIN_USER'
export CYCLE_PASS='$PAPER_ADMIN_PASS'
source '$SCRIPTS_DIR/cycle_env.sh'
help-cycle
INITEOF
chmod +x /tmp/cycle_init.sh

# ── Session oluştur ──────────────────────────────────────────
tmux new-session -d -s "$SESSION" -x 220 -y 50
tmux rename-window -t "$SESSION:0" "🧠 STRATEGY"

# ── Pencere 0: STRATEGY ─────────────────────────────────────
tmux send-keys -t "$SESSION:0" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧠  STRATEGY TERMİNALİ  (PyO3)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $ROOT && RUN_MODE=STRATEGY $BIN/engine
" Enter

# ── Pencere 1: LISTENER ─────────────────────────────────────
tmux new-window -t "$SESSION:1" -n "🛰️  LISTENER"
tmux send-keys -t "$SESSION:1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛰️   LISTENER  (Anlık Metrik Analizi)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/listener
" Enter

# ── Pencere 2: RISK ─────────────────────────────────────────
tmux new-window -t "$SESSION:2" -n "⚠️  RISK"
tmux send-keys -t "$SESSION:2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '⚠️   RİSK ANALİZİ  (market_data.db)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/risk_analysis --watch
" Enter

# ── Pencere 3: SHELL ────────────────────────────────────────
tmux new-window -t "$SESSION:3" -n "💻 SHELL"
tmux send-keys -t "$SESSION:3" "source /tmp/cycle_init.sh" Enter

# ── Pencere 4: DATA ─────────────────────────────────────────
tmux new-window -t "$SESSION:4" -n "📡 DATA"
tmux send-keys -t "$SESSION:4" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  DATA TERMİNALİ  (Binance WS)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
cd $ROOT && RUN_MODE=DATA $BIN/engine
" Enter

# ── Pencere 5: ALERT ────────────────────────────────────────
tmux new-window -t "$SESSION:5" -n "🔔 ALERT"
tmux send-keys -t "$SESSION:5" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔔  ALERT SERVİSİ  (Sesli Uyarı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/alert-service --config $ALERT_CONFIG
" Enter

# ── Pencere 6: PAPER ────────────────────────────────────────
tmux new-window -t "$SESSION:6" -n "🛡️ PAPER"
tmux send-keys -t "$SESSION:6" "
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

# ── Pencere 7: MONITOR ──────────────────────────────────────
tmux new-window -t "$SESSION:7" -n "Monitor"
tmux send-keys -t "$SESSION:7" "bash '$SCRIPTS_DIR/monitor.sh'" Enter

# ── Pencere 8: DETECT-MS ────────────────────────────────────
tmux new-window -t "$SESSION:8" -n "DETECT-MS"
tmux send-keys -t "$SESSION:8" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📈  DETECT-MS  (MSMP 2.0 :3002)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-ms
" Enter

# ── Pencere 9: BREAKOUT STRATEJİ ────────────────────────────
tmux new-window -t "$SESSION:9" -n "BREAKOUT"
tmux send-keys -t "$SESSION:9" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🎯  BREAKOUT  (Kırılım Stratejisi)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/breakout-strategy
" Enter

# ── Pencere 10: STREAM-OHLCV ────────────────────────────────
tmux new-window -t "$SESSION:10" -n "STREAM-OHLCV"
tmux send-keys -t "$SESSION:10" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  STREAM-OHLCV  (Canlı OHLCV :3008)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/stream-ohlcv
" Enter

# ── Pencere 11: CALC-IND ────────────────────────────────────
tmux new-window -t "$SESSION:11" -n "CALC-IND"
tmux send-keys -t "$SESSION:11" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧮  CALC-IND  (İndikatör Motoru :3007)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/calc-ind
" Enter

# ── Pencere 12: AI ENGINE ───────────────────────────────────
tmux new-window -t "$SESSION:12" -n "🤖 AI"
tmux send-keys -t "$SESSION:12" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🤖  AI ENGINE  (LLM Agent Katmanı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $CONFIG_DIR && $BIN/ai-engine
" Enter

# ── Pencere 13: EXEC CONSOLE ────────────────────────────────
tmux new-window -t "$SESSION:13" -n "🖥️ CONSOLE"
tmux send-keys -t "$SESSION:13" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🖥️  EXEC CONSOLE  (executiond :3010)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $ROOT && $BIN/exec-console
" Enter

# ── Görsel ayarlar (global) ──────────────────────────────────
tmux set-option -t "$SESSION" mouse on
tmux set-option -t "$SESSION" status-interval 1

# ── Pano yapıştırma: Ctrl+V / Ctrl+Shift+V → OS panosunu yapıştır ──
tmux bind -n C-v run-shell "$SCRIPTS_DIR/tmux_clipboard_paste.sh" 2>/dev/null || true
tmux bind -n C-S-v run-shell "$SCRIPTS_DIR/tmux_clipboard_paste.sh" 2>/dev/null || true
tmux set-option -g set-clipboard on 2>/dev/null || true

# Status bar — Matrix yeşili / siyah
tmux set-option -t "$SESSION" status-style          "bg=#000000,fg=#00ff41"
tmux set-option -t "$SESSION" status-left           "#[bg=#003300,fg=#00ff41,bold]  🏛️  Cycle Finance  #[bg=#000000,fg=#00ff41] "
tmux set-option -t "$SESSION" status-left-length    30
tmux set-option -t "$SESSION" status-right          "#[fg=#00ff41]0#[fg=#00cc33]:STRAT #[fg=#00ff41]1#[fg=#00cc33]:LISTEN #[fg=#00ff41]2#[fg=#00cc33]:RISK #[fg=#00ff41]4#[fg=#00cc33]:DATA #[fg=#00ff41]5#[fg=#00cc33]:ALERT #[fg=#00ff41]6#[fg=#00cc33]:PAPER #[fg=#00ff41]7#[fg=#00cc33]:Mon #[fg=#00ff41]10#[fg=#00cc33]:STREAM #[fg=#00ff41]11#[fg=#00cc33]:CALC #[fg=#00ff41]12#[fg=#00cc33]:AI #[fg=#00ff41]13#[fg=#00cc33]:CONSOLE #[fg=#00ff41]%H:%M:%S"
tmux set-option -t "$SESSION" status-right-length   80

# Window sekme renkleri — matrix
tmux set-option -t "$SESSION" window-status-format          "#[fg=#008a2e] #{window_index}:#{window_name} "
tmux set-option -t "$SESSION" window-status-current-format  "#[bg=#003300,fg=#00ff41,bold] #{window_index}:#{window_name} "

# ── Terminal penceresine dön ve bağlan ───────────────────────
tmux select-window -t "$SESSION:0"
tmux attach-session -t "$SESSION"
