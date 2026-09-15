use gtk4::cairo;
use gtk4::prelude::*;
use gtk4::{
    Align, Button, DrawingArea, DropDown, Entry, FileDialog, Label, ListBox, Orientation,
    Scale, StringList, Switch,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::config::ZenithConfig;
use crate::backend::i18n::Translations;
use crate::backend::process;
use crate::backend::shell_config::{CustomScriptModule, ZenithShellConfig};
use crate::backend::themes;

pub fn build_lockscreen_page(_state: &Rc<RefCell<ZenithConfig>>, tr: &Translations) -> PreferencesPage {
    let page = PreferencesPage::new();
    let shell_state = Rc::new(RefCell::new(ZenithShellConfig::load_or_default()));

    // ==========================================
    // 1. Realtime Live Canvas Preview (16:9)
    // ==========================================
    let group_preview = PreferencesGroup::builder()
        .title(&tr.ls_preview)
        .description("Realtime visuele weergave van het lockscreen volgens je actieve instellingen")
        .build();

    let drawing_area = DrawingArea::builder()
        .content_width(560)
        .content_height(315)
        .hexpand(false)
        .halign(Align::Center)
        .valign(Align::Center)
        .margin_top(8)
        .margin_bottom(12)
        .build();

    {
        let sh_st = Rc::clone(&shell_state);
        drawing_area.set_draw_func(move |_, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let cfg = sh_st.borrow();
            let ls = &cfg.lockscreen;
            let styling = &cfg.styling;

            // Monitor Bezel Frame (Afgeronde rechthoek)
            let radius = 14.0;
            cr.new_sub_path();
            cr.arc(w - radius, radius, radius, -std::f64::consts::FRAC_PI_2, 0.0);
            cr.arc(w - radius, h - radius, radius, 0.0, std::f64::consts::FRAC_PI_2);
            cr.arc(radius, h - radius, radius, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
            cr.arc(radius, radius, radius, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
            cr.close_path();
            cr.clip();

            // Achtergrond
            if ls.background_mode == "solid_black" {
                cr.set_source_rgb(0.0, 0.0, 0.0);
                let _ = cr.paint();
            } else {
                // Rijke donkere wallpaper gradient
                let pattern = cairo::LinearGradient::new(0.0, 0.0, w, h);
                pattern.add_color_stop_rgb(0.0, 0.11, 0.12, 0.18);
                pattern.add_color_stop_rgb(0.5, 0.18, 0.20, 0.28);
                pattern.add_color_stop_rgb(1.0, 0.08, 0.09, 0.13);
                let _ = cr.set_source(&pattern);
                let _ = cr.paint();
            }

            // Dimming Overlay
            let dim = if ls.background_mode == "solid_black" { 1.0 } else { ls.dim_opacity };
            cr.set_source_rgba(0.0, 0.0, 0.0, dim);
            let _ = cr.paint();

            // Accent kleuren
            let acc_hex = styling.accent.trim_start_matches('#');
            let ar = u8::from_str_radix(acc_hex.get(0..2).unwrap_or("89"), 16).unwrap_or(137) as f64 / 255.0;
            let ag = u8::from_str_radix(acc_hex.get(2..4).unwrap_or("b4"), 16).unwrap_or(180) as f64 / 255.0;
            let ab = u8::from_str_radix(acc_hex.get(4..6).unwrap_or("fa"), 16).unwrap_or(250) as f64 / 255.0;

            let text_hex = styling.text_color.trim_start_matches('#');
            let tr_r = u8::from_str_radix(text_hex.get(0..2).unwrap_or("cd"), 16).unwrap_or(205) as f64 / 255.0;
            let tg = u8::from_str_radix(text_hex.get(2..4).unwrap_or("d6"), 16).unwrap_or(214) as f64 / 255.0;
            let tb = u8::from_str_radix(text_hex.get(4..6).unwrap_or("f4"), 16).unwrap_or(244) as f64 / 255.0;

            // Schaal factoren voor monitor preview
            let s = h / 720.0;
            let font_px = (ls.clock_font_size as f64 * s * 0.48).max(22.0);

            // Tijdstring
            let time_str = if ls.clock_format.contains("ss") { "14:35:09" } else { "14:35" };

            let preset = ls.layout_preset.as_str();

            if preset == "split" {
                // SPLIT LAYOUT: Klok links, Auth rechts
                // Linker kolom
                cr.set_source_rgb(tr_r, tg, tb);
                cr.set_font_size(font_px);
                cr.move_to(w * 0.12, h * 0.44);
                let _ = cr.show_text(time_str);

                cr.set_source_rgb(ar, ag, ab);
                cr.set_font_size(font_px * 0.28);
                cr.move_to(w * 0.12, h * 0.54);
                let _ = cr.show_text("Dinsdag, 15 September");

                // Verticale scheidingslijn
                cr.set_source_rgba(0.27, 0.28, 0.35, 0.5);
                cr.set_line_width(1.0);
                cr.move_to(w * 0.50, h * 0.25);
                cr.line_to(w * 0.50, h * 0.75);
                let _ = cr.stroke();

                // Rechter kolom: Avatar
                let av_x = w * 0.75;
                let av_y = h * 0.38;
                cr.set_source_rgb(ar, ag, ab);
                cr.arc(av_x, av_y, 16.0, 0.0, 2.0 * std::f64::consts::PI);
                let _ = cr.stroke();

                // Begroeting
                let greet = ls.custom_greeting.replace("{user}", "dicey");
                cr.set_source_rgb(tr_r, tg, tb);
                cr.set_font_size(11.0);
                let ext_w = cr.text_extents(&greet).map(|e| e.width()).unwrap_or(60.0);
                cr.move_to(av_x - ext_w / 2.0, h * 0.48);
                let _ = cr.show_text(&greet);

                // Auth Box
                let bw = 120.0;
                let bh = 22.0;
                let bx = av_x - bw / 2.0;
                let by = h * 0.54;
                cr.set_source_rgba(0.09, 0.09, 0.15, 0.85);
                cr.rectangle(bx, by, bw, bh);
                let _ = cr.fill();
                cr.set_source_rgb(ar, ag, ab);
                cr.rectangle(bx, by, bw, bh);
                let _ = cr.stroke();

                cr.set_source_rgb(ar, ag, ab);
                cr.set_font_size(10.0);
                cr.move_to(bx + 8.0, by + 15.0);
                let _ = cr.show_text(&ls.auth_indicator_icon);

                cr.set_source_rgba(tr_r, tg, tb, 0.6);
                cr.move_to(bx + 26.0, by + 14.0);
                let _ = cr.show_text("••••••••");
            } else if preset == "left_aligned" {
                // LEFT ALIGNED LAYOUT
                let start_x = w * 0.12;

                cr.set_source_rgb(tr_r, tg, tb);
                cr.set_font_size(font_px);
                cr.move_to(start_x, h * 0.38);
                let _ = cr.show_text(time_str);

                cr.set_source_rgb(ar, ag, ab);
                cr.set_font_size(font_px * 0.28);
                cr.move_to(start_x, h * 0.48);
                let _ = cr.show_text("Dinsdag, 15 September");

                // Begroeting
                let greet = ls.custom_greeting.replace("{user}", "dicey");
                cr.set_source_rgb(tr_r, tg, tb);
                cr.set_font_size(12.0);
                cr.move_to(start_x, h * 0.58);
                let _ = cr.show_text(&greet);

                // Auth capsule
                let bw = 140.0;
                let bh = 24.0;
                let by = h * 0.64;
                cr.set_source_rgba(0.09, 0.09, 0.15, 0.85);
                cr.rectangle(start_x, by, bw, bh);
                let _ = cr.fill();
                cr.set_source_rgb(ar, ag, ab);
                cr.rectangle(start_x, by, bw, bh);
                let _ = cr.stroke();

                cr.set_source_rgb(ar, ag, ab);
                cr.set_font_size(11.0);
                cr.move_to(start_x + 8.0, by + 16.0);
                let _ = cr.show_text(&ls.auth_indicator_icon);

                cr.set_source_rgba(tr_r, tg, tb, 0.6);
                cr.move_to(start_x + 30.0, by + 16.0);
                let _ = cr.show_text("••••••••");
            } else {
                // CENTERED / FREE CANVAS
                cr.set_source_rgb(tr_r, tg, tb);
                cr.set_font_size(font_px);
                let ext_w = cr.text_extents(time_str).map(|e| e.width()).unwrap_or(80.0);
                cr.move_to(w / 2.0 - ext_w / 2.0, h * 0.36);
                let _ = cr.show_text(time_str);

                cr.set_source_rgb(ar, ag, ab);
                cr.set_font_size(font_px * 0.26);
                let date_str = "Dinsdag, 15 September";
                let dext_w = cr.text_extents(date_str).map(|e| e.width()).unwrap_or(110.0);
                cr.move_to(w / 2.0 - dext_w / 2.0, h * 0.46);
                let _ = cr.show_text(date_str);

                // Ronde Avatar Cirkel
                cr.set_source_rgb(ar, ag, ab);
                cr.arc(w / 2.0, h * 0.56, 16.0, 0.0, 2.0 * std::f64::consts::PI);
                let _ = cr.stroke();

                // Begroeting
                let greet = ls.custom_greeting.replace("{user}", "dicey");
                cr.set_source_rgb(tr_r, tg, tb);
                cr.set_font_size(11.0);
                let gext_w = cr.text_extents(&greet).map(|e| e.width()).unwrap_or(60.0);
                cr.move_to(w / 2.0 - gext_w / 2.0, h * 0.66);
                let _ = cr.show_text(&greet);

                // Auth Capsule Box
                let bw = 130.0;
                let bh = 22.0;
                let bx = w / 2.0 - bw / 2.0;
                let by = h * 0.72;
                cr.set_source_rgba(0.09, 0.09, 0.15, 0.85);
                cr.rectangle(bx, by, bw, bh);
                let _ = cr.fill();
                cr.set_source_rgb(ar, ag, ab);
                cr.rectangle(bx, by, bw, bh);
                let _ = cr.stroke();

                cr.set_source_rgb(ar, ag, ab);
                cr.set_font_size(11.0);
                cr.move_to(bx + 8.0, by + 15.0);
                let _ = cr.show_text(&ls.auth_indicator_icon);

                cr.set_source_rgba(tr_r, tg, tb, 0.6);
                cr.move_to(bx + 26.0, by + 15.0);
                let _ = cr.show_text("••••••••");

                // Media & Status Pills onderin
                cr.set_source_rgba(0.18, 0.18, 0.25, 0.6);
                cr.rectangle(w / 2.0 - 55.0, h * 0.84, 110.0, 18.0);
                let _ = cr.fill();
                cr.set_source_rgb(tr_r, tg, tb);
                cr.set_font_size(9.0);
                cr.move_to(w / 2.0 - 45.0, h * 0.84 + 13.0);
                let _ = cr.show_text("📶 Wi-Fi   🔋 100%");
            }

            // Monitor Buitenkant Border
            cr.reset_clip();
            cr.set_source_rgb(0.19, 0.20, 0.27);
            cr.set_line_width(2.0);
            cr.arc(w - radius, radius, radius, -std::f64::consts::FRAC_PI_2, 0.0);
            cr.arc(w - radius, h - radius, radius, 0.0, std::f64::consts::FRAC_PI_2);
            cr.arc(radius, h - radius, radius, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
            cr.arc(radius, radius, radius, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
            cr.close_path();
            let _ = cr.stroke();
        });
    }

    let da_rc = Rc::new(drawing_area);

    // Knoppen direct onder preview
    let row_preview_actions = ActionRow::builder()
        .title("Test & Directe Bediening")
        .subtitle("Open het lockscreen in een veilig venster of activeer de echte schermvergrendeling")
        .build();

    let btn_test_win = Button::builder()
        .label(&format!("👁️ {}", tr.ls_test_preview))
        .valign(Align::Center)
        .build();
    btn_test_win.connect_clicked(|_| {
        process::execute_cmd("quickshell ipc call lockscreen preview");
    });
    row_preview_actions.add_suffix(&btn_test_win);

    let btn_lock_now = Button::builder()
        .label(&format!("🔒 {}", tr.ls_lock_now))
        .valign(Align::Center)
        .build();
    btn_lock_now.add_css_class("destructive-action");
    btn_lock_now.connect_clicked(|_| {
        process::execute_cmd("quickshell ipc call lockscreen lock");
    });
    row_preview_actions.add_suffix(&btn_lock_now);

    group_preview.add(&*da_rc);
    group_preview.add(&row_preview_actions);
    page.add(&group_preview);

    // ==========================================
    // 2. Lay-out & Achtergrond
    // ==========================================
    let group_layout = PreferencesGroup::builder()
        .title(&tr.ls_layout_preset)
        .description("Configureer de positie van modules en de achtergrondvervaging")
        .build();

    // Inschakelen toggle
    let row_enabled = ActionRow::builder()
        .title("Lockscreen Inschakelen")
        .subtitle("Activeert Quickshell Wayland SessionLock")
        .build();
    let sw_enabled = Switch::builder().active(shell_state.borrow().lockscreen.enabled).valign(Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_enabled.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.enabled = s.is_active();
            themes::sync_shell_config(&sh);
        });
    }
    row_enabled.add_suffix(&sw_enabled);
    group_layout.add(&row_enabled);

    // Layout Preset DropDown
    let row_preset = ActionRow::builder()
        .title(&tr.ls_layout_preset)
        .subtitle("Kies hoe de klok, profiel en authenticatiebox op het scherm worden geplaatst")
        .build();
    let presets = [
        ("centered", "Centraal (centered)"),
        ("split", "Gesplitst (split)"),
        ("left_aligned", "Links uitgelijnd (left_aligned)"),
        ("free_canvas", "Vrij Canvas (free_canvas)"),
    ];
    let preset_labels: Vec<&str> = presets.iter().map(|(_, l)| *l).collect();
    let preset_model = StringList::new(&preset_labels);
    let dd_preset = DropDown::builder().model(&preset_model).valign(Align::Center).build();

    let cur_pre = shell_state.borrow().lockscreen.layout_preset.clone();
    if let Some(pos) = presets.iter().position(|(id, _)| *id == cur_pre) {
        dd_preset.set_selected(pos as u32);
    }
    {
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        dd_preset.connect_selected_notify(move |d| {
            let idx = d.selected() as usize;
            if let Some((id, _)) = presets.get(idx) {
                let mut sh = sh_st.borrow_mut();
                sh.lockscreen.layout_preset = id.to_string();
                themes::sync_shell_config(&sh);
                drop(sh);
                da.queue_draw();
            }
        });
    }
    row_preset.add_suffix(&dd_preset);
    group_layout.add(&row_preset);

    // Achtergrondmodus DropDown
    let row_bg_mode = ActionRow::builder()
        .title(&tr.ls_bg_mode)
        .subtitle("Kies de achtergrondweergave van het vergrendelscherm")
        .build();
    let bg_modes = [
        ("wallpaper_blur", "Wallpaper met Blur (wallpaper_blur)"),
        ("custom_image", "Eigen Afbeelding (custom_image)"),
        ("solid_black", "Solide Zwart (solid_black)"),
        ("transparent_acrylic", "Transparant Acryl (transparent_acrylic)"),
    ];
    let bg_labels: Vec<&str> = bg_modes.iter().map(|(_, l)| *l).collect();
    let bg_model = StringList::new(&bg_labels);
    let dd_bg_mode = DropDown::builder().model(&bg_model).valign(Align::Center).build();

    let cur_bg = shell_state.borrow().lockscreen.background_mode.clone();
    if let Some(pos) = bg_modes.iter().position(|(id, _)| *id == cur_bg) {
        dd_bg_mode.set_selected(pos as u32);
    }
    {
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        dd_bg_mode.connect_selected_notify(move |d| {
            let idx = d.selected() as usize;
            if let Some((id, _)) = bg_modes.get(idx) {
                let mut sh = sh_st.borrow_mut();
                sh.lockscreen.background_mode = id.to_string();
                themes::sync_shell_config(&sh);
                drop(sh);
                da.queue_draw();
            }
        });
    }
    row_bg_mode.add_suffix(&dd_bg_mode);
    group_layout.add(&row_bg_mode);

    // Blur Radius Slider
    let cur_blur = shell_state.borrow().lockscreen.blur_radius;
    let row_blur = ActionRow::builder()
        .title(&tr.ls_blur_radius)
        .subtitle(&format!("{} px", cur_blur))
        .build();
    let s_blur = Scale::with_range(Orientation::Horizontal, 0.0, 80.0, 2.0);
    s_blur.set_value(cur_blur as f64);
    s_blur.set_width_request(150);
    {
        let r_c = row_blur.clone();
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        s_blur.connect_value_changed(move |s| {
            let v = s.value().round() as u32;
            r_c.set_subtitle(&format!("{} px", v));
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.blur_radius = v;
            themes::sync_shell_config(&sh);
            drop(sh);
            da.queue_draw();
        });
    }
    row_blur.add_suffix(&s_blur);
    group_layout.add(&row_blur);

    // Dim Opacity Slider
    let cur_dim = shell_state.borrow().lockscreen.dim_opacity;
    let row_dim = ActionRow::builder()
        .title(&tr.ls_dim_opacity)
        .subtitle(&format!("{:.0}%", cur_dim * 100.0))
        .build();
    let s_dim = Scale::with_range(Orientation::Horizontal, 0.0, 0.90, 0.05);
    s_dim.set_value(cur_dim);
    s_dim.set_width_request(150);
    {
        let r_c = row_dim.clone();
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        s_dim.connect_value_changed(move |s| {
            let v = s.value();
            r_c.set_subtitle(&format!("{:.0}%", v * 100.0));
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.dim_opacity = v;
            themes::sync_shell_config(&sh);
            drop(sh);
            da.queue_draw();
        });
    }
    row_dim.add_suffix(&s_dim);
    group_layout.add(&row_dim);

    page.add(&group_layout);

    // ==========================================
    // 3. Klok, Tekst & Begroeting
    // ==========================================
    let group_clock = PreferencesGroup::builder()
        .title(&tr.ls_clock_format)
        .description("Pas de typografie, klokgrootte en persoonlijke begroeting aan")
        .build();

    // Klokformaat
    let cur_cformat = shell_state.borrow().lockscreen.clock_format.clone();
    let entry_cformat = Entry::builder()
        .text(&cur_cformat)
        .placeholder_text("bijv. 'hh:mm' of 'hh:mm:ss'")
        .width_chars(12)
        .valign(Align::Center)
        .build();
    let row_cformat = ActionRow::builder().title(&tr.ls_clock_format).build();
    row_cformat.add_suffix(&entry_cformat);
    group_clock.add(&row_cformat);
    {
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        entry_cformat.connect_changed(move |e| {
            let txt = e.text().trim().to_string();
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.clock_format = if txt.is_empty() { "hh:mm".to_string() } else { txt };
            themes::sync_shell_config(&sh);
            drop(sh);
            da.queue_draw();
        });
    }

    // Klokgrootte Slider
    let cur_csize = shell_state.borrow().lockscreen.clock_font_size;
    let row_csize = ActionRow::builder()
        .title(&tr.ls_clock_size)
        .subtitle(&format!("{} pt", cur_csize))
        .build();
    let s_csize = Scale::with_range(Orientation::Horizontal, 32.0, 140.0, 2.0);
    s_csize.set_value(cur_csize as f64);
    s_csize.set_width_request(150);
    {
        let r_c = row_csize.clone();
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        s_csize.connect_value_changed(move |s| {
            let v = s.value().round() as u32;
            r_c.set_subtitle(&format!("{} pt", v));
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.clock_font_size = v;
            themes::sync_shell_config(&sh);
            drop(sh);
            da.queue_draw();
        });
    }
    row_csize.add_suffix(&s_csize);
    group_clock.add(&row_csize);

    // Welkomstboodschap
    let cur_greet = shell_state.borrow().lockscreen.custom_greeting.clone();
    let entry_greet = Entry::builder()
        .text(&cur_greet)
        .placeholder_text("bijv. 'Welkom terug, {user}'")
        .width_chars(24)
        .valign(Align::Center)
        .build();
    let row_greet = ActionRow::builder()
        .title(&tr.ls_greeting)
        .subtitle("Gebruik {user} als variabele voor je computernaam")
        .build();
    row_greet.add_suffix(&entry_greet);
    group_clock.add(&row_greet);
    {
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        entry_greet.connect_changed(move |e| {
            let txt = e.text().trim().to_string();
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.custom_greeting = txt;
            themes::sync_shell_config(&sh);
            drop(sh);
            da.queue_draw();
        });
    }

    // Avatar Bestand
    let cur_avatar = shell_state.borrow().lockscreen.user_avatar_path.clone().unwrap_or_default();
    let entry_avatar = Entry::builder()
        .text(&cur_avatar)
        .placeholder_text("/pad/naar/profielfoto.png")
        .width_chars(24)
        .valign(Align::Center)
        .build();
    let btn_avatar_browse = Button::builder().label("📁 Blader").valign(Align::Center).build();
    let row_avatar = ActionRow::builder()
        .title("Avatar Afbeelding")
        .subtitle("Kies een profielfoto voor op het scherm")
        .build();
    row_avatar.add_suffix(&entry_avatar);
    row_avatar.add_suffix(&btn_avatar_browse);
    group_clock.add(&row_avatar);
    {
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        entry_avatar.connect_changed(move |e| {
            let txt = e.text().trim().to_string();
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.user_avatar_path = if txt.is_empty() { None } else { Some(txt) };
            themes::sync_shell_config(&sh);
            drop(sh);
            da.queue_draw();
        });
    }
    {
        let e_ref = entry_avatar.clone();
        btn_avatar_browse.connect_clicked(move |_| {
            let fd = FileDialog::builder().title("Kies Profielfoto").build();
            let e = e_ref.clone();
            fd.open(None::<&gtk4::Window>, None::<&gtk4::gio::Cancellable>, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        if let Some(p) = path.to_str() {
                            e.set_text(p);
                        }
                    }
                }
            });
        });
    }

    page.add(&group_clock);

    // ==========================================
    // 4. Authenticatie & Icoon
    // ==========================================
    let group_auth = PreferencesGroup::builder()
        .title(&tr.ls_auth_icon)
        .description("Pas het authenticatieveld en de foutanimaties aan")
        .build();

    let cur_icon = shell_state.borrow().lockscreen.auth_indicator_icon.clone();
    let entry_icon = Entry::builder()
        .text(&cur_icon)
        .placeholder_text("bijv. 🔒 of 🔑 of ⚡")
        .width_chars(6)
        .valign(Align::Center)
        .build();
    let row_icon = ActionRow::builder()
        .title(&tr.ls_auth_icon)
        .subtitle("Emoji of Nerd Font glyph voor het invoerveld")
        .build();
    row_icon.add_suffix(&entry_icon);
    group_auth.add(&row_icon);
    {
        let sh_st = Rc::clone(&shell_state);
        let da = Rc::clone(&da_rc);
        entry_icon.connect_changed(move |e| {
            let txt = e.text().trim().to_string();
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.auth_indicator_icon = if txt.is_empty() { "🔒".to_string() } else { txt };
            themes::sync_shell_config(&sh);
            drop(sh);
            da.queue_draw();
        });
    }

    let row_shake = ActionRow::builder()
        .title(&tr.ls_shake)
        .subtitle("Schud het invoerveld zachtjes heen-en-weer bij een fout wachtwoord")
        .build();
    let sw_shake = Switch::builder().active(shell_state.borrow().lockscreen.auth_shake_animation).valign(Align::Center).build();
    {
        let sh_st = Rc::clone(&shell_state);
        sw_shake.connect_active_notify(move |s| {
            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.auth_shake_animation = s.is_active();
            themes::sync_shell_config(&sh);
        });
    }
    row_shake.add_suffix(&sw_shake);
    group_auth.add(&row_shake);

    page.add(&group_auth);

    // ==========================================
    // 5. Actieve Kaarten Manager
    // ==========================================
    let group_cards = PreferencesGroup::builder()
        .title(&tr.ls_cards)
        .description("Rangschik, verwijder of voeg interactieve widgets toe aan het lockscreen")
        .build();

    let list_box_cards = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let list_cards_rc = Rc::new(list_box_cards);

    let dd_add_card = DropDown::builder().valign(Align::Center).build();
    let dd_add_rc = Rc::new(dd_add_card);

    let refresh_picker = {
        let sh_st = Rc::clone(&shell_state);
        let dd = Rc::clone(&dd_add_rc);
        Rc::new(move || {
            let all = sh_st.borrow().discover_lockscreen_cards_for_instance();
            let labels: Vec<String> = all.iter().map(|c| format!("{} {}", c.icon, c.name)).collect();
            let labels_ref: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
            let model = StringList::new(&labels_ref);
            dd.set_model(Some(&model));
        })
    };

    let refresh_cards_list = {
        let sh_st = Rc::clone(&shell_state);
        let lb = Rc::clone(&list_cards_rc);
        let pick = Rc::clone(&refresh_picker);
        let da = Rc::clone(&da_rc);
        Rc::new(move || {
            rebuild_cards_ui(&lb, &sh_st, &da);
            pick();
        })
    };

    fn rebuild_cards_ui(lb: &ListBox, sh_st: &Rc<RefCell<ZenithShellConfig>>, da: &Rc<DrawingArea>) {
        while let Some(child) = lb.first_child() {
            lb.remove(&child);
        }

        let all = sh_st.borrow().discover_lockscreen_cards_for_instance();
        let active = sh_st.borrow().lockscreen.cards.clone();
        let total = active.len();

        for (idx, card_id) in active.iter().enumerate() {
            let info = all.iter().find(|c| c.id == *card_id || card_id.starts_with(&format!("{}:", c.id)));
            let title = match info {
                Some(i) => format!("{} {}", i.icon, i.name),
                None => format!("🎴 {}", card_id),
            };
            let desc = match info {
                Some(i) => i.description.clone(),
                None => format!("Kaart ID: {}", card_id),
            };

            let row = ActionRow::builder().title(&title).subtitle(&desc).build();

            if idx > 0 {
                let btn_up = Button::builder().label("▲").valign(Align::Center).tooltip_text("Omhoog").build();
                let st = Rc::clone(sh_st);
                let l = lb.clone();
                let d = Rc::clone(da);
                btn_up.connect_clicked(move |_| {
                    let mut s = st.borrow_mut();
                    if idx > 0 && idx < s.lockscreen.cards.len() {
                        s.lockscreen.cards.swap(idx, idx - 1);
                    }
                    themes::sync_shell_config(&s);
                    drop(s);
                    rebuild_cards_ui(&l, &st, &d);
                    d.queue_draw();
                });
                row.add_suffix(&btn_up);
            }

            if idx + 1 < total {
                let btn_down = Button::builder().label("▼").valign(Align::Center).tooltip_text("Omlaag").build();
                let st = Rc::clone(sh_st);
                let l = lb.clone();
                let d = Rc::clone(da);
                btn_down.connect_clicked(move |_| {
                    let mut s = st.borrow_mut();
                    if idx + 1 < s.lockscreen.cards.len() {
                        s.lockscreen.cards.swap(idx, idx + 1);
                    }
                    themes::sync_shell_config(&s);
                    drop(s);
                    rebuild_cards_ui(&l, &st, &d);
                    d.queue_draw();
                });
                row.add_suffix(&btn_down);
            }

            let btn_del = Button::builder().label("✖").valign(Align::Center).tooltip_text("Verwijderen").build();
            btn_del.add_css_class("destructive-action");
            {
                let st = Rc::clone(sh_st);
                let l = lb.clone();
                let d = Rc::clone(da);
                btn_del.connect_clicked(move |_| {
                    let mut s = st.borrow_mut();
                    if idx < s.lockscreen.cards.len() {
                        s.lockscreen.cards.remove(idx);
                    }
                    themes::sync_shell_config(&s);
                    drop(s);
                    rebuild_cards_ui(&l, &st, &d);
                    d.queue_draw();
                });
            }
            row.add_suffix(&btn_del);

            lb.append(&row);
        }
    }

    let row_add_card = ActionRow::builder()
        .title("+ Kaart Toevoegen")
        .subtitle("Selecteer een widget en plaats deze op het vergrendelscherm")
        .build();
    let btn_add_c = Button::builder().label(&tr.ls_add_card).valign(Align::Center).build();
    btn_add_c.add_css_class("suggested-action");
    {
        let sh_st = Rc::clone(&shell_state);
        let dd = Rc::clone(&dd_add_rc);
        let refresh_list = Rc::clone(&refresh_cards_list);
        let da = Rc::clone(&da_rc);
        btn_add_c.connect_clicked(move |_| {
            let all = sh_st.borrow().discover_lockscreen_cards_for_instance();
            let idx = dd.selected() as usize;
            if let Some(card) = all.get(idx) {
                let mut sh = sh_st.borrow_mut();
                sh.lockscreen.cards.push(card.id.clone());
                themes::sync_shell_config(&sh);
                drop(sh);
                refresh_list();
                da.queue_draw();
            }
        });
    }
    row_add_card.add_suffix(&*dd_add_rc);
    row_add_card.add_suffix(&btn_add_c);

    group_cards.add(&*list_cards_rc);
    group_cards.add(&row_add_card);
    page.add(&group_cards);

    // Initial render
    refresh_cards_list();

    // ==========================================
    // 6. Custom Script Builder voor Lockscreen
    // ==========================================
    let group_custom_script = PreferencesGroup::builder()
        .title("Eigen Shell Commando op Lockscreen")
        .description("Toon realtime informatie (zoals security alerts, IP of status) via een eigen commando")
        .build();

    let entry_sname = Entry::builder().placeholder_text("Label (bijv. Beveiligingsstatus)").build();
    let row_sname = ActionRow::builder().title("Script Naam").build();
    row_sname.add_suffix(&entry_sname);
    group_custom_script.add(&row_sname);

    let entry_sico = Entry::builder().placeholder_text("Emoji (bijv. 🛡️)").width_chars(6).build();
    let row_sico = ActionRow::builder().title("Icoon").build();
    row_sico.add_suffix(&entry_sico);
    group_custom_script.add(&row_sico);

    let entry_scmd = Entry::builder().placeholder_text("Commando (bijv. whoami)").width_chars(28).build();
    let row_scmd = ActionRow::builder().title("Bash Commando").subtitle("Wordt periodiek uitgevoerd").build();
    row_scmd.add_suffix(&entry_scmd);
    group_custom_script.add(&row_scmd);

    let btn_create_script = Button::builder().label("+ Script Kaart Toevoegen").valign(Align::Center).build();
    btn_create_script.add_css_class("suggested-action");
    {
        let sh_st = Rc::clone(&shell_state);
        let refresh_list = Rc::clone(&refresh_cards_list);
        let e_n = entry_sname.clone();
        let e_i = entry_sico.clone();
        let e_c = entry_scmd.clone();
        let da = Rc::clone(&da_rc);
        btn_create_script.connect_clicked(move |_| {
            let name = e_n.text().trim().to_string();
            let cmd = e_c.text().trim().to_string();
            if name.is_empty() || cmd.is_empty() { return; }

            let icon = if e_i.text().trim().is_empty() { "💻".to_string() } else { e_i.text().trim().to_string() };
            let safe_id: String = name.to_lowercase().chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
            let id = format!("{}_{}", safe_id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));

            let script = CustomScriptModule {
                id: id.clone(),
                name,
                icon,
                command: cmd,
                interval_seconds: 15,
                on_click: None,
            };

            let mut sh = sh_st.borrow_mut();
            sh.lockscreen.custom_scripts.push(script);
            sh.lockscreen.cards.push(format!("script:{}", id));
            themes::sync_shell_config(&sh);
            drop(sh);

            e_n.set_text("");
            e_i.set_text("");
            e_c.set_text("");
            refresh_list();
            da.queue_draw();
        });
    }
    let row_add_btn = ActionRow::builder().build();
    row_add_btn.add_suffix(&btn_create_script);
    group_custom_script.add(&row_add_btn);
    page.add(&group_custom_script);

    // ==========================================
    // 7. Opslaan & Bevestiging
    // ==========================================
    let group_save = PreferencesGroup::builder()
        .title("Configuratie Opslaan")
        .description("Slaat direct op naar ~/.config/quickshell/zenith-shell.json")
        .build();

    let btn_save = Button::builder().label(&format!("💾 {}", tr.ls_save)).valign(Align::Center).build();
    btn_save.add_css_class("suggested-action");

    let status_lbl = Label::builder().label("").halign(Align::Start).margin_start(12).build();

    {
        let sh_st = Rc::clone(&shell_state);
        let lbl = status_lbl.clone();
        let toast = tr.ls_saved_toast.clone();
        btn_save.connect_clicked(move |_| {
            let sh = sh_st.borrow();
            match sh.save() {
                Ok(_) => {
                    themes::sync_shell_config(&sh);
                    lbl.set_markup(&format!("<span foreground=\"#a6e3a1\">✔ {}</span>", toast));
                }
                Err(e) => {
                    lbl.set_markup(&format!("<span foreground=\"#f38ba8\">✖ Fout bij opslaan: {}</span>", e));
                }
            }
        });
    }

    let row_save = ActionRow::builder().title(&tr.ls_save).build();
    row_save.add_suffix(&btn_save);
    group_save.add(&row_save);
    group_save.add(&status_lbl);
    page.add(&group_save);

    page
}
