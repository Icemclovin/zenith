use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ZenithConfig {
    pub gaps_out: i32,
    pub gaps_in: i32,
    pub border_size: i32,
    pub rounding: i32,
    pub active_border_color: String,
    pub active_opacity: f64,
    pub inactive_opacity: f64,
    pub blur_enabled: bool,
    pub blur_size: i32,
    pub shadow_enabled: bool,
    pub animations_enabled: bool,
    // Waybar
    pub waybar_position: String,
    pub waybar_height: i32,
    pub waybar_bg_color: String,
    pub waybar_opacity: f64,
    pub waybar_rounding: i32,
    // Defaults
    pub active_bar: String,
    pub default_launcher: String,
    pub default_terminal: String,
    pub default_shell: String,
    // Kitty
    pub kitty_bg: String,
    pub kitty_fg: String,
    pub kitty_opacity: f64,
    pub kitty_font_size: f64,
    // Rofi
    pub rofi_bg: String,
    pub rofi_fg: String,
    pub rofi_opacity: f64,
    pub rofi_rounding: i32,
    // Quickshell
    pub qs_bg: String,
    pub qs_accent: String,
    pub qs_opacity: f64,
    pub qs_height: i32,
    pub qs_position: String,
    pub qs_rounding: i32,
    pub qs_margin_h: i32,
    pub qs_margin_v: i32,
    pub qs_border_width: i32,
    pub qs_border_color: String,
    pub qs_text_color: String,
    pub qs_font_size: i32,
    pub qs_floating: bool,
    pub qs_bar_style: String,
    pub qs_pill_bg: String,
    pub qs_pill_opacity: f64,
    pub qs_pill_rounding: i32,
    pub qs_module_spacing: i32,
    pub qs_brand_text: String,
    pub qs_show_brand: bool,
    pub qs_status_text: String,
    pub qs_show_status_badge: bool,
    pub qs_show_workspaces: bool,
    pub qs_show_launcher_btn: bool,
    pub qs_show_cpu: bool,
    pub qs_show_ram: bool,
    pub qs_show_battery: bool,
    pub qs_show_volume: bool,
    pub qs_show_bluetooth: bool,
    pub qs_show_network: bool,
    pub qs_show_media: bool,
    pub qs_show_clock: bool,
    pub qs_show_seconds: bool,
    pub qs_show_power_btn: bool,
    // Monitors (Hyprland display rules)
    pub monitor_rules: Vec<String>,
}

impl Default for ZenithConfig {
    fn default() -> Self {
        Self {
            gaps_out: 10,
            gaps_in: 5,
            border_size: 2,
            rounding: 8,
            active_border_color: "33ccff".to_string(),
            active_opacity: 1.0,
            inactive_opacity: 0.9,
            blur_enabled: true,
            blur_size: 6,
            shadow_enabled: true,
            animations_enabled: true,
            waybar_position: "top".to_string(),
            waybar_height: 32,
            waybar_bg_color: "1e1e2e".to_string(),
            waybar_opacity: 0.90,
            waybar_rounding: 10,
            active_bar: "waybar".to_string(),
            default_launcher: "rofi".to_string(),
            default_terminal: "kitty".to_string(),
            default_shell: "zsh".to_string(),
            kitty_bg: "1e1e2e".to_string(),
            kitty_fg: "cdd6f4".to_string(),
            kitty_opacity: 0.95,
            kitty_font_size: 11.5,
            rofi_bg: "1e1e2e".to_string(),
            rofi_fg: "cdd6f4".to_string(),
            rofi_opacity: 0.95,
            rofi_rounding: 12,
            qs_bg: "1e1e2e".to_string(),
            qs_accent: "89b4fa".to_string(),
            qs_opacity: 0.90,
            qs_height: 38,
            qs_position: "top".to_string(),
            qs_rounding: 12,
            qs_margin_h: 8,
            qs_margin_v: 6,
            qs_border_width: 1,
            qs_border_color: "45475a".to_string(),
            qs_text_color: "cdd6f4".to_string(),
            qs_font_size: 11,
            qs_floating: true,
            qs_bar_style: "unified".to_string(),
            qs_pill_bg: "181825".to_string(),
            qs_pill_opacity: 0.85,
            qs_pill_rounding: 8,
            qs_module_spacing: 12,
            qs_brand_text: "Zenith".to_string(),
            qs_show_brand: true,
            qs_status_text: "Hyprland".to_string(),
            qs_show_status_badge: true,
            qs_show_workspaces: true,
            qs_show_launcher_btn: true,
            qs_show_cpu: true,
            qs_show_ram: true,
            qs_show_battery: true,
            qs_show_volume: true,
            qs_show_bluetooth: true,
            qs_show_network: true,
            qs_show_media: true,
            qs_show_clock: true,
            qs_show_seconds: false,
            qs_show_power_btn: true,
            monitor_rules: Vec::new(),
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/hypr/zenith.conf"))
}

pub fn load_config() -> ZenithConfig {
    let mut cfg = ZenithConfig::default();
    let path = match get_config_path() {
        Some(p) => p,
        None => return cfg,
    };

    if let Ok(content) = read_to_string(path) {
        for line in content.lines() {
            let l = line.trim();
            if let Some(rule) = l.strip_prefix("monitor = ") {
                cfg.monitor_rules.push(rule.trim().to_string());
            } else if let Some(v) = parse_i32(l, "$zenith_gaps_out = ") { cfg.gaps_out = v; }
            else if let Some(v) = parse_i32(l, "$zenith_gaps_in = ") { cfg.gaps_in = v; }
            else if let Some(v) = parse_i32(l, "$zenith_border_size = ") { cfg.border_size = v; }
            else if let Some(v) = parse_i32(l, "$zenith_rounding = ") { cfg.rounding = v; }
            else if let Some(v) = parse_str(l, "$zenith_active_border_color = ") { cfg.active_border_color = v; }
            else if let Some(v) = parse_f64(l, "$zenith_active_opacity = ") { cfg.active_opacity = v; }
            else if let Some(v) = parse_f64(l, "$zenith_inactive_opacity = ") { cfg.inactive_opacity = v; }
            else if let Some(v) = parse_bool(l, "$zenith_blur_enabled = ") { cfg.blur_enabled = v; }
            else if let Some(v) = parse_i32(l, "$zenith_blur_size = ") { cfg.blur_size = v; }
            else if let Some(v) = parse_bool(l, "$zenith_shadow_enabled = ") { cfg.shadow_enabled = v; }
            else if let Some(v) = parse_bool(l, "$zenith_animations_enabled = ") { cfg.animations_enabled = v; }
            else if let Some(v) = parse_str(l, "$zenith_wb_pos = ") { cfg.waybar_position = v; }
            else if let Some(v) = parse_i32(l, "$zenith_wb_height = ") { cfg.waybar_height = v; }
            else if let Some(v) = parse_str(l, "$zenith_wb_bg = ") { cfg.waybar_bg_color = v; }
            else if let Some(v) = parse_f64(l, "$zenith_wb_opacity = ") { cfg.waybar_opacity = v; }
            else if let Some(v) = parse_i32(l, "$zenith_wb_round = ") { cfg.waybar_rounding = v; }
            else if let Some(v) = parse_str(l, "$zenith_active_bar = ") { cfg.active_bar = v; }
            else if let Some(v) = parse_str(l, "$zenith_launcher = ") { cfg.default_launcher = v; }
            else if let Some(v) = parse_str(l, "$zenith_terminal = ") { cfg.default_terminal = v; }
            else if let Some(v) = parse_str(l, "$zenith_shell = ") { cfg.default_shell = v; }
            else if let Some(v) = parse_str(l, "$zenith_kitty_bg = ") { cfg.kitty_bg = v; }
            else if let Some(v) = parse_str(l, "$zenith_kitty_fg = ") { cfg.kitty_fg = v; }
            else if let Some(v) = parse_f64(l, "$zenith_kitty_opacity = ") { cfg.kitty_opacity = v; }
            else if let Some(v) = parse_f64(l, "$zenith_kitty_font = ") { cfg.kitty_font_size = v; }
            else if let Some(v) = parse_str(l, "$zenith_rofi_bg = ") { cfg.rofi_bg = v; }
            else if let Some(v) = parse_str(l, "$zenith_rofi_fg = ") { cfg.rofi_fg = v; }
            else if let Some(v) = parse_f64(l, "$zenith_rofi_opacity = ") { cfg.rofi_opacity = v; }
            else if let Some(v) = parse_i32(l, "$zenith_rofi_round = ") { cfg.rofi_rounding = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_bg = ") { cfg.qs_bg = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_accent = ") { cfg.qs_accent = v; }
            else if let Some(v) = parse_f64(l, "$zenith_qs_opacity = ") { cfg.qs_opacity = v; }
            else if let Some(v) = parse_i32(l, "$zenith_qs_height = ") { cfg.qs_height = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_position = ") { cfg.qs_position = v; }
            else if let Some(v) = parse_i32(l, "$zenith_qs_rounding = ") { cfg.qs_rounding = v; }
            else if let Some(v) = parse_i32(l, "$zenith_qs_margin_h = ") { cfg.qs_margin_h = v; }
            else if let Some(v) = parse_i32(l, "$zenith_qs_margin_v = ") { cfg.qs_margin_v = v; }
            else if let Some(v) = parse_i32(l, "$zenith_qs_border_width = ") { cfg.qs_border_width = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_border_color = ") { cfg.qs_border_color = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_text_color = ") { cfg.qs_text_color = v; }
            else if let Some(v) = parse_i32(l, "$zenith_qs_font_size = ") { cfg.qs_font_size = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_floating = ") { cfg.qs_floating = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_workspaces = ") { cfg.qs_show_workspaces = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_cpu = ") { cfg.qs_show_cpu = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_battery = ") { cfg.qs_show_battery = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_volume = ") { cfg.qs_show_volume = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_bluetooth = ") { cfg.qs_show_bluetooth = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_network = ") { cfg.qs_show_network = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_clock = ") { cfg.qs_show_clock = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_seconds = ") { cfg.qs_show_seconds = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_bar_style = ") { cfg.qs_bar_style = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_pill_bg = ") { cfg.qs_pill_bg = v; }
            else if let Some(v) = parse_f64(l, "$zenith_qs_pill_opacity = ") { cfg.qs_pill_opacity = v; }
            else if let Some(v) = parse_i32(l, "$zenith_qs_pill_rounding = ") { cfg.qs_pill_rounding = v; }
            else if let Some(v) = parse_i32(l, "$zenith_qs_module_spacing = ") { cfg.qs_module_spacing = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_brand_text = ") { cfg.qs_brand_text = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_brand = ") { cfg.qs_show_brand = v; }
            else if let Some(v) = parse_str(l, "$zenith_qs_status_text = ") { cfg.qs_status_text = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_status_badge = ") { cfg.qs_show_status_badge = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_launcher_btn = ") { cfg.qs_show_launcher_btn = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_ram = ") { cfg.qs_show_ram = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_media = ") { cfg.qs_show_media = v; }
            else if let Some(v) = parse_bool(l, "$zenith_qs_show_power_btn = ") { cfg.qs_show_power_btn = v; }
        }
    }

    // Dedupliceer monitor-regels: bewaar alleen de laatste regel per monitor naam
    let mut seen = std::collections::HashSet::new();
    cfg.monitor_rules = cfg.monitor_rules
        .into_iter()
        .rev()
        .filter(|rule| {
            let mon_name = rule.split(',').next().unwrap_or("").trim().to_string();
            seen.insert(mon_name)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    cfg
}

fn parse_i32(line: &str, prefix: &str) -> Option<i32> {
    line.strip_prefix(prefix).and_then(|s| s.trim().parse::<i32>().ok())
}
fn parse_f64(line: &str, prefix: &str) -> Option<f64> {
    line.strip_prefix(prefix).and_then(|s| s.trim().parse::<f64>().ok())
}
fn parse_str(line: &str, prefix: &str) -> Option<String> {
    line.strip_prefix(prefix).map(|s| s.trim().to_string())
}
fn parse_bool(line: &str, prefix: &str) -> Option<bool> {
    line.strip_prefix(prefix).and_then(|s| s.trim().parse::<bool>().ok())
}

pub fn save_config(cfg: &ZenithConfig) {
    if let Some(path) = get_config_path() {
        if let Some(parent) = path.parent() { let _ = create_dir_all(parent); }
        if let Ok(mut f) = File::create(path) {
            let launcher_cmd = if cfg.default_launcher == "wofi" { "wofi --show drun" } else { "rofi -show drun" };

            let mut monitor_lines = String::new();
            for rule in &cfg.monitor_rules {
                monitor_lines.push_str(&format!("monitor = {}\n", rule));
            }

            let content = format!(
                "# Generated by Zenith Control\n\
                {}\n\
                $zenith_gaps_out = {}\n\
                $zenith_gaps_in = {}\n\
                $zenith_border_size = {}\n\
                $zenith_rounding = {}\n\
                $zenith_active_border_color = {}\n\
                $zenith_active_opacity = {:.2}\n\
                $zenith_inactive_opacity = {:.2}\n\
                $zenith_blur_enabled = {}\n\
                $zenith_blur_size = {}\n\
                $zenith_shadow_enabled = {}\n\
                $zenith_animations_enabled = {}\n\
                $zenith_wb_pos = {}\n\
                $zenith_wb_height = {}\n\
                $zenith_wb_bg = {}\n\
                $zenith_wb_opacity = {:.2}\n\
                $zenith_wb_round = {}\n\
                $zenith_active_bar = {}\n\
                $zenith_launcher = {}\n\
                $zenith_terminal = {}\n\
                $zenith_shell = {}\n\
                $zenith_kitty_bg = {}\n\
                $zenith_kitty_fg = {}\n\
                $zenith_kitty_opacity = {:.2}\n\
                $zenith_kitty_font = {:.1}\n\
                $zenith_rofi_bg = {}\n\
                $zenith_rofi_fg = {}\n\
                $zenith_rofi_opacity = {:.2}\n\
                $zenith_rofi_round = {}\n\
                $zenith_qs_bg = {}\n\
                $zenith_qs_accent = {}\n\
                $zenith_qs_opacity = {:.2}\n\
                $zenith_qs_height = {}\n\
                $zenith_qs_position = {}\n\
                $zenith_qs_rounding = {}\n\
                $zenith_qs_margin_h = {}\n\
                $zenith_qs_margin_v = {}\n\
                $zenith_qs_border_width = {}\n\
                $zenith_qs_border_color = {}\n\
                $zenith_qs_text_color = {}\n\
                $zenith_qs_font_size = {}\n\
                $zenith_qs_floating = {}\n\
                $zenith_qs_show_workspaces = {}\n\
                $zenith_qs_show_cpu = {}\n\
                $zenith_qs_show_battery = {}\n\
                $zenith_qs_show_volume = {}\n\
                $zenith_qs_show_bluetooth = {}\n\
                $zenith_qs_show_network = {}\n\
                $zenith_qs_show_clock = {}\n\
                $zenith_qs_show_seconds = {}\n\
                $zenith_qs_bar_style = {}\n\
                $zenith_qs_pill_bg = {}\n\
                $zenith_qs_pill_opacity = {:.2}\n\
                $zenith_qs_pill_rounding = {}\n\
                $zenith_qs_module_spacing = {}\n\
                $zenith_qs_brand_text = {}\n\
                $zenith_qs_show_brand = {}\n\
                $zenith_qs_status_text = {}\n\
                $zenith_qs_show_status_badge = {}\n\
                $zenith_qs_show_launcher_btn = {}\n\
                $zenith_qs_show_ram = {}\n\
                $zenith_qs_show_media = {}\n\
                $zenith_qs_show_power_btn = {}\n\n\
                $terminal = {}\n\
                $menu = {}\n\n\
                general {{\n\
                    gaps_out = $zenith_gaps_out\n\
                    gaps_in = $zenith_gaps_in\n\
                    border_size = $zenith_border_size\n\
                    col.active_border = rgb($zenith_active_border_color)\n\
                }}\n\n\
                decoration {{\n\
                    rounding = $zenith_rounding\n\
                    active_opacity = $zenith_active_opacity\n\
                    inactive_opacity = $zenith_inactive_opacity\n\
                    shadow {{\n\
                        enabled = $zenith_shadow_enabled\n\
                    }}\n\
                    blur {{\n\
                        enabled = $zenith_blur_enabled\n\
                        size = $zenith_blur_size\n\
                    }}\n\
                }}\n\n\
                animations {{\n\
                    enabled = $zenith_animations_enabled\n\
                }}\n",
                monitor_lines,
                cfg.gaps_out, cfg.gaps_in, cfg.border_size, cfg.rounding, cfg.active_border_color,
                cfg.active_opacity, cfg.inactive_opacity, cfg.blur_enabled, cfg.blur_size,
                cfg.shadow_enabled, cfg.animations_enabled,
                cfg.waybar_position, cfg.waybar_height, cfg.waybar_bg_color,
                cfg.waybar_opacity, cfg.waybar_rounding,
                cfg.active_bar, cfg.default_launcher, cfg.default_terminal, cfg.default_shell,
                cfg.kitty_bg, cfg.kitty_fg, cfg.kitty_opacity, cfg.kitty_font_size,
                cfg.rofi_bg, cfg.rofi_fg, cfg.rofi_opacity, cfg.rofi_rounding,
                cfg.qs_bg, cfg.qs_accent, cfg.qs_opacity, cfg.qs_height, cfg.qs_position,
                cfg.qs_rounding, cfg.qs_margin_h, cfg.qs_margin_v, cfg.qs_border_width,
                cfg.qs_border_color, cfg.qs_text_color, cfg.qs_font_size, cfg.qs_floating,
                cfg.qs_show_workspaces, cfg.qs_show_cpu, cfg.qs_show_battery, cfg.qs_show_volume,
                cfg.qs_show_bluetooth, cfg.qs_show_network, cfg.qs_show_clock, cfg.qs_show_seconds,
                cfg.qs_bar_style, cfg.qs_pill_bg, cfg.qs_pill_opacity, cfg.qs_pill_rounding,
                cfg.qs_module_spacing, cfg.qs_brand_text, cfg.qs_show_brand, cfg.qs_status_text,
                cfg.qs_show_status_badge, cfg.qs_show_launcher_btn, cfg.qs_show_ram, cfg.qs_show_media,
                cfg.qs_show_power_btn,
                cfg.default_terminal, launcher_cmd
            );
            let _ = f.write_all(content.as_bytes());
        }
    }
}