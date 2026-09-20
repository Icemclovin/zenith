# ZenithOS — Volledige Systeemspecificatie, Architectuur en Design Documentatie

> Bron: `ZenithOS - Volledige Systeemspecificatie, Architectuur en Design Documentatie.docx`
> Deze markdown is een getrouwe, leesbare versie van het oorspronkelijke document.

---

## 1. Executive Summary & Visie

ZenithOS is niet simpelweg een Linux-distributie, maar een herdefiniëring van de relatie tussen de gebruiker en de onderliggende **Arch Linux**-basis. De visie **"Arch Power, Extreme Simplicity"** wordt gerealiseerd door een radicale **GUI-first** benadering zonder de transparantie van het systeem op te offeren.

### Kernprincipes

- **Zero-Barrier GUI first**: Elke systeeminstelling is visueel aanpasbaar zonder ooit een terminal te openen.
- **100% Arch-transparantie**: Geen eigen black-box configuratieformaten. De GUI modificeert direct de standaard Linux configuratiebestanden.
- **Single Source of Truth**: Door middel van een Rust-driven **Two-Way Sync Engine** blijven de visuele interface en de tekstuele configuratiebestanden altijd in sync.
- **Doelgroep**: Ontworpen voor de betrouwbaarheid die nodig is in de agrarische en financiële sector, gecombineerd met de tools en privacy-eisen van cybersecurity-specialisten en developers.

---

## 2. Epics & User Stories

### EPIC-01: Zenith Core & Live Two-Way Sync
De kern van het besturingssysteem is de **zenithd** daemon. Deze zorgt voor de communicatie tussen de verschillende componenten.

- **User Story**: Als gebruiker wil ik dat wijzigingen in een configuratiebestand (bijv. `hyprland.conf`) direct zichtbaar zijn in de GUI, en vice versa, zonder herstart.
- **Technologie**: Rust, Unix Domain Sockets, Inotify API, Hyprland IPC.

### EPIC-02: Visueel Drag-and-Drop Canvas
Een volledig aanpasbare desktopomgeving gebaseerd op **Quickshell (QML)**.

- **User Story**: Als gebruiker wil ik mijn statusbalkmodules herschikken door ze simpelweg te slepen op een canvas in het Control Center.
- **Technologie**: Quickshell, Qt Quick, Rust bindings.

### EPIC-03: Curated Tool Hub
Een gecureerde interface voor pakketbeheer.

- **User Story**: Als developer wil ik met één klik LazyVim of Wireshark installeren, waarbij ZenithOS automatisch de juiste groepspermissies en dependencies configureert.
- **Tools**: Pacman, Flatpak, AUR-support, automatische recepten (Blueprints).

### EPIC-04: Dual-State OpSec Engine
Privacy en anonimiteit als eerste-klas burgers.

- **User Story**: Als journalist wil ik met één toggle alle netwerkverkeer over een VPN dwingen met een hardwarematige killswitch en mijn MAC-adres maskeren.
- **Functies**: NFTables killswitch, MAC-spoofing via NetworkManager, RAM-backed browser profielen.

### EPIC-05: Visual Terminal & Fastfetch Studio
De terminal is niet langer een zwart gat, maar een visueel ontwerptool.

- **User Story**: Als power user wil ik mijn Fastfetch output visueel ontwerpen en direct zien hoe mijn iconen en kleuren eruitzien in de terminal.

### EPIC-06: Btrfs Fail-safe Rollbacks
Onverwoestbare stabiliteit door slim snapshot-beheer.

- **User Story**: Als een update mijn systeem onstabiel maakt, wil ik vanuit de bootloader direct terugkeren naar de staat van vóór de update.
- **Technologie**: Snapper, Btrfs, Grub/Systemd-boot hooks.

---

## 3. Software Requirements Specification (SRS)

### Niet-Functionele Eisen (NFR)

| ID      | Requirement          | Doelwaarde     |
|---------|----------------------|----------------|
| NFR-01  | Idle Memory Usage    | < 350 MB RAM   |
| NFR-02  | CPU Idle Overhead    | < 0.1%         |
| NFR-03  | IPC Latentie         | < 16 ms (sub-frame) |
| NFR-04  | Cold Start UI        | < 120 ms       |
| NFR-05  | Binary Size          | < 25 MB (Core Rust componenten) |

### Functionele Eisen (FR)

- **FR-01**: Alle communicatie tussen UI en Backend verloopt via JSON-RPC 2.0 op `$XDG_RUNTIME_DIR/zenith.sock`.
- **FR-02**: Het systeem moet gebruikmaken van een `zenith-priv-broker` voor acties waarvoor root-rechten nodig zijn, gebruikmakend van Polkit.
- **FR-03**: De parser moet in staat zijn om handmatige edits in `.conf`-bestanden te behouden (inclusief commentaar) tijdens het terugschrijven van GUI-wijzigingen.

---

## 4. Technisch Systeemontwerp & Architectuur

De architectuur is opgebouwd uit **drie lagen**, waarbij veiligheid en snelheid centraal staan door het gebruik van **Rust**.

### Componentenoverzicht

- **Presentation Layer (Quickshell/QML)**: De visuele schil voor de statusbalk, dashboard en lockscreen.
- **Zenith Control Center (Rust/Gtk4)**: De centrale applicatie voor systeembeheer.
- **zenithd (Rust Daemon)**: De state-manager die inotify events afhandelt en de "Two-Way Sync" aanstuurt.
- **zenith-priv-broker (Rust)**: Een minimalistische helper die via D-Bus communiceert voor bevoorrechte operaties (bijv. schrijven naar `/etc`).

### Datamodellen (Rust Snippets)

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct ZenithConfig {
    pub appearance: AppearanceConfig,
    pub opsec: OpSecConfig,
    pub status_bar: StatusBarConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpSecConfig {
    pub vpn_killswitch: bool,
    pub mac_randomization: bool,
    pub dns_over_https: String,
    pub ram_profiles_active: Vec, // Vec<String>
}
```

### IPC Protocol Voorbeeld

Verzoek om de statusbalk module te verplaatsen:

```json
{
  "jsonrpc": "2.0",
  "method": "update_module_position",
  "params": {
    "module_id": "clock_widget",
    "new_index": 2,
    "alignment": "center"
  },
  "id": 1
}
```

---

## 5. UI/UX Design System

Het design is gebaseerd op een **"Glassmorphism"** esthetiek met hoge contrastwaarden voor leesbaarheid.

### Visuele Specificaties

**Kleurenpalet:**

| Rol       | Kleur        | Omschrijving             |
|-----------|--------------|--------------------------|
| Primary   | `#458588`    | Deep Blue/Slate          |
| Background| `#1D2021`    | Ebony met 85% opacity    |
| Accent    | `#D79921`    | Muted Gold               |
| Success   | `#98971A`    | Sage Green               |

**Typografie:**
- UI Tekst: Inter (Medium, 11pt)
- Code/Terminal: JetBrainsMono Nerd Font (10pt)

**Componenten:**
- Sliders: 4px dikke tracks met 12px ronde handles.
- Badges: Gebruikt voor actieve OpSec status (bijv. "VPN ACTIVE").

### Drag-and-Drop Canvas

Het Control Center bevat een interactief canvas dat de statusbalk simuleert. Gebruikers kunnen modules (Klok, Netwerk, Batterij, CPU) verslepen. Bij het loslaten wordt een **SIGUSR1** naar Quickshell gestuurd om de UI live te hertekenen zonder verlies van state.

---

## 6. Implementatie Roadmap & Build Instructies

### Fasering (Sprints)

- **Sprint 1 — Core Foundation**: Setup `zenithd`, JSON-RPC server en basis inotify watchers.
- **Sprint 2 — Quickshell Integration**: Bouwen van de statusbalk en de communicatiebridge met de daemon.
- **Sprint 3 — OpSec & Privacy Engine**: Implementatie van de nftables wrappers en MAC-spoofing logica.
- **Sprint 4 — Tool Hub & Blueprints**: Ontwikkelen van de ALPM wrapper en installatierecepten.
- **Sprint 5 — ISO Orchestration**: Integratie met `archiso` en configureren van de Calamares installer.

### Build Instructies

```bash
# Clone de repository
git clone https://github.com/zenithos/core.git
cd core

# Build alle crates in release mode
cargo build --release

# Start de daemon voor testen
./target/release/zenithd --debug

# Build de ISO (vereist archiso)
sudo ./scripts/build-iso.sh --profile zenith-standard
```

### Bestandsstructuur

| Pad                   | Omschrijving                                  |
|-----------------------|-----------------------------------------------|
| `/src/zenithd`        | De Rust backend daemon.                       |
| `/src/zenith-gui`     | De Gtk4/Rust interface voor het Control Center. |
| `/shell`              | QML bestanden voor Quickshell.                |
| `/blueprints`         | YAML-bestanden met applicatie-recepten.       |