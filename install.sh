#!/usr/bin/env bash
set -e

echo "=== Zenith Control Center - Systeembrede Installer ==="

if ! command -v pacman &> /dev/null; then
    echo "Fout: pacman niet gevonden. Dit script is bedoeld voor Arch Linux."
    exit 1
fi

echo "-> Basis-dependencies installeren..."
PACKAGES=(
    base-devel
    gtk4
    libadwaita
    polkit
    which
    procps-ng
    psmisc
    kitty
    rofi
    waybar
)

if ! command -v cargo &> /dev/null && ! command -v rustup &> /dev/null; then
    PACKAGES+=(rust cargo)
fi

sudo pacman -S --needed --noconfirm "${PACKAGES[@]}"

echo "-> Geoptimaliseerde release binary compileren..."
cargo build --release

echo "-> Binary systeembreed installeren naar /usr/local/bin/zenith..."
sudo install -Dm755 target/release/zenith /usr/local/bin/zenith

echo "-> Desktop-bestand registreren voor alle gebruikers (/usr/share/applications)..."
sudo mkdir -p /usr/share/applications
sudo tee /usr/share/applications/org.zenith.control.desktop > /dev/null <<EOF
[Desktop Entry]
Name=Zenith
Comment=Hyprland Desktop Control Center
Exec=/usr/local/bin/zenith
Icon=preferences-system
Terminal=false
Type=Application
Categories=Settings;System;Utility;
Keywords=Hyprland;Settings;Control;Waybar;Theme;
EOF

HYPR_CONF="$HOME/.config/hypr/hyprland.conf"
if [ -f "$HYPR_CONF" ]; then
    if ! grep -q "org.zenith.control" "$HYPR_CONF"; then
        echo "-> Hyprland vensterregel toevoegen voor de huidige gebruiker..."
        cat <<EOF >> "$HYPR_CONF"

# Zenith Control Center window rules
windowrulev2 = float, class:^(org.zenith.control)$
windowrulev2 = size 560 700, class:^(org.zenith.control)$
windowrulev2 = center, class:^(org.zenith.control)$
EOF
    fi
fi

echo ""
echo "=== Installatie voltooid! ==="
echo "Zenith is nu systeembreed geïnstalleerd voor ELKE gebruiker op deze machine."
echo "Iedereen kan direct 'zenith' starten of openen via de launcher."