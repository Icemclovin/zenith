# Zenith

> ⚠️ **Status: in test en ontwikkeling.** Zenith is nog niet productierijp. Verwacht bugs en
> veranderingen. Test het bij voorkeur in een VM.

**Zenith** is een grafische Control Center-app voor **Arch Linux met Hyprland**, onderdeel van
het grotere idee *ZenithOS*. Je kunt je desktop instellen zonder zelf configbestanden te bewerken.

## Wat het doet

- **Statusbalk-designer**: stel je Quickshell/Waybar-balk samen met drag-and-drop
- **Sneltoetsen**: bekijk en wijzig Hyprland-keybinds via een UI
- **Thema's & uiterlijk**: kleurenpaletten, iconen, lockscreen, OSD en Fastfetch
- **Back-up & herstel**: sla je volledige configuratie op en zet hem terug
- **Tool Hub**: installeer pakketten en vooraf ingestelde "blueprints"
- **zenithd**: een kleine achtergrond-daemon (JSON-RPC via een Unix-socket) die de configbestanden
  synchroon houdt met de app

Gebouwd met **Rust**, **GTK4** en **libadwaita**.

## Installeren (Arch + Hyprland)

```bash
git clone https://github.com/Icemclovin/zenith.git
cd zenith
./install.sh
```

Meer details over de architectuur en de voortgang per sprint staan in [`docs/`](docs/).

## Over dit project: gebouwd met AI

Dit project is **volledig met AI gegenereerd** (vibe coding). Ik heb zelf geen Rust-code
geschreven. Ik heb het gemaakt om te oefenen met AI-assisted development en om te verkennen
hoe ver je kunt komen in een taal die ik zelf (nog) niet beheers. Mijn rol: het idee, de
specificatie, de aansturing en het testen.
