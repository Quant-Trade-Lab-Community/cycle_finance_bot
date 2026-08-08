#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Binance Futures API anahtarlarını güvenli şekilde gir (shell üzerinden).
#
# - Anahtar girilirken ekrana YAZILMAZ (read -s).
# - .env dosyası 600 (yalnızca sahip) iznine alınır.
# - Varsayılan EXEC_DRY_RUN=true (güvenlik): gerçek emir için ayrıca
#   `EXEC_DRY_RUN=false` onayı gerekir.
#
# Kullanım:
#   ./additional-services/scripts/exec_setup.sh          # anahtar gir
#   ./additional-services/scripts/exec_setup.sh --show   # hangi değişkenler set?
#   ./additional-services/scripts/exec_setup.sh --testnet # testnet URL'leri yaz
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

ENV_FILE="${EXEC_ENV_FILE:-$(cd "$(dirname "$0")/../.." && pwd)/.env}"
TESTNET_BASE="https://testnet.binancefuture.com"
TESTNET_WS="wss://stream.binancefuture.com"

# Önceki yapıştırmadan artık satır başı kalıntılarını terminal tamponundan atar.
# CRLF pano yapıştırınca ilk read '\r'de biter, kalan '\n' bir sonraki read'i
# boş yutardı — bu yüzden her okumadan önce tamponu temizle.
drain_input() {
    local ch
    while read -r -t 0 -s -n 1 ch 2>/dev/null; do :; done
}

trim() {
    local v="$1"
    v="${v#"${v%%[![:space:]]*}"}"   # öndeki boşlukları kırp
    v="${v%"${v##*[![:space:]]}"}"   # arkadaki boşlukları kırp
    printf '%s' "$v"
}

prompt_key() {
    local label="$1"
    local value
    # İpuçları stderr'e gider; sadece girilen değer stdout'a döner ($( ) yakalamasın).
    echo "  İpucu: yapıştırmak için Ctrl+Shift+V (Linux) veya Ctrl+V (tmux bağı)." >&2
    echo "  Girdiğiniz karakterler EKRANA YAZILMAZ (güvenlik) — yapıştırın, Enter." >&2
    while :; do
        drain_input
        read -rsp "$label: " value
        echo >&2
        # Bracketed-paste artıkları + satır sonlarını temizle (pano yapıştırınca)
        value="${value//$'\e[200~'/}"
        value="${value//$'\e[201~'/}"
        value="${value//$'\r'/}"
        value="${value//$'\n'/}"
        value="$(trim "$value")"
        if [[ -n "$value" ]]; then
            break
        fi
        echo "  Boş olamaz, tekrar dene." >&2
    done
    printf '%s' "$value"
}

write_env() {
    local key="$1" val="$2"
    local tmp="${ENV_FILE}.tmp"
    if grep -qE "^${key}=" "$ENV_FILE" 2>/dev/null; then
        # awk ile sadece ilgili satırı değiştir; dosyadaki \r kalıntılarını da temizle.
        awk -v k="$key" -v v="$val" '
            { gsub(/\r$/, ""); }
            $0 ~ "^"k"=" { print k "=" v; found=1; next }
            { print }
            END{ if (!found) print k "=" v }
        ' "$ENV_FILE" > "$tmp" && mv "$tmp" "$ENV_FILE"
    else
        # Satır sonu kalıntılarını temizle, sonra ekle.
        sed -i 's/\r$//' "$ENV_FILE" 2>/dev/null || true
        printf '%s=%s\n' "$key" "$val" >> "$ENV_FILE"
    fi
}

case "${1:-}" in
    --show)
        echo "ENV_FILE: $ENV_FILE"
        for v in BINANCE_API_KEY BINANCE_SECRET_KEY EXEC_MODE EXEC_DRY_RUN EXEC_BASE_URL EXEC_WS_URL; do
            if grep -qE "^${v}=" "$ENV_FILE" 2>/dev/null; then
                val=$(grep -E "^${v}=" "$ENV_FILE" | head -1 | cut -d= -f2-)
                if [[ "$v" == *KEY* ]]; then
                    masked="${val:0:6}****${val: -4}"
                    [[ -z "$val" ]] && masked="(boş)"
                    echo "  $v = $masked"
                else
                    echo "  $v = ${val:-}"
                fi
            else
                echo "  $v = (tanımsız)"
            fi
        done
        ;;
    --testnet)
        write_env EXEC_BASE_URL "$TESTNET_BASE"
        write_env EXEC_WS_URL "$TESTNET_WS"
        write_env EXEC_DRY_RUN "true"
        echo "Testnet yapılandırması yazıldı:"
        echo "  EXEC_BASE_URL=$TESTNET_BASE"
        echo "  EXEC_WS_URL=$TESTNET_WS"
        ;;
    *)
        if [[ ! -f "$ENV_FILE" ]]; then
            echo "  .env yok — oluşturuluyor: $ENV_FILE"
            : > "$ENV_FILE"
        fi

        echo "Binance Futures API anahtarları — ekrana yazılmaz."
        api_key=$(prompt_key "BINANCE_API_KEY")
        secret=$(prompt_key "BINANCE_SECRET_KEY")

        if [[ -z "$api_key" || -z "$secret" ]]; then
            echo "  ✘ Anahtar(lar) boş — hiçbir şey kaydedilmedi. Tekrar dene." >&2
            exit 1
        fi

        # Maskeli doğrulama — yanlış yapıştırma burada yakalanır.
        mask() { local v="$1"; [[ -z "$v" ]] && echo "(boş)" || echo "${v:0:4}****${v: -4}"; }
        echo
        echo "  Girilen anahtarlar:"
        echo "    BINANCE_API_KEY    = $(mask "$api_key")  (uzunluk: ${#api_key})"
        echo "    BINANCE_SECRET_KEY = $(mask "$secret")  (uzunluk: ${#secret})"
        drain_input
        read -rsp "  Kaydetmek için 'EVET' yazın, iptal için Enter: " onay
        echo
        if [[ "$onay" != "EVET" ]]; then
            echo "İptal edildi — hiçbir şey yazılmadı."
            exit 0
        fi

        write_env "BINANCE_API_KEY" "$api_key"
        write_env "BINANCE_SECRET_KEY" "$secret"
        write_env "EXEC_MODE" "LIVE"
        # Güvenlik: DRY_RUN varsayılan açık. Gerçek emir için kullanıcı ayrıca
        # `EXEC_DRY_RUN=false` ayarlamalıdır.
        write_env "EXEC_DRY_RUN" "true"

        chmod 600 "$ENV_FILE"
        echo
        echo "Anahtarlar kaydedildi: $ENV_FILE (izin 600)"
        echo "  EXEC_DRY_RUN=true  → emirler gönderilmez."
        echo "  Gerçek emir için:  EXEC_DRY_RUN=false ./target/debug/executiond"
        echo "  Testnet için:      $0 --testnet"
        ;;
esac
