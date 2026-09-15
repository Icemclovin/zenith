use gtk4::prelude::*;
use gtk4::{
    Align, Button, DropDown, Entry, FileDialog, Label, ListBox, Orientation, Scale,
    ScrolledWindow, StringList,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::fastfetch::{
    ansi_to_pango, generate_preview, FastfetchConfig, FastfetchDisplay, FastfetchDisplayKey,
    FastfetchLogo, FastfetchLogoPadding, FastfetchModuleConfig, FastfetchModuleItem,
};
use crate::backend::i18n::Translations;
use crate::backend::shell_config::ZenithShellConfig;

const AVAILABLE_MODULES: &[(&str, &str)] = &[
    ("break", "Lege Regel (Break)"),
    ("title", "Gebruiker & Host (Title)"),
    ("os", "Besturingssysteem (OS)"),
    ("host", "Hardware Model (Host)"),
    ("kernel", "Linux Kernel"),
    ("uptime", "Systeemtijd (Uptime)"),
    ("packages", "Pakketten (Packages)"),
    ("shell", "Shell (Bash/Zsh/Fish)"),
    ("wm", "Window Manager (Hyprland)"),
    ("terminal", "Terminal Emulator"),
    ("cpu", "Processor (CPU)"),
    ("gpu", "Videokaart (GPU)"),
    ("memory", "Werkgeheugen (RAM)"),
    ("swap", "Wisselbestand (Swap)"),
    ("disk", "Schijfopslag (Disk)"),
    ("battery", "Batterijstatus"),
    ("localip", "Lokaal IP-adres"),
    ("locale", "Systeemtaal (Locale)"),
    ("colors", "Terminal Kleurenpalet"),
];

pub fn build_fastfetch_page(tr: &Translations) -> PreferencesPage {
    let page = PreferencesPage::new();
    let ff_state = Rc::new(RefCell::new(FastfetchConfig::load_or_default()));

    // ==========================================
    // 1. Live Terminal Preview Box
    // ==========================================
    let group_preview = PreferencesGroup::builder()
        .title(&tr.ff_preview)
        .description("Realtime terminal weergave van Fastfetch met de huidige configuratie")
        .build();

    let preview_label = Label::builder()
        .use_markup(true)
        .selectable(true)
        .xalign(0.0)
        .yalign(0.0)
        .margin_start(16)
        .margin_end(16)
        .margin_top(16)
        .margin_bottom(16)
        .build();

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        "label.fastfetch-preview { \
            font-family: 'JetBrains Mono', 'Fira Code', 'DejaVu Sans Mono', monospace; \
            font-size: 11px; \
            background-color: #11111b; \
            color: #cdd6f4; \
            border-radius: 10px; \
            border: 1px solid #313244; \
            padding: 12px; \
        }"
    );
    #[allow(deprecated)]
    preview_label.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    preview_label.add_css_class("fastfetch-preview");

    let scroll_preview = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(240)
        .max_content_height(380)
        .child(&preview_label)
        .build();

    let btn_refresh = Button::builder()
        .label(&format!("🔄 {}", tr.ff_refresh_preview))
        .valign(Align::Center)
        .build();

    let lbl_weak = preview_label.downgrade();
    let ff_weak = Rc::downgrade(&ff_state);

    let refresh_preview_fn: Rc<dyn Fn()> = Rc::new(move || {
        let Some(lbl) = lbl_weak.upgrade() else { return };
        let Some(st) = ff_weak.upgrade() else { return };
        let cfg = st.borrow().clone();

        match generate_preview(&cfg) {
            Ok(raw) => {
                let pango = ansi_to_pango(&raw);
                lbl.set_markup(&pango);
            }
            Err(e) => {
                lbl.set_markup(&format!("<span foreground=\"#f38ba8\">Fout bij uitvoeren fastfetch: {}</span>", e));
            }
        }
    });

    {
        let rf = Rc::clone(&refresh_preview_fn);
        btn_refresh.connect_clicked(move |_| {
            rf();
        });
    }

    let row_preview_ctrl = ActionRow::builder()
        .title("Preview Bediening")
        .subtitle("Klik om de Fastfetch weergave direct te verversen")
        .build();
    row_preview_ctrl.add_suffix(&btn_refresh);

    group_preview.add(&scroll_preview);
    group_preview.add(&row_preview_ctrl);
    page.add(&group_preview);

    // Initial preview render
    refresh_preview_fn();

    // ==========================================
    // 2. Logo Instellingen
    // ==========================================
    let group_logo = PreferencesGroup::builder()
        .title(&tr.ff_logo_settings)
        .description("Configureer het ASCII- of afbeeldingslogo in Fastfetch")
        .build();

    // Logo Type
    let logo_types = ["small", "builtin", "auto", "file", "chafa", "kitty", "none"];
    let logo_type_strings: Vec<&str> = logo_types.to_vec();
    let logo_type_model = StringList::new(&logo_type_strings);
    let dd_logo_type = DropDown::builder()
        .model(&logo_type_model)
        .valign(Align::Center)
        .build();

    let current_ltype = ff_state.borrow().logo.as_ref().and_then(|l| l.logo_type.clone()).unwrap_or_else(|| "small".to_string());
    if let Some(pos) = logo_types.iter().position(|&t| t == current_ltype) {
        dd_logo_type.set_selected(pos as u32);
    }

    let row_logo_type = ActionRow::builder()
        .title(&tr.ff_logo_type)
        .subtitle("Kies het weergavetype van het logo (small, builtin, bestand, chafa, etc.)")
        .build();
    row_logo_type.add_suffix(&dd_logo_type);
    group_logo.add(&row_logo_type);

    {
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        dd_logo_type.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            if let Some(&choice) = logo_types.get(idx) {
                let mut st = ff_st.borrow_mut();
                if st.logo.is_none() {
                    st.logo = Some(FastfetchLogo::default());
                }
                if let Some(ref mut l) = st.logo {
                    l.logo_type = Some(choice.to_string());
                }
                drop(st);
                rf();
            }
        });
    }

    // Logo Source (Distro of Bestand)
    let current_source = ff_state.borrow().logo.as_ref().and_then(|l| l.source.clone()).unwrap_or_default();
    let entry_source = Entry::builder()
        .text(&current_source)
        .placeholder_text("bijv. arch, ubuntu, of /pad/naar/logo.png")
        .width_chars(24)
        .valign(Align::Center)
        .build();

    let btn_browse_logo = Button::builder()
        .label("📁 Blader")
        .valign(Align::Center)
        .build();

    let row_logo_source = ActionRow::builder()
        .title(&tr.ff_logo_source)
        .subtitle("Distronaam of pad naar een afbeelding/ASCII bestand")
        .build();
    row_logo_source.add_suffix(&entry_source);
    row_logo_source.add_suffix(&btn_browse_logo);
    group_logo.add(&row_logo_source);

    {
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        entry_source.connect_changed(move |e| {
            let txt = e.text().trim().to_string();
            let mut st = ff_st.borrow_mut();
            if st.logo.is_none() {
                st.logo = Some(FastfetchLogo::default());
            }
            if let Some(ref mut l) = st.logo {
                l.source = if txt.is_empty() { None } else { Some(txt) };
            }
            drop(st);
            rf();
        });
    }

    {
        let entry_c = entry_source.clone();
        btn_browse_logo.connect_clicked(move |_| {
            let fd = FileDialog::builder().title("Kies Logo Afbeelding of Tekst").build();
            let e_ref = entry_c.clone();
            fd.open(None::<&gtk4::Window>, None::<&gtk4::gio::Cancellable>, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        if let Some(p_str) = path.to_str() {
                            e_ref.set_text(p_str);
                        }
                    }
                }
            });
        });
    }

    // Padding Instellingen
    let cur_pad_top = ff_state.borrow().logo.as_ref().and_then(|l| l.padding.as_ref()).and_then(|p| p.top).unwrap_or(1);
    let row_pad_top = ActionRow::builder()
        .title("Logo Padding Boven")
        .subtitle(&format!("{} regels", cur_pad_top))
        .build();
    let s_pad_top = Scale::with_range(Orientation::Horizontal, 0.0, 8.0, 1.0);
    s_pad_top.set_value(cur_pad_top as f64);
    s_pad_top.set_width_request(140);
    {
        let r_c = row_pad_top.clone();
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        s_pad_top.connect_value_changed(move |s| {
            let v = s.value().round() as u32;
            r_c.set_subtitle(&format!("{} regels", v));
            let mut st = ff_st.borrow_mut();
            if st.logo.is_none() { st.logo = Some(FastfetchLogo::default()); }
            if let Some(ref mut l) = st.logo {
                if l.padding.is_none() { l.padding = Some(FastfetchLogoPadding::default()); }
                if let Some(ref mut p) = l.padding { p.top = Some(v); }
            }
            drop(st);
            rf();
        });
    }
    row_pad_top.add_suffix(&s_pad_top);
    group_logo.add(&row_pad_top);

    let cur_pad_right = ff_state.borrow().logo.as_ref().and_then(|l| l.padding.as_ref()).and_then(|p| p.right).unwrap_or(2);
    let row_pad_right = ActionRow::builder()
        .title("Logo Padding Rechts")
        .subtitle(&format!("{} spaties", cur_pad_right))
        .build();
    let s_pad_right = Scale::with_range(Orientation::Horizontal, 0.0, 10.0, 1.0);
    s_pad_right.set_value(cur_pad_right as f64);
    s_pad_right.set_width_request(140);
    {
        let r_c = row_pad_right.clone();
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        s_pad_right.connect_value_changed(move |s| {
            let v = s.value().round() as u32;
            r_c.set_subtitle(&format!("{} spaties", v));
            let mut st = ff_st.borrow_mut();
            if st.logo.is_none() { st.logo = Some(FastfetchLogo::default()); }
            if let Some(ref mut l) = st.logo {
                if l.padding.is_none() { l.padding = Some(FastfetchLogoPadding::default()); }
                if let Some(ref mut p) = l.padding { p.right = Some(v); }
            }
            drop(st);
            rf();
        });
    }
    row_pad_right.add_suffix(&s_pad_right);
    group_logo.add(&row_pad_right);

    page.add(&group_logo);

    // ==========================================
    // 3. Display & Styling
    // ==========================================
    let group_display = PreferencesGroup::builder()
        .title(&tr.ff_styling)
        .description("Pas de scheidingstekenstijl en visuele opmaak aan")
        .build();

    let cur_sep = ff_state.borrow().display.as_ref().and_then(|d| d.separator.clone()).unwrap_or_else(|| " ➜ ".to_string());
    let entry_sep = Entry::builder()
        .text(&cur_sep)
        .placeholder_text("bijv. ' ➜ ' of ': ' of ' '")
        .width_chars(12)
        .valign(Align::Center)
        .build();

    let row_sep = ActionRow::builder()
        .title(&tr.ff_separator)
        .subtitle("Het scheidingsteken tussen modulelabels en waarden")
        .build();
    row_sep.add_suffix(&entry_sep);
    group_display.add(&row_sep);

    {
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        entry_sep.connect_changed(move |e| {
            let txt = e.text().to_string();
            let mut st = ff_st.borrow_mut();
            if st.display.is_none() { st.display = Some(FastfetchDisplay::default()); }
            if let Some(ref mut d) = st.display {
                d.separator = Some(txt);
            }
            drop(st);
            rf();
        });
    }

    let cur_key_w = ff_state.borrow().display.as_ref().and_then(|d| d.key.as_ref()).and_then(|k| k.width).unwrap_or(12);
    let row_key_w = ActionRow::builder()
        .title("Sleutelbreedte (Key Width)")
        .subtitle(&format!("{} tekens", cur_key_w))
        .build();
    let s_key_w = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_key_w.set_value(cur_key_w as f64);
    s_key_w.set_width_request(140);
    {
        let r_c = row_key_w.clone();
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        s_key_w.connect_value_changed(move |s| {
            let v = s.value().round() as u32;
            r_c.set_subtitle(&format!("{} tekens", v));
            let mut st = ff_st.borrow_mut();
            if st.display.is_none() { st.display = Some(FastfetchDisplay::default()); }
            if let Some(ref mut d) = st.display {
                if d.key.is_none() { d.key = Some(FastfetchDisplayKey::default()); }
                if let Some(ref mut k) = d.key { k.width = Some(v); }
            }
            drop(st);
            rf();
        });
    }
    row_key_w.add_suffix(&s_key_w);
    group_display.add(&row_key_w);

    // Sync met Zenith Kleurenpalet
    let row_sync_theme = ActionRow::builder()
        .title("Zenith Thema Synchronisatie")
        .subtitle("Pas de actieve Zenith accentkleuren automatisch toe op Fastfetch")
        .build();
    let btn_sync_theme = Button::builder()
        .label("🎨 Thema Synchroniseren")
        .valign(Align::Center)
        .build();

    {
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        btn_sync_theme.connect_clicked(move |_| {
            let scfg = ZenithShellConfig::load_or_default();
            let primary = scfg.styling.accent.clone();
            let secondary = scfg.styling.border_color.clone();
            let text_color = scfg.styling.text_color.clone();

            let mut st = ff_st.borrow_mut();
            if st.display.is_none() { st.display = Some(FastfetchDisplay::default()); }
            if let Some(ref mut d) = st.display {
                let mut col_map = std::collections::BTreeMap::new();
                col_map.insert("separator".to_string(), primary.clone());
                col_map.insert("keys".to_string(), secondary.clone());
                d.color = Some(col_map);
            }

            if st.logo.is_none() { st.logo = Some(FastfetchLogo::default()); }
            if let Some(ref mut l) = st.logo {
                let mut col_map = std::collections::BTreeMap::new();
                col_map.insert("1".to_string(), primary.clone());
                col_map.insert("2".to_string(), secondary.clone());
                col_map.insert("3".to_string(), text_color);
                l.color = Some(col_map);
            }

            drop(st);
            rf();
        });
    }
    row_sync_theme.add_suffix(&btn_sync_theme);
    group_display.add(&row_sync_theme);

    page.add(&group_display);

    // ==========================================
    // 4. Actieve Modules Manager
    // ==========================================
    let group_modules = PreferencesGroup::builder()
        .title(&tr.ff_modules)
        .description("Beheer de volgorde, labels en inhoud van elke fetch-regel")
        .build();

    let list_box_modules = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let list_box_rc = Rc::new(list_box_modules);

    let refresh_modules_ui: Rc<dyn Fn()> = {
        let lb_ref = Rc::clone(&list_box_rc);
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);

        Rc::new(move || {
            rebuild_modules_list(&lb_ref, &ff_st, &rf);
        })
    };

    // Helper closure to trigger list rebuild
    fn rebuild_modules_list(lb: &ListBox, st: &Rc<RefCell<FastfetchConfig>>, rf: &Rc<dyn Fn()>) {
        while let Some(child) = lb.first_child() {
            lb.remove(&child);
        }

        let modules = st.borrow().modules.clone();
        let total = modules.len();

        for (idx, item) in modules.into_iter().enumerate() {
            let title = item.display_label();
            let subtitle = match &item {
                FastfetchModuleItem::Simple(s) => format!("Standaard module: {}", s),
                FastfetchModuleItem::Detailed(d) => {
                    if d.module_type == "command" {
                        format!("Shell commando: {}", d.text.as_deref().unwrap_or(""))
                    } else {
                        format!("Type: {}", d.module_type)
                    }
                }
            };

            let row = ActionRow::builder().title(&title).subtitle(&subtitle).build();

            if idx > 0 {
                let btn_up = Button::builder().label("▲").valign(Align::Center).tooltip_text("Omhoog").build();
                let st_c = Rc::clone(st);
                let lb_c = lb.clone();
                let rf_c = Rc::clone(rf);
                btn_up.connect_clicked(move |_| {
                    let mut s = st_c.borrow_mut();
                    if idx > 0 && idx < s.modules.len() {
                        s.modules.swap(idx, idx - 1);
                    }
                    drop(s);
                    rebuild_modules_list(&lb_c, &st_c, &rf_c);
                    rf_c();
                });
                row.add_suffix(&btn_up);
            }

            if idx + 1 < total {
                let btn_down = Button::builder().label("▼").valign(Align::Center).tooltip_text("Omlaag").build();
                let st_c = Rc::clone(st);
                let lb_c = lb.clone();
                let rf_c = Rc::clone(rf);
                btn_down.connect_clicked(move |_| {
                    let mut s = st_c.borrow_mut();
                    if idx + 1 < s.modules.len() {
                        s.modules.swap(idx, idx + 1);
                    }
                    drop(s);
                    rebuild_modules_list(&lb_c, &st_c, &rf_c);
                    rf_c();
                });
                row.add_suffix(&btn_down);
            }

            let btn_del = Button::builder().label("✖").valign(Align::Center).tooltip_text("Verwijderen").build();
            btn_del.add_css_class("destructive-action");
            {
                let st_c = Rc::clone(st);
                let lb_c = lb.clone();
                let rf_c = Rc::clone(rf);
                btn_del.connect_clicked(move |_| {
                    let mut s = st_c.borrow_mut();
                    if idx < s.modules.len() {
                        s.modules.remove(idx);
                    }
                    drop(s);
                    rebuild_modules_list(&lb_c, &st_c, &rf_c);
                    rf_c();
                });
            }
            row.add_suffix(&btn_del);

            lb.append(&row);
        }
    }

    // Module Toevoegen Dropdown + Knop
    let mod_labels: Vec<&str> = AVAILABLE_MODULES.iter().map(|(_, label)| *label).collect();
    let mod_model = StringList::new(&mod_labels);
    let dd_add_mod = DropDown::builder().model(&mod_model).valign(Align::Center).build();

    let btn_add_mod = Button::builder().label(&tr.ff_add_module).valign(Align::Center).build();
    btn_add_mod.add_css_class("suggested-action");

    let row_add_mod = ActionRow::builder()
        .title("+ Module Toevoegen")
        .subtitle("Selecteer een systeeminformatie-veld en voeg deze toe aan je fetch")
        .build();
    row_add_mod.add_suffix(&dd_add_mod);
    row_add_mod.add_suffix(&btn_add_mod);

    {
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        let refresh_list = Rc::clone(&refresh_modules_ui);
        btn_add_mod.connect_clicked(move |_| {
            let idx = dd_add_mod.selected() as usize;
            if let Some(&(mod_name, _)) = AVAILABLE_MODULES.get(idx) {
                let mut st = ff_st.borrow_mut();
                if mod_name == "break" {
                    st.modules.push(FastfetchModuleItem::Simple("break".to_string()));
                } else if mod_name == "colors" {
                    st.modules.push(FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                        module_type: "colors".to_string(),
                        symbol: Some("circle".to_string()),
                        ..Default::default()
                    }));
                } else {
                    st.modules.push(FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                        module_type: mod_name.to_string(),
                        key: Some(mod_name.to_uppercase()),
                        ..Default::default()
                    }));
                }
                drop(st);
                refresh_list();
                rf();
            }
        });
    }

    group_modules.add(&*list_box_rc);
    group_modules.add(&row_add_mod);
    page.add(&group_modules);

    // Initial render of module list
    refresh_modules_ui();

    // ==========================================
    // 5. Custom Shell Commando Builder
    // ==========================================
    let group_custom_cmd = PreferencesGroup::builder()
        .title(&tr.ff_custom_command)
        .description("Voer een willekeurig shell-commando uit en toon de uitkomst als fetch-regel")
        .build();

    let entry_cmd_key = Entry::builder()
        .placeholder_text("Label / Sleutel (bijv. Temp)")
        .width_chars(14)
        .valign(Align::Center)
        .build();
    let row_cmd_key = ActionRow::builder().title(&tr.ff_cmd_name).build();
    row_cmd_key.add_suffix(&entry_cmd_key);
    group_custom_cmd.add(&row_cmd_key);

    let entry_cmd_script = Entry::builder()
        .placeholder_text("Shell script (bijv. uname -r of uptime -p)")
        .width_chars(28)
        .valign(Align::Center)
        .build();
    let row_cmd_script = ActionRow::builder()
        .title(&tr.ff_cmd_script)
        .subtitle("Het bash/sh commando waarvan de output wordt getoond")
        .build();
    row_cmd_script.add_suffix(&entry_cmd_script);
    group_custom_cmd.add(&row_cmd_script);

    let btn_add_cmd = Button::builder()
        .label("+ Eigen Commando Toevoegen")
        .valign(Align::Center)
        .build();
    btn_add_cmd.add_css_class("suggested-action");

    let row_btn_add_cmd = ActionRow::builder().build();
    row_btn_add_cmd.add_suffix(&btn_add_cmd);
    group_custom_cmd.add(&row_btn_add_cmd);

    {
        let ff_st = Rc::clone(&ff_state);
        let rf = Rc::clone(&refresh_preview_fn);
        let refresh_list = Rc::clone(&refresh_modules_ui);
        let e_k = entry_cmd_key.clone();
        let e_s = entry_cmd_script.clone();
        btn_add_cmd.connect_clicked(move |_| {
            let key = e_k.text().trim().to_string();
            let script = e_s.text().trim().to_string();
            if key.is_empty() || script.is_empty() { return; }

            let mut st = ff_st.borrow_mut();
            st.modules.push(FastfetchModuleItem::Detailed(FastfetchModuleConfig {
                module_type: "command".to_string(),
                key: Some(key),
                text: Some(script),
                ..Default::default()
            }));
            drop(st);

            e_k.set_text("");
            e_s.set_text("");
            refresh_list();
            rf();
        });
    }

    page.add(&group_custom_cmd);

    // ==========================================
    // 6. Opslaan naar Bestandsconfiguratie
    // ==========================================
    let group_save = PreferencesGroup::builder()
        .title("Configuratie Opslaan")
        .description("Slaat direct op naar ~/.config/fastfetch/config.jsonc met automatische backup")
        .build();

    let btn_save = Button::builder()
        .label(&format!("💾 {}", tr.ff_save))
        .valign(Align::Center)
        .build();
    btn_save.add_css_class("suggested-action");

    let row_save = ActionRow::builder()
        .title(&tr.ff_save)
        .subtitle("Veilige backup wordt automatisch bewaard als config.jsonc.bak")
        .build();
    row_save.add_suffix(&btn_save);

    let status_label = Label::builder()
        .label("")
        .halign(Align::Start)
        .margin_start(12)
        .margin_top(4)
        .build();

    {
        let ff_st = Rc::clone(&ff_state);
        let lbl_status = status_label.clone();
        let toast_msg = tr.ff_saved_toast.clone();
        btn_save.connect_clicked(move |_| {
            let st = ff_st.borrow();
            match st.save() {
                Ok(_) => {
                    lbl_status.set_markup(&format!("<span foreground=\"#a6e3a1\">✔ {}</span>", toast_msg));
                }
                Err(e) => {
                    lbl_status.set_markup(&format!("<span foreground=\"#f38ba8\">✖ Fout bij opslaan: {}</span>", e));
                }
            }
        });
    }

    group_save.add(&row_save);
    group_save.add(&status_label);
    page.add(&group_save);

    page
}
