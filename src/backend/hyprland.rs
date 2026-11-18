use serde::Deserialize;
use std::process::Command;

/// Hyprland >= 0.55 met `hyprland.lua` accepteert geen `hyprctl keyword` meer, alleen `eval`.
pub fn uses_lua() -> bool {
    std::env::var("HOME").map(|h| std::path::Path::new(&h).join(".config/hypr/hyprland.lua").exists()).unwrap_or(false)
}

fn eval_lua(script: &str) {
    let _ = Command::new("hyprctl").args(["eval", script]).spawn();
}

/// "decoration:blur:size" + 8 → `hl.config({ decoration = { blur = { size = 8 } } })`.
fn lua_config(key: &str, value: &str) -> String {
    let lit = if value == "true" || value == "false" || value.parse::<f64>().is_ok() {
        value.to_string()
    } else {
        crate::backend::config::lua_str(value)
    };
    let nested = key.split([':', '.']).rev().fold(lit, |inner, part| format!("{{ {} = {} }}", part, inner));
    format!("hl.config({})", nested)
}

/// Zet één Hyprland-optie live (keyword voor hyprlang, eval voor Lua-configs).
pub fn set_keyword(key: &str, value: impl std::fmt::Display) {
    let value = value.to_string();
    if uses_lua() {
        eval_lua(&lua_config(key, &value));
    } else {
        let _ = Command::new("hyprctl").args(["keyword", key, &value]).spawn();
    }
}

/// Leest de actuele waarden uit Hyprland (voor Lua-gebruikers zonder zenith.lua).
pub fn import_live(cfg: &mut crate::backend::config::ZenithConfig) {
    let get = |opt: &str| -> Option<serde_json::Value> {
        let out = Command::new("hyprctl").args(["getoption", opt, "-j"]).output().ok()?;
        serde_json::from_slice(&out.stdout).ok()
    };
    let int = |opt: &str| get(opt).and_then(|v| v.get("int").and_then(|n| n.as_i64())
        .or_else(|| v.get("css").and_then(|c| c.as_str()?.split_whitespace().next()?.parse().ok()))).map(|n| n as i32);
    let float = |opt: &str| get(opt).and_then(|v| v.get("float")?.as_f64());
    let boolean = |opt: &str| get(opt).and_then(|v| v.get("bool")?.as_bool().or_else(|| v.get("int")?.as_i64().map(|n| n != 0)));
    let color = |opt: &str| get(opt).and_then(|v| gradient_to_hex(v.get("gradient")?.as_str()?));

    if let Some(v) = int("general:gaps_out") { cfg.gaps_out = v; }
    if let Some(v) = int("general:gaps_in") { cfg.gaps_in = v; }
    if let Some(v) = int("general:border_size") { cfg.border_size = v; }
    if let Some(v) = int("decoration:rounding") { cfg.rounding = v; }
    if let Some(v) = float("decoration:active_opacity") { cfg.active_opacity = v; }
    if let Some(v) = float("decoration:inactive_opacity") { cfg.inactive_opacity = v; }
    if let Some(v) = boolean("decoration:dim_inactive") { cfg.dim_inactive = v; }
    if let Some(v) = boolean("decoration:blur:enabled") { cfg.blur_enabled = v; }
    if let Some(v) = int("decoration:blur:size") { cfg.blur_size = v; }
    if let Some(v) = int("decoration:blur:passes") { cfg.blur_passes = v; }
    if let Some(v) = boolean("decoration:shadow:enabled") { cfg.shadow_enabled = v; }
    if let Some(v) = boolean("animations:enabled") { cfg.animations_enabled = v; }
    cfg.anim_style = "own".to_string(); // eigen curves/animaties van de gebruiker niet overschrijven
    if let Some(v) = get("general:layout").and_then(|v| v.get("str")?.as_str().map(String::from)) { cfg.layout = v; }
    if let Some(v) = color("general:col.active_border") { cfg.active_border_color = v; }
    if let Some(v) = color("general:col.inactive_border") { cfg.inactive_border_color = v; }
}

/// "ee33ccff 0deg" (aarrggbb) → "33ccff".
fn gradient_to_hex(g: &str) -> Option<String> {
    let first = g.split_whitespace().next()?;
    (first.len() == 8 && first.chars().all(|c| c.is_ascii_hexdigit())).then(|| first[2..].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lua_config_nests_keys_and_quotes_strings() {
        assert_eq!(lua_config("decoration:blur:size", "8"), "hl.config({ decoration = { blur = { size = 8 } } })");
        assert_eq!(lua_config("general:col.active_border", "rgb(33ccff)"), "hl.config({ general = { col = { active_border = \"rgb(33ccff)\" } } })");
        assert_eq!(lua_config("decoration:dim_inactive", "true"), "hl.config({ decoration = { dim_inactive = true } })");
        assert_eq!(lua_config("input:kb_layout", "us\"x"), "hl.config({ input = { kb_layout = \"us\\\"x\" } })");
    }

    #[test]
    fn gradient_to_hex_parses_hyprland_colors() {
        assert_eq!(gradient_to_hex("ee33ccff 0deg").as_deref(), Some("33ccff"));
        assert_eq!(gradient_to_hex("ffffffff 0deg").as_deref(), Some("ffffff"));
        assert_eq!(gradient_to_hex("bogus"), None);
    }
}

/// Past alleen de animaties live toe (stijl- of snelheidswijziging).
pub fn apply_animations(cfg: &crate::backend::config::ZenithConfig) {
    if uses_lua() {
        return eval_lua(&crate::backend::config::lua_script(cfg));
    }
    let mut batch = vec![format!("keyword animations:enabled {}", cfg.animations_enabled)];
    if cfg.animations_enabled {
        batch.extend(crate::backend::config::animation_lines(cfg).iter().map(|l| format!("keyword {}", l.replacen(" = ", " ", 1))));
    }
    let _ = Command::new("hyprctl").args(["--batch", &batch.join(" ; ")]).spawn();
}

/// Past alle Hyprland-velden uit de config in één keer live toe (presets).
pub fn apply_all(cfg: &crate::backend::config::ZenithConfig) {
    if uses_lua() {
        return eval_lua(&crate::backend::config::lua_script(cfg));
    }
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
    if cfg.animations_enabled {
        batch.extend(crate::backend::config::animation_lines(cfg).iter().map(|l| format!("keyword {}", l.replacen(" = ", " ", 1))));
    }
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