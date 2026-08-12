#!/usr/bin/env bash
# ============================================================
#  cycle_ip.sh — genel (public) IP algılama + değişim takibi
#
#  Binance API anahtarlarına IP kısıtı uygulandığında, IP
#  değişince anahtar kilitlenir. Bu script güncel public IP'yi
#  gösterir, değişimi yakalar ve uyarır.
#
#  Kullanım:
#    cycle_ip.sh now       → güncel public IP'yi gösterir
#    cycle_ip.sh record    → güncel IP'yi state dosyasına kaydeder
#    cycle_ip.sh check     → kayıtlı IP ile karşılaştırır (değişirse uyarır + günceller)
#    cycle_ip.sh state     → kayıtlı IP'yi gösterir
#    cycle_ip.sh watch [SN] → her SN saniyede bir kontrol eder (varsayılan 300)
#
#  State dosyası: $CYCLE_IP_STATE (varsayılan /tmp/cycle_public_ip)
# ============================================================
set -euo pipefail

STATE="${CYCLE_IP_STATE:-/tmp/cycle_public_ip}"

# ── Public IP algılama ───────────────────────────────────────
# Birden çok sağlayıcı — biri bloklarsa diğerine düş.
public_ip() {
  local url ip
  for url in \
    "https://api.ipify.org" \
    "https://ifconfig.me/ip" \
    "https://icanhazip.com" \
    "https://ipv4.icanhazip.com"; do
    ip=$(curl -s --max-time 8 "$url" 2>/dev/null | tr -d '[:space:]' | grep -E '^[0-9]{1,3}(\.[0-9]{1,3}){3}$' || true)
    if [ -n "$ip" ]; then
      echo "$ip"
      return 0
    fi
  done
  echo ""
  return 1
}

# ── Komutlar ─────────────────────────────────────────────────
case "${1:-now}" in
  now)
    if ip=$(public_ip); then
      echo "🌐 Güncel public IP: $ip"
    else
      echo "❌ Public IP alınamadı (sağlayıcılara erişilemiyor)." >&2
      exit 1
    fi
    ;;
  record)
    if ip=$(public_ip); then
      echo "$ip" > "$STATE"
      echo "💾 IP kaydedildi: $ip → $STATE"
    else
      echo "❌ Public IP alınamadı." >&2
      exit 1
    fi
    ;;
  state)
    if [ -f "$STATE" ]; then
      echo "📌 Kayıtlı IP: $(cat "$STATE")"
    else
      echo "ℹ️  Kayıtlı IP yok. cycle_ip.sh record ile kaydedin."
    fi
    ;;
  check)
    if ! now_ip=$(public_ip); then
      echo "❌ Public IP alınamadı." >&2
      exit 1
    fi
    if [ -f "$STATE" ]; then
      old_ip=$(cat "$STATE")
      if [ "$old_ip" = "$now_ip" ]; then
        echo "✅ IP değişmedi: $now_ip (kayıtlı ile aynı)"
      else
        echo "⚠️  IP DEĞİŞTİ: $old_ip → $now_ip"
        echo "   Binance API Management → anahtarın IP whitelist'ine yeni IP'yi ekle veya kısıtı kapat!"
        echo "$now_ip" > "$STATE"
        exit 2
      fi
    else
      echo "$now_ip" > "$STATE"
      echo "💾 İlk kontrol — IP kaydedildi: $now_ip"
    fi
    ;;
  watch)
    interval="${2:-300}"
    echo "👀 IP takibi başladı (her ${interval}s). Ctrl+C ile durdur."
    while true; do
      bash "$0" check
      sleep "$interval"
    done
    ;;
  *)
    echo "Kullanım: cycle_ip.sh {now|record|check|state|watch [SN]}" >&2
    exit 1
    ;;
esac
