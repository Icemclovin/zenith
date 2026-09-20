use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, ColorDialog, ColorDialogButton, DropDown, Entry, FileDialog, Image, Label, ListBox,
    ListBoxRow, Orientation, Scale, StringList, Switch,
};
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, ComboRow, HeaderBar, PreferencesGroup,
    PreferencesPage, ViewStack,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use crate::backend::{config, hyprland, ipc_client, process, themes, waybar};
use crate::backend::{i18n, palette, shell_config::ZenithShellConfig};

pub fn build_window(app: &Application) {
    let initial_cfg = config::load_config();
    let state = Rc::new(RefCell::new(initial_cfg.clone()));

    let shell_cfg = ZenithShellConfig::load_or_default();
    let lang = i18n::Language::from_str(&shell_cfg.language).unwrap_or(i18n::Language::Nl);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Zenith")
        .default_width(940)
        .default_height(700)
        .build();

    let root_box = build_window_content(&window, app, &state, &lang, "dashboard");
    window.set_content(Some(&root_box));
    window.present();
}

fn is_valid_image(path: &str) -> bool {
    let p = std::path::Path::new(path);
    if !p.exists() || !p.is_file() {
        return false;
    }
    if let Ok(meta) = p.metadata() {
        if meta.len() < 1024 {
            return false;
        }
    }
    image::image_dimensions(p).is_ok()
}

/// Helper om het huidige actieve wallpaper-bestand te detecteren
fn detect_current_wallpaper() -> Option<String> {
    let scfg = ZenithShellConfig::load_or_default();
    if !scfg.wallpaper_path.is_empty() && is_valid_image(&scfg.wallpaper_path) {
        return Some(scfg.wallpaper_path);
    }

    // Try to find the wallpaper from a running swaybg process
    if let Ok(output) = std::process::Command::new("pgrep").args(["-a", "swaybg"]).output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            // Format: "PID swaybg -i /path/to/img -m fill"
            for line in text.lines() {
                if let Some(idx) = line.find("-i ") {
                    let rest = &line[idx + 3..];
                    let path = rest.split_whitespace().next().unwrap_or("");
                    if is_valid_image(path) {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }

    // Fallback: pick the first valid image from ~/Pictures/Wallpapers
    if let Ok(home) = std::env::var("HOME") {
        let wp_dir = std::path::PathBuf::from(home).join("Pictures/Wallpapers");
        if let Ok(entries) = std::fs::read_dir(wp_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(p_str) = p.to_str() {
                    if is_valid_image(p_str) {
                        return Some(p_str.to_string());
                    }
                }
            }
        }
    }

    // System wallpapers
    for sys_wp in &["/usr/share/hypr/wall2.png", "/usr/share/hypr/wall1.png", "/usr/share/hypr/wall0.png"] {
        if is_valid_image(sys_wp) {
            return Some(sys_wp.to_string());
        }
    }

    None
}

fn update_swatches_chips(swatch_box: &Box, bg: &str, surface: &str, accent: &str, border: &str, fg: &str) {
    while let Some(child) = swatch_box.first_child() {
        swatch_box.remove(&child);
    }
    let swatches_info = [
        ("Background", bg),
        ("Surface", surface),
        ("Primary Accent", accent),
        ("Border", border),
        ("Foreground", fg),
    ];
    for (role, hex) in swatches_info {
        let chip = Label::builder()
            .label(format!(" {} ", hex))
            .tooltip_text(role)
            .valign(gtk4::Align::Center)
            .build();
        let fg_col = if hex.starts_with("#1") || hex.starts_with("#2") || hex.starts_with("#0") || hex.starts_with("#3") {
            "#cdd6f4"
        } else {
            "#11111b"
        };
        let prov = gtk4::CssProvider::new();
        prov.load_from_data(&format!(
            "label {{ background-color: {}; color: {}; font-weight: bold; font-size: 10px; border-radius: 6px; padding: 4px 6px; border: 1px solid #45475a; }}",
            hex, fg_col
        ));
        #[allow(deprecated)]
        chip.style_context().add_provider(&prov, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        swatch_box.append(&chip);
    }
}

fn build_window_content(
    window: &ApplicationWindow,
    app: &Application,
    state: &Rc<RefCell<config::ZenithConfig>>,
    lang: &i18n::Language,
    active_tab: &str,
) -> Box {
    let tr = i18n::get_translations(lang);
    let initial_cfg = state.borrow().clone();
    let shell_cfg = ZenithShellConfig::load_or_default();

    let root_box = Box::new(Orientation::Horizontal, 0);

    // ========================================================
    // Linker Zijbalk (Modern Sidebar Navigation)
    // ========================================================
    let sidebar = Box::new(Orientation::Vertical, 12);
    sidebar.add_css_class("zenith-sidebar");
    sidebar.set_width_request(230);

    // Header Branding
    let brand_box = Box::new(Orientation::Vertical, 2);
    brand_box.set_margin_bottom(6);

    let title_lbl = Label::builder()
        .label("ZENITH")
        .halign(gtk4::Align::Start)
        .css_classes(["zenith-brand-title"])
        .build();

    let sub_lbl = Label::builder()
        .label("HYPRLAND STUDIO")
        .halign(gtk4::Align::Start)
        .css_classes(["zenith-brand-sub"])
        .build();

    brand_box.append(&title_lbl);
    brand_box.append(&sub_lbl);
    sidebar.append(&brand_box);

    // Status Pill Badge (Quickshell Daemon Status)
    let qs_pill = Label::builder()
        .label(format!("● {}", tr.status_active))
        .halign(gtk4::Align::Start)
        .css_classes(["status-pill-active"])
        .build();

    let qs_running = process::is_process_running("quickshell");
    if qs_running {
        qs_pill.set_label(&format!("● {}", tr.status_active));
        qs_pill.set_css_classes(&["status-pill-active"]);
    } else {
        qs_pill.set_label(&format!("○ {}", tr.status_inactive));
        qs_pill.set_css_classes(&["status-pill-inactive"]);
    }

    let pill_weak = qs_pill.downgrade();
    let status_active = tr.status_active.clone();
    let status_inactive = tr.status_inactive.clone();
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
        let Some(pill) = pill_weak.upgrade() else {
            return gtk4::glib::ControlFlow::Break;
        };
        let running = process::is_process_running("quickshell");
        if running {
            pill.set_label(&format!("● {}", status_active));
            pill.set_css_classes(&["status-pill-active"]);
        } else {
            pill.set_label(&format!("○ {}", status_inactive));
            pill.set_css_classes(&["status-pill-inactive"]);
        }
        gtk4::glib::ControlFlow::Continue
    });

    sidebar.append(&qs_pill);

    // Navigatie Lijst in Zijbalk
    let nav_list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .css_classes(["sidebar-list"])
        .margin_top(8)
        .build();

    struct NavItem {
        id: &'static str,
        title: String,
        icon: &'static str,
        /// `true` voor pagina's die vooral voor power users interessant zijn
        /// en worden verborgen in de "basis"-modus.
        advanced: bool,
    }

    let nav_items = [
        NavItem { id: "dashboard", title: tr.sidebar_dashboard.clone(), icon: "view-grid-symbolic", advanced: false },
        NavItem { id: "hyprland", title: tr.sidebar_hyprland.clone(), icon: "applications-graphics-symbolic", advanced: false },
        NavItem { id: "statusbar", title: tr.sidebar_statusbar.clone(), icon: "utilities-terminal-symbolic", advanced: false },
        NavItem { id: "control_center", title: tr.sidebar_control_center.clone(), icon: "preferences-desktop-keyboard-shortcuts-symbolic", advanced: false },
        NavItem { id: "osd", title: tr.sidebar_osd.clone(), icon: "video-display-symbolic", advanced: false },
        NavItem { id: "lockscreen", title: tr.sidebar_lockscreen.clone(), icon: "system-lock-screen-symbolic", advanced: false },
        NavItem { id: "themes", title: tr.sidebar_themes.clone(), icon: "applications-accessories-symbolic", advanced: false },
        NavItem { id: "system", title: tr.sidebar_system.clone(), icon: "emblem-system-symbolic", advanced: false },
        NavItem { id: "backup", title: tr.sidebar_backup.clone(), icon: "folder-download-symbolic", advanced: false },
        NavItem { id: "icons", title: tr.icons_title.clone(), icon: "emblem-favorite-symbolic", advanced: true },
        NavItem { id: "fastfetch", title: tr.sidebar_fastfetch.clone(), icon: "utilities-terminal-symbolic", advanced: true },
    ];

    let shell_initial = ZenithShellConfig::load_or_default();
    let advanced_init = shell_initial.advanced_mode;
    let mut advanced_rows: Vec<gtk4::ListBoxRow> = Vec::new();

    for item in &nav_items {
        let row = ListBoxRow::new();
        row.set_widget_name(item.id);
        row.set_visible(!item.advanced || advanced_init);

        let row_box = Box::new(Orientation::Horizontal, 10);
        row_box.set_margin_top(4);
        row_box.set_margin_bottom(4);
        row_box.set_margin_start(4);
        row_box.set_margin_end(4);

        let img = Image::from_icon_name(item.icon);
        img.set_pixel_size(16);

        let lbl = Label::builder()
            .label(crate::ui::escape::pango_escape(&item.title))
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        row_box.append(&img);
        row_box.append(&lbl);
        row.set_child(Some(&row_box));
        nav_list.append(&row);

        if item.advanced {
            advanced_rows.push(row);
        }
    }

    sidebar.append(&nav_list);

    // ========================================================
    // Rechter Inhoud (HeaderBar + ViewStack)
    // ========================================================
    let content_box = Box::new(Orientation::Vertical, 0);
    content_box.set_hexpand(true);
    content_box.set_vexpand(true);

    let header = HeaderBar::new();
    header.set_show_end_title_buttons(true);
    let title_center = Label::builder()
        .label("Zenith Control Center")
        .css_classes(["title"])
        .build();
    header.set_title_widget(Some(&title_center));

    content_box.append(&header);

    let stack = ViewStack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    content_box.append(&stack);

    // Language Selector (Direct live UI refresh zonder herstart!)
    let lang_box = Box::new(Orientation::Horizontal, 8);
    lang_box.set_margin_top(12);
    lang_box.set_margin_start(12);
    lang_box.set_margin_end(12);
    let lang_label = Label::builder().label(&tr.lang_select).halign(gtk4::Align::Start).hexpand(true).build();
    let lang_model = StringList::new(&["Nederlands", "English", "Deutsch", "Español"]);
    let lang_drop = DropDown::builder().model(&lang_model).build();
    lang_drop.set_selected(match lang {
        i18n::Language::Nl => 0,
        i18n::Language::En => 1,
        i18n::Language::De => 2,
        i18n::Language::Es => 3,
    });

    let win_weak = window.downgrade();
    let app_clone = app.clone();
    let state_clone = Rc::clone(state);
    let stack_for_lang = stack.clone();

    lang_drop.connect_selected_notify(move |dd| {
        let lang_str = match dd.selected() {
            1 => "en",
            2 => "de",
            3 => "es",
            _ => "nl",
        };

        let mut scfg = ZenithShellConfig::load_or_default();
        if scfg.language == lang_str {
            return;
        }
        scfg.language = lang_str.to_string();
        let _ = scfg.save();

        let new_lang = i18n::Language::from_str(lang_str).unwrap_or(i18n::Language::Nl);
        let win_weak_c = win_weak.clone();
        let app_c = app_clone.clone();
        let state_c = Rc::clone(&state_clone);
        let stack_c = stack_for_lang.clone();

        gtk4::glib::idle_add_local_once(move || {
            if let Some(win) = win_weak_c.upgrade() {
                let current_tab = stack_c.visible_child_name().unwrap_or_else(|| "dashboard".into());
                let new_content = build_window_content(&win, &app_c, &state_c, &new_lang, current_tab.as_str());
                win.set_content(Some(&new_content));
            }
        });
    });

    lang_box.append(&lang_label);
    lang_box.append(&lang_drop);
    sidebar.append(&lang_box);

    // Modus-schakelaar: Basis (gericht) vs Power User (alles zichtbaar)
    let mode_box = Box::new(Orientation::Horizontal, 8);
    mode_box.set_margin_top(8);
    mode_box.set_margin_start(12);
    mode_box.set_margin_end(12);
    let mode_lbl_col = Box::new(Orientation::Vertical, 2);
    let mode_title = Label::builder()
        .label(crate::ui::escape::pango_escape(&tr.mode_advanced))
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let mode_sub = Label::builder()
        .label(crate::ui::escape::pango_escape(&tr.mode_advanced_desc))
        .halign(gtk4::Align::Start)
        .css_classes(["zenith-mode-sub"])
        .build();
    mode_lbl_col.append(&mode_title);
    mode_lbl_col.append(&mode_sub);
    let sw_mode = Switch::builder().valign(gtk4::Align::Center).active(advanced_init).build();
    mode_box.append(&mode_lbl_col);
    mode_box.append(&sw_mode);
    sidebar.append(&mode_box);

    let advanced_rows_rc = Rc::new(std::cell::RefCell::new(advanced_rows));
    sw_mode.connect_active_notify(move |sw| {
        let active = sw.is_active();
        let rows = advanced_rows_rc.borrow();
        for row in rows.iter() {
            row.set_visible(active);
        }
        let mut scfg = ZenithShellConfig::load_or_default();
        scfg.advanced_mode = active;
        let _ = scfg.save();
    });

    let spacer = Box::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar.append(&spacer);

    let footer_lbl = Label::builder()
        .label("Zenith v2.2 | Arch Linux")
        .halign(gtk4::Align::Start)
        .css_classes(["zenith-brand-sub"])
        .build();
    sidebar.append(&footer_lbl);
    root_box.append(&sidebar);
    root_box.append(&content_box);

    let stack_clone = stack.clone();
    nav_list.connect_row_selected(move |_, row| {
        if let Some(r) = row {
            let name = r.widget_name();
            stack_clone.set_visible_child_name(&name);
        }
    });

    // ========================================================
    // PAGINA 1: Dashboard
    // ========================================================
    let page_dash = PreferencesPage::new();

    let group_quick = PreferencesGroup::builder()
        .title(&tr.dash_quick_controls)
        .description(&tr.dash_quick_desc)
        .build();

    let row_cc_test = ActionRow::builder()
        .title(&tr.dash_cc_title)
        .subtitle(&tr.dash_cc_sub)
        .build();
    let btn_cc_test = Button::builder().label(format!("🚀 {}", tr.dash_test_cc)).valign(gtk4::Align::Center).build();
    btn_cc_test.connect_clicked(|_| {
        process::execute_cmd("quickshell ipc call controlCenter toggle");
    });
    row_cc_test.add_suffix(&btn_cc_test);
    group_quick.add(&row_cc_test);

    let row_osd_test = ActionRow::builder()
        .title(&tr.dash_osd_title)
        .subtitle(&tr.dash_osd_sub)
        .build();
    let btn_osd_test = Button::builder().label(format!("🔔 {}", tr.dash_test_osd)).valign(gtk4::Align::Center).build();
    btn_osd_test.connect_clicked(|_| {
        process::execute_cmd("quickshell ipc call osd popup '⚡' 'Zenith OS' 0.85");
    });
    row_osd_test.add_suffix(&btn_osd_test);
    group_quick.add(&row_osd_test);

    let row_reload_hypr = ActionRow::builder()
        .title(&tr.dash_hypr_title)
        .subtitle(&tr.dash_hypr_sub)
        .build();
    let btn_reload_hypr = Button::builder().label(format!("🔄 {}", tr.dash_reload_hypr)).valign(gtk4::Align::Center).build();
    btn_reload_hypr.connect_clicked(|_| {
        process::execute_cmd("hyprctl reload");
    });
    row_reload_hypr.add_suffix(&btn_reload_hypr);
    group_quick.add(&row_reload_hypr);

    let row_restart_bar = ActionRow::builder()
        .title(&tr.dash_restart_bar)
        .subtitle("Quickshell / Waybar daemon")
        .build();
    let btn_restart_bar = Button::builder().label(format!("⚡ {}", tr.dash_restart_bar)).valign(gtk4::Align::Center).build();
    let st_rbar = Rc::clone(state);
    btn_restart_bar.connect_clicked(move |_| {
        let active = st_rbar.borrow().active_bar.clone();
        process::set_active_bar(&active);
    });
    row_restart_bar.add_suffix(&btn_restart_bar);
    group_quick.add(&row_restart_bar);

    // zenithd daemon-status (IPC-bridge voor de Two-Way Sync)
    let sub_on_owned = tr.daemon_sub_on.clone();
    let sub_off_owned = tr.daemon_sub_off.clone();
    let btn_start_owned = tr.daemon_btn_start.clone();
    let btn_stop_owned = tr.daemon_btn_stop.clone();
    let daemon_on = ipc_client::is_daemon_running();
    let row_daemon = ActionRow::builder()
        .title(crate::ui::escape::pango_escape(&tr.daemon_title))
        .subtitle(if daemon_on { sub_on_owned.as_str() } else { sub_off_owned.as_str() })
        .build();
    let btn_daemon = Button::builder()
        .label(if daemon_on { btn_stop_owned.as_str() } else { btn_start_owned.as_str() })
        .valign(gtk4::Align::Center)
        .build();
    row_daemon.add_suffix(&btn_daemon);
    group_quick.add(&row_daemon);

    let row_c = row_daemon.clone();
    let btn_c = btn_daemon.clone();
    let sub_on_c = sub_on_owned.clone();
    let sub_off_c = sub_off_owned.clone();
    let btn_on_c = btn_start_owned.clone();
    let btn_off_c = btn_stop_owned.clone();
    btn_daemon.connect_clicked(move |_| {
        if ipc_client::is_daemon_running() {
            process::execute_cmd("pkill -x zenithd");
        } else {
            process::execute_cmd("zenithd &");
        }
        // Frisse klonen voor de innerlijke timer-closure (Fn-correct).
        let row_inner = row_c.clone();
        let btn_inner = btn_c.clone();
        let sub_on_inner = sub_on_c.clone();
        let sub_off_inner = sub_off_c.clone();
        let btn_on_inner = btn_on_c.clone();
        let btn_off_inner = btn_off_c.clone();
        // Verfris de status na een korte pauze zodat de daemon de wijziging oppikt.
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(600), move || {
            let on = ipc_client::is_daemon_running();
            if on {
                row_inner.set_subtitle(sub_on_inner.as_str());
                btn_inner.set_label(btn_off_inner.as_str());
            } else {
                row_inner.set_subtitle(sub_off_inner.as_str());
                btn_inner.set_label(btn_on_inner.as_str());
            }
            gtk4::glib::ControlFlow::Continue
        });
    });

    page_dash.add(&group_quick);

    // ============================================================
    // ONBOARDING / WELKOM: installatiestatus + één-klik-installatie
    // (duidelijke gid voor nieuwe gebruikers, zonder de kracht te beperken)
    // ============================================================
    let missing = process::missing_dependencies();
    let group_onboard = PreferencesGroup::builder()
        .title(&tr.onb_welcome_title)
        .description(&tr.onb_welcome_desc)
        .build();

    // Kern-onderdelen met een live statusregel en één-klik-installatie
    let status_keys = [
        process::ZenithDependency::Quickshell,
        process::ZenithDependency::Hyprland,
        process::ZenithDependency::Waybar,
    ];
    for dep in status_keys {
        let installed = dep.is_installed();
        let row = ActionRow::builder()
            .title(dep.title())
            .subtitle(if installed { tr.onb_installed.clone() } else { tr.onb_install.clone() })
            .build();
        if !installed {
            let btn_i = Button::builder().label(&tr.onb_install).valign(gtk4::Align::Center).build();
            btn_i.connect_clicked(move |_| {
                match dep {
                    process::ZenithDependency::Quickshell => process::install_quickshell(),
                    other => process::install_dependency(other),
                }
            });
            row.add_suffix(&btn_i);
        }
        group_onboard.add(&row);
    }

    // Eén-knop "Alles installeren" als er nog iets ontbreekt
    if !missing.is_empty() {
        let missing_names: Vec<&str> = missing.iter().map(|d| d.title()).collect();
        let row_install_all = ActionRow::builder()
            .title(&tr.onb_install_all)
            .subtitle(format!("{}: {}", tr.onb_core_status, missing_names.join(", ")))
            .build();
        let btn_all = Button::builder().label(&tr.onb_install_all).valign(gtk4::Align::Center).build();
        btn_all.connect_clicked(move |_| {
            for d in process::ZenithDependency::ALL {
                if !d.is_installed() {
                    process::install_dependency(d);
                }
            }
        });
        row_install_all.add_suffix(&btn_all);
        group_onboard.add(&row_install_all);
    } else {
        let row_set = ActionRow::builder()
            .title(&tr.onb_installed)
            .subtitle(&tr.onb_all_set)
            .build();
        group_onboard.add(&row_set);
    }

    // Power User-vrijheid: directe toegang tot ruwe Quickshell-bestanden & modules
    let group_power = PreferencesGroup::builder()
        .title(&tr.onb_power_title)
        .description(&tr.onb_power_desc)
        .build();
    let row_power = ActionRow::builder()
        .title(&tr.onb_open_power)
        .subtitle("~/.config/quickshell/ — shell.qml, modules, cards")
        .build();
    let btn_power = Button::builder().label(&tr.onb_open_power).valign(gtk4::Align::Center).build();
    let stack_power = stack.clone();
    btn_power.connect_clicked(move |_| {
        stack_power.set_visible_child_name("quickshell");
    });
    row_power.add_suffix(&btn_power);
    group_power.add(&row_power);

    page_dash.add(&group_onboard);
    page_dash.add(&group_power);


    let group_dash_bar = PreferencesGroup::builder()
        .title(&tr.sidebar_statusbar)
        .description(&tr.dash_quick_desc)
        .build();

    let bar_model_dash = StringList::new(&["Waybar", "Quickshell", "Geen"]);
    let row_dash_bar = ComboRow::builder().title(tr.dash_active_bar.as_str()).model(&bar_model_dash).build();
    row_dash_bar.set_selected(match initial_cfg.active_bar.as_str() { "quickshell" => 1, "none" => 2, _ => 0 });
    let st_dbar = Rc::clone(state);
    row_dash_bar.connect_selected_notify(move |r| {
        let choice = match r.selected() { 1 => "quickshell", 2 => "none", _ => "waybar" };
        process::set_active_bar(choice);
        st_dbar.borrow_mut().active_bar = choice.to_string();
        config::save_config(&st_dbar.borrow());
    });
    group_dash_bar.add(&row_dash_bar);

    // Wallpaper Selector & Palette Hook
    let current_wp = if !shell_cfg.wallpaper_path.is_empty() && is_valid_image(&shell_cfg.wallpaper_path) {
        shell_cfg.wallpaper_path.clone()
    } else {
        detect_current_wallpaper().unwrap_or_default()
    };
    let row_dash_wall = ActionRow::builder()
        .title(tr.dash_wallpaper.as_str())
        .subtitle(if current_wp.is_empty() { &tr.dash_wallpaper_sub } else { &current_wp })
        .build();

    // ========================================================
    // Wallpaper Palette Engine & Kleurenoverzicht
    // ========================================================
    let group_palette = PreferencesGroup::builder()
        .title(&tr.palette_title)
        .description(&tr.palette_desc)
        .build();

    let row_auto_pal = ActionRow::builder()
        .title(&tr.palette_auto_sync)
        .subtitle(&tr.palette_auto_sync_sub)
        .build();
    let sw_auto_pal = Switch::builder()
        .active(shell_cfg.auto_palette)
        .valign(gtk4::Align::Center)
        .build();

    // Live Kleurenoverzicht (Swatches)
    let row_swatches = ActionRow::builder()
        .title(&tr.palette_current)
        .subtitle(format!("Accent: {} • Achtergrond: {}", shell_cfg.styling.accent, shell_cfg.styling.background))
        .build();

    let swatch_box = Box::new(Orientation::Horizontal, 6);
    update_swatches_chips(
        &swatch_box,
        &shell_cfg.styling.background,
        &shell_cfg.styling.pill_bg,
        &shell_cfg.styling.accent,
        &shell_cfg.styling.border_color,
        &shell_cfg.styling.text_color,
    );
    row_swatches.add_suffix(&swatch_box);

    let btn_dash_wall = Button::builder().label(&tr.dash_choose_file).valign(gtk4::Align::Center).build();
    let r_wall_dash = row_dash_wall.clone();
    let r_sw_dash = row_swatches.clone();
    let sw_box_dash = swatch_box.clone();
    let choose_wall_title = tr.dash_choose_wallpaper_title.clone();
    btn_dash_wall.connect_clicked(move |_| {
        let fd = FileDialog::builder().title(&choose_wall_title).build();
        let r_cl = r_wall_dash.clone();
        let r_sw_cl = r_sw_dash.clone();
        let sw_b_cl = sw_box_dash.clone();
        fd.open(None::<&gtk4::Window>, None::<&gtk4::gio::Cancellable>, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    if let Some(p_str) = path.to_str() {
                        if is_valid_image(p_str) {
                            hyprland::set_wallpaper(p_str);
                            r_cl.set_subtitle(p_str);
                            let mut scfg = ZenithShellConfig::load_or_default();
                            scfg.wallpaper_path = p_str.to_string();
                            let _ = scfg.save();
                            if scfg.auto_palette {
                                if let Some(pal) = palette::extract_palette(p_str) {
                                    palette::apply_palette(&pal);
                                    r_sw_cl.set_subtitle(&format!("Accent: {} • Achtergrond: {}", pal.primary_accent, pal.background));
                                    update_swatches_chips(&sw_b_cl, &pal.background, &pal.surface, &pal.primary_accent, &pal.surface, &pal.foreground);
                                }
                            }
                        }
                    }
                }
            }
        });
    });
    row_dash_wall.add_suffix(&btn_dash_wall);
    group_dash_bar.add(&row_dash_wall);
    page_dash.add(&group_dash_bar);

    let r_wall_auto = row_dash_wall.clone();
    let r_sw_auto = row_swatches.clone();
    let sw_box_auto = swatch_box.clone();
    sw_auto_pal.connect_active_notify(move |sw| {
        let mut scfg = ZenithShellConfig::load_or_default();
        scfg.auto_palette = sw.is_active();

        if sw.is_active() {
            let target = if !scfg.wallpaper_path.is_empty() && is_valid_image(&scfg.wallpaper_path) {
                Some(scfg.wallpaper_path.clone())
            } else {
                detect_current_wallpaper()
            };
            if let Some(wp) = target {
                scfg.wallpaper_path = wp.clone();
                let _ = scfg.save();
                hyprland::set_wallpaper(&wp);
                r_wall_auto.set_subtitle(&wp);
                if let Some(pal) = palette::extract_palette(&wp) {
                    palette::apply_palette(&pal);
                    r_sw_auto.set_subtitle(&format!("Accent: {} • Achtergrond: {}", pal.primary_accent, pal.background));
                    update_swatches_chips(&sw_box_auto, &pal.background, &pal.surface, &pal.primary_accent, &pal.surface, &pal.foreground);
                }
            } else {
                let _ = scfg.save();
            }
        } else {
            let _ = scfg.save();
        }
    });
    row_auto_pal.add_suffix(&sw_auto_pal);
    group_palette.add(&row_auto_pal);
    group_palette.add(&row_swatches);

    let row_gen_pal = ActionRow::builder()
        .title(&tr.palette_generate)
        .subtitle(&tr.palette_gen_sub)
        .build();
    let btn_gen_pal = Button::builder().label(&tr.palette_gen_btn).valign(gtk4::Align::Center).build();
    let r_sw_gen = row_swatches.clone();
    let r_wall_gen = row_dash_wall.clone();
    let sw_box_gen = swatch_box.clone();
    let choose_wall_title_gen = tr.dash_choose_wallpaper_title.clone();
    btn_gen_pal.connect_clicked(move |_| {
        let mut scfg = ZenithShellConfig::load_or_default();
        let target = if !scfg.wallpaper_path.is_empty() && is_valid_image(&scfg.wallpaper_path) {
            Some(scfg.wallpaper_path.clone())
        } else {
            detect_current_wallpaper()
        };

        if let Some(wp) = target {
            scfg.wallpaper_path = wp.clone();
            let _ = scfg.save();
            hyprland::set_wallpaper(&wp);
            r_wall_gen.set_subtitle(&wp);
            if let Some(pal) = palette::extract_palette(&wp) {
                palette::apply_palette(&pal);
                r_sw_gen.set_subtitle(&format!("Accent: {} • Achtergrond: {}", pal.primary_accent, pal.background));
                update_swatches_chips(&sw_box_gen, &pal.background, &pal.surface, &pal.primary_accent, &pal.surface, &pal.foreground);
            }
        } else {
            let fd = FileDialog::builder().title(&choose_wall_title_gen).build();
            let r_sw2 = r_sw_gen.clone();
            let r_w2 = r_wall_gen.clone();
            let sw_b2 = sw_box_gen.clone();
            fd.open(None::<&gtk4::Window>, None::<&gtk4::gio::Cancellable>, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        if let Some(p_str) = path.to_str() {
                            if is_valid_image(p_str) {
                                hyprland::set_wallpaper(p_str);
                                r_w2.set_subtitle(p_str);
                                let mut scfg2 = ZenithShellConfig::load_or_default();
                                scfg2.wallpaper_path = p_str.to_string();
                                let _ = scfg2.save();
                                if let Some(pal) = palette::extract_palette(p_str) {
                                    palette::apply_palette(&pal);
                                    r_sw2.set_subtitle(&format!("Accent: {} • Achtergrond: {}", pal.primary_accent, pal.background));
                                    update_swatches_chips(&sw_b2, &pal.background, &pal.surface, &pal.primary_accent, &pal.surface, &pal.foreground);
                                }
                            }
                        }
                    }
                }
            });
        }
    });
    row_gen_pal.add_suffix(&btn_gen_pal);
    group_palette.add(&row_gen_pal);

    page_dash.add(&group_palette);

    stack.add_titled(&page_dash, Some("dashboard"), "Dashboard");

    // ========================================================
    // PAGINA 2: Hyprland (Geometry, Opacity, Shaders & Motion, Displays)
    // ========================================================
    let page_hypr = PreferencesPage::new();

    let group_geom = PreferencesGroup::builder().title(crate::ui::escape::pango_escape(&tr.hypr_geometry)).description(&tr.hypr_geom_desc).build();

    let row_out = ActionRow::builder().title(&tr.hypr_outer_gaps).subtitle(format!("{} px", initial_cfg.gaps_out)).build();
    let s_out = Scale::with_range(Orientation::Horizontal, 0.0, 40.0, 1.0);
    s_out.set_draw_value(false);
    s_out.set_value(initial_cfg.gaps_out as f64);
    s_out.set_width_request(160);
    let r_out_c = row_out.clone();
    let st_out = Rc::clone(state);
    s_out.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_out_c.set_subtitle(&format!("{} px", v));
        hyprland::set_gaps_out(v);
        st_out.borrow_mut().gaps_out = v;
        config::save_config(&st_out.borrow());
    });
    row_out.add_suffix(&s_out);
    group_geom.add(&row_out);

    let row_in = ActionRow::builder().title(&tr.hypr_inner_gaps).subtitle(format!("{} px", initial_cfg.gaps_in)).build();
    let s_in = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_in.set_draw_value(false);
    s_in.set_value(initial_cfg.gaps_in as f64);
    s_in.set_width_request(160);
    let r_in_c = row_in.clone();
    let st_in = Rc::clone(state);
    s_in.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_in_c.set_subtitle(&format!("{} px", v));
        hyprland::set_gaps_in(v);
        st_in.borrow_mut().gaps_in = v;
        config::save_config(&st_in.borrow());
    });
    row_in.add_suffix(&s_in);
    group_geom.add(&row_in);

    let row_border = ActionRow::builder().title(&tr.hypr_border_width).subtitle(format!("{} px", initial_cfg.border_size)).build();
    let s_border = Scale::with_range(Orientation::Horizontal, 0.0, 10.0, 1.0);
    s_border.set_draw_value(false);
    s_border.set_value(initial_cfg.border_size as f64);
    s_border.set_width_request(160);
    let r_b_c = row_border.clone();
    let st_border = Rc::clone(state);
    s_border.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_b_c.set_subtitle(&format!("{} px", v));
        hyprland::set_border_size(v);
        st_border.borrow_mut().border_size = v;
        config::save_config(&st_border.borrow());
    });
    row_border.add_suffix(&s_border);
    group_geom.add(&row_border);

    let row_round = ActionRow::builder().title(&tr.hypr_corner_rounding).subtitle(format!("{} px", initial_cfg.rounding)).build();
    let s_round = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_round.set_draw_value(false);
    s_round.set_value(initial_cfg.rounding as f64);
    s_round.set_width_request(160);
    let r_rnd_c = row_round.clone();
    let st_rnd = Rc::clone(state);
    s_round.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_rnd_c.set_subtitle(&format!("{} px", v));
        hyprland::set_rounding(v);
        st_rnd.borrow_mut().rounding = v;
        config::save_config(&st_rnd.borrow());
    });
    row_round.add_suffix(&s_round);
    group_geom.add(&row_round);

    let row_color = ActionRow::builder().title(&tr.hypr_border_color).subtitle("Border color").build();
    let c_dialog = ColorDialog::builder().title("Border Color").with_alpha(false).build();
    let c_btn = ColorDialogButton::builder().dialog(&c_dialog).valign(gtk4::Align::Center).build();
    c_btn.set_rgba(&hex_to_rgba(&initial_cfg.active_border_color));
    let st_col = Rc::clone(state);
    c_btn.connect_notify_local(Some("rgba"), move |b, _| {
        let r = b.rgba();
        let hex = format!("{:02x}{:02x}{:02x}", (r.red() * 255.0) as u8, (r.green() * 255.0) as u8, (r.blue() * 255.0) as u8);
        hyprland::set_active_border_color(&hex);
        st_col.borrow_mut().active_border_color = hex;
        config::save_config(&st_col.borrow());
    });
    row_color.add_suffix(&c_btn);
    group_geom.add(&row_color);

    page_hypr.add(&group_geom);

    let group_opacity = PreferencesGroup::builder().title(&tr.hypr_transparency).description(&tr.hypr_trans_desc).build();

    let row_act_op = ActionRow::builder().title(&tr.hypr_active_opacity).subtitle(format!("{:.0}%", initial_cfg.active_opacity * 100.0)).build();
    let s_act_op = Scale::with_range(Orientation::Horizontal, 0.2, 1.0, 0.05);
    s_act_op.set_draw_value(false);
    s_act_op.set_value(initial_cfg.active_opacity);
    s_act_op.set_width_request(160);
    let r_aop_c = row_act_op.clone();
    let st_aop = Rc::clone(state);
    s_act_op.connect_value_changed(move |s| {
        let v = s.value();
        r_aop_c.set_subtitle(&format!("{:.0}%", v * 100.0));
        hyprland::set_active_opacity(v);
        st_aop.borrow_mut().active_opacity = v;
        config::save_config(&st_aop.borrow());
    });
    row_act_op.add_suffix(&s_act_op);
    group_opacity.add(&row_act_op);

    let row_inact_op = ActionRow::builder().title(&tr.hypr_inactive_opacity).subtitle(format!("{:.0}%", initial_cfg.inactive_opacity * 100.0)).build();
    let s_inact_op = Scale::with_range(Orientation::Horizontal, 0.2, 1.0, 0.05);
    s_inact_op.set_draw_value(false);
    s_inact_op.set_value(initial_cfg.inactive_opacity);
    s_inact_op.set_width_request(160);
    let r_iop_c = row_inact_op.clone();
    let st_iop = Rc::clone(state);
    s_inact_op.connect_value_changed(move |s| {
        let v = s.value();
        r_iop_c.set_subtitle(&format!("{:.0}%", v * 100.0));
        hyprland::set_inactive_opacity(v);
        st_iop.borrow_mut().inactive_opacity = v;
        config::save_config(&st_iop.borrow());
    });
    row_inact_op.add_suffix(&s_inact_op);
    group_opacity.add(&row_inact_op);
    page_hypr.add(&group_opacity);

    let group_fx = PreferencesGroup::builder().title(crate::ui::escape::pango_escape(&tr.hypr_effects)).description(&tr.hypr_effects_desc).build();

    let row_blur = ActionRow::builder().title(&tr.hypr_blur).subtitle("Blur background").build();
    let sw_blur = Switch::builder().active(initial_cfg.blur_enabled).valign(gtk4::Align::Center).build();
    let st_blur = Rc::clone(state);
    sw_blur.connect_state_set(move |_, active| {
        hyprland::set_blur_enabled(active);
        st_blur.borrow_mut().blur_enabled = active;
        config::save_config(&st_blur.borrow());
        gtk4::glib::Propagation::Proceed
    });
    row_blur.add_suffix(&sw_blur);
    group_fx.add(&row_blur);

    let row_bsize = ActionRow::builder().title(&tr.hypr_blur_intensity).subtitle(format!("{} px", initial_cfg.blur_size)).build();
    let s_bsize = Scale::with_range(Orientation::Horizontal, 1.0, 20.0, 1.0);
    s_bsize.set_draw_value(false);
    s_bsize.set_value(initial_cfg.blur_size as f64);
    s_bsize.set_width_request(160);
    let r_bs_c = row_bsize.clone();
    let st_bs = Rc::clone(state);
    s_bsize.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_bs_c.set_subtitle(&format!("{} px", v));
        hyprland::set_blur_size(v);
        st_bs.borrow_mut().blur_size = v;
        config::save_config(&st_bs.borrow());
    });
    row_bsize.add_suffix(&s_bsize);
    group_fx.add(&row_bsize);

    let row_shd = ActionRow::builder().title(&tr.hypr_shadows).subtitle("Window shadow").build();
    let sw_shd = Switch::builder().active(initial_cfg.shadow_enabled).valign(gtk4::Align::Center).build();
    let st_shd = Rc::clone(state);
    sw_shd.connect_state_set(move |_, active| {
        hyprland::set_shadow_enabled(active);
        st_shd.borrow_mut().shadow_enabled = active;
        config::save_config(&st_shd.borrow());
        gtk4::glib::Propagation::Proceed
    });
    row_shd.add_suffix(&sw_shd);
    group_fx.add(&row_shd);

    let row_anim = ActionRow::builder().title(&tr.hypr_animations).subtitle("Window animations").build();
    let sw_anim = Switch::builder().active(initial_cfg.animations_enabled).valign(gtk4::Align::Center).build();
    let st_anim = Rc::clone(state);
    sw_anim.connect_state_set(move |_, active| {
        hyprland::set_animations_enabled(active);
        st_anim.borrow_mut().animations_enabled = active;
        config::save_config(&st_anim.borrow());
        gtk4::glib::Propagation::Proceed
    });
    row_anim.add_suffix(&sw_anim);
    group_fx.add(&row_anim);
    page_hypr.add(&group_fx);

    // Displays
    let detected_monitors = hyprland::get_monitors();
    if !detected_monitors.is_empty() {
        for mon in detected_monitors {
            let mon_name = mon.name.clone();
            let current_mode = format!("{}x{}@{:.2}Hz", mon.width, mon.height, mon.refresh_rate);

            let group_mon = PreferencesGroup::builder()
                .title(format!("{}: {}", tr.hypr_display, mon_name))
                .description(format!("{}: {}", tr.hypr_display_desc, current_mode))
                .build();

            let mut mode_strings: Vec<String> = mon.available_modes.clone();
            if mode_strings.is_empty() {
                mode_strings.push(format!("{}x{}@{:.2}Hz", mon.width, mon.height, mon.refresh_rate));
            }
            let mode_model = StringList::new(&mode_strings.iter().map(|s| s.as_str()).collect::<Vec<&str>>());
            let row_res = ComboRow::builder().title(crate::ui::escape::pango_escape(&tr.hypr_display_mode)).model(&mode_model).build();
            if let Some(pos) = mode_strings.iter().position(|m| m == &current_mode) {
                row_res.set_selected(pos as u32);
            }

            let m_name_c = mon_name.clone();
            let m_strings_c = mode_strings.clone();
            row_res.connect_selected_notify(move |r| {
                let idx = r.selected() as usize;
                if let Some(target_mode) = m_strings_c.get(idx) {
                    hyprland::set_monitor_mode(&m_name_c, target_mode);
                }
            });
            group_mon.add(&row_res);

            let row_scale = ActionRow::builder().title(&tr.hypr_display_scale).subtitle(format!("{:.2}x", mon.scale)).build();
            let s_scale = Scale::with_range(Orientation::Horizontal, 1.0, 2.5, 0.25);
    s_scale.set_draw_value(false);
            s_scale.set_value(mon.scale);
            s_scale.set_width_request(160);
            let r_scl_c = row_scale.clone();
            let m_name_scl = mon_name.clone();
            s_scale.connect_value_changed(move |s| {
                let v = s.value();
                r_scl_c.set_subtitle(&format!("{:.2}x", v));
                hyprland::set_monitor_scale(&m_name_scl, v);
            });
            row_scale.add_suffix(&s_scale);
            group_mon.add(&row_scale);

            page_hypr.add(&group_mon);
        }
    }

    stack.add_titled(&page_hypr, Some("hyprland"), "Hyprland");

    // ========================================================
    // PAGINA 3: Statusbalk (Quickshell Designer + Waybar Fallback)
    // ========================================================
    let page_qs = crate::ui::quickshell_designer::build_quickshell_page(state, &tr);

    let group_wb_layout = PreferencesGroup::builder()
        .title("Waybar Styling (Alternatief)")
        .description("Configuratie voor wanneer Waybar geselecteerd is als statusbalk")
        .build();

    let row_pos = ActionRow::builder().title("Positie op Scherm").build();
    let pos_model = StringList::new(&["top", "bottom"]);
    let dd_pos = DropDown::builder().model(&pos_model).valign(gtk4::Align::Center).build();
    dd_pos.set_selected(if initial_cfg.waybar_position == "bottom" { 1 } else { 0 });
    let st_pos = Rc::clone(state);
    dd_pos.connect_selected_notify(move |d| {
        let pos_str = if d.selected() == 1 { "bottom" } else { "top" };
        let h = st_pos.borrow().waybar_height;
        waybar::update_config(pos_str, h);
        st_pos.borrow_mut().waybar_position = pos_str.to_string();
        config::save_config(&st_pos.borrow());
    });
    row_pos.add_suffix(&dd_pos);
    group_wb_layout.add(&row_pos);

    let row_h = ActionRow::builder().title("Balk Hoogte").subtitle(format!("{} px", initial_cfg.waybar_height)).build();
    let s_h = Scale::with_range(Orientation::Horizontal, 20.0, 56.0, 2.0);
    s_h.set_draw_value(false);
    s_h.set_value(initial_cfg.waybar_height as f64);
    s_h.set_width_request(160);
    let r_h_c = row_h.clone();
    let st_h = Rc::clone(state);
    s_h.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_h_c.set_subtitle(&format!("{} px", v));
        let pos = st_h.borrow().waybar_position.clone();
        waybar::update_config(&pos, v);
        st_h.borrow_mut().waybar_height = v;
        config::save_config(&st_h.borrow());
    });
    row_h.add_suffix(&s_h);
    group_wb_layout.add(&row_h);

    let row_wb_bg = ActionRow::builder().title("Achtergrondkleur").build();
    let c_wb_dlg = ColorDialog::builder().title("Waybar Kleur").with_alpha(false).build();
    let c_wb_btn = ColorDialogButton::builder().dialog(&c_wb_dlg).valign(gtk4::Align::Center).build();
    c_wb_btn.set_rgba(&hex_to_rgba(&initial_cfg.waybar_bg_color));
    let st_wb_bg = Rc::clone(state);
    c_wb_btn.connect_notify_local(Some("rgba"), move |b, _| {
        let r = b.rgba();
        let hex = format!("{:02x}{:02x}{:02x}", (r.red() * 255.0) as u8, (r.green() * 255.0) as u8, (r.blue() * 255.0) as u8);
        let op = st_wb_bg.borrow().waybar_opacity;
        let rnd = st_wb_bg.borrow().waybar_rounding;
        waybar::update_style(&hex, op, rnd);
        st_wb_bg.borrow_mut().waybar_bg_color = hex;
        config::save_config(&st_wb_bg.borrow());
    });
    row_wb_bg.add_suffix(&c_wb_btn);
    group_wb_layout.add(&row_wb_bg);

    let row_wb_rnd = ActionRow::builder().title("Balk Afronding").subtitle(format!("{} px", initial_cfg.waybar_rounding)).build();
    let s_wb_rnd = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_wb_rnd.set_draw_value(false);
    s_wb_rnd.set_value(initial_cfg.waybar_rounding as f64);
    s_wb_rnd.set_width_request(160);
    let r_wrnd_c = row_wb_rnd.clone();
    let st_wb_rnd = Rc::clone(state);
    s_wb_rnd.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_wrnd_c.set_subtitle(&format!("{} px", v));
        let hex = st_wb_rnd.borrow().waybar_bg_color.clone();
        let op = st_wb_rnd.borrow().waybar_opacity;
        waybar::update_style(&hex, op, v);
        st_wb_rnd.borrow_mut().waybar_rounding = v;
        config::save_config(&st_wb_rnd.borrow());
    });
    row_wb_rnd.add_suffix(&s_wb_rnd);
    group_wb_layout.add(&row_wb_rnd);

    page_qs.add(&group_wb_layout);
    stack.add_titled(&page_qs, Some("statusbar"), &tr.sidebar_statusbar);

    // ========================================================
    // PAGINA 4: Control Center Designer
    // ========================================================
    let page_cc = crate::ui::control_center_designer::build_control_center_page(state, &tr);
    stack.add_titled(&page_cc, Some("control_center"), &tr.sidebar_control_center);

    // ========================================================
    // PAGINA 5: On-Screen Display (OSD) Designer
    // ========================================================
    let page_osd = crate::ui::osd_designer::build_osd_page(state, &tr);
    stack.add_titled(&page_osd, Some("osd"), &tr.sidebar_osd);

    // ========================================================
    // PAGINA 6: Lockscreen Studio
    // ========================================================
    let page_lockscreen = crate::ui::lockscreen_designer::build_lockscreen_page(state, &tr);
    stack.add_titled(&page_lockscreen, Some("lockscreen"), &tr.sidebar_lockscreen);

    // ========================================================
    // PAGINA 7: Thema's & Apps (Kitty, Rofi, Standaard Applicaties)
    // ========================================================
    let page_themes = PreferencesPage::new();

    let group_kitty = PreferencesGroup::builder().title("Kitty Terminal").description(&tr.theme_kitty_desc).build();
    
    let row_k_bg = ActionRow::builder().title(&tr.theme_bg_color).build();
    let dlg_k_bg = ColorDialog::builder().title("Kitty").with_alpha(false).build();
    let btn_k_bg = ColorDialogButton::builder().dialog(&dlg_k_bg).valign(gtk4::Align::Center).build();
    btn_k_bg.set_rgba(&hex_to_rgba(&initial_cfg.kitty_bg));
    let st_k_bg = Rc::clone(state);
    btn_k_bg.connect_notify_local(Some("rgba"), move |b, _| {
        let r = b.rgba();
        let hex = format!("{:02x}{:02x}{:02x}", (r.red() * 255.0) as u8, (r.green() * 255.0) as u8, (r.blue() * 255.0) as u8);
        let fg = st_k_bg.borrow().kitty_fg.clone();
        let op = st_k_bg.borrow().kitty_opacity;
        let fs = st_k_bg.borrow().kitty_font_size;
        themes::update_kitty(&hex, &fg, op, fs);
        st_k_bg.borrow_mut().kitty_bg = hex;
        config::save_config(&st_k_bg.borrow());
    });
    row_k_bg.add_suffix(&btn_k_bg);
    group_kitty.add(&row_k_bg);

    let row_k_op = ActionRow::builder().title(&tr.theme_window_trans).subtitle(format!("{:.0}%", initial_cfg.kitty_opacity * 100.0)).build();
    let s_k_op = Scale::with_range(Orientation::Horizontal, 0.4, 1.0, 0.05);
    s_k_op.set_draw_value(false);
    s_k_op.set_value(initial_cfg.kitty_opacity);
    s_k_op.set_width_request(160);
    let r_kop_c = row_k_op.clone();
    let st_k_op = Rc::clone(state);
    s_k_op.connect_value_changed(move |s| {
        let v = s.value();
        r_kop_c.set_subtitle(&format!("{:.0}%", v * 100.0));
        let bg = st_k_op.borrow().kitty_bg.clone();
        let fg = st_k_op.borrow().kitty_fg.clone();
        let fs = st_k_op.borrow().kitty_font_size;
        themes::update_kitty(&bg, &fg, v, fs);
        st_k_op.borrow_mut().kitty_opacity = v;
        config::save_config(&st_k_op.borrow());
    });
    row_k_op.add_suffix(&s_k_op);
    group_kitty.add(&row_k_op);

    let row_k_font = ActionRow::builder().title(&tr.theme_font_size).subtitle(format!("{:.1} pt", initial_cfg.kitty_font_size)).build();
    let s_k_font = Scale::with_range(Orientation::Horizontal, 8.0, 20.0, 0.5);
    s_k_font.set_draw_value(false);
    s_k_font.set_value(initial_cfg.kitty_font_size);
    s_k_font.set_width_request(160);
    let r_kfont_c = row_k_font.clone();
    let st_k_font = Rc::clone(state);
    s_k_font.connect_value_changed(move |s| {
        let v = s.value();
        r_kfont_c.set_subtitle(&format!("{:.1} pt", v));
        let bg = st_k_font.borrow().kitty_bg.clone();
        let fg = st_k_font.borrow().kitty_fg.clone();
        let op = st_k_font.borrow().kitty_opacity;
        themes::update_kitty(&bg, &fg, op, v);
        st_k_font.borrow_mut().kitty_font_size = v;
        config::save_config(&st_k_font.borrow());
    });
    row_k_font.add_suffix(&s_k_font);
    group_kitty.add(&row_k_font);
    page_themes.add(&group_kitty);

    let group_rofi = PreferencesGroup::builder().title("Rofi Menu").description(&tr.theme_rofi_desc).build();
    
    let row_r_bg = ActionRow::builder().title(&tr.theme_bg_color).build();
    let dlg_r_bg = ColorDialog::builder().title("Rofi").with_alpha(false).build();
    let btn_r_bg = ColorDialogButton::builder().dialog(&dlg_r_bg).valign(gtk4::Align::Center).build();
    btn_r_bg.set_rgba(&hex_to_rgba(&initial_cfg.rofi_bg));
    let st_r_bg = Rc::clone(state);
    btn_r_bg.connect_notify_local(Some("rgba"), move |b, _| {
        let r = b.rgba();
        let hex = format!("{:02x}{:02x}{:02x}", (r.red() * 255.0) as u8, (r.green() * 255.0) as u8, (r.blue() * 255.0) as u8);
        let fg = st_r_bg.borrow().rofi_fg.clone();
        let op = st_r_bg.borrow().rofi_opacity;
        let rnd = st_r_bg.borrow().rofi_rounding;
        themes::update_rofi(&hex, &fg, op, rnd);
        st_r_bg.borrow_mut().rofi_bg = hex;
        config::save_config(&st_r_bg.borrow());
    });
    row_r_bg.add_suffix(&btn_r_bg);
    group_rofi.add(&row_r_bg);

    let row_r_rnd = ActionRow::builder().title(&tr.theme_rounding).subtitle(format!("{} px", initial_cfg.rofi_rounding)).build();
    let s_r_rnd = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_r_rnd.set_draw_value(false);
    s_r_rnd.set_value(initial_cfg.rofi_rounding as f64);
    s_r_rnd.set_width_request(160);
    let r_rrnd_c = row_r_rnd.clone();
    let st_r_rnd = Rc::clone(state);
    s_r_rnd.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_rrnd_c.set_subtitle(&format!("{} px", v));
        let bg = st_r_rnd.borrow().rofi_bg.clone();
        let fg = st_r_rnd.borrow().rofi_fg.clone();
        let op = st_r_rnd.borrow().rofi_opacity;
        themes::update_rofi(&bg, &fg, op, v);
        st_r_rnd.borrow_mut().rofi_rounding = v;
        config::save_config(&st_r_rnd.borrow());
    });
    row_r_rnd.add_suffix(&s_r_rnd);
    group_rofi.add(&row_r_rnd);
    page_themes.add(&group_rofi);

    let group_tools = PreferencesGroup::builder()
        .title(&tr.theme_apps)
        .description("Default Applications")
        .build();

    let launcher_model = StringList::new(&["Rofi", "Wofi"]);
    let row_launcher = ComboRow::builder().title("Menu ($menu)").model(&launcher_model).build();
    row_launcher.set_selected(if initial_cfg.default_launcher == "wofi" { 1 } else { 0 });
    let st_lnc = Rc::clone(state);
    row_launcher.connect_selected_notify(move |r| {
        let choice = if r.selected() == 1 { "wofi" } else { "rofi" };
        process::ensure_launcher_installed(choice);
        st_lnc.borrow_mut().default_launcher = choice.to_string();
        config::save_config(&st_lnc.borrow());
    });
    group_tools.add(&row_launcher);

    let term_model = StringList::new(&["Kitty", "Alacritty", "Foot"]);
    let row_term = ComboRow::builder().title("Terminal ($terminal)").model(&term_model).build();
    row_term.set_selected(match initial_cfg.default_terminal.as_str() { "alacritty" => 1, "foot" => 2, _ => 0 });
    let st_trm = Rc::clone(state);
    row_term.connect_selected_notify(move |r| {
        let choice = match r.selected() { 1 => "alacritty", 2 => "foot", _ => "kitty" };
        process::ensure_terminal_installed(choice);
        st_trm.borrow_mut().default_terminal = choice.to_string();
        config::save_config(&st_trm.borrow());
    });
    group_tools.add(&row_term);

    let shell_model = StringList::new(&["Zsh", "Bash", "Fish"]);
    let row_shell = ComboRow::builder().title("Default User Shell").model(&shell_model).build();
    row_shell.set_selected(match initial_cfg.default_shell.as_str() { "bash" => 1, "fish" => 2, _ => 0 });
    let st_shl = Rc::clone(state);
    row_shell.connect_selected_notify(move |r| {
        let choice = match r.selected() { 1 => "bash", 2 => "fish", _ => "zsh" };
        process::set_user_shell(choice);
        st_shl.borrow_mut().default_shell = choice.to_string();
        config::save_config(&st_shl.borrow());
    });
    group_tools.add(&row_shell);

    page_themes.add(&group_tools);

    // GTK / systeemthema
    let group_gtk = PreferencesGroup::builder().title(&tr.theme_gtk_theme).build();

    let row_gtk_theme = ActionRow::builder().title(&tr.theme_gtk_theme).build();
    let ent_gtk_theme = Entry::builder().valign(gtk4::Align::Center).text(gsettings_get("gtk-theme").as_str()).build();
    ent_gtk_theme.set_width_chars(14);
    ent_gtk_theme.connect_activate(move |e| {
        gsettings_set("gtk-theme", e.text().as_str());
    });
    row_gtk_theme.add_suffix(&ent_gtk_theme);
    group_gtk.add(&row_gtk_theme);

    let row_icon_theme = ActionRow::builder().title(&tr.theme_icon_theme).build();
    let ent_icon_theme = Entry::builder().valign(gtk4::Align::Center).text(gsettings_get("icon-theme").as_str()).build();
    ent_icon_theme.set_width_chars(14);
    ent_icon_theme.connect_activate(move |e| {
        gsettings_set("icon-theme", e.text().as_str());
    });
    row_icon_theme.add_suffix(&ent_icon_theme);
    group_gtk.add(&row_icon_theme);

    let row_cursor = ActionRow::builder().title(&tr.theme_cursor).build();
    let ent_cursor = Entry::builder().valign(gtk4::Align::Center).text(gsettings_get("cursor-theme").as_str()).build();
    ent_cursor.set_width_chars(14);
    ent_cursor.connect_activate(move |e| {
        gsettings_set("cursor-theme", e.text().as_str());
    });
    row_cursor.add_suffix(&ent_cursor);
    group_gtk.add(&row_cursor);

    let row_font = ActionRow::builder().title(&tr.theme_font).build();
    let ent_font = Entry::builder().valign(gtk4::Align::Center).text(gsettings_get("font-name").as_str()).build();
    ent_font.set_width_chars(16);
    ent_font.connect_activate(move |e| {
        gsettings_set("font-name", e.text().as_str());
    });
    row_font.add_suffix(&ent_font);
    group_gtk.add(&row_font);

    page_themes.add(&group_gtk);

    // Wijzigingen toepassen
    let group_apply = PreferencesGroup::builder().title(&tr.theme_apply).build();
    let row_apply = ActionRow::builder()
        .title(&tr.common_apply)
        .subtitle(&tr.theme_apply)
        .build();
    let btn_apply = Button::builder().label(&tr.theme_apply).valign(gtk4::Align::Center).build();
    let st_apply = Rc::clone(state);
    let common_apply_label = tr.common_apply.clone();
    btn_apply.connect_clicked(move |b| {
        config::save_config(&st_apply.borrow());
        themes::update_quickshell(&st_apply.borrow());
        waybar::reload_waybar();
        b.set_label(&common_apply_label);
    });
    row_apply.add_suffix(&btn_apply);
    group_apply.add(&row_apply);
    page_themes.add(&group_apply);

    stack.add_titled(&page_themes, Some("themes"), &crate::ui::escape::pango_escape(&tr.sidebar_themes));

    // ========================================================
    // PAGINA 7: Iconen & Emoji's
    // ========================================================
    let page_icons = crate::ui::icon_studio::build_icon_studio_page(&tr);
    stack.add_titled(&page_icons, Some("icons"), &crate::ui::escape::pango_escape(&tr.icons_title));

    // ========================================================
    // PAGINA 8: Fastfetch Visual Studio
    // ========================================================
    let page_fastfetch = crate::ui::fastfetch_designer::build_fastfetch_page(&tr);
    stack.add_titled(&page_fastfetch, Some("fastfetch"), &tr.sidebar_fastfetch);

    // ========================================================
    // PAGINA 9: Systeem & Tools
    // ========================================================
    let page_system = PreferencesPage::new();
    let group_sys = PreferencesGroup::builder().title(crate::ui::escape::pango_escape(&tr.sidebar_system)).build();

    let row_dnd = ActionRow::builder().title(&tr.sys_dnd_title).subtitle(&tr.sys_dnd_sub).build();
    let sw_dnd = Switch::builder().valign(gtk4::Align::Center).build();
    sw_dnd.connect_state_set(|_, active| {
        process::toggle_dunst_dnd(active);
        gtk4::glib::Propagation::Proceed
    });
    row_dnd.add_suffix(&sw_dnd);
    group_sys.add(&row_dnd);

    let row_wall = ActionRow::builder()
        .title(tr.dash_wallpaper.as_str())
        .subtitle(if current_wp.is_empty() { &tr.dash_wallpaper_sub } else { &current_wp })
        .build();
    let btn_wall = Button::builder().label(&tr.dash_choose_file).valign(gtk4::Align::Center).build();
    let r_wall_sys = row_wall.clone();
    let choose_wall_sys_title = tr.dash_choose_wallpaper_title.clone();
    btn_wall.connect_clicked(move |_| {
        let fd = FileDialog::builder().title(&choose_wall_sys_title).build();
        let r_cl2 = r_wall_sys.clone();
        fd.open(None::<&gtk4::Window>, None::<&gtk4::gio::Cancellable>, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    if let Some(p_str) = path.to_str() {
                        if is_valid_image(p_str) {
                            hyprland::set_wallpaper(p_str);
                            r_cl2.set_subtitle(p_str);
                            let mut scfg = ZenithShellConfig::load_or_default();
                            scfg.wallpaper_path = p_str.to_string();
                            let _ = scfg.save();
                            if scfg.auto_palette {
                                if let Some(pal) = palette::extract_palette(p_str) {
                                    palette::apply_palette(&pal);
                                }
                            }
                        }
                    }
                }
            }
        });
    });
    row_wall.add_suffix(&btn_wall);
    group_sys.add(&row_wall);
    page_system.add(&group_sys);

    // Toetsenbord sneltoetsen
    let group_keys = PreferencesGroup::builder().title(&tr.sys_keybinds).build();
    let row_keys = ActionRow::builder()
        .title(&tr.sys_keybinds)
        .subtitle("~/.config/hypr/hyprland.conf")
        .build();
    let btn_keys = Button::builder().label(&tr.common_open_folder).valign(gtk4::Align::Center).build();
    let cfg_path_home = std::env::var("HOME").unwrap_or_default();
    let hypr_conf_path = std::path::PathBuf::from(&cfg_path_home).join(".config/hypr/hyprland.conf");
    btn_keys.connect_clicked(move |_| {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "xdg-open".to_string());
        process::execute_cmd(&format!(
            "{} {} &",
            editor,
            hypr_conf_path.to_string_lossy()
        ));
    });
    row_keys.add_suffix(&btn_keys);
    group_keys.add(&row_keys);
    page_system.add(&group_keys);

    // Autostart applicaties
    let group_autostart = PreferencesGroup::builder().title(&tr.sys_autostart).build();
    let row_autostart = ActionRow::builder()
        .title(&tr.sys_autostart)
        .subtitle("~/.config/autostart")
        .build();
    let btn_autostart = Button::builder().label(&tr.common_open_folder).valign(gtk4::Align::Center).build();
    let autostart_dir = std::path::PathBuf::from(&cfg_path_home).join(".config/autostart");
    if !autostart_dir.exists() {
        let _ = std::fs::create_dir_all(&autostart_dir);
    }
    let autostart_dir_clone = autostart_dir.clone();
    btn_autostart.connect_clicked(move |_| {
        process::execute_cmd(&format!("xdg-open '{}' &", autostart_dir_clone.to_string_lossy()));
    });
    row_autostart.add_suffix(&btn_autostart);
    group_autostart.add(&row_autostart);
    page_system.add(&group_autostart);

    // Monitor configuratie
    let group_monitor = PreferencesGroup::builder().title(&tr.sys_monitor).build();
    let mon_count = hyprland::get_monitors().len();
    let row_monitor = ActionRow::builder()
        .title(&tr.sys_monitor)
        .subtitle(format!("{} {}", mon_count, tr.hypr_display))
        .build();
    let btn_monitor = Button::builder()
        .label(tr.hypr_display.as_str())
        .valign(gtk4::Align::Center)
        .build();
    let stack_to_monitor = stack.clone();
    btn_monitor.connect_clicked(move |_| {
        stack_to_monitor.set_visible_child_name("hyprland");
    });
    row_monitor.add_suffix(&btn_monitor);
    group_monitor.add(&row_monitor);
    page_system.add(&group_monitor);

    // Back-up & Herstel (snelkoppeling naar de back-up-pagina)
    let group_backup = PreferencesGroup::builder()
        .title(crate::ui::escape::pango_escape(&tr.sidebar_backup))
        .description(crate::ui::escape::pango_escape(&tr.bk_create_desc))
        .build();
    let row_backup = ActionRow::builder()
        .title(crate::ui::escape::pango_escape(&tr.sidebar_backup))
        .subtitle(format!("{} — {}", tr.bk_create_title, tr.bk_create_desc))
        .build();
    let btn_backup = Button::builder()
        .label("🔄")
        .valign(gtk4::Align::Center)
        .tooltip_text(crate::ui::escape::pango_escape(&tr.bk_action_sub))
        .build();
    let stack_to_backup = stack.clone();
    btn_backup.connect_clicked(move |_| {
        stack_to_backup.set_visible_child_name("backup");
    });
    row_backup.add_suffix(&btn_backup);
    group_backup.add(&row_backup);
    page_system.add(&group_backup);

    // Over Zenith
    let group_about = PreferencesGroup::builder().title(&tr.sys_about).build();
    let row_about = ActionRow::builder()
        .title("Zenith")
        .subtitle(crate::ui::escape::pango_escape(&format!("v{} — Voor Hyprland & Quickshell", env!("CARGO_PKG_VERSION"))))
        .build();
    group_about.add(&row_about);
    page_system.add(&group_about);

    stack.add_titled(&page_system, Some("system"), &crate::ui::escape::pango_escape(&tr.sidebar_system));

    // ========================================================
    // PAGINA: Back-up & Herstel
    // ========================================================
    let page_backup = crate::ui::backup_designer::build_backup_page(&tr);
    stack.add_titled(&page_backup, Some("backup"), &crate::ui::escape::pango_escape(&tr.sidebar_backup));

    // Selecteer initiële pagina in de zijbalk
    let mut selected_index = 0;
    for (i, item) in nav_items.iter().enumerate() {
        if item.id == active_tab {
            selected_index = i as i32;
            break;
        }
    }
    if let Some(target_row) = nav_list.row_at_index(selected_index) {
        nav_list.select_row(Some(&target_row));
        stack.set_visible_child_name(active_tab);
    }

    root_box
}

fn hex_to_rgba(hex: &str) -> RGBA {
    let clean = hex.trim_start_matches('#');
    if clean.len() >= 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(51) as f32 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(204) as f32 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(255) as f32 / 255.0;
        RGBA::builder().red(r).green(g).blue(b).alpha(1.0).build()
    } else {
        RGBA::builder().red(0.2).green(0.8).blue(1.0).alpha(1.0).build()
    }
}

/// Leest een GNOME `interface`-sleutel via gsettings (best-effort).
fn gsettings_get(key: &str) -> String {
    if let Ok(out) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", key])
        .output()
    {
        if let Ok(text) = String::from_utf8(out.stdout) {
            // Verwijder omringende aanhalingstekens van stringwaarden.
            return text.trim().trim_matches('\'').to_string();
        }
    }
    String::new()
}

/// Schrijft een GNOME `interface`-sleutel via gsettings (best-effort).
fn gsettings_set(key: &str, value: &str) {
    let _ = std::process::Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", key, value])
        .spawn();
}