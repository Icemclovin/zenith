#!/usr/bin/env bash
#
# zenithos-vm-install.sh — Installer voor ZenithOS in een VM via de terminal.
#
# Deze installer draait je OP een werkende, verse Arch Linux-installatie in de VM
# (gemaakt met het archinstall-profiel in deze map, of handmatig). Hij zet in één
# keer de volledige ZenithOS-desktop op:
#   • desktop (Hyprland + Quickshell/Waybar)
#   • hulpmiddelen (Kitty, Rofi, NetworkManager, hulpprogramma's)
#   • de Zenith Control Center-app (build + install)
#   • de zenithd JSON-RPC IPC-daemon als systeemdienst
#   • een startersessie voor Hyprland
#
# Er wordt GEEN ISO gebouwd — je gebruikt dit in een bestaande Arch-VM.
#
# Gebruik (in de VM, vanaf de terminal):
#   sudo ./scripts/zenithos-vm-install.sh --user <je_gebruikersnaam>
#
# Opties:
#   --user <naam>     Doelgebruiker waarvan de configuratie wordt opgezet (verplicht).
#   --repo <pad|URL>  Pad naar de Zenith-repo (lokaal) OF een git-URL om te clonen.
#                     Standaard: de huidige map (wanneer je daar draait).
#   --no-daemon       Installeer de app maar start geen zenithd-systeemdienst.
#   --debug           Extra uitvoer tonen.

set -euo pipefail

TARGET_USER=""
REPO="${ZENITH_REPO:-$PWD}"
NO_DAEMON=0
DEBUG=0

while [ $# -gt 0 ]; do
    case "$1" in
        --user) TARGET_USER="$2"; shift 2 ;;
        --repo) REPO="$2"; shift 2 ;;
        --no-daemon) NO_DAEMON=1; shift ;;
        --debug) DEBUG=1; shift ;;
        *) echo "Onbekend argument: $1"; exit 2 ;;
    esac
done

if [ -z "$TARGET_USER" ]; then
    echo "Fout: geef een doelgebruiker op met --user <naam>"
    echo "Voorbeeld: sudo ./scripts/zenithos-vm-install.sh --user student"
    exit 1
fi

if ! command -v pacman &>/dev/null; then
    echo "Fout: pacman niet gevonden. ZenithOS is ontworpen voor Arch Linux."
    exit 1
fi

log() { if [ "$DEBUG" -eq 1 ] && [ -n "$1" ]; then echo ":: $*"; fi; }
step() { echo ""; echo "▶ $*"; }

# ------------------------------------------------------------------
step "Systeembibliotheken en bouw-afhankelijkheden installeren"
PACKAGES=(
    base-devel
    git
    rust cargo
    gtk4 libadwaita
    polkit which procps-ng psmisc xdg-utils curl jq
    hyprland hyprpolkitagent xdg-desktop-portal-hyprland
    kitty rofi waybar quickshell
    brightnessctl playerctl wireplumber bluez-utils
    networkmanager ttf-jetbrains-mono-nerd swaybg
    inter papirus-icon-theme
    openssh
)
echo "-> Pakketten (${#PACKAGES[@]}): ${PACKAGES[*]}"
if [ "$(id -u)" -eq 0 ]; then
    pacman -S --needed --noconfirm "${PACKAGES[@]}"
else
    sudo pacman -S --needed --noconfirm "${PACKAGES[@]}"
fi

# Doelgebruiker verifiëren (of aanmaken via useradd is bewust GEEN onderdeel:
# maak de gebruiker bij de basis-installatie van Arch).
if ! id "$TARGET_USER" &>/dev/null; then
    echo "! Gebruiker '$TARGET_USER' bestaat niet. Maak die eerst aan, bijv.:"
    echo "  sudo useradd -m -G wheel -s /bin/zsh $TARGET_USER"
    exit 1
fi
TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
[ -z "$TARGET_HOME" ] && TARGET_HOME="/home/$TARGET_USER"
echo "-> Doelgebruiker: $TARGET_USER ($TARGET_HOME)"

# ------------------------------------------------------------------
step "Zenith bouwen en installeren"
log "Repo: $REPO"
BUILD_DIR=""
if [ -f "$REPO/Cargo.toml" ]; then
    BUILD_DIR="$REPO"
else
    echo "-> Zenith-repo clonen (niet lokaal aanwezig)..."
    GIT_URL="${REPO:-https://github.com/zenithos/core.git}"
    BUILD_DIR="$(mktemp -d)/zenith"
    git clone --depth 1 "$GIT_URL" "$BUILD_DIR"
fi

pushd "$BUILD_DIR" >/dev/null 2>&1 || BUILD_DIR="$PWD"
cargo build --release

# Systeembreed installeren
install -Dm755 target/release/zenith /usr/local/bin/zenith
install -Dm755 target/release/zenithd /usr/local/bin/zenithd

# Desktop-entry
mkdir -p /usr/share/applications
cat > /usr/share/applications/org.zenith.control.desktop <<'DESKTOP_ENTRY'
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
popd >/dev/null 2>&1 || true

# ------------------------------------------------------------------
step "Hyprland & Quickshell configuratie voor '$TARGET_USER' opzetten"
# Basis-Hyprland sessie die de balk en de daemon start.
HYPR_DIR="$TARGET_HOME/.config/hypr"
mkdir -p "$HYPR_DIR/.v2" 2>/dev/null || true
mkdir -p "$HYPR_DIR" "$TARGET_HOME/.config/quickshell/modules"

if [ ! -f "$HYPR_DIR/hyprland.conf" ]; then
    echo "-> Basis hyprland.conf aanmaken..."
    cat > "$HYPR_DIR/hyprland.conf" <<EOF
# ZenithOS basis Hyprland configuratie
\$mainMod = SUPER

# Autostart: statusbalk + IPC-daemon
exec-once = /usr/local/bin/zenithd
exec-once = quickshell -d || waybar

bind = \$mainMod, Q, exec, kitty
bind = \$mainMod, C, killactive
bind = \$mainMod, M, exit
bind = \$mainMod, V, togglefloating
bind = \$mainMod, R, exec, rofi -show drun || wofi --show drun

# Zenith Control Center
bind = \$mainMod, Escape, exec, /usr/local/bin/zenith

source = ~/.config/hypr/zenith.conf
EOF
fi

# zenithd als systeemdienst voor de doelgebruiker.
if [ "$NO_DAEMON" -eq 0 ]; then
    echo "-> zenithd systeemdienst registreren..."
    mkdir -p /etc/systemd/system
    cat > /etc/systemd/system/zenithd.service <<EOF
[Unit]
Description=ZenithOS IPC daemon (JSON-RPC 2.0)
After=network.target

[Service]
Type=simple
User=$TARGET_USER
ExecStart=/usr/local/bin/zenithd
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload
    systemctl enable zenithd.service
    systemctl start zenithd.service || echo "! zenithd start mislukt; controleer met: journalctl -u zenithd -e"
    echo "✓ zenithd actief op \$XDG_RUNTIME_DIR/zenith.sock"
else
    echo "- zenithd-systeemdienst overgeslagen (--no-daemon)."
fi

# Rechten herstellen
chown -R "$TARGET_USER:$TARGET_USER" "$TARGET_HOME/.config" 2>/dev/null || true

# ------------------------------------------------------------------
cat <<EOF

================================================
  ZenithOS is in de VM geïnstalleerd!          =
================================================
   ✓ /usr/local/bin/zenith        (Control Center)
   ✓ /usr/local/bin/zenithd       (IPC-daemon)
   ✓ Hyprland-sessie voor '$TARGET_USER'
   ✓ Quickshell / Waybar statusbalk

Volgende stappen in de VM:
  1. Herstart:            sudo reboot
  2. Log in als '$TARGET_USER'.
  3. Start de desktop:    Hyprland          (of kies de sessie in je login-manager)
  4. Open Zenith:         SUPER + Escape    (of het commando 'zenith')

Tip: om de daemon te testen vanuit een terminal:
  python3 -c "import socket;s=socket.socket(socket.AF_UNIX);s.connect('/run/user/\$(id -u)/zenith.sock');s.sendall(b'{\\"method\\":\\"ping\\",\\"id\\":1}\\n');s.shutdown(1);print(s.recv(4096).decode())"
EOF