# ZenithOS — VM-installatie via de terminal

Dit is de manier om ZenithOS in een virtuele machine te installeren **totdat er een
ISO is** (die bewust nog niet wordt gebouwd). Je gebruikt een bestaand
Arch Linux-installatiemedium in de VM en installeert via de terminal.

---

## Stap 1 — Basis Arch installeren in de VM met archinstall

Start de VM vanaf een **Arch Linux ISO** en open de terminal. Kies één van de twee:

### Optie A (aanbevolen): interactief archinstall met het Desktop-profiel

```bash
pacman -Sy archinstall
archinstall
```

Kies in de wizard:
- **Disk layout**: Auto → btrfs (optioneel; je kunt ook ext4 kiezen).
- **Profile**: `Desktop` (selecteer daarin de extra pakketten of laat het basis).
- **Bootloader**: systemd-boot.
- **Users**: maak hier je gebruiker aan (bijv. `student`).
- **Audio**: Pipewire (of laat Auto).

### Optie B: scripted met het meegeleverde profiel

```bash
pacman -Sy archinstall
archinstall --config /mnt/zenithos/scripts/archinstall-zenithos.json --silent
```

> Het JSON-profiel is een startpunt/template en kan per archinstall-versie kleine
> aanpassing vragen. De eerste keer is "Optie A" het veiligst.

Na afloop invoegen de configuratie encore een keer te controleren:

```bash
umount -R /mnt 2>/dev/null; reboot
```

---

## Stap 2 — Log in en installeer ZenithOS-desktop

Na de reboot log je in (Op de eerste keer als de `installer`-gebruiker of via
`[F2]`). Open een terminal en draai de installer. Maak de Zenith-repo beschikbaar
(netwerk-hulpmiddelen zijn al geïnstalleerd):

### a) Repository binnenhalen (als je deze nog niet op de VM hebt)

```bash
cd ~
sudo pacman -S --needed git base-devel rust cargo
git clone https://github.com/zenithos/core.git zenithos
cd zenithos
```

### b) De ZenithOS-desktop installeren

```bash
sudo ./scripts/zenithos-vm-install.sh --user "$USER"
```

Dit doet automatisch:
- installeert alle desktop- en bouwpakketten (Hyprland, Quickshell, Waybar, Kitty,
  Rofi, NetworkManager, ...);
- bouwt `zenith` en `zenithd` (release) en installeert ze in `/usr/local/bin`;
- maakt een basis `hyprland.conf` aan die de statusbalk en de daemon start;
- registreert `zenithd` als systeemdienst (JSON-RPC 2.0 op
  `$XDG_RUNTIME_DIR/zenith.sock`).

Daarna:

```bash
sudo reboot
```

---

## Stap 3 — Vanaf de desktop

- Log in als je gebruiker en start Hyprland (of gebruik je login-manager).
- **Open het Zenith Control Center**: druk `SUPER + Escape`, of typ `zenith` in een
  terminal.
- Zodra de GUI draait zie je op het dashboard of de `zenithd` daemon actief is.

---

## Veelgestelde vragen

**Er is nog geen ISO, waarom?**
De gebruiker wil ZenithOS eerst in een VM testen vóórdat er een ISO wordt gemaakt.
Deze installateur is de testroute.

**Hoe test ik de daemon zonder GUI?**
In een terminal:

```bash
python3 -c \
  "import socket;s=socket.socket(socket.AF_UNIX);\
   s.connect('/run/user/$(id -u)/zenith.sock');\
   s.sendall(b'{\"method\":\"ping\",\"id\":1}\n');s.shutdown(1);\
   print(s.recv(4096).decode())"
```

Eén verbinding per verzoek (write-half-close = einde van een JSON-RPC-bericht).

**Wil ik later een ISO?**
Zodra de test in de VM slaagt, kan de ISO-orchestratie (Sprint 5) worden gebouwd.

---

## Bestanden

| Bestand                          | Omschrijving                                    |
|----------------------------------|-------------------------------------------------|
| `zenithos-vm-install.sh`         | Terminal-installer voor de ZenithOS-desktop.    |
| `archinstall-zenithos.json`      | Archinstall-profiel (template) voor de basis.   |
| `README.md`                      | Dit document.                                   |