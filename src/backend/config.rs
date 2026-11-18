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
    /// smooth | snappy | bouncy | slide, of "own" = Zenith laat de eigen animaties van de gebruiker staan
    pub anim_style: String,
    /// 0.5 = half zo snel, 2.0 = twee keer zo snel
    pub anim_speed: f64,
    pub inactive_border_color: String,
    pub layout: String,
    pub blur_passes: i32,
    pub dim_inactive: bool,
    /// Leeg = Zenith laat de toetsenbordindeling van hyprland.conf ongemoeid.
    pub kb_layout: String,
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
            anim_style: "smooth".to_string(),
            anim_speed: 1.0,
            inactive_border_color: "595959".to_string(),
            layout: "dwindle".to_string(),
            blur_passes: 1,
            dim_inactive: false,
            kb_layout: String::new(),
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
            else if let Some(v) = parse_str(l, "$zenith_anim_style = ") { cfg.anim_style = v; }
            else if let Some(v) = parse_f64(l, "$zenith_anim_speed = ") { cfg.anim_speed = v; }
            else if let Some(v) = parse_str(l, "$zenith_inactive_border_color = ") { cfg.inactive_border_color = v; }
            else if let Some(v) = parse_str(l, "$zenith_layout = ") { cfg.layout = v; }
            else if let Some(v) = parse_i32(l, "$zenith_blur_passes = ") { cfg.blur_passes = v; }
            else if let Some(v) = parse_bool(l, "$zenith_dim_inactive = ") { cfg.dim_inactive = v; }
            else if let Some(v) = parse_str(l, "kb_layout = ") { cfg.kb_layout = v; }
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

    // Lua-config zonder eigen zenith.lua: begin bij wat Hyprland nu echt gebruikt.
    if crate::backend::hyprland::uses_lua() && !get_config_path().map(|p| p.with_extension("lua").exists()).unwrap_or(true) {
        crate::backend::hyprland::import_live(&mut cfg);
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

/// Animatiestijlen voor Hyprland: (stijl-id, bezier-punten, basis-snelheid in ds, vensterstijl, werkbladstijl).
pub const ANIM_STYLES: [(&str, [f64; 4], f64, &str, &str); 4] = [
    ("smooth", [0.25, 1.0, 0.5, 1.0], 5.0, "popin 80%", "slide"),
    ("snappy", [0.2, 0.9, 0.1, 1.0], 2.5, "popin 90%", "slide"),
    ("bouncy", [0.34, 1.56, 0.64, 1.0], 5.0, "popin 60%", "slide"),
    ("slide", [0.16, 1.0, 0.3, 1.0], 4.5, "slide", "slidefade 20%"),
];

/// Bezier-punten en (naam, snelheid, stijl) per animatie voor de gekozen stijl en het tempo.
/// (animatie, snelheid, stijl)
pub type AnimEntry = (&'static str, String, Option<&'static str>);

pub fn animation_spec(cfg: &ZenithConfig) -> ([f64; 4], Vec<AnimEntry>) {
    let (_, b, base, win_style, ws_style) = ANIM_STYLES.iter().find(|s| s.0 == cfg.anim_style).copied().unwrap_or(ANIM_STYLES[0]);
    // Hyprland-snelheid is een duur in deciseconden: hoger tempo = kleiner getal.
    let speed = |mul: f64| format!("{:.2}", (base * mul / cfg.anim_speed.clamp(0.25, 4.0)).max(0.1));
    (b, vec![
        ("windows", speed(1.0), Some(win_style)),
        ("windowsOut", speed(0.8), Some("popin 80%")),
        ("fade", speed(0.8), None),
        ("border", speed(1.6), None),
        ("workspaces", speed(1.0), Some(ws_style)),
    ])
}

/// `bezier`/`animation`-regels (hyprlang) voor de gekozen stijl en het tempo.
pub fn animation_lines(cfg: &ZenithConfig) -> Vec<String> {
    if cfg.anim_style == "own" {
        return Vec::new();
    }
    let (b, anims) = animation_spec(cfg);
    let mut lines = vec![format!("bezier = zenith, {}, {}, {}, {}", b[0], b[1], b[2], b[3])];
    lines.extend(anims.into_iter().map(|(name, speed, style)| match style {
        Some(st) => format!("animation = {}, 1, {}, zenith, {}", name, speed, st),
        None => format!("animation = {}, 1, {}, zenith", name, speed),
    }));
    lines
}

/// Lua-stringliteral met escapes (waarden komen deels uit gebruikersinvoer).
pub fn lua_str(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " "))
}

/// Dezelfde instellingen als zenith.conf, maar voor de Lua-config van Hyprland >= 0.55.
pub fn lua_script(cfg: &ZenithConfig) -> String {
    let input = if cfg.kb_layout.is_empty() { String::new() } else { format!("\n  input = {{ kb_layout = {} }},", lua_str(&cfg.kb_layout)) };
    let mut out = format!(
        "-- Generated by Zenith Control: wordt overschreven, pas aan via de app.\n\
hl.config({{\n\
  general = {{ gaps_out = {}, gaps_in = {}, border_size = {}, layout = {}, col = {{ active_border = {}, inactive_border = {} }} }},\n\
  decoration = {{ rounding = {}, active_opacity = {:.2}, inactive_opacity = {:.2}, dim_inactive = {}, blur = {{ enabled = {}, size = {}, passes = {} }}, shadow = {{ enabled = {} }} }},\n\
  animations = {{ enabled = {} }},{}\n\
}})\n",
        cfg.gaps_out, cfg.gaps_in, cfg.border_size, lua_str(&cfg.layout),
        lua_str(&format!("rgb({})", cfg.active_border_color)), lua_str(&format!("rgb({})", cfg.inactive_border_color)),
        cfg.rounding, cfg.active_opacity, cfg.inactive_opacity, cfg.dim_inactive,
        cfg.blur_enabled, cfg.blur_size, cfg.blur_passes, cfg.shadow_enabled, cfg.animations_enabled, input,
    );
    if cfg.animations_enabled && cfg.anim_style != "own" {
        let (b, anims) = animation_spec(cfg);
        out.push_str(&format!("hl.curve(\"zenith\", {{ type = \"bezier\", points = {{ {{{}, {}}}, {{{}, {}}} }} }})\n", b[0], b[1], b[2], b[3]));
        for (name, speed, style) in anims {
            let style = style.map(|st| format!(", style = {}", lua_str(st))).unwrap_or_default();
            out.push_str(&format!("hl.animation({{ leaf = \"{}\", enabled = true, speed = {}, bezier = \"zenith\"{} }})\n", name, speed, style));
        }
    }
    for rule in &cfg.monitor_rules {
        let p: Vec<&str> = rule.split(',').map(str::trim).collect();
        if let [output, mode, position, scale] = p[..] {
            out.push_str(&format!("hl.monitor({{ output = {}, mode = {}, position = {}, scale = {} }})\n",
                lua_str(output), lua_str(mode), lua_str(position), scale.parse::<f64>().unwrap_or(1.0)));
        }
    }
    // Sneltoetsen bestaan pas nadat de gebruiker ze in Zenith opslaat.
    out.push_str("pcall(dofile, os.getenv(\"HOME\") .. \"/.config/hypr/zenith-keybinds.lua\")\n");
    let bar = match cfg.active_bar.as_str() { "quickshell" => Some("quickshell -d"), "waybar" => Some("waybar"), _ => None };
    if let Some(cmd) = bar {
        out.push_str(&format!("hl.on(\"hyprland.start\", function() hl.exec_cmd({}) end)\n", lua_str(cmd)));
    }
    out
}

/// Hyprland-presets voor wie niet aan elke schuif wil zitten. Kleuren, layout en
/// toetsenbord blijven staan: dat is smaak, geen preset.
pub const PRESETS: [&str; 4] = ["default", "minimal", "performance", "eyecandy"];

pub fn apply_preset(cfg: &mut ZenithConfig, preset: &str) {
    let d = ZenithConfig::default();
    // (gaps_out, gaps_in, border, rounding, active_op, inactive_op, blur, blur_size, passes, shadow, anim, dim)
    let p = match preset {
        "minimal" => (4, 2, 1, 4, 1.0, 1.0, false, d.blur_size, 1, false, true, false),
        "performance" => (2, 1, 1, 0, 1.0, 1.0, false, d.blur_size, 1, false, false, false),
        "eyecandy" => (14, 7, 2, 14, 0.95, 0.85, true, 8, 3, true, true, false),
        _ => (d.gaps_out, d.gaps_in, d.border_size, d.rounding, d.active_opacity, d.inactive_opacity,
              d.blur_enabled, d.blur_size, d.blur_passes, d.shadow_enabled, d.animations_enabled, d.dim_inactive),
    };
    (cfg.gaps_out, cfg.gaps_in, cfg.border_size, cfg.rounding, cfg.active_opacity, cfg.inactive_opacity,
     cfg.blur_enabled, cfg.blur_size, cfg.blur_passes, cfg.shadow_enabled, cfg.animations_enabled, cfg.dim_inactive) = p;
}

/// Vervangt (of voegt toe) de monitor-regel voor de monitor waar `rule` mee begint.
pub fn upsert_monitor_rule(cfg: &mut ZenithConfig, rule: String) {
    let name = rule.split(',').next().unwrap_or("").trim().to_string();
    cfg.monitor_rules.retain(|r| r.split(',').next().unwrap_or("").trim() != name);
    cfg.monitor_rules.push(rule);
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
        let tmp = path.with_extension("conf.tmp");
        if let Ok(mut f) = File::create(&tmp) {
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
                $zenith_anim_style = {}\n\
                $zenith_anim_speed = {:.2}\n\
                $zenith_inactive_border_color = {}\n\
                $zenith_layout = {}\n\
                $zenith_blur_passes = {}\n\
                $zenith_dim_inactive = {}\n\
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
                # Window Rules voor Zenith Control Center\n\
                windowrulev2 = float, class:^(org.zenith.control)$\n\
                windowrulev2 = size 1100 760, class:^(org.zenith.control)$\n\
                windowrulev2 = center, class:^(org.zenith.control)$\n\n\
                # Autostart actieve statusbalk beheerd door Zenith\n\
                {}\n\n\
                $terminal = {}\n\
                $menu = {}\n\n\
                general {{\n\
                    gaps_out = $zenith_gaps_out\n\
                    gaps_in = $zenith_gaps_in\n\
                    border_size = $zenith_border_size\n\
                    col.active_border = rgb($zenith_active_border_color)\n\
                    col.inactive_border = rgb($zenith_inactive_border_color)\n\
                    layout = $zenith_layout\n\
                }}\n\n\
                decoration {{\n\
                    rounding = $zenith_rounding\n\
                    active_opacity = $zenith_active_opacity\n\
                    inactive_opacity = $zenith_inactive_opacity\n\
                    dim_inactive = $zenith_dim_inactive\n\
                    shadow {{\n\
                        enabled = $zenith_shadow_enabled\n\
                    }}\n\
                    blur {{\n\
                        enabled = $zenith_blur_enabled\n\
                        size = $zenith_blur_size\n\
                        passes = $zenith_blur_passes\n\
                    }}\n\
                }}\n\n\
                animations {{\n\
                    enabled = $zenith_animations_enabled\n\
                    {}\n\
                }}\n\
                {}",
                monitor_lines,
                cfg.gaps_out, cfg.gaps_in, cfg.border_size, cfg.rounding, cfg.active_border_color,
                cfg.active_opacity, cfg.inactive_opacity, cfg.blur_enabled, cfg.blur_size,
                cfg.shadow_enabled, cfg.animations_enabled, cfg.anim_style, cfg.anim_speed,
                cfg.inactive_border_color, cfg.layout, cfg.blur_passes, cfg.dim_inactive,
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
                match cfg.active_bar.as_str() {
                    "quickshell" => "exec-once = quickshell -d",
                    "waybar" => "exec-once = waybar",
                    _ => "# Bar autostart uitgeschakeld",
                },
                cfg.default_terminal, launcher_cmd,
                animation_lines(cfg).join("\n    "),
                if cfg.kb_layout.is_empty() { String::new() } else { format!("\ninput {{\n    kb_layout = {}\n}}\n", cfg.kb_layout) }
            );
            if f.write_all(content.as_bytes()).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
            let lua = path.with_extension("lua");
            let lua_tmp = path.with_extension("lua.tmp");
            if std::fs::write(&lua_tmp, lua_script(cfg)).is_ok() {
                let _ = std::fs::rename(&lua_tmp, &lua);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_roundtrips_through_zenith_conf() {
        let _guard = crate::backend::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = std::path::PathBuf::from("/tmp/zenith-preset-test-home");
        let _ = std::fs::remove_dir_all(&home);
        std::env::set_var("HOME", &home);

        let mut cfg = ZenithConfig::default();
        apply_preset(&mut cfg, "eyecandy");
        cfg.inactive_border_color = "112233".into();
        cfg.layout = "master".into();
        cfg.dim_inactive = true;
        cfg.kb_layout = "be".into();
        save_config(&cfg);
        let back = load_config();
        assert_eq!((back.gaps_out, back.rounding, back.blur_passes), (14, 14, 3));
        assert_eq!(back.inactive_border_color, "112233");
        assert_eq!(back.layout, "master");
        assert!(back.dim_inactive);
        assert_eq!(back.kb_layout, "be");

        // Lege kb_layout schrijft geen input-blok (laat hyprland.conf ongemoeid).
        cfg.kb_layout.clear();
        save_config(&cfg);
        let text = std::fs::read_to_string(home.join(".config/hypr/zenith.conf")).unwrap();
        assert!(!text.contains("kb_layout"), "{text}");

        apply_preset(&mut cfg, "default");
        assert_eq!(cfg.gaps_out, ZenithConfig::default().gaps_out);
        assert_eq!(cfg.layout, "master", "preset laat layout staan");
        std::env::remove_var("HOME");
    }

    #[test]
    fn animation_lines_follow_style_and_speed() {
        let mut cfg = ZenithConfig { anim_style: "bouncy".into(), ..Default::default() };
        let lines = animation_lines(&cfg);
        assert_eq!(lines[0], "bezier = zenith, 0.34, 1.56, 0.64, 1");
        assert_eq!(lines[1], "animation = windows, 1, 5.00, zenith, popin 60%");
        cfg.anim_speed = 2.0; // twee keer zo snel = halve duur
        assert_eq!(animation_lines(&cfg)[1], "animation = windows, 1, 2.50, zenith, popin 60%");
        cfg.anim_style = "onbekend".into(); // valt terug op smooth
        assert!(animation_lines(&cfg)[0].starts_with("bezier = zenith, 0.25"));
    }

    #[test]
    fn lua_script_is_valid_shape_and_escapes() {
        let mut cfg = ZenithConfig { kb_layout: "us\",os.exit()--".into(), active_bar: "quickshell".into(), ..Default::default() };
        cfg.monitor_rules = vec!["DP-1,2560x1440@144.00Hz,auto,1.25".into()];
        let lua = lua_script(&cfg);
        assert!(lua.contains("kb_layout = \"us\\\",os.exit()--\""), "{lua}");
        assert!(lua.contains("col = { active_border = \"rgb(33ccff)\""));
        assert!(lua.contains("hl.curve(\"zenith\", { type = \"bezier\", points = { {0.25, 1}, {0.5, 1} } })"));
        assert!(lua.contains("hl.animation({ leaf = \"windows\", enabled = true, speed = 5.00, bezier = \"zenith\", style = \"popin 80%\" })"));
        assert!(lua.contains("hl.monitor({ output = \"DP-1\", mode = \"2560x1440@144.00Hz\", position = \"auto\", scale = 1.25 })"));
        assert!(lua.contains("hl.exec_cmd(\"quickshell -d\")"));
        cfg.animations_enabled = false;
        assert!(!lua_script(&cfg).contains("hl.animation"));
        cfg.animations_enabled = true;
        cfg.anim_style = "own".into(); // eigen animaties van de gebruiker blijven staan
        assert!(!lua_script(&cfg).contains("hl.curve"));
        assert!(animation_lines(&cfg).is_empty());
    }

    #[test]
    fn upsert_monitor_rule_replaces_same_monitor() {
        let mut cfg = ZenithConfig::default();
        upsert_monitor_rule(&mut cfg, "DP-1,1920x1080@60.00Hz,auto,1.00".into());
        upsert_monitor_rule(&mut cfg, "HDMI-A-1,preferred,auto,1.00".into());
        upsert_monitor_rule(&mut cfg, "DP-1,2560x1440@144.00Hz,auto,1.25".into());
        assert_eq!(cfg.monitor_rules, vec![
            "HDMI-A-1,preferred,auto,1.00".to_string(),
            "DP-1,2560x1440@144.00Hz,auto,1.25".to_string(),
        ]);
    }
}
