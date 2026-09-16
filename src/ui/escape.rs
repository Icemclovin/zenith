/// Escape van tekst zodat GTK/Libadwaita het niet als Pango-markup interpreteert.
/// GTK widgets zoals `Label`, `ActionRow` en `PreferencesGroup` parsen hun tekst
/// standaard als Pango markup. Letters als `&`, `<` en `>` moeten daarvoor worden
/// geëscape om weergavefouten ("Failed to set text from markup") te voorkomen.
pub fn pango_escape(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}