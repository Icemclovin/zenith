use gtk4::prelude::*;
use gtk4::{
    Button, DropDown, Entry, ListBox, Orientation, Scale, StringList, Switch,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::config::{self, ZenithConfig};
use crate::backend::i18n::Translations;
use crate::backend::shell_config::{CustomScriptModule, ZenithShellConfig};
use crate::backend::{process, themes};

pub fn build_quickshell_page(state: &Rc<RefCell<ZenithConfig>>, tr: &Translations) -> PreferencesPage {
    let page = PreferencesPage::new();
    let shell_state = Rc::new(RefCell::new(ZenithShellConfig::load_or_default()));

    // 1. Thema Presets (1-Klik Stijlen)
    let group_presets = PreferencesGroup::builder()
        .title(&tr.bar_preset_theme)
        .description("Pas direct een complete vooraf ontworpen balkstijl en module-indeling toe")
        .build();

    let row_preset = ActionRow::builder()
        .title("Kies een Thema Preset")
        .subtitle("Selecteer een vooraf geconfigureerde stijl en klik op Toepassen")
        .build();

    let preset_model = StringList::new(&[
        "🌟 Zenith Modern (Zwevende balk, Catppuccin)",
        "🏝️ Dynamic Islands (Losse capsule-eilanden)",
        "🌌 Cyber Neon (Diep donker met neon accenten)",
        "🍃 Nord Frost (Scandinavisch koel & clean)",
        "🍎 Cupertino Dock (Onderaan het scherm)",
        "⚡ Compact Edge (Bovenaan schermbreed & strak)",
    ]);
    let dd_preset = DropDown::builder().model(&preset_model).valign(gtk4::Align::Center).build();
    let btn_apply_preset = Button::builder().label(&tr.bar_apply_preset).valign(gtk4::Align::Center).build();

    // Boxed list containers for modules
    let list_box_left = ListBox::builder().selection_mode(gtk4::SelectionMode::None).css_classes(["boxed-list"]).build();
    let list_box_center = ListBox::builder().selection_mode(gtk4::SelectionMode::None).css_classes(["boxed-list"]).build();
    let list_box_right = ListBox::builder().selection_mode(gtk4::SelectionMode::None).css_classes(["boxed-list"]).build();

    let list_left_rc = Rc::new(list_box_left);
    let list_center_rc = Rc::new(list_box_center);
    let list_right_rc = Rc::new(list_box_right);

    // Closure to refresh all module lists
    let refresh_all_modules: Rc<dyn Fn()> = {
        let shell_state = Rc::clone(&shell_state);
        let list_l = Rc::clone(&list_left_rc);
        let list_c = Rc::clone(&list_center_rc);
        let list_r = Rc::clone(&list_right_rc);
        Rc::new(move || {
            refresh_slot(&list_l, "left", &shell_state);
            refresh_slot(&list_c, "center", &shell_state);
            refresh_slot(&list_r, "right", &shell_state);
        })
    };

    let picker_dropdowns: Rc<RefCell<Vec<DropDown>>> = Rc::new(RefCell::new(Vec::new()));
    let refresh_picker_models: Rc<dyn Fn()> = {
        let pickers = Rc::clone(&picker_dropdowns);
        let sh_st = Rc::clone(&shell_state);
        Rc::new(move || {
            let all_modules = sh_st.borrow().discover_modules_for_instance();
            let labels: Vec<String> = all_modules
                .iter()
                .map(|m| format!("{} {}", m.icon, m.name))
                .collect();
            let labels_ref: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
            for dd in pickers.borrow().iter() {
                let model = StringList::new(&labels_ref);
                dd.set_model(Some(&model));
            }
        })
    };

    // Apply preset handler
    {
        let st = Rc::clone(state);
        let sh_st = Rc::clone(&shell_state);
        let dd = dd_preset.clone();
        let refresh_mod = Rc::clone(&refresh_all_modules);
        let refresh_pick = Rc::clone(&refresh_picker_models);
        btn_apply_preset.connect_clicked(move |_| {
            let idx = dd.selected() as usize;
            let mut cfg = st.borrow_mut();
            let mut sh = sh_st.borrow_mut();

            match idx {
                1 => { // Dynamic Islands
                    sh.layout.bar_style = "islands".to_string();
                    sh.layout.position = "top".to_string();
                    sh.layout.height = 38;
                    sh.layout.floating = true;
                    sh.styling.background = "#1e1e2e".to_string();
                    sh.styling.accent = "#cba6f7".to_string();
                    sh.styling.pill_bg = "#11111b".to_string();
                    sh.styling.text_color = "#cdd6f4".to_string();
                    sh.styling.border_color = "#585b70".to_string();
                    sh.styling.rounding = 14;
                    sh.styling.pill_rounding = 10;
                    sh.styling.margin_h = 10;
                    sh.styling.margin_v = 8;
                    sh.styling.border_width = 1;
                    sh.styling.opacity = 0.80;
                    sh.styling.pill_opacity = 0.88;
                    sh.styling.font_size = 11;
                    sh.styling.module_spacing = 10;
                    sh.modules.left = vec!["launcher".to_string(), "workspaces".to_string()];
                    sh.modules.center = vec!["active_window".to_string(), "media".to_string()];
                    sh.modules.right = vec!["volume".to_string(), "brightness".to_string(), "network".to_string(), "battery".to_string(), "systray".to_string(), "clock".to_string(), "power".to_string()];
                }
                2 => { // Cyber Neon
                    sh.layout.bar_style = "islands".to_string();
                    sh.layout.position = "top".to_string();
                    sh.layout.height = 40;
                    sh.layout.floating = true;
                    sh.styling.background = "#0a0a14".to_string();
                    sh.styling.accent = "#00ffff".to_string();
                    sh.styling.pill_bg = "#101026".to_string();
                    sh.styling.text_color = "#00ffcc".to_string();
                    sh.styling.border_color = "#ff007f".to_string();
                    sh.styling.rounding = 16;
                    sh.styling.pill_rounding = 10;
                    sh.styling.margin_h = 12;
                    sh.styling.margin_v = 8;
                    sh.styling.border_width = 2;
                    sh.styling.opacity = 0.92;
                    sh.styling.pill_opacity = 0.90;
                    sh.styling.font_size = 12;
                    sh.styling.module_spacing = 12;
                    sh.modules.left = vec!["launcher".to_string(), "brand".to_string(), "workspaces".to_string()];
                    sh.modules.center = vec!["active_window".to_string(), "media".to_string()];
                    sh.modules.right = vec!["cpu".to_string(), "ram".to_string(), "volume".to_string(), "network".to_string(), "clock".to_string(), "power".to_string()];
                }
                3 => { // Nord Frost
                    sh.layout.bar_style = "unified".to_string();
                    sh.layout.position = "top".to_string();
                    sh.layout.height = 36;
                    sh.layout.floating = true;
                    sh.styling.background = "#2e3440".to_string();
                    sh.styling.accent = "#88c0d0".to_string();
                    sh.styling.pill_bg = "#3b4252".to_string();
                    sh.styling.text_color = "#eceff4".to_string();
                    sh.styling.border_color = "#4c566a".to_string();
                    sh.styling.rounding = 10;
                    sh.styling.pill_rounding = 8;
                    sh.styling.margin_h = 8;
                    sh.styling.margin_v = 6;
                    sh.styling.border_width = 1;
                    sh.styling.opacity = 0.95;
                    sh.styling.pill_opacity = 0.90;
                    sh.styling.font_size = 11;
                    sh.styling.module_spacing = 8;
                    sh.modules.left = vec!["workspaces".to_string()];
                    sh.modules.center = vec!["active_window".to_string()];
                    sh.modules.right = vec!["volume".to_string(), "brightness".to_string(), "network".to_string(), "battery".to_string(), "clock".to_string(), "power".to_string()];
                }
                4 => { // Cupertino Dock
                    sh.layout.bar_style = "unified".to_string();
                    sh.layout.position = "bottom".to_string();
                    sh.layout.height = 44;
                    sh.layout.floating = true;
                    sh.styling.background = "#1c1c1e".to_string();
                    sh.styling.accent = "#0a84ff".to_string();
                    sh.styling.pill_bg = "#2c2c2e".to_string();
                    sh.styling.text_color = "#ffffff".to_string();
                    sh.styling.border_color = "#3a3a3c".to_string();
                    sh.styling.rounding = 20;
                    sh.styling.pill_rounding = 12;
                    sh.styling.margin_h = 18;
                    sh.styling.margin_v = 10;
                    sh.styling.border_width = 1;
                    sh.styling.opacity = 0.75;
                    sh.styling.pill_opacity = 0.80;
                    sh.styling.font_size = 12;
                    sh.styling.module_spacing = 10;
                    sh.modules.left = vec!["launcher".to_string(), "brand".to_string()];
                    sh.modules.center = vec!["workspaces".to_string(), "active_window".to_string()];
                    sh.modules.right = vec!["volume".to_string(), "brightness".to_string(), "systray".to_string(), "clock".to_string(), "power".to_string()];
                }
                5 => { // Compact Edge
                    sh.layout.bar_style = "unified".to_string();
                    sh.layout.position = "top".to_string();
                    sh.layout.height = 28;
                    sh.layout.floating = false;
                    sh.styling.background = "#181825".to_string();
                    sh.styling.accent = "#89dceb".to_string();
                    sh.styling.pill_bg = "#11111b".to_string();
                    sh.styling.text_color = "#cdd6f4".to_string();
                    sh.styling.border_color = "#313244".to_string();
                    sh.styling.rounding = 0;
                    sh.styling.pill_rounding = 4;
                    sh.styling.margin_h = 0;
                    sh.styling.margin_v = 0;
                    sh.styling.border_width = 0;
                    sh.styling.opacity = 1.0;
                    sh.styling.pill_opacity = 1.0;
                    sh.styling.font_size = 10;
                    sh.styling.module_spacing = 6;
                    sh.modules.left = vec!["workspaces".to_string()];
                    sh.modules.center = vec!["active_window".to_string()];
                    sh.modules.right = vec!["cpu".to_string(), "ram".to_string(), "volume".to_string(), "clock".to_string(), "power".to_string()];
                }
                _ => { // Zenith Modern (Default)
                    sh.layout.bar_style = "unified".to_string();
                    sh.layout.position = "top".to_string();
                    sh.layout.height = 38;
                    sh.layout.floating = true;
                    sh.styling.background = "#1e1e2e".to_string();
                    sh.styling.accent = "#89b4fa".to_string();
                    sh.styling.pill_bg = "#181825".to_string();
                    sh.styling.text_color = "#cdd6f4".to_string();
                    sh.styling.border_color = "#45475a".to_string();
                    sh.styling.rounding = 12;
                    sh.styling.pill_rounding = 8;
                    sh.styling.margin_h = 8;
                    sh.styling.margin_v = 6;
                    sh.styling.border_width = 1;
                    sh.styling.opacity = 0.90;
                    sh.styling.pill_opacity = 0.85;
                    sh.styling.font_size = 11;
                    sh.styling.module_spacing = 8;
                    sh.modules.left = vec!["launcher".to_string(), "brand".to_string(), "workspaces".to_string()];
                    sh.modules.center = vec!["active_window".to_string(), "media".to_string()];
                    sh.modules.right = vec!["cpu".to_string(), "ram".to_string(), "battery".to_string(), "volume".to_string(), "brightness".to_string(), "network".to_string(), "bluetooth".to_string(), "systray".to_string(), "clock".to_string(), "power".to_string()];
                }
            }

            // Sync to legacy ZenithConfig as well
            cfg.qs_bar_style = sh.layout.bar_style.clone();
            cfg.qs_position = sh.layout.position.clone();
            cfg.qs_height = sh.layout.height;
            cfg.qs_floating = sh.layout.floating;
            cfg.qs_bg = sh.styling.background.trim_start_matches('#').to_string();
            cfg.qs_accent = sh.styling.accent.trim_start_matches('#').to_string();
            cfg.qs_pill_bg = sh.styling.pill_bg.trim_start_matches('#').to_string();
            cfg.qs_text_color = sh.styling.text_color.trim_start_matches('#').to_string();
            cfg.qs_border_color = sh.styling.border_color.trim_start_matches('#').to_string();
            cfg.qs_rounding = sh.styling.rounding;
            cfg.qs_pill_rounding = sh.styling.pill_rounding;
            cfg.qs_margin_h = sh.styling.margin_h;
            cfg.qs_margin_v = sh.styling.margin_v;
            cfg.qs_border_width = sh.styling.border_width;
            cfg.qs_opacity = sh.styling.opacity;
            cfg.qs_pill_opacity = sh.styling.pill_opacity;
            cfg.qs_font_size = sh.styling.font_size;
            cfg.qs_module_spacing = sh.styling.module_spacing;

            themes::sync_shell_config(&sh);
            config::save_config(&cfg);
            refresh_mod();
            refresh_pick();
        });
    }

    row_preset.add_suffix(&dd_preset);
    row_preset.add_suffix(&btn_apply_preset);
    group_presets.add(&row_preset);
    page.add(&group_presets);

    // ========================================================
    // 2. Statusbar Designer: Modulaire Indeling (Links/Midden/Rechts)
    // ========================================================
    let group_modules = PreferencesGroup::builder()
        .title("Statusbar Designer: Modulaire Indeling")
        .description("Bepaal exact welke modules op de statusbalk verschijnen en in welke volgorde. Custom widgets worden automatisch gedetecteerd!")
        .build();
    page.add(&group_modules);

    // Subgroep: Links
    let group_mod_left = PreferencesGroup::builder()
        .title(&tr.bar_left_modules)
        .description("Modules aan de linkerzijde van de statusbalk")
        .build();
    group_mod_left.add(&*list_left_rc);
    add_module_picker_row(&group_mod_left, "left", &shell_state, &list_left_rc, &picker_dropdowns);
    page.add(&group_mod_left);

    // Subgroep: Midden
    let group_mod_center = PreferencesGroup::builder()
        .title(&tr.bar_center_modules)
        .description("Modules gecentreerd op de statusbalk")
        .build();
    group_mod_center.add(&*list_center_rc);
    add_module_picker_row(&group_mod_center, "center", &shell_state, &list_center_rc, &picker_dropdowns);
    page.add(&group_mod_center);

    // Subgroep: Rechts
    let group_mod_right = PreferencesGroup::builder()
        .title(&tr.bar_right_modules)
        .description("Modules aan de rechterzijde van de statusbalk")
        .build();
    group_mod_right.add(&*list_right_rc);
    add_module_picker_row(&group_mod_right, "right", &shell_state, &list_right_rc, &picker_dropdowns);
    page.add(&group_mod_right);

    // Eerste vulling van de modulelijsten
    refresh_all_modules();

    // ========================================================
    // 3. Custom Commando / Script Modules Studio
    // ========================================================
    let group_custom_scripts = PreferencesGroup::builder()
        .title(&tr.bar_custom_script)
        .description("Maak en beheer eigen statusbalk-modules op basis van bash-commando's zonder QML te hoeven schrijven")
        .build();

    let list_box_scripts = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let list_scripts_rc = Rc::new(list_box_scripts);
    group_custom_scripts.add(&*list_scripts_rc);

    let refresh_scripts: Rc<dyn Fn()> = {
        let lb_s = Rc::clone(&list_scripts_rc);
        let sh_st = Rc::clone(&shell_state);
        let ref_all = Rc::clone(&refresh_all_modules);
        let ref_pick = Rc::clone(&refresh_picker_models);
        Rc::new(move || {
            refresh_scripts_list(&lb_s, &sh_st, &ref_all, &ref_pick);
        })
    };

    // Vulling van scriptlijst
    refresh_scripts();

    // Formulier voor nieuwe scriptmodule
    let row_cs_name = ActionRow::builder()
        .title(crate::ui::escape::pango_escape("Modulenaam & Icoon"))
        .subtitle("Geef je module een herkenbare naam en emoji-icoon")
        .build();
    let ent_cs_name = Entry::builder()
        .placeholder_text("Bv. Spotify of Bitcoin")
        .width_chars(16)
        .valign(gtk4::Align::Center)
        .build();
    let ent_cs_icon = Entry::builder()
        .placeholder_text("Icoon (bv. 🎵)")
        .width_chars(8)
        .valign(gtk4::Align::Center)
        .build();
    row_cs_name.add_suffix(&ent_cs_name);
    row_cs_name.add_suffix(&ent_cs_icon);
    group_custom_scripts.add(&row_cs_name);

    let row_cs_cmd = ActionRow::builder()
        .title("Shell Commando (Output)")
        .subtitle("Het bash-commando waarvan de output live op de statusbalk verschijnt")
        .build();
    let ent_cs_cmd = Entry::builder()
        .placeholder_text("Bv. playerctl metadata --format '{{title}}' || echo ''")
        .width_chars(28)
        .valign(gtk4::Align::Center)
        .build();
    row_cs_cmd.add_suffix(&ent_cs_cmd);
    group_custom_scripts.add(&row_cs_cmd);

    let row_cs_int = ActionRow::builder()
        .title("Update Interval")
        .subtitle("5 seconden")
        .build();
    let s_cs_int = Scale::with_range(Orientation::Horizontal, 1.0, 300.0, 1.0);
    s_cs_int.set_draw_value(false);
    s_cs_int.set_value(5.0);
    s_cs_int.set_width_request(160);
    {
        let r_c = row_cs_int.clone();
        s_cs_int.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} seconden", v));
        });
    }
    row_cs_int.add_suffix(&s_cs_int);
    group_custom_scripts.add(&row_cs_int);

    let row_cs_click = ActionRow::builder()
        .title("Actie bij Klik (On-Click)")
        .subtitle("Optioneel commando dat uitgevoerd wordt bij een klik op het widget")
        .build();
    let ent_cs_click = Entry::builder()
        .placeholder_text("Bv. alacritty -e btop of playerctl play-pause")
        .width_chars(28)
        .valign(gtk4::Align::Center)
        .build();
    row_cs_click.add_suffix(&ent_cs_click);
    group_custom_scripts.add(&row_cs_click);

    let row_cs_save = ActionRow::builder()
        .title("Nieuwe Scriptmodule Opslaan")
        .subtitle("Sla op in zenith-shell.json en maak direct beschikbaar in de modulekiezers hierboven")
        .build();
    let btn_save_script = Button::builder().label("➕ Scriptmodule Aanmaken").valign(gtk4::Align::Center).build();
    btn_save_script.add_css_class("suggested-action");
    {
        let sh_st = Rc::clone(&shell_state);
        let ref_scr = Rc::clone(&refresh_scripts);
        let ref_pick = Rc::clone(&refresh_picker_models);
        let e_name = ent_cs_name.clone();
        let e_icon = ent_cs_icon.clone();
        let e_cmd = ent_cs_cmd.clone();
        let e_click = ent_cs_click.clone();
        let s_int = s_cs_int.clone();

        btn_save_script.connect_clicked(move |_| {
            let name_raw = e_name.text().trim().to_string();
            let cmd_raw = e_cmd.text().trim().to_string();
            if cmd_raw.is_empty() {
                return;
            }
            let icon_raw = e_icon.text().trim().to_string();
            let icon = if icon_raw.is_empty() { "💻".to_string() } else { icon_raw };
            let name = if name_raw.is_empty() { "Custom Script".to_string() } else { name_raw };

            let mut sh = sh_st.borrow_mut();

            // Genereer veilige unieke ID
            let base_id: String = name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect();
            let trimmed_id = base_id.trim_matches('_').to_string();
            let start_id = if trimmed_id.is_empty() { "custom_script".to_string() } else { trimmed_id };

            let mut final_id = start_id.clone();
            let mut counter = 1;
            while sh.custom_scripts.iter().any(|s| s.id == final_id) {
                final_id = format!("{}_{}", start_id, counter);
                counter += 1;
            }

            let interval_sec = s_int.value().round() as u32;
            let onclick_raw = e_click.text().trim().to_string();
            let on_click = if onclick_raw.is_empty() { None } else { Some(onclick_raw) };

            let new_script = CustomScriptModule {
                id: final_id,
                name,
                icon,
                command: cmd_raw,
                interval_seconds: interval_sec,
                on_click,
            };

            sh.custom_scripts.push(new_script);
            themes::sync_shell_config(&sh);
            drop(sh);

            // Reset invoervelden
            e_name.set_text("");
            e_icon.set_text("");
            e_cmd.set_text("");
            e_click.set_text("");
            s_int.set_value(5.0);

            // Refresh UI
            ref_scr();
            ref_pick();
        });
    }
    row_cs_save.add_suffix(&btn_save_script);
    group_custom_scripts.add(&row_cs_save);

    page.add(&group_custom_scripts);

    // ========================================================
    // 4. Geometrie & Positie
    // ========================================================
    let group_geom = PreferencesGroup::builder()
        .title(&tr.bar_styling)
        .description("Afmetingen, architectuur, afronding en zwevende marges")
        .build();

    let initial_sh = shell_state.borrow().clone();

    // Balk Architectuur
    let row_style = ActionRow::builder()
        .title(&tr.bar_style)
        .subtitle("Kies tussen één doorlopende balk (Unified), losse eilanden (Islands) of Minimal")
        .build();
    let style_model = StringList::new(&["Eén Geheel (Unified)", "Losse Eilanden (Islands)", "Minimalistisch (Minimal)"]);
    let dd_style = DropDown::builder().model(&style_model).valign(gtk4::Align::Center).build();
    dd_style.set_selected(match initial_sh.layout.bar_style.as_str() { "islands" => 1, "minimal" => 2, _ => 0 });
    {
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        dd_style.connect_selected_notify(move |d| {
            let val = match d.selected() { 1 => "islands", 2 => "minimal", _ => "unified" };
            sh_st.borrow_mut().layout.bar_style = val.to_string();
            st.borrow_mut().qs_bar_style = val.to_string();
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_style.add_suffix(&dd_style);
    group_geom.add(&row_style);

    // Positie
    let row_pos = ActionRow::builder().title(&tr.bar_position).build();
    let pos_model = StringList::new(&["Bovenaan (top)", "Onderaan (bottom)", "Links (left)", "Rechts (right)"]);
    let dd_pos = DropDown::builder().model(&pos_model).valign(gtk4::Align::Center).build();
    dd_pos.set_selected(match initial_sh.layout.position.as_str() {
        "bottom" => 1,
        "left" => 2,
        "right" => 3,
        _ => 0,
    });
    {
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        dd_pos.connect_selected_notify(move |d| {
            let pos_str = match d.selected() {
                1 => "bottom",
                2 => "left",
                3 => "right",
                _ => "top",
            };
            sh_st.borrow_mut().layout.position = pos_str.to_string();
            st.borrow_mut().qs_position = pos_str.to_string();
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_pos.add_suffix(&dd_pos);
    group_geom.add(&row_pos);

    // Hoogte
    let row_h = ActionRow::builder()
        .title(&tr.bar_height)
        .subtitle(format!("{} px", initial_sh.layout.height))
        .build();
    let s_h = Scale::with_range(Orientation::Horizontal, 20.0, 64.0, 2.0);
    s_h.set_draw_value(false);
    s_h.set_value(initial_sh.layout.height as f64);
    s_h.set_width_request(160);
    {
        let r_c = row_h.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_h.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            sh_st.borrow_mut().layout.height = v;
            st.borrow_mut().qs_height = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_h.add_suffix(&s_h);
    group_geom.add(&row_h);

    // Zwevende Modus
    let row_flt = ActionRow::builder()
        .title(&tr.bar_floating)
        .subtitle("Balk losmaken van de beeldschermrand met marges")
        .build();
    let sw_flt = Switch::builder().active(initial_sh.layout.floating).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        sw_flt.connect_state_set(move |_, active| {
            sh_st.borrow_mut().layout.floating = active;
            st.borrow_mut().qs_floating = active;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
            gtk4::glib::Propagation::Proceed
        });
    }
    row_flt.add_suffix(&sw_flt);
    group_geom.add(&row_flt);

    // Afronding
    let row_rnd = ActionRow::builder()
        .title("Balkafronding (Radius)")
        .subtitle(format!("{} px", initial_sh.styling.rounding))
        .build();
    let s_rnd = Scale::with_range(Orientation::Horizontal, 0.0, 32.0, 1.0);
    s_rnd.set_draw_value(false);
    s_rnd.set_value(initial_sh.styling.rounding as f64);
    s_rnd.set_width_request(160);
    {
        let r_c = row_rnd.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_rnd.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            sh_st.borrow_mut().styling.rounding = v;
            st.borrow_mut().qs_rounding = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_rnd.add_suffix(&s_rnd);
    group_geom.add(&row_rnd);

    // Eiland Afronding
    let row_prnd = ActionRow::builder()
        .title("Eiland / Capsule Afronding")
        .subtitle(format!("{} px", initial_sh.styling.pill_rounding))
        .build();
    let s_prnd = Scale::with_range(Orientation::Horizontal, 0.0, 24.0, 1.0);
    s_prnd.set_draw_value(false);
    s_prnd.set_value(initial_sh.styling.pill_rounding as f64);
    s_prnd.set_width_request(160);
    {
        let r_c = row_prnd.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_prnd.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            sh_st.borrow_mut().styling.pill_rounding = v;
            st.borrow_mut().qs_pill_rounding = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_prnd.add_suffix(&s_prnd);
    group_geom.add(&row_prnd);

    // Horizontale Marge
    let row_mh = ActionRow::builder()
        .title("Horizontale Marge (Zwevend)")
        .subtitle(format!("{} px", initial_sh.styling.margin_h))
        .build();
    let s_mh = Scale::with_range(Orientation::Horizontal, 0.0, 32.0, 1.0);
    s_mh.set_draw_value(false);
    s_mh.set_value(initial_sh.styling.margin_h as f64);
    s_mh.set_width_request(160);
    {
        let r_c = row_mh.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_mh.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            sh_st.borrow_mut().styling.margin_h = v;
            st.borrow_mut().qs_margin_h = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_mh.add_suffix(&s_mh);
    group_geom.add(&row_mh);

    // Verticale Marge
    let row_mv = ActionRow::builder()
        .title("Verticale Marge (Zwevend)")
        .subtitle(format!("{} px", initial_sh.styling.margin_v))
        .build();
    let s_mv = Scale::with_range(Orientation::Horizontal, 0.0, 24.0, 1.0);
    s_mv.set_draw_value(false);
    s_mv.set_value(initial_sh.styling.margin_v as f64);
    s_mv.set_width_request(160);
    {
        let r_c = row_mv.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_mv.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            sh_st.borrow_mut().styling.margin_v = v;
            st.borrow_mut().qs_margin_v = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_mv.add_suffix(&s_mv);
    group_geom.add(&row_mv);

    // Module Spacing
    let row_sp = ActionRow::builder()
        .title("Afstand Tussen Modules (Spacing)")
        .subtitle(format!("{} px", initial_sh.styling.module_spacing))
        .build();
    let s_sp = Scale::with_range(Orientation::Horizontal, 2.0, 24.0, 1.0);
    s_sp.set_draw_value(false);
    s_sp.set_value(initial_sh.styling.module_spacing as f64);
    s_sp.set_width_request(160);
    {
        let r_c = row_sp.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_sp.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            sh_st.borrow_mut().styling.module_spacing = v;
            st.borrow_mut().qs_module_spacing = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_sp.add_suffix(&s_sp);
    group_geom.add(&row_sp);

    // Randdikte
    let row_bw = ActionRow::builder()
        .title("Randdikte (Border)")
        .subtitle(format!("{} px", initial_sh.styling.border_width))
        .build();
    let s_bw = Scale::with_range(Orientation::Horizontal, 0.0, 6.0, 1.0);
    s_bw.set_draw_value(false);
    s_bw.set_value(initial_sh.styling.border_width as f64);
    s_bw.set_width_request(160);
    {
        let r_c = row_bw.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_bw.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} px", v));
            sh_st.borrow_mut().styling.border_width = v;
            st.borrow_mut().qs_border_width = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_bw.add_suffix(&s_bw);
    group_geom.add(&row_bw);

    page.add(&group_geom);

    // ========================================================
    // 4. Kleur & Transparantie
    // ========================================================
    let group_colors = PreferencesGroup::builder()
        .title(crate::ui::escape::pango_escape("Kleuren & Transparantie"))
        .description("Achtergronden, contrast, accenten en lettergrootte")
        .build();

    // Achtergrondkleur
    let row_bg = ActionRow::builder().title("Achtergrondkleur Balk").subtitle("Hex-code of kleurkiezer").build();
    let ent_bg = Entry::builder().text(&initial_sh.styling.background).width_chars(10).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        ent_bg.connect_changed(move |e| {
            let val = e.text().to_string();
            sh_st.borrow_mut().styling.background = val.clone();
            st.borrow_mut().qs_bg = val.trim_start_matches('#').to_string();
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_bg.add_suffix(&ent_bg);
    group_colors.add(&row_bg);

    // Transparantie Balk
    let row_op = ActionRow::builder()
        .title("Balk Dekking (Opacity)")
        .subtitle(format!("{:.0}%", initial_sh.styling.opacity * 100.0))
        .build();
    let s_op = Scale::with_range(Orientation::Horizontal, 0.10, 1.0, 0.05);
    s_op.set_draw_value(false);
    s_op.set_value(initial_sh.styling.opacity);
    s_op.set_width_request(160);
    {
        let r_c = row_op.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_op.connect_value_changed(move |s| {
            let v = s.value();
            r_c.set_subtitle(&format!("{:.0}%", v * 100.0));
            sh_st.borrow_mut().styling.opacity = v;
            st.borrow_mut().qs_opacity = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_op.add_suffix(&s_op);
    group_colors.add(&row_op);

    // Eiland Achtergrondkleur
    let row_pbg = ActionRow::builder().title("Eiland Achtergrond (Pill BG)").subtitle("Hex-kleur van de capsule-eilanden").build();
    let ent_pbg = Entry::builder().text(&initial_sh.styling.pill_bg).width_chars(10).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        ent_pbg.connect_changed(move |e| {
            let val = e.text().to_string();
            sh_st.borrow_mut().styling.pill_bg = val.clone();
            st.borrow_mut().qs_pill_bg = val.trim_start_matches('#').to_string();
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_pbg.add_suffix(&ent_pbg);
    group_colors.add(&row_pbg);

    // Eiland Transparantie
    let row_pop = ActionRow::builder()
        .title("Eiland Dekking (Pill Opacity)")
        .subtitle(format!("{:.0}%", initial_sh.styling.pill_opacity * 100.0))
        .build();
    let s_pop = Scale::with_range(Orientation::Horizontal, 0.10, 1.0, 0.05);
    s_pop.set_draw_value(false);
    s_pop.set_value(initial_sh.styling.pill_opacity);
    s_pop.set_width_request(160);
    {
        let r_c = row_pop.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_pop.connect_value_changed(move |s| {
            let v = s.value();
            r_c.set_subtitle(&format!("{:.0}%", v * 100.0));
            sh_st.borrow_mut().styling.pill_opacity = v;
            st.borrow_mut().qs_pill_opacity = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_pop.add_suffix(&s_pop);
    group_colors.add(&row_pop);

    // Accentkleur
    let row_acc = ActionRow::builder().title("Accentkleur").subtitle("Kleur voor actieve werkbladen, iconen en badges").build();
    let ent_acc = Entry::builder().text(&initial_sh.styling.accent).width_chars(10).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        ent_acc.connect_changed(move |e| {
            let val = e.text().to_string();
            sh_st.borrow_mut().styling.accent = val.clone();
            st.borrow_mut().qs_accent = val.trim_start_matches('#').to_string();
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_acc.add_suffix(&ent_acc);
    group_colors.add(&row_acc);

    // Tekstkleur
    let row_tc = ActionRow::builder().title(crate::ui::escape::pango_escape("Tekst- & Icoonkleur")).subtitle("Voorgrondkleur voor tekstlabels").build();
    let ent_tc = Entry::builder().text(&initial_sh.styling.text_color).width_chars(10).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        ent_tc.connect_changed(move |e| {
            let val = e.text().to_string();
            sh_st.borrow_mut().styling.text_color = val.clone();
            st.borrow_mut().qs_text_color = val.trim_start_matches('#').to_string();
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_tc.add_suffix(&ent_tc);
    group_colors.add(&row_tc);

    // Randkleur
    let row_bc = ActionRow::builder().title("Randkleur (Border)").subtitle("Kleur van omlijning van balk en modules").build();
    let ent_bc = Entry::builder().text(&initial_sh.styling.border_color).width_chars(10).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        ent_bc.connect_changed(move |e| {
            let val = e.text().to_string();
            sh_st.borrow_mut().styling.border_color = val.clone();
            st.borrow_mut().qs_border_color = val.trim_start_matches('#').to_string();
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_bc.add_suffix(&ent_bc);
    group_colors.add(&row_bc);

    // Lettergrootte
    let row_fs = ActionRow::builder()
        .title("Lettergrootte (Font Size)")
        .subtitle(format!("{} pt", initial_sh.styling.font_size))
        .build();
    let s_fs = Scale::with_range(Orientation::Horizontal, 8.0, 18.0, 1.0);
    s_fs.set_draw_value(false);
    s_fs.set_value(initial_sh.styling.font_size as f64);
    s_fs.set_width_request(160);
    {
        let r_c = row_fs.clone();
        let sh_st = Rc::clone(&shell_state);
        let st = Rc::clone(state);
        s_fs.connect_value_changed(move |s| {
            let v = s.value().round() as i32;
            r_c.set_subtitle(&format!("{} pt", v));
            sh_st.borrow_mut().styling.font_size = v;
            st.borrow_mut().qs_font_size = v;
            themes::sync_shell_config(&sh_st.borrow());
            config::save_config(&st.borrow());
        });
    }
    row_fs.add_suffix(&s_fs);
    group_colors.add(&row_fs);

    page.add(&group_colors);

    // ========================================================
    // 5. Panels & On-Screen Displays (OSD)
    // ========================================================
    let group_panels = PreferencesGroup::builder()
        .title(crate::ui::escape::pango_escape("Panels & On-Screen Displays (OSD)"))
        .description("Schakel dynamische overlays, popups en systeemnotificaties in of uit")
        .build();

    let row_p_qs = ActionRow::builder()
        .title("Quick Settings Paneel")
        .subtitle("Geïntegreerd Quickshell popup-paneel voor audio, netwerk en helderheid")
        .build();
    let sw_p_qs = Switch::builder().active(initial_sh.panels.quick_settings).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_p_qs.connect_state_set(move |_, active| {
            sh_st.borrow_mut().panels.quick_settings = active;
            themes::sync_shell_config(&sh_st.borrow());
            gtk4::glib::Propagation::Proceed
        });
    }
    row_p_qs.add_suffix(&sw_p_qs);
    group_panels.add(&row_p_qs);

    let row_p_cc = ActionRow::builder().title("Control Center Integratie").subtitle("Toegang tot snelinstellingen via quickshell").build();
    let sw_p_cc = Switch::builder().active(initial_sh.panels.control_center).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_p_cc.connect_state_set(move |_, active| {
            sh_st.borrow_mut().panels.control_center = active;
            themes::sync_shell_config(&sh_st.borrow());
            gtk4::glib::Propagation::Proceed
        });
    }
    row_p_cc.add_suffix(&sw_p_cc);
    group_panels.add(&row_p_cc);

    let row_p_vosd = ActionRow::builder().title("Volume OSD Overlay").subtitle("Toon een floating overlay indicator bij volumewijzigingen").build();
    let sw_p_vosd = Switch::builder().active(initial_sh.panels.volume_osd).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_p_vosd.connect_state_set(move |_, active| {
            sh_st.borrow_mut().panels.volume_osd = active;
            themes::sync_shell_config(&sh_st.borrow());
            gtk4::glib::Propagation::Proceed
        });
    }
    row_p_vosd.add_suffix(&sw_p_vosd);
    group_panels.add(&row_p_vosd);

    let row_p_bosd = ActionRow::builder().title("Helderheid OSD Overlay").subtitle("Toon een floating overlay bij het verstellen van de helderheid").build();
    let sw_p_bosd = Switch::builder().active(initial_sh.panels.brightness_osd).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_p_bosd.connect_state_set(move |_, active| {
            sh_st.borrow_mut().panels.brightness_osd = active;
            themes::sync_shell_config(&sh_st.borrow());
            gtk4::glib::Propagation::Proceed
        });
    }
    row_p_bosd.add_suffix(&sw_p_bosd);
    group_panels.add(&row_p_bosd);

    let row_p_notif = ActionRow::builder().title("Notificaties Overlay").subtitle("Geïntegreerde quickshell notificatiewidgets").build();
    let sw_p_notif = Switch::builder().active(initial_sh.panels.notifications).valign(gtk4::Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_p_notif.connect_state_set(move |_, active| {
            sh_st.borrow_mut().panels.notifications = active;
            themes::sync_shell_config(&sh_st.borrow());
            gtk4::glib::Propagation::Proceed
        });
    }
    row_p_notif.add_suffix(&sw_p_notif);
    group_panels.add(&row_p_notif);

    page.add(&group_panels);

    // ========================================================
    // 6. Power User Hub & Grenzeloos Hacken
    // ========================================================
    let group_power = PreferencesGroup::builder()
        .title(crate::ui::escape::pango_escape("Power User Hub & Custom Modules"))
        .description("Directe toegang voor ontwikkelaars: bewerk ruwe bestanden of voeg eigen QML componenten toe zonder beperkingen")
        .build();

    let row_mod_dir = ActionRow::builder()
        .title("Aangepaste Modules Map")
        .subtitle("Plaats elk willekeurig .qml widget in ~/.config/quickshell/modules/")
        .build();
    let btn_mod_dir = Button::builder().label("Open Map").valign(gtk4::Align::Center).build();
    btn_mod_dir.connect_clicked(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = format!("{}/.config/quickshell/modules", home);
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    });
    row_mod_dir.add_suffix(&btn_mod_dir);
    group_power.add(&row_mod_dir);

    let row_tmpl = ActionRow::builder()
        .title("Nieuwe Custom Module Maken")
        .subtitle("Maak direct een nieuw CustomWidget.qml template aan in je modules-map")
        .build();
    let btn_tmpl = Button::builder().label("Template Aanmaken").valign(gtk4::Align::Center).build();
    {
        let refresh_mod = Rc::clone(&refresh_all_modules);
        let ref_pick = Rc::clone(&refresh_picker_models);
        btn_tmpl.connect_clicked(move |_| {
            let home = std::env::var("HOME").unwrap_or_default();
            let dest = format!("{}/.config/quickshell/modules/CustomWidget.qml", home);
            if !std::path::Path::new(&dest).exists() {
                let example = format!("{}/.config/quickshell/modules/CustomWidget.qml.example", home);
                if std::path::Path::new(&example).exists() {
                    let _ = std::fs::copy(&example, &dest);
                } else {
                    let boilerplate = "import QtQuick\nimport QtQuick.Layouts\n\nRectangle {\n    implicitWidth: txt.implicitWidth + 16\n    implicitHeight: 22\n    radius: 6\n    color: Qt.alpha(root.themeAccent || \"#89b4fa\", 0.20)\n    border.color: root.themeAccent || \"#89b4fa\"\n    border.width: 1\n    Text { id: txt; anchors.centerIn: parent; text: \"✨ Custom Widget\"; color: root.fgColor || \"#cdd6f4\"; font.pixelSize: root.userFontSize || 11 }\n}\n";
                    let _ = std::fs::write(&dest, boilerplate);
                }
            }
            let _ = std::process::Command::new("xdg-open").arg(&dest).spawn();
            refresh_mod();
            ref_pick();
        });
    }
    row_tmpl.add_suffix(&btn_tmpl);
    group_power.add(&row_tmpl);

    let row_json = ActionRow::builder()
        .title("zenith-shell.json Config Bewerken")
        .subtitle("Open het centrale data-gedreven configuratiebestand in je editor")
        .build();
    let btn_json = Button::builder().label("Open JSON").valign(gtk4::Align::Center).build();
    btn_json.connect_clicked(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = format!("{}/.config/quickshell/zenith-shell.json", home);
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    });
    row_json.add_suffix(&btn_json);
    group_power.add(&row_json);

    let row_qml = ActionRow::builder()
        .title("shell.qml Bronbestand")
        .subtitle("Bekijk of bewerk het dynamische QML root-venster")
        .build();
    let btn_qml = Button::builder().label("Open QML").valign(gtk4::Align::Center).build();
    btn_qml.connect_clicked(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = format!("{}/.config/quickshell/shell.qml", home);
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    });
    row_qml.add_suffix(&btn_qml);
    group_power.add(&row_qml);

    let row_restart = ActionRow::builder()
        .title("Quickshell Herstarten")
        .subtitle("Forceer een volledige herstart van het Quickshell proces")
        .build();
    let btn_restart = Button::builder().label("Herstarten").valign(gtk4::Align::Center).build();
    btn_restart.connect_clicked(|_| {
        process::set_active_bar("quickshell");
    });
    row_restart.add_suffix(&btn_restart);
    group_power.add(&row_restart);

    page.add(&group_power);

    page
}

/// Voegt de "Module Toevoegen" dropdown en knop toe aan een sectiegroep
fn add_module_picker_row(
    group: &PreferencesGroup,
    slot_name: &'static str,
    shell_state: &Rc<RefCell<ZenithShellConfig>>,
    list_box: &Rc<ListBox>,
    picker_dropdowns: &Rc<RefCell<Vec<DropDown>>>,
) {
    let row_add = ActionRow::builder()
        .title("Nieuwe Module Toevoegen")
        .subtitle("Kies een ingebouwde module of een ontdekt custom widget")
        .build();

    let all_modules = shell_state.borrow().discover_modules_for_instance();
    let labels: Vec<String> = all_modules
        .iter()
        .map(|m| format!("{} {}", m.icon, m.name))
        .collect();
    let labels_ref: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
    let model = StringList::new(&labels_ref);

    let dd = DropDown::builder().model(&model).valign(gtk4::Align::Center).build();
    picker_dropdowns.borrow_mut().push(dd.clone());

    let btn_add = Button::builder().label("➕ Toevoegen").valign(gtk4::Align::Center).build();

    {
        let sh_st = Rc::clone(shell_state);
        let list_rc = Rc::clone(list_box);
        let dd_c = dd.clone();
        btn_add.connect_clicked(move |_| {
            let sel = dd_c.selected() as usize;
            let current_modules = sh_st.borrow().discover_modules_for_instance();
            if let Some(info) = current_modules.get(sel) {
                let mut sh = sh_st.borrow_mut();
                match slot_name {
                    "left" => sh.modules.left.push(info.id.clone()),
                    "center" => sh.modules.center.push(info.id.clone()),
                    "right" => sh.modules.right.push(info.id.clone()),
                    _ => {}
                }
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_slot(&list_rc, slot_name, &sh_st);
            }
        });
    }

    row_add.add_suffix(&dd);
    row_add.add_suffix(&btn_add);
    group.add(&row_add);
}

/// Ververst de rijen in een module slot ListBox
fn refresh_slot(
    list_box: &ListBox,
    slot_name: &'static str,
    shell_state: &Rc<RefCell<ZenithShellConfig>>,
) {
    // Verwijder alle huidige kinderen
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let all_modules = shell_state.borrow().discover_modules_for_instance();
    let sh = shell_state.borrow();
    let current_list: Vec<String> = match slot_name {
        "left" => sh.modules.left.clone(),
        "center" => sh.modules.center.clone(),
        "right" => sh.modules.right.clone(),
        _ => Vec::new(),
    };
    let total = current_list.len();

    if total == 0 {
        let empty_row = ActionRow::builder()
            .title("(Geen modules actief)")
            .subtitle("Voeg een module toe met de knop hieronder")
            .build();
        list_box.append(&empty_row);
        return;
    }

    for (idx, mod_id) in current_list.iter().enumerate() {
        let info = all_modules.iter().find(|m| &m.id == mod_id);
        let title = match info {
            Some(i) => format!("{} {}", i.icon, i.name),
            None => format!("🧩 {}", mod_id),
        };
        let desc = match info {
            Some(i) => i.description.clone(),
            None => format!("Module ID: {}", mod_id),
        };

        let row = ActionRow::builder()
            .title(crate::ui::escape::pango_escape(&title))
            .subtitle(crate::ui::escape::pango_escape(&desc))
            .build();

        // Knop Omhoog (▲)
        if idx > 0 {
            let btn_up = Button::builder().label("▲").valign(gtk4::Align::Center).tooltip_text("Naar links/boven verplaatsen").build();
            let sh_st = Rc::clone(shell_state);
            let lb = list_box.clone();
            btn_up.connect_clicked(move |_| {
                let mut sh = sh_st.borrow_mut();
                let list = match slot_name {
                    "left" => &mut sh.modules.left,
                    "center" => &mut sh.modules.center,
                    "right" => &mut sh.modules.right,
                    _ => return,
                };
                if idx > 0 && idx < list.len() {
                    list.swap(idx, idx - 1);
                }
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_slot(&lb, slot_name, &sh_st);
            });
            row.add_suffix(&btn_up);
        }

        // Knop Omlaag (▼)
        if idx + 1 < total {
            let btn_down = Button::builder().label("▼").valign(gtk4::Align::Center).tooltip_text("Naar rechts/onder verplaatsen").build();
            let sh_st = Rc::clone(shell_state);
            let lb = list_box.clone();
            btn_down.connect_clicked(move |_| {
                let mut sh = sh_st.borrow_mut();
                let list = match slot_name {
                    "left" => &mut sh.modules.left,
                    "center" => &mut sh.modules.center,
                    "right" => &mut sh.modules.right,
                    _ => return,
                };
                if idx + 1 < list.len() {
                    list.swap(idx, idx + 1);
                }
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_slot(&lb, slot_name, &sh_st);
            });
            row.add_suffix(&btn_down);
        }

        // Knop Verwijderen (✖)
        let btn_remove = Button::builder().label("✖").valign(gtk4::Align::Center).tooltip_text("Van statusbalk verwijderen").build();
        btn_remove.add_css_class("destructive-action");
        {
            let sh_st = Rc::clone(shell_state);
            let lb = list_box.clone();
            btn_remove.connect_clicked(move |_| {
                let mut sh = sh_st.borrow_mut();
                let list = match slot_name {
                    "left" => &mut sh.modules.left,
                    "center" => &mut sh.modules.center,
                    "right" => &mut sh.modules.right,
                    _ => return,
                };
                if idx < list.len() {
                    list.remove(idx);
                }
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_slot(&lb, slot_name, &sh_st);
            });
        }
        row.add_suffix(&btn_remove);

        list_box.append(&row);
    }
}

/// Ververst de lijst met custom commando scriptmodules
fn refresh_scripts_list(
    list_box: &ListBox,
    shell_state: &Rc<RefCell<ZenithShellConfig>>,
    refresh_all_modules: &Rc<dyn Fn()>,
    refresh_picker_models: &Rc<dyn Fn()>,
) {
    // Verwijder alle huidige kinderen
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let sh = shell_state.borrow();
    if sh.custom_scripts.is_empty() {
        let empty_row = ActionRow::builder()
            .title("(Nog geen custom script modules aangemaakt)")
            .subtitle("Definieer hieronder een shell-commando om je eigen widget te creëren")
            .build();
        list_box.append(&empty_row);
        return;
    }

    for (idx, script) in sh.custom_scripts.iter().enumerate() {
        let title = format!("{} {} [script:{}]", script.icon, script.name, script.id);
        let click_info = match &script.on_click {
            Some(cmd) if !cmd.trim().is_empty() => format!(" | Klik: `{}`", cmd),
            _ => String::new(),
        };
        let desc = format!(
            "Commando: `{}` | Polling: {}s{}",
            script.command, script.interval_seconds, click_info
        );

        let row = ActionRow::builder()
            .title(crate::ui::escape::pango_escape(&title))
            .subtitle(crate::ui::escape::pango_escape(&desc))
            .build();

        // Knop Verwijderen (✖)
        let btn_remove = Button::builder()
            .label("✖")
            .valign(gtk4::Align::Center)
            .tooltip_text("Verwijder deze custom scriptmodule")
            .build();
        btn_remove.add_css_class("destructive-action");

        {
            let sh_st = Rc::clone(shell_state);
            let lb = list_box.clone();
            let ref_all = Rc::clone(refresh_all_modules);
            let ref_pick = Rc::clone(refresh_picker_models);
            let script_id = script.id.clone();

            btn_remove.connect_clicked(move |_| {
                let mut sh = sh_st.borrow_mut();
                if idx < sh.custom_scripts.len() {
                    sh.custom_scripts.remove(idx);
                }
                // Verwijder ook eventuele actieve instanties op de balk
                let full_id = format!("script:{}", script_id);
                sh.modules.left.retain(|m| m != &full_id);
                sh.modules.center.retain(|m| m != &full_id);
                sh.modules.right.retain(|m| m != &full_id);

                themes::sync_shell_config(&sh);
                drop(sh);

                refresh_scripts_list(&lb, &sh_st, &ref_all, &ref_pick);
                ref_all();
                ref_pick();
            });
        }

        row.add_suffix(&btn_remove);
        list_box.append(&row);
    }
}
