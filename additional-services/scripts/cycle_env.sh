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
  echo -e "  ${_G}flows-start${_N} / ${_R}flows-stop${_N}       8 veri akışı (WS→Ring→TimescaleDB)"
  echo -e "  ${_G}strategy-start${_N} / ${_R}strategy-stop${_N}  STRATEGY orkestrasyon konsolu"
  echo -e "  ${_G}paper-start${_N} / ${_R}paper-stop${_N}        Paper-service (REST :8080)"
  echo -e "  ${_G}alert-start${_N} / ${_R}alert-stop${_N}        Alert-service"
  echo -e "  ${_G}listener-start${_N} / ${_R}listener-stop${_N}  Listener (anlık metrik analizi)"
  echo -e "  ${_G}detect-ms-start${_N} / ${_R}detect-ms-stop${_N}  MSMP analiz motoru (:3002)"
  echo -e "  ${_G}calc-ind-start${_N} / ${_R}calc-ind-stop${_N}    İndikatör hesaplama motoru (:3007)"
  echo -e "  ${_G}stream-ohlcv-start${_N} / ${_R}stream-ohlcv-stop${_N}  Canlı OHLCV mum akışı (:3008)"

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

  echo -e "  ${_C}flows-start${_N}         8 akışı ayrı tmux sekmelerinde başlat"
  echo -e "  ${_C}flows-stop${_N}          Tüm akışları durdur"
  echo -e "  ${_C}flows-status${_N}        Akışların durumu (CPU/RAM)"

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
  echo -e "  ${_C}alert-add VELVETUSDT above 0.22 \"ses\"${_N}   Yeni alarm ekle"
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
  echo -e "  ${_C}strat run breakout${_N}      Stratejiyi başlat (VELVETUSDT 1m, 100 pencere)"
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

  echo -e "\n${_Y}━━━  ⏱ TRADE-OHLCV  (Trade → 1s OHLCV :3009)  ━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}trade-ohlcv-start${_N}    Servisi başlat (kaynak: /dev/shm/cycle_finance_trades)"
  echo -e "  ${_C}trade-ohlcv-stop${_N}     Servisi durdur"
  echo -e "  ${_C}trade-ohlcv-status${_N}   Çalışıyor mu? CPU/RAM göster"
  echo -e "  ${_C}trade-ohlcv-live${_N}     Canlı 1s OHLCV akışını izle (tmux pencere 20)"
  echo -e "  ${_C}trade-ohlcv-symbols${_N}  Takip edilen sembolleri listele"
  echo -e "  ${_C}trade-ohlcv-candles SYM [N]${_N}   Son N kapalı 1s mumu göster (örn. BTCUSDT 30)"

  echo -e "\n${_Y}━━━  📊 İZLEME  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}monitor-start${_N}        İzleme paneline geç (Ctrl+B → 10)"
  echo -e "  ${_C}cycle-ip now${_N}         Güncel public IP'yi göster (Binance whitelist)"
  echo -e "  ${_C}cycle-ip check${_N}       IP değişti mi? (kayıtlı ile karşılaştırır, uyarır)"
  echo -e "  ${_C}cycle-ip record${_N}      Güncel IP'yi kaydet"
  echo -e "  ${_C}cycle-ip watch [SN]${_N}  IP'yi sürekli izle (varsayılan 300s)"

  echo -e "\n${_Y}━━━  🗄️  VERİTABANI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_C}db-trades${_N}            Son 20 işlemi göster"
  echo -e "  ${_C}db-size${_N}              Veritabanı boyutu"

  echo -e "\n${_Y}━━━  🌐 TMUX KISAYOLLARI  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${_N}"
  echo -e "  ${_B}Ctrl+B → ok tuşu${_N}     Pencere değiştir"
  echo -e "  ${_B}Ctrl+B → d${_N}           Session'ı arka plana al"
  echo -e "  ${_B}Ctrl+B → 0${_N}           💻 SHELL sekmesi (orkestrasyon komutları)"
  echo -e "  ${_B}Ctrl+B → 1${_N}           🧠 STRATEGY sekmesi (orkestrasyon konsolu)"
  echo -e "  ${_B}Ctrl+B → 2${_N}           📈 DETECT-MS sekmesi"
  echo -e "  ${_B}Ctrl+B → 3${_N}           🧮 CALC-IND sekmesi"
  echo -e "  ${_B}Ctrl+B → 4${_N}           📊 STREAM-OHLCV sekmesi"
  echo -e "  ${_B}Ctrl+B → 5${_N}           🛡️ PAPER sekmesi"
  echo -e "  ${_B}Ctrl+B → 6${_N}           ⚠️ RISK sekmesi"
  echo -e "  ${_B}Ctrl+B → 7${_N}           🔔 ALERT sekmesi"
  echo -e "  ${_B}Ctrl+B → 8${_N}           Monitor sekmesi"
  echo -e "  ${_B}Ctrl+B → 9${_N}           🖥️ CONSOLE sekmesi"
  echo -e "  ${_B}Ctrl+B → 10-17${_N}       💹 Veri akışları (FLOWS)"
  echo -e "  ${_B}Ctrl+B → 18${_N}          🛢️ DB-QUERY (TimescaleDB sorgu paneli)"
  echo -e "  ${_B}Ctrl+B → 20${_N}          ⏱ TRADE-OHLCV (canlı 1s OHLCV akışı)"
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
  cd "$CYCLE_ROOT" && cargo build -p engine -p paper-service -p alert-service -p strategies-engine
}
cycle-build-full() {
  cd "$CYCLE_ROOT" && cargo build -p paper-service --features full
}

# ── IP yardımcıları (Binance whitelist için public IP takibi) ──
# cycle-ip now|record|check|state|watch [SN]  → cycle_ip.sh'e iletir
cycle-ip() {
  bash "$CYCLE_ROOT/additional-services/scripts/cycle_ip.sh" "$@"
}

# ============================================================
#  SİSTEMLERİ TEK TEK AÇ / KAPAT
#  Her servis ayrı sekme (pencere) olarak açılır.
# ============================================================
# Yardımcı: ilgili pencereye komut gönder
# Pencere haritası: 0=SHELL 1=STRATEGY 2=DETECT-MS
#                   3=CALC-IND 4=STREAM-OHLCV 5=PAPER
#                   6=RISK 7=ALERT 8=Monitor 9=CONSOLE
#                   10-17=FLOWS (8 veri akışı) 18=DB-QUERY 19=TELEGRAM
#                   20=TRADE-OHLCV
_tmux_pane() {
  local name="$1"; shift
  local session="cycle"
  local pane
  case "$name" in
    "💻SHELL")     pane="0" ;;
    "🧠STRATEGY")  pane="1" ;;
    "📈DETECT-MS") pane="2" ;;
    "🧮CALC-IND")  pane="3" ;;
    "📡STREAM-OHLCV") pane="4" ;;
    "🛡️PAPER")     pane="5" ;;
    "⚠️RISK")      pane="6" ;;
    "🔔ALERT")     pane="7" ;;
    "Monitor")     pane="8" ;;
    "🖥️CONSOLE")   pane="9" ;;
    "💹 FLOW-TRADE")   pane="10" ;;
    "📚 FLOW-DEPTH")   pane="11" ;;
    "💥 FLOW-LIQ")     pane="12" ;;
    "📈 FLOW-OI")      pane="13" ;;
    "💰 FLOW-FUNDING") pane="14" ;;
    "🎯 FLOW-MARK")    pane="15" ;;
    "🕐 FLOW-LAST")    pane="16" ;;
    "📉 FLOW-INDEX")   pane="17" ;;
    "🛢️DB-QUERY")     pane="18" ;;
    "🤖TELEGRAM")     pane="19" ;;
    "⏱TRADE-OHLCV")  pane="20" ;;
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

# ── VERİ AKIŞLARI (8 bağımsız süreç, her biri ayrı tmux sekmesi) ─
# WS → parse → validate → ring buffer → TimescaleDB
FLOW_BINS="flow-trade flow-depth flow-liquidation flow-oi flow-funding flow-markprice flow-lastprice flow-indexprice"
FLOW_TABS="💹 FLOW-TRADE 📚 FLOW-DEPTH 💥 FLOW-LIQ 📈 FLOW-OI 💰 FLOW-FUNDING 🎯 FLOW-MARK 🕐 FLOW-LAST 📉 FLOW-INDEX"

flows-start() {
  _start_guard
  cd "$CYCLE_ROOT" && cargo build -p flows 2>&1 | tail -1
  rm -f /dev/shm/cycle_finance_trades /dev/shm/cycle_finance_depth /dev/shm/cycle_finance_liquidations /dev/shm/cycle_finance_open_interest /dev/shm/cycle_finance_funding /dev/shm/cycle_finance_markprice /dev/shm/cycle_finance_lastprice /dev/shm/cycle_finance_indexprice /dev/shm/cycle_finance_api_gate
  local i=0 bin tab
  for tab in $FLOW_TABS; do
    bin=$(echo "$FLOW_BINS" | tr ' ' '\n' | sed -n "$((i+1))p")
    _tmux_pane "$tab" "cd $CYCLE_ROOT && ./target/debug/$bin" Enter
    i=$((i+1))
  done
  echo "✅ Veri akışları başlatıldı (8 akış — her biri ayrı sekme)"
}
flows-stop() {
  _start_guard
  for bin in $FLOW_BINS; do
    pkill -TERM -x "$bin" 2>/dev/null || pkill -TERM -f "$bin" 2>/dev/null || true
  done
  sleep 1
  echo "✅ Veri akışları durduruldu"
}
flows-status() {
  for bin in $FLOW_BINS; do
    local pid; pid=$(pgrep -x "$bin" 2>/dev/null | head -1 || true)
    [ -z "$pid" ] && pid=$(pgrep -f "$bin" 2>/dev/null | head -1 || true)
    if [ -n "$pid" ]; then
      local mem cpu
      mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
      cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
      echo "  ✔ $bin  [pid:$pid]  CPU:${cpu}%  RAM:${mem}"
    else
      echo "  ✘ $bin  (durdurulmuş)"
    fi
  done
}
flows-log() {
  echo "Akışlar tmux sekmelerinde çalışır (Ctrl+B → 10-17)."
  echo "  start_paper.sh ile başlatıldıysa: tail -f /tmp/flow-<isim>.log"
}

# ── DB-QUERY (TimescaleDB sorgu paneli) ─────────────────────
db-query-start() {
  _start_guard
  _tmux_pane "🛢️DB-QUERY" "cd $CYCLE_ROOT && ./target/release/db-query" Enter
}
# Örnek: db-query-recent trades BTCUSDT 10
db-query-recent() {
  cd "$CYCLE_ROOT" && ./target/release/db-query --recent "${1:-trades}" "${2:-BTCUSDT}" "${3:-10}"
}

# ── STRATEGY — ana strateji binary'si (strategies-engine) ───
strategy-start() {
  _start_guard
  if pgrep -f "strategies-engine" &>/dev/null; then echo "⚠️  STRATEGY zaten çalışıyor"; return 1; fi
  cd "$CYCLE_ROOT" && cargo build -p strategies-engine 2>&1 | tail -1
  _tmux_pane "🧠STRATEGY" "cd $CYCLE_ROOT && ./target/release/strategies-engine" Enter
  echo "✅ STRATEJİ MOTORU başlatıldı (pencere 1 — 🧠 STRATEGY)"
}
strategy-stop() {
  _start_guard
  local p; p=$(pgrep -f "strategies-engine" 2>/dev/null | head -1)
  if [ -n "$p" ]; then
    kill -TERM "$p" 2>/dev/null; sleep 1
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

# ============================================================
#  VERİ AKIŞLARI — 8 bağımsız süreç (WS → parse → ring → TimescaleDB)
#  Temel yardımcılar: flows-start / flows-stop / flows-status
# ============================================================
data-live() {
  flows-start
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
  echo "  alert-add VELVETUSDT above 0.22 [voice metni] [cooldown]"
  echo "  alert-update VELVETUSDT above 0.21628 0.22 [voice] [cooldown]"
  echo "  alert-remove VELVETUSDT above 0.21628"
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
  local url="${TIMESCALEDB_URL:-postgres://cycle:cycle@localhost:5432/market_data}"
  psql "$url" -c \
    "SELECT symbol, price, quantity, timestamp FROM trades ORDER BY timestamp DESC LIMIT 20;" \
    2>/dev/null || echo "DB boş veya bulunamadı."
}
db-size() {
  local url="${TIMESCALEDB_URL:-postgres://cycle:cycle@localhost:5432/market_data}"
  psql "$url" -c \
    "SELECT pg_size_pretty(pg_database_size(current_database())) AS db_size;" \
    2>/dev/null || echo "DB bağlanılamadı."
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
#  TRADE-OHLCV  (trade-ohlcv — trade data → 1s OHLCV :3009)
#  Kaynak: /dev/shm/cycle_finance_trades (flow ring)
#  Yayın:  /dev/shm/cycle_finance_trade_ohlcv (binary mumlar)
#  Canlı:  daemon her kapanan 1s mumu stdout'a stream eder (tmux pencere 20)
# ============================================================
TRADE_OHLCV_ADDR="${TRADE_OHLCV_ADDR:-127.0.0.1:3009}"

trade-ohlcv-start() {
  _start_guard
  if pgrep -x trade-ohlcv &>/dev/null; then
    echo "⚠️  trade-ohlcv zaten çalışıyor (pid: $(pgrep -x trade-ohlcv | head -1))"
    return 1
  fi
  if [ ! -f "$CYCLE_ROOT/target/debug/trade-ohlcv" ]; then
    echo "🔨 trade-ohlcv derleniyor..."
    cd "$CYCLE_ROOT" && cargo build -p trade-ohlcv 2>&1 | tail -5
  fi
  echo "🚀 trade-ohlcv başlatılıyor → http://$TRADE_OHLCV_ADDR"
  _tmux_pane "⏱TRADE-OHLCV" "cd $CYCLE_ROOT && ./target/debug/trade-ohlcv" Enter
  sleep 1
  if pgrep -x trade-ohlcv &>/dev/null; then
    echo "✅ trade-ohlcv başladı [pid: $(pgrep -x trade-ohlcv | head -1)]"
    echo "   Canlı 1s OHLCV akışı pencere 20'de. API: /api/candles/{symbol}"
  else
    echo "❌ trade-ohlcv başlatılamadı."
  fi
}

trade-ohlcv-stop() {
  _start_guard
  if pgrep -x trade-ohlcv &>/dev/null; then
    pkill -TERM -x trade-ohlcv && echo "✅ trade-ohlcv durduruldu"
  else
    echo "⚠️  trade-ohlcv zaten çalışmıyor"
  fi
}

trade-ohlcv-status() {
  local pid
  pid=$(pgrep -x trade-ohlcv 2>/dev/null | head -1 || true)
  if [ -n "$pid" ]; then
    local cpu mem
    cpu=$(ps -p "$pid" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$pid" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ trade-ohlcv ÇALIŞIYOR  [pid:$pid  CPU:${cpu}%  RAM:${mem}]"
    echo "   API: http://$TRADE_OHLCV_ADDR/api/health"
  else
    echo "✘  trade-ohlcv durdurulmuş"
  fi
}

trade-ohlcv-live() {
  if tmux has-session -t cycle 2>/dev/null; then
    tmux select-window -t cycle:20
  else
    echo "ℹ️  cycle session'ı yok. Canlı akış: cd $CYCLE_ROOT && ./target/debug/trade-ohlcv"
  fi
}

# Kullanım: trade-ohlcv-symbols
trade-ohlcv-symbols() {
  curl -s "http://$TRADE_OHLCV_ADDR/api/symbols" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. trade-ohlcv-start ile başlat."
}

# Kullanım: trade-ohlcv-candles [SYMBOL] [LIMIT]
trade-ohlcv-candles() {
  local sym="${1:-BTCUSDT}" lim="${2:-20}"
  echo "⏱ Sorgu: $sym son $lim kapalı 1s mum → http://$TRADE_OHLCV_ADDR"
  curl -s "http://$TRADE_OHLCV_ADDR/api/candles/${sym}?limit=${lim}" \
    | python3 -m json.tool 2>/dev/null || echo "❌ Servis yanıt vermiyor. trade-ohlcv-start ile başlat."
}

# ============================================================
#  STRATEJİ (strategies-engine — ana binary, pencere 1)
#  Kırılım stratejisi artık strategies-engine'in native kodudur.
# ============================================================
breakout-start() {
  _start_guard
  strategy-start
}

breakout-stop() {
  _start_guard
  strategy-stop
}

breakout-status() {
  _start_guard
  local p; p=$(pgrep -f "strategies-engine" 2>/dev/null | head -1 || true)
  if [ -n "$p" ]; then
    local cpu mem
    cpu=$(ps -p "$p" -o pcpu= 2>/dev/null | tr -d ' ')
    mem=$(ps -p "$p" -o rss= 2>/dev/null | awk '{printf "%.0fM", $1/1024}')
    echo "✅ STRATEJİ MOTORU ÇALIŞIYOR  [pid:$p  CPU:${cpu}%  RAM:${mem}]"
    echo "   Son sinyal: $(journalctl --user -u cycle-strategy.service -n 1 --no-pager 2>/dev/null | tail -1 | cut -c80-140)"
  else
    echo "✘  STRATEJİ MOTORU durdurulmuş (strategy-start)"
  fi
}

breakout-log() {
  echo "📌 Strateji pencere 1'de (🧠 STRATEGY) canlı çalışır."
  echo "   systemd ile çalışıyorsa: journalctl --user -u cycle-strategy.service -f"
  if tmux has-session -t cycle 2>/dev/null; then tmux select-window -t cycle:1; fi
}

breakout-query() {
  cd "$CYCLE_ROOT" && ./target/release/strategies-engine --once
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
