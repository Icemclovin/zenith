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
    if (max - min).abs() < std::f32::EPSILON {
        h = 0.0;
        s = 0.0;
    } else {
        let d = max - min;
        s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        h = if (max - r).abs() < std::f32::EPSILON {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if (max - g).abs() < std::f32::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
    }
    (h * 60.0, s, l)
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s.abs() < std::f32::EPSILON {
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

fn contrast_ratio(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f32 {
    let l1 = relative_luminance(r1, g1, b1);
    let l2 = relative_luminance(r2, g2, b2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

#[derive(Debug, Clone, Copy)]
struct PixelData {
    h: f32,
    s: f32,
    l: f32,
}

pub fn extract_palette(image_path: &str) -> Option<WallpaperPalette> {
    let img = image::open(image_path).ok()?;
    let resized = image::imageops::resize(&img, 64, 64, FilterType::Nearest);

    let mut pixels: Vec<PixelData> = vec![];
    for pixel in resized.pixels() {
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];
        let (h, s, l) = rgb_to_hsl(r, g, b);
        pixels.push(PixelData { h, s, l });
    }

    pixels.sort_by(|a, b| a.l.partial_cmp(&b.l).unwrap_or(Ordering::Equal));

    let bucket_size = pixels.len() / 5;
    let mut representatives = vec![];

    for i in 0..5 {
        let start = i * bucket_size;
        let end = if i == 4 { pixels.len() } else { (i + 1) * bucket_size };
        let bucket = &pixels[start..end];
        let rep = bucket.iter().max_by(|a, b| a.s.partial_cmp(&b.s).unwrap_or(Ordering::Equal)).copied().unwrap_or(PixelData { h: 0.0, s: 0.0, l: 0.0 });
        representatives.push(rep);
    }

    let reps_with_index: Vec<(usize, PixelData)> = representatives.into_iter().enumerate().collect();
    let mut sorted_by_sat = reps_with_index.clone();
    sorted_by_sat.sort_by(|a, b| b.1.s.partial_cmp(&a.1.s).unwrap_or(Ordering::Equal));
    
    let primary_idx = sorted_by_sat[0].0;
    let secondary_idx = sorted_by_sat[1].0;
    
    let to_hex = |pd: PixelData| -> String {
        let (r, g, b) = hsl_to_rgb(pd.h, pd.s, pd.l);
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    };

    let background = to_hex(reps_with_index[0].1);
    let surface = to_hex(reps_with_index[1].1);
    let primary_accent = to_hex(reps_with_index[primary_idx].1);
    let secondary_accent = to_hex(reps_with_index[secondary_idx].1);
    
    let (bg_r, bg_g, bg_b) = hsl_to_rgb(reps_with_index[0].1.h, reps_with_index[0].1.s, reps_with_index[0].1.l);
    let (fg_r, fg_g, fg_b) = hsl_to_rgb(reps_with_index[4].1.h, reps_with_index[4].1.s, reps_with_index[4].1.l);
    
    let contrast = contrast_ratio(bg_r, bg_g, bg_b, fg_r, fg_g, fg_b);
    let foreground = if contrast < 4.5 {
        "#cdd6f4".to_string()
    } else {
        to_hex(reps_with_index[4].1)
    };

    Some(WallpaperPalette {
        background,
        surface,
        primary_accent,
        secondary_accent,
        foreground,
    })
}

pub fn apply_palette(palette: &WallpaperPalette) {
    let mut cfg = ZenithShellConfig::load_or_default();
    cfg.styling.background = palette.background.clone();
    cfg.styling.accent = palette.primary_accent.clone();
    cfg.styling.border_color = palette.surface.clone();
    cfg.styling.text_color = palette.foreground.clone();
    cfg.styling.pill_bg = palette.surface.clone();
    let _ = cfg.save();

    let border_hex = palette.primary_accent.trim_start_matches('#');
    crate::backend::hyprland::set_active_border_color(border_hex);
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
}
