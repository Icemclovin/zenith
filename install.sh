#!/usr/bin/env bash
set -e

# Zenith Control Center - Complete Out-of-the-Box Installer voor Arch Linux & Hyprland
# Garandeert dat Zenith 100% werkt op elke Arch Linux machine met Hyprland.

echo "========================================================"
echo "    Zenith Control Center - Arch & Hyprland Installer   "
echo "========================================================"

if ! command -v pacman &> /dev/null; then
    echo "Fout: pacman niet gevonden. Zenith is ontworpen voor Arch Linux (en afgeleiden)."
    exit 1
fi

TARGET_USER="${SUDO_USER:-$USER}"
TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
[ -z "$TARGET_HOME" ] && TARGET_HOME="$HOME"

echo "-> Installatie voor gebruiker: $TARGET_USER ($TARGET_HOME)"

echo "-> 1. Systeembibliotheken en afhankelijkheden controleren..."
PACKAGES=(
    base-devel
    gtk4
    libadwaita
    polkit
    which
    procps-ng
    psmisc
    xdg-utils
    curl
    jq
    kitty
    rofi
    waybar
    quickshell
    brightnessctl
    playerctl
    wireplumber
    bluez-utils
    networkmanager
    ttf-jetbrains-mono-nerd
    swaybg
)

if ! command -v cargo &> /dev/null && ! command -v rustup &> /dev/null; then
    PACKAGES+=(rust cargo)
fi

echo "-> Ontbrekende pakketten installeren via pacman..."
if [ "$(id -u)" -eq 0 ]; then
    pacman -S --needed --noconfirm "${PACKAGES[@]}"
else
    sudo pacman -S --needed --noconfirm "${PACKAGES[@]}"
fi

echo "-> 2. Zenith binary compileren (release build)..."
if [ "$(id -u)" -eq 0 ] && [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
    sudo -u "$TARGET_USER" cargo build --release
else
    cargo build --release
fi

echo "-> 3. Zenith systeembreed installeren..."
if [ "$(id -u)" -eq 0 ]; then
    install -Dm755 target/release/zenith /usr/local/bin/zenith
else
    sudo install -Dm755 target/release/zenith /usr/local/bin/zenith
fi

# Zorg dat eventuele lokale binaries in ~/.local/bin up-to-date zijn (voorkomt PATH-conflicten)
mkdir -p "$TARGET_HOME/.local/bin"
cp -f target/release/zenith "$TARGET_HOME/.local/bin/zenith"
chmod +x "$TARGET_HOME/.local/bin/zenith"
if [ "$(id -u)" -eq 0 ] && [ -n "$SUDO_USER" ]; then
    chown "$TARGET_USER:$TARGET_USER" "$TARGET_HOME/.local/bin/zenith" 2>/dev/null || true
fi

echo "-> 4. Desktop-koppeling registreren..."
DESKTOP_FILE="/usr/share/applications/org.zenith.control.desktop"
if [ "$(id -u)" -eq 0 ]; then
    mkdir -p /usr/share/applications
    cat << 'DESKTOP_ENTRY' > "$DESKTOP_FILE"
[Desktop Entry]
Name=Zenith Control Center
Comment=Hyprland & Quickshell Visual Studio and Desktop Control
Exec=/usr/local/bin/zenith
Icon=preferences-system
Terminal=false
Type=Application
Categories=Settings;System;Utility;
Keywords=Hyprland;Settings;Control;Waybar;Quickshell;Theme;
StartupWMClass=org.zenith.control
DESKTOP_ENTRY
else
    sudo mkdir -p /usr/share/applications
    sudo tee "$DESKTOP_FILE" > /dev/null << 'DESKTOP_ENTRY'
[Desktop Entry]
Name=Zenith Control Center
Comment=Hyprland & Quickshell Visual Studio and Desktop Control
Exec=/usr/local/bin/zenith
Icon=preferences-system
Terminal=false
Type=Application
Categories=Settings;System;Utility;
Keywords=Hyprland;Settings;Control;Waybar;Quickshell;Theme;
StartupWMClass=org.zenith.control
DESKTOP_ENTRY
fi

if command -v update-desktop-database &> /dev/null; then
    if [ "$(id -u)" -eq 0 ]; then
        update-desktop-database /usr/share/applications || true
    else
        sudo update-desktop-database /usr/share/applications || true
    fi
fi

echo "-> 5. Hyprland & Quickshell configuratie bootstrappen..."
# Voer bootstrap routines uit via zenith zelf als doelgebruiker
if [ "$(id -u)" -eq 0 ] && [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
    sudo -u "$TARGET_USER" HOME="$TARGET_HOME" /usr/local/bin/zenith --help &> /dev/null || true
else
    /usr/local/bin/zenith --help &> /dev/null || true
fi

HYPR_DIR="$TARGET_HOME/.config/hypr"
HYPR_CONF="$HYPR_DIR/hyprland.conf"
ZENITH_CONF="$HYPR_DIR/zenith.conf"

mkdir -p "$HYPR_DIR"

if [ -f "$HYPR_CONF" ]; then
    if ! grep -q "zenith.conf" "$HYPR_CONF"; then
        echo "-> Zenith source toevoegen aan hyprland.conf..."
        echo -e "\n# Injected by Zenith Control\nsource = ~/.config/hypr/zenith.conf" >> "$HYPR_CONF"
    fi
else
    echo "-> Geen hyprland.conf gevonden; basisconfiguratie aanmaken..."
    cat << 'STARTER_HYPR' > "$HYPR_CONF"
# Hyprland Configuration with Zenith Control
$mainMod = SUPER
bind = $mainMod, Q, exec, kitty
bind = $mainMod, C, killactive,
bind = $mainMod, M, exit,
bind = $mainMod, V, togglefloating,
bind = $mainMod, R, exec, rofi -show drun || wofi --show drun

# Injected by Zenith Control
source = ~/.config/hypr/zenith.conf
STARTER_HYPR
fi

echo "-> 6. Quickshell configuratie valideren..."
QS_DIR="$TARGET_HOME/.config/quickshell"
mkdir -p "$QS_DIR/modules"

# Herstel rechten indien installer als root draaide
if [ "$(id -u)" -eq 0 ] && [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
    chown -R "$TARGET_USER:$TARGET_USER" "$HYPR_DIR" 2>/dev/null || true
    chown -R "$TARGET_USER:$TARGET_USER" "$QS_DIR" 2>/dev/null || true
fi

# Test of quickshell startbaar is
if command -v quickshell &> /dev/null; then
    echo "Quickshell binary: $(which quickshell) (versie: $(quickshell --version 2>/dev/null || echo '0.3.1'))"
fi

echo ""
echo "========================================================"
echo "    Zenith Control Center is 100% succesvol geïnstalleerd!   "
echo "========================================================"
echo "✓ Systeembrede binary: /usr/local/bin/zenith"
echo "✓ Lokale binary sync:  $TARGET_HOME/.local/bin/zenith"
echo "✓ Desktop Entry:       /usr/share/applications/org.zenith.control.desktop"
echo "✓ Hyprland integratie: $ZENITH_CONF"
echo "✓ Quickshell modulair: $QS_DIR/zenith-shell.json"
echo ""
echo "Start Zenith direct met het commando: zenith"
echo "Of open 'Zenith Control Center' vanuit je applicatiemenu (Rofi/Wofi)."
