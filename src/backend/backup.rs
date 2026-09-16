use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs::{create_dir_all, read_to_string, remove_dir, remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};

// Volledige back-up/herstel van alle configuratiebestanden die Zenith beheert.
//
// Ontwerp:
//  - Elke back-up is een aparte "snapshot"-map onder `~/.local/share/zenith/backups/`.
//  - Een snapshot bevat een `manifest.json` (metadata) en een `config/`-map met de
//    daadwerkelijke kopieën, met de relatieve paden bewaard.
//  - Herstellen kopieert de inhoud één-op-één terug; er wordt nooit iets onbedoeld
//    verwijderd.
//  - Er worden maximaal `MAX_SNAPSHOTS` snapshots bewaard (oudste eerst verwijderd)
//    zodat de map niet onbeperkt groeit en de back-up altijd betrouwbaar is.

pub const MAX_SNAPSHOTS: usize = 10;

/// Metadata van één snapshot, weergegeven in de UI.
#[derive(Clone, Debug, PartialEq)]
pub struct BackupInfo {
    /// Naam van de snapshot-map (uniek, op tijdstempel gebaseerd).
    pub name: String,
    /// Door de gebruiker gekozen label.
    pub label: String,
    /// UNIX-tijdstempel (seconden) waarop de back-up gemaakt is.
    pub created_at_unix: i64,
    /// Aantal opgeslagen bestanden.
    pub file_count: usize,
    /// Totale grootte in bytes.
    pub size_bytes: u64,
}

/// Manifest dat in elke snapshot-map staat.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct BackupManifest {
    label: String,
    created_at_unix: i64,
    file_count: usize,
    size_bytes: u64,
    /// Relatieve paden (t.o.v. `config/`) van de opgeslagen bronnen.
    files: Vec<String>,
}

/// Beheerde bronbestanden. De `&str` is het relatieve pad binnen de snapshot-`config/`-map.
fn managed_sources() -> Option<Vec<(PathBuf, &'static str)>> {
    let home = std::path::PathBuf::from(std::env::var("HOME").ok()?);
    Some(vec![
        (home.join(".config/quickshell"), "quickshell"),
        (home.join(".config/hypr/zenith.conf"), "hypr/zenith.conf"),
        (home.join(".config/hypr/hyprland.conf"), "hypr/hyprland.conf"),
        (home.join(".config/hypr/hyprland.lua"), "hypr/hyprland.lua"),
        (home.join(".config/waybar"), "waybar"),
        (home.join(".config/kitty/zenith-theme.conf"), "kitty/zenith-theme.conf"),
        (home.join(".config/rofi/zenith-theme.rasi"), "rofi/zenith-theme.rasi"),
    ])
}

fn backups_root() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(home).join(".local/share/zenith/backups"))
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("unix time")
        .as_secs() as i64
}

/// Maak een leesbare map-naam van een label (spaties en tekens naar `-`).
fn slugify(label: &str) -> String {
    let mut out = String::new();
    for ch in label.chars() {
        if ch.is_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == ' ' || ch == '-' || ch == '_' {
            out.push('-');
        }
    }
    if out.is_empty() {
        out = "backup".to_string();
    }
    out
}

/// Kopieer een bron (bestand of map, recursief) naar `dest`. Telt bestanden en grootte.
fn copy_recursive(src: &Path, dest: &Path, count: &mut usize, size: &mut u64) {
    if src.is_file() {
        if let Some(parent) = dest.parent() {
            let _ = create_dir_all(parent);
        }
        if let Ok(meta) = std::fs::metadata(src) {
            *size += meta.len();
        }
        let _ = std::fs::copy(src, dest);
        *count += 1;
    } else if src.is_dir() {
        if let Ok(entries) = std::fs::read_dir(src) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if let Some(name) = entry_path.file_name().and_then(|s| s.to_str()) {
                    let child_src = src.join(name);
                    let child_dest = dest.join(name);
                    if entry_path.is_dir() {
                        let _ = create_dir_all(&child_dest);
                        copy_recursive(&child_src, &child_dest, count, size);
                    } else {
                        if let Some(parent) = child_dest.parent() {
                            let _ = create_dir_all(parent);
                        }
                        if let Ok(meta) = std::fs::metadata(&child_src) {
                            *size += meta.len();
                        }
                        let _ = std::fs::copy(&child_src, &child_dest);
                        *count += 1;
                    }
                }
            }
        }
    }
}

/// Geeft het absolute snapshot-pad op basis van naam, of None als het ongeldig is.
/// Veiligheid: alleen een platte mapnaam toestaan (geen `/`, `\` of `.`-traversal).
fn snapshot_path(name: &str) -> Option<PathBuf> {
    let root = backups_root()?;
    if name.is_empty() || name.chars().any(|c| c == '/' || c == '\\' || c == '.') {
        return None;
    }
    let p = root.join(name);
    if p.is_dir() && p.join("manifest.json").is_file() {
        Some(p)
    } else {
        None
    }
}

/// Maak een volledige back-up van alle Zenith-instellingen.
/// Returnt de metadata van de aangemaakte snapshot, of een foutmelding.
pub fn create_backup(label: &str) -> Result<BackupInfo, String> {
    let root = match backups_root() {
        Some(r) => r,
        None => return Err("Kon back-upmap niet bepalen".to_string()),
    };
    let _ = create_dir_all(&root);

    prune_old_snapshots(&root);

    let created = now_unix();
    let slug = slugify(label);
    let name = format!("{}_{}", created, slug);
    let snap = root.join(name.clone());

    if create_dir_all(&snap).is_err() {
        return Err(format!("Kon back-upmap niet aanmaken: {}", snap.to_string_lossy()));
    }
    let config_dir = snap.join("config");
    let _ = create_dir_all(&config_dir);

    let mut manifest_files = Vec::new();
    let mut count = 0usize;
    let mut total_size = 0u64;

    let sources = match managed_sources() {
        Some(s) => s,
        None => return Err("Kon home-map niet bepalen".to_string()),
    };
    for (src, rel) in sources {
        if !src.exists() {
            continue;
        }
        let dest = config_dir.join(rel);
        if src.is_dir() {
            let _ = create_dir_all(&dest);
            copy_recursive(&src, &dest, &mut count, &mut total_size);
        } else {
            if let Some(parent) = dest.parent() {
                let _ = create_dir_all(parent);
            }
            if let Ok(meta) = std::fs::metadata(&src) {
                total_size += meta.len();
            }
            if std::fs::copy(&src, &dest).is_err() {
                let _ = remove_dir(&snap);
                return Err(format!("Back-up mislukt: kon {} niet kopiëren", src.to_string_lossy()));
            }
            count += 1;
        }
        manifest_files.push(rel.to_string());
    }

    let manifest = BackupManifest {
        label: label.to_string(),
        created_at_unix: created,
        file_count: count,
        size_bytes: total_size,
        files: manifest_files,
    };

    if let Ok(serialized) = serde_json::to_string_pretty(&manifest) {
        let _ = File::create(snap.join("manifest.json"))
            .and_then(|mut f| f.write_all(serialized.as_bytes()));
    }

    Ok(BackupInfo {
        name: name.to_string(),
        label: label.to_string(),
        created_at_unix: created,
        file_count: count,
        size_bytes: total_size,
    })
}

/// Verwijder de oudste snapshots tot er maximaal MAX_SNAPSHOTS overblijven.
fn prune_old_snapshots(root: &Path) {
    let mut backups = discover_snapshots(root);
    while backups.len() > MAX_SNAPSHOTS && !backups.is_empty() {
        let oldest = backups.pop().unwrap();
        let _ = remove_dir(&oldest.0);
    }
}

/// Lees de maps van alle snapshots, nieuwste eerst.
fn discover_snapshots(root: &Path) -> Vec<(PathBuf, BackupInfo)> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let manifest_path = dir.join("manifest.json");
            if !manifest_path.is_file() {
                continue;
            }
            if let Ok(content) = read_to_string(&manifest_path) {
                if let Ok(m) = serde_json::from_str::<BackupManifest>(&content) {
                    let inner_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    result.push((
                        dir.clone(),
                        BackupInfo {
                            name: inner_name.to_string(),
                            label: m.label.clone(),
                            created_at_unix: m.created_at_unix,
                            file_count: m.file_count,
                            size_bytes: m.size_bytes,
                        },
                    ));
                }
            }
        }
    }
    result.sort_by(|a, b| b.1.created_at_unix.partial_cmp(&a.1.created_at_unix).unwrap_or(Ordering::Equal));
    result
}

/// Lijst alle back-ups, nieuwste eerst.
pub fn list_backups() -> Vec<BackupInfo> {
    match backups_root() {
        Some(root) => {
            let _ = create_dir_all(&root);
            discover_snapshots(&root).iter().map(|item| item.1.clone()).collect::<Vec<_>>()
        }
        None => Vec::new(),
    }
}

/// Formatteer een UNIX-tijdstempel als `YYYY-MM-DD HH:MM`.
/// Gebruikt een puur rekenkundige conversie (UTC) zodat dit altijd werkt,
/// zonder afhankelijkheid van locale datum-APIs.
pub fn format_timestamp(unix_secs: i64) -> String {
    let (y, m, d, hh, mm) = civil_from_unix(unix_secs);
    format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m, d, hh, mm)
}

/// Rekenkundige omzetting van een UNIX-tijdstempel (UTC) naar jaar/maand/dag/uur/min.
/// Implementeert het beproefde "civil from days"-algoritme (Howard Hinnant).
fn civil_from_unix(unix_secs: i64) -> (i64, i64, i64, i64, i64) {
    let days = unix_secs.div_euclid(86400);
    let secs_in_day = unix_secs.rem_euclid(86400);
    let hh = secs_in_day.div_euclid(3600);
    let mm = secs_in_day.rem_euclid(3600).div_euclid(60);

    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097); // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d, hh, mm)
}

/// Formatteer een grootte in bytes naar een leesbare string.
pub fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Herstel een back-up door alle opgeslagen config-bestanden terug te kopiëren.
/// Returnt Ok of een foutmelding.
pub fn restore_backup(name: &str) -> Result<(), String> {
    let snap = match snapshot_path(name) {
        Some(p) => p,
        None => return Err("Back-up niet gevonden".to_string()),
    };
    let config_dir = snap.join("config");

    match managed_sources() {
        Some(sources) => {
            for (src, rel) in sources {
                let backed = config_dir.join(rel);
                if !backed.exists() {
                    continue;
                }
                if backed.is_dir() {
                    let _ = create_dir_all(&src);
                    merge_dir_into(&backed, &src);
                } else {
                    if let Some(parent) = src.parent() {
                        let _ = create_dir_all(parent);
                    }
                    if std::fs::copy(&backed, &src).is_err() {
                        return Err(format!(
                            "Herstellen mislukt bij: {}. Wijzigingen tot nu toe zijn wél opgeslagen.",
                            src.to_string_lossy()
                        ));
                    }
                }
            }
            Ok(())
        }
        None => Err("Kon home-map niet bepalen".to_string()),
    }
}

/// Kopieer de inhoud van map `from` naar map `to` (beide bestaan). Overschrijft bestanden.
fn merge_dir_into(from: &Path, to: &Path) {
    if let Ok(entries) = std::fs::read_dir(from) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if let Some(name) = entry_path.file_name().and_then(|s| s.to_str()) {
                let child_src = from.join(name);
                let child_dst = to.join(name);
                if entry_path.is_dir() {
                    let _ = create_dir_all(&child_dst);
                    merge_dir_into(&child_src, &child_dst);
                } else {
                    let _ = std::fs::copy(&child_src, &child_dst);
                }
            }
        }
    }
}

/// Verwijder een back-up definitief.
pub fn delete_backup(name: &str) -> Result<(), String> {
    let snap = match snapshot_path(name) {
        Some(p) => p,
        None => return Err("Back-up niet gevonden".to_string()),
    };
    remove_dir_recursive(&snap);
    Ok(())
}

/// Verwijder een map recursief (inclusief alle inhoud).
fn remove_dir_recursive(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                remove_dir_recursive(&p);
            } else {
                let _ = remove_file(&p);
            }
        }
    }
    let _ = remove_file(dir.join("manifest.json"));
    let _ = remove_dir(dir);
}
#[test]
fn test_civil_from_unix() {
    // Unix epoch 0 = 1970-01-01 00:00 UTC
    let (y, m, d, hh, mm) = civil_from_unix(0);
    assert_eq!((y, m, d, hh, mm), (1970, 1, 1, 0, 0));

    // 2000-01-01 00:00:00 UTC = 946684800 (na een schrikkeljaar-tijdperk)
    let (y2, m2, d2, h2, mm2) = civil_from_unix(946_684_800);
    assert_eq!((y2, m2, d2, h2, mm2), (2000, 1, 1, 0, 0));

    // 2000-03-01 00:00:00 UTC = 951868800 (schrikkeldag 2000-02-29 oversloeg)
    let (y3, m3, d3, _, _) = civil_from_unix(951_868_800);
    assert_eq!(y3, 2000);
    assert_eq!(m3, 3);
    assert_eq!(d3, 1);

    // 2024-03-16 16:30:00 UTC = 1710606600 (geverifieerd via `date`)
    let (y4, m4, d4, h4, mm4) = civil_from_unix(1_710_606_600);
    assert_eq!(y4, 2024);
    assert_eq!(m4, 3);
    assert_eq!(d4, 16);
    assert_eq!(h4, 16);
    assert_eq!(mm4, 30);
}

#[test]
fn test_format_timestamp_smoke() {
    let s = format_timestamp(1_710_606_600);
    assert!(s.len() >= 16, "timestamp string should be non-trivial");
    assert!(s.contains("-"));
    assert!(s.contains(":"));
}

#[test]
fn test_create_list_backup_roundtrip() {
    // Gebruik een tijdelijke HOME zodat testen nooit de echte config aanraken.
    let fake_home = std::path::PathBuf::from("/tmp/zenith-backup-test-home");
    std::env::set_var("HOME", &fake_home);

    // Zet een nep-config neer.
    let qs_dir = fake_home.join(".config/quickshell");
    let _ = std::fs::create_dir_all(&qs_dir);
    let _ = std::fs::write(qs_dir.join("zenith-shell.json"), r#"{"label":"test"}"#);

    let info = create_backup("Mijn eerste back-up");
    assert!(info.is_ok(), "create_backup should succeed");
    let backups = list_backups();
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].label, "Mijn eerste back-up");
    assert!(backups[0].file_count >= 1);

    // Herstel na het verwijderen van het origineel.
    let _ = std::fs::remove_file(qs_dir.join("zenith-shell.json"));
    let restored = restore_backup(&backups[0].name);
    assert!(restored.is_ok(), "restore_backup should succeed");
    assert!(qs_dir.join("zenith-shell.json").is_file(), "file should be restored");

    // Opruimen.
    let del = delete_backup(&backups[0].name);
    assert!(del.is_ok(), "delete_backup should succeed");
    assert_eq!(list_backups().len(), 0);

    std::env::remove_var("HOME");
}
