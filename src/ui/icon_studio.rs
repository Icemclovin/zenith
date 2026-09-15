use gtk4::prelude::*;
use gtk4::{Align, Entry};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;
use crate::backend::{i18n::Translations, shell_config::ZenithShellConfig};

pub fn build_icon_studio_page(tr: &Translations) -> PreferencesPage {
    let page = PreferencesPage::new();
    let cfg = Rc::new(RefCell::new(ZenithShellConfig::load_or_default()));

    // Helper macro / closure to create an ActionRow for an icon
    let create_icon_row = |title: &str, subtitle: &str, get_val: fn(&ZenithShellConfig) -> String, set_val: fn(&mut ZenithShellConfig, String)| -> ActionRow {
        let current_val = get_val(&cfg.borrow());
        let row = ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();

        let entry = Entry::builder()
            .text(&current_val)
            .placeholder_text(&current_val)
            .width_chars(8)
            .valign(Align::Center)
            .halign(Align::End)
            .build();

        let cfg_clone = Rc::clone(&cfg);
        entry.connect_changed(move |e| {
            let txt = e.text().to_string();
            let mut c = cfg_clone.borrow_mut();
            set_val(&mut c, txt);
            let _ = c.save();
        });

        row.add_suffix(&entry);
        row
    };

    // ========================================================
    // GROEP 1: Statusbalk Modules
    // ========================================================
    let group_bar = PreferencesGroup::builder()
        .title(&tr.icons_module_icons)
        .description(&tr.icons_bar_desc)
        .build();

    group_bar.add(&create_icon_row(
        "Werkbladen (Workspaces)",
        "Standaard: 🗂️",
        |c| c.custom_icons.workspaces.clone(),
        |c, v| c.custom_icons.workspaces = v,
    ));

    group_bar.add(&create_icon_row(
        "Actief Venster (Active Window)",
        "Standaard: 🪟",
        |c| c.custom_icons.active_window.clone(),
        |c, v| c.custom_icons.active_window = v,
    ));

    group_bar.add(&create_icon_row(
        "Klok & Kalender (Clock)",
        "Standaard: 🕒",
        |c| c.custom_icons.clock.clone(),
        |c, v| c.custom_icons.clock = v,
    ));

    group_bar.add(&create_icon_row(
        "Systeemvak (Systray)",
        "Standaard: 📥",
        |c| c.custom_icons.systray.clone(),
        |c, v| c.custom_icons.systray = v,
    ));

    group_bar.add(&create_icon_row(
        "Processor (CPU)",
        "Standaard: 🖥",
        |c| c.custom_icons.cpu.clone(),
        |c, v| c.custom_icons.cpu = v,
    ));

    group_bar.add(&create_icon_row(
        "Werkgeheugen (RAM)",
        "Standaard: 💾",
        |c| c.custom_icons.ram.clone(),
        |c, v| c.custom_icons.ram = v,
    ));

    group_bar.add(&create_icon_row(
        "Netwerk (Network)",
        "Standaard: 🌐",
        |c| c.custom_icons.network.clone(),
        |c, v| c.custom_icons.network = v,
    ));

    group_bar.add(&create_icon_row(
        "Bluetooth",
        "Standaard: ᛒ",
        |c| c.custom_icons.bluetooth.clone(),
        |c, v| c.custom_icons.bluetooth = v,
    ));

    group_bar.add(&create_icon_row(
        "App Launcher",
        "Standaard: 🚀",
        |c| c.custom_icons.launcher.clone(),
        |c, v| c.custom_icons.launcher = v,
    ));

    group_bar.add(&create_icon_row(
        "Zenith Brand Label",
        "Standaard: 🌟",
        |c| c.custom_icons.brand.clone(),
        |c, v| c.custom_icons.brand = v,
    ));

    page.add(&group_bar);

    // ========================================================
    // GROEP 2: Audio & Helderheid
    // ========================================================
    let group_audio = PreferencesGroup::builder()
        .title("Audio & Schermhelderheid")
        .description("Iconen voor geluidsvolume, microfoon en schermhelderheid")
        .build();

    group_audio.add(&create_icon_row(
        "Volume Hoog (>50%)",
        "Standaard: 🔊",
        |c| c.custom_icons.volume_high.clone(),
        |c, v| c.custom_icons.volume_high = v,
    ));

    group_audio.add(&create_icon_row(
        "Volume Gemiddeld / Laag (<=50%)",
        "Standaard: 🔉",
        |c| c.custom_icons.volume_medium.clone(),
        |c, v| c.custom_icons.volume_medium = v,
    ));

    group_audio.add(&create_icon_row(
        "Volume Gedempt (Mute)",
        "Standaard: 🔇",
        |c| c.custom_icons.volume_muted.clone(),
        |c, v| c.custom_icons.volume_muted = v,
    ));

    group_audio.add(&create_icon_row(
        "Microfoon Actief",
        "Standaard: 🎙️",
        |c| c.custom_icons.mic.clone(),
        |c, v| c.custom_icons.mic = v,
    ));

    group_audio.add(&create_icon_row(
        "Microfoon Gedempt",
        "Standaard: 🎙️❌",
        |c| c.custom_icons.mic_muted.clone(),
        |c, v| c.custom_icons.mic_muted = v,
    ));

    group_audio.add(&create_icon_row(
        "Schermhelderheid",
        "Standaard: ☀️",
        |c| c.custom_icons.brightness.clone(),
        |c, v| c.custom_icons.brightness = v,
    ));

    page.add(&group_audio);

    // ========================================================
    // GROEP 3: Batterij & Energie
    // ========================================================
    let group_power = PreferencesGroup::builder()
        .title("Batterij & Energiebeheer")
        .description("Accu indicatoren en actieknoppen voor het systeem")
        .build();

    group_power.add(&create_icon_row(
        "Batterij Normaal / Vol",
        "Standaard: 🔋",
        |c| c.custom_icons.battery_full.clone(),
        |c, v| c.custom_icons.battery_full = v,
    ));

    group_power.add(&create_icon_row(
        "Batterij Laag (<20%)",
        "Standaard: 🪫",
        |c| c.custom_icons.battery_low.clone(),
        |c, v| c.custom_icons.battery_low = v,
    ));

    group_power.add(&create_icon_row(
        "Batterij Opladen",
        "Standaard: ⚡",
        |c| c.custom_icons.battery_charging.clone(),
        |c, v| c.custom_icons.battery_charging = v,
    ));

    group_power.add(&create_icon_row(
        "Vergrendelen (Lock)",
        "Standaard: 🔒",
        |c| c.custom_icons.lock.clone(),
        |c, v| c.custom_icons.lock = v,
    ));

    group_power.add(&create_icon_row(
        "Slaapstand (Sleep)",
        "Standaard: 💤",
        |c| c.custom_icons.sleep.clone(),
        |c, v| c.custom_icons.sleep = v,
    ));

    group_power.add(&create_icon_row(
        "Afmelden (Logout)",
        "Standaard: 🚪",
        |c| c.custom_icons.logout.clone(),
        |c, v| c.custom_icons.logout = v,
    ));

    group_power.add(&create_icon_row(
        "Herstarten (Reboot)",
        "Standaard: 🔄",
        |c| c.custom_icons.reboot.clone(),
        |c, v| c.custom_icons.reboot = v,
    ));

    group_power.add(&create_icon_row(
        "Afsluiten (Shutdown)",
        "Standaard: ⏻",
        |c| c.custom_icons.shutdown.clone(),
        |c, v| c.custom_icons.shutdown = v,
    ));

    page.add(&group_power);

    // ========================================================
    // GROEP 4: Control Center & Media
    // ========================================================
    let group_cc = PreferencesGroup::builder()
        .title(&tr.icons_card_icons)
        .description(&tr.icons_cc_desc)
        .build();

    group_cc.add(&create_icon_row(
        "Wi-Fi Schakelaar",
        "Standaard: 📶",
        |c| c.custom_icons.wifi.clone(),
        |c, v| c.custom_icons.wifi = v,
    ));

    group_cc.add(&create_icon_row(
        "Niet Storen Actief (DND On)",
        "Standaard: 🔕",
        |c| c.custom_icons.dnd_on.clone(),
        |c, v| c.custom_icons.dnd_on = v,
    ));

    group_cc.add(&create_icon_row(
        "Niet Storen Inactief (DND Off)",
        "Standaard: 🔔",
        |c| c.custom_icons.dnd_off.clone(),
        |c, v| c.custom_icons.dnd_off = v,
    ));

    group_cc.add(&create_icon_row(
        "Nachtmodus (Nightlight)",
        "Standaard: 🌙",
        |c| c.custom_icons.nightlight.clone(),
        |c, v| c.custom_icons.nightlight = v,
    ));

    group_cc.add(&create_icon_row(
        "Mediaspeler Logo",
        "Standaard: 🎵",
        |c| c.custom_icons.media.clone(),
        |c, v| c.custom_icons.media = v,
    ));

    group_cc.add(&create_icon_row(
        "Media Afspelen (Play)",
        "Standaard: ▶",
        |c| c.custom_icons.media_play.clone(),
        |c, v| c.custom_icons.media_play = v,
    ));

    group_cc.add(&create_icon_row(
        "Media Pauzeren (Pause)",
        "Standaard: ⏸",
        |c| c.custom_icons.media_pause.clone(),
        |c, v| c.custom_icons.media_pause = v,
    ));

    group_cc.add(&create_icon_row(
        "Media Vorige (Prev)",
        "Standaard: ⏮",
        |c| c.custom_icons.media_prev.clone(),
        |c, v| c.custom_icons.media_prev = v,
    ));

    group_cc.add(&create_icon_row(
        "Media Volgende (Next)",
        "Standaard: ⏭",
        |c| c.custom_icons.media_next.clone(),
        |c, v| c.custom_icons.media_next = v,
    ));

    group_cc.add(&create_icon_row(
        "Quick Settings / Instellingen",
        "Standaard: ⚙️",
        |c| c.custom_icons.settings.clone(),
        |c, v| c.custom_icons.settings = v,
    ));

    group_cc.add(&create_icon_row(
        "Custom Script Standaard Icoon",
        "Standaard: 💻",
        |c| c.custom_icons.script.clone(),
        |c, v| c.custom_icons.script = v,
    ));

    group_cc.add(&create_icon_row(
        "Custom Widget Standaard Icoon",
        "Standaard: ✨",
        |c| c.custom_icons.custom.clone(),
        |c, v| c.custom_icons.custom = v,
    ));

    group_cc.add(&create_icon_row(
        "Sluitknop (Close)",
        "Standaard: ✕",
        |c| c.custom_icons.close.clone(),
        |c, v| c.custom_icons.close = v,
    ));

    page.add(&group_cc);

    page
}
