# Maintainer: Zenith Team <dev@zenith.org>
pkgname=zenith-control
pkgver=0.1.0
pkgrel=1
pkgdesc="Next-generation visual studio and control center for Hyprland and Quickshell"
arch=('x86_64')
url="https://github.com/zenith/zenith"
license=('MIT')
depends=(
    'gtk4'
    'libadwaita'
    'quickshell'
    'waybar'
    'rofi'
    'kitty'
    'brightnessctl'
    'playerctl'
    'wireplumber'
    'bluez-utils'
    'networkmanager'
    'ttf-jetbrains-mono-nerd'
    'polkit'
    'xdg-utils'
    'which'
    'procps-ng'
    'psmisc'
    'curl'
    'jq'
)
makedepends=('cargo' 'rust')
source=()

build() {
    cargo build --release
}

package() {
    install -Dm755 "target/release/zenith" "${pkgdir}/usr/bin/zenith"

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
}
