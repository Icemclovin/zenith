#!/usr/bin/env bash
#
# install-binary.sh — Installeert de nieuwste Zenith release-binary naar:
#   1. /usr/local/bin/zenith   (systeembreed; vereist root/sudo)
#   2. ~/.local/bin/zenith     (per-gebruiker; vereist schrijfrecht op $HOME/.local)
#
# Gebruik:
#   ./install-binary.sh                 # beide locaties (vraagt sudo voor /usr/local/bin indien nodig)
#   ./install-binary.sh --local-only    # alleen ~/.local/bin
#   ./install-binary.sh --system-only   # alleen /usr/local/bin
#
# Het script hoeft alleen lokaal in de Zenith-werkmap te draaien (target/release/zenith).

set -euo pipefail

BIN_SRC="${BIN_SRC:-target/release/zenith}"

if [ ! -f "$BIN_SRC" ]; then
    echo "Fout: $BIN_SRC niet gevonden. Druk eerst 'cargo build --release'."
    exit 1
fi

echo "Bron-binary: $BIN_SRC"
echo "  $("$BIN_SRC" --help 2>/dev/null | head -1 || true)"
echo

LOCAL_ONLY=0
SYSTEM_ONLY=0
for arg in "$@"; do
    case "$arg" in
        --local-only) LOCAL_ONLY=1 ;;
        --system-only) SYSTEM_ONLY=1 ;;
        *) echo "Onbekend argument: $arg"; exit 2 ;;
    esac
done

install_user_bin() {
    local dir="$HOME/.local/bin"
    mkdir -p "$dir"
    install -m755 "$BIN_SRC" "$dir/zenith"
    echo "✓ Geïnstalleerd naar: $dir/zenith"
}

# ---- Systeem-brede installatie: /usr/local/bin/zenith ----
if [ "$SYSTEM_ONLY" -eq 1 ] || { [ "$LOCAL_ONLY" -eq 0 ] && [ "$SYSTEM_ONLY" -eq 0 ]; }; then
    if [ -w /usr/local/bin ]; then
        install -m755 "$BIN_SRC" /usr/local/bin/zenith
        echo "✓ Geïnstalleerd naar: /usr/local/bin/zenith"
    elif command -v sudo &>/dev/null; then
        echo "-> /usr/local/bin is niet beschrijfbaar; sudo gebruiken..."
        sudo install -m755 "$BIN_SRC" /usr/local/bin/zenith
        echo "✓ Geïnstalleerd naar: /usr/local/bin/zenith"
    else
        echo "! /usr/local/bin is niet beschrijfbaar en sudo is niet beschikbaar; deze stap overgeslagen."
        [ "$SYSTEM_ONLY" -eq 1 ] && echo "  OPMERKING: --system-only gevraagd maar niet mogelijk in deze omgeving." && exit 1
    fi
fi

# ---- Lokale installatie: ~/.local/bin/zenith ----
if [ "$LOCAL_ONLY" -eq 1 ] || [ "$SYSTEM_ONLY" -eq 0 ]; then
    if [ -w "$HOME" ]; then
        install_user_bin
    else
        echo "! $HOME is niet beschrijfbaar; lokale installatie overgeslagen."
        [ "$LOCAL_ONLY" -eq 1 ] && echo "  OPMERKING: --local-only gevraagd maar niet mogelijk in deze omgeving." && exit 1
    fi
fi

echo
echo "Klaar. Start Zenith met: zenith"
echo "(Let op: als ~/.local/bin in je PATH staat vóór /usr/local/bin, dan wint de lokale kopie.)"