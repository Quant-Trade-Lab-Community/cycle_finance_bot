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
  echo -e "  ${_G}strategy-start${_N} / ${_R}strategy-stop${_N}  STRATEGY terminali (PyO3)"
  echo -e "  ${_G}paper-start${_N} / ${_R}paper-stop${_N}        Paper-service (REST :8080)"
  echo -e "  ${_G}alert-start${_N} / ${_R}alert-stop${_N}        Alert-service"
  echo -e "  ${_G}listener-start${_N} / ${_R}listener-stop${_N}  Listener (anlık metrik analizi)"
  echo -e "  ${_G}detect-ms-start${_N} / ${_R}detect-ms-stop${_N}  MSMP analiz motoru (:3002)"
  echo -e "  ${_G}heiusdt-start${_N} / ${_R}heiusdt-stop${_N}    HEIUSDT kırılım stratejisi"

  echo -e "\n${_Y}━━━  🛰️  LISTENER  (Anlık Metrik Analizi)  ━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}listener-start${_N}      Pane 0.2'de başlat"
  echo -e "  ${_C}listener-stop${_N}       Durdur"
  echo -e "  ${_C}listener-status${_N}     Çalışıyor mu? CPU/RAM"
  echo -e "  ${_C}listenconfig-list${_N}   Metrik parametrelerini göster"
  echo -e "  ${_C}listenconfig-set KEY VAL${_N}  Parametre değiştir (lambda, k_abs, gamma...) "
  echo -e "  ${_C}listenconfig-reset${_N}  Varsayılanlara dön"
  echo -e "  ${_C}listener-log${_N}        Metrik çıktısını izle (/tmp/listener_metrics.json)"

  echo -e "\n${_Y}━━━  ⚠️  RİSK ANALİZİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}risk-start${_N}           Pane 0.1'de başlat (5 sn yenileme)"
  echo -e "  ${_C}risk-stop${_N}            Durdur"
  echo -e "  ${_C}risk-query${_N}           Tek seferlik analiz çalıştır"

  echo -e "\n${_Y}━━━  💹 PRICE-FEED  (WS→Ring, Anlık Last/Mark/Index)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}pricefeed-start${_N}     Arka planda başlat (:3004)"
  echo -e "  ${_C}pricefeed-stop${_N}      Durdur"
  echo -e "  ${_C}pricefeed-status${_N}    Çalışıyor mu? CPU/RAM + health"
  echo -e "  ${_C}pricefeed-query SYM${_N} Tek sembol sorgula (örn. pricefeed-query HEIUSDT)"
  echo -e "  ${_C}pricefeed-log${_N}       Canlı log izle"

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
  echo -e "  ${_C}strategy-start${_N}       Strategy terminalini başlat (arka plan)"
  echo -e "  ${_C}strategy-stop${_N}        Strategy terminalini durdur"
  echo -e "  ${_C}correlation-start${_N}    Korelasyon analizini başlat"

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

  echo -e "\n${_Y}━━━  🏛️  DETECT-WYCKOFF  (Wyckoff Faz Motoru :3005)  ━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}detect-wyckoff-start${_N}  Servisi başlat (port 3005)"
  echo -e "  ${_C}detect-wyckoff-stop${_N}   Servisi durdur"
  echo -e "  ${_C}detect-wyckoff-status${_N} Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}detect-wyckoff-query${_N}  BTCUSDT 1h analiz (JSON çıktı)"
  echo -e "  ${_C}detect-wyckoff-query${_N}  HEIUSDT 15m 500${_N}   Özel sorgu"

  echo -e "\n${_Y}━━━  🌊 DETECT-TRB  (Navier-Stokes Çözücü :3006)  ━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}detect-trb-start${_N}      Servisi başlat (port 3006)"
  echo -e "  ${_C}detect-trb-stop${_N}       Servisi durdur"
  echo -e "  ${_C}detect-trb-status${_N}     Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}detect-trb-query${_N}      Son raporu göster (JSON çıktı)"
  echo -e "  ${_C}detect-trb-start --symbol ETHUSDT --port 3007${_N}   Özel parametreler"

  echo -e "\n${_Y}━━━  🎯 HEIUSDT KIRILIM STRATEJİSİ  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}heiusdt-start${_N}        Stratejiyi başlat (HEIUSDT 1m, 100 pencere)"
  echo -e "  ${_C}heiusdt-stop${_N}         Stratejiyi durdur"
  echo -e "  ${_C}heiusdt-status${_N}       Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}heiusdt-query${_N}        Tek seferlik analiz (emir açmaz)"
  echo -e "  ${_C}heiusdt-query --dry-run${_N}  Analiz + kırılım simülasyonu"
  echo -e "  ${_C}heiusdt-wait 600${_N}     Bekleme süresini ayarla (saniye)"
  echo -e "  ${_C}heiusdt-log${_N}          Canlı strateji logu izle"

  echo -e "\n${_Y}━━━  📊 İZLEME  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}monitor-start${_N}        İzleme paneline geç (Ctrl+B → 4)"

  echo -e "\n${_Y}━━━  🗄️  VERİTABANI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}db-trades${_N}            Son 20 işlemi göster"
  echo -e "  ${_C}db-size${_N}              Veritabanı boyutu"

  echo -e "\n${_Y}━━━  🌐 TMUX KISAYOLLARI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_B}Ctrl+B → ok tuşu${_N}     Panel değiştir"
  echo -e "  ${_B}Ctrl+B → z${_N}           Paneli tam ekran yap / küçült"
  echo -e "  ${_B}Ctrl+B → d${_N}           Session'ı arka plana al"
  echo -e "  ${_B}Ctrl+B → 0${_N}           Trading sekmesi (4 panel)"
  echo -e "  ${_B}Ctrl+B → 1${_N}           📡 DATA sekmesi"
  echo -e "  ${_B}Ctrl+B → 2${_N}           🔔 ALERT sekmesi"
  echo -e "  ${_B}Ctrl+B → 3${_N}           🛡️ PAPER sekmesi"
  echo -e "  ${_B}Ctrl+B → 4${_N}           Monitor sekmesi"
  echo -e "  ${_B}Ctrl+B → 5${_N}           DETECT-MS sekmesi"
  echo -e "  ${_B}Ctrl+B → 6${_N}           HEIUSDT sekmesi"
  echo -e "  ${_B}Ctrl+B → 7${_N}           WYCKOFF sekmesi"
  echo -e "  ${_B}Fare tıklama/scroll${_N}  Panel seç / scroll"

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
  cd "$CYCLE_ROOT" && cargo build -p core -p paper-service -p alert-service
}
cycle-build-full() {
  cd "$CYCLE_ROOT" && cargo build -p paper-service --features full
}

# ============================================================
#  SİSTEMLERİ TEK TEK AÇ / KAPAT  (4 panelli Trading penceresi)
#  DATA, ALERT ve PAPER ayrı sekme (pencere) olarak açılır.
#  Her servis kendi pane'inde başlar.
# ============================================================
# Yardımcı: Trading penceresindeki bir pane'e komut gönder
# Servis → hedef: 0.0=STRATEGY 0.2=LISTENER 0.1=RISK 0.3=SHELL
#                1=DATA sekmesi  2=ALERT sekmesi  3=PAPER sekmesi
_tmux_pane() {
  local name="$1"; shift
  local session="cycle"
  local pane
  case "$name" in
    "📡DATA")   pane="1" ;;
    "🛡️PAPER")  pane="3" ;;
    "🧠STRATEGY") pane="0.0" ;;
    "🔔ALERT")  pane="2" ;;
    "🛰️LISTENER") pane="0.2" ;;
    "⚠️RISK")  pane="0.1" ;;
    "💻SHELL")  pane="0.3" ;;
    *)
      # Tanınmayan → yeni pencere (ör. DETECT-MS, HEIUSDT)
      if ! tmux has-session -t "$session" 2>/dev/null; then
        tmux new-session -d -s "$session" -x 220 -y 50
        tmux rename-window -t "$session:0" "Trading"
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

# ── DATA terminali (Binance WS → ring) ──────────────────────
# RUN_MODE env değişkeni ps'de görünmez → /proc/*/environ ile kontrol et
_core_mode_pid() {
  local mode="$1"
  for p in $(pgrep -x core 2>/dev/null); do
    if tr '\0' '\n' < "/proc/$p/environ" 2>/dev/null | grep -q "^RUN_MODE=$mode$"; then
      echo "$p"
      return 0
    fi
  done
  return 1
}

data-start() {
  _start_guard
  if _core_mode_pid DATA &>/dev/null; then echo "⚠️  DATA zaten çalışıyor (pid: $(_core_mode_pid DATA))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p core 2>&1 | tail -1
  rm -f /dev/shm/demir_yumruk_ring /dev/shm/demir_yumruk_orders
  _tmux_pane "📡DATA" "cd $CYCLE_ROOT && RUN_MODE=DATA ./target/debug/core" Enter
  echo "✅ DATA başlatıldı (sekme 1 — 📡 DATA)"
}
data-stop() {
  _start_guard
  local p; p=$(_core_mode_pid DATA)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; echo "✅ DATA durduruldu [pid:$p]"; else echo "ℹ️  DATA çalışmıyor"; fi
}

# ── STRATEGY terminali (core) ────────────────────────────────
strategy-start() {
  _start_guard
  if _core_mode_pid STRATEGY &>/dev/null; then echo "⚠️  STRATEGY zaten çalışıyor (pid: $(_core_mode_pid STRATEGY))"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p core 2>&1 | tail -1
  _tmux_pane "🧠STRATEGY" "cd $CYCLE_ROOT && RUN_MODE=STRATEGY ./target/debug/core" Enter
  echo "✅ STRATEGY başlatıldı (pane 0.0)"
}
strategy-stop() {
  _start_guard
  local p; p=$(_core_mode_pid STRATEGY)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; echo "✅ STRATEGY durduruldu [pid:$p]"; else echo "ℹ️  STRATEGY çalışmıyor"; fi
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
  echo "✅ PAPER-SERVICE başlatıldı (sekme 3 — 🛡️ PAPER, http://127.0.0.1:8080)"
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
  echo "✅ ALERT-SERVICE başlatıldı (sekme 2 — 🔔 ALERT)"
}
alert-stop() {
  _start_guard
  local p; p=$(pgrep -x alert-service 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then kill -TERM "$p" 2>/dev/null; sleep 1; kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null; echo "✅ alert-service durduruldu [pid:$p]"; else echo "ℹ️  alert-service çalışmıyor"; fi
}

# ── LISTENER (Anlık Metrik Analizi, pane 0.1) ──────────
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
  _tmux_pane "🛰️LISTENER" "cd $CYCLE_ROOT && $CYCLE_ROOT/target/debug/listener" Enter
  sleep 2
  if pgrep -x listener &>/dev/null; then
    echo "✅ LISTENER başlatıldı (pane 0.2)"
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

# ── RISK (Anlık risk analizi, pane 0.3) ──────────────────────
risk-start() {
  _start_guard
  if pgrep -x risk_analysis &>/dev/null; then
    echo "⚠️  RISK zaten çalışıyor (pid: $(pgrep -x risk_analysis | head -1))"
    return 1
  fi
  _tmux_pane "⚠️RISK" "cd $CYCLE_ROOT && ./target/debug/risk_analysis --watch" Enter
  sleep 2
  echo "✅ RISK başlatıldı (pane 0.1)"
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
  cd "$CYCLE_ROOT" && ./target/debug/risk_analysis
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
# Not: strategy-start/stop artık "SİSTEMLERİ TEK TEK AÇ/KAPAT" bölümünde
# (arka planda, pid dosyalı). correlation-start foreground çalıştırır.
correlation-start() {
  cd "$CYCLE_ROOT" && RUN_MODE=CORRELATION ./target/debug/core
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
  tmux send-keys -t "cycle:2" C-c 2>/dev/null
  tmux send-keys -t "cycle:2" "cd $CYCLE_ROOT && ./target/debug/alert-service --config $CYCLE_ROOT/alerts.toml" Enter 2>/dev/null
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
    tmux select-window -t cycle:4
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
#  DETECT-WYCKOFF  —  Wyckoff Faz Analiz Motoru
#  REST API: http://127.0.0.1:3005/api/wyckoff?symbol=BTCUSDT&interval=1h
# ============================================================
DETECT_WYCKOFF_ADDR="${DETECT_WYCKOFF_ADDR:-127.0.0.1:3005}"

detect-wyckoff-start() {
  _start_guard
  if pgrep -x "detect-wyckoff" &>/dev/null; then
    echo "⚠️  detect-wyckoff zaten çalışıyor (pid: $(pgrep -x detect-wyckoff))"
    echo "   → detect-wyckoff-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/debug/detect-wyckoff" ]; then
    echo "🔨 detect-wyckoff derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p detect-wyckoff 2>&1 | tail -5
  fi

  echo "🚀 detect-wyckoff başlatılıyor → http://$DETECT_WYCKOFF_ADDR"
  _tmux_pane "🏛️WYCKOFF" "cd $CYCLE_ROOT && ./target/debug/detect-wyckoff" Enter
  sleep 1
  if pgrep -x detect-wyckoff &>/dev/null; then
    echo "✅ detect-wyckoff başladı [pid: $(pgrep -x detect-wyckoff)]"
    echo "   API: http://$DETECT_WYCKOFF_ADDR/api/wyckoff?symbol=BTCUSDT&interval=1h"
  else
    echo "❌ detect-wyckoff başlatılamadı."
  fi
}

detect-wyckoff-stop() {
  _start_guard
  if pgrep -x "detect-wyckoff" &>/dev/null; then
    pkill -TERM -x "detect-wyckoff" && echo "✅ detect-wyckoff durduruldu"
  else
    echo "⚠️  detect-wyckoff zaten çalışmıyor"
  fi
}

detect-wyckoff-status() {
  local pid
  pid=$(pgrep -x "detect-wyckoff" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ detect-wyckoff ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$DETECT_WYCKOFF_ADDR/api/wyckoff?symbol=BTCUSDT&interval=1h"
  else
    echo "✘  detect-wyckoff durdurulmuş"
  fi
}

# Sorgu kısayolu — Kullanım: detect-wyckoff-query [SYMBOL] [INTERVAL] [LIMIT]
detect-wyckoff-query() {
  local sym="${1:-BTCUSDT}" itv="${2:-1h}" lim="${3:-300}"
  echo "🏛️  Sorgu: $sym $itv (limit: $lim) → http://$DETECT_WYCKOFF_ADDR"
  curl -s "http://$DETECT_WYCKOFF_ADDR/api/wyckoff?symbol=${sym}&interval=${itv}&limit=${lim}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. detect-wyckoff-start ile başlat."
}

# ============================================================
#  DETECT-TRB  —  Navier-Stokes Çözücü / Kavitasyon Motoru
#  REST API: http://127.0.0.1:3006/api/trb  (+ /api/trb/status)
# ============================================================
DETECT_TRB_ADDR="${DETECT_TRB_ADDR:-127.0.0.1:3006}"

detect-trb-start() {
  _start_guard
  if pgrep -x "detect-trb" &>/dev/null; then
    echo "⚠️  detect-trb zaten çalışıyor (pid: $(pgrep -x detect-trb))"
    echo "   → detect-trb-stop ile önce durdur"
    return 1
  fi

  # Derle (yoksa)
  if [ ! -f "$CYCLE_ROOT/target/release/detect-trb" ]; then
    echo "🔨 detect-trb derleniyor..."
    cd "$CYCLE_ROOT" && cargo build --release -p detect-trb 2>&1 | tail -5
  fi

  echo "🚀 detect-trb başlatılıyor → http://$DETECT_TRB_ADDR"
  # Ek parametreler (ör. --symbol, --port) start'a geçirilebilir
  _tmux_pane "🌊TRB" "cd $CYCLE_ROOT && ./target/release/detect-trb $*" Enter
  sleep 1
  if pgrep -x detect-trb &>/dev/null; then
    echo "✅ detect-trb başladı [pid: $(pgrep -x detect-trb)]"
    echo "   API: http://$DETECT_TRB_ADDR/api/trb/status"
  else
    echo "❌ detect-trb başlatılamadı."
  fi
}

detect-trb-stop() {
  _start_guard
  if pgrep -x "detect-trb" &>/dev/null; then
    pkill -TERM -x "detect-trb" && echo "✅ detect-trb durduruldu"
  else
    echo "⚠️  detect-trb zaten çalışmıyor"
  fi
}

detect-trb-status() {
  local pid
  pid=$(pgrep -x "detect-trb" 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ detect-trb ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$DETECT_TRB_ADDR/api/trb/status"
  else
    echo "✘  detect-trb durdurulmuş"
  fi
}

# Son raporu göster — Kullanım: detect-trb-query
detect-trb-query() {
  curl -s "http://$DETECT_TRB_ADDR/api/trb" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. detect-trb-start ile başlat."
}

# ============================================================
#  HEIUSDT KIRILIM STRATEJİSİ  (strategies/heiusdt_breakout.py)
#  detect-ms + paper-service kullanır. HEIUSDT 1m, 100 pencere,
#  her 20 pencerede bir analiz.
# ============================================================
heiusdt-start() {
  _start_guard
  if pgrep -x heiusdt &>/dev/null; then
    echo "⚠️  HEIUSDT stratejisi zaten çalışıyor (pid: $(pgrep -f '[h]eiusdt_breakout.py' | head -1))"
    return 1
  fi
  # Bağımlılık kontrolü
  if ! curl -s -o /dev/null -w "%{http_code}" "http://$DETECT_MS_ADDR/api/ms?symbol=HEIUSDT&interval=1m&limit=5" 2>/dev/null | grep -q 200; then
    echo "⚠️  detect-ms yanıt vermiyor → heiusdt-start ile başlatın"
    return 1
  fi
  echo "🎯 HEIUSDT stratejisi başlatılıyor (HEIUSDT 1m, 100 pencere, 20 pencere/kontrol)..."
  _tmux_pane "🎯HEIUSDT" "cd $CYCLE_ROOT && $CYCLE_ROOT/target/debug/heiusdt" Enter
  sleep 2
  if pgrep -x heiusdt &>/dev/null; then
    echo "✅ HEIUSDT stratejisi başladı [pid: $(pgrep -f '[h]eiusdt_breakout.py' | head -1)]"
    echo "   Pencere: cycle → 🎯HEIUSDT"
  else
    echo "❌ HEIUSDT stratejisi başlatılamadı."
  fi
}

heiusdt-stop() {
  _start_guard
  local pid
  pid=$(pgrep -x heiusdt 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    pkill -TERM -f "[h]eiusdt_breakout.py" 2>/dev/null
    sleep 1
    pkill -KILL -f "[h]eiusdt_breakout.py" 2>/dev/null || true
    echo "✅ HEIUSDT stratejisi durduruldu [pid:$pid]"
  else
    echo "⚠️  HEIUSDT stratejisi zaten çalışmıyor"
  fi
}

heiusdt-status() {
  local pid
  pid=$(pgrep -x heiusdt 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ HEIUSDT stratejisi ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
  else
    echo "✘  HEIUSDT stratejisi durdurulmuş"
  fi
}

heiusdt-log() {
  tail -f /tmp/heiusdt.log
}

# Bekleme süresini saniye cinsinden ayarla (çalışan strateji bir sonraki döngüde uygular)
# Kullanım: heiusdt-wait 600   (10 dakika)  |  heiusdt-wait 1200  (20 dakika)
heiusdt-wait() {
  _start_guard
  local sec="${1:-}"
  if [ -z "$sec" ]; then
    local cur; cur=$(cat /tmp/heiusdt_wait_sec.txt 2>/dev/null || echo "1200")
    echo "ℹ️  Mevcut bekleme: $cur sn"
    echo "Kullanım: heiusdt-wait <saniye>   (örn. heiusdt-wait 600 → 10dk)"
    return 0
  fi
  if ! echo "$sec" | grep -qE '^[0-9]+$' || [ "$sec" -lt 10 ]; then
    echo "❌ Saniye değeri geçerli değil (min 10): $sec"
    return 1
  fi
  echo "$sec" > /tmp/heiusdt_wait_sec.txt
  echo "✅ Bekleme süresi ayarlandı: $sec sn ($((sec/60)) dk)"
  echo "   Çalışan strateji bir sonraki döngüde bu değeri kullanır."
  if pgrep -x heiusdt >/dev/null 2>&1; then
    echo "   ℹ️  Strateji çalışıyor — yeni süre otomatik uygulanacak."
  fi
}

heiusdt-query() {
  # Kullanım: heiusdt-query [--dry-run]
  if [ "${1:-}" = "--dry-run" ]; then
    cd "$CYCLE_ROOT" && $CYCLE_ROOT/target/debug/heiusdt --once --dry-run
  else
    cd "$CYCLE_ROOT" && $CYCLE_ROOT/target/debug/heiusdt --once
  fi
}

# ── Yüklendiğini bildir ──────────────────────────────────────
echo -e "${_D}[cycle_env] Yüklendi — ROOT: $CYCLE_ROOT | API: $CYCLE_API${_N}"
