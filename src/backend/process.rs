use std::process::{Command, Stdio};

/// Kern-pakketten waar Zenith (impliciet) op kan steunen. `install` is de
/// Arch-pakketnaam, `binary` het commando dat in $PATH moet bestaan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZenithDependency {
    Quickshell,
    Hyprland,
    Waybar,
    Rofi,
    Wofi,
}

impl ZenithDependency {
    pub const ALL: [ZenithDependency; 5] = [
        ZenithDependency::Quickshell,
        ZenithDependency::Hyprland,
        ZenithDependency::Waybar,
        ZenithDependency::Rofi,
        ZenithDependency::Wofi,
    ];

    /// Het commando dat in $PATH moet voorkomen om te bepalen of het geïnstalleerd is.
    pub fn binary(&self) -> &'static str {
        match self {
            ZenithDependency::Quickshell => "quickshell",
            ZenithDependency::Hyprland => "Hyprland",
            ZenithDependency::Waybar => "waybar",
            ZenithDependency::Rofi => "rofi",
            ZenithDependency::Wofi => "wofi",
        }
    }

    /// Arch-pakketnaam die via pacman geïnstalleerd moet worden.
    pub fn arch_package(&self) -> &'static str {
        match self {
            ZenithDependency::Quickshell => "quickshell",
            ZenithDependency::Hyprland => "hyprland",
            ZenithDependency::Waybar => "waybar",
            ZenithDependency::Rofi => "rofi-wayland",
            ZenithDependency::Wofi => "wofi",
        }
    }

    /// Een korte, mensvriendelijke omschrijving voor de UI.
    pub fn title(&self) -> &'static str {
        match self {
            ZenithDependency::Quickshell => "Quickshell",
            ZenithDependency::Hyprland => "Hyprland",
            ZenithDependency::Waybar => "Waybar",
            ZenithDependency::Rofi => "Rofi",
            ZenithDependency::Wofi => "Wofi",
        }
    }

    /// Of dit pakket op dit moment (niet) geïnstalleerd is.
    pub fn is_installed(&self) -> bool {
        is_command_available(self.binary())
    }
}

/// Geeft de lijst van kern-afhankelijkheden die ontbreken.
pub fn missing_dependencies() -> Vec<ZenithDependency> {
    ZenithDependency::ALL.iter().copied().filter(|d| !d.is_installed()).collect()
}

/// Installeert één pakket via pkexec pacman; valt terug op `sudo` wanneer
/// pkexec niet beschikbaar is. Dit toont een beheerders-prompt voor de gebruiker.
pub fn install_dependency(dep: ZenithDependency) {
    let pkg = dep.arch_package();
    install_packages(&[pkg]);
}

/// Installeert Quickshell (en eventuele quickshell-aanvullingen) met een beheerders-prompt.
pub fn install_quickshell() {
    install_packages(&["quickshell"]);
}

/// Installeert een lijst pakketten via pkexec (of sudo) met `pacman -S --noconfirm`.
pub fn install_packages(packages: &[&str]) {
    if packages.is_empty() {
        return;
    }
    // Gebruik pkexec voor de GUI-omgeving; val terug op sudo voor terminal-gebruikers.
    let installer = if is_command_available("pkexec") { "pkexec" } else { "sudo" };
    let _ = Command::new(installer)
        .args([pacman_path()])
        .args(["-S", "--noconfirm"])
        .args(packages)
        .spawn();
}

/// Pad naar het pacman-commando (altijd `/usr/bin/pacman`, tenzij het elders staat).
fn pacman_path() -> &'static str {
    if is_command_available("pacman") { "pacman" } else { "/usr/bin/pacman" }
}

/// Zorgt dat de geselecteerde statusbalk geïnstalleerd is (via pkexec pacman indien nodig)
pub fn ensure_bar_installed(bar: &str) {
    let pkg = match bar {
        "quickshell" => "quickshell",
        "waybar" => "waybar",
        _ => return,
    };

    if !is_command_available(bar) {
        install_packages(&[pkg]);
    }
}

/// Beheert het wisselen tussen verschillende statusbalken (Waybar, Quickshell of Geen)
pub fn set_active_bar(choice: &str) {
    ensure_bar_installed(choice);

    // 1. Sluit actieve instanties af zonder ruis op stderr als ze niet draaien
    let _ = Command::new("killall")
        .args(["-q", "-9", "waybar"])
        .status();

    let _ = Command::new("killall")
        .args(["-q", "-9", "quickshell"])
        .status();

    // 2. Start de geselecteerde statusbalk volledig ontkoppeld op
    match choice {
        "waybar" => {
            let _ = Command::new("waybar")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }
        "quickshell" => {
            let _ = Command::new("quickshell")
                .arg("-d")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }
        _ => {} // "none" laat beide gesloten
    }
}

/// Zorgt dat de geselecteerde launcher geïnstalleerd is (via pkexec indien nodig)
pub fn ensure_launcher_installed(launcher: &str) {
    let pkg = match launcher {
        "wofi" => "wofi",
        _ => "rofi-wayland",
    };

    if !is_command_available(launcher) {
        install_packages(&[pkg]);
    }
}

/// Zorgt dat de geselecteerde terminal emulator geïnstalleerd is
pub fn ensure_terminal_installed(term: &str) {
    let pkg = match term {
        "alacritty" => "alacritty",
        "foot" => "foot",
        _ => "kitty",
    };

    if !is_command_available(term) {
        install_packages(&[pkg]);
    }
}

/// Wijzigt de standaard login-shell van de huidige gebruiker via chsh
pub fn set_user_shell(shell: &str) {
    let user = match std::env::var("USER") {
        Ok(u) => u,
        Err(_) => return,
    };

    let shell_path = match shell {
        "bash" => "/bin/bash",
        "fish" => "/bin/fish",
        _ => "/bin/zsh",
    };

    if std::path::Path::new(shell_path).exists() {
        let _ = Command::new("chsh")
            .args(["-s", shell_path, &user])
            .spawn();
    }
}

/// Schakelt Dunst 'Do Not Disturb' modus in of uit via dunstctl
pub fn toggle_dunst_dnd(paused: bool) {
    let arg = if paused { "true" } else { "false" };
    let _ = Command::new("dunstctl")
        .args(["set-paused", arg])
        .spawn();
}

/// Herstart dunst (bij thema-wijzigingen)
#[allow(dead_code)]
pub fn restart_dunst() {
    let _ = Command::new("killall").args(["-q", "dunst"]).status();
    let _ = Command::new("dunst")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// Hulpmiddel om te controleren of een binary in $PATH staat
pub fn is_command_available(cmd: &str) -> bool {
    // 1. Directe controle via de PATH-omgevingsvariabele (onafhankelijk van 'which' binary)
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let full_path = dir.join(cmd);
            if full_path.is_file() {
                return true;
            }
        }
    }
    // 2. Val terug op which binary indien aanwezig
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Controleert of een procesnaam actief draait via pgrep
pub fn is_process_running(proc_name: &str) -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(proc_name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Voert een willekeurig shell-commando asynchroon uit
pub fn execute_cmd(cmd_str: &str) {
    let _ = Command::new("bash")
        .arg("-c")
        .arg(cmd_str)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_metadata_is_consistent() {
        // Elke dependency heeft een logisch binary- en pakketnaam.
        assert_eq!(ZenithDependency::Quickshell.binary(), "quickshell");
        assert_eq!(ZenithDependency::Quickshell.arch_package(), "quickshell");
        assert_eq!(ZenithDependency::Hyprland.binary(), "Hyprland");
        assert_eq!(ZenithDependency::Waybar.arch_package(), "waybar");
        assert_eq!(ZenithDependency::Rofi.arch_package(), "rofi-wayland");
        // Titel is nooit leeg en de enum is compleet.
        for d in ZenithDependency::ALL {
            assert!(!d.title().is_empty());
        }
        assert_eq!(ZenithDependency::ALL.len(), 5);
    }

    #[test]
    fn is_command_available_works() {
        // Een commando dat vrijwel nooit bestaat moet op false uitkomen.
        assert!(!is_command_available("zenith_deze_bestaat_niet_probeer_xyz"));
        // Onszelf (of een gegarandeerd aanwezige shell) moet gevonden worden.
        assert!(is_command_available("/bin/sh") || is_command_available("sh"));
    }

    #[test]
    fn install_packages_handles_empty() {
        // Lege lijst moet een no-op zijn; dit hoeft geen systeemcommando te voeren.
        install_packages(&[]);
    }
}