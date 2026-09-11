use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, ColorDialog, ColorDialogButton, DropDown, FileDialog, Orientation, Scale, Switch, StringList,
};
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, ComboRow, HeaderBar, PreferencesGroup, PreferencesPage,
    ViewStack, ViewSwitcher,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::backend::{config, hyprland, process, themes, waybar};

pub fn build_window(app: &Application) {
    let initial_cfg = config::load_config();
    let state = Rc::new(RefCell::new(initial_cfg.clone()));

    let main_box = Box::new(Orientation::Vertical, 0);
    let stack = ViewStack::new();

    let header = HeaderBar::new();
    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(libadwaita::ViewSwitcherPolicy::Wide)
        .build();
    header.set_title_widget(Some(&switcher));

    main_box.append(&header);
    main_box.append(&stack);

    // ==========================================
    // TAB 1: Layout
    // ==========================================
    let page_layout = PreferencesPage::new();
    let group_geom = PreferencesGroup::builder().title("Geometry").description("Marges en vensterranden").build();

    let row_out = ActionRow::builder().title("Outer Gaps").subtitle(&format!("{} px", initial_cfg.gaps_out)).build();
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

    let row_in = ActionRow::builder().title("Inner Gaps").subtitle(&format!("{} px", initial_cfg.gaps_in)).build();
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

    let row_border = ActionRow::builder().title("Border Width").subtitle(&format!("{} px", initial_cfg.border_size)).build();
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

    let row_round = ActionRow::builder().title("Corner Rounding").subtitle(&format!("{} px", initial_cfg.rounding)).build();
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

    let row_color = ActionRow::builder().title("Active Border Color").subtitle("Kleur van actief venster").build();
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

    page_layout.add(&group_geom);
    let tab1 = stack.add_titled(&page_layout, Some("layout"), "Layout");
    tab1.set_icon_name(Some("view-grid-symbolic"));

    // ==========================================
    // TAB 2: Effects
    // ==========================================
    let page_effects = PreferencesPage::new();
    let group_opacity = PreferencesGroup::builder().title("Transparantie").build();

    let row_act_op = ActionRow::builder().title("Active Opacity").subtitle(&format!("{:.0}%", initial_cfg.active_opacity * 100.0)).build();
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

    let row_inact_op = ActionRow::builder().title("Inactive Opacity").subtitle(&format!("{:.0}%", initial_cfg.inactive_opacity * 100.0)).build();
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
    page_effects.add(&group_opacity);

    let group_fx = PreferencesGroup::builder().title("Shaders & Motion").build();

    let row_blur = ActionRow::builder().title("Window Blur").subtitle("Achtergrond van vensters vervagen").build();
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

    let row_bsize = ActionRow::builder().title("Blur Intensity").subtitle(&format!("{} px", initial_cfg.blur_size)).build();
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

    let row_shd = ActionRow::builder().title("Window Shadows").subtitle("Diepte-schaduw achter vensters").build();
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

    let row_anim = ActionRow::builder().title("Animations").subtitle("Venster overgangen en animaties").build();
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

    page_effects.add(&group_fx);
    let tab2 = stack.add_titled(&page_effects, Some("effects"), "Effects");
    tab2.set_icon_name(Some("applications-graphics-symbolic"));

    // ==========================================
    // TAB 3: Displays
    // ==========================================
    let page_displays = PreferencesPage::new();
    let detected_monitors = hyprland::get_monitors();

    if detected_monitors.is_empty() {
        let empty_group = PreferencesGroup::builder()
            .title("Geen monitoren gevonden")
            .description("Kon geen actieve Hyprland beeldschermen detecteren.")
            .build();
        page_displays.add(&empty_group);
    } else {
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
            page_displays.add(&group_mon);
        }
    }

    let tab_disp = stack.add_titled(&page_displays, Some("displays"), "Displays");
    tab_disp.set_icon_name(Some("video-display-symbolic"));

    // ==========================================
    // TAB 4: Themes (Kitty, Rofi)
    // ==========================================
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

    let tab_thm = stack.add_titled(&page_themes, Some("themes"), "Themes");
    tab_thm.set_icon_name(Some("applications-accessories-symbolic"));

    // ==========================================
    // TAB 5: Defaults
    // ==========================================
    let page_defaults = PreferencesPage::new();
    let group_tools = PreferencesGroup::builder()
        .title("Standaard Applicaties & Shell")
        .description("Kies je statusbar, launcher, terminal en shell")
        .build();

    let bar_model = StringList::new(&["Waybar", "Quickshell", "Geen"]);
    let row_bar = ComboRow::builder().title("Statusbalk").model(&bar_model).build();
    row_bar.set_selected(match initial_cfg.active_bar.as_str() { "quickshell" => 1, "none" => 2, _ => 0 });
    let st_bar = Rc::clone(&state);
    row_bar.connect_selected_notify(move |r| {
        let choice = match r.selected() { 1 => "quickshell", 2 => "none", _ => "waybar" };
        process::set_active_bar(choice);
        st_bar.borrow_mut().active_bar = choice.to_string();
        config::save_config(&st_bar.borrow());
    });
    group_tools.add(&row_bar);

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

    page_defaults.add(&group_tools);
    let tab_def = stack.add_titled(&page_defaults, Some("defaults"), "Defaults");
    tab_def.set_icon_name(Some("preferences-system-symbolic"));

    // ==========================================
    // TAB 6: Quickshell (Statusbar Designer)
    // ==========================================
    let page_qs = crate::ui::quickshell_designer::build_quickshell_page(&state);
    let tab_qs = stack.add_titled(&page_qs, Some("quickshell"), "Quickshell");
    tab_qs.set_icon_name(Some("utilities-terminal-symbolic"));

    // ==========================================
    // TAB 7: Waybar
    // ==========================================
    let page_waybar = PreferencesPage::new();

    let group_wb_layout = PreferencesGroup::builder().title("Waybar Layout & Styling").build();

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

    page_waybar.add(&group_wb_layout);
    let tab_wb = stack.add_titled(&page_waybar, Some("waybar"), "Waybar");
    tab_wb.set_icon_name(Some("open-menu-symbolic"));

    // ==========================================
    // TAB 8: System
    // ==========================================
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
    let tab_sys = stack.add_titled(&page_system, Some("system"), "System");
    tab_sys.set_icon_name(Some("emblem-system-symbolic"));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Zenith")
        .default_width(620)
        .default_height(720)
        .content(&main_box)
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