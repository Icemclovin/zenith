// ZenithOS — Tool Hub & Pakketbeheer (EPIC-03).
//
// Beheert de gecureerde "Tool Hub": een catalogus van tools (LazyVim, Wireshark,
// ...) die met één klik worden geïnstalleerd via een elwrapper over pacman/yay.
// Elke tool komt uit een "blueprint" (recept) en draait door `zenith-priv-broker`
// (polkit), zodat de GUI zelf geen root-rechten nodig heeft. De installatiestatus
// wordt live in de GUI getoond (download → install → gereed / fout).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::backend::process::is_command_available;

/// Fasen van een installatie, voor de live statusfeedback in de GUI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageStatus {
    Idle,
    Downloading,
    Installing,
    Done,
    Failed,
}

/// Machine-leesbare fase-labels die de GUI/e2e-tests kunnen matchen.
#[allow(dead_code)]
pub fn status_key(s: PackageStatus) -> &'static str {
    match s {
        PackageStatus::Idle => "idle",
        PackageStatus::Downloading => "download",
        PackageStatus::Installing => "install",
        PackageStatus::Done => "done",
        PackageStatus::Failed => "failed",
    }
}

/// Een tool in de Tool Hub (een "recept" uit een blueprint-bestand).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ToolHubItem {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Officiële/Arch-repo pakketten (via pacman/AUR).
    #[serde(default)]
    pub packages: Vec<String>,
    /// Optionele Flatpak-applicatie-id's (bijv. "org.wireshark.Wireshark").
    #[serde(default)]
    pub flatpak_ids: Vec<String>,
    /// Commando dat aanwezig moet zijn om "geïnstalleerd" te bepalen.
    #[serde(default)]
    pub check_binary: String,
    #[serde(default)]
    pub icon: String,
}

impl Default for ToolHubItem {
    fn default() -> Self {
        Self {
            key: String::new(),
            name: String::new(),
            description: String::new(),
            packages: Vec::new(),
            flatpak_ids: Vec::new(),
            check_binary: String::new(),
            icon: "🧩".to_string(),
        }
    }
}

impl ToolHubItem {
    /// Of deze tool op dit moment (niet) geïnstalleerd is.
    pub fn is_installed(&self) -> bool {
        if self.check_binary.is_empty() {
            return false;
        }
        is_command_available(self.check_binary.as_str())
    }
}

/// Ingebouwde catalogus (fallback en basisset wanneer er nog geen blueprint-
/// bestanden aanwezig zijn).
fn builtin_catalog() -> Vec<ToolHubItem> {
    vec![
        ToolHubItem {
            key: "lazyvim".to_string(),
            name: "LazyVim".to_string(),
            description: "Neovim met de LazyVim-distributie".to_string(),
            packages: vec!["neovim".to_string(), "git".to_string(), "ripgrep".to_string(), "fd".to_string()],
            flatpak_ids: Vec::new(),
            check_binary: "nvim".to_string(),
            icon: "💻".to_string(),
        },
        ToolHubItem {
            key: "wireshark".to_string(),
            name: "Wireshark".to_string(),
            description: "Netwerkanalyse voor pakketinspectie".to_string(),
            packages: vec!["wireshark".to_string(), "tshark".to_string()],
            flatpak_ids: vec!["org.wireshark.Wireshark".to_string()],
            check_binary: "wireshark".to_string(),
            icon: "🕵️".to_string(),
        },
    ]
}

/// Repository-blueprint-map (naast de binaries).
fn repo_blueprints_dir() -> PathBuf {
    PathBuf::from("blueprints")
}

/// Gebruikers-blueprint-map (om eigen recepten toe te voegen).
fn user_blueprints_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config/zenith/blueprints"))
}

/// Laad alle tools: ingebouwde catalogus aangevuld met blueprints uit de
/// blueprint-mappen (bestanden overschrijven de ingebouwde per `key`).
pub fn load_catalog() -> Vec<ToolHubItem> {
    let mut items = builtin_catalog();

    let mut dirs: Vec<PathBuf> = vec![repo_blueprints_dir()];
    if let Some(u) = user_blueprints_dir() {
        dirs.push(u);
    }

    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() || !path.extension().is_some_and(|e| e == "json") {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(item) = serde_json::from_str::<ToolHubItem>(&content) {
                        let existing = items.iter().position(|e| e.key == item.key);
                        match existing {
                            Some(i) => items[i] = item.clone(),
                            None => items.push(item),
                        }
                    }
                }
            }
        }
    }
    items
}

/// Pad van het statuslogbestand voor een tool, in $XDG_RUNTIME_DIR.
pub fn log_path(key: &str) -> PathBuf {
    let run = std::env::var("XDG_RUNTIME_DIR").ok();
    let name = format!("zenith-toolhub-{}.log", key);
    match run {
        Some(r) if !r.is_empty() => PathBuf::from(r).join(name),
        _ => PathBuf::from("/tmp").join(name),
    }
}

/// Lees de laatste STATUS-regel uit een logbestand; geeft de fase terug.
pub fn read_status(path: &PathBuf) -> PackageStatus {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.ends_with("STATUS done") {
                PackageStatus::Done
            } else if trimmed.ends_with("STATUS failed") {
                PackageStatus::Failed
            } else if trimmed.ends_with("STATUS installing") {
                PackageStatus::Installing
            } else if trimmed.ends_with("STATUS download") {
                PackageStatus::Downloading
            } else {
                PackageStatus::Idle
            }
        }
        Err(_) => PackageStatus::Idle,
    }
}

/// Zoek het `zenith-priv-broker`-script (PATH of naast de repo).
fn broker_path() -> String {
    // Eerst in PATH (geïnstalleerde systeembroker).
    if is_command_available("zenith-priv-broker") {
        return "zenith-priv-broker".to_string();
    }
    // Daarna lokaal naast de repo.
    if PathBuf::from("scripts/zenith-priv-broker").is_file() {
        return "scripts/zenith-priv-broker".to_string();
    }
    String::new()
}

/// Start een installatie van een tool (async, via de privileged broker).
/// De voortgang leest de GUI uit via `read_status(log_path(key))`.
pub fn spawn_install(item: &ToolHubItem) -> bool {
    let broker = broker_path();
    if broker.is_empty() {
        // Broker niet gevonden: val terug op de klassieke pkexec-pacman-pad.
        let names: Vec<&str> = item.packages.iter().map(|p| p.as_str()).collect::<Vec<&str>>();
        crate::backend::process::install_packages(&names);
        return true;
    }

    let joined = item.packages.join(" ");
    let script = format!("{} install {}", broker, joined);
    let _ = std::process::Command::new("bash")
        .arg("-c")
        .arg(script.as_str())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_key_roundtrip() {
        assert_eq!(status_key(PackageStatus::Idle), "idle");
        assert_eq!(status_key(PackageStatus::Downloading), "download");
        assert_eq!(status_key(PackageStatus::Installing), "install");
        assert_eq!(status_key(PackageStatus::Done), "done");
        assert_eq!(status_key(PackageStatus::Failed), "failed");
    }

    #[test]
    fn read_status_parses_log() {
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("zenith-toolhub-{}.log", std::process::id()));
        let _ = std::fs::write(&tmp, "STATUS installing\n");
        assert_eq!(read_status(&tmp), PackageStatus::Installing);
        let _ = std::fs::write(&tmp, "STATUS done\n");
        assert_eq!(read_status(&tmp), PackageStatus::Done);
        let _ = std::fs::write(&tmp, "STATUS failed\n");
        assert_eq!(read_status(&tmp), PackageStatus::Failed);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn builtin_catalog_has_tools() {
        let cat = builtin_catalog();
        assert!(cat.iter().any(|t| t.key == "lazyvim"));
        assert!(cat.iter().any(|t| t.key == "wireshark"));
    }

    #[test]
    fn load_catalog_includes_core_tools() {
        // Test draait vanuit de projectroot, dus de blueprint-bestanden worden
        // meegelezen (en overschrijven de ingebouwde per key).
        let cat = load_catalog();
        assert!(!cat.is_empty());
        assert!(cat.iter().any(|t| t.key == "lazyvim"));
        assert!(cat.iter().any(|t| t.key == "wireshark"));
        for t in cat {
            assert!(!t.name.is_empty());
            assert!(!t.key.is_empty());
        }
    }
}