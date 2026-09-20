// ZenithOS — Statusbalk Drag-and-Drop-Canvas (EPIC-02 / H5 UI&UX).
//
// Interactief canvas dat de statusbalk van Quickshell simuleert. De gebruiker
// kan modules over drie zones slepen — links, midden, rechts — door een module-
// chip op te pakken en naar een andere zone te slepen. Bij loslaten wordt de
// positie via JSON-RPC `update_module_position` naar zenithd gestuurd (live via
// de socket), waarna de balk via SIGUSR1 hertekent. Wanneer de daemon niet
// draait, vallen we terug op een lokale config-write + SIGUSR1.

use gtk4::prelude::*;
use gtk4::{Box, Button, DropTarget, DragSource, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::i18n::Translations;
use crate::backend::ipc_client;
use crate::backend::shell_config::{ModuleInfo, ZenithShellConfig};
use crate::backend::themes;

/// Payload dat tijdens een drag tussen bronchip en drop-zone wordt gedeeld.
struct DragPayload {
    module_id: String,
}

/// Verplaats een module naar een zone (append aan het einde van de doelzone).
/// - Daemon draait: laat zenithd de wijziging atomair wegschrijven + SIGUSR1.
/// - Daemon uit: lokale mutatie + `sync_shell_config` (save + SIGUSR1).
fn apply_move(
    state: &Rc<RefCell<ZenithShellConfig>>,
    module_id: &str,
    target_zone: &str,
) -> bool {
    let mut sh = state.borrow_mut();

    // Doel-index = huidige lengte van de doelzone (append).
    let target_len = match target_zone {
        "left" => sh.modules.left.len() as i64,
        "center" => sh.modules.center.len() as i64,
        _ => sh.modules.right.len() as i64,
    };

    // Verwijder de module uit eender welke zone waar die momenteel staat.
    if let Some(idx) = sh.modules.left.iter().position(|m| *m == module_id) {
        sh.modules.left.remove(idx);
    }
    if let Some(idx) = sh.modules.center.iter().position(|m| *m == module_id) {
        sh.modules.center.remove(idx);
    }
    if let Some(idx) = sh.modules.right.iter().position(|m| *m == module_id) {
        sh.modules.right.remove(idx);
    }

    // Voeg toe aan de doelzone (module is hierboven al uit alle zones verwijderd).
    match target_zone {
        "left" => sh.modules.left.push(module_id.to_string()),
        "center" => sh.modules.center.push(module_id.to_string()),
        _ => sh.modules.right.push(module_id.to_string()),
    }
    drop(sh);

    // Favoriete pad: JSON-RPC naar zenithd.
    if ipc_client::is_daemon_running()
        && ipc_client::update_module_position(module_id, target_len, target_zone)
    {
        return true;
    }

    // Fallback: lokaal herladen (al gemuteerd hierboven).
    let sh = state.borrow_mut();
    themes::sync_shell_config(&sh);
    true
}

/// Bouw de module-chip-knop (draagbaar) met het juiste icon + naam.
fn build_chip(
    payload: &Rc<RefCell<DragPayload>>,
    info: &ModuleInfo,
) -> Button {
    let text = format!("{} {}", info.icon, info.name);
    let chip = Button::builder()
        .label(crate::ui::escape::pango_escape(&text))
        .css_classes(["zenith-drag-chip"])
        .build();
    chip.set_width_request(110);
    chip.set_height_request(40);

    let ds = DragSource::new();
    ds.set_actions(gtk4::gdk::DragAction::MOVE);
    ds.set_content(Some(&gtk4::gdk::ContentProvider::for_value(&info.id.to_value())));
    {
        let pl = Rc::clone(payload);
        let mid = info.id.clone();
        ds.connect_drag_begin(move |_, _drag| {
            pl.borrow_mut().module_id = mid.clone();
        });
    }
    chip.add_controller(ds);
    chip
}

/// Hertekent de chips van de drie zones (op basis van de gedeelde config-staat).
fn rebuild_canvas(
    zl: &Box,
    zc: &Box,
    zr: &Box,
    st: &Rc<RefCell<ZenithShellConfig>>,
    pl: &Rc<RefCell<DragPayload>>,
) {
    while let Some(child) = zl.first_child() {
        zl.remove(&child);
    }
    while let Some(child) = zc.first_child() {
        zc.remove(&child);
    }
    while let Some(child) = zr.first_child() {
        zr.remove(&child);
    }

    let all = ZenithShellConfig::discover_available_modules();
    let sh = st.borrow();
    let left_ids = sh.modules.left.clone();
    let center_ids = sh.modules.center.clone();
    let right_ids = sh.modules.right.clone();
    drop(sh);

    for id in left_ids {
        let info = all.iter().find(|m| m.id == id);
        match info {
            Some(m) => zl.append(&build_chip(pl, m)),
            None => zl.append(&build_chip(pl, &ModuleInfo { id: id.clone(), name: id.clone(), icon: "🧩".to_string(), description: String::new(), is_custom: false })),
        }
    }
    for id in center_ids {
        let info = all.iter().find(|m| m.id == id);
        match info {
            Some(m) => zc.append(&build_chip(pl, m)),
            None => zc.append(&build_chip(pl, &ModuleInfo { id: id.clone(), name: id.clone(), icon: "🧩".to_string(), description: String::new(), is_custom: false })),
        }
    }
    for id in right_ids {
        let info = all.iter().find(|m| m.id == id);
        match info {
            Some(m) => zr.append(&build_chip(pl, m)),
            None => zr.append(&build_chip(pl, &ModuleInfo { id: id.clone(), name: id.clone(), icon: "🧩".to_string(), description: String::new(), is_custom: false })),
        }
    }
}

#[allow(deprecated)]
/// Bouw het interactieve sleep-canvas. Returnt het bovenliggende verticale `Box`
/// dat op de statusbalk-pagina kan worden ingevoegd.
pub fn build_drag_canvas(tr: &Translations) -> Box {
    let root = Box::new(Orientation::Vertical, 8);
    root.set_margin_top(4);
    root.set_margin_bottom(8);

    let title = gtk4::Label::builder()
        .label(crate::ui::escape::pango_escape(&tr.drag_title))
        .css_classes(["zenith-section-title"])
        .halign(gtk4::Align::Start)
        .build();
    root.append(&title);

    let hint = gtk4::Label::builder()
        .label(crate::ui::escape::pango_escape(&tr.drag_hint))
        .css_classes(["zenith-dim"])
        .halign(gtk4::Align::Start)
        .build();
    root.append(&hint);

    // Statusfeedback na een succesvolle drop.
    let status_lbl = gtk4::Label::builder()
        .css_classes(["zenith-status-ok"])
        .build();
    root.append(&status_lbl);
    let status = Rc::new(RefCell::new(status_lbl));

    let state = Rc::new(RefCell::new(ZenithShellConfig::load_or_default()));
    let payload = Rc::new(RefCell::new(DragPayload { module_id: String::new() }));

    // Drie stabiele zone-boxen (gewone widgets, klonen voor gebruik in closures).
    let zone_left = Box::new(Orientation::Vertical, 6);
    let zone_center = Box::new(Orientation::Vertical, 6);
    let zone_right = Box::new(Orientation::Vertical, 6);
    zone_left.set_css_classes(&["zenith-drop-zone"]);
    zone_center.set_css_classes(&["zenith-drop-zone"]);
    zone_right.set_css_classes(&["zenith-drop-zone"]);

    let canvas = Box::new(Orientation::Horizontal, 12);
    canvas.set_css_classes(&["zenith-canvas"]);

    // Bouw per zone een kolom + DropTarget met de juiste zone-sleutel.
    let zone_keys = ["left".to_string(), "center".to_string(), "right".to_string()];
    let zone_title_vals = [tr.drag_left.clone(), tr.drag_center.clone(), tr.drag_right.clone()];
    let saved_text = tr.drag_saved.clone();

    for i in 0..3 {
        let zkey = zone_keys[i].clone();
        let ztitle = zone_title_vals[i].clone();

        let frame_box = Box::new(Orientation::Vertical, 4);
        let lbl = gtk4::Label::builder()
            .label(crate::ui::escape::pango_escape(&ztitle))
            .css_classes(["zenith-drop-label"])
            .build();
        frame_box.append(&lbl);
        match i {
            0 => frame_box.append(&zone_left),
            1 => frame_box.append(&zone_center),
            _ => frame_box.append(&zone_right),
        }
        frame_box.set_hexpand(true);
        frame_box.set_vexpand(true);

        // DropTarget op de frame-box (zone).
        let dt = DropTarget::new(gtk4::glib::types::Type::STRING, gtk4::gdk::DragAction::MOVE);
        frame_box.add_controller(dt.clone());
        dt.connect_accept(|_, _| { true });
        dt.connect_enter(|_, _x, _y| { gtk4::gdk::DragAction::MOVE });
        dt.connect_motion(|_, _x, _y| { gtk4::gdk::DragAction::MOVE });

        let zl_c = zone_left.clone();
        let zc_c = zone_center.clone();
        let zr_c = zone_right.clone();
        let st_c = Rc::clone(&state);
        let pl_c = Rc::clone(&payload);
        let sts_c = Rc::clone(&status);
        let zkey_c = zkey.clone();
        let saved = saved_text.clone();
        dt.connect_drop_notify(move |_| {
            let pl = pl_c.borrow();
            let mid = pl.module_id.clone();
            drop(pl);
            if mid.is_empty() {
                return;
            }
            if apply_move(&st_c, mid.as_str(), zkey_c.as_str()) {
                let text = crate::ui::escape::pango_escape(&saved);
                let lbl = sts_c.borrow();
                lbl.set_label(&text);
            }
            rebuild_canvas(&zl_c, &zc_c, &zr_c, &st_c, &pl_c);
        });

        canvas.append(&frame_box);
    }
    root.append(&canvas);

    // Eerste weergave van de chips.
    rebuild_canvas(&zone_left, &zone_center, &zone_right, &state, &payload);

    root
}