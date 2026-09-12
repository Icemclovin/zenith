mod backend;
mod ui;

use libadwaita::prelude::*;
use libadwaita::{Application, StyleManager};

const APP_ID: &str = "org.zenith.control";

fn is_hyprland_session() -> bool {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
}

fn main() {
    if !is_hyprland_session() {
        eprintln!("Zenith waarschuwing: Geen actieve Hyprland sessie gedetecteerd.");
    }

    // Initialiseer ontbrekende bestanden en imports
    backend::bootstrap::ensure_environment();

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        // Gebruik de officiële Libadwaita StyleManager in plaats van legacy GtkSettings
        let style_manager = StyleManager::default();
        style_manager.set_color_scheme(libadwaita::ColorScheme::PreferDark);

        // Laad het custom Hyprland dark theme in
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(include_str!("ui/style.css"));
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    app.connect_activate(ui::window::build_window);
    app.run();
}