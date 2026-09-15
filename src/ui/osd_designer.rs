use gtk4::prelude::*;
use gtk4::{
    Button, DropDown, Orientation, Scale, StringList, Switch,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::config::ZenithConfig;
use crate::backend::i18n::Translations;
use crate::backend::shell_config::ZenithShellConfig;
use crate::backend::themes;

pub fn build_osd_page(_state: &Rc<RefCell<ZenithConfig>>, tr: &Translations) -> PreferencesPage {
    let page = PreferencesPage::new();
    let shell_state = Rc::new(RefCell::new(ZenithShellConfig::load_or_default()));

    // ==========================================
    // 1. OSD Weergave & Vormgeving
    // ==========================================
    let group_style = PreferencesGroup::builder()
        .title("On-Screen Display (OSD)")
        .description("Pas de zwevende overlay-notificaties voor volume en helderheid aan")
        .build();

    // Toggle Ingeschakeld
    let row_enabled = ActionRow::builder()
        .title(&tr.osd_enabled)
        .subtitle("Toon interactieve overlays bij het wijzigen van volume of helderheid")
        .build();
    let sw_enabled = Switch::builder().active(shell_state.borrow().osd.enabled).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_enabled.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            sh.osd.enabled = s.is_active();
            themes::sync_shell_config(&sh);
        });
    }
    row_enabled.add_suffix(&sw_enabled);
    group_style.add(&row_enabled);

    // Positie
    let row_pos = ActionRow::builder()
        .title(&tr.osd_position)
        .subtitle("Kies waar de OSD op het beeldscherm verschijnt")
        .build();
    let pos_model = StringList::new(&[
        "Onderaan (bottom)",
        "Bovenaan (top)",
        "Midden (center)",
        "Rechtsboven (top-right)",
        "Rechtsonder (bottom-right)",
    ]);
    let dd_pos = DropDown::builder().model(&pos_model).valign(gtk4::Align::Center).build();
    let cur_pos = shell_state.borrow().osd.position.clone();
    let cur_pos_idx = match cur_pos.as_str() {
        "top" => 1,
        "center" => 2,
        "top-right" => 3,
        "bottom-right" => 4,
        _ => 0,
    };
    dd_pos.set_selected(cur_pos_idx);
    {
        let sh_st = Rc::clone(&shell_state);
        dd_pos.connect_selected_notify(move |d| {
            let pos_str = match d.selected() {
                1 => "top",
                2 => "center",
                3 => "top-right",
                4 => "bottom-right",
                _ => "bottom",
            };
            let mut sh = sh_st.borrow_mut();
            sh.osd.position = pos_str.to_string();
            themes::sync_shell_config(&sh);
        });
    }
    row_pos.add_suffix(&dd_pos);
    group_style.add(&row_pos);

    // Oriëntatie
    let row_orient = ActionRow::builder()
        .title(&tr.osd_orientation)
        .subtitle("Horizontale capsule of verticale slider")
        .build();
    let orient_model = StringList::new(&["Horizontale Capsule", "Verticale Slider"]);
    let dd_orient = DropDown::builder().model(&orient_model).valign(gtk4::Align::Center).build();
    dd_orient.set_selected(if shell_state.borrow().osd.orientation == "vertical" { 1 } else { 0 });
    {
        let sh_st = Rc::clone(&shell_state);
        dd_orient.connect_selected_notify(move |d| {
            let o_str = if d.selected() == 1 { "vertical" } else { "horizontal" };
            let mut sh = sh_st.borrow_mut();
            sh.osd.orientation = o_str.to_string();
            themes::sync_shell_config(&sh);
        });
    }
    row_orient.add_suffix(&dd_orient);
    group_style.add(&row_orient);

    // Breedte Slider
    let initial_width = shell_state.borrow().osd.width;
    let row_width = ActionRow::builder()
        .title(&tr.osd_width)
        .subtitle(&format!("{} px", initial_width))
        .build();
    let s_width = Scale::with_range(Orientation::Horizontal, 160.0, 500.0, 10.0);
    s_width.set_value(initial_width as f64);
    s_width.set_width_request(160);
    {
        let r_c = row_width.clone();
        let sh_st = Rc::clone(&shell_state);
        s_width.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            let mut sh = sh_st.borrow_mut();
            sh.osd.width = v;
            themes::sync_shell_config(&sh);
        });
    }
    row_width.add_suffix(&s_width);
    group_style.add(&row_width);

    // Hoogte / Dikte Slider
    let initial_height = shell_state.borrow().osd.height;
    let row_height = ActionRow::builder()
        .title(&tr.osd_height)
        .subtitle(&format!("{} px", initial_height))
        .build();
    let s_height = Scale::with_range(Orientation::Horizontal, 32.0, 100.0, 4.0);
    s_height.set_value(initial_height as f64);
    s_height.set_width_request(160);
    {
        let r_c = row_height.clone();
        let sh_st = Rc::clone(&shell_state);
        s_height.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            let mut sh = sh_st.borrow_mut();
            sh.osd.height = v;
            themes::sync_shell_config(&sh);
        });
    }
    row_height.add_suffix(&s_height);
    group_style.add(&row_height);

    // Weergaveduur (Timeout)
    let initial_timeout = shell_state.borrow().osd.timeout_ms;
    let row_timeout = ActionRow::builder()
        .title(&tr.osd_timeout)
        .subtitle(&format!("{} ms", initial_timeout))
        .build();
    let s_timeout = Scale::with_range(Orientation::Horizontal, 500.0, 5000.0, 250.0);
    s_timeout.set_value(initial_timeout as f64);
    s_timeout.set_width_request(160);
    {
        let r_c = row_timeout.clone();
        let sh_st = Rc::clone(&shell_state);
        s_timeout.connect_value_changed(move |s| {
            let v = s.value().round() as u32;
            r_c.set_subtitle(&format!("{} ms", v));
            let mut sh = sh_st.borrow_mut();
            sh.osd.timeout_ms = v;
            themes::sync_shell_config(&sh);
        });
    }
    row_timeout.add_suffix(&s_timeout);
    group_style.add(&row_timeout);

    // Percentage tonen
    let row_pct = ActionRow::builder()
        .title(&tr.osd_percentage)
        .subtitle("Toon het numerieke percentage naast de voortgangsbalk")
        .build();
    let sw_pct = Switch::builder().active(shell_state.borrow().osd.show_percentage).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_pct.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            sh.osd.show_percentage = s.is_active();
            themes::sync_shell_config(&sh);
        });
    }
    row_pct.add_suffix(&sw_pct);
    group_style.add(&row_pct);

    // Icoon tonen
    let row_ico = ActionRow::builder()
        .title(&tr.osd_icon)
        .subtitle("Toon het relevante hardware-icoon in de capsule")
        .build();
    let sw_ico = Switch::builder().active(shell_state.borrow().osd.show_icon).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_ico.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            sh.osd.show_icon = s.is_active();
            themes::sync_shell_config(&sh);
        });
    }
    row_ico.add_suffix(&sw_ico);
    group_style.add(&row_ico);

    page.add(&group_style);

    // ==========================================
    // 2. Hardware Doelen (Triggers)
    // ==========================================
    let group_targets = PreferencesGroup::builder()
        .title(&tr.osd_hardware)
        .description("Kies voor welke hardware-wijzigingen de OSD geactiveerd wordt")
        .build();

    // Volume target
    let row_tg_vol = ActionRow::builder()
        .title("Luidsprekervolume (Sink)")
        .subtitle("Reageert op WirePlumber volume- en dempingswijzigingen")
        .build();
    let is_vol_active = shell_state.borrow().osd.hardware_targets.contains(&"volume".to_string());
    let sw_tg_vol = Switch::builder().active(is_vol_active).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_tg_vol.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            if s.is_active() {
                if !sh.osd.hardware_targets.contains(&"volume".to_string()) {
                    sh.osd.hardware_targets.push("volume".to_string());
                }
            } else {
                sh.osd.hardware_targets.retain(|x| x != "volume");
            }
            themes::sync_shell_config(&sh);
        });
    }
    row_tg_vol.add_suffix(&sw_tg_vol);
    group_targets.add(&row_tg_vol);

    // Mic target
    let row_tg_mic = ActionRow::builder()
        .title("Microfoon Ingang (Source)")
        .subtitle("Reageert op WirePlumber microfoonvolume en mute")
        .build();
    let is_mic_active = shell_state.borrow().osd.hardware_targets.contains(&"mic".to_string());
    let sw_tg_mic = Switch::builder().active(is_mic_active).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_tg_mic.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            if s.is_active() {
                if !sh.osd.hardware_targets.contains(&"mic".to_string()) {
                    sh.osd.hardware_targets.push("mic".to_string());
                }
            } else {
                sh.osd.hardware_targets.retain(|x| x != "mic");
            }
            themes::sync_shell_config(&sh);
        });
    }
    row_tg_mic.add_suffix(&sw_tg_mic);
    group_targets.add(&row_tg_mic);

    // Brightness target
    let row_tg_br = ActionRow::builder()
        .title("Schermhelderheid")
        .subtitle("Reageert op brightnessctl wijzigingen")
        .build();
    let is_br_active = shell_state.borrow().osd.hardware_targets.contains(&"brightness".to_string());
    let sw_tg_br = Switch::builder().active(is_br_active).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_tg_br.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            if s.is_active() {
                if !sh.osd.hardware_targets.contains(&"brightness".to_string()) {
                    sh.osd.hardware_targets.push("brightness".to_string());
                }
            } else {
                sh.osd.hardware_targets.retain(|x| x != "brightness");
            }
            themes::sync_shell_config(&sh);
        });
    }
    row_tg_br.add_suffix(&sw_tg_br);
    group_targets.add(&row_tg_br);

    page.add(&group_targets);

    // ==========================================
    // 3. Live Test Knoppen
    // ==========================================
    let group_test = PreferencesGroup::builder()
        .title(&tr.osd_test)
        .description("Test de live weergave van de OSD via Quickshell IPC")
        .build();

    let row_test_vol = ActionRow::builder()
        .title("Test Geluidsvolume OSD")
        .subtitle("Triggert direct een volume-overlay op het scherm")
        .build();
    let btn_test_vol = Button::builder().label("🔊 Test Volume").valign(gtk4::Align::Center).build();
    btn_test_vol.connect_clicked(|_| {
        let _ = std::process::Command::new("quickshell")
            .args(["ipc", "call", "osd", "popup", "🔊", "75%", "0.75"])
            .spawn();
    });
    row_test_vol.add_suffix(&btn_test_vol);
    group_test.add(&row_test_vol);

    let row_test_br = ActionRow::builder()
        .title("Test Schermhelderheid OSD")
        .subtitle("Triggert direct een helderheids-overlay op het scherm")
        .build();
    let btn_test_br = Button::builder().label("☀️ Test Helderheid").valign(gtk4::Align::Center).build();
    btn_test_br.connect_clicked(|_| {
        let _ = std::process::Command::new("quickshell")
            .args(["ipc", "call", "osd", "popup", "☀️", "60%", "0.60"])
            .spawn();
    });
    row_test_br.add_suffix(&btn_test_br);
    group_test.add(&row_test_br);

    let row_test_mic = ActionRow::builder()
        .title("Test Microfoon OSD")
        .subtitle("Triggert direct een microfoon-overlay op het scherm")
        .build();
    let btn_test_mic = Button::builder().label("🎙️ Test Microfoon").valign(gtk4::Align::Center).build();
    btn_test_mic.connect_clicked(|_| {
        let _ = std::process::Command::new("quickshell")
            .args(["ipc", "call", "osd", "popup", "🎙️", "80%", "0.80"])
            .spawn();
    });
    row_test_mic.add_suffix(&btn_test_mic);
    group_test.add(&row_test_mic);

    page.add(&group_test);

    page
}
