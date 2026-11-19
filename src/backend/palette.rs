use crate::backend::shell_config::ZenithShellConfig;
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WallpaperPalette {
    pub background: String,
    pub surface: String,
    pub primary_accent: String,
    pub secondary_accent: String,
    pub foreground: String,
}

impl Default for WallpaperPalette {
    fn default() -> Self {
        Self {
            background: "#1e1e2e".to_string(),
            surface: "#181825".to_string(),
            primary_accent: "#89b4fa".to_string(),
            secondary_accent: "#cba6f7".to_string(),
            foreground: "#cdd6f4".to_string(),
        }
    }
}

pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    let h;
    let s;
    if (max - min).abs() < f32::EPSILON {
        h = 0.0;
        s = 0.0;
    } else {
        let d = max - min;
        s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        h = if (max - r).abs() < f32::EPSILON {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if (max - g).abs() < f32::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
    }
    (h * 60.0, s, l)
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s.abs() < f32::EPSILON {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let h_k = h / 360.0;

    let tr = h_k + 1.0 / 3.0;
    let tg = h_k;
    let tb = h_k - 1.0 / 3.0;

    let adjust = |mut t: f32| -> f32 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 1.0 / 2.0 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };

    let r = (adjust(tr) * 255.0).round() as u8;
    let g = (adjust(tg) * 255.0).round() as u8;
    let b = (adjust(tb) * 255.0).round() as u8;

    (r, g, b)
}

#[cfg(test)]
fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    let rs = r as f32 / 255.0;
    let gs = g as f32 / 255.0;
    let bs = b as f32 / 255.0;

    let calc = |c: f32| {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };

    0.2126 * calc(rs) + 0.7152 * calc(gs) + 0.0722 * calc(bs)
}

#[cfg(test)]
fn contrast_ratio(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f32 {
    let l1 = relative_luminance(r1, g1, b1);
    let l2 = relative_luminance(r2, g2, b2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Material-You-achtig: kies de dominante kleurtoon (hue) van de wallpaper en leid daar
/// een tonaal palet van af, zodat achtergrond donker, tekst licht en accent pastel is.
pub fn extract_palette(image_path: &str) -> Option<WallpaperPalette> {
    let img = image::open(image_path).ok()?.to_rgb8();
    let resized = image::imageops::resize(&img, 64, 64, FilterType::Triangle);

    // Hue-histogram (36 vakjes van 10°), gewogen naar verzadiging en "middentonen".
    let mut weight = [0f32; 36];
    let mut hue_sum = [0f32; 36];
    for px in resized.pixels() {
        let (h, s, l) = rgb_to_hsl(px[0], px[1], px[2]);
        let w = s * (1.0 - (l - 0.5).abs() * 2.0);
        let bin = ((h / 10.0) as usize).min(35);
        weight[bin] += w;
        hue_sum[bin] += h * w;
    }
    let best = |skip: Option<usize>| -> Option<usize> {
        (0..36)
            .filter(|&i| weight[i] > 0.0)
            // tweede kleur minstens 40° van de eerste, anders lijkt hij er te veel op
            .filter(|&i| skip.is_none_or(|j| { let d = i.abs_diff(j); d.min(36 - d) >= 4 }))
            .max_by(|&a, &b| weight[a].partial_cmp(&weight[b]).unwrap_or(Ordering::Equal))
    };
    let mean_hue = |i: usize| hue_sum[i] / weight[i];

    // Grijze wallpaper: neutraal palet met een rustig blauw accent.
    let (seed, sat) = match best(None) {
        Some(i) => (mean_hue(i), 1.0),
        None => (220.0, 0.0),
    };
    let second = best(best(None)).map(mean_hue).unwrap_or((seed + 40.0) % 360.0);

    let hex = |h: f32, s: f32, l: f32| -> String {
        let (r, g, b) = hsl_to_rgb(h, s, l);
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    };
    Some(WallpaperPalette {
        background: hex(seed, 0.22 * sat, 0.09),
        surface: hex(seed, 0.20 * sat, 0.14),
        primary_accent: hex(seed, 0.65 * sat.max(0.4), 0.74),
        secondary_accent: hex(second, 0.50 * sat.max(0.3), 0.76),
        foreground: hex(seed, 0.25 * sat, 0.91),
    })
}

pub fn apply_palette(palette: &WallpaperPalette) {
    // 1. Update zenith-shell.json voor Quickshell
    let mut cfg = ZenithShellConfig::load_or_default();
    cfg.styling.background = palette.background.clone();
    cfg.styling.accent = palette.primary_accent.clone();
    cfg.styling.border_color = palette.surface.clone();
    cfg.styling.text_color = palette.foreground.clone();
    cfg.styling.pill_bg = palette.surface.clone();
    let _ = cfg.save();

    // 2. Update Hyprland live borderkleur
    let border_hex = palette.primary_accent.trim_start_matches('#');
    crate::backend::hyprland::set_keyword("general:col.active_border", format!("rgb({})", border_hex));

    // 3. Update en sla op in zenith.conf (Hyprland persistente configuratie)
    let mut hypr_cfg = crate::backend::config::load_config();
    hypr_cfg.active_border_color = border_hex.to_string();
    hypr_cfg.qs_bg = palette.background.trim_start_matches('#').to_string();
    hypr_cfg.qs_accent = palette.primary_accent.trim_start_matches('#').to_string();
    hypr_cfg.qs_border_color = palette.surface.trim_start_matches('#').to_string();
    hypr_cfg.qs_text_color = palette.foreground.trim_start_matches('#').to_string();
    hypr_cfg.qs_pill_bg = palette.surface.trim_start_matches('#').to_string();
    crate::backend::config::save_config(&hypr_cfg);

    // 4. Update Kitty en Rofi thema's
    crate::backend::themes::update_kitty(&palette.background, &palette.foreground, 0.95, 11.5);
    crate::backend::themes::update_rofi(&palette.background, &palette.foreground, 0.95, 12);

    // 5. Synchroniseer Quickshell en herlaad thema's live
    crate::backend::themes::sync_shell_config(&cfg);
    crate::backend::themes::update_quickshell(&hypr_cfg);
}

/// Glassmorphism-palet (ZenithOS-spec): frost glas UI-identiteit.
/// Primary `#458588`, achtergrond `#1D2021` (85% ondoorzichtig),
/// Accent `#D79921`, Success `#98971A`.
pub fn glassmorphism_palette() -> WallpaperPalette {
    WallpaperPalette {
        background: "#1d2021".to_string(),
        surface: "#232627".to_string(),
        primary_accent: "#458588".to_string(),
        secondary_accent: "#98971a".to_string(),
        foreground: "#e6e1d8".to_string(),
    }
}

/// Pas het Glassmorphism-palet toe met de frosted-glas laag (85% ondoorzichtig).
pub fn apply_glassmorphism() {
    let pal = glassmorphism_palette();
    let mut cfg = ZenithShellConfig::load_or_default();
    cfg.styling.opacity = 0.85;
    cfg.styling.pill_opacity = 0.85;
    let _ = cfg.save();
    apply_palette(&pal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsl() {
        let (h, s, l) = rgb_to_hsl(255, 0, 0);
        assert!((h - 0.0).abs() < 0.1);
        assert!((s - 1.0).abs() < 0.1);
        assert!((l - 0.5).abs() < 0.1);

        let (h, _, _) = rgb_to_hsl(0, 255, 0);
        assert!((h - 120.0).abs() < 0.1);
        
        let (_, _, l) = rgb_to_hsl(255, 255, 255);
        assert!((l - 1.0).abs() < 0.1);

        let (_, _, l) = rgb_to_hsl(0, 0, 0);
        assert!((l - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_hsl_to_rgb() {
        let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);

        let (r, g, b) = hsl_to_rgb(120.0, 1.0, 0.5);
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_palette_defaults() {
        let p = WallpaperPalette::default();
        assert_eq!(p.background, "#1e1e2e");
        assert_eq!(p.surface, "#181825");
        assert_eq!(p.primary_accent, "#89b4fa");
        assert_eq!(p.secondary_accent, "#cba6f7");
        assert_eq!(p.foreground, "#cdd6f4");
    }

    #[test]
    fn test_extract_palette_nonexistent() {
        assert!(extract_palette("/path/to/nowhere.png").is_none());
    }

    #[test]
    fn test_extract_palette_from_image() {
        let test_path = "/tmp/zenith_test_wallpaper.png";
        let mut img = image::RgbImage::new(16, 16);
        for x in 0..16 {
            for y in 0..16 {
                let color = match (x % 4, y % 4) {
                    (0, 0) => image::Rgb([30, 30, 46]),   // Dark background
                    (1, 1) => image::Rgb([137, 180, 250]), // Vivid blue accent
                    (2, 2) => image::Rgb([203, 166, 247]), // Mauve secondary
                    _ => image::Rgb([205, 214, 244]),      // Light foreground
                };
                img.put_pixel(x, y, color);
            }
        }
        img.save(test_path).expect("Failed to save test image");

        let palette_opt = extract_palette(test_path);
        let _ = std::fs::remove_file(test_path);

        assert!(palette_opt.is_some(), "Palette extraction should succeed on valid image");
        let p = palette_opt.unwrap();
        assert!(p.background.starts_with('#') && p.background.len() == 7);
        assert!(p.surface.starts_with('#') && p.surface.len() == 7);
        assert!(p.primary_accent.starts_with('#') && p.primary_accent.len() == 7);
        assert!(p.secondary_accent.starts_with('#') && p.secondary_accent.len() == 7);
        assert!(p.foreground.starts_with('#') && p.foreground.len() == 7);
    }

    fn rgb(hex: &str) -> (u8, u8, u8) {
        let v = u32::from_str_radix(&hex[1..], 16).unwrap();
        ((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }

    #[test]
    fn extracted_palette_is_readable_and_follows_wallpaper() {
        // Olijfgroene/gele wallpaper gaf vroeger achtergrond == accent (#6b6300).
        let path = std::env::temp_dir().join("zenith_test_olive.png");
        let mut img = image::RgbImage::new(16, 16);
        for (x, _, p) in img.enumerate_pixels_mut() {
            *p = if x < 12 { image::Rgb([107, 99, 0]) } else { image::Rgb([20, 20, 20]) };
        }
        img.save(&path).unwrap();
        let p = extract_palette(path.to_str().unwrap()).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_ne!(p.background, p.primary_accent);
        let (b, f, a) = (rgb(&p.background), rgb(&p.foreground), rgb(&p.primary_accent));
        assert!(contrast_ratio(b.0, b.1, b.2, f.0, f.1, f.2) >= 7.0);
        assert!(contrast_ratio(b.0, b.1, b.2, a.0, a.1, a.2) >= 4.5);
        let (h, _, _) = rgb_to_hsl(a.0, a.1, a.2);
        assert!((50.0..65.0).contains(&h), "accent-hue {h} volgt de wallpaper niet");
    }

    #[test]
    fn glassmorphism_palette_values() {
        let p = glassmorphism_palette();
        assert_eq!(p.background, "#1d2021");
        assert_eq!(p.primary_accent, "#458588");
        assert_eq!(p.secondary_accent, "#98971a");
        assert_eq!(p.foreground, "#e6e1d8");
    }
}
