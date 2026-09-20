# Blueprints — Curated Tool Hub (EPIC-03)

> **Status: gepland (Sprint 4).** Deze map wordt gevuld zodra de Tool Hub
> (ALPM-wrapper + installatierecepten) wordt gebouwd.

Een blueprint is een YAML-bestand met een applicatierecept: welke pakketten,
welke groepspermissies en welke configuratie een "één-klik-installatie" nodig
heeft (bijv. LazyVim of Wireshark uit de specificatie).

Voorbeeld (tijdelijk, ter illustratie van de werkende indeling):

```yaml
# examples/lazyvim.yaml
name: LazyVim
packages:
  - neovim
  - git
deps:
  manager: pacman
configure: |
  git clone --filter=blob:none https://github.com/LazyVim/starter $HOME/.config/nvim
  nvim --headless "+Lazy! sync" +qa
```