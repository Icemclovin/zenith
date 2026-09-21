use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use std::process::Command;

// ==========================================
// KEYBIND MODEL
// ==========================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Keybind {
    #[serde(default)]
    pub id: String,
    /// Modifiers (b.v. ["SUPER","SHIFT"]) gevolgd door de toets.
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub group: String,
}

impl Keybind {
    /// Alle toetsen behalve de laatste zijn modifiers.
    pub fn modifiers(&self) -> Vec<String> {
        let len = self.keys.len();
        if len <= 1 {
            return Vec::new();
        }
        self.keys.iter().take(len - 1).cloned().collect::<Vec<String>>()
    }

    /// De laatste toets is de daadwerkelijke ingedrukte toets (of leeg).
    pub fn key(&self) -> String {
        match self.keys.last() {
            Some(k) => k.clone(),
            None => String::new(),
        }
    }
}

fn push_bind(list: &mut Vec<Keybind>, id: &str, keys: &[&str], desc: &str, group: &str) {
    list.push(Keybind {
        id: id.to_string(),
        keys: keys.iter().map(|k| k.to_string()).collect::<Vec<String>>(),
        description: desc.to_string(),
        group: group.to_string(),
    });
}

/// De standaard Arch/Hyprland-sneltoetsen (volledige set).
pub fn default_keybinds() -> Vec<Keybind> {
    let mut list = Vec::new();

    // Basis
    push_bind(&mut list, "terminal", &["SUPER", "Return"], "Terminal openen", "Basis");
    push_bind(&mut list, "launcher", &["SUPER", "Space"], "Applicatie-launcher", "Basis");
    push_bind(&mut list, "filemanager", &["SUPER", "E"], "Bestandsbeheer", "Basis");
    push_bind(&mut list, "lock", &["SUPER", "L"], "Scherm vergrendelen", "Basis");
    push_bind(&mut list, "control_center", &["SUPER", "Escape"], "Zenith Control Center openen", "Basis");

    // Focus
    push_bind(&mut list, "focus_left", &["SUPER", "Left"], "Focus naar links", "Focus");
    push_bind(&mut list, "focus_right", &["SUPER", "Right"], "Focus naar rechts", "Focus");
    push_bind(&mut list, "focus_up", &["SUPER", "Up"], "Focus naar boven", "Focus");
    push_bind(&mut list, "focus_down", &["SUPER", "Down"], "Focus naar beneden", "Focus");
    push_bind(&mut list, "move_left", &["SUPER", "SHIFT", "Left"], "Venster naar links verplaatsen", "Focus");
    push_bind(&mut list, "move_right", &["SUPER", "SHIFT", "Right"], "Venster naar rechts verplaatsen", "Focus");
    push_bind(&mut list, "move_up", &["SUPER", "SHIFT", "Up"], "Venster naar boven verplaatsen", "Focus");
    push_bind(&mut list, "move_down", &["SUPER", "SHIFT", "Down"], "Venster naar beneden verplaatsen", "Focus");

    // Werkruimten
    push_bind(&mut list, "workspace_1", &["SUPER", "1"], "Naar werkruimte 1", "Werkruimten");
    push_bind(&mut list, "workspace_2", &["SUPER", "2"], "Naar werkruimte 2", "Werkruimten");
    push_bind(&mut list, "workspace_3", &["SUPER", "3"], "Naar werkruimte 3", "Werkruimten");
    push_bind(&mut list, "workspace_4", &["SUPER", "4"], "Naar werkruimte 4", "Werkruimten");
    push_bind(&mut list, "workspace_5", &["SUPER", "5"], "Naar werkruimte 5", "Werkruimten");
    push_bind(&mut list, "workspace_6", &["SUPER", "6"], "Naar werkruimte 6", "Werkruimten");
    push_bind(&mut list, "workspace_7", &["SUPER", "7"], "Naar werkruimte 7", "Werkruimten");
    push_bind(&mut list, "workspace_8", &["SUPER", "8"], "Naar werkruimte 8", "Werkruimten");
    push_bind(&mut list, "workspace_9", &["SUPER", "9"], "Naar werkruimte 9", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_1", &["SUPER", "SHIFT", "1"], "Venster naar werkruimte 1", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_2", &["SUPER", "SHIFT", "2"], "Venster naar werkruimte 2", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_3", &["SUPER", "SHIFT", "3"], "Venster naar werkruimte 3", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_4", &["SUPER", "SHIFT", "4"], "Venster naar werkruimte 4", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_5", &["SUPER", "SHIFT", "5"], "Venster naar werkruimte 5", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_6", &["SUPER", "SHIFT", "6"], "Venster naar werkruimte 6", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_7", &["SUPER", "SHIFT", "7"], "Venster naar werkruimte 7", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_8", &["SUPER", "SHIFT", "8"], "Venster naar werkruimte 8", "Werkruimten");
    push_bind(&mut list, "move_to_workspace_9", &["SUPER", "SHIFT", "9"], "Venster naar werkruimte 9", "Werkruimten");

    // Venster
    push_bind(&mut list, "toggle_floating", &["SUPER", "F"], "Floating venster schakelen", "Venster");
    push_bind(&mut list, "toggle_fullscreen", &["SUPER", "M"], "Volledig scherm schakelen", "Venster");
    push_bind(&mut list, "kill_active", &["SUPER", "C"], "Actief venster sluiten", "Venster");
    push_bind(&mut list, "toggle_group", &["SUPER", "G"], "Groeperen schakelen", "Venster");

    // Multimedia
    push_bind(&mut list, "volume_up", &["XF86AudioRaiseVolume"], "Volume omhoog", "Multimedia");
    push_bind(&mut list, "volume_down", &["XF86AudioLowerVolume"], "Volume omlaag", "Multimedia");
    push_bind(&mut list, "volume_mute", &["XF86AudioMute"], "Volume dempen", "Multimedia");
    push_bind(&mut list, "brightness_up", &["XF86MonBrightnessUp"], "Helderheid omhoog", "Multimedia");
    push_bind(&mut list, "brightness_down", &["XF86MonBrightnessDown"], "Helderheid omlaag", "Multimedia");
    push_bind(&mut list, "media_play_pause", &["XF86AudioPlay"], "Media afspelen/pauzeren", "Multimedia");
    push_bind(&mut list, "media_next", &["XF86AudioNext"], "Volgende media", "Multimedia");
    push_bind(&mut list, "media_prev", &["XF86AudioPrev"], "Vorige media", "Multimedia");

    // Screenshots
    push_bind(&mut list, "screenshot", &["Print"], "Schermafbeelding volledig", "Screenshots");
    push_bind(&mut list, "screenshot_region", &["SUPER", "SHIFT", "Print"], "Schermafbeelding selectie", "Screenshots");

    list
}

// ==========================================
// PATHS
// ==========================================

fn config_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    PathBuf::from(format!("{}/.config/hypr", home))
}

pub fn json_path() -> PathBuf {
    config_dir().join("zenith-keybinds.json")
}

pub fn conf_path() -> PathBuf {
    config_dir().join("zenith-keybinds.conf")
}

// ==========================================
// LOAD / SAVE
// ==========================================

/// Laadt de door de gebruiker opgeslagen keybinds, of de standaard set.
pub fn load_keybinds() -> Vec<Keybind> {
    let p = json_path();
    if let Ok(text) = read_to_string(&p) {
        if let Ok(binds) = serde_json::from_str::<Vec<Keybind>>(&text) {
            if !binds.is_empty() {
                return binds;
            }
        }
    }
    default_keybinds()
}

/// Schrijft de keybinds atomair naar JSON en rendert de Hyprland .conf.
pub fn write_keybinds(binds: &Vec<Keybind>) -> bool {
    let _ = create_dir_all(config_dir());
    let final_path = json_path();
    let tmp_path = PathBuf::from(format!("{}.tmp", final_path.to_string_lossy()));
    if let Ok(json) = serde_json::to_string(binds) {
        if let Err(_e) = write(&tmp_path, &json) {
            return false;
        }
        let _ = std::fs::rename(&tmp_path, &final_path);
    }
    render_conf(binds)
}

// ==========================================
// Hyprland CONFIG GENERATION
// ==========================================

/// Vertaalt een keybind-id naar de juiste hyprland dispatch.
fn dispatch_for(id: &str) -> String {
    match id {
        "terminal" => "exec, kitty".to_string(),
        "launcher" => "exec, rofi -show drun".to_string(),
        "filemanager" => "exec, dolphin".to_string(),
        "lock" => "exec, hyprlock".to_string(),
        "control_center" => "exec, zenith".to_string(),
        "volume_up" => "exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+".to_string(),
        "volume_down" => "exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-".to_string(),
        "volume_mute" => "exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle".to_string(),
        "brightness_up" => "exec, brightnessctl set +5%".to_string(),
        "brightness_down" => "exec, brightnessctl set 5%-".to_string(),
        "media_play_pause" => "exec, playerctl play-pause".to_string(),
        "media_next" => "exec, playerctl next".to_string(),
        "media_prev" => "exec, playerctl previous".to_string(),
        "screenshot" => "exec, grimblast save active".to_string(),
        "screenshot_region" => "exec, grimblast save area".to_string(),
        _ => default_dispatch(id),
    }
}

fn default_dispatch(id: &str) -> String {
    if id.starts_with("move_to_workspace_") {
        let n = id.strip_prefix("move_to_workspace_").map(|s| s.to_string()).unwrap_or_default();
        return format!("movetoworkspacesilent, {}", n);
    } else if id.starts_with("workspace_") {
        let n = id.strip_prefix("workspace_").map(|s| s.to_string()).unwrap_or_default();
        return format!("workspace, {}", n);
    } else if id.starts_with("focus_") {
        let dir = id.strip_prefix("focus_").map(|s| s.to_string()).unwrap_or_default();
        return format!("movefocus, {}", dir);
    } else if id.starts_with("move_") {
        let dir = id.strip_prefix("move_").map(|s| s.to_string()).unwrap_or_default();
        return format!("movewindow, {}", dir);
    } else if id == "toggle_floating" {
        return "togglefloating".to_string();
    } else if id == "toggle_fullscreen" {
        return "fullscreen".to_string();
    } else if id == "kill_active" {
        return "killactive".to_string();
    } else if id == "toggle_group" {
        return "togglegroup".to_string();
    }
    "pass".to_string()
}

/// Bouwt de volledige .conf-inhoud uit een lijst keybinds (PURE functie).
pub fn build_conf_contents(binds: &[Keybind]) -> String {
    let mut out = String::new();
    out.push_str("# Generated by Zenith - wijzig je sneltoetsen via het Control Center.\n");
    out.push_str("$mainMod = SUPER\n\n");
    for bind in binds.iter() {
        if bind.keys.is_empty() {
            continue;
        }
        let mods = bind.modifiers();
        let mods_str = mods.join(", ");
        let keys_str = if mods_str.is_empty() {
            bind.key().clone()
        } else {
            format!("{}, {}", mods_str, bind.key())
        };
        let dispatch = dispatch_for(&bind.id);
        out.push_str(&format!("bind = {}, {}\n", keys_str, dispatch));
    }
    out
}

/// Schrijft de .conf atomair weg en herlaadt Hyprland.
fn render_conf(binds: &[Keybind]) -> bool {
    let _ = create_dir_all(config_dir());
    let final_path = conf_path();
    let tmp_path = PathBuf::from(format!("{}-tmp", final_path.to_string_lossy()));
    let content = build_conf_contents(binds);
    if let Err(_e) = write(&tmp_path, &content) {
        return false;
    }
    let _ = std::fs::rename(&tmp_path, &final_path);
    let _ = reload_keybinds();
    true
}

/// Herlaadt Hyprland zodat wijzigingen live in gaan.
fn reload_keybinds() -> bool {
    let _ = Command::new("hyprctl").args(["reload"]).spawn();
    true
}

// ==========================================
// INSTALL / SOURCE HOOK
// ==========================================

/// Zorgt dat de zenith-keybinds.conf bestaat en in hyprland.conf staat ingesourced.
pub fn ensure_installed() {
    let conf = conf_path();
    if !conf.exists() {
        let binds = default_keybinds();
        let _ = render_conf(&binds);
    }
    ensure_sourced();
}

/// Voegt `source = ...` toe aan hyprland.conf vóór de marker, als die er niet is.
fn ensure_sourced() {
    let hypr_conf = config_dir().join("hyprland.conf");
    if !hypr_conf.exists() {
        return;
    }
    if let Ok(content) = read_to_string(&hypr_conf) {
        let source_line = "source = ~/.config/hypr/zenith-keybinds.conf";
        let lines = content.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
        if lines.iter().any(|l| l.contains(source_line)) {
            return;
        }
        // Vind de regelindex van de marker "# Injected by Zenith" (indien aanwezig).
        let mut insert_at = lines.len();
        for (i, line) in lines.iter().enumerate() {
            if line.contains("# Injected by Zenith") {
                insert_at = i;
                break;
            }
        }
        let mut new_lines = Vec::new();
        for line in lines.iter().take(insert_at) {
            new_lines.push(line.clone());
        }
        new_lines.push(source_line.to_string());
        for line in lines.iter().skip(insert_at) {
            new_lines.push(line.clone());
        }
        let _ = write(&hypr_conf, new_lines.join("\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::default_keybinds;
    use super::{
        build_conf_contents, default_dispatch, dispatch_for, Keybind,
    };

    #[test]
    fn default_set_is_complete() {
        let binds = default_keybinds();
        assert!(binds.len() >= 30, "default set should contain >= 30 binds");
        let ids: Vec<String> = binds.iter().map(|b| b.id.clone()).collect::<Vec<String>>();
        for want in ["terminal", "launcher", "filemanager", "lock", "screenshot", "workspace_1", "volume_up", "toggle_fullscreen"] {
            assert!(ids.iter().any(|i| *i == want), "default set should contain all representative ids");
        }
    }

    #[test]
    fn dispatch_known_and_unknown() {
        assert!(dispatch_for("terminal").starts_with("exec, kitty"));
        assert!(dispatch_for("volume_up").contains("wpctl"));
        assert!(dispatch_for("workspace_3").starts_with("workspace, 3"));
        assert!(dispatch_for("move_to_workspace_5").starts_with("movetoworkspacesilent, 5"));
        assert!(dispatch_for("focus_left").starts_with("movefocus, left"));
        assert!(dispatch_for("unknown_id").starts_with("pass"));
        let d = default_dispatch("no_such_bind");
        assert_eq!(d, "pass");
    }

    #[test]
    fn modifiers_and_key_split() {
        let kb = Keybind {
            id: "test".to_string(),
            keys: vec!["SUPER".to_string(), "SHIFT".to_string(), "Return".to_string()],
            description: "d".to_string(),
            group: "g".to_string(),
        };
        let mods = kb.modifiers();
        assert_eq!(mods.len(), 2);
        assert_eq!(mods[0], "SUPER");
        assert_eq!(mods[1], "SHIFT");
        assert_eq!(kb.key(), "Return");
        let solo = Keybind {
            id: "solo".to_string(),
            keys: vec!["Print".to_string()],
            description: "d".to_string(),
            group: "g".to_string(),
        };
        assert_eq!(solo.modifiers().len(), 0);
        assert_eq!(solo.key(), "Print");
    }

    #[test]
    fn conf_contents_has_bind_lines() {
        let binds = default_keybinds();
        let content = build_conf_contents(&binds);
        assert!(content.contains("$mainMod = SUPER"));
        assert!(content.contains("bind = "), "conf should contain bind lines");
        assert!(content.contains("SUPER, Return, exec, kitty"));
        assert!(content.contains("SUPER, 3, workspace, 3"));
    }

    #[test]
    fn json_serde_roundtrip_pure() {
        let binds = default_keybinds();
        let json = serde_json::to_string(&binds).expect("serialize");
        let parsed: Vec<Keybind> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.len(), binds.len());
        assert_eq!(parsed[0].id, binds[0].id);
    }
}
