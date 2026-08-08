#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# OS panosunu tmux paste buffer'a yükleyip yapıştırır.
# Wayland (wl-paste) → X11 (xclip/xsel) sırasıyla dener.
#
# tmux'ta Ctrl+V (veya Ctrl+Shift+V) bu betiği çalıştırır:
#   bind -n C-v   run-shell "~/.cycle_tmux_paste.sh"
# ─────────────────────────────────────────────────────────────────────────────
set -u

tmp="$(mktemp /tmp/tmux_paste.XXXXXX)"

if command -v wl-paste >/dev/null 2>&1 && [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
    wl-paste -p 2>/dev/null > "$tmp"
elif command -v xclip >/dev/null 2>&1; then
    xclip -o -selection clipboard 2>/dev/null > "$tmp"
elif command -v xsel >/dev/null 2>&1; then
    xsel -b -o 2>/dev/null > "$tmp"
fi

if [[ -s "$tmp" ]]; then
    # Panodaki sondaki satır sonlarını (CR/LF) temizle: paste sırasında fazladan
    # Enter/\r yutulup bir sonraki "read"i boş tetiklemesin.
    perl -0777 -pi -e 's/[\r\n]+\z//' "$tmp" 2>/dev/null || sed -i 's/\r$//' "$tmp"
    tmux load-buffer "$tmp" 2>/dev/null
    tmux paste-buffer 2>/dev/null
fi

rm -f "$tmp"
