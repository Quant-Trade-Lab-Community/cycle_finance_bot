#!/usr/bin/env bash
# ============================================================
#  Cycle Finance — Servis İzleme Paneli
#  Her saniye güncellenir. Ctrl+C ile çıkılır.
#
#  İzlenen servisler:
#    core (DATA / STRATEGY / BACKTEST / CORRELATION)
#    paper-service
#    alert-service
# ============================================================

# ── Renkler ──────────────────────────────────────────────────
R='\033[0;31m'    # kırmızı
G='\033[0;32m'    # yeşil
Y='\033[1;33m'    # sarı
C='\033[0;36m'    # camgöbeği
B='\033[1;34m'    # mavi
M='\033[0;35m'    # mor
W='\033[1;37m'    # beyaz kalın
DIM='\033[2m'     # soluk
N='\033[0m'       # reset
BG='\033[40m'     # siyah arka plan

# ── GPU sysfs yolu (AMD RX 5500) ─────────────────────────────
GPU_CARD=""
for card in /sys/class/drm/card*/device/gpu_busy_percent; do
    if [ -r "$card" ]; then
        GPU_CARD="$(dirname "$card")"
        break
    fi
done

# ── Bar çizici ───────────────────────────────────────────────
# Kullanım: draw_bar <yüzde(0-100)> <genişlik> <renk>
draw_bar() {
    local pct="${1:-0}"
    local width="${2:-20}"
    local color="${3:-$G}"
    # Yüzde'yi tam sayıya dönüştür
    pct=$(echo "$pct" | awk '{printf "%d", $1}')
    [ "$pct" -gt 100 ] 2>/dev/null && pct=100
    [ "$pct" -lt 0 ]   2>/dev/null && pct=0
    local filled=$(( pct * width / 100 ))
    local empty=$(( width - filled ))
    # Yüksek kullanımda renk değiştir
    if [ "$pct" -ge 80 ]; then color="$R"
    elif [ "$pct" -ge 50 ]; then color="$Y"
    fi
    local bar=""
    for ((i=0; i<filled; i++)); do bar+="█"; done
    for ((i=0; i<empty;  i++)); do bar+="░"; done
    echo -ne "${color}${bar}${N}"
}

# ── RAM okunabilir format ─────────────────────────────────────
human_mb() {
    local kb="${1:-0}"
    local mb=$(( kb / 1024 ))
    if [ "$mb" -ge 1024 ]; then
        echo "$(awk "BEGIN{printf \"%.1f\", $mb/1024}")G"
    else
        echo "${mb}M"
    fi
}

# ── Servis satırı çiz ─────────────────────────────────────────
# Kullanım: print_service <etiket> <pid> <renk>
print_service() {
    local label="$1"
    local pid="$2"
    local color="${3:-$C}"
    local bar_width=18

    if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
        printf "${color}%-22s${N} ${DIM}%-8s${N} ${R}%-6s${N}   %-${bar_width}s   ${R}%-8s${N}   %-${bar_width}s\n" \
            "$label" "-" "KAPALI" "$(draw_bar 0 $bar_width $R)" "-" "$(draw_bar 0 $bar_width $R)"
        return
    fi

    # CPU ve bellek bilgisi
    local stat
    stat=$(ps -p "$pid" -o pid,pcpu,rss,vsz --no-headers 2>/dev/null | head -1)
    [ -z "$stat" ] && return

    local cpu  rss vsz
    cpu=$(echo "$stat" | awk '{printf "%.1f", $2}')
    rss=$(echo "$stat" | awk '{print $3}')   # KB
    vsz=$(echo "$stat" | awk '{print $4}')   # KB (sanal)

    local rss_str vsz_str cpu_int
    rss_str=$(human_mb "$rss")
    vsz_str=$(human_mb "$vsz")
    cpu_int=$(echo "$cpu" | awk '{printf "%d", $1}')

    # Çok çekirdekli sistemlerde CPU > 100 olabilir, bar için sıkıştır
    local cpu_bar_pct=$(( cpu_int > 100 ? 100 : cpu_int ))

    printf "${color}%-22s${N} ${W}%-8s${N} ${Y}%5s%%${N}  %s  ${C}%-8s${N}  %s\n" \
        "$label" "[$pid]" "$cpu" \
        "$(draw_bar "$cpu_bar_pct" "$bar_width")" \
        "$rss_str" \
        "$(draw_bar "$(( rss / 1024 > 100 ? 100 : rss / 1024 ))" "$bar_width")"
}

# ── Sistem toplamı ───────────────────────────────────────────
system_summary() {
    # CPU toplam kullanımı
    local cpu_idle cpu_use
    cpu_idle=$(top -bn1 | grep "Cpu(s)" | awk '{print $8}' | tr -d '%,')
    [ -z "$cpu_idle" ] && cpu_idle=$(vmstat 1 1 | tail -1 | awk '{print $15}')
    cpu_use=$(awk "BEGIN{printf \"%.1f\", 100 - ${cpu_idle:-0}}")

    # RAM
    local mem_total mem_avail mem_used mem_pct
    mem_total=$(awk '/MemTotal/{print $2}' /proc/meminfo)
    mem_avail=$(awk '/MemAvailable/{print $2}' /proc/meminfo)
    mem_used=$(( mem_total - mem_avail ))
    mem_pct=$(awk "BEGIN{printf \"%d\", $mem_used * 100 / $mem_total}")

    # GPU (AMD sysfs)
    local gpu_use="N/A" gpu_vram_pct=0 gpu_vram_str="N/A"
    if [ -n "$GPU_CARD" ]; then
        gpu_use=$(cat "${GPU_CARD}/gpu_busy_percent" 2>/dev/null || echo "0")
        local vram_used vram_total
        vram_used=$(cat "${GPU_CARD}/mem_info_vram_used"  2>/dev/null || echo "0")
        vram_total=$(cat "${GPU_CARD}/mem_info_vram_total" 2>/dev/null || echo "1")
        gpu_vram_pct=$(awk "BEGIN{printf \"%d\", $vram_used * 100 / $vram_total}")
        local vram_used_mb=$(( vram_used / 1024 / 1024 ))
        local vram_total_mb=$(( vram_total / 1024 / 1024 ))
        gpu_vram_str="${vram_used_mb}M / ${vram_total_mb}M"
    fi

    local cpu_int=${cpu_use%.*}
    printf "${DIM}Sistem Geneli:${N}\n"
    printf "  ${W}CPU  ${N}%5s%%  %s\n" "$cpu_use"  "$(draw_bar "$cpu_int" 30)"
    printf "  ${W}RAM  ${N}%5s%%  %s  ${DIM}(%s / %s)${N}\n" \
        "$mem_pct" "$(draw_bar "$mem_pct" 30)" \
        "$(human_mb "$mem_used")" "$(human_mb "$mem_total")"
    if [ -n "$GPU_CARD" ]; then
        printf "  ${W}GPU  ${N}%5s%%  %s\n" "$gpu_use" "$(draw_bar "$gpu_use" 30)"
        printf "  ${W}VRAM ${N}%5s%%  %s  ${DIM}(%s)${N}\n" \
            "$gpu_vram_pct" "$(draw_bar "$gpu_vram_pct" 30)" "$gpu_vram_str"
    else
        printf "  ${W}GPU  ${N}${DIM}  AMD sysfs okunamadı${N}\n"
    fi
}

# ── PID bul ─────────────────────────────────────────────────
find_pid() {
    local name="$1"
    pgrep -x "$name" 2>/dev/null | head -1
}

find_pid_env() {
    # RUN_MODE=X olan core process'ini bul
    local mode="$1"
    pgrep -x "core" 2>/dev/null | while read -r pid; do
        if grep -qa "RUN_MODE=$mode" /proc/"$pid"/environ 2>/dev/null; then
            echo "$pid"
            return
        fi
    done
}

# ── Ana döngü ────────────────────────────────────────────────
INTERVAL="${MONITOR_INTERVAL:-1}"

while true; do
    clear

    local_time=$(date '+%H:%M:%S')
    local_date=$(date '+%d.%m.%Y')

    echo -e "${W}╔══════════════════════════════════════════════════════════════════════════════════╗${N}"
    printf "${W}║${N}  ${M}📊 CYCLE FINANCE — SERVİS İZLEME PANELİ${N}%$((39 - ${#local_time}))s${Y}%s${N}  ${W}║${N}\n" "" "$local_time  $local_date"
    echo -e "${W}╚══════════════════════════════════════════════════════════════════════════════════╝${N}"
    echo ""

    # ── Sistem özeti ─────────────────────────────────────────
    system_summary
    echo ""

    # ── Servis başlıkları ─────────────────────────────────────
    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"
    printf "${DIM}%-22s  %-8s  %-7s  %-18s  %-8s  %-18s${N}\n" \
        "SERVİS" "PID" "CPU%" "CPU KULLANIMI" "RAM" "RAM KULLANIMI"
    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"

    # ── Servisler ────────────────────────────────────────────
    PID_DATA=$(find_pid_env "DATA")
    PID_STRATEGY=$(find_pid_env "STRATEGY")
    PID_BACKTEST=$(find_pid_env "BACKTEST")
    PID_CORRELATION=$(find_pid_env "CORRELATION")
    PID_PAPER=$(find_pid "paper-service")
    PID_ALERT=$(find_pid "alert-service")

    # core binary tek isimle görünüyorsa genel bul
    [ -z "$PID_DATA" ] && [ -z "$PID_STRATEGY" ] && [ -z "$PID_BACKTEST" ] && [ -z "$PID_CORRELATION" ] && {
        ALL_CORE=$(pgrep -x "core" 2>/dev/null | head -1)
    }

    print_service "📡 DATA"          "${PID_DATA:-$ALL_CORE}" "$C"
    print_service "🧠 STRATEGY"      "$PID_STRATEGY"          "$B"
    print_service "🔄 BACKTEST"      "$PID_BACKTEST"          "$M"
    print_service "📈 CORRELATION"   "$PID_CORRELATION"       "$Y"
    echo -e "${DIM}──────────────────────────────────────────────────────────────────────────────────${N}"
    print_service "🛡️  PAPER-SERVICE" "$PID_PAPER"             "$G"
    print_service "🔔 ALERT-SERVICE" "$PID_ALERT"             "$Y"

    echo -e "${W}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${N}"

    # ── Çalışan servis sayısı ─────────────────────────────────
    running=0
    for p in "$PID_DATA" "$PID_STRATEGY" "$PID_BACKTEST" "$PID_CORRELATION" "$PID_PAPER" "$PID_ALERT"; do
        [ -n "$p" ] && kill -0 "$p" 2>/dev/null && (( running++ )) || true
    done

    echo ""
    printf "  ${DIM}Çalışan servis: ${W}%d/6${N}${DIM}   |   Yenileme: her %ss   |   Çıkış: Ctrl+C${N}\n" \
        "$running" "$INTERVAL"

    # ── Ring buffer bilgisi ───────────────────────────────────
    echo ""
    echo -e "  ${DIM}Ring Buffer Durumu:${N}"
    for ring in demir_yumruk_ring demir_yumruk_orders; do
        if [ -f "/dev/shm/$ring" ]; then
            ring_size=$(du -sh "/dev/shm/$ring" 2>/dev/null | cut -f1)
            printf "    ${G}✔${N} /dev/shm/%-28s %s\n" "$ring" "$ring_size"
        else
            printf "    ${R}✘${N} /dev/shm/%-28s ${DIM}(yok)${N}\n" "$ring"
        fi
    done

    sleep "$INTERVAL"
done
