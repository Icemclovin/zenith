use serde::Deserialize;
use std::process::Command;

/// Zet één Hyprland-optie live via `hyprctl keyword`.
pub fn set_keyword(key: &str, value: impl std::fmt::Display) {
    let _ = Command::new("hyprctl").args(["keyword", key, &value.to_string()]).spawn();
}

/// Past alle Hyprland-velden uit de config in één keer live toe (presets).
pub fn apply_all(cfg: &crate::backend::config::ZenithConfig) {
    let mut batch = vec![
        format!("keyword general:gaps_out {}", cfg.gaps_out),
        format!("keyword general:gaps_in {}", cfg.gaps_in),
        format!("keyword general:border_size {}", cfg.border_size),
        format!("keyword general:col.active_border rgb({})", cfg.active_border_color),
        format!("keyword general:col.inactive_border rgb({})", cfg.inactive_border_color),
        format!("keyword general:layout {}", cfg.layout),
        format!("keyword decoration:rounding {}", cfg.rounding),
        format!("keyword decoration:active_opacity {:.2}", cfg.active_opacity),
        format!("keyword decoration:inactive_opacity {:.2}", cfg.inactive_opacity),
        format!("keyword decoration:dim_inactive {}", cfg.dim_inactive),
        format!("keyword decoration:blur:enabled {}", cfg.blur_enabled),
        format!("keyword decoration:blur:size {}", cfg.blur_size),
        format!("keyword decoration:blur:passes {}", cfg.blur_passes),
        format!("keyword decoration:shadow:enabled {}", cfg.shadow_enabled),
        format!("keyword animations:enabled {}", cfg.animations_enabled),
    ];
    if !cfg.kb_layout.is_empty() {
        batch.push(format!("keyword input:kb_layout {}", cfg.kb_layout));
    }
    let _ = Command::new("hyprctl").args(["--batch", &batch.join(" ; ")]).spawn();
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

/// Past de monitorconfiguratie direct live toe en geeft de regel terug
/// zodat de aanroeper hem in zenith.conf kan bewaren.
pub fn apply_monitor_rule(name: &str, res_hz: &str, scale: f64) -> String {
    let rule = format!("{},{},auto,{:.2}", name, res_hz, scale);
    let _ = Command::new("hyprctl")
        .args(["keyword", "monitor", &rule])
        .spawn();
    rule
}

pub fn set_monitor_mode(name: &str, mode: &str) -> String {
    let monitors = get_monitors();
    let scale = monitors.iter().find(|m| m.name == name).map(|m| m.scale).unwrap_or(1.0);
    apply_monitor_rule(name, mode, scale)
}

pub fn set_monitor_scale(name: &str, scale: f64) -> String {
    let monitors = get_monitors();
    let mode = monitors.iter().find(|m| m.name == name)
        .map(|m| format!("{}x{}@{:.2}Hz", m.width, m.height, m.refresh_rate))
        .unwrap_or_else(|| "preferred".to_string());
    apply_monitor_rule(name, &mode, scale)
}