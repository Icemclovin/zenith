// Desktopwidgets-editor: een verkleind scherm met je wallpaper waarop je widgets versleept.
// Elke wijziging gaat direct naar zenith-desktop.json, zodat de echte desktop live meebeweegt.

use gtk4::prelude::*;
use gtk4::{Box, Button, Fixed, GestureDrag, Label, Orientation, Overlay, Picture, Scale, Switch};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, EntryRow, PreferencesGroup, PreferencesPage};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::backend::desktop_widgets::{kind_info, place, DesktopWidget, DesktopWidgetsConfig, KINDS};
use crate::backend::shell_config::ZenithShellConfig;

const CANVAS_W: f64 = 620.0;

/// Gedeelde staat van de editor.
struct Editor {
    cfg: RefCell<DesktopWidgetsConfig>,
    fixed: Fixed,
    /// Logische schermgrootte van de eerste monitor (widgets worden in px getekend).
    screen: (f64, f64),
    canvas_h: f64,
    list: PreferencesGroup,
    rows: RefCell<Vec<ActionRow>>,
}

impl Editor {
    fn save(&self) {
        if let Err(e) = self.cfg.borrow().save() {
            eprintln!("zenith-desktop.json opslaan mislukt: {e}");
        }
    }

    /// Grootte van widget `w` als fractie van het scherm.
    fn frac_size(&self, w: &DesktopWidget) -> (f64, f64) {
        let (_, pw, ph) = kind_info(&w.kind);
        (pw as f64 * w.scale / self.screen.0, ph as f64 * w.scale / self.screen.1)
    }

    fn redraw_canvas(&self) {
        while let Some(c) = self.fixed.first_child() {
            self.fixed.remove(&c);
        }
        for (i, w) in self.cfg.borrow().items.iter().enumerate() {
            let (name, _, _) = kind_info(&w.kind);
            let (fw, fh) = self.frac_size(w);
            let chip = Label::builder()
                .label(name)
                .css_classes(["card", "zenith-desktop-chip"])
                .build();
            chip.set_widget_name(&format!("dw-{i}"));
            chip.set_size_request((fw * CANVAS_W) as i32, (fh * self.canvas_h) as i32);
            self.fixed.put(&chip, w.x * CANVAS_W, w.y * self.canvas_h);
        }
    }
}

fn rebuild_list(ed: &Rc<Editor>) {
    for r in ed.rows.borrow_mut().drain(..) {
        ed.list.remove(&r);
    }
    let count = ed.cfg.borrow().items.len();
    for i in 0..count {
        let w = ed.cfg.borrow().items[i].clone();
        let row = ActionRow::builder().title(kind_info(&w.kind).0).subtitle("Grootte").build();

        let scale = Scale::with_range(Orientation::Horizontal, 0.5, 2.0, 0.05);
        scale.set_value(w.scale);
        scale.set_width_request(160);
        scale.set_valign(gtk4::Align::Center);
        {
            let ed = Rc::clone(ed);
            scale.connect_value_changed(move |s| {
                {
                    let mut cfg = ed.cfg.borrow_mut();
                    let snap = cfg.grid_snap;
                    cfg.items[i].scale = s.value();
                    let it = cfg.items[i].clone();
                    let (fw, fh) = ed.frac_size(&it);
                    (cfg.items[i].x, cfg.items[i].y) = place(it.x, it.y, fw, fh, snap);
                }
                ed.redraw_canvas();
                ed.save();
            });
        }
        row.add_suffix(&scale);

        let del = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Verwijderen")
            .valign(gtk4::Align::Center)
            .css_classes(["flat", "destructive-action"])
            .build();
        {
            let ed = Rc::clone(ed);
            del.connect_clicked(move |_| {
                ed.cfg.borrow_mut().items.remove(i);
                ed.save();
                ed.redraw_canvas();
                rebuild_list(&ed);
            });
        }
        row.add_suffix(&del);
        ed.list.add(&row);
        ed.rows.borrow_mut().push(row);
    }
}

fn first_monitor_size() -> (f64, f64) {
    gtk4::gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gtk4::gdk::Monitor>().ok())
        .map(|m| (m.geometry().width() as f64, m.geometry().height() as f64))
        .filter(|(w, h)| *w > 0.0 && *h > 0.0)
        .unwrap_or((1920.0, 1080.0))
}

pub fn build_desktop_widgets_page() -> PreferencesPage {
    let page = PreferencesPage::new();
    let screen = first_monitor_size();
    let canvas_h = CANVAS_W * screen.1 / screen.0;

    // --- Algemeen ---
    let general = PreferencesGroup::builder()
        .title("Desktopwidgets")
        .description("Widgets op je bureaublad, achter je vensters. Wijzigingen zie je meteen.")
        .build();
    let list = PreferencesGroup::builder()
        .title("Widgets")
        .description("Pas de grootte aan of verwijder een widget")
        .build();

    let ed = Rc::new(Editor {
        cfg: RefCell::new(DesktopWidgetsConfig::load()),
        fixed: Fixed::new(),
        screen,
        canvas_h,
        list: list.clone(),
        rows: RefCell::new(Vec::new()),
    });

    let switch_row = |title: &str, sub: &str, active: bool, f: fn(&mut DesktopWidgetsConfig, bool)| {
        let row = ActionRow::builder().title(title).subtitle(sub).build();
        let sw = Switch::builder().active(active).valign(gtk4::Align::Center).build();
        let ed = Rc::clone(&ed);
        sw.connect_active_notify(move |s| {
            f(&mut ed.cfg.borrow_mut(), s.is_active());
            ed.save();
        });
        row.add_suffix(&sw);
        row.set_activatable_widget(Some(&sw));
        row
    };
    let (enabled, snap, loc) = {
        let c = ed.cfg.borrow();
        (c.enabled, c.grid_snap, c.weather_location.clone())
    };
    general.add(&switch_row("Widgets tonen", "Zet alle desktopwidgets aan of uit", enabled, |c, v| c.enabled = v));
    general.add(&switch_row("Uitlijnen op raster", "Widgets klikken vast op een raster van 2,5% van het scherm", snap, |c, v| c.grid_snap = v));

    let loc_row = EntryRow::builder().title("Plaats voor het weer (leeg = automatisch)").text(loc).show_apply_button(true).build();
    {
        let ed = Rc::clone(&ed);
        loc_row.connect_apply(move |r| {
            ed.cfg.borrow_mut().weather_location = r.text().trim().to_string();
            ed.save();
        });
    }
    general.add(&loc_row);
    page.add(&general);

    // --- Editor ---
    let editor_group = PreferencesGroup::builder()
        .title("Editor")
        .description("Sleep widgets naar hun plek. Het voorbeeld heeft de verhouding van je scherm.")
        .build();

    let overlay = Overlay::new();
    overlay.set_size_request(CANVAS_W as i32, canvas_h as i32);
    overlay.set_halign(gtk4::Align::Center);
    overlay.set_overflow(gtk4::Overflow::Hidden);
    overlay.add_css_class("card");
    let wallpaper = ZenithShellConfig::load_or_default().wallpaper_path;
    let bg = if wallpaper.is_empty() { Picture::new() } else { Picture::for_filename(&wallpaper) };
    bg.set_content_fit(gtk4::ContentFit::Cover);
    bg.set_can_shrink(true);
    overlay.set_child(Some(&bg));
    ed.fixed.set_size_request(CANVAS_W as i32, canvas_h as i32);
    overlay.add_overlay(&ed.fixed);

    // Eén drag-gebaar op het canvas: een gebaar op de chip zelf verspringt zodra de chip beweegt.
    let drag = GestureDrag::new();
    let dragging: Rc<Cell<Option<(usize, f64, f64)>>> = Rc::new(Cell::new(None));
    {
        let ed = Rc::clone(&ed);
        let dragging = Rc::clone(&dragging);
        drag.connect_drag_begin(move |_, x, y| {
            let mut w = ed.fixed.pick(x, y, gtk4::PickFlags::DEFAULT);
            while let Some(widget) = w {
                if let Some(i) = widget.widget_name().strip_prefix("dw-").and_then(|n| n.parse::<usize>().ok()) {
                    let (px, py) = ed.fixed.child_position(&widget);
                    dragging.set(Some((i, px, py)));
                    return;
                }
                w = widget.parent();
            }
            dragging.set(None);
        });
    }
    let target = move |ed: &Editor, i: usize, sx: f64, sy: f64, dx: f64, dy: f64| {
        let cfg = ed.cfg.borrow();
        let (fw, fh) = ed.frac_size(&cfg.items[i]);
        place((sx + dx) / CANVAS_W, (sy + dy) / ed.canvas_h, fw, fh, cfg.grid_snap)
    };
    {
        let ed = Rc::clone(&ed);
        let dragging = Rc::clone(&dragging);
        drag.connect_drag_update(move |_, dx, dy| {
            let Some((i, sx, sy)) = dragging.get() else { return };
            let (x, y) = target(&ed, i, sx, sy, dx, dy);
            let mut c = ed.fixed.first_child();
            while let Some(chip) = c {
                if chip.widget_name() == format!("dw-{i}") {
                    ed.fixed.move_(&chip, x * CANVAS_W, y * ed.canvas_h);
                }
                c = chip.next_sibling();
            }
        });
    }
    {
        let ed = Rc::clone(&ed);
        drag.connect_drag_end(move |_, dx, dy| {
            let Some((i, sx, sy)) = dragging.take() else { return };
            let (x, y) = target(&ed, i, sx, sy, dx, dy);
            {
                let mut cfg = ed.cfg.borrow_mut();
                cfg.items[i].x = x;
                cfg.items[i].y = y;
            }
            ed.save();
        });
    }
    ed.fixed.add_controller(drag);
    editor_group.add(&overlay);

    // Toevoegknoppen
    let add_box = Box::new(Orientation::Horizontal, 8);
    add_box.set_halign(gtk4::Align::Center);
    add_box.set_margin_top(12);
    for (kind, name, _, _) in KINDS {
        let b = Button::builder().label(format!("+ {name}")).css_classes(["pill"]).build();
        let ed = Rc::clone(&ed);
        b.connect_clicked(move |_| {
            {
                let mut cfg = ed.cfg.borrow_mut();
                let n = cfg.items.len() as f64;
                // Iets verschoven per widget, zodat nieuwe widgets niet precies op elkaar vallen.
                let mut w = DesktopWidget { kind: kind.to_string(), x: 0.05 + 0.05 * n, y: 0.08 + 0.05 * n, scale: 1.0 };
                let (fw, fh) = ed.frac_size(&w);
                (w.x, w.y) = place(w.x, w.y, fw, fh, cfg.grid_snap);
                cfg.items.push(w);
            }
            ed.save();
            ed.redraw_canvas();
            rebuild_list(&ed);
        });
        add_box.append(&b);
    }
    editor_group.add(&add_box);
    page.add(&editor_group);
    page.add(&list);

    ed.redraw_canvas();
    rebuild_list(&ed);
    page
}
