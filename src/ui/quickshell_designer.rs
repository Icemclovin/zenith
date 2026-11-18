use gtk4::prelude::*;
use gtk4::{Box, Button, DropDown, FlowBox, Label, ListBox, Orientation, StringList};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, ComboRow, EntryRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::config::{self, ZenithConfig};
use crate::backend::i18n::Translations;
use crate::backend::shell_config::{self, BarStyle, CustomScriptModule, ZenithShellConfig};
use crate::backend::{process, themes};
use crate::ui::rows;

type Shell = Rc<RefCell<ZenithShellConfig>>;
type Notify = Rc<dyn Fn(&str)>;
/// Past een wijziging toe op de shell-config, spiegelt naar zenith.conf en slaat alles op.
type Commit = Rc<dyn Fn(&dyn Fn(&mut ZenithShellConfig))>;

/// Neemt een complete stijl over en spiegelt hem naar de legacy ZenithConfig (zenith.conf).
fn apply_style(sh: &mut ZenithShellConfig, cfg: &mut ZenithConfig, style: &BarStyle) {
    sh.styling = style.styling.clone();
    sh.layout = style.layout.clone();
    sh.modules = style.modules.clone();
    mirror_to_legacy(sh, cfg);
}

fn mirror_to_legacy(sh: &ZenithShellConfig, cfg: &mut ZenithConfig) {
    let hex = |s: &str| s.trim_start_matches('#').to_string();
    cfg.qs_bar_style = sh.layout.bar_style.clone();
    cfg.qs_position = sh.layout.position.clone();
    cfg.qs_height = sh.layout.height;
    cfg.qs_floating = sh.layout.floating;
    cfg.qs_bg = hex(&sh.styling.background);
    cfg.qs_accent = hex(&sh.styling.accent);
    cfg.qs_pill_bg = hex(&sh.styling.pill_bg);
    cfg.qs_text_color = hex(&sh.styling.text_color);
    cfg.qs_border_color = hex(&sh.styling.border_color);
    cfg.qs_rounding = sh.styling.rounding;
    cfg.qs_pill_rounding = sh.styling.pill_rounding;
    cfg.qs_margin_h = sh.styling.margin_h;
    cfg.qs_margin_v = sh.styling.margin_v;
    cfg.qs_border_width = sh.styling.border_width;
    cfg.qs_opacity = sh.styling.opacity;
    cfg.qs_pill_opacity = sh.styling.pill_opacity;
    cfg.qs_font_size = sh.styling.font_size;
    cfg.qs_module_spacing = sh.styling.module_spacing;
}

/// `rebuild` herbouwt de pagina (na een stijlwissel), `notify` toont een toast.
pub fn build_quickshell_page(
    state: &Rc<RefCell<ZenithConfig>>,
    tr: &Translations,
    rebuild: Rc<dyn Fn()>,
    notify: Notify,
) -> PreferencesPage {
    let page = PreferencesPage::new();
    let tr = Rc::new(tr.clone());
    let shell_state: Shell = Rc::new(RefCell::new(ZenithShellConfig::load_or_default()));

    // Eén plek die een wijziging toepast op beide configs en alles wegschrijft.
    let commit: Commit = {
        let (sh, st) = (Rc::clone(&shell_state), Rc::clone(state));
        Rc::new(move |change| {
            change(&mut sh.borrow_mut());
            mirror_to_legacy(&sh.borrow(), &mut st.borrow_mut());
            themes::sync_shell_config(&sh.borrow());
            config::save_config(&st.borrow());
        })
    };

    // ========================================================
    // 1. Stijlen: ingebouwd + eigen (één klik)
    // ========================================================
    let group_presets = PreferencesGroup::builder().title(&tr.bar_preset_theme).description(&tr.sb_presets_desc).build();
    let builtin = shell_config::builtin_bar_styles();
    let style_grid = FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .homogeneous(true)
        .min_children_per_line(2)
        .max_children_per_line(3)
        .column_spacing(8)
        .row_spacing(8)
        .build();
    for style in builtin {
        style_grid.insert(&style_card(&style, state, &shell_state, &rebuild, &notify, &tr), -1);
    }
    group_presets.add(&style_grid);
    page.add(&group_presets);

    let group_mine = PreferencesGroup::builder().title(&tr.sb_my_styles).description(&tr.sb_my_styles_desc).build();
    let user_styles = shell_config::load_user_styles();
    if user_styles.is_empty() {
        group_mine.add(&ActionRow::builder().title(tr.sb_no_styles.as_str()).css_classes(["zenith-dim"]).build());
    }
    for (idx, style) in user_styles.iter().enumerate() {
        let row = ActionRow::builder()
            .title(crate::ui::escape::pango_escape(&style.name))
            .subtitle(format!("{} · {} · {}", style.layout.bar_style, style.layout.position, style.styling.accent))
            .activatable(true)
            .build();
        row.add_prefix(&swatch(&style.styling.accent));
        let (st, sh, rb, nt, msg, s) = (Rc::clone(state), Rc::clone(&shell_state), Rc::clone(&rebuild), Rc::clone(&notify), tr.sb_style_applied.clone(), style.clone());
        row.connect_activated(move |_| {
            apply_style(&mut sh.borrow_mut(), &mut st.borrow_mut(), &s);
            themes::sync_shell_config(&sh.borrow());
            config::save_config(&st.borrow());
            nt(&msg);
            rb();
        });
        let btn_del = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text(tr.sb_delete_style.as_str())
            .valign(gtk4::Align::Center)
            .css_classes(["flat"])
            .build();
        let rb = Rc::clone(&rebuild);
        btn_del.connect_clicked(move |_| {
            let mut styles = shell_config::load_user_styles();
            if idx < styles.len() {
                styles.remove(idx);
                let _ = shell_config::save_user_styles(&styles);
            }
            rb();
        });
        row.add_suffix(&btn_del);
        group_mine.add(&row);
    }
    let row_save = EntryRow::builder().title(tr.sb_style_name.as_str()).show_apply_button(true).build();
    row_save.set_tooltip_text(Some(tr.sb_save_style.as_str()));
    {
        let (sh, rb, nt, msg) = (Rc::clone(&shell_state), Rc::clone(&rebuild), Rc::clone(&notify), tr.sb_style_saved.clone());
        row_save.connect_apply(move |r| {
            let name = r.text().trim().to_string();
            if name.is_empty() {
                return;
            }
            let cur = sh.borrow();
            let mut styles = shell_config::load_user_styles();
            styles.retain(|s| s.name != name); // zelfde naam = overschrijven
            styles.push(BarStyle { name, styling: cur.styling.clone(), layout: cur.layout.clone(), modules: cur.modules.clone() });
            drop(cur);
            match shell_config::save_user_styles(&styles) {
                Ok(()) => nt(&msg),
                Err(e) => nt(&e.to_string()),
            }
            rb();
        });
    }
    group_mine.add(&row_save);
    page.add(&group_mine);

    // ========================================================
    // 2. Modules: canvas + lijsten per gedeelte
    // ========================================================
    let canvas_group = PreferencesGroup::new(); // de canvas tekent zijn eigen titel
    canvas_group.add(&crate::ui::drag_canvas::build_drag_canvas(&tr));
    page.add(&canvas_group);

    let list_left = Rc::new(boxed_list());
    let list_center = Rc::new(boxed_list());
    let list_right = Rc::new(boxed_list());

    let refresh_all_modules: Rc<dyn Fn()> = {
        let (sh, l, c, r, tr) = (Rc::clone(&shell_state), Rc::clone(&list_left), Rc::clone(&list_center), Rc::clone(&list_right), Rc::clone(&tr));
        Rc::new(move || {
            refresh_slot(&l, "left", &sh, &tr);
            refresh_slot(&c, "center", &sh, &tr);
            refresh_slot(&r, "right", &sh, &tr);
        })
    };

    let picker_dropdowns: Rc<RefCell<Vec<DropDown>>> = Rc::new(RefCell::new(Vec::new()));
    let refresh_picker_models: Rc<dyn Fn()> = {
        let (pickers, sh) = (Rc::clone(&picker_dropdowns), Rc::clone(&shell_state));
        Rc::new(move || {
            let labels = module_labels(&sh);
            let labels_ref: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
            for dd in pickers.borrow().iter() {
                dd.set_model(Some(&StringList::new(&labels_ref)));
            }
        })
    };

    let group_modules = PreferencesGroup::builder().title(&tr.sb_layout_title).description(&tr.sb_layout_desc).build();
    page.add(&group_modules);
    for (slot, title, list) in [
        ("left", &tr.bar_left_modules, &list_left),
        ("center", &tr.bar_center_modules, &list_center),
        ("right", &tr.bar_right_modules, &list_right),
    ] {
        let group = PreferencesGroup::builder().title(title).build();
        group.add(&**list);
        add_module_picker_row(&group, slot, &shell_state, list, &picker_dropdowns, &tr);
        page.add(&group);
    }
    refresh_all_modules();

    // ========================================================
    // 3. Scriptmodules (shell-commando als widget)
    // ========================================================
    let group_scripts = PreferencesGroup::builder().title(&tr.bar_custom_script).description(&tr.sb_scripts_desc).build();
    let list_scripts = Rc::new(boxed_list());
    group_scripts.add(&*list_scripts);
    let refresh_scripts: Rc<dyn Fn()> = {
        let (lb, sh, ra, rp, tr) = (Rc::clone(&list_scripts), Rc::clone(&shell_state), Rc::clone(&refresh_all_modules), Rc::clone(&refresh_picker_models), Rc::clone(&tr));
        Rc::new(move || refresh_scripts_list(&lb, &sh, &ra, &rp, &tr))
    };
    refresh_scripts();

    let form = boxed_list();
    let ent_name = EntryRow::builder().title(tr.sb_script_name.as_str()).build();
    let ent_icon = EntryRow::builder().title(tr.sb_script_icon.as_str()).build();
    let ent_cmd = EntryRow::builder().title(tr.sb_script_cmd.as_str()).build();
    let ent_click = EntryRow::builder().title(tr.sb_script_click.as_str()).build();
    let interval = Rc::new(std::cell::Cell::new(5.0));
    let iv = Rc::clone(&interval);
    let row_int = rows::slider_row(&tr.sb_script_interval, 1.0, 300.0, 1.0, 5.0, |v| format!("{} s", v), move |v| iv.set(v));
    let row_create = ActionRow::new();
    let btn_create = Button::builder()
        .label(tr.sb_script_create.as_str())
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .css_classes(["suggested-action", "pill"])
        .build();
    row_create.add_suffix(&btn_create);
    for w in [ent_name.upcast_ref::<gtk4::Widget>(), ent_icon.upcast_ref(), ent_cmd.upcast_ref(), ent_click.upcast_ref(), row_int.upcast_ref(), row_create.upcast_ref()] {
        form.append(w);
    }
    {
        let (sh, rs, rp, nt, need) = (Rc::clone(&shell_state), Rc::clone(&refresh_scripts), Rc::clone(&refresh_picker_models), Rc::clone(&notify), tr.sb_script_need_cmd.clone());
        let (e_name, e_icon, e_cmd, e_click) = (ent_name.clone(), ent_icon.clone(), ent_cmd.clone(), ent_click.clone());
        btn_create.connect_clicked(move |_| {
            let cmd = e_cmd.text().trim().to_string();
            if cmd.is_empty() {
                nt(&need);
                return;
            }
            let name = Some(e_name.text().trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "Custom Script".into());
            let icon = Some(e_icon.text().trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "💻".into());
            let on_click = Some(e_click.text().trim().to_string()).filter(|s| !s.is_empty());

            let mut cfg = sh.borrow_mut();
            let id = unique_script_id(&name, &cfg.custom_scripts);
            cfg.custom_scripts.push(CustomScriptModule {
                id,
                name,
                icon,
                command: cmd,
                interval_seconds: interval.get() as u32,
                on_click,
            });
            themes::sync_shell_config(&cfg);
            drop(cfg);
            for e in [&e_name, &e_icon, &e_cmd, &e_click] {
                e.set_text("");
            }
            rs();
            rp();
        });
    }
    group_scripts.add(&form);
    page.add(&group_scripts);

    // ========================================================
    // 4. Vorm & positie
    // ========================================================
    let init = shell_state.borrow().clone();
    let px = |v: f64| format!("{} px", v);
    let pct = |v: f64| format!("{:.0}%", v * 100.0);

    let group_geom = PreferencesGroup::builder().title(&tr.bar_styling).description(&tr.sb_geom_desc).build();

    let styles = ["unified", "islands", "minimal"];
    let row_style = ComboRow::builder()
        .title(tr.bar_style.as_str())
        .subtitle(tr.sb_style_sub.as_str())
        .model(&StringList::new(&[&tr.sb_style_unified, &tr.sb_style_islands, &tr.sb_style_minimal]))
        .selected(styles.iter().position(|s| *s == init.layout.bar_style).unwrap_or(0) as u32)
        .build();
    let c = Rc::clone(&commit);
    row_style.connect_selected_notify(move |r| {
        let v = styles[r.selected() as usize % styles.len()];
        c(&|sh| sh.layout.bar_style = v.to_string());
    });
    group_geom.add(&row_style);

    let positions = ["top", "bottom", "left", "right"];
    let row_pos = ComboRow::builder()
        .title(tr.bar_position.as_str())
        .model(&StringList::new(&[&tr.sb_pos_top, &tr.sb_pos_bottom, &tr.sb_pos_left, &tr.sb_pos_right]))
        .selected(positions.iter().position(|p| *p == init.layout.position).unwrap_or(0) as u32)
        .build();
    let c = Rc::clone(&commit);
    row_pos.connect_selected_notify(move |r| {
        let v = positions[r.selected() as usize % positions.len()];
        c(&|sh| sh.layout.position = v.to_string());
    });
    group_geom.add(&row_pos);

    // Schuifrijen: (titel, min, max, stap, waarde, setter)
    type IntSet = fn(&mut ZenithShellConfig, i32);
    let int_rows: [(&String, f64, f64, f64, i32, IntSet); 7] = [
        (&tr.bar_height, 20.0, 64.0, 2.0, init.layout.height, |sh, v| sh.layout.height = v),
        (&tr.sb_rounding, 0.0, 32.0, 1.0, init.styling.rounding, |sh, v| sh.styling.rounding = v),
        (&tr.sb_pill_rounding, 0.0, 24.0, 1.0, init.styling.pill_rounding, |sh, v| sh.styling.pill_rounding = v),
        (&tr.sb_margin_h, 0.0, 32.0, 1.0, init.styling.margin_h, |sh, v| sh.styling.margin_h = v),
        (&tr.sb_margin_v, 0.0, 24.0, 1.0, init.styling.margin_v, |sh, v| sh.styling.margin_v = v),
        (&tr.sb_spacing, 2.0, 24.0, 1.0, init.styling.module_spacing, |sh, v| sh.styling.module_spacing = v),
        (&tr.sb_border_width, 0.0, 6.0, 1.0, init.styling.border_width, |sh, v| sh.styling.border_width = v),
    ];
    let c = Rc::clone(&commit);
    group_geom.add(&rows::switch_row(&tr.bar_floating, &tr.sb_floating_sub, init.layout.floating, move |on| c(&|sh| sh.layout.floating = on)));
    for (title, min, max, step, value, set) in int_rows {
        let c = Rc::clone(&commit);
        group_geom.add(&rows::slider_row(title, min, max, step, value as f64, px, move |v| c(&|sh| set(sh, v as i32))));
    }
    page.add(&group_geom);

    // ========================================================
    // 5. Kleuren & tekst
    // ========================================================
    let group_colors = PreferencesGroup::builder().title(crate::ui::escape::pango_escape(&tr.sb_colors)).description(&tr.sb_colors_desc).build();
    type ColorSet = fn(&mut ZenithShellConfig, String);
    let color_rows: [(&String, &str, &str, ColorSet); 5] = [
        (&tr.sb_bg, "", &init.styling.background, |sh, v| sh.styling.background = v),
        (&tr.sb_pill_bg, "", &init.styling.pill_bg, |sh, v| sh.styling.pill_bg = v),
        (&tr.sb_accent, &tr.sb_accent_sub, &init.styling.accent, |sh, v| sh.styling.accent = v),
        (&tr.sb_text, "", &init.styling.text_color, |sh, v| sh.styling.text_color = v),
        (&tr.sb_border_color, "", &init.styling.border_color, |sh, v| sh.styling.border_color = v),
    ];
    for (title, sub, hex, set) in color_rows {
        let c = Rc::clone(&commit);
        group_colors.add(&rows::color_row(title, sub, hex, move |hex| c(&|sh| set(sh, format!("#{}", hex)))));
    }
    let c = Rc::clone(&commit);
    group_colors.add(&rows::slider_row(&tr.sb_opacity, 0.10, 1.0, 0.05, init.styling.opacity, pct, move |v| c(&|sh| sh.styling.opacity = v)));
    let c = Rc::clone(&commit);
    group_colors.add(&rows::slider_row(&tr.sb_pill_opacity, 0.10, 1.0, 0.05, init.styling.pill_opacity, pct, move |v| c(&|sh| sh.styling.pill_opacity = v)));
    let c = Rc::clone(&commit);
    group_colors.add(&rows::slider_row(&tr.sb_font_size, 8.0, 18.0, 1.0, init.styling.font_size as f64, |v| format!("{} pt", v), move |v| c(&|sh| sh.styling.font_size = v as i32)));
    page.add(&group_colors);

    // ========================================================
    // 5b. Animaties in de balk (shell.qml leest `animations`)
    // ========================================================
    let group_anim = PreferencesGroup::builder().title(&tr.hypr_animations).description(&tr.sb_anim_desc).build();
    let curves = ["OutCubic", "OutBack", "OutElastic", "InOutQuad", "Linear"];
    let row_curve = ComboRow::builder()
        .title(tr.sb_anim_curve.as_str())
        .model(&StringList::new(&[&tr.anim_smooth, &tr.anim_bouncy, &tr.anim_elastic, &tr.anim_soft, &tr.anim_linear]))
        .selected(curves.iter().position(|c| *c == init.animations.curve).unwrap_or(0) as u32)
        .build();
    let c = Rc::clone(&commit);
    row_curve.connect_selected_notify(move |r| {
        let v = curves[r.selected() as usize % curves.len()];
        c(&|sh| sh.animations.curve = v.to_string());
    });
    group_anim.add(&row_curve);
    let c = Rc::clone(&commit);
    group_anim.add(&rows::slider_row(&tr.sb_anim_duration, 0.0, 800.0, 10.0, init.animations.duration_ms as f64, |v| format!("{} ms", v), move |v| {
        c(&|sh| sh.animations.duration_ms = v as i32)
    }));
    let c = Rc::clone(&commit);
    group_anim.add(&rows::switch_row(&tr.sb_hover_zoom, &tr.sb_hover_zoom_sub, init.animations.hover_zoom, move |on| c(&|sh| sh.animations.hover_zoom = on)));
    let c = Rc::clone(&commit);
    group_anim.add(&rows::switch_row(&tr.sb_fade_in, &tr.sb_fade_in_sub, init.animations.fade_in, move |on| c(&|sh| sh.animations.fade_in = on)));
    page.add(&group_anim);

    // ========================================================
    // 6. Panelen & pop-ups
    // ========================================================
    let group_panels = PreferencesGroup::builder().title(crate::ui::escape::pango_escape(&tr.sb_panels)).description(&tr.sb_panels_desc).build();
    type BoolSet = fn(&mut ZenithShellConfig, bool);
    let panel_rows: [(&String, &String, bool, BoolSet); 5] = [
        (&tr.sb_p_qs, &tr.sb_p_qs_sub, init.panels.quick_settings, |sh, v| sh.panels.quick_settings = v),
        (&tr.sb_p_cc, &tr.sb_p_cc_sub, init.panels.control_center, |sh, v| sh.panels.control_center = v),
        (&tr.sb_p_vosd, &tr.sb_p_vosd_sub, init.panels.volume_osd, |sh, v| sh.panels.volume_osd = v),
        (&tr.sb_p_bosd, &tr.sb_p_bosd_sub, init.panels.brightness_osd, |sh, v| sh.panels.brightness_osd = v),
        (&tr.sb_p_notif, &tr.sb_p_notif_sub, init.panels.notifications, |sh, v| sh.panels.notifications = v),
    ];
    for (title, sub, active, set) in panel_rows {
        let c = Rc::clone(&commit);
        group_panels.add(&rows::switch_row(title, sub, active, move |on| c(&|sh| set(sh, on))));
    }
    page.add(&group_panels);

    // ========================================================
    // 7. Geavanceerd: bestanden en eigen QML
    // ========================================================
    let group_power = PreferencesGroup::builder().title(&tr.sb_power).description(&tr.sb_power_desc).build();
    let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
    let qs_dir = home.join(".config/quickshell");
    for (title, sub, path) in [
        (&tr.sb_mod_dir, tr.sb_mod_dir_sub.clone(), qs_dir.join("modules")),
        (&tr.sb_json, "~/.config/quickshell/zenith-shell.json".to_string(), qs_dir.join("zenith-shell.json")),
        (&tr.sb_qml, "~/.config/quickshell/shell.qml".to_string(), qs_dir.join("shell.qml")),
    ] {
        let row = ActionRow::builder().title(title.as_str()).subtitle(sub).build();
        let btn = Button::builder().label(tr.sb_open.as_str()).valign(gtk4::Align::Center).build();
        btn.connect_clicked(move |_| {
            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
        });
        row.add_suffix(&btn);
        group_power.add(&row);
    }

    let row_tmpl = ActionRow::builder().title(tr.sb_tmpl.as_str()).subtitle(tr.sb_tmpl_sub.as_str()).build();
    let btn_tmpl = Button::builder().label(tr.sb_tmpl_btn.as_str()).valign(gtk4::Align::Center).build();
    {
        let (ra, rp, dir) = (Rc::clone(&refresh_all_modules), Rc::clone(&refresh_picker_models), qs_dir.join("modules"));
        btn_tmpl.connect_clicked(move |_| {
            let dest = dir.join("CustomWidget.qml");
            if !dest.exists() {
                let example = dir.join("CustomWidget.qml.example");
                if example.exists() {
                    let _ = std::fs::copy(&example, &dest);
                } else {
                    let _ = std::fs::create_dir_all(&dir);
                    let boilerplate = "import QtQuick\nimport QtQuick.Layouts\n\nRectangle {\n    implicitWidth: txt.implicitWidth + 16\n    implicitHeight: 22\n    radius: 6\n    color: Qt.alpha(root.themeAccent || \"#89b4fa\", 0.20)\n    border.color: root.themeAccent || \"#89b4fa\"\n    border.width: 1\n    Text { id: txt; anchors.centerIn: parent; text: \"✨ Custom Widget\"; color: root.fgColor || \"#cdd6f4\"; font.pixelSize: root.userFontSize || 11 }\n}\n";
                    let _ = std::fs::write(&dest, boilerplate);
                }
            }
            let _ = std::process::Command::new("xdg-open").arg(&dest).spawn();
            ra();
            rp();
        });
    }
    row_tmpl.add_suffix(&btn_tmpl);
    group_power.add(&row_tmpl);

    let row_restart = ActionRow::builder().title(tr.sb_restart.as_str()).subtitle(tr.sb_restart_sub.as_str()).build();
    let btn_restart = Button::builder().label(tr.sb_restart_btn.as_str()).valign(gtk4::Align::Center).build();
    btn_restart.connect_clicked(|_| process::set_active_bar("quickshell"));
    row_restart.add_suffix(&btn_restart);
    group_power.add(&row_restart);
    page.add(&group_power);

    page
}

fn boxed_list() -> ListBox {
    ListBox::builder().selection_mode(gtk4::SelectionMode::None).css_classes(["boxed-list"]).build()
}

/// Klein kleurbolletje als voorbeeld van een stijl.
fn swatch(hex: &str) -> Label {
    let l = Label::builder().width_request(14).height_request(14).valign(gtk4::Align::Center).css_classes(["zenith-swatch"]).build();
    let prov = gtk4::CssProvider::new();
    prov.load_from_data(&format!("label {{ background-color: {}; }}", hex));
    #[allow(deprecated)]
    l.style_context().add_provider(&prov, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    l
}

/// Kaart voor een ingebouwde stijl: naam, kleurvoorbeeld, één klik = toepassen.
fn style_card(style: &BarStyle, state: &Rc<RefCell<ZenithConfig>>, sh: &Shell, rebuild: &Rc<dyn Fn()>, notify: &Notify, tr: &Translations) -> Button {
    let content = Box::new(Orientation::Vertical, 8);
    let colors = Box::new(Orientation::Horizontal, 4);
    colors.set_halign(gtk4::Align::Center);
    for hex in [&style.styling.background, &style.styling.pill_bg, &style.styling.accent, &style.styling.text_color] {
        colors.append(&swatch(hex));
    }
    content.append(&colors);
    content.append(&Label::new(Some(&style.name)));
    let btn = Button::builder().child(&content).css_classes(["zenith-preset"]).build();
    let (st, sh, rb, nt, msg, s) = (Rc::clone(state), Rc::clone(sh), Rc::clone(rebuild), Rc::clone(notify), format!("{} — {}", style.name, tr.sb_style_applied), style.clone());
    btn.connect_clicked(move |_| {
        apply_style(&mut sh.borrow_mut(), &mut st.borrow_mut(), &s);
        themes::sync_shell_config(&sh.borrow());
        config::save_config(&st.borrow());
        nt(&msg);
        rb();
    });
    btn
}

fn module_labels(sh: &Shell) -> Vec<String> {
    sh.borrow().discover_modules_for_instance().iter().map(|m| format!("{} {}", m.icon, m.name)).collect()
}

/// Veilige, unieke id op basis van de naam (bv. "Mijn IP" → "mijn_ip", "mijn_ip_1", …).
fn unique_script_id(name: &str, existing: &[CustomScriptModule]) -> String {
    let base: String = name.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
    let base = Some(base.trim_matches('_').to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "custom_script".into());
    let mut id = base.clone();
    let mut n = 1;
    while existing.iter().any(|s| s.id == id) {
        id = format!("{}_{}", base, n);
        n += 1;
    }
    id
}

fn slot_list<'a>(sh: &'a mut ZenithShellConfig, slot: &str) -> Option<&'a mut Vec<String>> {
    match slot {
        "left" => Some(&mut sh.modules.left),
        "center" => Some(&mut sh.modules.center),
        "right" => Some(&mut sh.modules.right),
        _ => None,
    }
}

/// Voegt de "Module toevoegen"-rij toe aan een gedeelte.
fn add_module_picker_row(
    group: &PreferencesGroup,
    slot: &'static str,
    shell_state: &Shell,
    list_box: &Rc<ListBox>,
    picker_dropdowns: &Rc<RefCell<Vec<DropDown>>>,
    tr: &Rc<Translations>,
) {
    let row_add = ActionRow::builder().title(tr.sb_add_module.as_str()).build();
    let labels = module_labels(shell_state);
    let labels_ref: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
    let dd = DropDown::builder().model(&StringList::new(&labels_ref)).valign(gtk4::Align::Center).build();
    picker_dropdowns.borrow_mut().push(dd.clone());

    let btn_add = Button::builder().label(tr.sb_add.as_str()).valign(gtk4::Align::Center).build();
    let (sh, lb, dd_c, tr) = (Rc::clone(shell_state), Rc::clone(list_box), dd.clone(), Rc::clone(tr));
    btn_add.connect_clicked(move |_| {
        let modules = sh.borrow().discover_modules_for_instance();
        if let Some(info) = modules.get(dd_c.selected() as usize) {
            let mut cfg = sh.borrow_mut();
            if let Some(list) = slot_list(&mut cfg, slot) {
                list.push(info.id.clone());
            }
            themes::sync_shell_config(&cfg);
            drop(cfg);
            refresh_slot(&lb, slot, &sh, &tr);
        }
    });

    row_add.add_suffix(&dd);
    row_add.add_suffix(&btn_add);
    group.add(&row_add);
}

/// Ververst de rijen van één gedeelte (links/midden/rechts).
fn refresh_slot(list_box: &ListBox, slot: &'static str, shell_state: &Shell, tr: &Rc<Translations>) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let all_modules = shell_state.borrow().discover_modules_for_instance();
    let current: Vec<String> = slot_list(&mut shell_state.borrow_mut(), slot).map(|l| l.clone()).unwrap_or_default();
    if current.is_empty() {
        list_box.append(&ActionRow::builder().title(tr.sb_no_modules.as_str()).css_classes(["zenith-dim"]).build());
        return;
    }

    let total = current.len();
    for (idx, mod_id) in current.iter().enumerate() {
        let info = all_modules.iter().find(|m| &m.id == mod_id);
        let title = info.map(|i| format!("{} {}", i.icon, i.name)).unwrap_or_else(|| format!("🧩 {}", mod_id));
        let desc = info.map(|i| i.description.clone()).unwrap_or_else(|| mod_id.clone());
        let row = ActionRow::builder()
            .title(crate::ui::escape::pango_escape(&title))
            .subtitle(crate::ui::escape::pango_escape(&desc))
            .build();

        // (icoon, tooltip, zichtbaar, actie op de lijst)
        type Op = fn(&mut Vec<String>, usize);
        let ops: [(&str, &str, bool, Op); 3] = [
            ("go-up-symbolic", &tr.sb_move_up, idx > 0, |l, i| if i > 0 && i < l.len() { l.swap(i, i - 1) }),
            ("go-down-symbolic", &tr.sb_move_down, idx + 1 < total, |l, i| if i + 1 < l.len() { l.swap(i, i + 1) }),
            ("list-remove-symbolic", &tr.sb_remove, true, |l, i| if i < l.len() { l.remove(i); }),
        ];
        for (icon, tip, visible, op) in ops {
            let btn = Button::builder().icon_name(icon).tooltip_text(tip).valign(gtk4::Align::Center).css_classes(["flat"]).build();
            btn.set_sensitive(visible);
            let (sh, lb, tr) = (Rc::clone(shell_state), list_box.clone(), Rc::clone(tr));
            btn.connect_clicked(move |_| {
                let mut cfg = sh.borrow_mut();
                if let Some(list) = slot_list(&mut cfg, slot) {
                    op(list, idx);
                }
                themes::sync_shell_config(&cfg);
                drop(cfg);
                refresh_slot(&lb, slot, &sh, &tr);
            });
            row.add_suffix(&btn);
        }
        list_box.append(&row);
    }
}

/// Ververst de lijst met scriptmodules.
fn refresh_scripts_list(list_box: &ListBox, shell_state: &Shell, refresh_all: &Rc<dyn Fn()>, refresh_pickers: &Rc<dyn Fn()>, tr: &Rc<Translations>) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let scripts = shell_state.borrow().custom_scripts.clone();
    if scripts.is_empty() {
        list_box.append(&ActionRow::builder().title(tr.sb_no_scripts.as_str()).css_classes(["zenith-dim"]).build());
        return;
    }

    for script in scripts {
        let row = ActionRow::builder()
            .title(crate::ui::escape::pango_escape(&format!("{} {}", script.icon, script.name)))
            .subtitle(crate::ui::escape::pango_escape(&format!("{} · {} s", script.command, script.interval_seconds)))
            .build();
        let btn_remove = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text(tr.sb_remove.as_str())
            .valign(gtk4::Align::Center)
            .css_classes(["flat"])
            .build();
        let (sh, lb, ra, rp, tr) = (Rc::clone(shell_state), list_box.clone(), Rc::clone(refresh_all), Rc::clone(refresh_pickers), Rc::clone(tr));
        btn_remove.connect_clicked(move |_| {
            let mut cfg = sh.borrow_mut();
            cfg.custom_scripts.retain(|s| s.id != script.id);
            // Ook actieve plaatsingen op de balk weghalen.
            let full_id = format!("script:{}", script.id);
            for slot in ["left", "center", "right"] {
                if let Some(list) = slot_list(&mut cfg, slot) {
                    list.retain(|m| m != &full_id);
                }
            }
            themes::sync_shell_config(&cfg);
            drop(cfg);
            refresh_scripts_list(&lb, &sh, &ra, &rp, &tr);
            ra();
            rp();
        });
        row.add_suffix(&btn_remove);
        list_box.append(&row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_script_id_is_safe_and_unique() {
        let mk = |id: &str| CustomScriptModule { id: id.into(), name: String::new(), icon: String::new(), command: String::new(), interval_seconds: 5, on_click: None };
        assert_eq!(unique_script_id("Mijn IP!", &[]), "mijn_ip");
        assert_eq!(unique_script_id("Mijn IP", &[mk("mijn_ip"), mk("mijn_ip_1")]), "mijn_ip_2");
        assert_eq!(unique_script_id("  ", &[]), "custom_script");
    }

    #[test]
    fn apply_style_mirrors_to_legacy_config() {
        let mut sh = ZenithShellConfig::default();
        let mut cfg = ZenithConfig::default();
        let style = &shell_config::builtin_bar_styles()[4];
        apply_style(&mut sh, &mut cfg, style);
        assert_eq!(sh.layout.position, "bottom");
        assert_eq!(cfg.qs_position, "bottom");
        assert_eq!(cfg.qs_accent, style.styling.accent.trim_start_matches('#'));
        assert_eq!(sh.modules, style.modules);
    }
}
