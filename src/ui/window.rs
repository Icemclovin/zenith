use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, ColorDialog, ColorDialogButton, DropDown, Entry, FileDialog, Image, Label, ListBox,
    ListBoxRow, Orientation, Scale, StringList, Switch,
};
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, ComboRow, HeaderBar, NavigationPage,
    NavigationSplitView, PreferencesGroup, PreferencesPage, SwitchRow, Toast, ToastOverlay, ToolbarView,
    ViewStack, WindowTitle,
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
        .default_width(1100)
        .default_height(760)
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

type State = Rc<RefCell<config::ZenithConfig>>;

/// Herbouw het venster (na taalwissel of preset) en blijf op dezelfde pagina.
fn rebuild(window: &ApplicationWindow, app: &Application, state: &State, lang: &i18n::Language, tab: &str) {
    let (win, app, state, lang, tab) = (window.downgrade(), app.clone(), Rc::clone(state), lang.clone(), tab.to_string());
    gtk4::glib::idle_add_local_once(move || {
        if let Some(win) = win.upgrade() {
            win.set_content(Some(&build_window_content(&win, &app, &state, &lang, &tab)));
        }
    });
}

/// Schuifrij met de waarde ernaast; `apply` zet het veld en past het live toe, daarna wordt opgeslagen.
#[allow(clippy::too_many_arguments)]
fn slider_row(
    title: &str, min: f64, max: f64, step: f64, value: f64,
    fmt: impl Fn(f64) -> String + 'static,
    state: &State,
    apply: impl Fn(&mut config::ZenithConfig, f64) + 'static,
) -> ActionRow {
    let row = ActionRow::builder().title(title).build();
    let value_lbl = Label::builder().label(fmt(value)).width_chars(6).xalign(1.0).css_classes(["dim-label", "numeric"]).build();
    let scale = Scale::with_range(Orientation::Horizontal, min, max, step);
    scale.set_draw_value(false);
    scale.set_value(value);
    scale.set_width_request(240);
    scale.update_property(&[gtk4::accessible::Property::Label(title)]);
    let st = Rc::clone(state);
    let lbl = value_lbl.clone();
    scale.connect_value_changed(move |s| {
        let v = if step >= 1.0 { s.value().round() } else { s.value() };
        lbl.set_label(&fmt(v));
        apply(&mut st.borrow_mut(), v);
        config::save_config(&st.borrow());
    });
    row.add_suffix(&scale);
    row.add_suffix(&value_lbl);
    row
}

fn switch_row(title: &str, subtitle: &str, active: bool, state: &State, apply: impl Fn(&mut config::ZenithConfig, bool) + 'static) -> SwitchRow {
    let row = SwitchRow::builder().title(title).subtitle(subtitle).active(active).build();
    let st = Rc::clone(state);
    row.connect_active_notify(move |r| {
        apply(&mut st.borrow_mut(), r.is_active());
        config::save_config(&st.borrow());
    });
    row
}

fn color_row(title: &str, hex: &str, state: &State, apply: impl Fn(&mut config::ZenithConfig, String) + 'static) -> ActionRow {
    let row = ActionRow::builder().title(title).build();
    let btn = ColorDialogButton::builder()
        .dialog(&ColorDialog::builder().title(title).with_alpha(false).build())
        .valign(gtk4::Align::Center)
        .build();
    btn.set_rgba(&hex_to_rgba(hex));
    let st = Rc::clone(state);
    btn.connect_notify_local(Some("rgba"), move |b, _| {
        apply(&mut st.borrow_mut(), rgba_to_hex(&b.rgba()));
        config::save_config(&st.borrow());
    });
    row.add_suffix(&btn);
    row.set_activatable_widget(Some(&btn));
    row
}

fn build_window_content(
    window: &ApplicationWindow,
    app: &Application,
    state: &Rc<RefCell<config::ZenithConfig>>,
    lang: &i18n::Language,
    active_tab: &str,
) -> NavigationSplitView {
    let tr = i18n::get_translations(lang);
    let initial_cfg = state.borrow().clone();
    let shell_cfg = ZenithShellConfig::load_or_default();

    // ========================================================
    // Zijbalk: eigen Adwaita-headerbar + gegroepeerde navigatie
    // ========================================================
    let sidebar = Box::new(Orientation::Vertical, 0);

    // Status Pill Badge (Quickshell Daemon Status)
    let qs_pill = Label::builder()
        .halign(gtk4::Align::Start)
        .margin_start(12)
        .margin_top(6)
        .margin_bottom(6)
        .build();
    let set_pill = {
        let on = format!("● {}", tr.status_active);
        let off = format!("○ {}", tr.status_inactive);
        move |pill: &Label| {
            if process::is_process_running("quickshell") {
                pill.set_label(&on);
                pill.set_css_classes(&["status-pill-active"]);
            } else {
                pill.set_label(&off);
                pill.set_css_classes(&["status-pill-inactive"]);
            }
        }
    };
    set_pill(&qs_pill);
    let pill_weak = qs_pill.downgrade();
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
        let Some(pill) = pill_weak.upgrade() else {
            return gtk4::glib::ControlFlow::Break;
        };
        set_pill(&pill);
        gtk4::glib::ControlFlow::Continue
    });
    sidebar.append(&qs_pill);

    let nav_list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .css_classes(["navigation-sidebar"])
        .build();

    struct NavItem {
        id: &'static str,
        title: String,
        icon: &'static str,
        /// Kopje boven het eerste item van een sectie.
        section: Option<String>,
        /// `true` voor pagina's die vooral voor power users interessant zijn
        /// en worden verborgen in de "basis"-modus.
        advanced: bool,
    }

    let nav_items = [
        NavItem { id: "dashboard", title: tr.sidebar_dashboard.clone(), icon: "view-grid-symbolic", section: None, advanced: false },
        NavItem { id: "hyprland", title: tr.sidebar_hyprland.clone(), icon: "preferences-desktop-display-symbolic", section: Some(tr.nav_group_desktop.clone()), advanced: false },
        NavItem { id: "themes", title: tr.sidebar_themes.clone(), icon: "preferences-desktop-appearance-symbolic", section: None, advanced: false },
        NavItem { id: "keybinds", title: tr.sidebar_keybinds.clone(), icon: "input-keyboard-symbolic", section: None, advanced: false },
        NavItem { id: "statusbar", title: tr.sidebar_statusbar.clone(), icon: "view-continuous-symbolic", section: Some(tr.nav_group_shell.clone()), advanced: false },
        NavItem { id: "control_center", title: tr.sidebar_control_center.clone(), icon: "view-app-grid-symbolic", section: None, advanced: false },
        NavItem { id: "osd", title: tr.sidebar_osd.clone(), icon: "audio-volume-high-symbolic", section: None, advanced: false },
        NavItem { id: "lockscreen", title: tr.sidebar_lockscreen.clone(), icon: "system-lock-screen-symbolic", section: None, advanced: false },
        NavItem { id: "icons", title: tr.icons_title.clone(), icon: "emblem-favorite-symbolic", section: None, advanced: true },
        NavItem { id: "fastfetch", title: tr.sidebar_fastfetch.clone(), icon: "utilities-terminal-symbolic", section: None, advanced: true },
        NavItem { id: "system", title: tr.sidebar_system.clone(), icon: "emblem-system-symbolic", section: Some(tr.nav_group_system.clone()), advanced: false },
        NavItem { id: "backup", title: tr.sidebar_backup.clone(), icon: "document-save-symbolic", section: None, advanced: false },
    ];

    let advanced_init = shell_cfg.advanced_mode;
    let mut advanced_rows: Vec<gtk4::ListBoxRow> = Vec::new();

    for item in &nav_items {
        let row = ListBoxRow::new();
        row.set_widget_name(item.id);
        row.set_visible(!item.advanced || advanced_init);

        let row_box = Box::new(Orientation::Horizontal, 12);
        row_box.append(&Image::from_icon_name(item.icon));
        row_box.append(&Label::builder().label(item.title.as_str()).halign(gtk4::Align::Start).hexpand(true).build());
        row.set_child(Some(&row_box));

        nav_list.append(&row);

        if item.advanced {
            advanced_rows.push(row);
        }
    }

    // Sectiekopjes via een header-func: GTK wist handmatig gezette headers.
    let sections: Vec<(&'static str, String)> = nav_items
        .iter()
        .filter_map(|i| i.section.clone().map(|s| (i.id, s)))
        .collect();
    nav_list.set_header_func(move |row, _| {
        let header = sections.iter().find(|(id, _)| *id == row.widget_name().as_str()).map(|(_, s)| {
            Label::builder().label(s.as_str()).halign(gtk4::Align::Start).css_classes(["zenith-nav-header"]).build()
        });
        row.set_header(header.as_ref());
    });

    let nav_scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .child(&nav_list)
        .build();
    sidebar.append(&nav_scroll);

    // ========================================================
    // Inhoud: headerbar met paginatitel + toasts boven de pagina's
    // ========================================================
    let page_title = WindowTitle::new(&tr.sidebar_dashboard, "");
    let header = HeaderBar::new();
    header.set_title_widget(Some(&page_title));

    let stack = ViewStack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    let toasts = ToastOverlay::new();
    toasts.set_child(Some(&stack));

    let content_view = ToolbarView::new();
    content_view.add_top_bar(&header);
    content_view.set_content(Some(&toasts));

    // Instellingen onderaan de zijbalk: taal + modus
    let prefs_box = Box::new(Orientation::Vertical, 10);
    prefs_box.add_css_class("zenith-sidebar-footer");

    // Language Selector (Direct live UI refresh zonder herstart!)
    let lang_box = Box::new(Orientation::Horizontal, 8);
    let lang_label = Label::builder().label(&tr.lang_select).halign(gtk4::Align::Start).hexpand(true).css_classes(["zenith-dim"]).build();
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
        if let Some(win) = win_weak.upgrade() {
            let tab = stack_for_lang.visible_child_name().unwrap_or_else(|| "dashboard".into());
            rebuild(&win, &app_clone, &state_clone, &new_lang, &tab);
        }
    });

    lang_box.append(&lang_label);
    lang_box.append(&lang_drop);
    prefs_box.append(&lang_box);

    // Modus-schakelaar: Basis (gericht) vs Power User (alles zichtbaar)
    let mode_box = Box::new(Orientation::Horizontal, 8);
    let mode_lbl_col = Box::new(Orientation::Vertical, 2);
    mode_lbl_col.set_hexpand(true);
    mode_lbl_col.append(&Label::builder().label(tr.mode_advanced.as_str()).halign(gtk4::Align::Start).build());
    mode_lbl_col.append(&Label::builder()
        .label(tr.mode_advanced_desc.as_str())
        .halign(gtk4::Align::Start)
        .wrap(true)
        .xalign(0.0)
        .css_classes(["zenith-dim", "caption"])
        .build());
    let sw_mode = Switch::builder().valign(gtk4::Align::Center).active(advanced_init).build();
    mode_box.append(&mode_lbl_col);
    mode_box.append(&sw_mode);
    prefs_box.append(&mode_box);

    prefs_box.append(&Label::builder()
        .label(format!("Zenith v{}", env!("CARGO_PKG_VERSION")))
        .halign(gtk4::Align::Start)
        .css_classes(["zenith-dim", "caption"])
        .build());
    sidebar.append(&prefs_box);

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

    let sidebar_header = HeaderBar::new();
    sidebar_header.set_title_widget(Some(&WindowTitle::new("Zenith", "Hyprland Studio")));
    let sidebar_view = ToolbarView::new();
    sidebar_view.add_top_bar(&sidebar_header);
    sidebar_view.set_content(Some(&sidebar));

    let split = NavigationSplitView::new();
    split.set_min_sidebar_width(240.0);
    split.set_max_sidebar_width(280.0);
    split.set_sidebar(Some(&NavigationPage::new(&sidebar_view, "Zenith")));
    split.set_content(Some(&NavigationPage::new(&content_view, "Zenith Control Center")));

    let stack_clone = stack.clone();
    let titles: Vec<(&'static str, String)> = nav_items.iter().map(|i| (i.id, i.title.clone())).collect();
    nav_list.connect_row_selected(move |_, row| {
        if let Some(r) = row {
            let name = r.widget_name();
            stack_clone.set_visible_child_name(&name);
            if let Some((_, t)) = titles.iter().find(|(id, _)| *id == name.as_str()) {
                page_title.set_title(t);
            }
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
    let btn_cc_test = Button::builder().label(tr.dash_test_cc.as_str()).valign(gtk4::Align::Center).build();
    let (ov, msg) = (toasts.clone(), format!("{} — {}", tr.dash_test_cc, tr.toast_done));
    btn_cc_test.connect_clicked(move |_| {
        process::execute_cmd("quickshell ipc call controlCenter toggle");
        toast(&ov, &msg);
    });
    row_cc_test.add_suffix(&btn_cc_test);
    group_quick.add(&row_cc_test);

    let row_osd_test = ActionRow::builder()
        .title(&tr.dash_osd_title)
        .subtitle(&tr.dash_osd_sub)
        .build();
    let btn_osd_test = Button::builder().label(tr.dash_test_osd.as_str()).valign(gtk4::Align::Center).build();
    let (ov, msg) = (toasts.clone(), format!("{} — {}", tr.dash_test_osd, tr.toast_done));
    btn_osd_test.connect_clicked(move |_| {
        process::execute_cmd("quickshell ipc call osd popup '⚡' 'Zenith OS' 0.85");
        toast(&ov, &msg);
    });
    row_osd_test.add_suffix(&btn_osd_test);
    group_quick.add(&row_osd_test);

    let row_reload_hypr = ActionRow::builder()
        .title(&tr.dash_hypr_title)
        .subtitle(&tr.dash_hypr_sub)
        .build();
    let btn_reload_hypr = Button::builder().label(tr.dash_reload_hypr.as_str()).valign(gtk4::Align::Center).build();
    let (ov, msg) = (toasts.clone(), format!("{} — {}", tr.dash_reload_hypr, tr.toast_done));
    btn_reload_hypr.connect_clicked(move |_| {
        process::execute_cmd("hyprctl reload");
        toast(&ov, &msg);
    });
    row_reload_hypr.add_suffix(&btn_reload_hypr);
    group_quick.add(&row_reload_hypr);

    let row_restart_bar = ActionRow::builder()
        .title(&tr.dash_restart_bar)
        .subtitle("Quickshell / Waybar daemon")
        .build();
    let btn_restart_bar = Button::builder().label(tr.dash_restart_bar.as_str()).valign(gtk4::Align::Center).build();
    let st_rbar = Rc::clone(state);
    let (ov, msg) = (toasts.clone(), format!("{} — {}", tr.dash_restart_bar, tr.toast_done));
    btn_restart_bar.connect_clicked(move |_| {
        let active = st_rbar.borrow().active_bar.clone();
        process::set_active_bar(&active);
        toast(&ov, &msg);
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
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
            let on = ipc_client::is_daemon_running();
            if on {
                row_inner.set_subtitle(sub_on_inner.as_str());
                btn_inner.set_label(btn_off_inner.as_str());
            } else {
                row_inner.set_subtitle(sub_off_inner.as_str());
                btn_inner.set_label(btn_on_inner.as_str());
            }
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
        if dep.is_installed() {
            continue; // alleen tonen wat nog ontbreekt; "alles klaar" staat eronder
        }
        let row = ActionRow::builder()
            .title(dep.title())
            .subtitle(tr.onb_install.as_str())
            .build();
        let btn_i = Button::builder().label(&tr.onb_install).valign(gtk4::Align::Center).css_classes(["suggested-action", "pill"]).build();
        let (ov, msg) = (toasts.clone(), format!("{}: {}…", tr.onb_install, dep.title()));
        btn_i.connect_clicked(move |_| {
            match dep {
                process::ZenithDependency::Quickshell => process::install_quickshell(),
                other => process::install_dependency(other),
            }
            toast(&ov, &msg);
        });
        row.add_suffix(&btn_i);
        group_onboard.add(&row);
    }

    // Eén-knop "Alles installeren" als er nog iets ontbreekt
    if !missing.is_empty() {
        let missing_names: Vec<&str> = missing.iter().map(|d| d.title()).collect();
        let row_install_all = ActionRow::builder()
            .title(&tr.onb_install_all)
            .subtitle(format!("{}: {}", tr.onb_core_status, missing_names.join(", ")))
            .build();
        let btn_all = Button::builder().label(&tr.onb_install_all).valign(gtk4::Align::Center).css_classes(["suggested-action", "pill"]).build();
        let (ov, msg) = (toasts.clone(), format!("{}…", tr.onb_install_all));
        btn_all.connect_clicked(move |_| {
            toast(&ov, &msg);
            let pkgs: Vec<&str> = process::missing_dependencies().iter().map(|d| d.arch_package()).collect();
            process::install_packages(&pkgs);
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
    let nav_power = nav_list.clone();
    btn_power.connect_clicked(move |_| navigate(&nav_power, "statusbar"));
    row_power.add_suffix(&btn_power);
    group_power.add(&row_power);

    page_dash.add(&group_onboard);
    page_dash.add(&group_power);


    let group_dash_bar = PreferencesGroup::builder()
        .title(&tr.sidebar_statusbar)
        .build();

    let bar_model_dash = StringList::new(&["Waybar", "Quickshell", tr.common_none.as_str()]);
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

    let sw_auto_pal = SwitchRow::builder()
        .title(tr.palette_auto_sync.as_str())
        .subtitle(tr.palette_auto_sync_sub.as_str())
        .active(shell_cfg.auto_palette)
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
    group_palette.add(&sw_auto_pal);
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

    let row_glass = ActionRow::builder()
        .title(&tr.palette_glass_title)
        .subtitle(&tr.palette_glass_sub)
        .build();
    let btn_glass = Button::builder().label(&tr.palette_glass_btn).valign(gtk4::Align::Center).build();
    let (ov, msg) = (toasts.clone(), format!("{} — {}", tr.palette_glass_title, tr.toast_done));
    btn_glass.connect_clicked(move |_| {
        palette::apply_glassmorphism();
        toast(&ov, &msg);
    });
    row_glass.add_suffix(&btn_glass);
    group_palette.add(&row_glass);

    page_dash.add(&group_palette);

    stack.add_titled(&page_dash, Some("dashboard"), "Dashboard");

    // ========================================================
    // PAGINA 2: Hyprland (Geometry, Opacity, Shaders & Motion, Displays)
    // ========================================================
    let page_hypr = PreferencesPage::new();

    // Presets: één klik zet alle schuiven; daarna kan alles nog fijn worden afgesteld.
    let group_presets = PreferencesGroup::builder().title(&tr.hypr_presets).description(&tr.hypr_presets_desc).build();
    let preset_box = Box::new(Orientation::Horizontal, 8);
    preset_box.set_homogeneous(true);
    let preset_labels = [&tr.preset_default, &tr.preset_minimal, &tr.preset_performance, &tr.preset_eyecandy];
    for (key, label) in config::PRESETS.iter().zip(preset_labels) {
        let btn = Button::builder().label(label.as_str()).css_classes(["zenith-preset"]).build();
        let (st, win, app_c, lang_c) = (Rc::clone(state), window.downgrade(), app.clone(), lang.clone());
        let key = *key;
        btn.connect_clicked(move |_| {
            config::apply_preset(&mut st.borrow_mut(), key);
            hyprland::apply_all(&st.borrow());
            config::save_config(&st.borrow());
            if let Some(w) = win.upgrade() {
                rebuild(&w, &app_c, &st, &lang_c, "hyprland");
            }
        });
        preset_box.append(&btn);
    }
    group_presets.add(&preset_box);
    page_hypr.add(&group_presets);

    let px = |v: f64| format!("{} px", v.round());
    let pct = |v: f64| format!("{:.0}%", v * 100.0);

    let group_geom = PreferencesGroup::builder().title(crate::ui::escape::pango_escape(&tr.hypr_geometry)).description(&tr.hypr_geom_desc).build();
    group_geom.add(&slider_row(&tr.hypr_outer_gaps, 0.0, 40.0, 1.0, initial_cfg.gaps_out as f64, px, state, |c, v| {
        c.gaps_out = v as i32;
        hyprland::set_keyword("general:gaps_out", c.gaps_out);
    }));
    group_geom.add(&slider_row(&tr.hypr_inner_gaps, 0.0, 30.0, 1.0, initial_cfg.gaps_in as f64, px, state, |c, v| {
        c.gaps_in = v as i32;
        hyprland::set_keyword("general:gaps_in", c.gaps_in);
    }));
    group_geom.add(&slider_row(&tr.hypr_border_width, 0.0, 10.0, 1.0, initial_cfg.border_size as f64, px, state, |c, v| {
        c.border_size = v as i32;
        hyprland::set_keyword("general:border_size", c.border_size);
    }));
    group_geom.add(&slider_row(&tr.hypr_corner_rounding, 0.0, 30.0, 1.0, initial_cfg.rounding as f64, px, state, |c, v| {
        c.rounding = v as i32;
        hyprland::set_keyword("decoration:rounding", c.rounding);
    }));
    group_geom.add(&color_row(&tr.hypr_border_color, &initial_cfg.active_border_color, state, |c, hex| {
        hyprland::set_keyword("general:col.active_border", format!("rgb({})", hex));
        c.active_border_color = hex;
    }));
    group_geom.add(&color_row(&tr.hypr_inactive_border, &initial_cfg.inactive_border_color, state, |c, hex| {
        hyprland::set_keyword("general:col.inactive_border", format!("rgb({})", hex));
        c.inactive_border_color = hex;
    }));

    let layouts = ["dwindle", "master"];
    let row_layout = ComboRow::builder()
        .title(tr.hypr_layout.as_str())
        .subtitle(tr.hypr_layout_sub.as_str())
        .model(&StringList::new(&["Dwindle", "Master"]))
        .build();
    row_layout.set_selected(if initial_cfg.layout == "master" { 1 } else { 0 });
    let st_layout = Rc::clone(state);
    row_layout.connect_selected_notify(move |r| {
        let l = layouts[r.selected() as usize % layouts.len()];
        hyprland::set_keyword("general:layout", l);
        st_layout.borrow_mut().layout = l.to_string();
        config::save_config(&st_layout.borrow());
    });
    group_geom.add(&row_layout);
    page_hypr.add(&group_geom);

    let group_opacity = PreferencesGroup::builder().title(&tr.hypr_transparency).description(&tr.hypr_trans_desc).build();
    group_opacity.add(&slider_row(&tr.hypr_active_opacity, 0.2, 1.0, 0.05, initial_cfg.active_opacity, pct, state, |c, v| {
        c.active_opacity = v;
        hyprland::set_keyword("decoration:active_opacity", format!("{:.2}", v));
    }));
    group_opacity.add(&slider_row(&tr.hypr_inactive_opacity, 0.2, 1.0, 0.05, initial_cfg.inactive_opacity, pct, state, |c, v| {
        c.inactive_opacity = v;
        hyprland::set_keyword("decoration:inactive_opacity", format!("{:.2}", v));
    }));
    group_opacity.add(&switch_row(&tr.hypr_dim_inactive, &tr.hypr_dim_sub, initial_cfg.dim_inactive, state, |c, on| {
        c.dim_inactive = on;
        hyprland::set_keyword("decoration:dim_inactive", on);
    }));
    page_hypr.add(&group_opacity);

    let group_fx = PreferencesGroup::builder().title(crate::ui::escape::pango_escape(&tr.hypr_effects)).description(&tr.hypr_effects_desc).build();
    group_fx.add(&switch_row(&tr.hypr_blur, &tr.hypr_blur_sub, initial_cfg.blur_enabled, state, |c, on| {
        c.blur_enabled = on;
        hyprland::set_keyword("decoration:blur:enabled", on);
    }));
    group_fx.add(&slider_row(&tr.hypr_blur_intensity, 1.0, 20.0, 1.0, initial_cfg.blur_size as f64, px, state, |c, v| {
        c.blur_size = v as i32;
        hyprland::set_keyword("decoration:blur:size", c.blur_size);
    }));
    group_fx.add(&slider_row(&tr.hypr_blur_passes, 1.0, 6.0, 1.0, initial_cfg.blur_passes as f64, |v| format!("{}×", v.round()), state, |c, v| {
        c.blur_passes = v as i32;
        hyprland::set_keyword("decoration:blur:passes", c.blur_passes);
    }));
    group_fx.add(&switch_row(&tr.hypr_shadows, &tr.hypr_shadows_sub, initial_cfg.shadow_enabled, state, |c, on| {
        c.shadow_enabled = on;
        hyprland::set_keyword("decoration:shadow:enabled", on);
    }));
    group_fx.add(&switch_row(&tr.hypr_animations, &tr.hypr_animations_sub, initial_cfg.animations_enabled, state, |c, on| {
        c.animations_enabled = on;
        hyprland::set_keyword("animations:enabled", on);
    }));
    page_hypr.add(&group_fx);

    // Invoer: leeg laten = Zenith blijft van de toetsenbordindeling in hyprland.conf af.
    let group_input = PreferencesGroup::builder().title(&tr.hypr_input).build();
    let row_kb = libadwaita::EntryRow::builder()
        .title(tr.hypr_kb_layout.as_str())
        .text(initial_cfg.kb_layout.as_str())
        .show_apply_button(true)
        .build();
    row_kb.set_tooltip_text(Some(tr.hypr_kb_sub.as_str()));
    let st_kb = Rc::clone(state);
    let (ov_kb, msg_kb) = (toasts.clone(), format!("{} — {}", tr.hypr_kb_layout, tr.toast_done));
    row_kb.connect_apply(move |r| {
        // Alleen layout-codes (bv. "us", "be", "us,nl"): de waarde gaat naar hyprland.conf.
        let kb: String = r.text().chars().filter(|ch| ch.is_ascii_alphanumeric() || *ch == ',' || *ch == '_').collect();
        r.set_text(&kb);
        if !kb.is_empty() {
            hyprland::set_keyword("input:kb_layout", &kb);
        }
        st_kb.borrow_mut().kb_layout = kb;
        config::save_config(&st_kb.borrow());
        toast(&ov_kb, &msg_kb);
    });
    group_input.add(&row_kb);
    group_input.add(&Label::builder()
        .label(tr.hypr_kb_sub.as_str())
        .halign(gtk4::Align::Start)
        .wrap(true)
        .xalign(0.0)
        .margin_top(6)
        .css_classes(["zenith-dim", "caption"])
        .build());
    page_hypr.add(&group_input);

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
            let st_mon = Rc::clone(state);
            row_res.connect_selected_notify(move |r| {
                let idx = r.selected() as usize;
                if let Some(target_mode) = m_strings_c.get(idx) {
                    let rule = hyprland::set_monitor_mode(&m_name_c, target_mode);
                    config::upsert_monitor_rule(&mut st_mon.borrow_mut(), rule);
                    config::save_config(&st_mon.borrow());
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
            let st_scl = Rc::clone(state);
            s_scale.connect_value_changed(move |s| {
                let v = s.value();
                r_scl_c.set_subtitle(&format!("{:.2}x", v));
                let rule = hyprland::set_monitor_scale(&m_name_scl, v);
                config::upsert_monitor_rule(&mut st_scl.borrow_mut(), rule);
                config::save_config(&st_scl.borrow());
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

    let group_wb_layout = PreferencesGroup::builder().title(&tr.wb_title).description(&tr.wb_desc).build();
    let px = |v: f64| format!("{} px", v.round());
    let pct = |v: f64| format!("{:.0}%", v * 100.0);

    let row_pos = ComboRow::builder()
        .title(tr.bar_position.as_str())
        .model(&StringList::new(&["Top", "Bottom"]))
        .build();
    row_pos.set_selected(if initial_cfg.waybar_position == "bottom" { 1 } else { 0 });
    let st_pos = Rc::clone(state);
    row_pos.connect_selected_notify(move |r| {
        let pos_str = if r.selected() == 1 { "bottom" } else { "top" };
        let h = st_pos.borrow().waybar_height;
        waybar::update_config(pos_str, h);
        st_pos.borrow_mut().waybar_position = pos_str.to_string();
        config::save_config(&st_pos.borrow());
    });
    group_wb_layout.add(&row_pos);
    group_wb_layout.add(&slider_row(&tr.bar_height, 20.0, 56.0, 2.0, initial_cfg.waybar_height as f64, px, state, |c, v| {
        c.waybar_height = v as i32;
        waybar::update_config(&c.waybar_position, c.waybar_height);
    }));
    group_wb_layout.add(&color_row(&tr.theme_bg_color, &initial_cfg.waybar_bg_color, state, |c, hex| {
        waybar::update_style(&hex, c.waybar_opacity, c.waybar_rounding);
        c.waybar_bg_color = hex;
    }));
    group_wb_layout.add(&slider_row(&tr.theme_rounding, 0.0, 30.0, 1.0, initial_cfg.waybar_rounding as f64, px, state, |c, v| {
        c.waybar_rounding = v as i32;
        waybar::update_style(&c.waybar_bg_color, c.waybar_opacity, c.waybar_rounding);
    }));
    page_qs.add(&group_wb_layout);
    stack.add_titled(&page_qs, Some("statusbar"), &tr.sidebar_statusbar);

    // ========================================================
    // PAGINA 4: Control Center Designer
    // ========================================================
    let page_cc = crate::ui::control_center_designer::build_control_center_page(state, &tr);
    stack.add_titled(&page_cc, Some("control_center"), &tr.sidebar_control_center);

    // ========================================================
    // PAGINA 5: Toetsenbord & Sneltoetsen
    // ========================================================
    let page_keybinds = crate::ui::keybinds_designer::build_keybinds_page(&tr);
    stack.add_titled(&page_keybinds, Some("keybinds"), &tr.sidebar_keybinds);

    // ========================================================
    // PAGINA 6: On-Screen Display (OSD) Designer
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

    let group_kitty = PreferencesGroup::builder().title("Kitty").description(&tr.theme_kitty_desc).build();
    group_kitty.add(&color_row(&tr.theme_bg_color, &initial_cfg.kitty_bg, state, |c, hex| {
        themes::update_kitty(&hex, &c.kitty_fg, c.kitty_opacity, c.kitty_font_size);
        c.kitty_bg = hex;
    }));
    group_kitty.add(&color_row(&tr.theme_fg_color, &initial_cfg.kitty_fg, state, |c, hex| {
        themes::update_kitty(&c.kitty_bg, &hex, c.kitty_opacity, c.kitty_font_size);
        c.kitty_fg = hex;
    }));
    group_kitty.add(&slider_row(&tr.theme_window_trans, 0.4, 1.0, 0.05, initial_cfg.kitty_opacity, pct, state, |c, v| {
        c.kitty_opacity = v;
        themes::update_kitty(&c.kitty_bg, &c.kitty_fg, v, c.kitty_font_size);
    }));
    group_kitty.add(&slider_row(&tr.theme_font_size, 8.0, 20.0, 0.5, initial_cfg.kitty_font_size, |v| format!("{:.1} pt", v), state, |c, v| {
        c.kitty_font_size = v;
        themes::update_kitty(&c.kitty_bg, &c.kitty_fg, c.kitty_opacity, v);
    }));
    page_themes.add(&group_kitty);

    let group_rofi = PreferencesGroup::builder().title("Rofi").description(&tr.theme_rofi_desc).build();
    group_rofi.add(&color_row(&tr.theme_bg_color, &initial_cfg.rofi_bg, state, |c, hex| {
        themes::update_rofi(&hex, &c.rofi_fg, c.rofi_opacity, c.rofi_rounding);
        c.rofi_bg = hex;
    }));
    group_rofi.add(&color_row(&tr.theme_fg_color, &initial_cfg.rofi_fg, state, |c, hex| {
        themes::update_rofi(&c.rofi_bg, &hex, c.rofi_opacity, c.rofi_rounding);
        c.rofi_fg = hex;
    }));
    group_rofi.add(&slider_row(&tr.theme_window_trans, 0.4, 1.0, 0.05, initial_cfg.rofi_opacity, pct, state, |c, v| {
        c.rofi_opacity = v;
        themes::update_rofi(&c.rofi_bg, &c.rofi_fg, v, c.rofi_rounding);
    }));
    group_rofi.add(&slider_row(&tr.theme_rounding, 0.0, 30.0, 1.0, initial_cfg.rofi_rounding as f64, px, state, |c, v| {
        c.rofi_rounding = v as i32;
        themes::update_rofi(&c.rofi_bg, &c.rofi_fg, c.rofi_opacity, c.rofi_rounding);
    }));
    page_themes.add(&group_rofi);

    let group_tools = PreferencesGroup::builder()
        .title(&tr.theme_apps)
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
    let row_shell = ComboRow::builder().title(tr.theme_user_shell.as_str()).model(&shell_model).build();
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

    let row_icon_theme = ActionRow::builder().title(&tr.theme_icon_theme).subtitle("Papirus / Adwaita").build();
    let icon_model = StringList::new(&["Papirus", "Adwaita"]);
    let dd_icon = DropDown::builder().model(&icon_model).valign(gtk4::Align::Center).build();
    let cur_icon = gsettings_get("icon-theme");
    dd_icon.set_selected(match cur_icon.as_str() {
        "Adwaita" => 1,
        _ => 0,
    });
    dd_icon.connect_selected_notify(move |d| {
        match d.selected() {
            1 => gsettings_set("icon-theme", "Adwaita"),
            _ => gsettings_set("icon-theme", "Papirus"),
        }
    });
    row_icon_theme.add_suffix(&dd_icon);
    group_gtk.add(&row_icon_theme);

    let row_cursor = ActionRow::builder().title(&tr.theme_cursor).build();
    let ent_cursor = Entry::builder().valign(gtk4::Align::Center).text(gsettings_get("cursor-theme").as_str()).build();
    ent_cursor.set_width_chars(14);
    ent_cursor.connect_activate(move |e| {
        gsettings_set("cursor-theme", e.text().as_str());
    });
    row_cursor.add_suffix(&ent_cursor);
    group_gtk.add(&row_cursor);

    let row_font = ActionRow::builder().title(&tr.theme_font).subtitle("Inter 11 (UI) · JetBrains Mono 10 (code)").build();
    let font_model = StringList::new(&["Inter 11", "JetBrains Mono 10", "Cantarell 11 (systeem)"]);
    let dd_font = DropDown::builder().model(&font_model).valign(gtk4::Align::Center).build();
    let cur_font = gsettings_get("font-name");
    dd_font.set_selected(match cur_font.as_str() {
        "JetBrains Mono 10" => 1,
        "Cantarell 11" => 2,
        _ => 0,
    });
    dd_font.connect_selected_notify(move |d| {
        match d.selected() {
            1 => gsettings_set("font-name", "JetBrains Mono 10"),
            2 => gsettings_set("font-name", "Cantarell 11"),
            _ => gsettings_set("font-name", "Inter 11"),
        }
    });
    row_font.add_suffix(&dd_font);
    group_gtk.add(&row_font);

    page_themes.add(&group_gtk);

    // Wijzigingen toepassen
    let group_apply = PreferencesGroup::builder().title(&tr.theme_apply).build();
    let row_apply = ActionRow::builder()
        .title(&tr.common_apply)
        .subtitle(&tr.theme_apply)
        .build();
    let btn_apply = Button::builder().label(&tr.theme_apply).valign(gtk4::Align::Center).css_classes(["suggested-action", "pill"]).build();
    let st_apply = Rc::clone(state);
    let (ov, msg) = (toasts.clone(), format!("{} — {}", tr.theme_apply, tr.toast_done));
    btn_apply.connect_clicked(move |_| {
        config::save_config(&st_apply.borrow());
        themes::update_quickshell(&st_apply.borrow());
        waybar::reload_waybar();
        toast(&ov, &msg);
    });
    row_apply.add_suffix(&btn_apply);
    group_apply.add(&row_apply);
    page_themes.add(&group_apply);

    stack.add_titled(&page_themes, Some("themes"), &tr.sidebar_themes);

    // ========================================================
    // PAGINA 7: Iconen & Emoji's
    // ========================================================
    let page_icons = crate::ui::icon_studio::build_icon_studio_page(&tr);
    stack.add_titled(&page_icons, Some("icons"), &tr.icons_title);

    // ========================================================
    // PAGINA 8: Fastfetch Visual Studio
    // ========================================================
    let page_fastfetch = crate::ui::fastfetch_designer::build_fastfetch_page(&tr);
    stack.add_titled(&page_fastfetch, Some("fastfetch"), &tr.sidebar_fastfetch);

    // ========================================================
    // PAGINA 9: Systeem & Tools
    // ========================================================
    let page_system = PreferencesPage::new();

    // Tool Hub (EPIC-03): gecureerd pakketbeheer via blueprints + privileged broker.
    {
        let group_toolhub = PreferencesGroup::builder()
            .title(crate::ui::escape::pango_escape(&tr.toolhub_title))
            .description(crate::ui::escape::pango_escape(&tr.toolhub_desc))
            .build();

        let catalog = crate::backend::packages::load_catalog();
        if catalog.is_empty() {
            let row_empty = ActionRow::builder()
                .title(crate::ui::escape::pango_escape(&tr.toolhub_none))
                .build();
            group_toolhub.add(&row_empty);
        }

        let t_installed = tr.toolhub_installed.clone();
        let t_install = tr.toolhub_install.clone();
        let t_dl_base = tr.toolhub_downloading.clone();
        let t_ins_base = tr.toolhub_installing.clone();
        let t_done_base = tr.toolhub_done.clone();
        let t_fail_base = tr.toolhub_failed.clone();

        for item in &catalog {
            let title = crate::ui::escape::pango_escape(&format!("{} {}", item.icon, item.name));
            let desc = crate::ui::escape::pango_escape(&item.description);
            let row = ActionRow::builder().title(&title).subtitle(&desc).build();
            let item_copy = item.clone();

            let initial = if item_copy.is_installed() { t_installed.clone() } else { t_install.clone() };
            let status_lbl = Label::new(Some(initial.as_str()));
            status_lbl.set_css_classes(&["zenith-dim"]);
            row.add_suffix(&status_lbl);
            let stl = Rc::new(RefCell::new(status_lbl));

            if !item_copy.is_installed() {
                // Verse per-iteratie klonen zodat elke boxed `Fn`-closure zijn
                // eigen eigendoms-kopie heeft.
                let t_dl = t_dl_base.clone();
                let t_ins = t_ins_base.clone();
                let t_done = t_done_base.clone();
                let t_fail = t_fail_base.clone();
                let btn_i = Button::builder().label(&t_install).valign(gtk4::Align::Center).build();
                let stl_c = Rc::clone(&stl);
                let key = item_copy.key.clone();
                let log = crate::backend::packages::log_path(key.as_str());
                btn_i.connect_clicked(move |_| {
                    let st = stl_c.borrow();
                    st.set_label(&t_dl);
                    crate::backend::packages::spawn_install(&item_copy);
                    drop(st);
                    // Clone frisse locals voor de inner timer-closure (boxed
                    // `Fn` indienste closures geen buitenste captures mogen lenen).
                    let stl_t = Rc::clone(&stl_c);
                    let log_t = log.clone();
                    let t_ins_t = t_ins.clone();
                    let t_done_t = t_done.clone();
                    let t_fail_t = t_fail.clone();
                    let _ = gtk4::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                        let st = stl_t.borrow();
                        match crate::backend::packages::read_status(&log_t) {
                            crate::backend::packages::PackageStatus::Done => {
                                st.set_label(&t_done_t);
                                gtk4::glib::ControlFlow::Break
                            }
                            crate::backend::packages::PackageStatus::Failed => {
                                st.set_label(&t_fail_t);
                                gtk4::glib::ControlFlow::Break
                            }
                            crate::backend::packages::PackageStatus::Installing => {
                                st.set_label(&t_ins_t);
                                gtk4::glib::ControlFlow::Continue
                            }
                            _ => gtk4::glib::ControlFlow::Continue
                        }
                    });
                });
                row.add_suffix(&btn_i);
            }
            group_toolhub.add(&row);
        }

        page_system.add(&group_toolhub);
    }

    let group_sys = PreferencesGroup::builder().title(crate::ui::escape::pango_escape(&tr.sidebar_system)).build();

    let dnd_paused = std::process::Command::new("dunstctl").arg("is-paused").output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true").unwrap_or(false);
    let row_dnd = SwitchRow::builder().title(tr.sys_dnd_title.as_str()).subtitle(tr.sys_dnd_sub.as_str()).active(dnd_paused).build();
    row_dnd.connect_active_notify(|r| process::toggle_dunst_dnd(r.is_active()));
    group_sys.add(&row_dnd);

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
        let _ = std::process::Command::new("xdg-open").arg(&hypr_conf_path).spawn();
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
        let _ = std::process::Command::new("xdg-open").arg(&autostart_dir_clone).spawn();
    });
    row_autostart.add_suffix(&btn_autostart);
    group_autostart.add(&row_autostart);
    page_system.add(&group_autostart);

    // Over Zenith
    let group_about = PreferencesGroup::builder().title(&tr.sys_about).build();
    let row_about = ActionRow::builder()
        .title("Zenith")
        .subtitle(crate::ui::escape::pango_escape(&format!("v{} — Voor Hyprland & Quickshell", env!("CARGO_PKG_VERSION"))))
        .build();
    group_about.add(&row_about);
    page_system.add(&group_about);

    stack.add_titled(&page_system, Some("system"), &tr.sidebar_system);

    // ========================================================
    // PAGINA: Back-up & Herstel
    // ========================================================
    let page_backup = crate::ui::backup_designer::build_backup_page(&tr);
    stack.add_titled(&page_backup, Some("backup"), &tr.sidebar_backup);

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

    split
}

/// Korte bevestiging onderin het venster, zodat elke actie zichtbaar feedback geeft.
fn toast(overlay: &ToastOverlay, msg: &str) {
    let t = Toast::new(msg);
    t.set_timeout(2);
    overlay.add_toast(t);
}

fn rgba_to_hex(c: &RGBA) -> String {
    let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("{:02x}{:02x}{:02x}", ch(c.red()), ch(c.green()), ch(c.blue()))
}

/// Navigeer via de zijbalk zodat selectie en pagina altijd synchroon blijven.
fn navigate(nav_list: &ListBox, id: &str) {
    let mut i = 0;
    while let Some(row) = nav_list.row_at_index(i) {
        if row.widget_name() == id {
            row.set_visible(true);
            nav_list.select_row(Some(&row));
            return;
        }
        i += 1;
    }
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