use serde::Deserialize;
use std::process::Command;

pub fn set_gaps_out(val: i32) {
    let _ = Command::new("hyprctl").args(["keyword", "general:gaps_out", &val.to_string()]).spawn();
}

pub fn set_gaps_in(val: i32) {
    let _ = Command::new("hyprctl").args(["keyword", "general:gaps_in", &val.to_string()]).spawn();
}

pub fn set_border_size(val: i32) {
    let _ = Command::new("hyprctl").args(["keyword", "general:border_size", &val.to_string()]).spawn();
}

pub fn set_rounding(val: i32) {
    let _ = Command::new("hyprctl").args(["keyword", "decoration:rounding", &val.to_string()]).spawn();
}

pub fn set_active_border_color(hex: &str) {
    let _ = Command::new("hyprctl").args(["keyword", "general:col.active_border", &format!("rgb({})", hex)]).spawn();
}

pub fn set_active_opacity(val: f64) {
    let _ = Command::new("hyprctl").args(["keyword", "decoration:active_opacity", &format!("{:.2}", val)]).spawn();
}

pub fn set_inactive_opacity(val: f64) {
    let _ = Command::new("hyprctl").args(["keyword", "decoration:inactive_opacity", &format!("{:.2}", val)]).spawn();
}

pub fn set_blur_enabled(enabled: bool) {
    let state = if enabled { "true" } else { "false" };
    let _ = Command::new("hyprctl").args(["keyword", "decoration:blur:enabled", state]).spawn();
}

pub fn set_blur_size(val: i32) {
    let _ = Command::new("hyprctl").args(["keyword", "decoration:blur:size", &val.to_string()]).spawn();
}

pub fn set_shadow_enabled(enabled: bool) {
    let state = if enabled { "true" } else { "false" };
    let _ = Command::new("hyprctl").args(["keyword", "decoration:shadow:enabled", state]).spawn();
}

pub fn set_animations_enabled(enabled: bool) {
    let state = if enabled { "true" } else { "false" };
    let _ = Command::new("hyprctl").args(["keyword", "animations:enabled", state]).spawn();
}

pub fn set_wallpaper(path: &str) {
    // 1. swaybg is de meest betrouwbare en lichte Wayland/Hyprland wallpaper manager
    let _ = Command::new("killall")
        .args(["-q", "-9", "swaybg"])
        .status();

    let swaybg_spawned = Command::new("setsid")
        .args(["swaybg", "-i", path, "-m", "fill"])
        .spawn()
        .is_ok();

    // 2. Probeer daarnaast waypaper indien aanwezig
    let _ = Command::new("waypaper")
        .args(["--wallpaper", path])
        .spawn();

    // 3. Probeer hyprpaper met vereiste preload
    let _ = Command::new("hyprctl")
        .args(["hyprpaper", "preload", path])
        .status();
    let _ = Command::new("hyprctl")
        .args(["hyprpaper", "wallpaper", &format!(",{}", path)])
        .spawn();

    // 4. Probeer swww als fallback
    if !swaybg_spawned {
        let _ = Command::new("swww")
            .args(["img", path, "--transition-type", "grow"])
            .spawn();
    }
}

// ==========================================
// MONITOR CONTROLS
// ==========================================

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct MonitorInfo {
    pub id: i64,
    pub name: String,
    pub width: i32,
    pub height: i32,
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f64,
    pub scale: f64,
    pub x: i32,
    pub y: i32,
    #[serde(rename = "availableModes", default)]
    pub available_modes: Vec<String>,
}

/// Haalt alle aangesloten monitoren en ondersteunde modi op via hyprctl
pub fn get_monitors() -> Vec<MonitorInfo> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output();

    if let Ok(out) = output {
        if let Ok(text) = String::from_utf8(out.stdout) {
            if let Ok(monitors) = serde_json::from_str::<Vec<MonitorInfo>>(&text) {
                return monitors;
            }
        }
    }
    Vec::new()
}

/// Past de monitorconfiguratie direct live toe
pub fn apply_monitor_rule(name: &str, res_hz: &str, scale: f64) {
    let rule = format!("{},{},auto,{:.2}", name, res_hz, scale);
    let _ = Command::new("hyprctl")
        .args(["keyword", "monitor", &rule])
        .spawn();
}

pub fn set_monitor_mode(name: &str, mode: &str) {
    let monitors = get_monitors();
    let scale = monitors.iter().find(|m| m.name == name).map(|m| m.scale).unwrap_or(1.0);
    apply_monitor_rule(name, mode, scale);
}

pub fn set_monitor_scale(name: &str, scale: f64) {
    let monitors = get_monitors();
    let mode = monitors.iter().find(|m| m.name == name)
        .map(|m| format!("{}x{}@{:.2}Hz", m.width, m.height, m.refresh_rate))
        .unwrap_or_else(|| "preferred".to_string());
    apply_monitor_rule(name, &mode, scale);
}