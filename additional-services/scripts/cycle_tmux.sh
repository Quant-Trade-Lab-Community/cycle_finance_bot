#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — tmux tek-sekme başlatıcı
#  Kullanım: ./scripts/cycle_tmux.sh [attach|kill|status]
#
#  Pencere 0 — 💻 SHELL (cycle-engine shell — orkestrasyon komutları)
#  Pencere 1 — 🧠 STRATEGY (strateji orkestrasyon merkezi, strategy-console)
#  Pencere 2 — 📡 DATA (Binance WS veri hattı)
#  Pencere 3 — 📈 DETECT-MS (piyasa yapısı analizi :3002)
#  Pencere 4 — 🛰️  PRICE-FEED (fiyat akışı :3004)
#  Pencere 5 — 🧮 CALC-IND (indikatör motoru :3007)
#  Pencere 6 — 📊 STREAM-OHLCV (canlı OHLCV mum akışı :3008)
#  Pencere 7 — 🛡️  PAPER (paper trading REST API :8080)
#  Pencere 8 — ⚠️  RISK (risk analizi)
#  Pencere 9 — 🔔 ALERT (sesli uyarı)
#  Pencere 10 — Monitor (CPU/RAM/GPU izleme)
#  Pencere 11 — 🤖 AI (LLM agent katmanı)
#  Pencere 12 — 🖥️  CONSOLE (executiond elle komut konsolu)
#
#  Stratejiler ayrı pencerede DEĞİL, STRATEGY konsolunun içinde
#  (orkestrasyon merkezi altında) çalışır. Shell'den:
#     strat run breakout      strat stop breakout
#     strat run breakout xxx  strat status
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

# ── Uzun isimli süreçlerde Linux comm 15-karakter sınırı:
#    breakout-strategy / strategy-console → pgrep/pkill -f gerekir.
_proc_alive() {
  local name="$1"
  if [ "${#name}" -le 15 ]; then pgrep -x "$name" &>/dev/null; else pgrep -f "$name" &>/dev/null; fi
}
_proc_kill() {
  local name="$1" sig="${2:-TERM}"
  if [ "${#name}" -le 15 ]; then pkill -"$sig" -x "$name" 2>/dev/null || true; else pkill -"$sig" -f "$name" 2>/dev/null || true; fi
}

# ── Tam temizlik fonksiyonu ──────────────────────────────────
full_cleanup() {
  echo "🧹 Temizleniyor..."
  tmux kill-session -t "$SESSION" 2>/dev/null && echo "  ✔ tmux session kapatıldı" || echo "  - tmux session yoktu"
  for proc in core paper-service alert-service breakout-strategy; do
    if _proc_alive "$proc"; then
      _proc_kill "$proc" TERM
      sleep 0.5
      _proc_kill "$proc" KILL
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
for proc in core paper-service alert-service breakout-strategy; do
      pid=$(pgrep -x "$proc" 2>/dev/null | head -1 || true)
      [ -z "$pid" ] && pid=$(pgrep -f "$proc" 2>/dev/null | head -1 || true)
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
for proc in core paper-service alert-service breakout-strategy; do
  if _proc_alive "$proc"; then
    _proc_kill "$proc" TERM
    sleep 0.3
    _proc_kill "$proc" KILL
    echo "  ✔ $proc durduruldu"
  fi
done
rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders
rm -rf /tmp/strategy_cmd.d
echo "  ✔ Ring buffer'lar ve strateji komut kuyruğu temizlendi"
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
tmux rename-window -t "$SESSION:0" "💻 SHELL"

# ── Pencere 0: SHELL (cycle-engine shell — orkestrasyon komutları) ──
tmux send-keys -t "$SESSION:0" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '💻  CYCLE-ENGINE SHELL'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo 'Strateji orkestrasyonu:'
echo '   strat run breakout          bir stratejiyi başlat'
echo '   strat run breakout xxx      birden fazlasını başlat'
echo '   strat stop breakout         durdur'
echo '   strat list / strat status   durum'
echo '   strat attach                STRATEGY konsoluna git'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
" Enter
tmux send-keys -t "$SESSION:0" "source /tmp/cycle_init.sh" Enter

# ── Pencere 1: STRATEGY (strateji orkestrasyon merkezi) ─────
tmux new-window -t "$SESSION:1" -n "🧠 STRATEGY"
tmux send-keys -t "$SESSION:1" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧠  STRATEJİ ORKESTRASYON MERKEZİ'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
cd $ROOT && $BIN/strategy-console
" Enter

# ── Pencere 2: DATA ─────────────────────────────────────────
tmux new-window -t "$SESSION:2" -n "📡 DATA"
tmux send-keys -t "$SESSION:2" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📡  DATA TERMİNALİ  (Binance WS)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
cd $ROOT && $BIN/engine
" Enter

# ── Pencere 3: DETECT-MS ────────────────────────────────────
tmux new-window -t "$SESSION:3" -n "📈 DETECT-MS"
tmux send-keys -t "$SESSION:3" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📈  DETECT-MS  (MSMP 2.0 :3002)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/detect-ms
" Enter

# ── Pencere 4: PRICE-FEED ───────────────────────────────────
tmux new-window -t "$SESSION:4" -n "🛰️ PRICE-FEED"
tmux send-keys -t "$SESSION:4" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🛰️   PRICE-FEED  (Fiyat Akışı :3004)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/price-feed
" Enter

# ── Pencere 5: CALC-IND ─────────────────────────────────────
tmux new-window -t "$SESSION:5" -n "🧮 CALC-IND"
tmux send-keys -t "$SESSION:5" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🧮  CALC-IND  (İndikatör Motoru :3007)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/calc-ind
" Enter

# ── Pencere 6: STREAM-OHLCV ─────────────────────────────────
tmux new-window -t "$SESSION:6" -n "📊 STREAM-OHLCV"
tmux send-keys -t "$SESSION:6" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '📊  STREAM-OHLCV  (Canlı OHLCV :3008)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/stream-ohlcv
" Enter

# ── Pencere 7: PAPER ────────────────────────────────────────
tmux new-window -t "$SESSION:7" -n "🛡️ PAPER"
tmux send-keys -t "$SESSION:7" "
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

# ── Pencere 8: RISK ─────────────────────────────────────────
tmux new-window -t "$SESSION:8" -n "⚠️ RISK"
tmux send-keys -t "$SESSION:8" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '⚠️   RİSK ANALİZİ  (market_data.db)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 4
cd $ROOT && $BIN/risk_analysis --watch
" Enter

# ── Pencere 9: ALERT ────────────────────────────────────────
tmux new-window -t "$SESSION:9" -n "🔔 ALERT"
tmux send-keys -t "$SESSION:9" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🔔  ALERT SERVİSİ  (Sesli Uyarı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 2
cd $ROOT && $BIN/alert-service --config $ALERT_CONFIG
" Enter

# ── Pencere 10: MONITOR ─────────────────────────────────────
tmux new-window -t "$SESSION:10" -n "Monitor"
tmux send-keys -t "$SESSION:10" "bash '$SCRIPTS_DIR/monitor.sh'" Enter

# ── Pencere 11: AI ENGINE ───────────────────────────────────
tmux new-window -t "$SESSION:11" -n "🤖 AI"
tmux send-keys -t "$SESSION:11" "
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
echo '🤖  AI ENGINE  (LLM Agent Katmanı)'
echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
sleep 3
cd $CONFIG_DIR && $BIN/ai-engine
" Enter

# ── Pencere 12: EXEC CONSOLE ────────────────────────────────
tmux new-window -t "$SESSION:12" -n "🖥️ CONSOLE"
tmux send-keys -t "$SESSION:12" "
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
tmux set-option -t "$SESSION" status-right          "#[fg=#00ff41]1#[fg=#00cc33]:STRAT #[fg=#00ff41]2#[fg=#00cc33]:DATA #[fg=#00ff41]3#[fg=#00cc33]:DETECT #[fg=#00ff41]4#[fg=#00cc33]:PRICE #[fg=#00ff41]5#[fg=#00cc33]:CALC #[fg=#00ff41]7#[fg=#00cc33]:PAPER #[fg=#00ff41]11#[fg=#00cc33]:AI #[fg=#00ff41]12#[fg=#00cc33]:CONSOLE #[fg=#00ff41]%H:%M:%S"
tmux set-option -t "$SESSION" status-right-length   80

# Window sekme renkleri — matrix
tmux set-option -t "$SESSION" window-status-format          "#[fg=#008a2e] #{window_index}:#{window_name} "
tmux set-option -t "$SESSION" window-status-current-format  "#[bg=#003300,fg=#00ff41,bold] #{window_index}:#{window_name} "

# ── Terminal penceresine dön ve bağlan ───────────────────────
tmux select-window -t "$SESSION:0"
tmux attach-session -t "$SESSION"
