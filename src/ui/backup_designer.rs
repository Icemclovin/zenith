use gtk4::prelude::*;
use gtk4::{Button, Entry, ListBox};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::rc::Rc;

use crate::backend::backup;
use crate::backend::i18n::Translations;
use crate::ui::escape::pango_escape;

/// Bouwt de "Back-up & Herstel"-pagina.
///
/// De gebruiker kan:
///   1. een volledige back-up maken van alle Zenith-instellingen,
///   2. bestaande back-ups bekijken (label, tijd, bestandsaantal, grootte),
///   3. een oude back-up terugzetten (restore),
///   4. een back-up definitief verwijderen.
pub fn build_backup_page(tr: &Translations) -> PreferencesPage {
    let page = PreferencesPage::new();

    // Vertalingen die in signal-closures worden gebruikt, eerst klonen (eigenaarschap).
    let restore_ok_msg = tr.bk_restore_ok.clone();
    let deleted_ok_msg = tr.bk_deleted_ok.clone();
    let err_empty_msg = tr.bk_err_empty_label.clone();
    let err_prefix = tr.bk_error_prefix.clone();
    let created_ok_msg = tr.bk_created_ok.clone();
    let done_word = tr.bk_done.clone();
    let none_title_msg = tr.bk_none_title.clone();
    let none_sub_msg = tr.bk_none_sub.clone();
    let files_word = tr.bk_files.clone();
    let restore_btn_msg = tr.bk_restore_btn.clone();
    let delete_tip_msg = tr.bk_delete_tip.clone();

    // ==========================================
    // 1. Nieuwe back-up maken
    // ==========================================
    let group_create = PreferencesGroup::builder()
        .title(pango_escape(&tr.bk_create_title))
        .description(pango_escape(&tr.bk_create_desc))
        .build();

    let entry_label = Entry::builder().text("Back-up").hexpand(false).build();
    let row_label = ActionRow::builder()
        .title(pango_escape(&tr.bk_label_title))
        .subtitle(pango_escape(&tr.bk_label_sub))
        .build();
    row_label.add_suffix(&entry_label);
    group_create.add(&row_label);

    // Statusmelding, onderaan de maak-groep (wordt bij elke actie bijgewerkt).
    let row_status = ActionRow::builder()
        .title(pango_escape(&tr.bk_status_idle))
        .build();
    group_create.add(&row_status);

    let btn_create = Button::builder()
        .label(pango_escape(&tr.bk_create_btn).as_str())
        .valign(gtk4::Align::Center)
        .build();
    let row_create = ActionRow::builder()
        .title(pango_escape(&tr.bk_action_title))
        .subtitle(pango_escape(&tr.bk_action_sub))
        .build();
    row_create.add_suffix(&btn_create);
    group_create.add(&row_create);

    page.add(&group_create);

    // ==========================================
    // 2. Lijst met bestaande back-ups
    // ==========================================
    let group_list = PreferencesGroup::builder()
        .title(pango_escape(&tr.bk_list_title))
        .description(pango_escape(&tr.bk_list_desc))
        .build();

    let list_box = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let list_rc = Rc::new(list_box);
    group_list.add(&*list_rc);
    page.add(&group_list);

    // ==========================================
    // 3. Informatie
    // ==========================================
    let group_info = PreferencesGroup::builder().title(pango_escape(&tr.bk_info_title)).build();
    let home = std::env::var("HOME").unwrap_or_default();
    let backup_dir_owned = std::path::PathBuf::from(&home)
        .join(".local/share/zenith/backups")
        .to_string_lossy()
        .to_string();
    let row_location = ActionRow::builder()
        .title(pango_escape(&tr.bk_location_title))
        .subtitle(format!("{}  •  max. {} {}", backup_dir_owned, backup::MAX_SNAPSHOTS, tr.bk_snapshots))
        .build();
    group_info.add(&row_location);
    page.add(&group_info);

    // ==========================================
    // Kopieën die de refresh-closure nodig heeft (zodat de originelen in leven blijven
    // voor de maak-knop-closure hieronder).
    let restore_ok_r = restore_ok_msg.clone();
    let err_prefix_r = err_prefix.clone();
    let deleted_ok_r = deleted_ok_msg.clone();

    // Refresh-closure: herbouwt de back-uplijst
    // ==========================================
    let refresh_list: Rc<dyn Fn()> = {
        let lb = Rc::clone(&list_rc);
        let rs = row_status.clone();
        Rc::new(move || {
            while let Some(child) = (*lb).first_child() {
                (*lb).remove(&child);
            }

            let backups = backup::list_backups();
            if backups.is_empty() {
                let empty_row = ActionRow::builder()
                    .title(pango_escape(&none_title_msg))
                    .subtitle(pango_escape(&none_sub_msg))
                    .build();
                (*lb).append(&empty_row);
                return;
            }

            for info in backups {
                let timestamp = backup::format_timestamp(info.created_at_unix);
                let size = backup::format_size(info.size_bytes);
                let row_sub = format!(
                    "{}  •  {} {}  •  {}",
                    timestamp, info.file_count, files_word, size
                );

                let row = ActionRow::builder()
                    .title(pango_escape(&info.label))
                    .subtitle(pango_escape(&row_sub))
                    .build();

                // Herstel-knop
                let btn_restore = Button::builder()
                    .label(restore_btn_msg.as_str())
                    .valign(gtk4::Align::Center)
                    .build();
                btn_restore.add_css_class("suggested-action");
                let name_restore = info.name.clone();
                let status_restore = rs.clone();
                let restore_ok = restore_ok_r.clone();
                let err_pref = err_prefix_r.clone();
                btn_restore.connect_clicked(move |_| {
                    match backup::restore_backup(&name_restore) {
                        Ok(()) => status_restore.set_title(&restore_ok),
                        Err(e) => {
                            let msg = format!("{}: {}", err_pref, e);
                            status_restore.set_title(pango_escape(&msg).as_str());
                        }
                    }
                });
                row.add_suffix(&btn_restore);

                // Verwijder-knop
                let btn_del = Button::builder()
                    .label("🗑")
                    .valign(gtk4::Align::Center)
                    .tooltip_text(delete_tip_msg.as_str())
                    .build();
                btn_del.add_css_class("destructive-action");
                let name_del = info.name.clone();
                let status_del = rs.clone();
                let del_ok = deleted_ok_r.clone();
                let row_del = row.clone();
                let lb_del = lb.clone();
                btn_del.connect_clicked(move |_| {
                    let _ = backup::delete_backup(&name_del);
                    status_del.set_title(&del_ok);
                    // Verwijder de getoonde rij direct uit de lijst.
                    (*lb_del).remove(&row_del);
                });
                row.add_suffix(&btn_del);

                (*lb).append(&row);
            }
        })
    };

    // Koppel de maak-knop aan create_backup + verversing.
    {
        let entry_c = entry_label.clone();
        let stat_c = row_status.clone();
        let refresh_c = Rc::clone(&refresh_list);
        let err_empty_c = err_empty_msg.clone();
        let created_c = created_ok_msg.clone();
        let done_c = done_word.clone();
        let err_prefix_c = err_prefix.clone();
        btn_create.connect_clicked(move |_| {
            let text_owned = entry_c.text();
            let label = text_owned.as_str().trim();
            if label.is_empty() {
                stat_c.set_title(&err_empty_c);
                return;
            }
            match backup::create_backup(label) {
                Ok(info) => {
                    let msg = format!("{} «{}» — {}", created_c, info.label, done_c);
                    stat_c.set_title(pango_escape(&msg).as_str());
                    refresh_c();
                }
                Err(e) => {
                    let msg = format!("{}: {}", err_prefix_c, e);
                    stat_c.set_title(pango_escape(&msg).as_str());
                }
            }
        });
    }

    // Eerste vulling.
    refresh_list();

    page
}