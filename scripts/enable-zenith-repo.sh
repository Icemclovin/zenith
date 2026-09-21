#!/usr/bin/env bash
# enable-zenith-repo.sh — voegt de Zenith-repo toe aan pacman.conf en installeert
# zenith-control zonder git clone.
#
# Gebruik (in de VM, als root of met sudo):
#   sudo ./scripts/enable-zenith-repo.sh
#
# Optioneel: geef een gewenste versie-tag:
#   sudo ./scripts/enable-zenith-repo.sh zenith-control-0.1.0-1

set -euo pipefail

PACMAN_CONF="/etc/pacman.conf"
OWNER="Icemclovin"
REPO="zenith"
TAG="${1:-latest}"

if [ "$(id -u)" -ne 0 ]; then
  echo "! Draai dit als root: sudo $0"
  exit 1
fi

if grep -q '^\[zenith\]' "$PACMAN_CONF"; then
  echo "-> [zenith]-repo staat al in $PACMAN_CONF"
else
  cat >> "$PACMAN_CONF" <<EOF

# ZenithOS Control Center (toegevoegd door enable-zenith-repo.sh)
[zenith]
SigLevel = Optional TrustAll
Server = https://github.com/$OWNER/$REPO/releases/${TAG}/download
EOF
  echo "-> [zenith]-repo toegevoegd aan $PACMAN_CONF"
fi

echo "-> Synchroniseren..."
pacman -Sy

echo "-> Installeren..."
pacman -S --noconfirm zenith-control

echo ""
echo "✓ Klaar! Start met: zenith"
echo "  Daemon (optioneel): systemctl --user start zenithd"
echo ""
echo "  Updaten na wijzigingen: sudo pacman -S zenith-control"