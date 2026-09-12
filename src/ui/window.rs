use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, ColorDialog, ColorDialogButton, DropDown, FileDialog, Image, Label, ListBox,
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
use crate::backend::{config, hyprland, process, themes, waybar};
use crate::backend::{i18n, palette, shell_config::ZenithShellConfig};

pub fn build_window(app: &Application) {
    let initial_cfg = config::load_config();
    let state = Rc::new(RefCell::new(initial_cfg.clone()));

    let shell_cfg = ZenithShellConfig::load_or_default();
    let lang = i18n::Language::from_str(&shell_cfg.language).unwrap_or(i18n::Language::Nl);
    let tr = i18n::get_translations(&lang);

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
        .label(&format!("● {}", tr.status_active))
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

    let pill_clone = qs_pill.clone();
    let status_active = tr.status_active.clone();
    let status_inactive = tr.status_inactive.clone();
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
        let running = process::is_process_running("quickshell");
        if running {
            pill_clone.set_label(&format!("● {}", status_active));
            pill_clone.set_css_classes(&["status-pill-active"]);
        } else {
            pill_clone.set_label(&format!("○ {}", status_inactive));
            pill_clone.set_css_classes(&["status-pill-inactive"]);
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
    }

    let nav_items = [
        NavItem { id: "dashboard", title: tr.sidebar_dashboard.clone(), icon: "view-grid-symbolic" },
        NavItem { id: "hyprland", title: tr.sidebar_hyprland.clone(), icon: "applications-graphics-symbolic" },
        NavItem { id: "statusbar", title: tr.sidebar_statusbar.clone(), icon: "utilities-terminal-symbolic" },
        NavItem { id: "control_center", title: tr.sidebar_control_center.clone(), icon: "preferences-desktop-keyboard-shortcuts-symbolic" },
        NavItem { id: "osd", title: tr.sidebar_osd.clone(), icon: "video-display-symbolic" },
        NavItem { id: "themes", title: tr.sidebar_themes.clone(), icon: "applications-accessories-symbolic" },
        NavItem { id: "icons", title: tr.icons_title.clone(), icon: "emblem-favorite-symbolic" },
        NavItem { id: "system", title: tr.sidebar_system.clone(), icon: "emblem-system-symbolic" },
    ];

    for item in &nav_items {
        let row = ListBoxRow::new();
        row.set_widget_name(item.id);

        let row_box = Box::new(Orientation::Horizontal, 10);
        row_box.set_margin_top(4);
        row_box.set_margin_bottom(4);
        row_box.set_margin_start(4);
        row_box.set_margin_end(4);

        let img = Image::from_icon_name(item.icon);
        img.set_pixel_size(16);

        let lbl = Label::builder()
            .label(&item.title)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        row_box.append(&img);
        row_box.append(&lbl);
        row.set_child(Some(&row_box));
        nav_list.append(&row);
    }

    sidebar.append(&nav_list);

    // Language Selector
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
    lang_drop.connect_selected_notify(move |dd| {
        let lang_str = match dd.selected() {
            1 => "en",
            2 => "de",
            3 => "es",
            _ => "nl",
        };
        let mut scfg = ZenithShellConfig::load_or_default();
        scfg.language = lang_str.to_string();
        let _ = scfg.save();
    });
    lang_box.append(&lang_label);
    lang_box.append(&lang_drop);
    sidebar.append(&lang_box);

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
        .description("Test en bedien Quickshell en Hyprland componenten direct")
        .build();

    let row_cc_test = ActionRow::builder()
        .title("Control Center Paneel")
        .subtitle("Open of sluit het zwevende controlepaneel")
        .build();
    let btn_cc_test = Button::builder().label(&format!("🚀 {}", tr.dash_test_cc)).valign(gtk4::Align::Center).build();
    btn_cc_test.connect_clicked(|_| {
        process::execute_cmd("quickshell ipc call controlCenter toggle");
    });
    row_cc_test.add_suffix(&btn_cc_test);
    group_quick.add(&row_cc_test);

    let row_osd_test = ActionRow::builder()
        .title("OSD Notificatie Test")
        .subtitle("Activeer de geanimeerde On-Screen Display popup")
        .build();
    let btn_osd_test = Button::builder().label(&format!("🔔 {}", tr.dash_test_osd)).valign(gtk4::Align::Center).build();
    btn_osd_test.connect_clicked(|_| {
        process::execute_cmd("quickshell ipc call osd popup '⚡' 'Zenith OS' 0.85");
    });
    row_osd_test.add_suffix(&btn_osd_test);
    group_quick.add(&row_osd_test);

    let row_reload_hypr = ActionRow::builder()
        .title("Hyprland Compositor")
        .subtitle("Herlaad Hyprland window rules en configuratie")
        .build();
    let btn_reload_hypr = Button::builder().label(&format!("🔄 {}", tr.dash_reload_hypr)).valign(gtk4::Align::Center).build();
    btn_reload_hypr.connect_clicked(|_| {
        process::execute_cmd("hyprctl reload");
    });
    row_reload_hypr.add_suffix(&btn_reload_hypr);
    group_quick.add(&row_reload_hypr);

    let row_restart_bar = ActionRow::builder()
        .title("Herstart Actieve Statusbalk")
        .subtitle("Herstart Quickshell of Waybar daemon")
        .build();
    let btn_restart_bar = Button::builder().label(&format!("⚡ {}", tr.dash_restart_bar)).valign(gtk4::Align::Center).build();
    let st_rbar = Rc::clone(&state);
    btn_restart_bar.connect_clicked(move |_| {
        let active = st_rbar.borrow().active_bar.clone();
        process::set_active_bar(&active);
    });
    row_restart_bar.add_suffix(&btn_restart_bar);
    group_quick.add(&row_restart_bar);

    page_dash.add(&group_quick);

    let group_dash_bar = PreferencesGroup::builder()
        .title("Statusbalk & Achtergrond")
        .description("Kies je actieve statusbar en bureaublad wallpaper")
        .build();

    let bar_model_dash = StringList::new(&["Waybar", "Quickshell", "Geen"]);
    let row_dash_bar = ComboRow::builder().title(tr.dash_active_bar.as_str()).model(&bar_model_dash).build();
    row_dash_bar.set_selected(match initial_cfg.active_bar.as_str() { "quickshell" => 1, "none" => 2, _ => 0 });
    let st_dbar = Rc::clone(&state);
    row_dash_bar.connect_selected_notify(move |r| {
        let choice = match r.selected() { 1 => "quickshell", 2 => "none", _ => "waybar" };
        process::set_active_bar(choice);
        st_dbar.borrow_mut().active_bar = choice.to_string();
        config::save_config(&st_dbar.borrow());
    });
    group_dash_bar.add(&row_dash_bar);

    let row_dash_wall = ActionRow::builder().title(tr.dash_wallpaper.as_str()).subtitle("Selecteer een achtergrondafbeelding").build();
    let btn_dash_wall = Button::builder().label("Kies bestand...").valign(gtk4::Align::Center).build();
    btn_dash_wall.connect_clicked(move |_| {
        let fd = FileDialog::builder().title("Kies achtergrond").build();
        fd.open(None::<&gtk4::Window>, None::<&gtk4::gio::Cancellable>, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    if let Some(p_str) = path.to_str() {
                        hyprland::set_wallpaper(p_str);
                        // Save wallpaper path and auto-extract palette if enabled
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
        });
    });
    row_dash_wall.add_suffix(&btn_dash_wall);
    group_dash_bar.add(&row_dash_wall);

    page_dash.add(&group_dash_bar);

    // Wallpaper Palette Engine
    let group_palette = PreferencesGroup::builder()
        .title(&tr.palette_title)
        .description("Extraheer automatisch een kleurenschema uit je wallpaper")
        .build();

    let row_auto_pal = ActionRow::builder()
        .title(&tr.palette_auto_sync)
        .subtitle("Kleuren worden automatisch aangepast bij wallpaper-wijziging")
        .build();
    let sw_auto_pal = Switch::builder()
        .active(shell_cfg.auto_palette)
        .valign(gtk4::Align::Center)
        .build();
    sw_auto_pal.connect_active_notify(move |sw| {
        let mut scfg = ZenithShellConfig::load_or_default();
        scfg.auto_palette = sw.is_active();
        let _ = scfg.save();
    });
    row_auto_pal.add_suffix(&sw_auto_pal);
    group_palette.add(&row_auto_pal);

    let row_gen_pal = ActionRow::builder()
        .title(&tr.palette_generate)
        .subtitle("Analyseer de huidige wallpaper en genereer een kleurenpalet")
        .build();
    let btn_gen_pal = Button::builder().label("🎨 Genereer").valign(gtk4::Align::Center).build();
    btn_gen_pal.connect_clicked(|_| {
        let scfg = ZenithShellConfig::load_or_default();
        if !scfg.wallpaper_path.is_empty() {
            if let Some(pal) = palette::extract_palette(&scfg.wallpaper_path) {
                palette::apply_palette(&pal);
            }
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

    let group_geom = PreferencesGroup::builder().title(&tr.hypr_geometry).description("Marges, dikte en hoekafronding van vensters").build();

    let row_out = ActionRow::builder().title(&tr.hypr_outer_gaps).subtitle(&format!("{} px", initial_cfg.gaps_out)).build();
    let s_out = Scale::with_range(Orientation::Horizontal, 0.0, 40.0, 1.0);
    s_out.set_value(initial_cfg.gaps_out as f64);
    s_out.set_width_request(160);
    let r_out_c = row_out.clone();
    let st_out = Rc::clone(&state);
    s_out.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_out_c.set_subtitle(&format!("{} px", v));
        hyprland::set_gaps_out(v);
        st_out.borrow_mut().gaps_out = v;
        config::save_config(&st_out.borrow());
    });
    row_out.add_suffix(&s_out);
    group_geom.add(&row_out);

    let row_in = ActionRow::builder().title(&tr.hypr_inner_gaps).subtitle(&format!("{} px", initial_cfg.gaps_in)).build();
    let s_in = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_in.set_value(initial_cfg.gaps_in as f64);
    s_in.set_width_request(160);
    let r_in_c = row_in.clone();
    let st_in = Rc::clone(&state);
    s_in.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_in_c.set_subtitle(&format!("{} px", v));
        hyprland::set_gaps_in(v);
        st_in.borrow_mut().gaps_in = v;
        config::save_config(&st_in.borrow());
    });
    row_in.add_suffix(&s_in);
    group_geom.add(&row_in);

    let row_border = ActionRow::builder().title(&tr.hypr_border_width).subtitle(&format!("{} px", initial_cfg.border_size)).build();
    let s_border = Scale::with_range(Orientation::Horizontal, 0.0, 10.0, 1.0);
    s_border.set_value(initial_cfg.border_size as f64);
    s_border.set_width_request(160);
    let r_b_c = row_border.clone();
    let st_border = Rc::clone(&state);
    s_border.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_b_c.set_subtitle(&format!("{} px", v));
        hyprland::set_border_size(v);
        st_border.borrow_mut().border_size = v;
        config::save_config(&st_border.borrow());
    });
    row_border.add_suffix(&s_border);
    group_geom.add(&row_border);

    let row_round = ActionRow::builder().title(&tr.hypr_corner_rounding).subtitle(&format!("{} px", initial_cfg.rounding)).build();
    let s_round = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_round.set_value(initial_cfg.rounding as f64);
    s_round.set_width_request(160);
    let r_rnd_c = row_round.clone();
    let st_rnd = Rc::clone(&state);
    s_round.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_rnd_c.set_subtitle(&format!("{} px", v));
        hyprland::set_rounding(v);
        st_rnd.borrow_mut().rounding = v;
        config::save_config(&st_rnd.borrow());
    });
    row_round.add_suffix(&s_round);
    group_geom.add(&row_round);

    let row_color = ActionRow::builder().title(&tr.hypr_border_color).subtitle("Kleur van actief venster").build();
    let c_dialog = ColorDialog::builder().title("Border Color").with_alpha(false).build();
    let c_btn = ColorDialogButton::builder().dialog(&c_dialog).valign(gtk4::Align::Center).build();
    c_btn.set_rgba(&hex_to_rgba(&initial_cfg.active_border_color));
    let st_col = Rc::clone(&state);
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

    let group_opacity = PreferencesGroup::builder().title(&tr.hypr_transparency).build();

    let row_act_op = ActionRow::builder().title(&tr.hypr_active_opacity).subtitle(&format!("{:.0}%", initial_cfg.active_opacity * 100.0)).build();
    let s_act_op = Scale::with_range(Orientation::Horizontal, 0.2, 1.0, 0.05);
    s_act_op.set_value(initial_cfg.active_opacity);
    s_act_op.set_width_request(160);
    let r_aop_c = row_act_op.clone();
    let st_aop = Rc::clone(&state);
    s_act_op.connect_value_changed(move |s| {
        let v = s.value();
        r_aop_c.set_subtitle(&format!("{:.0}%", v * 100.0));
        hyprland::set_active_opacity(v);
        st_aop.borrow_mut().active_opacity = v;
        config::save_config(&st_aop.borrow());
    });
    row_act_op.add_suffix(&s_act_op);
    group_opacity.add(&row_act_op);

    let row_inact_op = ActionRow::builder().title(&tr.hypr_inactive_opacity).subtitle(&format!("{:.0}%", initial_cfg.inactive_opacity * 100.0)).build();
    let s_inact_op = Scale::with_range(Orientation::Horizontal, 0.2, 1.0, 0.05);
    s_inact_op.set_value(initial_cfg.inactive_opacity);
    s_inact_op.set_width_request(160);
    let r_iop_c = row_inact_op.clone();
    let st_iop = Rc::clone(&state);
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

    let group_fx = PreferencesGroup::builder().title(&tr.hypr_effects).build();

    let row_blur = ActionRow::builder().title(&tr.hypr_blur).subtitle("Achtergrond van vensters vervagen").build();
    let sw_blur = Switch::builder().active(initial_cfg.blur_enabled).valign(gtk4::Align::Center).build();
    let st_blur = Rc::clone(&state);
    sw_blur.connect_state_set(move |_, active| {
        hyprland::set_blur_enabled(active);
        st_blur.borrow_mut().blur_enabled = active;
        config::save_config(&st_blur.borrow());
        gtk4::glib::Propagation::Proceed
    });
    row_blur.add_suffix(&sw_blur);
    group_fx.add(&row_blur);

    let row_bsize = ActionRow::builder().title(&tr.hypr_blur_intensity).subtitle(&format!("{} px", initial_cfg.blur_size)).build();
    let s_bsize = Scale::with_range(Orientation::Horizontal, 1.0, 20.0, 1.0);
    s_bsize.set_value(initial_cfg.blur_size as f64);
    s_bsize.set_width_request(160);
    let r_bs_c = row_bsize.clone();
    let st_bs = Rc::clone(&state);
    s_bsize.connect_value_changed(move |s| {
        let v = s.value().round() as i32;
        r_bs_c.set_subtitle(&format!("{} px", v));
        hyprland::set_blur_size(v);
        st_bs.borrow_mut().blur_size = v;
        config::save_config(&st_bs.borrow());
    });
    row_bsize.add_suffix(&s_bsize);
    group_fx.add(&row_bsize);

    let row_shd = ActionRow::builder().title(&tr.hypr_shadows).subtitle("Diepte-schaduw achter vensters").build();
    let sw_shd = Switch::builder().active(initial_cfg.shadow_enabled).valign(gtk4::Align::Center).build();
    let st_shd = Rc::clone(&state);
    sw_shd.connect_state_set(move |_, active| {
        hyprland::set_shadow_enabled(active);
        st_shd.borrow_mut().shadow_enabled = active;
        config::save_config(&st_shd.borrow());
        gtk4::glib::Propagation::Proceed
    });
    row_shd.add_suffix(&sw_shd);
    group_fx.add(&row_shd);

    let row_anim = ActionRow::builder().title(&tr.hypr_animations).subtitle("Venster overgangen en animaties").build();
    let sw_anim = Switch::builder().active(initial_cfg.animations_enabled).valign(gtk4::Align::Center).build();
    let st_anim = Rc::clone(&state);
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
                .title(&format!("Scherm: {}", mon_name))
                .description(&format!("Actief: {}", current_mode))
                .build();

            let mut mode_strings: Vec<String> = mon.available_modes.clone();
            if mode_strings.is_empty() {
                mode_strings.push(format!("{}x{}@{:.2}Hz", mon.width, mon.height, mon.refresh_rate));
            }
            mode_strings.dedup();

            let mode_strs_slices: Vec<&str> = mode_strings.iter().map(|s| s.as_str()).collect();
            let mode_model = StringList::new(&mode_strs_slices);

            let row_mode = ComboRow::builder()
                .title("Resolutie & Refresh Rate")
                .subtitle("Kies de schermmodus")
                .model(&mode_model)
                .build();

            let active_idx = mode_strings.iter().position(|m| {
                m.starts_with(&format!("{}x{}", mon.width, mon.height))
            }).unwrap_or(0);
            row_mode.set_selected(active_idx as u32);

            let mon_name_scale = mon_name.clone();
            let scale_state = Rc::clone(&state);

            let row_scale = ActionRow::builder()
                .title("Beeldscherm Schaal")
                .subtitle(&format!("{:.2}x", mon.scale))
                .build();
            let s_scale = Scale::with_range(Orientation::Horizontal, 1.0, 2.5, 0.25);
            s_scale.set_value(mon.scale);
            s_scale.set_width_request(160);

            let r_sc_c = row_scale.clone();
            let scale_val_ref = Rc::new(RefCell::new(mon.scale));

            let sc_v_clone = Rc::clone(&scale_val_ref);
            let mon_name_mode = mon_name.clone();
            let modes_captured = mode_strings.clone();
            let st_mode = Rc::clone(&state);

            row_mode.connect_selected_notify(move |r| {
                let idx = r.selected() as usize;
                if let Some(target_mode) = modes_captured.get(idx) {
                    let current_scale = *sc_v_clone.borrow();
                    hyprland::apply_monitor_rule(&mon_name_mode, target_mode, current_scale);

                    let formatted_rule = format!("{},{},auto,{:.2}", mon_name_mode, target_mode, current_scale);
                    let mut cfg = st_mode.borrow_mut();
                    cfg.monitor_rules.retain(|x| !x.starts_with(&mon_name_mode));
                    cfg.monitor_rules.push(formatted_rule);
                    config::save_config(&cfg);
                }
            });

            let modes_scale_captured = mode_strings.clone();
            let row_mode_ref = row_mode.clone();
            s_scale.connect_value_changed(move |s| {
                let v = s.value();
                r_sc_c.set_subtitle(&format!("{:.2}x", v));
                *scale_val_ref.borrow_mut() = v;

                let sel_idx = row_mode_ref.selected() as usize;
                let target_mode = modes_scale_captured.get(sel_idx).cloned().unwrap_or_else(|| "preferred".to_string());
                
                hyprland::apply_monitor_rule(&mon_name_scale, &target_mode, v);

                let formatted_rule = format!("{},{},auto,{:.2}", mon_name_scale, target_mode, v);
                let mut cfg = scale_state.borrow_mut();
                cfg.monitor_rules.retain(|x| !x.starts_with(&mon_name_scale));
                cfg.monitor_rules.push(formatted_rule);
                config::save_config(&cfg);
            });

            row_scale.add_suffix(&s_scale);
            group_mon.add(&row_mode);
            group_mon.add(&row_scale);
            page_hypr.add(&group_mon);
        }
    }

    stack.add_titled(&page_hypr, Some("hyprland"), "Hyprland");

    // ========================================================
    // PAGINA 3: Statusbalk (Quickshell Designer + Waybar Fallback)
    // ========================================================
    let page_qs = crate::ui::quickshell_designer::build_quickshell_page(&state);

    let group_wb_layout = PreferencesGroup::builder()
        .title("Waybar Styling (Alternatief)")
        .description("Configuratie voor wanneer Waybar geselecteerd is als statusbalk")
        .build();

    let row_pos = ActionRow::builder().title("Positie op Scherm").build();
    let pos_model = StringList::new(&["top", "bottom"]);
    let dd_pos = DropDown::builder().model(&pos_model).valign(gtk4::Align::Center).build();
    dd_pos.set_selected(if initial_cfg.waybar_position == "bottom" { 1 } else { 0 });
    let st_pos = Rc::clone(&state);
    dd_pos.connect_selected_notify(move |d| {
        let pos_str = if d.selected() == 1 { "bottom" } else { "top" };
        let h = st_pos.borrow().waybar_height;
        waybar::update_config(pos_str, h);
        st_pos.borrow_mut().waybar_position = pos_str.to_string();
        config::save_config(&st_pos.borrow());
    });
    row_pos.add_suffix(&dd_pos);
    group_wb_layout.add(&row_pos);

    let row_h = ActionRow::builder().title("Balk Hoogte").subtitle(&format!("{} px", initial_cfg.waybar_height)).build();
    let s_h = Scale::with_range(Orientation::Horizontal, 20.0, 56.0, 2.0);
    s_h.set_value(initial_cfg.waybar_height as f64);
    s_h.set_width_request(160);
    let r_h_c = row_h.clone();
    let st_h = Rc::clone(&state);
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
    let st_wb_bg = Rc::clone(&state);
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

    let row_wb_rnd = ActionRow::builder().title("Balk Afronding").subtitle(&format!("{} px", initial_cfg.waybar_rounding)).build();
    let s_wb_rnd = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_wb_rnd.set_value(initial_cfg.waybar_rounding as f64);
    s_wb_rnd.set_width_request(160);
    let r_wrnd_c = row_wb_rnd.clone();
    let st_wb_rnd = Rc::clone(&state);
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
    stack.add_titled(&page_qs, Some("statusbar"), "Statusbalk");

    // ========================================================
    // PAGINA 4: Control Center Designer
    // ========================================================
    let page_cc = crate::ui::control_center_designer::build_control_center_page(&state);
    stack.add_titled(&page_cc, Some("control_center"), "Control Center");

    // ========================================================
    // PAGINA 5: On-Screen Display (OSD) Designer
    // ========================================================
    let page_osd = crate::ui::osd_designer::build_osd_page(&state);
    stack.add_titled(&page_osd, Some("osd"), "OSD & Meldingen");

    // ========================================================
    // PAGINA 6: Thema's & Apps (Kitty, Rofi, Standaard Applicaties)
    // ========================================================
    let page_themes = PreferencesPage::new();

    let group_kitty = PreferencesGroup::builder().title("Kitty Terminal").description("Kleuren, transparantie en fontgrootte").build();
    
    let row_k_bg = ActionRow::builder().title("Achtergrondkleur").build();
    let dlg_k_bg = ColorDialog::builder().title("Kitty Achtergrond").with_alpha(false).build();
    let btn_k_bg = ColorDialogButton::builder().dialog(&dlg_k_bg).valign(gtk4::Align::Center).build();
    btn_k_bg.set_rgba(&hex_to_rgba(&initial_cfg.kitty_bg));
    let st_k_bg = Rc::clone(&state);
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

    let row_k_op = ActionRow::builder().title("Venster Transparantie").subtitle(&format!("{:.0}%", initial_cfg.kitty_opacity * 100.0)).build();
    let s_k_op = Scale::with_range(Orientation::Horizontal, 0.4, 1.0, 0.05);
    s_k_op.set_value(initial_cfg.kitty_opacity);
    s_k_op.set_width_request(160);
    let r_kop_c = row_k_op.clone();
    let st_k_op = Rc::clone(&state);
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

    let row_k_font = ActionRow::builder().title("Lettergrootte").subtitle(&format!("{:.1} pt", initial_cfg.kitty_font_size)).build();
    let s_k_font = Scale::with_range(Orientation::Horizontal, 8.0, 20.0, 0.5);
    s_k_font.set_value(initial_cfg.kitty_font_size);
    s_k_font.set_width_request(160);
    let r_kfont_c = row_k_font.clone();
    let st_k_font = Rc::clone(&state);
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

    let group_rofi = PreferencesGroup::builder().title("Rofi Menu").description("Vensterkleur en afronding").build();
    
    let row_r_bg = ActionRow::builder().title("Achtergrondkleur").build();
    let dlg_r_bg = ColorDialog::builder().title("Rofi Achtergrond").with_alpha(false).build();
    let btn_r_bg = ColorDialogButton::builder().dialog(&dlg_r_bg).valign(gtk4::Align::Center).build();
    btn_r_bg.set_rgba(&hex_to_rgba(&initial_cfg.rofi_bg));
    let st_r_bg = Rc::clone(&state);
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

    let row_r_rnd = ActionRow::builder().title("Venster Afronding").subtitle(&format!("{} px", initial_cfg.rofi_rounding)).build();
    let s_r_rnd = Scale::with_range(Orientation::Horizontal, 0.0, 30.0, 1.0);
    s_r_rnd.set_value(initial_cfg.rofi_rounding as f64);
    s_r_rnd.set_width_request(160);
    let r_rrnd_c = row_r_rnd.clone();
    let st_r_rnd = Rc::clone(&state);
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
        .description("Kies je launcher, terminal en shell")
        .build();

    let launcher_model = StringList::new(&["Rofi", "Wofi"]);
    let row_launcher = ComboRow::builder().title("Applicatiemenu ($menu)").model(&launcher_model).build();
    row_launcher.set_selected(if initial_cfg.default_launcher == "wofi" { 1 } else { 0 });
    let st_lnc = Rc::clone(&state);
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
    let st_trm = Rc::clone(&state);
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
    let st_shl = Rc::clone(&state);
    row_shell.connect_selected_notify(move |r| {
        let choice = match r.selected() { 1 => "bash", 2 => "fish", _ => "zsh" };
        process::set_user_shell(choice);
        st_shl.borrow_mut().default_shell = choice.to_string();
        config::save_config(&st_shl.borrow());
    });
    group_tools.add(&row_shell);

    page_themes.add(&group_tools);
    stack.add_titled(&page_themes, Some("themes"), "Thema's & Apps");

    // ========================================================
    // PAGINA 7: Systeem & Tools
    // ========================================================
    let page_system = PreferencesPage::new();
    let group_sys = PreferencesGroup::builder().title("Systeem Hulpmiddelen").build();

    let row_dnd = ActionRow::builder().title("Do Not Disturb").subtitle("Meldingen pauzeren (Dunst)").build();
    let sw_dnd = Switch::builder().valign(gtk4::Align::Center).build();
    sw_dnd.connect_state_set(|_, active| {
        process::toggle_dunst_dnd(active);
        gtk4::glib::Propagation::Proceed
    });
    row_dnd.add_suffix(&sw_dnd);
    group_sys.add(&row_dnd);

    let row_wall = ActionRow::builder().title("Wallpaper").subtitle("Selecteer een achtergrondafbeelding").build();
    let btn_wall = Button::builder().label("Kies bestand...").valign(gtk4::Align::Center).build();
    btn_wall.connect_clicked(move |_| {
        let fd = FileDialog::builder().title("Kies achtergrond").build();
        fd.open(None::<&gtk4::Window>, None::<&gtk4::gio::Cancellable>, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    if let Some(p_str) = path.to_str() {
                        hyprland::set_wallpaper(p_str);
                    }
                }
            }
        });
    });
    row_wall.add_suffix(&btn_wall);
    group_sys.add(&row_wall);

    page_system.add(&group_sys);
    stack.add_titled(&page_system, Some("system"), "Systeem & Tools");

    let page_icons = crate::ui::icon_studio::build_icon_studio_page(&tr);
    stack.add_titled(&page_icons, Some("icons"), &tr.icons_title);

    // Selecteer initiële Dashboard pagina in de zijbalk
    if let Some(first_row) = nav_list.row_at_index(0) {
        nav_list.select_row(Some(&first_row));
    }

    // ========================================================
    // Hoofdvenster Builder
    // ========================================================
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Zenith")
        .default_width(940)
        .default_height(700)
        .content(&root_box)
        .build();

    window.present();
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