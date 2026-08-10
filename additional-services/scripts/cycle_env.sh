#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — Shell Yardımcı Komutları
#  Bu dosya cycle_tmux.sh tarafından otomatik source edilir.
#  Elle de kullanılabilir: source <proje-koku>/additional-services/scripts/cycle_env.sh
# ============================================================

# ── Kök dizini otomatik bul ──────────────────────────────────
CYCLE_ROOT="${CYCLE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
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

  echo -e "\n${_Y}━━━  ⚙️  SİSTEMLERİ TEK TEK AÇ / KAPAT  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_G}data-start${_N} / ${_R}data-stop${_N}          DATA terminali (Binance WS)"
  echo -e "  ${_G}strategy-start${_N} / ${_R}strategy-stop${_N}  STRATEGY orkestrasyon konsolu"
  echo -e "  ${_G}paper-start${_N} / ${_R}paper-stop${_N}        Paper-service (REST :8080)"
  echo -e "  ${_G}alert-start${_N} / ${_R}alert-stop${_N}        Alert-service"
  echo -e "  ${_G}listener-start${_N} / ${_R}listener-stop${_N}  Listener (anlık metrik analizi)"
  echo -e "  ${_G}detect-ms-start${_N} / ${_R}detect-ms-stop${_N}  MSMP analiz motoru (:3002)"
  echo -e "  ${_G}calc-ind-start${_N} / ${_R}calc-ind-stop${_N}    İndikatör hesaplama motoru (:3007)"
  echo -e "  ${_G}stream-ohlcv-start${_N} / ${_R}stream-ohlcv-stop${_N}  Canlı OHLCV mum akışı (:3008)"

  echo -e "\n${_Y}━━━  🤖 AI ENGINE (LLM Agent Katmanı)  ━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}ai-start${_N}           AI Engine'i başlat (ai.toml + OpenAI/Anthropic)"
  echo -e "  ${_R}ai-stop${_N}            Durdur"
  echo -e "  ${_C}ai-status${_N}          Çalışıyor mu? CPU/RAM + son döngü"
  echo -e "  ${_C}ai-approve${_N}         HITL modunda bekleyen emri onayla (echo approve)"
  echo -e "  ${_C}ai-reject${_N}          HITL modunda bekleyen emri reddet"
  echo -e "  ${_C}ai-log${_N}             Canlı log izle"

  echo -e "\n${_Y}━━━  🖥️  EXEC CONSOLE (Execution Engine elle komut)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}exec-console-start${_N}   Konsolu tmux sekmesinde başlat (executiond :3010)"
  echo -e "  ${_R}exec-console-stop${_N}    Durdur"
  echo -e "  ${_C}exec-console-status${_N}  Çalışıyor mu? CPU/RAM"
  echo -e "  ${_C}exec-console-log${_N}     Konsol penceresine geç (Ctrl+B → 13)"

  echo -e "\n${_Y}━━━  🛰️  LISTENER  (Anlık Metrik Analizi)  ━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}listener-start${_N}      Ayrı pencerede başlat"
  echo -e "  ${_C}listener-stop${_N}       Durdur"
  echo -e "  ${_C}listener-status${_N}     Çalışıyor mu? CPU/RAM"
  echo -e "  ${_C}listenconfig-list${_N}   Metrik parametrelerini göster"
  echo -e "  ${_C}listenconfig-set KEY VAL${_N}  Parametre değiştir (lambda, k_abs, gamma...) "
  echo -e "  ${_C}listenconfig-reset${_N}  Varsayılanlara dön"
  echo -e "  ${_C}listener-log${_N}        Metrik çıktısını izle (/tmp/listener_metrics.json)"

  echo -e "\n${_Y}━━━  ⚠️  RİSK ANALİZİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}risk-start${_N}           Risk analizini başlat (pencere 8)"
  echo -e "  ${_C}risk-worker-start${_N}    risk-worker daemon'ı başlat (korelasyon/VaR)"
  echo -e "  ${_C}risk-stop${_N}            Durdur"
  echo -e "  ${_C}risk-query${_N}           Tek seferlik analiz çalıştır"

  echo -e "\n${_Y}━━━  💹 PRICE-FEED  (WS→Ring, Anlık Last/Mark/Index)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}pricefeed-start${_N}     Arka planda başlat (:3004)"
  echo -e "  ${_C}pricefeed-stop${_N}      Durdur"
  echo -e "  ${_C}pricefeed-status${_N}    Çalışıyor mu? CPU/RAM + health"
  echo -e "  ${_C}pricefeed-query SYM${_N} Tek sembol sorgula (örn. pricefeed-query HEIUSDT)"
  echo -e "  ${_C}pricefeed-log${_N}       Canlı log izle"

  echo -e "\n${_Y}━━━  📡 DATA TERMİNALİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}data-live${_N}            Canlı Binance WS başlat (engine DATA konsolu)"
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

  echo -e "\n${_Y}━━━  🛡️ EXECUTION ENGINE (Canlı Binance :3010)  ━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}exec-setup${_N}          Anahtar gir (ekrana yazılmaz, .env 600)"
  echo -e "  ${_C}exec-show${_N}           Yapılandırma göster (anahtarlar maskeli)"
  echo -e "  ${_C}exec-testnet${_N}        Testnet URL'leri yaz"
  echo -e "  ${_C}exec-dry${_N}            executiond DRY_RUN'da başlat (emir gitmez)"
  echo -e "  ${_R}exec-live${_N}           Gerçek emir modu ('GO' onayı ister)"
  echo -e "  ${_C}exec-stop${_N}           executiond durdur"
  echo -e "  ${_C}exec-status${_N}         Mod + risk durumu"
  echo -e "  ${_C}exec-account / exec-positions / exec-balance / exec-orders${_N}"
  echo -e "  ${_R}exec-kill / exec-unkill${_N}  Kill switch aç/kapat (acil durum)"

  echo -e "\n${_Y}━━━  🧠 STRATEJİ ORKESTRASYONU  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}strat run breakout${_N}        Strateji(ler)i başlat (istendiği kadar)"
  echo -e "  ${_C}strat run breakout rsi${_N}    Birden fazla stratejiyi başlat"
  echo -e "  ${_C}strat stop breakout${_N}       Strateji(ler)i durdur"
  echo -e "  ${_C}strat list${_N}                Mevcut stratejiler (services-engine/strategies/)"
  echo -e "  ${_C}strat status${_N}              Orkestrasyon durumu (çalışan stratejiler)"
  echo -e "  ${_C}strat attach${_N}              STRATEGY konsoluna geç (pencere 1)"
  echo -e "  ${_C}strategy-start${_N} / ${_R}strategy-stop${_N}  Orkestrasyon konsolunu aç/kapat"
  echo -e "  ${_C}breakout-wait 600${_N}         Kırılım bekleme süresini ayarla (saniye)"

  echo -e "\n${_Y}━━━  🔔 ALERT SERVİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}alert-list${_N}           Aktif uyarıları listele"
  echo -e "  ${_C}alert-add HEIUSDT above 0.22 \"ses\"${_N}   Yeni alarm ekle"
  echo -e "  ${_C}alert-update SYM cond OLD NEW${_N}   Alarmı güncelle"
  echo -e "  ${_C}alert-remove SYM cond PRICE${_N}     Alarmı sil"
  echo -e "  ${_C}alert-reload${_N}         Alert servisini yeniden başlat"

  echo -e "\n${_Y}━━━  📈 DETECT-MS  (Market Structure Engine :3002)  ━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}detect-ms-start${_N}      Servisi arka planda başlat (port 3002)"
  echo -e "  ${_C}detect-ms-stop${_N}       Servisi durdur"
  echo -e "  ${_C}detect-ms-status${_N}     Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}detect-ms-query${_N}      BTCUSDT 15m analiz (JSON çıktı)"
  echo -e "  ${_C}detect-ms-query ETHUSDT 1h 500${_N}   Özel sorgu"
  echo -e "  ${_C}detect-ms-log${_N}        Canlı log izle"

  echo -e "\n${_Y}━━━  🎯 KIRILIM STRATEJİSİ (breakout)  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}strat run breakout${_N}      Stratejiyi başlat (HEIUSDT 1m, 100 pencere)"
  echo -e "  ${_C}strat stop breakout${_N}     Stratejiyi durdur"
  echo -e "  ${_C}strat status${_N}            Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}breakout-query${_N}          Tek seferlik analiz (emir açmaz)"
  echo -e "  ${_C}breakout-query --dry-run${_N}  Analiz + kırılım simülasyonu"
  echo -e "  ${_C}breakout-wait 600${_N}       Bekleme süresini ayarla (saniye)"
  echo -e "  ${_C}breakout-log${_N}            Canlı strateji logu izle"

  echo -e "\n${_Y}━━━  📡 STREAM-OHLCV  (Canlı OHLCV Mum Akışı :3008)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}stream-ohlcv-start${_N}    Servisi başlat (ring: /dev/shm/cycle_finance_stream_ohlcv)"
  echo -e "  ${_C}stream-ohlcv-stop${_N}     Servisi durdur"
  echo -e "  ${_C}stream-ohlcv-status${_N}   Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}stream-ohlcv-start-stream SYM ITV START_MS${_N}   Stream aç (örn. BTCUSDT 60 0)"
  echo -e "  ${_C}stream-ohlcv-streams${_N}  Aktif stream'leri listele"
  echo -e "  ${_C}stream-ohlcv-query SYM ITV START_MS${_N}   Stream aç + durum göster"

  echo -e "\n${_Y}━━━  📊 İZLEME  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}monitor-start${_N}        İzleme paneline geç (Ctrl+B → 10)"

  echo -e "\n${_Y}━━━  🗄️  VERİTABANI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}db-trades${_N}            Son 20 işlemi göster"
  echo -e "  ${_C}db-size${_N}              Veritabanı boyutu"

  echo -e "\n${_Y}━━━  🌐 TMUX KISAYOLLARI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_B}Ctrl+B → ok tuşu${_N}     Pencere değiştir"
  echo -e "  ${_B}Ctrl+B → d${_N}           Session'ı arka plana al"
  echo -e "  ${_B}Ctrl+B → 0${_N}           💻 SHELL sekmesi (orkestrasyon komutları)"
  echo -e "  ${_B}Ctrl+B → 1${_N}           🧠 STRATEGY sekmesi (orkestrasyon konsolu)"
  echo -e "  ${_B}Ctrl+B → 2${_N}           📡 DATA sekmesi"
  echo -e "  ${_B}Ctrl+B → 3${_N}           📈 DETECT-MS sekmesi"
  echo -e "  ${_B}Ctrl+B → 4${_N}           🛰️ PRICE-FEED sekmesi"
  echo -e "  ${_B}Ctrl+B → 5${_N}           🧮 CALC-IND sekmesi"
  echo -e "  ${_B}Ctrl+B → 6${_N}           📊 STREAM-OHLCV sekmesi"
  echo -e "  ${_B}Ctrl+B → 7${_N}           🛡️ PAPER sekmesi"
  echo -e "  ${_B}Ctrl+B → 8${_N}           ⚠️ RISK sekmesi"
  echo -e "  ${_B}Ctrl+B → 9${_N}           🔔 ALERT sekmesi"
  echo -e "  ${_B}Ctrl+B → 10${_N}          Monitor sekmesi"
  echo -e "  ${_B}Ctrl+B → 11${_N}          🤖 AI sekmesi"
  echo -e "  ${_B}Ctrl+B → 12${_N}          🖥️ CONSOLE sekmesi"
  echo -e "  ${_B}Fare tıklama/scroll${_N}  Pencere seç / scroll"

  echo -e "\n${_W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_D}help-cycle yazarak bu listeye tekrar ulaşabilirsin.${_N}"
  echo ""
}

# ============================================================
#  SİSTEM YÖNETİMİ
# ============================================================
# Bu dosya değiştiğinde fonksiyonları güncellemek için:
reload-cycle() {
  source "$CYCLE_ROOT/additional-services/scripts/cycle_env.sh" >/dev/null 2>&1
  echo "✅ cycle_env.sh yeniden yüklendi"
}

# Her start/stop fonksiyonunun güncel sürümü kullanması için otomatik yenileme
# (tmux SHELL paneli eski sürümü yüklemiş olsa bile sorun yaşanmaz)
_start_guard() {
  source "$CYCLE_ROOT/additional-services/scripts/cycle_env.sh" >/dev/null 2>&1
}
cycle-start() {
  "$CYCLE_ROOT/additional-services/scripts/cycle_tmux.sh"
}
cycle-kill() {
  "$CYCLE_ROOT/additional-services/scripts/cycle_tmux.sh" kill
}
cycle-status() {
  "$CYCLE_ROOT/additional-services/scripts/cycle_tmux.sh" status
}
cycle-build() {
  cd "$CYCLE_ROOT" && cargo build -p engine -p paper-service -p alert-service -p breakout-strategy
}
cycle-build-full() {
  cd "$CYCLE_ROOT" && cargo build -p paper-service --features full
}

# ============================================================
#  SİSTEMLERİ TEK TEK AÇ / KAPAT
#  Her servis ayrı sekme (pencere) olarak açılır.
# ============================================================
# Yardımcı: ilgili pencereye komut gönder
# Pencere haritası: 0=SHELL 1=STRATEGY 2=DATA 3=DETECT-MS
#                   4=PRICE-FEED 5=CALC-IND 6=STREAM-OHLCV
#                   7=PAPER 8=RISK 9=ALERT 10=Monitor
#                   11=AI 12=CONSOLE 13=EXEC 14=RISK-WORKER
_tmux_pane() {
  local name="$1"; shift
  local session="cycle"
  local pane
  case "$name" in
    "💻SHELL")     pane="0" ;;
    "🧠STRATEGY")  pane="1" ;;
    "📡DATA")      pane="2" ;;
    "📈DETECT-MS") pane="3" ;;
    "🛰️PRICE-FEED") pane="4" ;;
    "🧮CALC-IND")  pane="5" ;;
    "📡STREAM-OHLCV") pane="6" ;;
    "🛡️PAPER")     pane="7" ;;
    "⚠️RISK")      pane="8" ;;
    "🔔ALERT")     pane="9" ;;
    "Monitor")     pane="10" ;;
    "🤖AI")        pane="11" ;;
    "🖥️CONSOLE")   pane="12" ;;
    "🛡️EXEC")      pane="13" ;;
    "🧮RISK-WORKER") pane="14" ;;
    *)
      # Tanınmayan → yeni pencere (ör. özel servisler)
      if ! tmux has-session -t "$session" 2>/dev/null; then
        tmux new-session -d -s "$session" -x 220 -y 50
        tmux rename-window -t "$session:0" "💻 SHELL"
      fi
      local idx
      idx=$(tmux list-windows -t "$session" -F "#{window_name} #{window_index}" 2>/dev/null | awk -v n="$name" '$1==n{print $2}')
      if [ -z "$idx" ]; then
        tmux new-window -t "$session" -n "$name"
        idx=$(tmux list-windows -t "$session" -F "#{window_name} #{window_index}" 2>/dev/null | awk -v n="$name" '$1==n{print $2}')
      fi
      tmux send-keys -t "$session:$idx" "$@"
      return 0
      ;;
  esac
  tmux send-keys -t "$session:$pane" C-c
  tmux send-keys -t "$session:$pane" C-u
  tmux send-keys -t "$session:$pane" "$@"
}

# ── DATA/STRATEGY konsolları ─────────────────────────────────
# DATA → engine binary'si, STRATEGY → strategy-console binary'si (ayrı süreç).
# Not: Linux comm 15-karakter sınırı → uzun isimlerde -f gerekir.
_core_mode_pid() {
  local mode="$1"
  case "$mode" in
    STRATEGY) pgrep -f "strategy-console" 2>/dev/null | head -1 ;;
    *)        pgrep -x engine 2>/dev/null | head -1 ;;
  esac
}

data-start() {
  _start_guard
  if _core_mode_pid DATA &>/dev/null; then echo "⚠️  DATA zaten çalışıyor (pid: $(_core_mode_pid DATA))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p engine 2>&1 | tail -1
  rm -f /dev/shm/cycle_finance_ring /dev/shm/cycle_finance_orders
  _tmux_pane "📡DATA" "cd $CYCLE_ROOT && ./target/debug/engine" Enter
  echo "✅ DATA başlatıldı (pencere 2 — 📡 DATA)"
}
data-stop() {
  _start_guard
  local p; p=$(_core_mode_pid DATA)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; echo "✅ DATA durduruldu [pid:$p]"; else echo "ℹ️  DATA çalışmıyor"; fi
}

# ── STRATEGY orkestrasyon konsolu (ayrı strategy-console binary'si) ─
strategy-start() {
  _start_guard
  if _core_mode_pid STRATEGY &>/dev/null; then echo "⚠️  STRATEGY zaten çalışıyor (pid: $(_core_mode_pid STRATEGY))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p engine 2>&1 | tail -1
  mkdir -p /tmp/strategy_cmd.d
  _tmux_pane "🧠STRATEGY" "cd $CYCLE_ROOT && ./target/debug/strategy-console" Enter
  echo "✅ STRATEGY orkestrasyon konsolu başlatıldı (pencere 1 — 🧠 STRATEGY)"
}
strategy-stop() {
  _start_guard
  local p; p=$(_core_mode_pid STRATEGY)
  if [ -n "$p" ]; then
    kill -TERM "$p" 2>/dev/null; sleep 1
    # Orkestratör alt-süreçlerini de temizle (uzun isim → -f)
    pkill -TERM -f "breakout-strategy" 2>/dev/null || true
    echo "✅ STRATEGY durduruldu [pid:$p]"
  else
    echo "ℹ️  STRATEGY çalışmıyor"
  fi
}

# ── STRATEJİ ORKESTRASYONU (shell → STRATEGY konsolu) ────────
# Komut, maildir benzeri /tmp/strategy_cmd.d kuyruğuna yazılır;
# STRATEGY konsolundaki orkestrasyon merkezi okur ve yürütür.
__strat_send() {
  mkdir -p /tmp/strategy_cmd.d
  local f="/tmp/strategy_cmd.d/cmd_$(date +%s%N).cmd"
  printf '%s\n' "$*" > "$f"
}
__strat_wait_status() {
  # Orkestratörün işlemesi için kısa bekle, ardından durum dosyasını oku
  sleep 0.4
  if [ -f /tmp/strategy_status.txt ]; then
    cat /tmp/strategy_status.txt
  else
    echo "ℹ️  STRATEGY orkestrasyon konsolu çalışmıyor. 'strategy-start' ile başlatın."
  fi
}
strat() {
  local cmd="${1:-help}"; shift || true
  case "$cmd" in
    run|start)
      if [ $# -eq 0 ]; then echo "Kullanım: strat run <strateji> [<strateji>...]"; return 1; fi
      __strat_send "run $*"
      echo "🚀 Strateji komutu iletildi: run $*"
      echo "   (STRATEGY konsolu: pencere 1 — orkestrasyon merkezi)"
      __strat_wait_status
      ;;
    stop)
      if [ $# -eq 0 ]; then echo "Kullanım: strat stop <strateji> [<strateji>...]"; return 1; fi
      __strat_send "stop $*"
      echo "⏹  Strateji komutu iletildi: stop $*"
      __strat_wait_status
      ;;
    restart)
      if [ $# -eq 0 ]; then echo "Kullanım: strat restart <strateji>"; return 1; fi
      __strat_send "restart $*"
      echo "🔄 Strateji komutu iletildi: restart $*"
      __strat_wait_status
      ;;
    list|ls)
      __strat_send "list"
      __strat_wait_status
      ;;
    status)
      __strat_send "status"
      __strat_wait_status
      ;;
    attach)
      tmux select-window -t cycle:1 2>/dev/null && echo "📌 STRATEGY konsoluna geçildi (pencere 1)" || echo "ℹ️  tmux session 'cycle' çalışmıyor."
      ;;
    help)
      echo "Kullanım: strat <komut>"
      echo "  strat run <strateji> [...]   strateji(ler)i başlat (örn. breakout)"
      echo "  strat stop <strateji> [...]  strateji(ler)i durdur"
      echo "  strat restart <strateji>     stratejiyi yeniden başlat"
      echo "  strat list                   mevcut stratejileri listele"
      echo "  strat status                 orkestrasyon durumu"
      echo "  strat attach                 STRATEGY konsoluna geç"
      ;;
    *)
      echo "Bilinmeyen komut: '$cmd' (help yazın)"
      ;;
  esac
}

# ── PAPER-SERVICE (REST API :8080) ───────────────────────────
paper-start() {
  _start_guard
  if pgrep -x "paper-service" &>/dev/null; then echo "⚠️  paper-service zaten çalışıyor"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p paper-service 2>&1 | tail -1
  rm -rf "$CYCLE_ROOT/data-engine/data/paper_wal"
  _tmux_pane "🛡️PAPER" \
    "cd $CYCLE_ROOT && PAPER_ADMIN_USER=${PAPER_ADMIN_USER:-admin} PAPER_ADMIN_PASS=${PAPER_ADMIN_PASS:-changeme123} PAPER_API_ADDR=${PAPER_API_ADDR:-127.0.0.1:8080} PAPER_INITIAL_USDT=${PAPER_INITIAL_USDT:-100000} PAPER_DB_PATH=$CYCLE_ROOT/data-engine/data/paper_live.db PAPER_SLED_PATH=$CYCLE_ROOT/data-engine/data/paper_wal ./target/debug/paper-service" \
    Enter
  echo "✅ PAPER-SERVICE başlatıldı (pencere 7 — 🛡️ PAPER, http://127.0.0.1:8080)"
}
paper-stop() {
  _start_guard
  local p; p=$(pgrep -x paper-service 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ paper-service durduruldu [pid:$p]"; else echo "ℹ️  paper-service çalışmıyor"; fi
}

# ── ALERT-SERVICE ────────────────────────────────────────────
alert-start() {
  _start_guard
  if pgrep -x "alert-service" &>/dev/null; then echo "⚠️  alert-service zaten çalışıyor"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p alert-service 2>&1 | tail -1
  _tmux_pane "🔔ALERT" "cd $CYCLE_ROOT && ./target/debug/alert-service --config $CYCLE_ROOT/alerts.toml" Enter
  echo "✅ ALERT-SERVICE başlatıldı (pencere 9 — 🔔 ALERT)"
}
alert-stop() {
  _start_guard
  local p; p=$(pgrep -x alert-service 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ alert-service durduruldu [pid:$p]"; else echo "ℹ️  alert-service çalışmıyor"; fi
}

# ── RISK-WORKER (Soğuk yol parametre üretici — korelasyon/VaR) ──
risk-worker-start() {
  _start_guard
  if pgrep -x risk-worker &>/dev/null; then echo "⚠️  risk-worker zaten çalışıyor (pid: $(pgrep -x risk-worker | head -1))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p risk-engine 2>&1 | tail -1
  _tmux_pane "🧮RISK-WORKER" "cd $CYCLE_ROOT && ./target/debug/risk-worker" Enter
  sleep 2
  if pgrep -x risk-worker &>/dev/null; then
    echo "✅ RISK-WORKER başlatıldı (http://127.0.0.1:3011/healthz)"
  else
    echo "❌ RISK-WORKER başlatılamadı"
  fi
}
risk-worker-stop() {
  _start_guard
  local p; p=$(pgrep -x risk-worker 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ risk-worker durduruldu [pid:$p]"; else echo "ℹ️  risk-worker çalışmıyor"; fi
}
risk-worker-status() {
  _start_guard
  local p; p=$(pgrep -x risk-worker 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then echo "✅ RISK-WORKER ÇALIŞIYOR [pid:$p]"; else echo "✘  risk-worker durdurulmuş"; fi
}

# ── LISTENER (Anlık Metrik Analizi) ──────────
listener-start() {
  _start_guard
  if pgrep -x listener &>/dev/null; then
    echo "⚠️  listener zaten çalışıyor (pid: $(pgrep -x listener | head -1))"
    return 1
  fi
  # Bağımlılık: paper-service gerekli
  if ! pgrep -x paper-service &>/dev/null; then
    echo "⚠️  paper-service çalışmıyor — önce paper-start ile başlatın"
    return 1
  fi
  _tmux_pane "🛰️LISTENER" "cd $CYCLE_ROOT && $CYCLE_ROOT/target/release/listener" Enter
  sleep 2
  if pgrep -x listener &>/dev/null; then
    echo "✅ LISTENER başlatıldı (pencere: 🛰️ LISTENER)"
  else
    echo "❌ LISTENER başlatılamadı"
  fi
}
listener-stop() {
  _start_guard
  local p; p=$(pgrep -x listener 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    pkill -TERM -x listener 2>/dev/null
    sleep 1
    pkill -KILL -x listener 2>/dev/null || true
    echo "✅ LISTENER durduruldu [pid:$p]"
  else
    echo "ℹ️  LISTENER çalışmıyor"
  fi
}
listener-status() {
  _start_guard
  local p; p=$(pgrep -x listener 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    local cpu mem
    cpu=$(ps -p "$p" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$p" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ LISTENER ÇALIŞIYOR  [pid:$p  CPU:${cpu}%  RAM:${mem}]"
  else
    echo "✘  LISTENER durdurulmuş"
  fi
}
listener-log() {
  tail -f /tmp/listener_metrics.json 2>/dev/null || echo "metrik dosyası yok"
}

# ── RISK (Anlık risk analizi) ──────────────────────
risk-start() {
  _start_guard
  if pgrep -x risk_analysis &>/dev/null; then
    echo "⚠️  RISK zaten çalışıyor (pid: $(pgrep -x risk_analysis | head -1))"
    return 1
  fi
  _tmux_pane "⚠️RISK" "cd $CYCLE_ROOT && ./target/release/risk_analysis --watch" Enter
  sleep 2
  echo "✅ RISK başlatıldı (pencere 8 — ⚠️ RISK)"
}
risk-stop() {
  _start_guard
  local p; p=$(pgrep -x risk_analysis 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    pkill -TERM -x risk_analysis 2>/dev/null; sleep 1
    pkill -KILL -x risk_analysis 2>/dev/null || true
    echo "✅ RISK durduruldu [pid:$p]"
  else
    echo "ℹ️  RISK çalışmıyor"
  fi
}
risk-status() {
  _start_guard
  local p; p=$(pgrep -x risk_analysis 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    echo "✅ RISK ÇALIŞIYOR [pid:$p]"
  else
    echo "✘  RISK durdurulmuş"
  fi
}
risk-query() {
  _start_guard
  cd "$CYCLE_ROOT" && ./target/release/risk_analysis
 }

# ── Listener metrik parametreleri (shell'den ayarlanabilir) ──
# Config dosyası: /tmp/listener_metrics.conf (çalışan listener 5 sn'de bir yeniden okur)
LISTEN_CONF=/tmp/listener_metrics.conf

# listenconfig-list  → tüm parametreleri göster
# listenconfig-set lambda 0.02   → parametre değiştir
# listenconfig-reset          → varsayılanlara dön
listenconfig-list() {
  _start_guard
  local conf="$LISTEN_CONF"
  if [ -f "$conf" ]; then
    echo "=== Listener metrik parametreleri ($conf) ==="
    cat "$conf"
  else
    echo "ℹ️  Config dosyası yok — varsayılanlar kullanılıyor:"
    echo "  lambda = 0.015        (WLOBI decay)"
    echo "  theta_vol = 2.5       (Delta velocity eşiği)"
    echo "  alpha_bucket = 0.75   (aVPIN bucket sabiti)"
    echo "  k_abs = 100           (absorption penceresi, trade)"
    echo "  n_bucket = 50         (aVPIN bucket sayısı)"
    echo "  ice_threshold = 1.2   (Iceberg eşiği)"
    echo "  efp_threshold = 0.05  (Execution footprint eşiği)"
    echo "  noise_corr = 0.85     (Lee-Ready gürültü filtresi)"
    echo "  delta_window_sec = 60 (ΔV penceresi, saniye)"
    echo "  tps_window_sec = 10  (TPS penceresi, saniye)"
    echo "  corr_price_window_sec = 5 (fiyat korelasyonu penceresi, saniye)"
    echo "  corr_vol_window_sec = 5   (hacim korelasyonu penceresi, saniye)"
    echo "  gamma0..gamma5        (Alpha Basket ağırlıkları)"
  fi
}

listenconfig-set() {
  _start_guard
  local key="${1:-}" val="${2:-}"
  if [ -z "$key" ] || [ -z "$val" ]; then
    echo "Kullanım: listenconfig-set <key> <value>"
    echo "Örn: listenconfig-set lambda 0.02 | listenconfig-set k_abs 200"
    echo "     listenconfig-set gamma1 0.5 | listenconfig-set delta_window_sec 120"
    return 1
  fi
  local valid_keys="lambda theta_vol alpha_bucket k_abs n_bucket ice_threshold efp_threshold noise_corr delta_window_sec tps_window_sec corr_price_window_sec corr_vol_window_sec gamma0 gamma1 gamma2 gamma3 gamma4 gamma5"
  if ! echo "$valid_keys" | grep -qw "$key"; then
    echo "❌ Geçersiz parametre: $key"
    echo "Geçerli: $valid_keys"
    return 1
  fi
  # k_abs, n_bucket, delta_window_sec tam sayı olmalı
  if echo "k_abs n_bucket delta_window_sec tps_window_sec corr_price_window_sec corr_vol_window_sec" | grep -qw "$key"; then
    if ! echo "$val" | grep -qE '^[0-9]+$'; then
      echo "❌ $key tam sayı olmalı"; return 1
    fi
  else
    if ! echo "$val" | grep -qE '^-?[0-9]+(\.[0-9]+)?$'; then
      echo "❌ $key sayı olmalı"; return 1
    fi
  fi
  # Eski değeri değiştir veya ekle
  if grep -q "^${key} *=" "$LISTEN_CONF" 2>/dev/null; then
    sed -i "s|^${key} *=.*|${key} = ${val}|" "$LISTEN_CONF"
  else
    echo "${key} = ${val}" >> "$LISTEN_CONF"
  fi
  echo "✅ $key = $val kaydedildi ($LISTEN_CONF)"
  echo "   Çalışan listener 5 sn'de bir yeniden okur. list-restart ile hemen uygula."
}

listenconfig-reset() {
  _start_guard
  rm -f "$LISTEN_CONF"
  echo "✅ Varsayılan parametrelere dönüldü (config dosyası silindi)"
}

# Kısayollar
listener-config() { listenconfig-list; }
listener-set() { listenconfig-set "$@"; }

# ── PRICE-FEED (WS → ring buffer, anlık last/mark/index price) ──
pricefeed-start() {
  _start_guard
  if pgrep -x "price-feed" &>/dev/null; then
    echo "⚠️  price-feed zaten çalışıyor (pid: $(pgrep -x price-feed | head -1))"
    return 1
  fi
  cd "$CYCLE_ROOT" && cargo build -p price-feed 2>&1 | tail -1
  setsid nohup "$CYCLE_ROOT/target/debug/price-feed" > /tmp/price_feed.log 2>&1 < /dev/null &
  sleep 3
  if curl -s -m 2 http://127.0.0.1:3004/health >/dev/null 2>&1; then
    echo "✅ PRICE-FEED başlatıldı → http://127.0.0.1:3004/api/lastprice"
  else
    echo "❌ PRICE-FEED başlatılamadı:"; tail -5 /tmp/price_feed.log
  fi
}
pricefeed-stop() {
  _start_guard
  local p; p=$(pgrep -x "price-feed" 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ price-feed durduruldu [pid:$p]"; else echo "ℹ️  price-feed çalışmıyor"; fi
}
pricefeed-status() {
  _start_guard
  local p; p=$(pgrep -x "price-feed" 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    local cpu mem
    cpu=$(ps -p "$p" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$p" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ PRICE-FEED ÇALIŞIYOR  [pid:$p  CPU:${cpu}%  RAM:${mem}]"
    curl -s -m 2 http://127.0.0.1:3004/health
    echo
  else
    echo "✘  PRICE-FEED durdurulmuş"
  fi
}
pricefeed-query() {
  _start_guard
  local sym="${1:-BTCUSDT}"
  curl -s -m 3 "http://127.0.0.1:3004/api/lastprice/$sym" | python3 -m json.tool 2>/dev/null \
    || echo "❌ Servis yanıt vermiyor — pricefeed-start ile başlat."
}
pricefeed-log() {
  tail -f /tmp/price_feed.log
}

# ============================================================
#  DATA TERMİNALİ
# ============================================================
data-live() {
  cd "$CYCLE_ROOT" && ./target/debug/engine
}
data-backtest() {
  echo "ℹ️  BACKTEST modu kaldırıldı. Tek seferlik strateji analizi için:"
  echo "     breakout-query"
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
# Not: strategy-start/stop artık "SİSTEMLERİ TEK TEK AÇ/KAPAT" bölümünde.
# correlation_start: eski RUN_MODE kaldırıldı; strateji analizi breakout-query.
correlation-start() {
  echo "ℹ️  CORRELATION modu kaldırıldı. Listener ile mikro-yapı metriklerini kullanın:"
  echo "     listener-start && listener-log"
}

# ============================================================
#  ALERT SERVİSİ
# ============================================================
alert-list() {
  echo "=== alerts.toml — aktif uyarılar ==="
  "$CYCLE_ROOT/target/debug/alerts" list
  echo ""
  echo "Kullanım:"
  echo "  alert-add HEIUSDT above 0.22 [voice metni] [cooldown]"
  echo "  alert-update HEIUSDT above 0.21628 0.22 [voice] [cooldown]"
  echo "  alert-remove HEIUSDT above 0.21628"
}
alert-reload() {
  pkill -x alert-service 2>/dev/null || true
  sleep 1
  cd "$CYCLE_ROOT" && nohup ./target/debug/alert-service --config ./alerts.toml > /tmp/alert_service.log 2>&1 &
  echo "✅ Alert servisi yeniden başlatıldı (pid: $!)"
}

# ── Alarm yönetimi (shell'den) — değişiklik sonrası otomatik reload ──
_alert_apply() {
  local msg="$1"
  echo "$msg"
  echo "🔄 Alert servisi yeniden yükleniyor..."
  # Eski süreci durdur, tmux pane'inde yeniden başlat
  pkill -x alert-service 2>/dev/null || true
  sleep 1
  tmux send-keys -t "cycle:9" C-c 2>/dev/null
  tmux send-keys -t "cycle:9" "cd $CYCLE_ROOT && ./target/debug/alert-service --config $CYCLE_ROOT/alerts.toml" Enter 2>/dev/null
  sleep 1
  echo "✅ Tamamlandı. alert-list ile görüntüleyin."
}

# Yeni alarm ekle
# Kullanım: alert-add <SYMBOL> <above|below|cross|touch> <PRICE> [voice] [cooldown]
alert-add() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" price="${3:-}" voice="${4:-}" cooldown="${5:-30}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$price" ]; then
    echo "Kullanım: alert-add <SYMBOL> <above|below|cross|touch> <PRICE> [voice metni] [cooldown]"
    return 1
  fi
  local voice_arg=()
  [ -n "$voice" ] && voice_arg=(--voice "$voice")
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" add \
    --symbol "$sym" --condition "$cond" --price "$price" \
    "${voice_arg[@]}" --cooldown "$cooldown")"
}

# Mevcut alarmı güncelle (eski fiyata göre bulur)
# Kullanım: alert-update <SYMBOL> <cond> <OLD_PRICE> <NEW_PRICE> [voice] [cooldown]
alert-update() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" old="${3:-}" new="${4:-}" voice="${5:-}" cooldown="${6:-}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$old" ]; then
    echo "Kullanım: alert-update <SYMBOL> <cond> <OLD_PRICE> [NEW_PRICE] [voice] [cooldown]"
    return 1
  fi
  local args=(--symbol "$sym" --condition "$cond" --old-price "$old")
  [ -n "$new" ] && args+=(--price "$new")
  [ -n "$voice" ] && args+=(--voice "$voice")
  [ -n "$cooldown" ] && args+=(--cooldown "$cooldown")
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" update "${args[@]}")"
}

# Alarm sil
# Kullanım: alert-remove <SYMBOL> <cond> <PRICE>
alert-remove() {
  _start_guard
  local sym="${1:-}" cond="${2:-}" price="${3:-}"
  if [ -z "$sym" ] || [ -z "$cond" ] || [ -z "$price" ]; then
    echo "Kullanım: alert-remove <SYMBOL> <cond> <PRICE>"
    return 1
  fi
  _alert_apply "$("$CYCLE_ROOT/target/debug/alerts" remove \
    --symbol "$sym" --condition "$cond" --price "$price")"
}

# ============================================================
#  İZLEME
# ============================================================
monitor-start() {
  if tmux has-session -t cycle 2>/dev/null; then
    tmux select-window -t cycle:10
  else
    "$CYCLE_ROOT/additional-services/scripts/monitor.sh"
  fi
}

# ============================================================
#  VERİTABANI
# ============================================================
db-trades() {
  sqlite3 "$CYCLE_ROOT/data-engine/data/market_data.db" \
    "SELECT id,symbol,side,entry_price,exit_price,pnl FROM trades ORDER BY id DESC LIMIT 20;" \
    2>/dev/null || echo "DB boş veya bulunamadı."
}
db-size() {
  du -sh "$CYCLE_ROOT/data-engine/data/market_data.db" 2>/dev/null
}

# ============================================================
#  DETECT-MS  —  Market Structure Multi-Protocol Engine
#  REST API: http://127.0.0.1:3002/api/ms?symbol=BTCUSDT&interval=15m
# ============================================================
DETECT_MS_ADDR="${DETECT_MS_ADDR:-127.0.0.1:3002}"

detect-ms-start() {
  _start_guard
  if pgrep -x "detect-ms" &>/dev/null; then
    echo "⚠️  detect-ms zaten çalışıyor (pid: $(pgrep -x detect-ms))"
    echo "   → detect-ms-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/debug/detect-ms" ]; then
    echo "🔨 detect-ms derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p detect-ms 2>&1 | tail -5
  fi

  echo "🚀 detect-ms başlatılıyor → http://$DETECT_MS_ADDR"
  _tmux_pane "📈DETECT-MS" "cd $CYCLE_ROOT && ./target/debug/detect-ms" Enter
  sleep 1
  if pgrep -x detect-ms &>/dev/null; then
    echo "✅ detect-ms başladı [pid: $(pgrep -x detect-ms)]"
    echo "   API: http://$DETECT_MS_ADDR/api/ms?symbol=BTCUSDT&interval=15m"
  else
    echo "❌ detect-ms başlatılamadı."
  fi
}

detect-ms-stop() {
  _start_guard
  if pgrep -x "detect-ms" &>/dev/null; then
    pkill -TERM -x "detect-ms" && echo "✅ detect-ms durduruldu"
  else
    echo "⚠️  detect-ms zaten çalışmıyor"
  fi
}

detect-ms-status() {
  local pid
  pid=$(pgrep -x "detect-ms" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ detect-ms ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$DETECT_MS_ADDR/api/ms?symbol=BTCUSDT&interval=15m"
  else
    echo "✘  detect-ms durdurulmuş"
  fi
}

# ── calc-ind (İndikatör Hesaplama Motoru :3007) ─────────────
calc-ind-start() {
  _start_guard
  if pgrep -x "calc-ind" &>/dev/null; then
    echo "⚠️  calc-ind zaten çalışıyor (pid: $(pgrep -x calc-ind))"
    echo "   → calc-ind-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/debug/calc-ind" ]; then
    echo "🔨 calc-ind derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p calc-ind 2>&1 | tail -5
  fi

  echo "🚀 calc-ind başlatılıyor → http://127.0.0.1:3007"
  _tmux_pane "🧮CALC-IND" "cd $CYCLE_ROOT && ./target/debug/calc-ind" Enter
  sleep 1
  if pgrep -x calc-ind &>/dev/null; then
    echo "✅ calc-ind başladı [pid: $(pgrep -x calc-ind)]"
    echo "   API: http://127.0.0.1:3007/api/calc"
  else
    echo "❌ calc-ind başlatılamadı."
  fi
}

calc-ind-stop() {
  _start_guard
  if pgrep -x "calc-ind" &>/dev/null; then
    pkill -TERM -x "calc-ind" && echo "✅ calc-ind durduruldu"
  else
    echo "⚠️  calc-ind zaten çalışmıyor"
  fi
}

calc-ind-status() {
  local pid
  pid=$(pgrep -x "calc-ind" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ calc-ind ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://127.0.0.1:3007/api/calc"
  else
    echo "✘  calc-ind durdurulmuş"
  fi
}

# ============================================================
#  AI ENGINE (LLM Agent Katmanı — ai.toml + OpenAI/Anthropic)
#  Bağımlılık: price-feed (:3004), detect-ms (:3002), calc-ind (:3007), paper (:8080)
# ============================================================
AI_ADDR="${AI_ADDR:-127.0.0.1:3110}"

ai-start() {
  _start_guard
  if pgrep -x "ai-engine" &>/dev/null; then
    echo "⚠️  ai-engine zaten çalışıyor (pid: $(pgrep -x ai-engine | head -1))"
    echo "   → ai-stop ile önce durdur"
    return 1
  fi
  if [ ! -f "$CYCLE_ROOT/target/debug/ai-engine" ]; then
    echo "🔨 ai-engine derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p ai-engine 2>&1 | tail -5
  fi
  echo "🚀 ai-engine başlatılıyor → http://$AI_ADDR"
  _tmux_pane "🤖AI" "cd $CYCLE_ROOT && ./target/debug/ai-engine" Enter
  sleep 1
  if pgrep -x ai-engine &>/dev/null; then
    echo "✅ ai-engine başladı [pid: $(pgrep -x ai-engine | head -1)]"
    echo "   Status: http://$AI_ADDR/api/status"
  else
    echo "❌ ai-engine başlatılamadı. (OPENAI_API_KEY / ANTHROPIC_API_KEY gerekli olabilir)"
  fi
}

ai-stop() {
  _start_guard
  if pgrep -x "ai-engine" &>/dev/null; then
    pkill -TERM -x "ai-engine" && echo "✅ ai-engine durduruldu"
  else
    echo "⚠️  ai-engine zaten çalışmıyor"
  fi
}

ai-status() {
  local pid
  pid=$(pgrep -x "ai-engine" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ ai-engine ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    curl -s -m 2 "http://$AI_ADDR/api/status" | python3 -m json.tool 2>/dev/null \
      || echo "   (status API yanıt vermiyor)"
  else
    echo "✘  ai-engine durdurulmuş"
  fi
}

# HITL onayı — /tmp/ai_approve.txt üzerinden
ai-approve() {
  echo "approve" > /tmp/ai_approve.txt
  echo "✅ Onay verildi — bekleyen emir icra edilecek."
}

ai-reject() {
  echo "reject" > /tmp/ai_approve.txt
  echo "❌ Onay reddedildi."
}

ai-log() {
  # ai-engine tmux içinde çalıştığında log'u tmux penceresinden izlemek daha iyidir.
  echo "ℹ️  ai-engine tmux penceresinde çalışıyor; log için pencereye geçin:"
  echo "   tmux select-window -t cycle:11   (veya Ctrl-b + 11)"
}

# ============================================================
#  EXEC CONSOLE (executiond :3010 elle komut konsolu)
# ============================================================
exec-console-start() {
  _start_guard
  if pgrep -x "exec-console" &>/dev/null; then
    echo "⚠️  exec-console zaten çalışıyor (pid: $(pgrep -x exec-console | head -1))"
    return 1
  fi
  if [ ! -f "$CYCLE_ROOT/target/debug/exec-console" ]; then
    echo "🔨 exec-console derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p exec-console 2>&1 | tail -5
  fi
  echo "🚀 exec-console başlatılıyor (executiond :3010 bağlantılı)..."
  _tmux_pane "🖥️CONSOLE" "cd $CYCLE_ROOT && ./target/debug/exec-console" Enter
  sleep 1
  if pgrep -x exec-console &>/dev/null; then
    echo "✅ exec-console başladı [pid: $(pgrep -x exec-console | head -1)]"
    echo "   Sekme: Ctrl+B → 13  |  Komutlar: help"
  else
    echo "❌ exec-console başlatılamadı. (executiond çalışıyor mu? EXEC_ADMIN_PASS doğru mu?)"
  fi
}

exec-console-stop() {
  _start_guard
  if pgrep -x "exec-console" &>/dev/null; then
    pkill -TERM -x "exec-console" && echo "✅ exec-console durduruldu"
  else
    echo "⚠️  exec-console zaten çalışmıyor"
  fi
}

exec-console-status() {
  local pid
  pid=$(pgrep -x "exec-console" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ exec-console ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   Sekme: Ctrl+B → 13"
  else
    echo "✘  exec-console durdurulmuş"
  fi
}

exec-console-log() {
  echo "ℹ️  Konsol tmux penceresinde çalışıyor; geçmek için:"
  echo "   tmux select-window -t cycle:12   (veya Ctrl-b + 12)"
}

# Sorgu kısayolları
detect-ms-query() {
  # Kullanım: detect-ms-query [SYMBOL] [INTERVAL] [LIMIT]
  local sym="${1:-BTCUSDT}" itv="${2:-15m}" lim="${3:-200}"
  echo "📡 Sorgu: $sym $itv (limit: $lim) → http://$DETECT_MS_ADDR"
  curl -s "http://$DETECT_MS_ADDR/api/ms?symbol=${sym}&interval=${itv}&limit=${lim}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. detect-ms-start ile başlat."
}

detect-ms-log() {
  tail -f /tmp/detect_ms.log
}

# ============================================================
#  STREAM-OHLCV  (stream-ohlcv — canlı OHLCV mum akışı :3008)
#  istek: {symbol, start_ms, interval_secs} → POST /api/stream
#  mumlar binary olarak /dev/shm/cycle_finance_stream_ohlcv ring'ine yazılır.
# ============================================================
STREAM_OHLCV_ADDR="${STREAM_OHLCV_ADDR:-127.0.0.1:3008}"

stream-ohlcv-start() {
  _start_guard
  if pgrep -x stream-ohlcv &>/dev/null; then
    echo "⚠️  stream-ohlcv zaten çalışıyor (pid: $(pgrep -x stream-ohlcv | head -1))"
    return 1
  fi
  if [ ! -f "$CYCLE_ROOT/target/debug/stream-ohlcv" ]; then
    echo "🔨 stream-ohlcv derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p stream-ohlcv 2>&1 | tail -5
  fi
  echo "🚀 stream-ohlcv başlatılıyor → http://$STREAM_OHLCV_ADDR"
  _tmux_pane "📡STREAM-OHLCV" "cd $CYCLE_ROOT && ./target/debug/stream-ohlcv" Enter
  sleep 1
  if pgrep -x stream-ohlcv &>/dev/null; then
    echo "✅ stream-ohlcv başladı [pid: $(pgrep -x stream-ohlcv | head -1)]"
    echo "   POST http://$STREAM_OHLCV_ADDR/api/stream  {symbol, start_ms, interval_secs}"
  else
    echo "❌ stream-ohlcv başlatılamadı."
  fi
}

stream-ohlcv-stop() {
  _start_guard
  if pgrep -x stream-ohlcv &>/dev/null; then
    pkill -TERM -x stream-ohlcv && echo "✅ stream-ohlcv durduruldu"
  else
    echo "⚠️  stream-ohlcv zaten çalışmıyor"
  fi
}

stream-ohlcv-status() {
  local pid
  pid=$(pgrep -x stream-ohlcv 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ stream-ohlcv ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$STREAM_OHLCV_ADDR/api/health"
  else
    echo "✘  stream-ohlcv durdurulmuş"
  fi
}

# Kullanım: stream-ohlcv-start-stream [SYMBOL] [INTERVAL_SN] [START_MS]
stream-ohlcv-start-stream() {
  local sym="${1:-BTCUSDT}" itv="${2:-60}" start="${3:-0}"
  echo "📡 Stream açılıyor: $sym interval=${itv}s start_ms=${start}"
  curl -s -X POST "http://$STREAM_OHLCV_ADDR/api/stream" \
    -H "Content-Type: application/json" \
    -d "{\"symbol\":\"$sym\",\"start_ms\":$start,\"interval_secs\":$itv}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. stream-ohlcv-start ile başlat."
}

# Kullanım: stream-ohlcv-query [SYMBOL] [INTERVAL_SN] [START_MS]
stream-ohlcv-query() {
  local sym="${1:-BTCUSDT}" itv="${2:-60}" start="${3:-0}"
  echo "📡 Sorgu: $sym ${itv}s → http://$STREAM_OHLCV_ADDR"
  curl -s -X POST "http://$STREAM_OHLCV_ADDR/api/stream" \
    -H "Content-Type: application/json" \
    -d "{\"symbol\":\"$sym\",\"start_ms\":$start,\"interval_secs\":$itv}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor."
}

# Kullanım: stream-ohlcv-streams
stream-ohlcv-streams() {
  echo "📡 Aktif stream'ler: http://$STREAM_OHLCV_ADDR/api/streams"
  curl -s "http://$STREAM_OHLCV_ADDR/api/streams" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor."
}

# ============================================================
#  BREAKOUT STRATEJİSİ  (services-engine/strategies/breakout-strategy)
#  Strateji, STRATEGY orkestrasyon konsolu tarafından yönetilir:
#  breakout-start/stop → strat run/stop breakout komutunu iletir.
# ============================================================
breakout-start() {
  _start_guard
  strat run breakout
}

breakout-stop() {
  _start_guard
  strat stop breakout
}

breakout-status() {
  _start_guard
  strat status
}

breakout-log() {
  echo "📌 Strateji çıktısı STRATEGY konsolunda (pencere 1) görünür."
  echo "   Konsola geçmek için: strat attach"
  strat attach
}

# Bekleme süresini saniye cinsinden ayarla (çalışan strateji bir sonraki döngüde uygular)
# Kullanım: breakout-wait 600   (10 dakika)  |  breakout-wait 1200  (20 dakika)
breakout-wait() {
  _start_guard
  local sec="${1:-}"
  if [ -z "$sec" ]; then
    local cur; cur=$(cat /tmp/breakout_wait_sec.txt 2>/dev/null || echo "1200")
    echo "ℹ️  Mevcut bekleme: $cur sn"
    echo "Kullanım: breakout-wait <saniye>   (örn. breakout-wait 600 → 10dk)"
    return 0
  fi
  if ! echo "$sec" | grep -qE '^[0-9]+$' || [ "$sec" -lt 10 ]; then
    echo "❌ Saniye değeri geçerli değil (min 10): $sec"
    return 1
  fi
  echo "$sec" > /tmp/breakout_wait_sec.txt
  echo "✅ Bekleme süresi ayarlandı: $sec sn ($((sec/60)) dk)"
  echo "   Çalışan strateji bir sonraki döngüde bu değeri kullanır."
  if pgrep -f "breakout-strategy" >/dev/null 2>&1; then
    echo "   ℹ️  Strateji çalışıyor — yeni süre otomatik uygulanacak."
  fi
}

breakout-query() {
  # Kullanım: breakout-query [--dry-run]
  if [ "${1:-}" = "--dry-run" ]; then
    cd "$CYCLE_ROOT" && $CYCLE_ROOT/target/debug/breakout-strategy --once --dry-run
  else
    cd "$CYCLE_ROOT" && $CYCLE_ROOT/target/debug/breakout-strategy --once
  fi
}

# ============================================================
#  EXECUTION ENGINE  (Canlı Binance Futures — executiond :3010)
#  Anahtarları SHELL panelinden güvenli girmek için:
#    exec-setup     → anahtar gir (ekrana yazılmaz, .env 600)
#    exec-show      → mevcut yapılandırma (anahtarlar maskeli)
# ============================================================
exec-setup() {
  _start_guard
  "$CYCLE_ROOT/additional-services/scripts/exec_setup.sh"
}
exec-show() {
  _start_guard
  "$CYCLE_ROOT/additional-services/scripts/exec_setup.sh" --show
}
exec-testnet() {
  _start_guard
  "$CYCLE_ROOT/additional-services/scripts/exec_setup.sh" --testnet
  echo "✅ Testnet ayarlandı. exec-dry ile başlatın."
}
exec-dry() {
  _start_guard
  if pgrep -x executiond &>/dev/null; then echo "⚠️  executiond zaten çalışıyor → exec-stop"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p execution-engine --bins 2>&1 | tail -1
  _tmux_pane "🛡️EXEC" "cd $CYCLE_ROOT && EXEC_MODE=LIVE EXEC_DRY_RUN=true ./target/debug/executiond" Enter
  sleep 2
  exec-status
}
exec-live() {
  _start_guard
  if pgrep -x executiond &>/dev/null; then echo "⚠️  executiond zaten çalışıyor → exec-stop"; return 1; fi
  echo ""
  echo -e "${_R}⚠️  GERÇEK EMİR MODU (EXEC_DRY_RUN=false)${_N}"
  echo -e "${_R}    Emirler gerçek Binance hesabına gidecek.${_N}"
  echo "    Devam etmek için 'GO' yazın:"
  local onay
  read -r onay
  if [ "$onay" != "GO" ]; then echo "İptal."; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p execution-engine --bins 2>&1 | tail -1
  _tmux_pane "🛡️EXEC" "cd $CYCLE_ROOT && EXEC_DRY_RUN=false ./target/debug/executiond" Enter
  sleep 2
  exec-status
}
exec-stop() {
  _start_guard
  local p; p=$(pgrep -x executiond 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    kill -TERM "$p" 2>/dev/null; sleep 1
    kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null
    echo "✅ executiond durduruldu [pid:$p]"
  else
    echo "ℹ️  executiond çalışmıyor"
  fi
}
exec-status() {
  _start_guard
  local pid; pid=$(pgrep -x executiond 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    echo "✅ executiond ÇALIŞIYOR [pid:$pid]"
    curl -s -m 2 http://127.0.0.1:3010/api/v1/mode  | python3 -m json.tool 2>/dev/null
    curl -s -m 2 http://127.0.0.1:3010/api/v1/risk   | python3 -m json.tool 2>/dev/null
  else
    echo "✘ executiond durdurulmuş"
  fi
}
exec-account()   { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" account; }
exec-positions() { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" positions "${1:-}"; }
exec-balance()   { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" balance; }
exec-orders()    { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" orders "${1:-}"; }
exec-order()     { _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" order "$@"; }
exec-cancel-all(){ _start_guard; "$CYCLE_ROOT/target/debug/exec-cli" cancel-all "${1:-}"; }
exec-kill()      { _start_guard; touch "${EXEC_KILL_SWITCH_PATH:-/tmp/exec_kill_switch}"; echo "⚠️  Kill switch AÇIK — yeni emirler reddedilir"; }
exec-unkill()    { _start_guard; rm -f "${EXEC_KILL_SWITCH_PATH:-/tmp/exec_kill_switch}"; echo "Kill switch kapatıldı."; }

# ── Yüklendiğini bildir ──────────────────────────────────────
echo -e "${_D}[cycle_env] Yüklendi — ROOT: $CYCLE_ROOT | API: $CYCLE_API${_N}"
