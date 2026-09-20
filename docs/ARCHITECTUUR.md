# ZenithOS — Architectuur & Implementatiestatus

Dit bestand brengt de specificatie (zie `ZenithOS-Specificatie.md`) in kaart met de
daadwerkelijke code in deze repository. Zo zie je altijd wat er al gebouwd is en wat
nog openstaat.

> **Belangrijk:** Er wordt op dit moment **geen ISO** gebouwd. Het doel is eerst testen
> in een VM via de terminal.

---

## Huidige status (overzicht)

| Sprint                          | Status   |
|---------------------------------|----------|
| 1. Core Foundation (zenithd + IPC) | Klaar (Sprint 1) |
| 2. Quickshell Integration         | Deels (GUI stuurt Quickshell aan) |
| 3. OpSec & Privacy Engine         | Nog niet gestart |
| 4. Tool Hub & Blueprints          | Nog niet gestart |
| 5. ISO Orchestration              | **Uitgesteld (geen ISO)** |

---

## Mapping van specificatie → code

### Presentation Layer (Quickshell/QML) — Sprint 2
- Bestaande, werkende Quickshell-integratie in de Zenith Control Center GUI.
- Zie `src/backend/`, `src/backend/quickshell*` en de statusbalk-designer.

### Zenith Control Center (Rust/Gtk4) — klaar
- Centrale app voor systeembeheer. Zie `src/main.rs`, `src/ui/`.

### zenithd (Rust Daemon) — Sprint 1 (klaar)
- **Binair**: `src/zenithd_main.rs` + `src/zenith_ipc.rs` (en `target/release/zenithd`).
- **FR-01**: JSON-RPC 2.0-server op `$XDG_RUNTIME_DIR/zenith.sock` (Unix domain socket).
- Methoden: `ping`, `get_status`, `reload_statusbar` (SIGUSR1 naar Quickshell),
  `echo`; correcte JSON-RPC-foutantwoorden (-32700, -32601).
- **Two-way sync (FR-01/FR-03)**: `sync.list`, `sync.get_config`, `sync.set_config`.
  Watch-free: elk verzoek leest/schrijft het configbestand live, zodat het bestand
  de Single Source of Truth blijft en externe bewerkingen zichtbaar zijn. Alleen een
  allow-list van beheerde bestanden (zenith-hypr, quickshell, waybar) is lees-/
  schrijfbaar — willekeurige paden worden geweigerd.
- Transport: één verbinding per verzoek met write-half-close als berichtgrens.
- Binaire grootte: ~475 KB (voldoet ruim aan NFR-05).
- Client: `src/backend/ipc_client.rs` — de GUI toont op het dashboard of de daemon
  actief is en kan hem starten/stoppen.

### zenith-priv-broker (Rust) — Sprint 3+
- Helper via D-Bus/Polkit voor bevoorrechte operaties (schrijven naar `/etc`).
- Nog niet gestart.

---

## VM-installatie via terminal

Zonder ISO wordt ZenithOS in een VM geïnstalleerd op een bestaand Arch-installatiemedium.
Zie:

- [`scripts/`](../scripts/) — installer en archinstall-profiel.
- `scripts/README.md` — de exacte stappen in de VM (archinstall → `zenithos-vm-install.sh`).

De installer zet de volledige ZenithOS-desktop op (Hyprland, Quickshell/Waybar,
Zenith-app en de `zenithd`-systeemdienst).