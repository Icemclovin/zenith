use gtk4::prelude::*;
use gtk4::{
    Button, DropDown, Entry, ListBox, Orientation, Scale, StringList, Switch,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::config::ZenithConfig;
use crate::backend::i18n::Translations;
use crate::backend::shell_config::{CustomScriptModule, ZenithShellConfig};
use crate::backend::themes;

pub fn build_control_center_page(_state: &Rc<RefCell<ZenithConfig>>, tr: &Translations) -> PreferencesPage {
    let page = PreferencesPage::new();
    let shell_state = Rc::new(RefCell::new(ZenithShellConfig::load_or_default()));

    // ==========================================
    // 1. Layout & Afmetingen
    // ==========================================
    let group_appearance = PreferencesGroup::builder()
        .title("Control Center")
        .description("Pas de positie, afmetingen en achtergrondstijl van het paneel aan")
        .build();

    // Toggle Ingeschakeld
    let row_enabled = ActionRow::builder()
        .title(&tr.cc_enabled)
        .subtitle("Schakel het popup-bedieningspaneel in of uit")
        .build();
    let sw_enabled = Switch::builder().active(shell_state.borrow().control_center.enabled).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_enabled.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            sh.control_center.enabled = s.is_active();
            themes::sync_shell_config(&sh);
        });
    }
    row_enabled.add_suffix(&sw_enabled);
    group_appearance.add(&row_enabled);

    // Positie op Scherm
    let row_pos = ActionRow::builder()
        .title(&tr.cc_position)
        .subtitle("Kies de hoek of positie waarin het Control Center verschijnt")
        .build();
    let pos_model = StringList::new(&[
        "Rechtsboven (top-right)",
        "Rechtsonder (bottom-right)",
        "Linksboven (top-left)",
        "Linksonder (bottom-left)",
        "Zwevend Midden (floating-center)",
    ]);
    let dd_pos = DropDown::builder().model(&pos_model).valign(gtk4::Align::Center).build();
    let cur_pos = shell_state.borrow().control_center.position.clone();
    let cur_pos_idx = match cur_pos.as_str() {
        "bottom-right" => 1,
        "top-left" => 2,
        "bottom-left" => 3,
        "floating-center" => 4,
        _ => 0,
    };
    dd_pos.set_selected(cur_pos_idx);
    {
        let sh_st = Rc::clone(&shell_state);
        dd_pos.connect_selected_notify(move |d| {
            let pos_str = match d.selected() {
                1 => "bottom-right",
                2 => "top-left",
                3 => "bottom-left",
                4 => "floating-center",
                _ => "top-right",
            };
            let mut sh = sh_st.borrow_mut();
            sh.control_center.position = pos_str.to_string();
            themes::sync_shell_config(&sh);
        });
    }
    row_pos.add_suffix(&dd_pos);
    group_appearance.add(&row_pos);

    // Breedte Slider
    let initial_width = shell_state.borrow().control_center.width;
    let row_width = ActionRow::builder()
        .title(&tr.cc_width)
        .subtitle(format!("{} px", initial_width))
        .build();
    let s_width = Scale::with_range(Orientation::Horizontal, 280.0, 600.0, 10.0);
    s_width.set_draw_value(false);
    s_width.set_value(initial_width as f64);
    s_width.set_width_request(160);
    {
        let r_c = row_width.clone();
        let sh_st = Rc::clone(&shell_state);
        s_width.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            let mut sh = sh_st.borrow_mut();
            sh.control_center.width = v;
            themes::sync_shell_config(&sh);
        });
    }
    row_width.add_suffix(&s_width);
    group_appearance.add(&row_width);

    // Maximale Hoogte Slider
    let initial_max_h = shell_state.borrow().control_center.max_height;
    let row_max_h = ActionRow::builder()
        .title(&tr.cc_max_height)
        .subtitle(format!("{} px", initial_max_h))
        .build();
    let s_max_h = Scale::with_range(Orientation::Horizontal, 300.0, 900.0, 20.0);
    s_max_h.set_draw_value(false);
    s_max_h.set_value(initial_max_h as f64);
    s_max_h.set_width_request(160);
    {
        let r_c = row_max_h.clone();
        let sh_st = Rc::clone(&shell_state);
        s_max_h.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            let mut sh = sh_st.borrow_mut();
            sh.control_center.max_height = v;
            themes::sync_shell_config(&sh);
        });
    }
    row_max_h.add_suffix(&s_max_h);
    group_appearance.add(&row_max_h);

    // Venster Afronding
    let initial_rad = shell_state.borrow().control_center.border_radius;
    let row_rad = ActionRow::builder()
        .title(&tr.cc_border_radius)
        .subtitle(format!("{} px", initial_rad))
        .build();
    let s_rad = Scale::with_range(Orientation::Horizontal, 0.0, 32.0, 1.0);
    s_rad.set_draw_value(false);
    s_rad.set_value(initial_rad as f64);
    s_rad.set_width_request(160);
    {
        let r_c = row_rad.clone();
        let sh_st = Rc::clone(&shell_state);
        s_rad.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            let mut sh = sh_st.borrow_mut();
            sh.control_center.border_radius = v;
            themes::sync_shell_config(&sh);
        });
    }
    row_rad.add_suffix(&s_rad);
    group_appearance.add(&row_rad);

    // Dekking (Opacity)
    let initial_op = shell_state.borrow().control_center.opacity;
    let row_op = ActionRow::builder()
        .title(&tr.cc_opacity)
        .subtitle(format!("{}%", (initial_op * 100.0).round() as i32))
        .build();
    let s_op = Scale::with_range(Orientation::Horizontal, 0.40, 1.00, 0.02);
    s_op.set_draw_value(false);
    s_op.set_value(initial_op);
    s_op.set_width_request(160);
    {
        let r_c = row_op.clone();
        let sh_st = Rc::clone(&shell_state);
        s_op.connect_value_changed(move |s| {
            let v = s.value();
            r_c.set_subtitle(&format!("{}%", (v * 100.0).round() as i32));
            let mut sh = sh_st.borrow_mut();
            sh.control_center.opacity = v;
            themes::sync_shell_config(&sh);
        });
    }
    row_op.add_suffix(&s_op);
    group_appearance.add(&row_op);

    // Achtergrondvervaging (Blur Behind)
    let row_blur = ActionRow::builder()
        .title(&tr.cc_blur)
        .subtitle("Schakel Wayland compositor blur achter het venster in")
        .build();
    let sw_blur = Switch::builder().active(shell_state.borrow().control_center.blur_behind).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_blur.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            sh.control_center.blur_behind = s.is_active();
            themes::sync_shell_config(&sh);
        });
    }
    row_blur.add_suffix(&sw_blur);
    group_appearance.add(&row_blur);

    page.add(&group_appearance);

    // ==========================================
    // 2. Actieve Kaarten Grid & Volgorde
    // ==========================================
    let group_cards = PreferencesGroup::builder()
        .title(&tr.cc_cards)
        .description("Sorteer, verwijder of voeg interactieve widgets toe")
        .build();

    let list_box_cards = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let list_cards_rc = Rc::new(list_box_cards);

    // Dropdown voor toevoegen
    let dd_add_card = DropDown::builder().valign(gtk4::Align::Center).build();
    let dd_add_rc = Rc::new(dd_add_card);

    let refresh_cards_picker: Rc<dyn Fn()> = {
        let sh_st = Rc::clone(&shell_state);
        let dd_ref = Rc::clone(&dd_add_rc);
        Rc::new(move || {
            let all_cards = sh_st.borrow().discover_cards_for_instance();
            let labels: Vec<String> = all_cards
                .iter()
                .map(|c| format!("{} {}", c.icon, c.name))
                .collect();
            let labels_ref: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
            let model = StringList::new(&labels_ref);
            dd_ref.set_model(Some(&model));
        })
    };

    let refresh_cards_list: Rc<dyn Fn()> = {
        let sh_st = Rc::clone(&shell_state);
        let lb_ref = Rc::clone(&list_cards_rc);
        let picker_ref = Rc::clone(&refresh_cards_picker);
        Rc::new(move || {
            refresh_cards_ui(&lb_ref, &sh_st);
            picker_ref();
        })
    };

    // Knop Kaart Toevoegen
    let row_add = ActionRow::builder()
        .title("+ Kaart Toevoegen")
        .subtitle("Kies een widget of script en plaats deze op het paneel")
        .build();
    let btn_add_card = Button::builder().label("Toevoegen").valign(gtk4::Align::Center).build();
    btn_add_card.add_css_class("suggested-action");
    {
        let sh_st = Rc::clone(&shell_state);
        let dd_ref = Rc::clone(&dd_add_rc);
        let refresh_list = Rc::clone(&refresh_cards_list);
        btn_add_card.connect_clicked(move |_| {
            let all_cards = sh_st.borrow().discover_cards_for_instance();
            let idx = dd_ref.selected() as usize;
            if let Some(card) = all_cards.get(idx) {
                let mut sh = sh_st.borrow_mut();
                sh.control_center.cards.push(card.id.clone());
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_list();
            }
        });
    }
    row_add.add_suffix(&*dd_add_rc);
    row_add.add_suffix(&btn_add_card);

    group_cards.add(&*list_cards_rc);
    group_cards.add(&row_add);
    page.add(&group_cards);

    // Initial render
    refresh_cards_list();

    // ==========================================
    // 3. Custom Script Kaart Studio
    // ==========================================
    let group_custom_card = PreferencesGroup::builder()
        .title(&tr.cc_custom_card)
        .description("Koppel een bash-commando aan een interactieve tegel in het Control Center")
        .build();

    let entry_card_name = Entry::builder().placeholder_text("Kaartnaam (bijv. GPU Temperatuur)").build();
    let row_name = ActionRow::builder().title("Naam van de Kaart").build();
    row_name.add_suffix(&entry_card_name);
    group_custom_card.add(&row_name);

    let entry_card_icon = Entry::builder().placeholder_text("Emoji (bijv. 🌡️ of 🛡️)").width_chars(10).build();
    let row_icon = ActionRow::builder().title("Icoon").build();
    row_icon.add_suffix(&entry_card_icon);
    group_custom_card.add(&row_icon);

    let entry_card_cmd = Entry::builder().placeholder_text("Commando (bijv. cat /sys/class/thermal/thermal_zone0/temp)").width_chars(30).build();
    let row_cmd = ActionRow::builder().title("Bash Commando").subtitle("Geeft de live tekstoutput voor op de tegel").build();
    row_cmd.add_suffix(&entry_card_cmd);
    group_custom_card.add(&row_cmd);

    let row_interval = ActionRow::builder().title("Polling Interval").subtitle("10 seconden").build();
    let s_interval = Scale::with_range(Orientation::Horizontal, 1.0, 300.0, 1.0);
    s_interval.set_draw_value(false);
    s_interval.set_value(10.0);
    s_interval.set_width_request(160);
    let r_int_c = row_interval.clone();
    s_interval.connect_value_changed(move |s| {
        r_int_c.set_subtitle(&format!("{} seconden", s.value().round() as u32));
    });
    row_interval.add_suffix(&s_interval);
    group_custom_card.add(&row_interval);

    let entry_card_click = Entry::builder().placeholder_text("Optioneel klik-commando (bijv. kitty -e btop)").width_chars(25).build();
    let row_click = ActionRow::builder().title("Klik Actie (Optioneel)").subtitle("Wordt uitgevoerd bij het aanklikken van de tegel").build();
    row_click.add_suffix(&entry_card_click);
    group_custom_card.add(&row_click);

    let btn_create_card = Button::builder().label(crate::ui::escape::pango_escape(&tr.cc_create_card)).valign(gtk4::Align::Center).build();
    btn_create_card.add_css_class("suggested-action");
    {
        let sh_st = Rc::clone(&shell_state);
        let e_name = entry_card_name.clone();
        let e_ico = entry_card_icon.clone();
        let e_cmd = entry_card_cmd.clone();
        let s_int = s_interval.clone();
        let e_clk = entry_card_click.clone();
        let refresh_list = Rc::clone(&refresh_cards_list);
        btn_create_card.connect_clicked(move |_| {
            let name = e_name.text().trim().to_string();
            let cmd = e_cmd.text().trim().to_string();
            if name.is_empty() || cmd.is_empty() { return; }

            let icon = if e_ico.text().trim().is_empty() { "💻".to_string() } else { e_ico.text().trim().to_string() };
            let interval = s_int.value().round() as u32;
            let click = if e_clk.text().trim().is_empty() { None } else { Some(e_clk.text().trim().to_string()) };

            let safe_id: String = name
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let id = format!("{}_{}", safe_id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));

            let script = CustomScriptModule {
                id: id.clone(),
                name,
                icon,
                command: cmd,
                interval_seconds: interval,
                on_click: click,
            };

            let mut sh = sh_st.borrow_mut();
            sh.custom_scripts.push(script);
            sh.control_center.cards.push(format!("script:{}", id));
            themes::sync_shell_config(&sh);
            drop(sh);

            e_name.set_text("");
            e_ico.set_text("");
            e_cmd.set_text("");
            e_clk.set_text("");
            refresh_list();
        });
    }
    let row_btn_save = ActionRow::builder().build();
    row_btn_save.add_suffix(&btn_create_card);
    group_custom_card.add(&row_btn_save);

    page.add(&group_custom_card);

    // ==========================================
    // 4. Power User & Test Gereedschap
    // ==========================================
    let group_power = PreferencesGroup::builder()
        .title(crate::ui::escape::pango_escape("Power User & Directe Bestanden"))
        .description("Beheer losse QML kaarten direct in je bestandssysteem of test Quickshell IPC")
        .build();

    let row_cards_dir = ActionRow::builder()
        .title("Map met Kaarten Openen")
        .subtitle("Locatie: ~/.config/quickshell/cards/")
        .build();
    let btn_open_cards = Button::builder().label("📂 Map Openen").valign(gtk4::Align::Center).build();
    btn_open_cards.connect_clicked(|_| {
        if let Ok(home) = std::env::var("HOME") {
            let dir = format!("{}/.config/quickshell/cards", home);
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
        }
    });
    row_cards_dir.add_suffix(&btn_open_cards);
    group_power.add(&row_cards_dir);

    let row_tmpl = ActionRow::builder()
        .title("Nieuwe Kaart Template Maken")
        .subtitle("Genereer CustomCard.qml en open in teksteditor")
        .build();
    let btn_tmpl = Button::builder().label("✨ Maak Template").valign(gtk4::Align::Center).build();
    {
        let refresh_list = Rc::clone(&refresh_cards_list);
        btn_tmpl.connect_clicked(move |_| {
            if let Ok(home) = std::env::var("HOME") {
                let file_path = format!("{}/.config/quickshell/cards/CustomCard.qml", home);
                if !std::path::Path::new(&file_path).exists() {
                    let tmpl = r###"import QtQuick
import QtQuick.Layouts

Rectangle {
    Layout.fillWidth: true
    implicitHeight: 60
    radius: 10
    color: Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: "#45475a"

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        Text { text: "🚀"; font.pixelSize: 18 }
        ColumnLayout {
            Text { text: "Mijn Aangepaste Kaart"; font.bold: true; color: "#cdd6f4"; font.pixelSize: 12 }
            Text { text: "Gebouwd met Quickshell & QML"; color: "#a6adc8"; font.pixelSize: 10 }
        }
    }
}
"###;
                    let _ = std::fs::write(&file_path, tmpl);
                }
                let _ = std::process::Command::new("xdg-open").arg(&file_path).spawn();
                refresh_list();
            }
        });
    }
    row_tmpl.add_suffix(&btn_tmpl);
    group_power.add(&row_tmpl);

    let row_test_ipc = ActionRow::builder()
        .title("Test Toggle Control Center")
        .subtitle("Simuleer een sneltoetsaanroep via Quickshell IPC")
        .build();
    let btn_test_ipc = Button::builder().label("🚀 Toggle Nu").valign(gtk4::Align::Center).build();
    btn_test_ipc.connect_clicked(|_| {
        let _ = std::process::Command::new("quickshell")
            .args(["ipc", "call", "controlCenter", "toggle"])
            .spawn();
    });
    row_test_ipc.add_suffix(&btn_test_ipc);
    group_power.add(&row_test_ipc);

    page.add(&group_power);

    page
}

fn refresh_cards_ui(list_box: &ListBox, shell_state: &Rc<RefCell<ZenithShellConfig>>) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let all_cards = shell_state.borrow().discover_cards_for_instance();
    let current_cards = shell_state.borrow().control_center.cards.clone();
    let total = current_cards.len();

    if total == 0 {
        let empty = ActionRow::builder()
            .title("(Geen kaarten actief)")
            .subtitle("Voeg een kaart toe via de knop hieronder")
            .build();
        list_box.append(&empty);
        return;
    }

    for (idx, card_id) in current_cards.iter().enumerate() {
        let info = all_cards.iter().find(|c| &c.id == card_id);
        let title = match info {
            Some(i) => format!("{} {}", i.icon, i.name),
            None => format!("🎴 {}", card_id),
        };
        let desc = match info {
            Some(i) => i.description.clone(),
            None => format!("Kaart ID: {}", card_id),
        };

        let row = ActionRow::builder()
            .title(crate::ui::escape::pango_escape(&title))
            .subtitle(crate::ui::escape::pango_escape(&desc))
            .build();

        // Knop Omhoog (▲)
        if idx > 0 {
            let btn_up = Button::builder().label("▲").valign(gtk4::Align::Center).tooltip_text("Omhoog verplaatsen").build();
            let sh_st = Rc::clone(shell_state);
            let lb = list_box.clone();
            btn_up.connect_clicked(move |_| {
                let mut sh = sh_st.borrow_mut();
                if idx > 0 && idx < sh.control_center.cards.len() {
                    sh.control_center.cards.swap(idx, idx - 1);
                }
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_cards_ui(&lb, &sh_st);
            });
            row.add_suffix(&btn_up);
        }

        // Knop Omlaag (▼)
        if idx + 1 < total {
            let btn_down = Button::builder().label("▼").valign(gtk4::Align::Center).tooltip_text("Omlaag verplaatsen").build();
            let sh_st = Rc::clone(shell_state);
            let lb = list_box.clone();
            btn_down.connect_clicked(move |_| {
                let mut sh = sh_st.borrow_mut();
                if idx + 1 < sh.control_center.cards.len() {
                    sh.control_center.cards.swap(idx, idx + 1);
                }
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_cards_ui(&lb, &sh_st);
            });
            row.add_suffix(&btn_down);
        }

        // Knop Verwijderen (✖)
        let btn_remove = Button::builder().label("✖").valign(gtk4::Align::Center).tooltip_text("Van paneel verwijderen").build();
        btn_remove.add_css_class("destructive-action");
        {
            let sh_st = Rc::clone(shell_state);
            let lb = list_box.clone();
            btn_remove.connect_clicked(move |_| {
                let mut sh = sh_st.borrow_mut();
                if idx < sh.control_center.cards.len() {
                    sh.control_center.cards.remove(idx);
                }
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_cards_ui(&lb, &sh_st);
            });
        }
        row.add_suffix(&btn_remove);

        list_box.append(&row);
    }
}
