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
#[derive(Default)]
pub struct ShellCustom {
    pub raw_override: bool,
    pub custom_qml_path: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ControlCenterConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cc_pos")]
    pub position: String, // "top-right", "bottom-right", "top-left", "bottom-left", "floating-center"
    #[serde(default = "default_cc_width")]
    pub width: i32,
    #[serde(default = "default_cc_max_height")]
    pub max_height: i32,
    #[serde(default = "default_cc_border_radius")]
    pub border_radius: i32,
    #[serde(default = "default_true")]
    pub blur_behind: bool,
    #[serde(default = "default_cc_opacity")]
    pub opacity: f64,
    #[serde(default = "default_cc_cards")]
    pub cards: Vec<String>,
}

fn default_true() -> bool { true }
fn default_cc_pos() -> String { "top-right".to_string() }
fn default_cc_width() -> i32 { 380 }
fn default_cc_max_height() -> i32 { 600 }
fn default_cc_border_radius() -> i32 { 16 }
fn default_cc_opacity() -> f64 { 0.92 }
fn default_cc_cards() -> Vec<String> {
    vec![
        "wifi_toggle".to_string(),
        "bluetooth_toggle".to_string(),
        "dnd_toggle".to_string(),
        "nightlight_toggle".to_string(),
        "volume_slider".to_string(),
        "mic_slider".to_string(),
        "brightness_slider".to_string(),
        "mpris_card".to_string(),
        "battery_card".to_string(),
        "power_strip".to_string(),
    ]
}

impl Default for ControlCenterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            position: default_cc_pos(),
            width: default_cc_width(),
            max_height: default_cc_max_height(),
            border_radius: default_cc_border_radius(),
            blur_behind: true,
            opacity: default_cc_opacity(),
            cards: default_cc_cards(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OsdConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_osd_pos")]
    pub position: String, // "bottom", "top", "center", "top-right", "bottom-right"
    #[serde(default = "default_osd_timeout")]
    pub timeout_ms: u32,
    #[serde(default = "default_osd_width")]
    pub width: i32,
    #[serde(default = "default_osd_height")]
    pub height: i32,
    #[serde(default = "default_osd_orientation")]
    pub orientation: String, // "horizontal", "vertical"
    #[serde(default = "default_true")]
    pub show_percentage: bool,
    #[serde(default = "default_true")]
    pub show_icon: bool,
    #[serde(default = "default_osd_hardware_targets")]
    pub hardware_targets: Vec<String>, // ["volume", "mic", "brightness"]
}

fn default_osd_pos() -> String { "bottom".to_string() }
fn default_osd_timeout() -> u32 { 2000 }
fn default_osd_width() -> i32 { 260 }
fn default_osd_height() -> i32 { 48 }
fn default_osd_orientation() -> String { "horizontal".to_string() }
fn default_osd_hardware_targets() -> Vec<String> {
    vec![
        "volume".to_string(),
        "mic".to_string(),
        "brightness".to_string(),
    ]
}

impl Default for OsdConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            position: default_osd_pos(),
            timeout_ms: default_osd_timeout(),
            width: default_osd_width(),
            height: default_osd_height(),
            orientation: default_osd_orientation(),
            show_percentage: true,
            show_icon: true,
            hardware_targets: default_osd_hardware_targets(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LockscreenConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_ls_layout")]
    pub layout_preset: String, // "centered" | "left_aligned" | "split" | "free_canvas"
    #[serde(default = "default_ls_bg_mode")]
    pub background_mode: String, // "wallpaper_blur" | "custom_image" | "solid_black" | "transparent_acrylic"
    #[serde(default)]
    pub custom_image_path: Option<String>,
    #[serde(default = "default_ls_blur_radius")]
    pub blur_radius: u32,
    #[serde(default = "default_ls_dim_opacity")]
    pub dim_opacity: f64,
    #[serde(default = "default_ls_clock_format")]
    pub clock_format: String,
    #[serde(default = "default_ls_clock_font_size")]
    pub clock_font_size: u32,
    #[serde(default = "default_ls_clock_font_family")]
    pub clock_font_family: String,
    #[serde(default)]
    pub user_avatar_path: Option<String>,
    #[serde(default = "default_ls_greeting")]
    pub custom_greeting: String,
    #[serde(default = "default_ls_auth_icon")]
    pub auth_indicator_icon: String,
    #[serde(default = "default_true")]
    pub auth_shake_animation: bool,
    #[serde(default = "default_ls_cards")]
    pub cards: Vec<String>,
    #[serde(default)]
    pub custom_scripts: Vec<CustomScriptModule>,
}

fn default_ls_layout() -> String { "centered".to_string() }
fn default_ls_bg_mode() -> String { "wallpaper_blur".to_string() }
fn default_ls_blur_radius() -> u32 { 32 }
fn default_ls_dim_opacity() -> f64 { 0.45 }
fn default_ls_clock_format() -> String { "hh:mm".to_string() }
fn default_ls_clock_font_size() -> u32 { 72 }
fn default_ls_clock_font_family() -> String { "JetBrains Mono".to_string() }
fn default_ls_greeting() -> String { "Welkom terug, {user}".to_string() }
fn default_ls_auth_icon() -> String { "🔒".to_string() }
fn default_ls_cards() -> Vec<String> {
    vec![
        "clock".to_string(),
        "avatar".to_string(),
        "auth".to_string(),
        "mpris".to_string(),
        "battery_network".to_string(),
    ]
}

impl Default for LockscreenConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            layout_preset: default_ls_layout(),
            background_mode: default_ls_bg_mode(),
            custom_image_path: None,
            blur_radius: default_ls_blur_radius(),
            dim_opacity: default_ls_dim_opacity(),
            clock_format: default_ls_clock_format(),
            clock_font_size: default_ls_clock_font_size(),
            clock_font_family: default_ls_clock_font_family(),
            user_avatar_path: None,
            custom_greeting: default_ls_greeting(),
            auth_indicator_icon: default_ls_auth_icon(),
            auth_shake_animation: true,
            cards: default_ls_cards(),
            custom_scripts: Vec::new(),
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
pub struct CustomIcons {
    // Bar modules
    pub workspaces: String,
    pub active_window: String,
    pub clock: String,
    pub systray: String,
    pub volume_high: String,
    pub volume_medium: String,
    pub volume_muted: String,
    pub brightness: String,
    pub power: String,
    pub launcher: String,
    pub brand: String,
    pub media: String,
    pub media_play: String,
    pub media_pause: String,
    pub media_prev: String,
    pub media_next: String,
    pub cpu: String,
    pub ram: String,
    pub battery_full: String,
    pub battery_low: String,
    pub battery_charging: String,
    pub network: String,
    pub bluetooth: String,
    // Control Center cards
    pub wifi: String,
    pub dnd_on: String,
    pub dnd_off: String,
    pub nightlight: String,
    pub mic: String,
    pub mic_muted: String,
    // Power strip
    pub lock: String,
    pub sleep: String,
    pub logout: String,
    pub reboot: String,
    pub shutdown: String,
    // Misc
    pub settings: String,
    pub script: String,
    pub custom: String,
    pub close: String,
}

impl Default for CustomIcons {
    fn default() -> Self {
        Self {
            workspaces: "🗂️".to_string(),
            active_window: "🪟".to_string(),
            clock: "🕒".to_string(),
            systray: "📥".to_string(),
            volume_high: "🔊".to_string(),
            volume_medium: "🔉".to_string(),
            volume_muted: "🔇".to_string(),
            brightness: "☀️".to_string(),
            power: "⏻".to_string(),
            launcher: "🚀".to_string(),
            brand: "🌟".to_string(),
            media: "🎵".to_string(),
            media_play: "▶".to_string(),
            media_pause: "⏸".to_string(),
            media_prev: "⏮".to_string(),
            media_next: "⏭".to_string(),
            cpu: "🖥".to_string(),
            ram: "💾".to_string(),
            battery_full: "🔋".to_string(),
            battery_low: "🪫".to_string(),
            battery_charging: "⚡".to_string(),
            network: "🌐".to_string(),
            bluetooth: "ᛒ".to_string(),
            wifi: "📶".to_string(),
            dnd_on: "🔕".to_string(),
            dnd_off: "🔔".to_string(),
            nightlight: "🌙".to_string(),
            mic: "🎙️".to_string(),
            mic_muted: "🎙️❌".to_string(),
            lock: "🔒".to_string(),
            sleep: "💤".to_string(),
            logout: "🚪".to_string(),
            reboot: "🔄".to_string(),
            shutdown: "⏻".to_string(),
            settings: "⚙️".to_string(),
            script: "💻".to_string(),
            custom: "✨".to_string(),
            close: "✕".to_string(),
        }
    }
}

fn default_language() -> String { "nl".to_string() }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZenithShellConfig {
    pub styling: ShellStyling,
    pub layout: ShellLayout,
    pub modules: ShellModules,
    pub panels: ShellPanels,
    pub custom: ShellCustom,
    #[serde(default)]
    pub control_center: ControlCenterConfig,
    #[serde(default)]
    pub osd: OsdConfig,
    #[serde(default)]
    pub custom_scripts: Vec<CustomScriptModule>,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub auto_palette: bool,
    #[serde(default)]
    pub wallpaper_path: String,
    #[serde(default)]
    pub custom_icons: CustomIcons,
    #[serde(default)]
    pub lockscreen: LockscreenConfig,
    /// Wanneer `true` toont de zijbalk de geavanceerde pagina's (Placeholder voor
    /// power users); bij `false` blijft de UI gericht op de kern-onderdelen.
    #[serde(default)]
    pub advanced_mode: bool,
}

impl Default for ZenithShellConfig {
    fn default() -> Self {
        Self {
            styling: ShellStyling::default(),
            layout: ShellLayout::default(),
            modules: ShellModules::default(),
            panels: ShellPanels::default(),
            custom: ShellCustom::default(),
            control_center: ControlCenterConfig::default(),
            osd: OsdConfig::default(),
            custom_scripts: Vec::new(),
            language: default_language(),
            auto_palette: false,
            wallpaper_path: String::new(),
            custom_icons: CustomIcons::default(),
            lockscreen: LockscreenConfig::default(),
            advanced_mode: false,
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

    /// Ontdek custom QML modulen in een opgegeven map (injecteerbaar voor tests).
    /// Returnt alleen modulen die nog niet in `existing` zitten en die geen ingebouwde zijn.
    fn discover_custom_modules_in_dir(
        dir: &PathBuf,
        custom_icon: &str,
        existing: &[ModuleInfo],
    ) -> Vec<ModuleInfo> {
        let mut found = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "qml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if !existing.iter().any(|m| m.id == stem)
                            && !found.iter().any(|m: &ModuleInfo| m.id == stem)
                        {
                            found.push(ModuleInfo {
                                id: stem.to_string(),
                                name: format!("✨ Custom: {}", stem),
                                icon: custom_icon.to_string(),
                                description: format!("Aangepast QML widget: {}.qml", stem),
                                is_custom: true,
                            });
                        }
                    }
                }
            }
        }
        found
    }

    pub fn cards_dir() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".config/quickshell/cards"))
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
        let cfg = Self::load_or_default();
        let mut list = vec![
            ModuleInfo {
                id: "workspaces".to_string(),
                name: "Hyprland Werkbladen".to_string(),
                icon: cfg.custom_icons.workspaces.clone(),
                description: "Interactieve werkbladknoppen via Hyprland IPC".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "active_window".to_string(),
                name: "Actief Venster".to_string(),
                icon: cfg.custom_icons.active_window.clone(),
                description: "Titel van het huidige gefocuste venster".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "clock".to_string(),
                name: "Klok & Kalender".to_string(),
                icon: cfg.custom_icons.clock.clone(),
                description: "Tijd en datumweergave met klikbare acties".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "systray".to_string(),
                name: "Systeemvak (Tray)".to_string(),
                icon: cfg.custom_icons.systray.clone(),
                description: "StatusNotifiers en achtergrondapplicaties".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "volume".to_string(),
                name: "Geluid / Volume".to_string(),
                icon: cfg.custom_icons.volume_high.clone(),
                description: "Volumeniveau met klik voor dempen en scrolregeling".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "brightness".to_string(),
                name: "Helderheid".to_string(),
                icon: cfg.custom_icons.brightness.clone(),
                description: "Schermhelderheid aanpassen en uitlezen".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "power".to_string(),
                name: "Sessie / Afsluiten".to_string(),
                icon: cfg.custom_icons.power.clone(),
                description: "Sessiebeheer (afsluiten, herstarten, vergrendelen)".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "launcher".to_string(),
                name: "Applicatiemenu".to_string(),
                icon: cfg.custom_icons.launcher.clone(),
                description: "Snelstarter voor Rofi / Wofi menu".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "brand".to_string(),
                name: "Zenith Label".to_string(),
                icon: cfg.custom_icons.brand.clone(),
                description: "Zenith merklabel en distro-indicatie".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "media".to_string(),
                name: "Mediaspeler".to_string(),
                icon: cfg.custom_icons.media.clone(),
                description: "Now Playing trackweergave en play/pause knop".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "cpu".to_string(),
                name: "Processor (CPU)".to_string(),
                icon: cfg.custom_icons.cpu.clone(),
                description: "Realtime CPU-belasting monitor".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "ram".to_string(),
                name: "Werkgeheugen (RAM)".to_string(),
                icon: cfg.custom_icons.ram.clone(),
                description: "Actueel werkgeheugengebruik".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "battery".to_string(),
                name: "Batterij / Accu".to_string(),
                icon: cfg.custom_icons.battery_full.clone(),
                description: "Accuniveau, dynamische iconen en laadindicator".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "network".to_string(),
                name: "Netwerk & Wifi".to_string(),
                icon: cfg.custom_icons.network.clone(),
                description: "Verbindingsstatus en SSID/Ethernet indicatie".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "bluetooth".to_string(),
                name: "Bluetooth".to_string(),
                icon: cfg.custom_icons.bluetooth.clone(),
                description: "Bluetooth powerstatus en klik-schakelaar".to_string(),
                is_custom: false,
            },
        ];

        // Ontdek custom modules in ~/.config/quickshell/modules/*.qml
        if let Some(dir) = Self::modules_dir() {
            list.extend(Self::discover_custom_modules_in_dir(
                &dir,
                &cfg.custom_icons.custom,
                &list,
            ));
        }

        // Voeg ook gedefinieerde Custom Script modules toe
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

    /// Geeft alle beschikbare kaarten voor het Control Center (ingebouwd, custom .qml en scripts)
    pub fn discover_available_cards() -> Vec<ModuleInfo> {
        let cfg = Self::load_or_default();
        let mut list = vec![
            ModuleInfo {
                id: "wifi_toggle".to_string(),
                name: "Wi-Fi Schakelaar".to_string(),
                icon: cfg.custom_icons.wifi.clone(),
                description: "Draadloos netwerk in-/uitschakelen en actuele verbinding".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "bluetooth_toggle".to_string(),
                name: "Bluetooth Schakelaar".to_string(),
                icon: cfg.custom_icons.bluetooth.clone(),
                description: "Bluetooth in-/uitschakelen en gekoppelde apparaten".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "dnd_toggle".to_string(),
                name: "Niet Storen (DND)".to_string(),
                icon: cfg.custom_icons.dnd_on.clone(),
                description: "Dunst notificaties onderdrukken".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "nightlight_toggle".to_string(),
                name: "Nachtmodus (Blauwfilter)".to_string(),
                icon: cfg.custom_icons.nightlight.clone(),
                description: "Kleurtemperatuur van het scherm aanpassen voor de avond".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "volume_slider".to_string(),
                name: "Geluidsvolume Slider".to_string(),
                icon: cfg.custom_icons.volume_high.clone(),
                description: "Interactieve volumeregeling en dempknop via WirePlumber".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "mic_slider".to_string(),
                name: "Microfoon Slider".to_string(),
                icon: cfg.custom_icons.mic.clone(),
                description: "Opnamevolume en microfoondemping".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "brightness_slider".to_string(),
                name: "Schermhelderheid Slider".to_string(),
                icon: cfg.custom_icons.brightness.clone(),
                description: "Helderheidsregeling van het beeldscherm via brightnessctl".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "mpris_card".to_string(),
                name: "Media Player Kaart".to_string(),
                icon: cfg.custom_icons.media.clone(),
                description: "Volwaardige mediaspeler met album-art en bediening".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "battery_card".to_string(),
                name: "Accustatus Kaart".to_string(),
                icon: cfg.custom_icons.battery_full.clone(),
                description: "Batterijniveau, laadstatus en resterende capaciteit".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "power_strip".to_string(),
                name: "Systeem Powerstrip".to_string(),
                icon: cfg.custom_icons.power.clone(), // using power, as power_strip had ⚡ originally, but user did not define "power_strip" in CustomIcons, they defined lock, sleep, logout, reboot, shutdown. The power strip icon in available cards used ⚡ so battery_charging is an option, or power. I'll use power. The prompt didn't specify exactly which to use for power_strip, but let's check: "⚡" was used. The only ⚡ is battery_charging. Or maybe I should just use `cfg.custom_icons.power` which defaults to ⏻. Let's see: user defined `pub power: String`. Let's use `power`.
                description: "Snelknoppen voor vergrendelen, slaapstand, reboot en uitschakelen".to_string(),
                is_custom: false,
            },
        ];

        // Ontdek custom kaarten in ~/.config/quickshell/cards/*.qml
        if let Some(dir) = Self::cards_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "qml") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem != "script_card" && !list.iter().any(|m| m.id == stem) {
                                list.push(ModuleInfo {
                                    id: stem.to_string(),
                                    name: format!("✨ Custom: {}", stem),
                                    icon: cfg.custom_icons.custom.clone(),
                                    description: format!("Aangepaste QML kaart: {}.qml", stem),
                                    is_custom: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        list
    }

    /// Geeft alle kaarten inclusief de custom scripts voor deze instantie
    pub fn discover_cards_for_instance(&self) -> Vec<ModuleInfo> {
        let mut list = Self::discover_available_cards();
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

    pub fn lockscreen_cards_dir() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".config/quickshell/lockscreen_cards"))
    }

    /// Geeft alle beschikbare kaarten voor het Lockscreen (standaard, custom .qml en scripts)
    pub fn discover_available_lockscreen_cards() -> Vec<ModuleInfo> {
        let cfg = Self::load_or_default();
        let mut list = vec![
            ModuleInfo {
                id: "clock".to_string(),
                name: "Klok & Datum".to_string(),
                icon: cfg.custom_icons.clock.clone(),
                description: "Elegante tijd-, dag- en datumweergave".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "auth".to_string(),
                name: "Authenticatie Box".to_string(),
                icon: cfg.custom_icons.lock.clone(),
                description: "Wachtwoordinvoer met PAM verificatie en animaties".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "avatar".to_string(),
                name: "Profielfoto & Welkom".to_string(),
                icon: "👤".to_string(),
                description: "Gebruikersavatar met begroeting".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "mpris".to_string(),
                name: "Mediaspeler".to_string(),
                icon: cfg.custom_icons.media.clone(),
                description: "Huidig spelend nummer met cover art en controls".to_string(),
                is_custom: false,
            },
            ModuleInfo {
                id: "battery_network".to_string(),
                name: "Status Indicator".to_string(),
                icon: cfg.custom_icons.battery_full.clone(),
                description: "Batterij-, netwerk- en caps-lock status".to_string(),
                is_custom: false,
            },
        ];

        // Ontdek custom kaarten in ~/.config/quickshell/lockscreen_cards/*.qml
        if let Some(dir) = Self::lockscreen_cards_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "qml") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let card_stem = stem.strip_suffix("_card").unwrap_or(stem);
                            if !list.iter().any(|m| m.id == card_stem || m.id == stem) {
                                list.push(ModuleInfo {
                                    id: stem.to_string(),
                                    name: format!("✨ Custom: {}", stem),
                                    icon: cfg.custom_icons.custom.clone(),
                                    description: format!("Aangepaste Lockscreen kaart: {}.qml", stem),
                                    is_custom: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Voeg custom scripts toe
        for script in &cfg.lockscreen.custom_scripts {
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

    /// Geeft alle lockscreen kaarten inclusief de custom scripts voor deze instantie
    pub fn discover_lockscreen_cards_for_instance(&self) -> Vec<ModuleInfo> {
        let mut list = Self::discover_available_lockscreen_cards();
        for script in &self.lockscreen.custom_scripts {
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
        // Gebruik een tijdelijk pad in plaats van de live ~/.config map, zodat de
        // test betrouwbaar werkt, ook in omgevingen waar die map overschrijfbaar/niet
        // beschrijfbaar is en zonder echte gebruikersconfig te vervuilen.
        let mut tmp = std::env::temp_dir();
        let unique = format!("zenith-{}-test-modules", std::process::id());
        tmp.push(unique);
        let _ = std::fs::create_dir_all(&tmp);

        let custom_path = tmp.join("TestCustomProbe.qml");
        std::fs::write(&custom_path, "import QtQuick\nItem{}\n")
            .expect("moet een testbestand kunnen schrijven in de temp map");

        let existing: Vec<ModuleInfo> = Vec::new();
        let modules = ZenithShellConfig::discover_custom_modules_in_dir(&tmp, "✨", &existing);
        let found = modules.iter().find(|m| m.id == "TestCustomProbe");
        assert!(found.is_some(), "Custom module TestCustomProbe should be discovered");
        let m = found.unwrap();
        assert!(m.is_custom);
        assert!(m.name.contains("TestCustomProbe"));

        let _ = std::fs::remove_file(custom_path);
        let _ = std::fs::remove_dir(&tmp);
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

    #[test]
    fn test_control_center_and_osd_config() {
        let mut cfg = ZenithShellConfig::default();
        cfg.control_center.width = 420;
        cfg.control_center.position = "bottom-right".to_string();
        cfg.control_center.cards.push("script:gpu_temp".to_string());
        cfg.osd.orientation = "vertical".to_string();
        cfg.osd.timeout_ms = 3000;

        let json = serde_json::to_string_pretty(&cfg).expect("Serialization failed");
        let parsed: ZenithShellConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.control_center.width, 420);
        assert_eq!(parsed.control_center.position, "bottom-right");
        assert!(parsed.control_center.cards.contains(&"script:gpu_temp".to_string()));
        assert_eq!(parsed.osd.orientation, "vertical");
        assert_eq!(parsed.osd.timeout_ms, 3000);
    }

    #[test]
    fn test_discover_available_cards() {
        let cards = ZenithShellConfig::discover_available_cards();
        assert!(!cards.is_empty());
        assert!(cards.iter().any(|c| c.id == "wifi_toggle"));
        assert!(cards.iter().any(|c| c.id == "bluetooth_toggle"));
        assert!(cards.iter().any(|c| c.id == "volume_slider"));
        assert!(cards.iter().any(|c| c.id == "brightness_slider"));
        assert!(cards.iter().any(|c| c.id == "mpris_card"));
        assert!(cards.iter().any(|c| c.id == "power_strip"));
    }

    #[test]
    fn test_custom_icons_and_language() {
        let mut cfg = ZenithShellConfig::default();
        assert_eq!(cfg.language, "nl");
        assert!(!cfg.auto_palette);
        assert_eq!(cfg.custom_icons.volume_high, "🔊");
        assert_eq!(cfg.custom_icons.battery_full, "🔋");
        assert_eq!(cfg.custom_icons.wifi, "📶");

        cfg.custom_icons.volume_high = "󰕾".to_string();
        cfg.language = "en".to_string();
        cfg.auto_palette = true;

        let json = serde_json::to_string_pretty(&cfg).expect("Serialization failed");
        let parsed: ZenithShellConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.custom_icons.volume_high, "󰕾");
        assert_eq!(parsed.language, "en");
        assert!(parsed.auto_palette);
    }

    #[test]
    fn test_advanced_mode_roundtrip_and_default() {
        // Default is 'false' (basis-modus).
        let cfg = ZenithShellConfig::default();
        assert!(!cfg.advanced_mode);

        // Roundtrip: true wordt bewaard.
        let cfg2 = ZenithShellConfig { advanced_mode: true, ..ZenithShellConfig::default() };
        let json = serde_json::to_string(&cfg2).expect("Serialization failed");
        let parsed: ZenithShellConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert!(parsed.advanced_mode);

        // Achterwaartse compatibiliteit: een opgeslagen config ZONDER
        // 'advanced_mode' (oudere versie op schijf) moet nog steeds inlezen en
        // lever dan `false` op.
        let cfg_old = serde_json::to_string(&ZenithShellConfig::default()).expect("Serialization failed");
        let cfg_old_without_field = cfg_old
            .replace("\"advanced_mode\":false,", "")   // veld midden in het object
            .replace(",\"advanced_mode\":false", "");  // veld als laatste sleutel
        let parsed_legacy: ZenithShellConfig =
            serde_json::from_str(&cfg_old_without_field).expect("Legacy parse failed");
        assert!(!parsed_legacy.advanced_mode);
    }

    #[test]
    fn test_lockscreen_config() {
        let mut cfg = ZenithShellConfig::default();
        assert!(cfg.lockscreen.enabled);
        assert_eq!(cfg.lockscreen.layout_preset, "centered");
        assert_eq!(cfg.lockscreen.blur_radius, 32);
        assert_eq!(cfg.lockscreen.clock_font_size, 72);
        assert_eq!(cfg.lockscreen.auth_indicator_icon, "🔒");

        cfg.lockscreen.layout_preset = "split".to_string();
        cfg.lockscreen.blur_radius = 48;
        cfg.lockscreen.dim_opacity = 0.60;
        cfg.lockscreen.clock_format = "hh:mm:ss".to_string();
        cfg.lockscreen.clock_font_size = 96;
        cfg.lockscreen.auth_indicator_icon = "🔑".to_string();
        cfg.lockscreen.custom_scripts.push(CustomScriptModule {
            id: "ls_script".to_string(),
            name: "Lock Notice".to_string(),
            icon: "ℹ️".to_string(),
            command: "echo 'Security Notice'".to_string(),
            interval_seconds: 60,
            on_click: None,
        });

        let json = serde_json::to_string_pretty(&cfg).expect("Serialization failed");
        let parsed: ZenithShellConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.lockscreen.layout_preset, "split");
        assert_eq!(parsed.lockscreen.blur_radius, 48);
        assert_eq!(parsed.lockscreen.dim_opacity, 0.60);
        assert_eq!(parsed.lockscreen.clock_format, "hh:mm:ss");
        assert_eq!(parsed.lockscreen.clock_font_size, 96);
        assert_eq!(parsed.lockscreen.auth_indicator_icon, "🔑");
        assert_eq!(parsed.lockscreen.custom_scripts.len(), 1);
        assert_eq!(parsed.lockscreen.custom_scripts[0].name, "Lock Notice");
    }

    #[test]
    fn test_discover_available_lockscreen_cards() {
        let cards = ZenithShellConfig::discover_available_lockscreen_cards();
        assert!(!cards.is_empty());
        assert!(cards.iter().any(|c| c.id == "clock"));
        assert!(cards.iter().any(|c| c.id == "auth"));
        assert!(cards.iter().any(|c| c.id == "avatar"));
        assert!(cards.iter().any(|c| c.id == "mpris"));
        assert!(cards.iter().any(|c| c.id == "battery_network"));
    }
}

