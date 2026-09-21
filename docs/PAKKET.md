# Zenith installeren via `pacman` (zonder git clone)

Zenith wordt geleverd als een Arch-pakket (`zenith-control`) dat je met
`pacman` installeert. Geen `git clone`, geen handmatige cargo-build.

## Vereisten
- Arch Linux (x86_64) op uw systeem.
- `pacman` en het recht om `sudo` (of root) te gebruiken.

## Stap 1 — Zenith-repo toevoegen aan `pacman.conf`

Edableer `/etc/pacman.conf` als root en voeg **onderaan** toe:

```ini
[zenith]
SigLevel = Optional TrustAll
Server = https://github.com/Icemclovin/zenith/releases/latest/download
```

> Gebruik `Optional TrustAll` alleen als u de pakketten zelf vertrouwt. Voor een
> productiesetup vervangt u dit door een door u gegenereerde sleutel via
> `repo-add --sign` + `SigLevel = Required`.

Synchroniseer daarna:

```bash
sudo pacman -Sy
```

> **Let op:** `-Sy` (verversen zonder volledige `-Syu`) kan gedeeltelijke
> upgrades geven. Doe liever een volledige `sudo pacman -Syu` wanneer u een
> update plant.

## Stap 2 — Installeer

```bash
sudo pacman -S zenith-control
```

Dit installeert:
- `/usr/bin/zenith`   — het Control Center (GUI)
- `/usr/bin/zenithd`  — de JSON-RPC IPC-daemon
- `/usr/lib/zenith/zenith-priv-broker` — de priviléged Tool Hub-broker (polkit)
- een desktop-entry en een systemd *user*-eenheid `zenithd.service`

Start met:
```bash
zenith              # het Control Center
systemctl --user start zenithd   # de IPC-daemon (optioneel, automatisch via exec-once)
```

## Stap 3 — Updaten na een wijziging

Omdat we het pakket via GitHub Releases hosten, is een update een nieuwe
versie van hetzelfde pakket op dezelfde Release-URL.

```bash
sudo pacman -Syu
# of specifiek:
sudo pacman -S zenith-control
```

Als uw `pacman.conf` de `[zenith]`-repo hierboven gebruikt, blijft `pacman —Syu`
uw Zenith automatisch bijwerken zodra er een nieuwe Release is geüpload.

## Release-procedure voor de beheerder

1. Bouw het pakket (vanuit de repo-root):
   ```bash
   makepkg -f --nodeps
   # → zenith-control-0.1.0-1-x86_64.pkg.tar.zst
   ```
2. Publiceer als GitHub Release (label: `zenith-control-0.1.0-1`, met als asset
   het `.pkg.tar.zst`-bestand):
   ```bash
   gh auth login
   gh release create zenith-control-0.1.0-1 \
       zenith-control-0.1.0-1-x86_64.pkg.tar.zst \
       --repo Icemclovin/zenith \
       --title "Zenith 0.1.0-1" --notes "Zenith Control v0.1.0-1"
   ```
3. Voor een klassieke lokale repo (optioneel) in plaats van GitHub Releases:
   ```bash
   repo-add /tmp/zenith.db.tar.zst zenith-control-0.1.0-1-x86_64.pkg.tar.zst
   ```
   en wijs `Server = file:///pad/naar/` aan in `pacman.conf`.

**Broncode/rebuilds:** de PKGBUILD verwijst via `_zenith_commit` en
`source=` naar het exacte git-commit waarop het pakket is gebouwd; `sha256sums`
staat in de PKGBUILD, zodat `makepkg` de download verifieert.