// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

// Gedeelde instellingen voor iconen: één FileView voor de hele shell.
Singleton {
    id: zs

    FileView {
        id: cfgFile
        path: Quickshell.env("HOME") + "/.config/quickshell/zenith-shell.json"
        watchChanges: true
        onFileChanged: reload()
    }
    readonly property var cfg: {
        try { var t = cfgFile.text().trim(); if (t.length > 0) return JSON.parse(t); } catch (e) {}
        return {};
    }
    readonly property bool iconAnim: (zs.cfg.animations || {}).icon_anim !== false
    readonly property var custom: zs.cfg.custom_icons || {}

    // Nerd Font-glyphs (Material Design-set). Namen gelijk aan CustomIcons in shell_config.rs,
    // plus enkele vaste iconen die niet aanpasbaar zijn.
    readonly property var defaults: ({
        workspaces: "\u{f0a1d}", active_window: "\u{f0614}", clock: "\u{f0150}", systray: "\u{f0120}",
        volume_high: "\u{f057e}", volume_medium: "\u{f0580}", volume_muted: "\u{f0581}", brightness: "\u{f00df}",
        power: "\u{f0425}", launcher: "\u{f003b}", brand: "\u{f0ae3}", media: "\u{f0387}",
        media_play: "\u{f040a}", media_pause: "\u{f03e4}", media_prev: "\u{f04ae}", media_next: "\u{f04ad}",
        cpu: "\u{f0ee0}", ram: "\u{f035b}", battery_full: "\u{f0079}", battery_low: "\u{f12a1}",
        battery_charging: "\u{f0084}", network: "\u{f05a9}", bluetooth: "\u{f00af}", wifi: "\u{f05a9}",
        dnd_on: "\u{f0a91}", dnd_off: "\u{f009c}", nightlight: "\u{f0594}", mic: "\u{f036c}", mic_muted: "\u{f036d}",
        lock: "\u{f0341}", sleep: "\u{f04b2}", logout: "\u{f0343}", reboot: "\u{f0709}", shutdown: "\u{f0425}",
        settings: "\u{f08bb}", script: "\u{f018d}", custom: "\u{f0674}", close: "\u{f0156}",
        // vast
        wifi_off: "\u{f05aa}", network_off: "\u{f05aa}", search: "\u{f0349}", image: "\u{f0976}", text: "\u{f09ed}",
        trash: "\u{f09e7}", calendar: "\u{f0e18}", clipboard: "\u{f0a38}", headphones: "\u{f02cb}",
        loading: "\u{f0772}", eye: "\u{f06d0}", keyboard: "\u{f097b}", awake: "\u{f06ca}", idle: "\u{f0904}",
        power_saver: "\u{f032a}", balanced: "\u{f05d1}", performance: "\u{f04c5}", arrow: "\u{f0054}",
        disk: "\u{f02ca}", thermometer: "\u{f050f}", sunny: "\u{f0599}", partly_cloudy: "\u{f0595}",
        cloudy: "\u{f0590}", fog: "\u{f0591}", storm: "\u{f067e}", snow: "\u{f0598}", rain: "\u{f0597}"
    })

    function glyph(name) {
        return zs.custom[name] || zs.defaults[name] || "";
    }
}
