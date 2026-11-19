pub mod backup;
pub mod bootstrap;
pub mod config;
pub mod desktop_widgets;
pub mod fastfetch;
pub mod hyprland;
pub mod i18n;
pub mod ipc_client;
pub mod keybinds;
pub mod packages;
pub mod palette;
pub mod process;
pub mod shell_config;
pub mod themes;
pub mod waybar;

/// Tests die `HOME` omzetten draaien parallel; deze lock serialiseert ze.
#[cfg(test)]
pub static HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
