use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FastfetchLogoPadding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FastfetchLogo {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub logo_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<FastfetchLogoPadding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<BTreeMap<String, String>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FastfetchDisplayKey {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FastfetchDisplay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<FastfetchDisplayKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<BTreeMap<String, String>>,
    #[serde(rename = "compactType", skip_serializing_if = "Option::is_none")]
    pub compact_type: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FastfetchModuleConfig {
    #[serde(rename = "type")]
    pub module_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "keyColor")]
    pub key_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FastfetchModuleItem {
    Simple(String),
    Detailed(FastfetchModuleConfig),
}

impl FastfetchModuleItem {
    #[allow(dead_code)]
    pub fn module_type(&self) -> &str {
        match self {
            FastfetchModuleItem::Simple(s) => s.as_str(),
            FastfetchModuleItem::Detailed(d) => d.module_type.as_str(),
        }
    }

    pub fn display_label(&self) -> String {
        match self {
            FastfetchModuleItem::Simple(s) => s.clone(),
            FastfetchModuleItem::Detailed(d) => {
                if d.module_type == "command" {
                    let k = d.key.as_deref().unwrap_or("cmd");
                    let cmd = d.text.as_deref().unwrap_or("");
                    format!("$ {} ({})", k, cmd)
                } else if let Some(ref k) = d.key {
                    format!("{} ({})", d.module_type, k.trim())
                } else {
                    d.module_type.clone()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FastfetchConfig {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<FastfetchLogo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<FastfetchDisplay>,
    #[serde(default)]
    pub modules: Vec<FastfetchModuleItem>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

pub fn get_fastfetch_config_path() -> PathBuf {
    if let Ok(cfg_dir) = std::env::var("XDG_CONFIG_HOME") {
        Path::new(&cfg_dir).join("fastfetch/config.jsonc")
    } else if let Ok(home) = std::env::var("HOME") {
        Path::new(&home).join(".config/fastfetch/config.jsonc")
    } else {
        PathBuf::from(".config/fastfetch/config.jsonc")
    }
}

/// Robust JSONC comment stripper that removes // line comments and /* */ block comments
/// while preserving quoted string literals and escape characters.
pub fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut escape = false;

    while i < len {
        let ch = chars[i];
        let next_ch = if i + 1 < len { Some(chars[i + 1]) } else { None };

        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                out.push('\n');
            }
        } else if in_block_comment {
            if ch == '*' && next_ch == Some('/') {
                in_block_comment = false;
                i += 1; // skip '/'
            }
        } else {
            if ch == '"' {
                in_string = true;
                out.push(ch);
            } else if ch == '/' && next_ch == Some('/') {
                in_line_comment = true;
                i += 1; // skip second '/'
            } else if ch == '/' && next_ch == Some('*') {
                in_block_comment = true;
                i += 1; // skip '*'
            } else {
                out.push(ch);
            }
        }
        i += 1;
    }
    out
}

impl FastfetchConfig {
    pub fn default_preset() -> Self {
        Self {
            schema: Some("https://github.com/fastfetch-cli/fastfetch/raw/master/doc/json_schema.json".to_string()),
            logo: Some(FastfetchLogo {
                logo_type: Some("small".to_string()),
                source: None,
                padding: Some(FastfetchLogoPadding {
                    top: Some(1),
                    left: Some(2),
                    right: Some(3),
                }),
                width: None,
                height: None,
                color: None,
                extra: BTreeMap::new(),
            }),
            display: Some(FastfetchDisplay {
                separator: Some(" ➜ ".to_string()),
                key: Some(FastfetchDisplayKey { width: Some(12) }),
                color: None,
                compact_type: None,
                extra: BTreeMap::new(),
            }),
            modules: vec![
                FastfetchModuleItem::Simple("break".to_string()),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "title".to_string(),
                    key: None,
                    key_color: None,
                    format: Some("{1}".to_string()),
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Simple("break".to_string()),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "os".to_string(),
                    key: Some("OS".to_string()),
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "kernel".to_string(),
                    key: Some("Kernel".to_string()),
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "uptime".to_string(),
                    key: Some("Uptime".to_string()),
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "packages".to_string(),
                    key: Some("Packages".to_string()),
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "shell".to_string(),
                    key: Some("Shell".to_string()),
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "wm".to_string(),
                    key: Some("WM".to_string()),
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "cpu".to_string(),
                    key: Some("CPU".to_string()),
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "memory".to_string(),
                    key: Some("Memory".to_string()),
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: None,
                    extra: BTreeMap::new(),
                }),
                FastfetchModuleItem::Simple("break".to_string()),
                FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                    module_type: "colors".to_string(),
                    key: None,
                    key_color: None,
                    format: None,
                    text: None,
                    symbol: Some("circle".to_string()),
                    extra: BTreeMap::new(),
                }),
            ],
            extra: BTreeMap::new(),
        }
    }

    pub fn load_or_default() -> Self {
        let path = get_fastfetch_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                let clean_json = strip_jsonc_comments(&content);
                if let Ok(cfg) = serde_json::from_str::<FastfetchConfig>(&clean_json) {
                    return cfg;
                }
            }
        }
        Self::default_preset()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = get_fastfetch_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Automatic safe backup
        if path.exists() {
            let bak_path = path.with_extension("jsonc.bak");
            let _ = fs::copy(&path, &bak_path);
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)
    }
}

/// Runs fastfetch with the specified configuration and returns the raw ANSI output.
pub fn generate_preview(cfg: &FastfetchConfig) -> Result<String, String> {
    let temp_path = PathBuf::from(format!("/tmp/zenith-ff-{}.json", std::process::id()));
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&temp_path, json).map_err(|e| e.to_string())?;

    let output = Command::new("fastfetch")
        .args(["-c", temp_path.to_str().unwrap_or(""), "--pipe", "false"])
        .output();

    let _ = fs::remove_file(&temp_path);

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(String::from_utf8_lossy(&out.stdout).to_string())
            } else {
                let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
                if err_msg.is_empty() {
                    Ok(String::from_utf8_lossy(&out.stdout).to_string())
                } else {
                    Err(err_msg)
                }
            }
        }
        Err(e) => Err(format!("Kan fastfetch niet uitvoeren: {}", e)),
    }
}

/// Maps 256-color palette index to hex string (#RRGGBB).
fn ansi_256_to_hex(idx: u8) -> String {
    match idx {
        0 => "#000000".to_string(),
        1 => "#cd0000".to_string(),
        2 => "#00cd00".to_string(),
        3 => "#cdcd00".to_string(),
        4 => "#0000ee".to_string(),
        5 => "#cd00cd".to_string(),
        6 => "#00cdcd".to_string(),
        7 => "#e5e5e5".to_string(),
        8 => "#7f7f7f".to_string(),
        9 => "#ff0000".to_string(),
        10 => "#00ff00".to_string(),
        11 => "#ffff00".to_string(),
        12 => "#5c5cff".to_string(),
        13 => "#ff00ff".to_string(),
        14 => "#00ffff".to_string(),
        15 => "#ffffff".to_string(),
        16..=231 => {
            let offset = idx - 16;
            let r = offset / 36;
            let g = (offset % 36) / 6;
            let b = offset % 6;
            let val = |x: u8| if x == 0 { 0 } else { 55 + x * 40 };
            format!("#{:02x}{:02x}{:02x}", val(r), val(g), val(b))
        }
        232..=255 => {
            let gray = 8 + (idx - 232) * 10;
            format!("#{:02x}{:02x}{:02x}", gray, gray, gray)
        }
    }
}

fn ansi_standard_to_hex(code: u32) -> &'static str {
    match code {
        30 => "#000000",
        31 => "#cd0000",
        32 => "#00cd00",
        33 => "#cdcd00",
        34 => "#0000ee",
        35 => "#cd00cd",
        36 => "#00cdcd",
        37 => "#e5e5e5",
        90 => "#7f7f7f",
        91 => "#ff5555",
        92 => "#50fa7b",
        93 => "#f1fa8c",
        94 => "#bd93f9",
        95 => "#ff79c6",
        96 => "#8be9fd",
        97 => "#ffffff",
        _ => "#cdd6f4",
    }
}

/// Converts raw ANSI text output into Pango markup suitable for GTK4 labels.
pub fn ansi_to_pango(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    let mut is_bold = false;
    let mut current_fg: Option<String> = None;

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '\x1b' && i + 1 < len && chars[i + 1] == '[' {
            // ANSI escape sequence start
            let mut j = i + 2;
            while j < len && !chars[j].is_alphabetic() && chars[j] != 'm' {
                j += 1;
            }

            if j < len {
                let code_char = chars[j];
                let params_str: String = chars[i + 2..j].iter().collect();
                i = j + 1;

                if code_char == 'm' {
                    // SGR (Select Graphic Rendition)
                    if params_str.is_empty() || params_str == "0" {
                        // Reset all
                        if current_fg.is_some() {
                            out.push_str("</span>");
                            current_fg = None;
                        }
                        if is_bold {
                            out.push_str("</b>");
                            is_bold = false;
                        }
                    } else {
                        let parts: Vec<u32> = params_str
                            .split(';')
                            .filter_map(|s| s.parse::<u32>().ok())
                            .collect();

                        let mut p_idx = 0;
                        while p_idx < parts.len() {
                            let code = parts[p_idx];
                            match code {
                                0 => {
                                    if current_fg.is_some() {
                                        out.push_str("</span>");
                                        current_fg = None;
                                    }
                                    if is_bold {
                                        out.push_str("</b>");
                                        is_bold = false;
                                    }
                                }
                                1 => {
                                    if !is_bold {
                                        out.push_str("<b>");
                                        is_bold = true;
                                    }
                                }
                                22 => {
                                    if is_bold {
                                        out.push_str("</b>");
                                        is_bold = false;
                                    }
                                }
                                39 => {
                                    if current_fg.is_some() {
                                        out.push_str("</span>");
                                        current_fg = None;
                                    }
                                }
                                30..=37 | 90..=97 => {
                                    if current_fg.is_some() {
                                        out.push_str("</span>");
                                    }
                                    let hex = ansi_standard_to_hex(code);
                                    out.push_str(&format!("<span foreground=\"{}\">", hex));
                                    current_fg = Some(hex.to_string());
                                }
                                38 => {
                                    // 38;2;R;G;B or 38;5;N
                                    if p_idx + 4 < parts.len() && parts[p_idx + 1] == 2 {
                                        let r = parts[p_idx + 2] as u8;
                                        let g = parts[p_idx + 3] as u8;
                                        let b = parts[p_idx + 4] as u8;
                                        p_idx += 4;
                                        let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
                                        if current_fg.is_some() {
                                            out.push_str("</span>");
                                        }
                                        out.push_str(&format!("<span foreground=\"{}\">", hex));
                                        current_fg = Some(hex);
                                    } else if p_idx + 2 < parts.len() && parts[p_idx + 1] == 5 {
                                        let color_idx = parts[p_idx + 2] as u8;
                                        p_idx += 2;
                                        let hex = ansi_256_to_hex(color_idx);
                                        if current_fg.is_some() {
                                            out.push_str("</span>");
                                        }
                                        out.push_str(&format!("<span foreground=\"{}\">", hex));
                                        current_fg = Some(hex);
                                    }
                                }
                                _ => {}
                            }
                            p_idx += 1;
                        }
                    }
                }
            } else {
                i = len;
            }
        } else {
            match chars[i] {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                ch => out.push(ch),
            }
            i += 1;
        }
    }

    if current_fg.is_some() {
        out.push_str("</span>");
    }
    if is_bold {
        out.push_str("</b>");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_jsonc_comments() {
        let input = r#"{
            // Line comment
            "key": "value // not a comment",
            /* Block
               comment */
            "nested": {
                "num": 42
            }
        }"#;
        let stripped = strip_jsonc_comments(input);
        assert!(!stripped.contains("Line comment"));
        assert!(!stripped.contains("Block"));
        assert!(stripped.contains("value // not a comment"));
        assert!(stripped.contains("\"num\": 42"));

        let val: serde_json::Value = serde_json::from_str(&stripped).expect("Valid JSON");
        assert_eq!(val["key"], "value // not a comment");
        assert_eq!(val["nested"]["num"], 42);
    }

    #[test]
    fn test_fastfetch_config_serde() {
        let sample = r#"{
            "logo": {
                "type": "small",
                "padding": { "top": 1, "left": 2 }
            },
            "display": {
                "separator": " -> "
            },
            "modules": [
                "break",
                {
                    "type": "os",
                    "key": "OS"
                },
                {
                    "type": "command",
                    "key": "Disk Usage",
                    "text": "df -h / | awk 'NR==2 {print $3 \"/\" $2}'"
                }
            ]
        }"#;

        let cfg: FastfetchConfig = serde_json::from_str(sample).expect("Parse config");
        assert_eq!(cfg.logo.as_ref().unwrap().logo_type.as_deref(), Some("small"));
        assert_eq!(cfg.display.as_ref().unwrap().separator.as_deref(), Some(" -> "));
        assert_eq!(cfg.modules.len(), 3);

        match &cfg.modules[2] {
            FastfetchModuleItem::Detailed(d) => {
                assert_eq!(d.module_type, "command");
                assert_eq!(d.key.as_deref(), Some("Disk Usage"));
                assert!(d.text.as_deref().unwrap().contains("df -h"));
            }
            _ => panic!("Expected Detailed module"),
        }

        let serialized = serde_json::to_string_pretty(&cfg).expect("Serialize config");
        assert!(serialized.contains("Disk Usage"));
    }

    #[test]
    fn test_ansi_to_pango() {
        let ansi_str = "\x1b[1m\x1b[38;2;255;181;160mArch\x1b[m \x1b[93mLinux\x1b[0m <test>";
        let pango = ansi_to_pango(ansi_str);
        assert!(pango.contains("<b>"));
        assert!(pango.contains("<span foreground=\"#ffb5a0\">Arch</span>"));
        assert!(pango.contains("<span foreground=\"#f1fa8c\">Linux</span>"));
        assert!(pango.contains("&lt;test&gt;"));
    }
}
