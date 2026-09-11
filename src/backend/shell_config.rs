use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShellStyling {
    pub background: String,
    pub accent: String,
    pub opacity: f64,
    pub rounding: i32,
    pub border_width: i32,
    pub border_color: String,
    pub text_color: String,
    pub font_size: i32,
    pub margin_h: i32,
    pub margin_v: i32,
    pub module_spacing: i32,
    pub pill_bg: String,
    pub pill_opacity: f64,
    pub pill_rounding: i32,
}

impl Default for ShellStyling {
    fn default() -> Self {
        Self {
            background: "#1e1e2e".to_string(),
            accent: "#89b4fa".to_string(),
            opacity: 0.90,
            rounding: 12,
            border_width: 1,
            border_color: "#45475a".to_string(),
            text_color: "#cdd6f4".to_string(),
            font_size: 11,
            margin_h: 8,
            margin_v: 6,
            module_spacing: 8,
            pill_bg: "#181825".to_string(),
            pill_opacity: 0.85,
            pill_rounding: 8,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShellLayout {
    pub position: String,
    pub height: i32,
    pub floating: bool,
    pub bar_style: String, // "unified", "islands", "minimal"
}

impl Default for ShellLayout {
    fn default() -> Self {
        Self {
            position: "top".to_string(),
            height: 38,
            floating: true,
            bar_style: "unified".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShellModules {
    pub left: Vec<String>,
    pub center: Vec<String>,
    pub right: Vec<String>,
}

impl Default for ShellModules {
    fn default() -> Self {
        Self {
            left: vec![
                "launcher".to_string(),
                "brand".to_string(),
                "workspaces".to_string(),
            ],
            center: vec![
                "active_window".to_string(),
                "media".to_string(),
            ],
            right: vec![
                "cpu".to_string(),
                "ram".to_string(),
                "battery".to_string(),
                "volume".to_string(),
                "brightness".to_string(),
                "network".to_string(),
                "bluetooth".to_string(),
                "systray".to_string(),
                "clock".to_string(),
                "power".to_string(),
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShellPanels {
    #[serde(default)]
    pub quick_settings: bool,
    pub control_center: bool,
    pub volume_osd: bool,
    pub brightness_osd: bool,
    pub notifications: bool,
}

impl Default for ShellPanels {
    fn default() -> Self {
        Self {
            quick_settings: true,
            control_center: false,
            volume_osd: true,
            brightness_osd: true,
            notifications: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShellCustom {
    pub raw_override: bool,
    pub custom_qml_path: Option<String>,
}

impl Default for ShellCustom {
    fn default() -> Self {
        Self {
            raw_override: false,
            custom_qml_path: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CustomScriptModule {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub command: String,
    pub interval_seconds: u32,
    #[serde(default)]
    pub on_click: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZenithShellConfig {
    pub styling: ShellStyling,
    pub layout: ShellLayout,
    pub modules: ShellModules,
    pub panels: ShellPanels,
    pub custom: ShellCustom,
    #[serde(default)]
    pub custom_scripts: Vec<CustomScriptModule>,
}

impl Default for ZenithShellConfig {
    fn default() -> Self {
        Self {
            styling: ShellStyling::default(),
            layout: ShellLayout::default(),
            modules: ShellModules::default(),
            panels: ShellPanels::default(),
            custom: ShellCustom::default(),
            custom_scripts: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub is_custom: bool,
}

impl ZenithShellConfig {
    pub fn config_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".config/quickshell/zenith-shell.json"))
    }

    pub fn modules_dir() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".config/quickshell/modules"))
    }

    pub fn load_or_default() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = read_to_string(&path) {
                    if let Ok(cfg) = serde_json::from_str::<ZenithShellConfig>(&content) {
                        return cfg;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = create_dir_all(parent);
            }
            let json = serde_json::to_string_pretty(self)
                .unwrap_or_else(|_| "{}".to_string());
            let mut file = File::create(&path)?;
            file.write_all(json.as_bytes())?;
        }
        Ok(())
    }

    /// Geeft de lijst van alle bekende ingebouwde modules en ontdekte custom modules
    pub fn discover_available_modules() -> Vec<ModuleInfo> {
        let mut list = vec![
            ModuleInfo {
                id: "workspaces".to_string(),
                name: "Hyprland Werkbladen".to_string(),
                icon: "🗂️".to_string(),
                description: "Interactieve werkbladknoppen via Hyprland IPC".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "active_window".to_string(),
                name: "Actief Venster".to_string(),
                icon: "🪟".to_string(),
                description: "Titel van het huidige gefocuste venster".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "clock".to_string(),
                name: "Klok & Kalender".to_string(),
                icon: "🕒".to_string(),
                description: "Tijd en datumweergave met klikbare acties".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "systray".to_string(),
                name: "Systeemvak (Tray)".to_string(),
                icon: "📥".to_string(),
                description: "StatusNotifiers en achtergrondapplicaties".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "volume".to_string(),
                name: "Geluid / Volume".to_string(),
                icon: "🔊".to_string(),
                description: "Volumeniveau met klik voor dempen en scrolregeling".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "brightness".to_string(),
                name: "Helderheid".to_string(),
                icon: "☀️".to_string(),
                description: "Schermhelderheid aanpassen en uitlezen".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "power".to_string(),
                name: "Sessie / Afsluiten".to_string(),
                icon: "⏻".to_string(),
                description: "Sessiebeheer (afsluiten, herstarten, vergrendelen)".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "launcher".to_string(),
                name: "Applicatiemenu".to_string(),
                icon: "🚀".to_string(),
                description: "Snelstarter voor Rofi / Wofi menu".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "brand".to_string(),
                name: "Zenith Label".to_string(),
                icon: "🌟".to_string(),
                description: "Zenith merklabel en distro-indicatie".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "media".to_string(),
                name: "Mediaspeler".to_string(),
                icon: "🎵".to_string(),
                description: "Now Playing trackweergave en play/pause knop".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "cpu".to_string(),
                name: "Processor (CPU)".to_string(),
                icon: "🖥️".to_string(),
                description: "Realtime CPU-belasting monitor".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "ram".to_string(),
                name: "Werkgeheugen (RAM)".to_string(),
                icon: "💾".to_string(),
                description: "Actueel werkgeheugengebruik".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "battery".to_string(),
                name: "Batterij / Accu".to_string(),
                icon: "🔋".to_string(),
                description: "Accuniveau, dynamische iconen en laadindicator".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "network".to_string(),
                name: "Netwerk & Wifi".to_string(),
                icon: "🌐".to_string(),
                description: "Verbindingsstatus en SSID/Ethernet indicatie".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "bluetooth".to_string(),
                name: "Bluetooth".to_string(),
                icon: "ᛒ".to_string(),
                description: "Bluetooth powerstatus en klik-schakelaar".to_string(),
                is_custom: false,
            },
        ];

        // Ontdek custom modules in ~/.config/quickshell/modules/*.qml
        if let Some(dir) = Self::modules_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "qml") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            // Als het nog niet in de lijst staat, is het een custom module
                            if !list.iter().any(|m| m.id == stem) {
                                list.push(ModuleInfo {
                                    id: stem.to_string(),
                                    name: format!("✨ Custom: {}", stem),
                                    icon: "✨".to_string(),
                                    description: format!("Aangepast QML widget: {}.qml", stem),
                                    is_custom: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Voeg ook gedefinieerde Custom Script modules toe
        let cfg = Self::load_or_default();
        for script in &cfg.custom_scripts {
            let script_id = format!("script:{}", script.id);
            if !list.iter().any(|m| m.id == script_id) {
                list.push(ModuleInfo {
                    id: script_id,
                    name: format!("💻 Script: {}", script.name),
                    icon: script.icon.clone(),
                    description: format!("`{}` (elke {}s)", script.command, script.interval_seconds),
                    is_custom: true,
                });
            }
        }

        list
    }

    /// Geeft alle modules inclusief de scripts van deze specifieke instantie
    pub fn discover_modules_for_instance(&self) -> Vec<ModuleInfo> {
        let mut list = Self::discover_available_modules();
        for script in &self.custom_scripts {
            let script_id = format!("script:{}", script.id);
            if !list.iter().any(|m| m.id == script_id) {
                list.push(ModuleInfo {
                    id: script_id,
                    name: format!("💻 Script: {}", script.name),
                    icon: script.icon.clone(),
                    description: format!("`{}` (elke {}s)", script.command, script.interval_seconds),
                    is_custom: true,
                });
            }
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_config_serialization() {
        let mut cfg = ZenithShellConfig::default();
        cfg.custom_scripts.push(CustomScriptModule {
            id: "weather".to_string(),
            name: "Weerbericht".to_string(),
            icon: "🌤️".to_string(),
            command: "curl -s 'wttr.in?format=1'".to_string(),
            interval_seconds: 300,
            on_click: Some("kitty -e curl wttr.in".to_string()),
        });
        cfg.layout.position = "left".to_string();

        let json = serde_json::to_string_pretty(&cfg).expect("Serialization failed");
        let parsed: ZenithShellConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(cfg, parsed);
        assert_eq!(parsed.styling.accent, "#89b4fa");
        assert_eq!(parsed.layout.bar_style, "unified");
        assert_eq!(parsed.layout.position, "left");
        assert_eq!(parsed.custom_scripts.len(), 1);
        assert_eq!(parsed.custom_scripts[0].name, "Weerbericht");
    }

    #[test]
    fn test_discover_available_modules() {
        let modules = ZenithShellConfig::discover_available_modules();
        assert!(!modules.is_empty());
        assert!(modules.iter().any(|m| m.id == "workspaces"));
        assert!(modules.iter().any(|m| m.id == "clock"));
        assert!(modules.iter().any(|m| m.id == "volume"));
        assert!(modules.iter().any(|m| m.id == "active_window"));
        assert!(modules.iter().any(|m| m.id == "systray"));
        assert!(modules.iter().any(|m| m.id == "power"));
    }

    #[test]
    fn test_discover_custom_module() {
        if let Some(dir) = ZenithShellConfig::modules_dir() {
            let test_custom_path = dir.join("TestCustomProbe.qml");
            let _ = std::fs::write(&test_custom_path, "import QtQuick\nItem{}\n");

            let modules = ZenithShellConfig::discover_available_modules();
            let found = modules.iter().find(|m| m.id == "TestCustomProbe");
            assert!(found.is_some(), "Custom module TestCustomProbe should be discovered");
            let m = found.unwrap();
            assert!(m.is_custom);
            assert!(m.name.contains("TestCustomProbe"));

            let _ = std::fs::remove_file(test_custom_path);
        }
    }

    #[test]
    fn test_custom_script_discovery() {
        let mut cfg = ZenithShellConfig::default();
        cfg.custom_scripts.push(CustomScriptModule {
            id: "my_ip".to_string(),
            name: "Mijn IP".to_string(),
            icon: "🌐".to_string(),
            command: "curl -s ifconfig.me".to_string(),
            interval_seconds: 60,
            on_click: None,
        });

        let modules = cfg.discover_modules_for_instance();
        let found = modules.iter().find(|m| m.id == "script:my_ip");
        assert!(found.is_some(), "Script module script:my_ip should be in list");
        assert!(found.unwrap().name.contains("Mijn IP"));
    }
}

