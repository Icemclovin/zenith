//! Gedeelde instellingenrijen: elke pagina geeft alleen een `on_change` mee.
use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{ColorDialog, ColorDialogButton, Label, Orientation, Scale};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, SwitchRow};

pub fn rgba_to_hex(c: &RGBA) -> String {
    let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("{:02x}{:02x}{:02x}", ch(c.red()), ch(c.green()), ch(c.blue()))
}

pub fn hex_to_rgba(hex: &str) -> RGBA {
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

/// Schuifrij met de actuele waarde ernaast.
pub fn slider_row(
    title: &str, min: f64, max: f64, step: f64, value: f64,
    fmt: impl Fn(f64) -> String + 'static,
    on_change: impl Fn(f64) + 'static,
) -> ActionRow {
    let row = ActionRow::builder().title(title).build();
    let value_lbl = Label::builder().label(fmt(value)).width_chars(6).xalign(1.0).css_classes(["dim-label", "numeric"]).build();
    let scale = Scale::with_range(Orientation::Horizontal, min, max, step);
    scale.set_draw_value(false);
    scale.set_value(value);
    scale.set_width_request(240);
    scale.update_property(&[gtk4::accessible::Property::Label(title)]);
    let lbl = value_lbl.clone();
    scale.connect_value_changed(move |s| {
        let v = if step >= 1.0 { s.value().round() } else { s.value() };
        lbl.set_label(&fmt(v));
        on_change(v);
    });
    row.add_suffix(&scale);
    row.add_suffix(&value_lbl);
    row
}

pub fn switch_row(title: &str, subtitle: &str, active: bool, on_change: impl Fn(bool) + 'static) -> SwitchRow {
    let row = SwitchRow::builder().title(title).subtitle(subtitle).active(active).build();
    row.connect_active_notify(move |r| on_change(r.is_active()));
    row
}

/// Kleurrij; `on_change` krijgt hex zonder `#`.
pub fn color_row(title: &str, subtitle: &str, hex: &str, on_change: impl Fn(String) + 'static) -> ActionRow {
    let row = ActionRow::builder().title(title).subtitle(subtitle).build();
    let btn = ColorDialogButton::builder()
        .dialog(&ColorDialog::builder().title(title).with_alpha(false).build())
        .valign(gtk4::Align::Center)
        .build();
    btn.set_rgba(&hex_to_rgba(hex));
    btn.connect_notify_local(Some("rgba"), move |b, _| on_change(rgba_to_hex(&b.rgba())));
    row.add_suffix(&btn);
    row.set_activatable_widget(Some(&btn));
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        for hex in ["1e1e2e", "89b4fa", "000000", "ffffff"] {
            assert_eq!(rgba_to_hex(&hex_to_rgba(hex)), hex);
            assert_eq!(rgba_to_hex(&hex_to_rgba(&format!("#{hex}"))), hex);
        }
    }
}
