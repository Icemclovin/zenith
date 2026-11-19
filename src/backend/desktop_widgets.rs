//! Desktopwidgets (klok, systeem, media, weer) op de achtergrondlaag van Quickshell.
//! Eigen bestand naast zenith-shell.json: andere pagina's bewaren een kopie van die config
//! en zouden widgetposities anders overschrijven bij hun volgende save.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Widgettypes met hun grootte in px op schaal 1.0 (gelijk aan DesktopWidgets.qml).
pub const KINDS: [(&str, &str, i32, i32); 4] = [
    ("clock", "Klok", 320, 140),
    ("system", "Systeem", 360, 140),
    ("media", "Media", 380, 130),
    ("weather", "Weer", 260, 130),
];

pub fn kind_info(kind: &str) -> (&'static str, i32, i32) {
    KINDS.iter().find(|k| k.0 == kind).map(|k| (k.1, k.2, k.3)).unwrap_or(("?", 200, 100))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct DesktopWidget {
    pub kind: String,
    /// Linkerbovenhoek als fractie (0..1) van de schermbreedte/-hoogte.
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

impl Default for DesktopWidget {
    fn default() -> Self {
        Self { kind: "clock".into(), x: 0.05, y: 0.08, scale: 1.0 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct DesktopWidgetsConfig {
    pub enabled: bool,
    pub grid_snap: bool,
    /// Plaats voor het weer (bv. "Amsterdam"); leeg = wttr.in bepaalt de plek via je IP.
    pub weather_location: String,
    pub items: Vec<DesktopWidget>,
}

impl Default for DesktopWidgetsConfig {
    fn default() -> Self {
        Self { enabled: true, grid_snap: true, weather_location: String::new(), items: Vec::new() }
    }
}

/// Raster van 2,5% van het scherm: fijn genoeg om te plaatsen, grof genoeg om uit te lijnen.
pub const GRID: f64 = 0.025;

/// Klem een positie binnen het scherm (widget blijft volledig zichtbaar) en snap optioneel.
pub fn place(x: f64, y: f64, w_frac: f64, h_frac: f64, snap: bool) -> (f64, f64) {
    let s = |v: f64| if snap { (v / GRID).round() * GRID } else { v };
    let clamp = |v: f64, size: f64| v.clamp(0.0, (1.0 - size).max(0.0));
    (clamp(s(x), w_frac), clamp(s(y), h_frac))
}

fn path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var("HOME").ok()?).join(".config/quickshell/zenith-desktop.json"))
}

impl DesktopWidgetsConfig {
    pub fn load() -> Self {
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let p = path().ok_or_else(|| std::io::Error::other("HOME niet gezet"))?;
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        // Atomisch: Quickshell leest het bestand live in (FileView watchChanges).
        let tmp = p.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(tmp, p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_snaps_and_keeps_widget_on_screen() {
        assert_eq!(place(0.112, 0.0, 0.1, 0.1, true), (0.1, 0.0));
        let (x, y) = place(0.97, -0.2, 0.2, 0.1, false);
        assert!((x - 0.8).abs() < 1e-9 && y == 0.0);
        // Widget groter dan het scherm: links/boven uitlijnen i.p.v. negatief.
        assert_eq!(place(0.5, 0.5, 1.5, 1.5, true), (0.0, 0.0));
    }

    #[test]
    fn partial_json_fills_defaults() {
        let c: DesktopWidgetsConfig = serde_json::from_str(r#"{"items":[{"kind":"media"}]}"#).unwrap();
        assert!(c.enabled && c.grid_snap);
        assert_eq!(c.items[0].scale, 1.0);
        assert_eq!(kind_info("media").1, 380);
    }
}
