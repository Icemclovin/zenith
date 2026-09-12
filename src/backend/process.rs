use std::process::{Command, Stdio};

/// Zorgt dat de geselecteerde statusbalk geïnstalleerd is (via pkexec pacman indien nodig)
pub fn ensure_bar_installed(bar: &str) {
    let pkg = match bar {
        "quickshell" => "quickshell",
        "waybar" => "waybar",
        _ => return,
    };

    if !is_command_available(bar) {
        let _ = Command::new("pkexec")
            .args(["pacman", "-S", "--noconfirm", pkg])
            .spawn();
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
        let _ = Command::new("pkexec")
            .args(["pacman", "-S", "--noconfirm", pkg])
            .spawn();
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
        let _ = Command::new("pkexec")
            .args(["pacman", "-S", "--noconfirm", pkg])
            .spawn();
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
fn is_command_available(cmd: &str) -> bool {
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