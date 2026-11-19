# Maintainer: Zenith Team <dev@zenith.org>
# ZenithOS Control Center + zenithd daemon — een pakket voor `pacman -S`.
#
# Vanuit deze PKGBUILD bouw je het `.pkg.tar.zst`-artefact dat via een repo/
# GitHub Release geïnstalleerd kan worden met:
#   sudo pacman -S zenith-control
#
_zenith_commit="e4f23505bd0e91ece6cb877699750fba2c004a0b"
pkgname=zenith-control
pkgver=0.1.0
pkgrel=13
pkgdesc="Visual studio en controlecentrum voor Hyprland & Quickshell (zenith + zenithd JSON-RPC-daemon)"
arch=('x86_64')
url="https://github.com/Icemclovin/zenith"
license=('MIT')
depends=(
    'gtk4'
    'libadwaita'
    'quickshell'
    'waybar'
    'rofi'
    'kitty'
    'dolphin'
    'brightnessctl'
    'playerctl'
    'wireplumber'
    'bluez-utils'
    'networkmanager'
    'inter'
    'ttf-jetbrains-mono-nerd'
    'papirus-icon-theme'
    'polkit'
    'xdg-utils'
    'curl'
    'jq'
    'procps-ng'
    'psmisc'
    'grimblast'
    'hyprland'
)
optdepends=(
    'hyprsunset: nachtlicht-widget in de balk'
    'power-profiles-daemon: energieprofiel-widget in de balk'
)
makedepends=('cargo' 'rust' 'git')

source=("${url}/archive/${_zenith_commit}.tar.gz")
sha256sums=('dd4b5342458e3c260fad175d9dcd95ac7d554accd84164864e33f20709eb3b40')
noextract=()

build() {
    cd "$srcdir/zenith-${_zenith_commit}"
    cargo build --release
}

package() {
    cd "$srcdir/zenith-${_zenith_commit}"
    install -Dm755 "target/release/zenith" "${pkgdir}/usr/bin/zenith"
    install -Dm755 "target/release/zenithd" "${pkgdir}/usr/bin/zenithd"

    # Zenith testen met de toetsen van een andere shell (bv. serpantinum), zonder iets op te slaan
    install -Dm755 "scripts/zenith-test-session" "${pkgdir}/usr/bin/zenith-test-session"

    # Private broker (polkit) voor de Tool Hub
    install -Dm755 "scripts/zenith-priv-broker" "${pkgdir}/usr/lib/zenith/zenith-priv-broker"

    # Desktop entry
    install -d "${pkgdir}/usr/share/applications"
    cat << 'DESKTOPEOF' > "${pkgdir}/usr/share/applications/org.zenith.control.desktop"
[Desktop Entry]
Name=Zenith Control Center
Comment=Hyprland & Quickshell Visual Studio and Desktop Control
Exec=/usr/bin/zenith
Icon=preferences-system
Terminal=false
Type=Application
Categories=Settings;System;Utility;
Keywords=Hyprland;Settings;Control;Waybar;Quickshell;Theme;
StartupWMClass=org.zenith.control
DESKTOPEOF

    # zenitd daemon als systeemdienst (user)
    install -d "${pkgdir}/usr/lib/systemd/user"
    cat << 'UNITEOF' > "${pkgdir}/usr/lib/systemd/user/zenithd.service"
[Unit]
Description=Zenith IPC daemon (JSON-RPC 2.0)
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/zenithd
Restart=on-failure

[Install]
WantedBy=graphical-session.target
UNITEOF
}