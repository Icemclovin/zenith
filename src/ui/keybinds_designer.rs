use gtk4::prelude::*;
use gtk4::{Button, Entry, Label};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::i18n::Translations;
use crate::backend::keybinds;
use crate::ui::escape::pango_escape;

/// Eén bewerkbare rij: een sneltoets-id + zijn invoerveld.
struct EditableRow {
    pub id: String,
    pub entry: Entry,
    pub description: String,
    pub group: String,
}

/// Vertaalde groepsnaam voor een standaardgroepssleutel.
fn group_label(group: &str, tr: &Translations) -> String {
    match group {
        "Basis" => tr.keybinds_group_basis.clone(),
        "Focus" => tr.keybinds_group_focus.clone(),
        "Venster" => tr.keybinds_group_venster.clone(),
        "Werkruimten" => tr.keybinds_group_werkruimten.clone(),
        "Multimedia" => tr.keybinds_group_multimedia.clone(),
        "Screenshots" => tr.keybinds_group_screenshots.clone(),
        _ => tr.keybinds_group_basis.clone(),
    }
}

/// Bouwt een Entry-rij voor een Keybind.
fn make_entry(keys: &[String]) -> Entry {
    let text = keys.join(" ");
    Entry::builder().text(text).hexpand(false).build()
}

/// Verzamelt de ingevulde waarden terug naar Keybind-objecten.
fn collect_entries(rows_cell: &Rc<RefCell<Vec<EditableRow>>>) -> Vec<keybinds::Keybind> {
    let rows = rows_cell.borrow();
    let mut out: Vec<keybinds::Keybind> = Vec::new();
    for row in rows.iter() {
        let txt = row.entry.text();
        let combo = txt.split_whitespace().map(|p| p.to_string()).collect::<Vec<String>>();
        out.push(keybinds::Keybind {
            id: row.id.clone(),
            keys: combo,
            description: row.description.clone(),
            group: row.group.clone(),
        });
    }
    out
}

/// Hermaak alle Entry-teksten uit de opgegeven keybinds (na reset).
fn repopulate_entries(rows_cell: &Rc<RefCell<Vec<EditableRow>>>, binds: &[keybinds::Keybind]) {
    let rows = rows_cell.borrow_mut();
    for row in rows.iter() {
        for kb in binds.iter() {
            if kb.id == row.id {
                row.entry.set_text(&kb.keys.join(" "));
            }
        }
    }
}

/// Bouwt de "Toetsenbord & Sneltoetsen"-pagina (bewerkings-UI).
pub fn build_keybinds_page(tr: &Translations) -> PreferencesPage {
    crate::backend::keybinds::ensure_installed();

    let page = PreferencesPage::new();

    // Inleidende groep.
    let intro = PreferencesGroup::builder()
        .title(pango_escape(&tr.keybinds_title).as_str())
        .description(pango_escape(&tr.keybinds_desc).as_str())
        .build();
    page.add(&intro);

    // Laad de huidige keybinds.
    let current = keybinds::load_keybinds();

    // Bouw de rijen-vector.
    let rows_cell: Rc<RefCell<Vec<EditableRow>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let mut rows = rows_cell.borrow_mut();
        for kb in current.iter() {
            rows.push(EditableRow {
                id: kb.id.clone(),
                entry: make_entry(&kb.keys),
                description: kb.description.clone(),
                group: kb.group.clone(),
            });
        }
    }

    // Verzamel de unieke groepen in volgorde van voorkomen.
    let mut group_order: Vec<String> = Vec::new();
    {
        let rows = rows_cell.borrow();
        for row in rows.iter() {
            if !group_order.contains(&row.group) {
                group_order.push(row.group.clone());
            }
        }
    }

    // Bouw één PreferencesGroup per groep met daarin alle rijen.
    for g in group_order {
        let title = group_label(g.as_str(), tr);
        let grp = PreferencesGroup::builder()
            .title(pango_escape(&title).as_str())
            .description("")
            .build();
        let rows = rows_cell.borrow();
        for row in rows.iter() {
            if row.group == g {
                let ar = ActionRow::builder()
                    .title(&row.description)
                    .subtitle("")
                    .build();
                ar.add_suffix(&row.entry);
                grp.add(&ar);
            }
        }
        page.add(&grp);
    }

    // Statusmelding.
    let status_label = Label::builder().build();
    status_label.add_css_class("dim-label");

    // Actiegroep met Save / Reset.
    let actions = PreferencesGroup::builder()
        .title(tr.keybinds_edit.as_str())
        .description("")
        .build();

    let saved_save = tr.keybinds_saved.clone();
    let saved_reset = tr.keybinds_saved.clone();

    let btn_save = Button::builder()
        .label(&tr.keybinds_save)
        .valign(gtk4::Align::Center)
        .build();
    btn_save.add_css_class("suggested-action");
    {
        let rows_cell_c = Rc::clone(&rows_cell);
        let status_c = status_label.clone();
        let saved_c = saved_save.clone();
        btn_save.connect_clicked(move |_| {
            let binds = collect_entries(&rows_cell_c);
            let _ = crate::backend::keybinds::write_keybinds(&binds);
            status_c.set_label(&saved_c);
        });
    }

    let btn_reset = Button::builder()
        .label(&tr.keybinds_reset)
        .valign(gtk4::Align::Center)
        .build();
    {
        let rows_cell_c = Rc::clone(&rows_cell);
        let status_c = status_label.clone();
        let saved_c = saved_reset.clone();
        btn_reset.connect_clicked(move |_| {
            let defaults = crate::backend::keybinds::default_keybinds();
            let _ = crate::backend::keybinds::write_keybinds(&defaults);
            repopulate_entries(&rows_cell_c, &defaults);
            status_c.set_label(&saved_c);
        });
    }

    let row_save = ActionRow::builder()
        .title(&tr.keybinds_save)
        .subtitle(&tr.keybinds_new_keys)
        .build();
    row_save.add_suffix(&btn_save);
    actions.add(&row_save);

    let row_reset = ActionRow::builder()
        .title(&tr.keybinds_reset)
        .build();
    row_reset.add_suffix(&btn_reset);
    actions.add(&row_reset);

    let row_status = ActionRow::builder()
        .title("")
        .build();
    row_status.add_suffix(&status_label);
    actions.add(&row_status);

    page.add(&actions);

    page
}
