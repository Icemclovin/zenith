use std::fs::{create_dir_all, read_to_string, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

fn get_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Zorgt dat alle configuratiebestanden en imports aanwezig zijn
pub fn ensure_environment() {
    let home = match get_home() {
        Some(h) => h,
        None => return,
    };

    // 1. Zorg voor Hyprland source-regel
    ensure_hyprland_source(&home);

    // 2. Garandeer placeholder-bestanden (voorkomt crashes bij imports)
    ensure_file_exists(&home.join(".config/waybar/zenith-style.css"), "/* Zenith Waybar Styles */\n");
    ensure_file_exists(&home.join(".config/kitty/zenith-theme.conf"), "# Zenith Kitty Theme\n");
    ensure_file_exists(&home.join(".config/rofi/zenith-theme.rasi"), "/* Zenith Rofi Theme */\n");
    ensure_file_exists(
        &home.join(".config/quickshell/zenith-theme.json"),
        "{\n  \
  \"background\": \"#1e1e2e\",\n  \
  \"accent\": \"#89b4fa\",\n  \
  \"opacity\": 0.90,\n  \
  \"height\": 38,\n  \
  \"position\": \"top\",\n  \
  \"rounding\": 12,\n  \
  \"margin_h\": 8,\n  \
  \"margin_v\": 6,\n  \
  \"border_width\": 1,\n  \
  \"border_color\": \"#45475a\",\n  \
  \"text_color\": \"#cdd6f4\",\n  \
  \"font_size\": 11,\n  \
  \"floating\": true,\n  \
  \"bar_style\": \"unified\",\n  \
  \"pill_bg\": \"#181825\",\n  \
  \"pill_opacity\": 0.85,\n  \
  \"pill_rounding\": 8,\n  \
  \"module_spacing\": 12,\n  \
  \"brand_text\": \"Zenith\",\n  \
  \"show_brand\": true,\n  \
  \"status_text\": \"Hyprland\",\n  \
  \"show_status_badge\": true,\n  \
  \"show_workspaces\": true,\n  \
  \"show_launcher_btn\": true,\n  \
  \"show_cpu\": true,\n  \
  \"show_ram\": true,\n  \
  \"show_battery\": true,\n  \
  \"show_volume\": true,\n  \
  \"show_bluetooth\": true,\n  \
  \"show_network\": true,\n  \
  \"show_media\": true,\n  \
  \"show_clock\": true,\n  \
  \"show_seconds\": false,\n  \
  \"show_power_btn\": true\n\
}\n",
    );

    // 3. Garandeer reactieve Quickshell configuratie (shell.qml)
    ensure_quickshell_config(&home);

    // 4. Garandeer Waybar configuratie met systeemmodules
    ensure_waybar_config(&home);

    // 5. Injecteer automatische imports in bestaande dotfiles als ze bestaan
    ensure_kitty_import(&home);
    ensure_waybar_import(&home);
}

fn ensure_file_exists(path: &Path, default_content: &str) {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }
        if let Ok(mut f) = File::create(path) {
            let _ = f.write_all(default_content.as_bytes());
        }
    }
}

/// Schrijft een Zenith-paneel, of werkt het bij als de inhoud afwijkt en het bestand van Zenith
/// zelf is (markering, of een oudere versie die ZenithPopup gebruikt). Eigen bestanden van de
/// gebruiker blijven ongemoeid; de vorige versie wordt altijd eerst als `.bak` bewaard.
fn ensure_zenith_panel(path: &Path, content: &str) {
    let Ok(current) = read_to_string(path) else {
        return ensure_file_exists(path, content);
    };
    // Oudere Zenith-versies: de panelen gebruiken ZenithPopup, en ZenithPopup.qml zelf heeft deze kopregel.
    let ours = ["zenith-panel", "ZenithPopup", "Gedeeld pop-uppaneel van Zenith",
        // oude Control Center en lockscreen (met onveilige wachtwoord-terugval)
        "Dismiss Overlay: klik buiten paneel sluit Control Center", "Native PAM authenticatie"]
        .iter()
        .any(|m| current.contains(m));
    if current == content || !ours {
        return;
    }
    if std::fs::copy(path, path.with_extension("qml.bak")).is_ok() {
        let _ = std::fs::write(path, content);
    }
}

fn ensure_quickshell_config(home: &Path) {
    let qs_dir = home.join(".config/quickshell");
    let modules_dir = qs_dir.join("modules");
    let cards_dir = qs_dir.join("cards");
    let panels_dir = qs_dir.join("panels");
    let _ = create_dir_all(&modules_dir);
    let _ = create_dir_all(&cards_dir);
    let _ = create_dir_all(&panels_dir);

    // 1. Garandeer zenith-shell.json
    let shell_json_path = qs_dir.join("zenith-shell.json");
    if !shell_json_path.exists() {
        let default_shell_json = r###"{
  "styling": {
    "background": "#1e1e2e",
    "accent": "#89b4fa",
    "opacity": 0.90,
    "rounding": 12,
    "border_width": 1,
    "border_color": "#45475a",
    "text_color": "#cdd6f4",
    "font_size": 11,
    "margin_h": 8,
    "margin_v": 6,
    "module_spacing": 8,
    "pill_bg": "#181825",
    "pill_opacity": 0.85,
    "pill_rounding": 8
  },
  "layout": {
    "position": "top",
    "height": 38,
    "floating": true,
    "bar_style": "unified"
  },
  "modules": {
    "left": ["launcher", "brand", "workspaces"],
    "center": ["active_window", "media"],
    "right": ["cpu", "ram", "battery", "volume", "brightness", "network", "bluetooth", "systray", "clock", "power"]
  },
  "panels": {
    "quick_settings": true,
    "control_center": false,
    "volume_osd": true,
    "brightness_osd": true,
    "notifications": true
  },
  "control_center": {
    "enabled": true,
    "position": "top-right",
    "width": 380,
    "max_height": 600,
    "border_radius": 16,
    "blur_behind": true,
    "opacity": 0.92,
    "cards": ["wifi_toggle", "bluetooth_toggle", "dnd_toggle", "nightlight_toggle", "volume_slider", "mic_slider", "brightness_slider", "mpris_card", "battery_card", "power_strip"]
  },
  "osd": {
    "enabled": true,
    "position": "bottom",
    "timeout_ms": 2000,
    "width": 260,
    "height": 48,
    "orientation": "horizontal",
    "show_percentage": true,
    "show_icon": true,
    "hardware_targets": ["volume", "mic", "brightness"]
  },
  "custom": {
    "raw_override": false,
    "custom_qml_path": null
  },
  "custom_scripts": []
}"###;
        ensure_file_exists(&shell_json_path, default_shell_json);
    } else {
        // Valideer en vul ontbrekende velden aan in bestaande config
        let shell_cfg = crate::backend::shell_config::ZenithShellConfig::load_or_default();
        let _ = shell_cfg.save();
    }

    // 2. Garandeer modulaire shell.qml
    let shell_qml = qs_dir.join("shell.qml");
    let modular_shell_qml = r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Io
import Quickshell.Hyprland
import "panels" as Panels

Scope {
    id: rootScope

    PanelWindow {
        id: root

    // Centrale configuratie inladen via FileView met watchChanges voor instant reactieve updates
    FileView {
        id: shellConfigFile
        path: Quickshell.env("HOME") + "/.config/quickshell/zenith-shell.json"
        watchChanges: true
    }

    // Fallback naar zenith-theme.json indien zenith-shell.json ontbreekt
    FileView {
        id: legacyConfigFile
        path: Quickshell.env("HOME") + "/.config/quickshell/zenith-theme.json"
        watchChanges: true
    }

    readonly property var shellConfig: {
        try {
            var txt = shellConfigFile.text().trim();
            if (txt.length > 0) return JSON.parse(txt);
        } catch (e) {
            console.log("Error parsing zenith-shell.json: " + e);
        }
        try {
            var ltxt = legacyConfigFile.text().trim();
            if (ltxt.length > 0) {
                var leg = JSON.parse(ltxt);
                return {
                    "styling": {
                        "background": leg.background || "#1e1e2e",
                        "accent": leg.accent || "#89b4fa",
                        "opacity": leg.opacity !== undefined ? leg.opacity : 0.90,
                        "rounding": leg.rounding !== undefined ? leg.rounding : 12,
                        "border_width": leg.border_width !== undefined ? leg.border_width : 1,
                        "border_color": leg.border_color || "#45475a",
                        "text_color": leg.text_color || "#cdd6f4",
                        "font_size": leg.font_size || 11,
                        "margin_h": leg.margin_h !== undefined ? leg.margin_h : 8,
                        "margin_v": leg.margin_v !== undefined ? leg.margin_v : 6,
                        "module_spacing": leg.module_spacing !== undefined ? leg.module_spacing : 8,
                        "pill_bg": leg.pill_bg || "#181825",
                        "pill_opacity": leg.pill_opacity !== undefined ? leg.pill_opacity : 0.85,
                        "pill_rounding": leg.pill_rounding !== undefined ? leg.pill_rounding : 8
                    },
                    "layout": {
                        "position": leg.position || "top",
                        "height": leg.height || 38,
                        "floating": leg.floating !== undefined ? leg.floating : true,
                        "bar_style": leg.bar_style || "unified"
                    },
                    "modules": {
                        "left": ["launcher", "brand", "workspaces"],
                        "center": ["active_window", "media"],
                        "right": ["cpu", "ram", "battery", "volume", "brightness", "network", "bluetooth", "systray", "clock", "power"]
                    },
                    "panels": {
                        "quick_settings": true,
                        "control_center": false,
                        "volume_osd": true,
                        "brightness_osd": true,
                        "notifications": true
                    },
                    "custom": { "raw_override": false, "custom_qml_path": null },
                    "custom_scripts": []
                };
            }
        } catch (e2) {}
        return {
            "styling": { "background": "#1e1e2e", "accent": "#89b4fa", "opacity": 0.90, "rounding": 12, "border_width": 1, "border_color": "#45475a", "text_color": "#cdd6f4", "font_size": 11, "margin_h": 8, "margin_v": 6, "module_spacing": 8, "pill_bg": "#181825", "pill_opacity": 0.85, "pill_rounding": 8 },
            "layout": { "position": "top", "height": 38, "floating": true, "bar_style": "unified" },
            "modules": { "left": ["launcher", "brand", "workspaces"], "center": ["active_window", "media"], "right": ["cpu", "ram", "battery", "volume", "brightness", "network", "bluetooth", "systray", "clock", "power"] },
            "panels": { "quick_settings": true, "control_center": false, "volume_osd": true, "brightness_osd": true, "notifications": true },
            "custom": { "raw_override": false, "custom_qml_path": null },
            "custom_scripts": []
        };
    }

    readonly property var styling: root.shellConfig.styling || {}
    readonly property var layout: root.shellConfig.layout || {}
    readonly property var modulesCfg: root.shellConfig.modules || {}
    readonly property var panelsCfg: root.shellConfig.panels || {}
    readonly property var customScripts: root.shellConfig.custom_scripts || []
    readonly property var icons: root.shellConfig.custom_icons || {}

    // Oriëntatie: horizontaal (top/bottom) of verticaal (left/right)
    readonly property bool isVertical: root.layout.position === "left" || root.layout.position === "right"

    // Gedeelde visuele constanten
    readonly property color fgColor: root.styling.text_color || "#cdd6f4"
    readonly property color themeAccent: root.styling.accent || "#89b4fa"
    readonly property color borderColor: root.styling.border_color || "#45475a"
    readonly property int userFontSize: root.styling.font_size || 11
    readonly property bool isIslands: root.layout.bar_style === "islands"
    readonly property int moduleSpacing: root.styling.module_spacing !== undefined ? root.styling.module_spacing : 8

    // Animaties (ingesteld in Zenith → Statusbalk → Animaties)
    readonly property var animCfg: root.shellConfig.animations || {}
    readonly property int animMs: root.animCfg.duration_ms !== undefined ? root.animCfg.duration_ms : 250
    readonly property int animEasing: {
        var t = Easing[root.animCfg.curve || "OutCubic"];
        return t !== undefined ? t : Easing.OutCubic;
    }
    readonly property bool hoverZoom: root.animCfg.hover_zoom !== undefined ? root.animCfg.hover_zoom : true
    readonly property bool fadeIn: root.animCfg.fade_in !== undefined ? root.animCfg.fade_in : true

    component Anim: NumberAnimation { duration: root.animMs; easing.type: root.animEasing }
    component CAnim: ColorAnimation { duration: root.animMs; easing.type: root.animEasing }

    // Positie (top, bottom, left, right)
    anchors {
        top: root.layout.position !== "bottom"
        bottom: root.layout.position !== "top"
        left: root.layout.position !== "right"
        right: root.layout.position !== "left"
    }

    // Zwevende marges
    margins {
        top: root.layout.floating ? (root.styling.margin_v !== undefined ? root.styling.margin_v : 6) : 0
        bottom: root.layout.floating ? (root.styling.margin_v !== undefined ? root.styling.margin_v : 6) : 0
        left: root.layout.floating ? (root.styling.margin_h !== undefined ? root.styling.margin_h : 8) : 0
        right: root.layout.floating ? (root.styling.margin_h !== undefined ? root.styling.margin_h : 8) : 0
    }

    // Afmetingen afhankelijk van oriëntatie
    implicitWidth: root.isVertical ? (root.layout.height || 48) : 0
    implicitHeight: !root.isVertical ? (root.layout.height || 38) : 0
    color: "transparent"

    // Achtergrond container voor 'unified' stijl
    Rectangle {
        id: bgContainer
        visible: !root.isIslands
        anchors.fill: parent
        radius: root.layout.floating ? (root.styling.rounding !== undefined ? root.styling.rounding : 12) : 0
        color: root.layout.bar_style === "minimal"
            ? "transparent"
            : Qt.alpha(root.styling.background || "#1e1e2e", root.styling.opacity !== undefined ? root.styling.opacity : 0.90)
        border.width: root.styling.border_width !== undefined ? root.styling.border_width : 1
        border.color: root.styling.border_color || "#45475a"
        Behavior on color { CAnim {} }
        Behavior on border.color { CAnim {} }
        Behavior on radius { Anim {} }
    }

    // Helper component voor het dynamisch inladen van modules (standaard of script)
    component ModuleLoader: Loader {
        id: mLoader
        required property var modelData

        HoverHandler { id: modHover }
        scale: root.hoverZoom && modHover.hovered ? 1.08 : 1.0
        opacity: (status === Loader.Ready || !root.fadeIn) ? 1.0 : 0.0
        Behavior on scale { Anim {} }
        Behavior on opacity { Anim {} }

        source: {
            if (typeof modelData === "string" && modelData.indexOf("script:") === 0) {
                return Quickshell.env("HOME") + "/.config/quickshell/modules/script_runner.qml";
            }
            return Quickshell.env("HOME") + "/.config/quickshell/modules/" + modelData + ".qml";
        }

        onLoaded: {
            if (typeof modelData === "string" && modelData.indexOf("script:") === 0 && item) {
                var sId = modelData.substring(7);
                for (var i = 0; i < root.customScripts.length; i++) {
                    var s = root.customScripts[i];
                    if (s.id === sId) {
                        item.scriptCommand = s.command || "";
                        item.intervalSec = s.interval_seconds || 10;
                        item.scriptIcon = s.icon || "💻";
                        item.onClickCmd = s.on_click || "";
                        break;
                    }
                }
            }
        }
    }

    // ====== Horizontale Balk Layout (Top of Bottom) ======
    RowLayout {
        visible: !root.isVertical
        anchors.fill: parent
        anchors.leftMargin: root.isIslands ? 0 : 16
        anchors.rightMargin: root.isIslands ? 0 : 16

        // LINKS
        Rectangle {
            Layout.fillHeight: true
            radius: root.isIslands ? (root.styling.pill_rounding !== undefined ? root.styling.pill_rounding : 8) : 0
            color: root.isIslands ? Qt.alpha(root.styling.pill_bg || "#181825", root.styling.pill_opacity !== undefined ? root.styling.pill_opacity : 0.85) : "transparent"
            border.width: root.isIslands ? (root.styling.border_width !== undefined ? root.styling.border_width : 1) : 0
            border.color: root.styling.border_color || "#45475a"
            implicitWidth: leftLayout.implicitWidth + (root.isIslands ? 20 : 0)
            Behavior on color { CAnim {} }
            Behavior on radius { Anim {} }
            Behavior on implicitWidth { Anim {} }

            RowLayout {
                id: leftLayout
                anchors.centerIn: parent
                spacing: root.moduleSpacing

                Repeater {
                    model: root.modulesCfg.left || []
                    ModuleLoader { Layout.alignment: Qt.AlignVCenter }
                }
            }
        }

        Item { Layout.fillWidth: true }

        // MIDDEN
        Rectangle {
            Layout.fillHeight: true
            radius: root.isIslands ? (root.styling.pill_rounding !== undefined ? root.styling.pill_rounding : 8) : 0
            color: root.isIslands ? Qt.alpha(root.styling.pill_bg || "#181825", root.styling.pill_opacity !== undefined ? root.styling.pill_opacity : 0.85) : "transparent"
            border.width: root.isIslands ? (root.styling.border_width !== undefined ? root.styling.border_width : 1) : 0
            border.color: root.styling.border_color || "#45475a"
            implicitWidth: midLayout.implicitWidth + (root.isIslands ? 24 : 0)
            Behavior on color { CAnim {} }
            Behavior on radius { Anim {} }
            Behavior on implicitWidth { Anim {} }

            RowLayout {
                id: midLayout
                anchors.centerIn: parent
                spacing: root.moduleSpacing

                Repeater {
                    model: root.modulesCfg.center || []
                    ModuleLoader { Layout.alignment: Qt.AlignVCenter }
                }
            }
        }

        Item { Layout.fillWidth: true }

        // RECHTS
        Rectangle {
            Layout.fillHeight: true
            radius: root.isIslands ? (root.styling.pill_rounding !== undefined ? root.styling.pill_rounding : 8) : 0
            color: root.isIslands ? Qt.alpha(root.styling.pill_bg || "#181825", root.styling.pill_opacity !== undefined ? root.styling.pill_opacity : 0.85) : "transparent"
            border.width: root.isIslands ? (root.styling.border_width !== undefined ? root.styling.border_width : 1) : 0
            border.color: root.styling.border_color || "#45475a"
            implicitWidth: rightLayout.implicitWidth + (root.isIslands ? 16 : 0)
            Behavior on color { CAnim {} }
            Behavior on radius { Anim {} }
            Behavior on implicitWidth { Anim {} }

            RowLayout {
                id: rightLayout
                anchors.centerIn: parent
                spacing: root.moduleSpacing

                Repeater {
                    model: root.modulesCfg.right || []
                    ModuleLoader { Layout.alignment: Qt.AlignVCenter }
                }
            }
        }
    }

    // ====== Verticale Balk Layout (Left of Right) ======
    ColumnLayout {
        visible: root.isVertical
        anchors.fill: parent
        anchors.topMargin: root.isIslands ? 0 : 16
        anchors.bottomMargin: root.isIslands ? 0 : 16

        // BOVEN
        Rectangle {
            Layout.fillWidth: true
            radius: root.isIslands ? (root.styling.pill_rounding !== undefined ? root.styling.pill_rounding : 8) : 0
            color: root.isIslands ? Qt.alpha(root.styling.pill_bg || "#181825", root.styling.pill_opacity !== undefined ? root.styling.pill_opacity : 0.85) : "transparent"
            border.width: root.isIslands ? (root.styling.border_width !== undefined ? root.styling.border_width : 1) : 0
            border.color: root.styling.border_color || "#45475a"
            implicitHeight: topLayout.implicitHeight + (root.isIslands ? 20 : 0)
            Behavior on color { CAnim {} }
            Behavior on radius { Anim {} }
            Behavior on implicitHeight { Anim {} }

            ColumnLayout {
                id: topLayout
                anchors.centerIn: parent
                spacing: root.moduleSpacing

                Repeater {
                    model: root.modulesCfg.left || []
                    ModuleLoader { Layout.alignment: Qt.AlignHCenter }
                }
            }
        }

        Item { Layout.fillHeight: true }

        // MIDDEN (Verticaal)
        Rectangle {
            Layout.fillWidth: true
            radius: root.isIslands ? (root.styling.pill_rounding !== undefined ? root.styling.pill_rounding : 8) : 0
            color: root.isIslands ? Qt.alpha(root.styling.pill_bg || "#181825", root.styling.pill_opacity !== undefined ? root.styling.pill_opacity : 0.85) : "transparent"
            border.width: root.isIslands ? (root.styling.border_width !== undefined ? root.styling.border_width : 1) : 0
            border.color: root.styling.border_color || "#45475a"
            implicitHeight: vMidLayout.implicitHeight + (root.isIslands ? 24 : 0)
            Behavior on color { CAnim {} }
            Behavior on radius { Anim {} }
            Behavior on implicitHeight { Anim {} }

            ColumnLayout {
                id: vMidLayout
                anchors.centerIn: parent
                spacing: root.moduleSpacing

                Repeater {
                    model: root.modulesCfg.center || []
                    ModuleLoader { Layout.alignment: Qt.AlignHCenter }
                }
            }
        }

        Item { Layout.fillHeight: true }

        // ONDER
        Rectangle {
            Layout.fillWidth: true
            radius: root.isIslands ? (root.styling.pill_rounding !== undefined ? root.styling.pill_rounding : 8) : 0
            color: root.isIslands ? Qt.alpha(root.styling.pill_bg || "#181825", root.styling.pill_opacity !== undefined ? root.styling.pill_opacity : 0.85) : "transparent"
            border.width: root.isIslands ? (root.styling.border_width !== undefined ? root.styling.border_width : 1) : 0
            border.color: root.styling.border_color || "#45475a"
            implicitHeight: bottomLayout.implicitHeight + (root.isIslands ? 16 : 0)
            Behavior on color { CAnim {} }
            Behavior on radius { Anim {} }
            Behavior on implicitHeight { Anim {} }

            ColumnLayout {
                id: bottomLayout
                anchors.centerIn: parent
                spacing: root.moduleSpacing

                Repeater {
                    model: root.modulesCfg.right || []
                    ModuleLoader { Layout.alignment: Qt.AlignHCenter }
                }
            }
        }
        }
    }

    // ==========================================
    // 2. Control Center Paneel
    // ==========================================
    Panels.ControlCenter {
        id: controlCenterPanel
    }

    // ==========================================
    // 3. On-Screen Display (OSD)
    // ==========================================
    Panels.Osd {
        id: osdPanel
    }

    // ==========================================
    // 4. Pop-uppanelen: klembord, kalender, media (via IPC te openen)
    // ==========================================
    Panels.Clipboard {}
    Panels.Calendar {}
    Panels.Media {}
    Panels.Launcher {}
    Panels.Notifications {}
    Panels.Lockscreen {}
}
"###;
    if shell_qml.exists() {
        if let Ok(content) = read_to_string(&shell_qml) {
            // Als het bestaande bestand verouderd is (mist ModuleLoader, FileView, Scope of Panels)
            if !content.contains("ModuleLoader") || !content.contains("FileView") || !content.contains("Panels.ControlCenter") || !content.contains("Scope") || !content.contains("animEasing") || !content.contains("Panels.Lockscreen") {
                // Eigen aanpassingen nooit kwijtraken: unieke back-up, en alleen overschrijven als die gelukt is.
                let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                let backup = qs_dir.join(format!("shell.qml.bak-{}", stamp));
                if std::fs::copy(&shell_qml, &backup).is_ok() {
                    let _ = std::fs::write(&shell_qml, modular_shell_qml);
                }
            }
        }
    } else {
        ensure_file_exists(&shell_qml, modular_shell_qml);
    }

    // 3. Garandeer standaard modules in ~/.config/quickshell/modules/
    ensure_default_modules(&modules_dir);

    // 4. Garandeer Control Center & OSD panels in ~/.config/quickshell/panels/
    ensure_panels(&panels_dir);

    // 5. Garandeer dynamische kaarten in ~/.config/quickshell/cards/
    ensure_default_cards(&cards_dir);
}

fn ensure_default_modules(dir: &Path) {
    ensure_file_exists(
        &dir.join("workspaces.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Hyprland

RowLayout {
    id: modWorkspaces
    spacing: 4

    Repeater {
        model: Hyprland.workspaces

        Rectangle {
            id: wsPill
            required property var modelData
            property bool isActive: (Hyprland.focusedMonitor && Hyprland.focusedMonitor.activeWorkspace)
                ? modelData.id === Hyprland.focusedMonitor.activeWorkspace.id
                : false

            width: isActive ? 32 : (wsHover.containsMouse ? 28 : 24)
            height: 22
            radius: 6

            color: isActive
                ? Qt.alpha(root.themeAccent || "#89b4fa", 0.30)
                : (wsHover.containsMouse ? Qt.alpha(root.fgColor || "#cdd6f4", 0.12) : Qt.alpha(root.fgColor || "#cdd6f4", 0.05))

            border.color: isActive
                ? (root.themeAccent || "#89b4fa")
                : (wsHover.containsMouse ? Qt.alpha(root.themeAccent || "#89b4fa", 0.5) : (root.borderColor || "#45475a"))
            border.width: 1

            scale: wsHover.containsMouse ? 1.08 : 1.0

            Behavior on width {
                NumberAnimation { duration: 160; easing.type: Easing.OutCubic }
            }
            Behavior on scale {
                NumberAnimation { duration: 140; easing.type: Easing.OutBack; easing.overshoot: 1.2 }
            }
            Behavior on color {
                ColorAnimation { duration: 140 }
            }
            Behavior on border.color {
                ColorAnimation { duration: 140 }
            }

            Text {
                anchors.centerIn: parent
                text: modelData.name || modelData.id
                color: isActive ? (root.themeAccent || "#89b4fa") : (root.fgColor || "#cdd6f4")
                font.pixelSize: Math.max(9, (root.userFontSize || 11) - 1)
                font.bold: isActive
            }

            MouseArea {
                id: wsHover
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: Hyprland.dispatch("workspace " + modelData.id)
            }
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("active_window.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: activeWinMod
    implicitWidth: activeWinText.implicitWidth
    implicitHeight: 22

    property string winTitle: ""

    Process {
        id: winProc
        command: ["bash", "-c", "hyprctl activewindow -j 2>/dev/null | grep -o '\"title\": *\"[^\"]*' | cut -d'\"' -f4"]
        running: true
        onExited: winTimer.start()
        stdout: SplitParser {
            onRead: data => {
                var s = data.trim();
                activeWinMod.winTitle = s.length > 35 ? s.substring(0, 32) + "..." : s;
            }
        }
    }
    Timer { id: winTimer; interval: 1000; onTriggered: winProc.running = true }

    Text {
        id: activeWinText
        anchors.centerIn: parent
        visible: activeWinMod.winTitle.length > 0
        text: "🪟 " + activeWinMod.winTitle
        color: root.fgColor || "#cdd6f4"
        font.pixelSize: root.userFontSize || 11
        font.bold: true
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("clock.qml"),
        r###"import QtQuick
import QtQuick.Layouts

Item {
    id: clockMod
    implicitWidth: clockText.implicitWidth
    implicitHeight: 22

    Text {
        id: clockText
        anchors.centerIn: parent
        color: root.fgColor || "#cdd6f4"
        font.pixelSize: (root.userFontSize || 11) + 1
        font.bold: true

        Timer {
            interval: 1000
            running: true
            repeat: true
            triggeredOnStart: true
            onTriggered: {
                clockText.text = Qt.formatDateTime(new Date(), "ddd d MMM  HH:mm");
            }
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("systray.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Services.SystemTray
import Quickshell.Widgets

RowLayout {
    id: trayMod
    spacing: 6

    Repeater {
        model: SystemTray.items

        Rectangle {
            required property var modelData
            width: 22
            height: 22
            radius: 4
            color: "transparent"

            IconImage {
                anchors.centerIn: parent
                width: 16
                height: 16
                source: modelData.icon
            }

            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                cursorShape: Qt.PointingHandCursor
                onClicked: mouse => {
                    if (mouse.button === Qt.RightButton) {
                        modelData.openMenu();
                    } else {
                        modelData.activate();
                    }
                }
            }
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("volume.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: volMod
    implicitWidth: volText.implicitWidth
    implicitHeight: 22

    property string volumeLevel: "??%"
    property bool volumeMuted: false

    Process {
        id: volProc
        command: ["wpctl", "get-volume", "@DEFAULT_AUDIO_SINK@"]
        running: true
        onExited: volTimer.start()
        stdout: SplitParser {
            onRead: data => {
                var match = data.match(/Volume:\s*([\d.]+)/);
                if (match) {
                    volMod.volumeLevel = Math.round(parseFloat(match[1]) * 100) + "%";
                    volMod.volumeMuted = data.indexOf("[MUTED]") !== -1;
                }
            }
        }
    }
    Timer { id: volTimer; interval: 2000; onTriggered: volProc.running = true }

    Process { id: muteProc; command: ["wpctl", "set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"] }
    Process { id: upProc; command: ["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"] }
    Process { id: downProc; command: ["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"] }

    Text {
        id: volText
        anchors.centerIn: parent
        text: (volMod.volumeMuted ? "🔇 " : "🔊 ") + volMod.volumeLevel
        color: volMod.volumeMuted ? "#6c7086" : (root.fgColor || "#cdd6f4")
        font.pixelSize: root.userFontSize || 11
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: { muteProc.running = true; volProc.running = true; }
        onWheel: wheel => {
            if (wheel.angleDelta.y > 0) upProc.running = true;
            else downProc.running = true;
            volProc.running = true;
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("brightness.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: brightMod
    implicitWidth: brightText.implicitWidth
    implicitHeight: 22

    property string brightLevel: "??%"

    Process {
        id: brightProc
        command: ["bash", "-c", "brightnessctl -m 2>/dev/null | cut -d, -f4"]
        running: true
        onExited: brightTimer.start()
        stdout: SplitParser {
            onRead: data => {
                var s = data.trim();
                if (s.length > 0) brightMod.brightLevel = s;
            }
        }
    }
    Timer { id: brightTimer; interval: 3000; onTriggered: brightProc.running = true }

    Process { id: upProc; command: ["brightnessctl", "set", "+5%"] }
    Process { id: downProc; command: ["brightnessctl", "set", "5%-"] }

    Text {
        id: brightText
        anchors.centerIn: parent
        text: "☀️ " + brightMod.brightLevel
        color: root.fgColor || "#cdd6f4"
        font.pixelSize: root.userFontSize || 11
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onWheel: wheel => {
            if (wheel.angleDelta.y > 0) upProc.running = true;
            else downProc.running = true;
            brightProc.running = true;
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("power.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: powerMod
    width: 24; height: 22; radius: 6
    color: Qt.alpha("#f38ba8", 0.20)
    border.color: "#f38ba8"
    border.width: 1

    Process { id: powerProc; command: ["bash", "-c", "wlogout || rofi -show power-menu || hyprctl dispatch exit"] }

    Text {
        anchors.centerIn: parent
        text: "⏻"
        color: "#f38ba8"
        font.pixelSize: 11
        font.bold: true
    }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: powerProc.running = true
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("launcher.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: launcherMod
    width: 24; height: 22; radius: 6
    color: Qt.alpha(root.themeAccent || "#89b4fa", 0.20)
    border.color: root.themeAccent || "#89b4fa"
    border.width: 1

    Process { id: launcherProc; command: ["bash", "-c", "rofi -show drun || wofi --show drun"] }

    Text {
        anchors.centerIn: parent
        text: "🚀"
        font.pixelSize: 11
    }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: launcherProc.running = true
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("brand.qml"),
        r###"import QtQuick
import QtQuick.Layouts

RowLayout {
    id: brandMod
    spacing: 6

    Rectangle {
        width: 8; height: 8; radius: 4
        color: root.themeAccent || "#89b4fa"
    }
    Text {
        text: "Zenith"
        color: root.fgColor || "#cdd6f4"
        font.pixelSize: (root.userFontSize || 11) + 1
        font.bold: true
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("media.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: mediaMod
    implicitWidth: mediaLayout.implicitWidth
    implicitHeight: 22
    visible: mediaPlaying

    property bool mediaPlaying: false
    property string mediaTitle: ""

    Process {
        id: mediaProc
        command: ["playerctl", "metadata", "--format", "{{artist}} - {{title}}"]
        running: true
        onExited: mediaTimer.start()
        stdout: SplitParser {
            onRead: data => {
                var s = data.trim();
                mediaMod.mediaPlaying = s.length > 0;
                mediaMod.mediaTitle = s.length > 25 ? (s.substring(0, 22) + "...") : s;
            }
        }
    }
    Timer { id: mediaTimer; interval: 2000; onTriggered: mediaProc.running = true }

    Process { id: mediaToggleProc; command: ["playerctl", "play-pause"] }

    RowLayout {
        id: mediaLayout
        anchors.centerIn: parent
        spacing: 4

        Text {
            text: "🎵 " + mediaMod.mediaTitle
            color: root.themeAccent || "#89b4fa"
            font.pixelSize: root.userFontSize || 11
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            mediaToggleProc.running = true;
            mediaProc.running = true;
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("cpu.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: cpuMod
    implicitWidth: cpuText.implicitWidth
    implicitHeight: 22

    property string cpuLoad: "0.00"

    Process {
        id: cpuProc
        command: ["cat", "/proc/loadavg"]
        running: true
        onExited: cpuTimer.start()
        stdout: SplitParser {
            onRead: data => { cpuMod.cpuLoad = data.split(" ")[0] || "0.00"; }
        }
    }
    Timer { id: cpuTimer; interval: 3000; onTriggered: cpuProc.running = true }

    Text {
        id: cpuText
        anchors.centerIn: parent
        text: "🖥 " + cpuMod.cpuLoad
        color: root.fgColor || "#cdd6f4"
        font.pixelSize: root.userFontSize || 11
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("ram.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: ramMod
    implicitWidth: ramText.implicitWidth
    implicitHeight: 22

    property string ramUsage: "0.0G"

    Process {
        id: ramProc
        command: ["bash", "-c", "awk '/MemTotal/{t=$2} /MemAvailable/{a=$2} END{printf \"%.1fG (%.0f%%)\", (t-a)/1048576, (t-a)/t*100}' /proc/meminfo"]
        running: true
        onExited: ramTimer.start()
        stdout: SplitParser {
            onRead: data => { if (data.trim().length > 0) ramMod.ramUsage = data.trim(); }
        }
    }
    Timer { id: ramTimer; interval: 3000; onTriggered: ramProc.running = true }

    Text {
        id: ramText
        anchors.centerIn: parent
        text: "💾 " + ramMod.ramUsage
        color: root.fgColor || "#cdd6f4"
        font.pixelSize: root.userFontSize || 11
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("battery.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: batMod
    implicitWidth: batText.implicitWidth
    implicitHeight: 22

    FileView {
        id: batteryFile
        path: "/sys/class/power_supply/BAT0/capacity"
        watchChanges: true
    }
    FileView {
        id: batteryStatusFile
        path: "/sys/class/power_supply/BAT0/status"
        watchChanges: true
    }

    readonly property string batteryLevel: {
        var raw = batteryFile.text().trim();
        return raw.length > 0 ? raw : "??";
    }
    readonly property string batteryStatus: {
        var raw = batteryStatusFile.text().trim();
        return raw.length > 0 ? raw : "";
    }
    readonly property string batteryIcon: {
        var level = parseInt(batteryLevel);
        if (batteryStatus === "Charging") return "⚡";
        if (isNaN(level)) return "🔋";
        if (level > 40) return "🔋";
        return "🪫";
    }

    Text {
        id: batText
        anchors.centerIn: parent
        text: batMod.batteryIcon + " " + batMod.batteryLevel + "%"
        color: {
            var lvl = parseInt(batMod.batteryLevel);
            if (isNaN(lvl)) return root.fgColor || "#cdd6f4";
            if (lvl <= 15) return "#f38ba8";
            if (lvl <= 40) return "#fab387";
            return root.fgColor || "#cdd6f4";
        }
        font.pixelSize: root.userFontSize || 11
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("network.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: netMod
    implicitWidth: netText.implicitWidth
    implicitHeight: 22

    property string netStatus: "unknown"
    property string netName: ""

    Process {
        id: netProc
        command: ["nmcli", "-t", "-f", "TYPE,STATE,CONNECTION", "d"]
        running: true
        onExited: netTimer.start()
        stdout: SplitParser {
            onRead: data => {
                var line = data.trim();
                if (line.indexOf("wifi:connected:") !== -1) {
                    netMod.netStatus = "connected";
                    netMod.netName = line.split(":")[2] || "WiFi";
                } else if (line.indexOf("ethernet:connected:") !== -1) {
                    netMod.netStatus = "connected";
                    netMod.netName = "Ethernet";
                } else if (netMod.netStatus !== "connected") {
                    netMod.netStatus = "disconnected";
                    netMod.netName = "";
                }
            }
        }
    }
    Timer { id: netTimer; interval: 5000; onTriggered: { netMod.netStatus = "unknown"; netProc.running = true; } }

    Text {
        id: netText
        anchors.centerIn: parent
        text: netMod.netStatus === "connected" ? ("🌐 " + netMod.netName) : "🌐 –"
        color: netMod.netStatus === "connected" ? (root.fgColor || "#cdd6f4") : "#6c7086"
        font.pixelSize: root.userFontSize || 11
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("bluetooth.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: btMod
    implicitWidth: btText.implicitWidth
    implicitHeight: 22

    property bool btPowered: false

    Process {
        id: btProc
        command: ["bluetoothctl", "show"]
        running: true
        onExited: btTimer.start()
        stdout: SplitParser {
            onRead: data => {
                if (data.indexOf("Powered: yes") !== -1) btMod.btPowered = true;
                else if (data.indexOf("Powered: no") !== -1) btMod.btPowered = false;
            }
        }
    }
    Timer { id: btTimer; interval: 5000; onTriggered: btProc.running = true }

    Process { id: btToggleProc; command: ["bluetoothctl", "power", "toggle"] }

    Text {
        id: btText
        anchors.centerIn: parent
        text: "ᛒ " + (btMod.btPowered ? "aan" : "uit")
        color: btMod.btPowered ? (root.themeAccent || "#89b4fa") : "#6c7086"
        font.pixelSize: root.userFontSize || 11
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: { btToggleProc.running = true; btProc.running = true; }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("CustomWidget.qml.example"),
        r###"import QtQuick
import QtQuick.Layouts

Rectangle {
    id: customWidget
    implicitWidth: customText.implicitWidth + 16
    implicitHeight: 22
    radius: 6
    color: Qt.alpha(root.themeAccent || "#89b4fa", 0.15)
    border.color: root.themeAccent || "#89b4fa"
    border.width: 1

    Text {
        id: customText
        anchors.centerIn: parent
        text: "✨ Hello from Custom QML!"
        color: root.fgColor || "#cdd6f4"
        font.pixelSize: root.userFontSize || 11
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            console.log("Custom widget clicked!");
        }
    }
}
"###,
    );
    ensure_file_exists(
        &dir.join("script_runner.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: scriptMod
    implicitWidth: rowContent.implicitWidth + 8
    implicitHeight: 22

    property string scriptCommand: ""
    property int intervalSec: 10
    property string scriptIcon: "💻"
    property string onClickCmd: ""
    property string outputText: ""

    Process {
        id: proc
        command: ["bash", "-c", scriptMod.scriptCommand]
        running: scriptMod.scriptCommand.length > 0
        onExited: timer.start()
        stdout: SplitParser {
            onRead: data => {
                scriptMod.outputText = data.trim();
            }
        }
    }

    Timer {
        id: timer
        interval: Math.max(1, scriptMod.intervalSec) * 1000
        onTriggered: {
            if (scriptMod.scriptCommand.length > 0) {
                proc.running = true;
            }
        }
    }

    Process {
        id: clickProc
        command: ["bash", "-c", scriptMod.onClickCmd]
    }

    RowLayout {
        id: rowContent
        anchors.centerIn: parent
        spacing: 4

        Text {
            visible: scriptMod.scriptIcon.length > 0
            text: scriptMod.scriptIcon
            font.pixelSize: root.userFontSize || 11
        }

        Text {
            text: scriptMod.outputText.length > 0 ? scriptMod.outputText : "..."
            color: root.fgColor || "#cdd6f4"
            font.pixelSize: root.userFontSize || 11
            elide: Text.ElideRight
            maximumLineCount: 1
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: scriptMod.onClickCmd.length > 0 ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: {
            if (scriptMod.onClickCmd.length > 0) {
                clickProc.running = true;
            }
            if (scriptMod.scriptCommand.length > 0) {
                proc.running = true;
            }
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("quick_settings.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: quickSettingsMod
    implicitWidth: rowBadges.implicitWidth + 14
    implicitHeight: 22
    radius: 6
    color: Qt.alpha(root.themeAccent || "#89b4fa", 0.18)
    border.color: root.themeAccent || "#89b4fa"
    border.width: 1

    RowLayout {
        id: rowBadges
        anchors.centerIn: parent
        spacing: 6

        Text {
            text: "⚙️"
            font.pixelSize: 11
        }
        Text {
            text: "Quick Settings"
            color: root.fgColor || "#cdd6f4"
            font.pixelSize: root.userFontSize || 11
            font.bold: true
        }
    }

    Process { id: qsLauncherProc; command: ["bash", "-c", "quickshell ipc call controlCenter toggle 2>/dev/null || rofi -show drun || wofi --show drun"] }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            qsLauncherProc.running = true;
        }
    }
}
"###,
    );
}


fn ensure_panels(dir: &Path) {
    // Pop-uppanelen delen ZenithPopup.qml; ze worden bijgewerkt als Zenith een nieuwere versie heeft.
    ensure_zenith_panel(
        &dir.join("ZenithPopup.qml"),
        r###"// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Io

// Gedeeld pop-uppaneel van Zenith: sluit bij klik ernaast of Escape, animeert met de
// instellingen uit Zenith → Statusbalk → Animaties en volgt de kleuren van de balk.
Scope {
    id: pop

    property bool active: false
    property string title: ""
    property string icon: ""
    // "top" (midden), "top-left" of "top-right"
    property string position: "top"
    property int panelWidth: 380
    property int panelHeight: 440
    property bool wantsKeyboard: false
    default property alias content: body.data

    FileView {
        id: cfgFile
        path: Quickshell.env("HOME") + "/.config/quickshell/zenith-shell.json"
        watchChanges: true
    }
    readonly property var cfg: {
        try {
            var t = cfgFile.text().trim();
            if (t.length > 0) return JSON.parse(t);
        } catch (e) {}
        return {};
    }
    readonly property var st: pop.cfg.styling || {}
    readonly property var an: pop.cfg.animations || {}
    readonly property color bg: pop.st.background || "#1e1e2e"
    readonly property color surface: pop.st.pill_bg || "#181825"
    readonly property color fg: pop.st.text_color || "#cdd6f4"
    readonly property color accent: pop.st.accent || "#89b4fa"
    readonly property color border: pop.st.border_color || "#45475a"
    readonly property int fontSize: pop.st.font_size || 11
    readonly property int ms: pop.an.duration_ms !== undefined ? pop.an.duration_ms : 250
    // Zelfde taal als de Zenith-app (nl/en/de/es)
    readonly property string lang: pop.cfg.language || "nl"
    readonly property var locale: Qt.locale(({ nl: "nl_NL", en: "en_GB", de: "de_DE", es: "es_ES" })[pop.lang] || "nl_NL")
    readonly property var dict: {
        "nl": {
            "apps": "Apps",
            "searchapps": "Zoek een app…",
            "noapps": "Geen apps gevonden",
            "launchhint": "↑↓ kiezen · Enter openen · Esc sluiten",
            "notifications": "Meldingen",
            "nonotif": "Geen meldingen",
            "dnd": "Niet storen",
            "clipboard": "Klembord",
            "search": "Zoek in klembord…",
            "empty": "Klembord is leeg",
            "noresults": "Geen resultaten",
            "hint": "↑↓ kiezen · Enter kopiëren · Esc sluiten",
            "wipe": "Alles wissen",
            "wipeconfirm": "Klik nogmaals om alles te wissen",
            "image": "Afbeelding",
            "calendar": "Kalender",
            "media": "Media",
            "untitled": "Onbekende titel",
            "nothing": "Er speelt nu niets"
        },
        "en": {
            "apps": "Apps",
            "searchapps": "Search apps…",
            "noapps": "No apps found",
            "launchhint": "↑↓ select · Enter open · Esc close",
            "notifications": "Notifications",
            "nonotif": "No notifications",
            "dnd": "Do not disturb",
            "clipboard": "Clipboard",
            "search": "Search clipboard…",
            "empty": "Clipboard is empty",
            "noresults": "No results",
            "hint": "↑↓ select · Enter copy · Esc close",
            "wipe": "Clear all",
            "wipeconfirm": "Click again to clear everything",
            "image": "Image",
            "calendar": "Calendar",
            "media": "Media",
            "untitled": "Unknown title",
            "nothing": "Nothing is playing"
        },
        "de": {
            "apps": "Apps",
            "searchapps": "App suchen…",
            "noapps": "Keine Apps gefunden",
            "launchhint": "↑↓ wählen · Enter öffnen · Esc schließen",
            "notifications": "Benachrichtigungen",
            "nonotif": "Keine Benachrichtigungen",
            "dnd": "Nicht stören",
            "clipboard": "Zwischenablage",
            "search": "Zwischenablage durchsuchen…",
            "empty": "Zwischenablage ist leer",
            "noresults": "Keine Ergebnisse",
            "hint": "↑↓ wählen · Enter kopieren · Esc schließen",
            "wipe": "Alles löschen",
            "wipeconfirm": "Erneut klicken, um alles zu löschen",
            "image": "Bild",
            "calendar": "Kalender",
            "media": "Medien",
            "untitled": "Unbekannter Titel",
            "nothing": "Es wird nichts abgespielt"
        },
        "es": {
            "apps": "Aplicaciones",
            "searchapps": "Buscar aplicación…",
            "noapps": "No se encontraron aplicaciones",
            "launchhint": "↑↓ elegir · Enter abrir · Esc cerrar",
            "notifications": "Notificaciones",
            "nonotif": "Sin notificaciones",
            "dnd": "No molestar",
            "clipboard": "Portapapeles",
            "search": "Buscar en el portapapeles…",
            "empty": "El portapapeles está vacío",
            "noresults": "Sin resultados",
            "hint": "↑↓ elegir · Enter copiar · Esc cerrar",
            "wipe": "Borrar todo",
            "wipeconfirm": "Haz clic otra vez para borrar todo",
            "image": "Imagen",
            "calendar": "Calendario",
            "media": "Multimedia",
            "untitled": "Título desconocido",
            "nothing": "No se está reproduciendo nada"
        }
    }
    function tr(key) {
        var d = pop.dict[pop.lang] || pop.dict.nl;
        return d[key] || pop.dict.nl[key] || key;
    }
    readonly property int easing: {
        var e = Easing[pop.an.curve || "OutCubic"];
        return e !== undefined ? e : Easing.OutCubic;
    }

    // Klik buiten het paneel sluit het
    PanelWindow {
        visible: pop.active
        color: "transparent"
        anchors { top: true; bottom: true; left: true; right: true }
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        MouseArea {
            anchors.fill: parent
            onClicked: pop.active = false
        }
    }

    PanelWindow {
        visible: pop.active || card.opacity > 0.01
        color: "transparent"
        implicitWidth: pop.panelWidth
        implicitHeight: pop.panelHeight
        anchors {
            top: true
            left: pop.position === "top-left"
            right: pop.position === "top-right"
        }
        margins { top: 52; left: 12; right: 12 }
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: pop.active && pop.wantsKeyboard ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

        Rectangle {
            id: card
            anchors.fill: parent
            radius: 20
            color: Qt.alpha(pop.bg, 0.95)
            border.width: 1
            border.color: Qt.alpha(pop.border, 0.8)
            clip: true

            opacity: pop.active ? 1.0 : 0.0
            scale: pop.active ? 1.0 : 0.94
            transformOrigin: Item.Top
            transform: Translate {
                y: pop.active ? 0 : -14
                Behavior on y { NumberAnimation { duration: pop.ms; easing.type: pop.easing } }
            }
            Behavior on opacity { NumberAnimation { duration: Math.max(120, pop.ms * 0.8); easing.type: Easing.OutCubic } }
            Behavior on scale { NumberAnimation { duration: pop.ms; easing.type: pop.easing } }

            Keys.onEscapePressed: pop.active = false
            focus: pop.active

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 12

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    Text {
                        visible: pop.icon.length > 0
                        text: pop.icon
                        font.pixelSize: pop.fontSize + 5
                    }
                    Text {
                        text: pop.title
                        color: pop.fg
                        font.pixelSize: pop.fontSize + 4
                        font.bold: true
                    }
                    Item { Layout.fillWidth: true }
                    Rectangle {
                        implicitWidth: 26
                        implicitHeight: 26
                        radius: 13
                        color: closeArea.containsMouse ? Qt.alpha(pop.fg, 0.18) : Qt.alpha(pop.fg, 0.08)
                        Behavior on color { ColorAnimation { duration: 120 } }
                        Text {
                            anchors.centerIn: parent
                            text: "✕"
                            color: pop.fg
                            font.pixelSize: pop.fontSize
                        }
                        MouseArea {
                            id: closeArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: pop.active = false
                        }
                    }
                }

                Item {
                    id: body
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                }
            }
        }
    }
}
"###,
    );
    ensure_zenith_panel(
        &dir.join("Clipboard.qml"),
        r###"// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

// Klembord-geschiedenis via cliphist: zoeken, ↑/↓ + Enter of klik om te kopiëren.
Scope {
    id: clip

    property var entries: []
    property string query: ""
    // Wissen vraagt een tweede klik binnen 3 s: de geschiedenis is niet terug te halen.
    property bool confirmWipe: false
    Timer { id: wipeReset; interval: 3000; onTriggered: clip.confirmWipe = false }
    readonly property var filtered: {
        var q = clip.query.toLowerCase();
        var out = [];
        for (var i = 0; i < clip.entries.length && out.length < 200; i++) {
            var e = clip.entries[i];
            if (q.length === 0 || e.text.toLowerCase().indexOf(q) !== -1) out.push(e);
        }
        return out;
    }

    IpcHandler {
        target: "clipboard"
        function toggle() { popup.active ? clip.close() : clip.open(); }
        function open() { clip.open(); }
        function close() { clip.close(); }
    }

    function open() {
        clip.query = "";
        search.text = "";
        listProc.running = true;
        popup.active = true;
        search.forceActiveFocus();
    }
    function close() { popup.active = false; }

    function parse(text) {
        var out = [];
        var lines = text.split("\n");
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i];
            if (line.length === 0) continue;
            var tab = line.indexOf("\t");
            var body = tab >= 0 ? line.substring(tab + 1) : line;
            var isImage = body.indexOf("[[ binary data") === 0;
            out.push({ raw: line, text: isImage ? popup.tr("image") + " · " + body.replace("[[ binary data ", "").replace(" ]]", "") : body.trim(), image: isImage });
        }
        clip.entries = out;
        list.currentIndex = 0;
    }

    function copy(entry) {
        if (!entry) return;
        // De regel gaat als argument mee, nooit als shell-code.
        copyProc.command = ["bash", "-c", "printf '%s' \"$1\" | cliphist decode | wl-copy", "_", entry.raw];
        copyProc.running = true;
        clip.close();
    }

    function remove(entry) {
        deleteProc.command = ["bash", "-c", "printf '%s' \"$1\" | cliphist delete", "_", entry.raw];
        deleteProc.running = true;
    }

    Process {
        id: listProc
        command: ["cliphist", "list"]
        stdout: StdioCollector { onStreamFinished: clip.parse(this.text) }
    }
    Process { id: copyProc }
    Process {
        id: deleteProc
        onExited: listProc.running = true
    }
    Process {
        id: wipeProc
        command: ["cliphist", "wipe"]
        onExited: listProc.running = true
    }

    ZenithPopup {
        id: popup
        title: popup.tr("clipboard")
        icon: "📋"
        position: "top"
        panelWidth: 460
        panelHeight: 520
        wantsKeyboard: true

        ColumnLayout {
            anchors.fill: parent
            spacing: 10

            // Zoekveld
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 38
                radius: 12
                color: Qt.alpha(popup.fg, 0.06)
                border.width: search.activeFocus ? 1 : 0
                border.color: popup.accent

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 12
                    spacing: 8
                    Text { text: "🔍"; font.pixelSize: popup.fontSize + 1 }
                    TextInput {
                        id: search
                        Layout.fillWidth: true
                        color: popup.fg
                        font.pixelSize: popup.fontSize + 2
                        clip: true
                        onTextChanged: { clip.query = text; list.currentIndex = 0; }
                        Keys.onEscapePressed: clip.close()
                        Keys.onDownPressed: list.incrementCurrentIndex()
                        Keys.onUpPressed: list.decrementCurrentIndex()
                        Keys.onReturnPressed: clip.copy(clip.filtered[list.currentIndex])
                        Text {
                            visible: search.text.length === 0
                            text: popup.tr("search")
                            color: Qt.alpha(popup.fg, 0.4)
                            font.pixelSize: popup.fontSize + 2
                        }
                    }
                    Text {
                        text: clip.filtered.length + " / " + clip.entries.length
                        color: Qt.alpha(popup.fg, 0.45)
                        font.pixelSize: popup.fontSize
                    }
                }
            }

            ListView {
                id: list
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 4
                model: clip.filtered
                highlightMoveDuration: popup.ms
                keyNavigationEnabled: true

                delegate: Rectangle {
                    id: row
                    required property var modelData
                    required property int index
                    width: list.width
                    implicitHeight: 44
                    radius: 10
                    readonly property bool current: ListView.isCurrentItem
                    color: current ? Qt.alpha(popup.accent, 0.18) : (hover.containsMouse ? Qt.alpha(popup.fg, 0.07) : "transparent")
                    Behavior on color { ColorAnimation { duration: 120 } }

                    // Zachte entree per regel
                    opacity: 0
                    Component.onCompleted: opacity = 1
                    Behavior on opacity { NumberAnimation { duration: popup.ms } }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 8
                        spacing: 10
                        Text {
                            text: row.modelData.image ? "🖼️" : "📝"
                            font.pixelSize: popup.fontSize + 2
                        }
                        Text {
                            Layout.fillWidth: true
                            text: row.modelData.text
                            color: popup.fg
                            font.pixelSize: popup.fontSize + 1
                            elide: Text.ElideRight
                            maximumLineCount: 1
                        }
                        Rectangle {
                            visible: hover.containsMouse || row.current
                            implicitWidth: 26
                            implicitHeight: 26
                            radius: 8
                            color: delArea.containsMouse ? Qt.alpha("#f38ba8", 0.25) : "transparent"
                            Text { anchors.centerIn: parent; text: "🗑"; font.pixelSize: popup.fontSize }
                            MouseArea {
                                id: delArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: clip.remove(row.modelData)
                            }
                        }
                    }
                    MouseArea {
                        id: hover
                        anchors.fill: parent
                        anchors.rightMargin: 40
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: clip.copy(row.modelData)
                    }
                }

                Text {
                    anchors.centerIn: parent
                    visible: clip.filtered.length === 0
                    text: clip.entries.length === 0 ? popup.tr("empty") : popup.tr("noresults")
                    color: Qt.alpha(popup.fg, 0.45)
                    font.pixelSize: popup.fontSize + 1
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Text {
                    text: popup.tr("hint")
                    color: Qt.alpha(popup.fg, 0.4)
                    font.pixelSize: popup.fontSize - 1
                }
                Item { Layout.fillWidth: true }
                Rectangle {
                    implicitWidth: wipeText.implicitWidth + 20
                    implicitHeight: 26
                    radius: 8
                    color: clip.confirmWipe ? Qt.alpha("#f38ba8", 0.35) : (wipeArea.containsMouse ? Qt.alpha("#f38ba8", 0.22) : Qt.alpha(popup.fg, 0.06))
                    Behavior on color { ColorAnimation { duration: 120 } }
                    Text { id: wipeText; anchors.centerIn: parent; text: clip.confirmWipe ? popup.tr("wipeconfirm") : popup.tr("wipe"); color: popup.fg; font.pixelSize: popup.fontSize }
                    MouseArea {
                        id: wipeArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (clip.confirmWipe) {
                                clip.confirmWipe = false;
                                wipeProc.running = true;
                            } else {
                                clip.confirmWipe = true;
                                wipeReset.restart();
                            }
                        }
                    }
                }
            }
        }
    }
}
"###,
    );
    ensure_zenith_panel(
        &dir.join("Calendar.qml"),
        r###"// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

// Kalender: klok, maandoverzicht met vandaag gemarkeerd, bladeren met pijlen of scrollwiel.
Scope {
    id: cal

    property date now: new Date()
    property int viewYear: now.getFullYear()
    property int viewMonth: now.getMonth()
    readonly property var loc: popup.locale
    // Maandag-eerst tenzij de locale anders zegt (Qt: 1 = maandag … 7 = zondag)
    readonly property int firstDay: cal.loc.firstDayOfWeek === 0 ? 7 : cal.loc.firstDayOfWeek

    IpcHandler {
        target: "calendar"
        function toggle() { popup.active ? cal.close() : cal.open(); }
        function open() { cal.open(); }
        function close() { cal.close(); }
    }

    function open() {
        cal.now = new Date();
        cal.viewYear = cal.now.getFullYear();
        cal.viewMonth = cal.now.getMonth();
        popup.active = true;
    }
    function close() { popup.active = false; }

    function shift(delta) {
        var m = cal.viewMonth + delta;
        cal.viewYear += Math.floor(m / 12);
        cal.viewMonth = ((m % 12) + 12) % 12;
        gridFade.restart();
    }

    // 42 dagen (6 weken) vanaf de eerste weekdag vóór de 1e van de maand
    readonly property var days: {
        var first = new Date(cal.viewYear, cal.viewMonth, 1);
        var jsDay = first.getDay() === 0 ? 7 : first.getDay(); // 1 = ma … 7 = zo
        var offset = (jsDay - cal.firstDay + 7) % 7;
        var out = [];
        for (var i = 0; i < 42; i++) {
            var d = new Date(cal.viewYear, cal.viewMonth, 1 - offset + i);
            out.push({
                day: d.getDate(),
                inMonth: d.getMonth() === cal.viewMonth,
                today: d.toDateString() === cal.now.toDateString(),
                weekend: d.getDay() === 0 || d.getDay() === 6
            });
        }
        return out;
    }

    Timer {
        interval: 1000
        running: popup.active
        repeat: true
        onTriggered: cal.now = new Date()
    }

    ZenithPopup {
        id: popup
        title: popup.tr("calendar")
        icon: "📅"
        position: "top-right"
        panelWidth: 360
        panelHeight: 470

        ColumnLayout {
            anchors.fill: parent
            spacing: 12

            // Klok en datum
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0
                Text {
                    text: Qt.formatTime(cal.now, "HH:mm")
                    color: popup.fg
                    font.pixelSize: popup.fontSize * 4
                    font.weight: Font.Light
                }
                Text {
                    text: cal.loc.toString(cal.now, "dddd d MMMM yyyy")
                    color: popup.accent
                    font.pixelSize: popup.fontSize + 2
                    font.bold: true
                }
            }

            Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Qt.alpha(popup.fg, 0.1) }

            // Maandnavigatie
            RowLayout {
                Layout.fillWidth: true
                Text {
                    text: cal.loc.standaloneMonthName(cal.viewMonth, Locale.LongFormat) + " " + cal.viewYear
                    color: popup.fg
                    font.pixelSize: popup.fontSize + 3
                    font.bold: true
                    font.capitalization: Font.Capitalize
                }
                Item { Layout.fillWidth: true }
                Repeater {
                    model: [{ label: "‹", delta: -1 }, { label: "•", delta: 0 }, { label: "›", delta: 1 }]
                    Rectangle {
                        required property var modelData
                        implicitWidth: 28
                        implicitHeight: 28
                        radius: 9
                        color: navArea.containsMouse ? Qt.alpha(popup.accent, 0.2) : Qt.alpha(popup.fg, 0.06)
                        Behavior on color { ColorAnimation { duration: 120 } }
                        Text {
                            anchors.centerIn: parent
                            text: parent.modelData.label
                            color: popup.fg
                            font.pixelSize: popup.fontSize + 4
                        }
                        MouseArea {
                            id: navArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: parent.modelData.delta === 0 ? cal.open() : cal.shift(parent.modelData.delta)
                        }
                    }
                }
            }

            // Weekdagen
            RowLayout {
                Layout.fillWidth: true
                spacing: 0
                Repeater {
                    model: 7
                    Text {
                        required property int index
                        Layout.fillWidth: true
                        horizontalAlignment: Text.AlignHCenter
                        text: cal.loc.dayName(((cal.firstDay - 1 + index) % 7 + 1) % 7, Locale.ShortFormat)
                        color: Qt.alpha(popup.fg, 0.5)
                        font.pixelSize: popup.fontSize
                        font.bold: true
                        font.capitalization: Font.Capitalize
                    }
                }
            }

            // Dagenraster
            GridLayout {
                id: grid
                Layout.fillWidth: true
                Layout.fillHeight: true
                columns: 7
                rowSpacing: 4
                columnSpacing: 4
                NumberAnimation on opacity { id: gridFade; from: 0.2; to: 1.0; duration: popup.ms; easing.type: popup.easing; running: false }

                Repeater {
                    model: cal.days
                    Rectangle {
                        required property var modelData
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        radius: 10
                        color: modelData.today ? popup.accent : (dayArea.containsMouse && modelData.inMonth ? Qt.alpha(popup.fg, 0.08) : "transparent")
                        Behavior on color { ColorAnimation { duration: 120 } }
                        Text {
                            anchors.centerIn: parent
                            text: parent.modelData.day
                            font.pixelSize: popup.fontSize + 1
                            font.bold: parent.modelData.today
                            color: parent.modelData.today ? popup.bg
                                 : !parent.modelData.inMonth ? Qt.alpha(popup.fg, 0.25)
                                 : parent.modelData.weekend ? Qt.alpha(popup.fg, 0.6)
                                 : popup.fg
                        }
                        MouseArea { id: dayArea; anchors.fill: parent; hoverEnabled: true }
                    }
                }
            }
        }

        // Scrollwiel bladert door maanden
        WheelHandler {
            onWheel: event => cal.shift(event.angleDelta.y > 0 ? -1 : 1)
        }
    }
}
"###,
    );
    ensure_zenith_panel(
        &dir.join("Media.qml"),
        r###"// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Quickshell.Services.Mpris

// Mediaspeler via MPRIS: hoes, titel/artiest, voortgang (klik om te spoelen), bediening.
Scope {
    id: media

    readonly property var players: Mpris.players.values
    property int chosen: -1
    // Gekozen speler, anders de eerste die speelt, anders de eerste
    readonly property var player: {
        if (media.chosen >= 0 && media.chosen < media.players.length) return media.players[media.chosen];
        for (var i = 0; i < media.players.length; i++) {
            if (media.players[i].isPlaying) return media.players[i];
        }
        return media.players.length > 0 ? media.players[0] : null;
    }

    IpcHandler {
        target: "media"
        function toggle() { popup.active = !popup.active; }
        function open() { popup.active = true; }
        function close() { popup.active = false; }
    }

    function fmt(sec) {
        if (!isFinite(sec) || sec < 0) return "0:00";
        var s = Math.floor(sec % 60);
        return Math.floor(sec / 60) + ":" + (s < 10 ? "0" : "") + s;
    }

    // MPRIS stuurt geen positie-updates tijdens afspelen: zelf verversen
    Timer {
        interval: 1000
        repeat: true
        running: popup.active && media.player !== null && media.player.isPlaying
        onTriggered: media.player.positionChanged()
    }

    ZenithPopup {
        id: popup
        title: popup.tr("media")
        icon: "🎵"
        position: "top"
        panelWidth: 420
        panelHeight: 300

        // Vervaagde hoes als achtergrond
        Image {
            anchors.fill: parent
            anchors.margins: -16
            source: media.player ? media.player.trackArtUrl : ""
            fillMode: Image.PreserveAspectCrop
            opacity: status === Image.Ready ? 0.14 : 0
            Behavior on opacity { NumberAnimation { duration: popup.ms * 2 } }
            layer.enabled: true
            layer.smooth: true
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: 14
            visible: media.player !== null

            RowLayout {
                Layout.fillWidth: true
                spacing: 16

                // Hoes
                Rectangle {
                    implicitWidth: 120
                    implicitHeight: 120
                    radius: 16
                    color: Qt.alpha(popup.fg, 0.08)
                    clip: true
                    scale: media.player && media.player.isPlaying ? 1.0 : 0.94
                    Behavior on scale { NumberAnimation { duration: popup.ms; easing.type: popup.easing } }
                    Image {
                        id: art
                        anchors.fill: parent
                        source: media.player ? media.player.trackArtUrl : ""
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                        opacity: status === Image.Ready ? 1 : 0
                        Behavior on opacity { NumberAnimation { duration: popup.ms } }
                    }
                    Text {
                        anchors.centerIn: parent
                        visible: art.status !== Image.Ready
                        text: "🎵"
                        font.pixelSize: 40
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Text {
                        Layout.fillWidth: true
                        text: media.player ? (media.player.trackTitle || popup.tr("untitled")) : ""
                        color: popup.fg
                        font.pixelSize: popup.fontSize + 5
                        font.bold: true
                        elide: Text.ElideRight
                        maximumLineCount: 2
                        wrapMode: Text.Wrap
                    }
                    Text {
                        Layout.fillWidth: true
                        text: media.player ? (media.player.trackArtist || "") : ""
                        color: popup.accent
                        font.pixelSize: popup.fontSize + 2
                        elide: Text.ElideRight
                    }
                    Text {
                        Layout.fillWidth: true
                        text: media.player ? (media.player.trackAlbum || "") : ""
                        color: Qt.alpha(popup.fg, 0.5)
                        font.pixelSize: popup.fontSize
                        elide: Text.ElideRight
                    }
                    Item { Layout.fillHeight: true }
                    // Speler wisselen (alleen bij meerdere spelers)
                    Text {
                        visible: media.players.length > 1
                        text: "⇄ " + (media.player ? media.player.identity : "")
                        color: Qt.alpha(popup.fg, 0.55)
                        font.pixelSize: popup.fontSize
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: media.chosen = (media.players.indexOf(media.player) + 1) % media.players.length
                        }
                    }
                }
            }

            // Voortgang: klik om te spoelen
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                visible: media.player !== null && media.player.lengthSupported && media.player.length > 0
                Rectangle {
                    id: track
                    Layout.fillWidth: true
                    implicitHeight: seekArea.containsMouse ? 8 : 5
                    radius: height / 2
                    color: Qt.alpha(popup.fg, 0.14)
                    Behavior on implicitHeight { NumberAnimation { duration: 120 } }
                    Rectangle {
                        height: parent.height
                        radius: parent.radius
                        color: popup.accent
                        width: media.player && media.player.length > 0 ? parent.width * Math.min(1, media.player.position / media.player.length) : 0
                        Behavior on width { NumberAnimation { duration: 900; easing.type: Easing.Linear } }
                    }
                    MouseArea {
                        id: seekArea
                        anchors.fill: parent
                        anchors.topMargin: -6
                        anchors.bottomMargin: -6
                        hoverEnabled: true
                        cursorShape: media.player && media.player.canSeek ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: mouse => {
                            if (media.player && media.player.canSeek) media.player.position = media.player.length * (mouse.x / width);
                        }
                    }
                }
                RowLayout {
                    Layout.fillWidth: true
                    Text { text: media.fmt(media.player ? media.player.position : 0); color: Qt.alpha(popup.fg, 0.55); font.pixelSize: popup.fontSize - 1 }
                    Item { Layout.fillWidth: true }
                    Text { text: media.fmt(media.player ? media.player.length : 0); color: Qt.alpha(popup.fg, 0.55); font.pixelSize: popup.fontSize - 1 }
                }
            }

            // Bediening
            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 18
                Repeater {
                    model: [
                        { icon: "⏮", act: "prev", big: false },
                        { icon: media.player && media.player.isPlaying ? "⏸" : "▶", act: "toggle", big: true },
                        { icon: "⏭", act: "next", big: false }
                    ]
                    Rectangle {
                        required property var modelData
                        implicitWidth: modelData.big ? 54 : 40
                        implicitHeight: implicitWidth
                        radius: implicitWidth / 2
                        color: modelData.big ? popup.accent : (btnArea.containsMouse ? Qt.alpha(popup.fg, 0.14) : Qt.alpha(popup.fg, 0.06))
                        scale: btnArea.pressed ? 0.9 : (btnArea.containsMouse ? 1.06 : 1.0)
                        Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutBack } }
                        Behavior on color { ColorAnimation { duration: 120 } }
                        Text {
                            anchors.centerIn: parent
                            text: parent.modelData.icon
                            color: parent.modelData.big ? popup.bg : popup.fg
                            font.pixelSize: parent.modelData.big ? 22 : 16
                        }
                        MouseArea {
                            id: btnArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                var p = media.player;
                                if (!p) return;
                                if (parent.modelData.act === "prev" && p.canGoPrevious) p.previous();
                                else if (parent.modelData.act === "next" && p.canGoNext) p.next();
                                else if (parent.modelData.act === "toggle" && p.canTogglePlaying) p.togglePlaying();
                            }
                        }
                    }
                }
            }
        }

        // Niets speelt
        ColumnLayout {
            anchors.centerIn: parent
            visible: media.player === null
            spacing: 6
            Text { Layout.alignment: Qt.AlignHCenter; text: "🎧"; font.pixelSize: 44 }
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: popup.tr("nothing")
                color: Qt.alpha(popup.fg, 0.6)
                font.pixelSize: popup.fontSize + 2
            }
        }
    }
}
"###,
    );
    ensure_zenith_panel(
        &dir.join("Launcher.qml"),
        r###"// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

// App-launcher: zoek op naam, omschrijving of trefwoord; ↑/↓ + Enter of klik om te openen.
Scope {
    id: launcher

    property string query: ""
    readonly property var apps: DesktopEntries.applications.values
    readonly property var results: {
        var q = launcher.query.toLowerCase().trim();
        var scored = [];
        for (var i = 0; i < launcher.apps.length; i++) {
            var a = launcher.apps[i];
            if (a.noDisplay) continue;
            var name = (a.name || "").toLowerCase();
            var score = 0;
            if (q.length === 0) score = 1;
            else if (name.indexOf(q) === 0) score = 4;
            else if (name.indexOf(q) !== -1) score = 3;
            else if ((a.genericName || "").toLowerCase().indexOf(q) !== -1) score = 2;
            else if ((a.keywords || []).join(" ").toLowerCase().indexOf(q) !== -1 || (a.comment || "").toLowerCase().indexOf(q) !== -1) score = 1;
            if (score > 0) scored.push({ app: a, score: score, name: name });
        }
        scored.sort((x, y) => y.score - x.score || x.name.localeCompare(y.name));
        return scored.slice(0, 60).map(s => s.app);
    }

    IpcHandler {
        target: "launcher"
        function toggle() { popup.active ? launcher.close() : launcher.open(); }
        function open() { launcher.open(); }
        function close() { launcher.close(); }
    }

    function open() {
        search.text = "";
        launcher.query = "";
        list.currentIndex = 0;
        popup.active = true;
        search.forceActiveFocus();
    }
    function close() { popup.active = false; }
    function launch(app) {
        if (!app) return;
        app.execute();
        launcher.close();
    }

    ZenithPopup {
        id: popup
        title: popup.tr("apps")
        icon: "🚀"
        position: "top"
        panelWidth: 540
        panelHeight: 540
        wantsKeyboard: true

        ColumnLayout {
            anchors.fill: parent
            spacing: 10

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 44
                radius: 14
                color: Qt.alpha(popup.fg, 0.06)
                border.width: search.activeFocus ? 1 : 0
                border.color: popup.accent

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 14
                    anchors.rightMargin: 14
                    spacing: 10
                    Text { text: "🔍"; font.pixelSize: popup.fontSize + 3 }
                    TextInput {
                        id: search
                        Layout.fillWidth: true
                        color: popup.fg
                        font.pixelSize: popup.fontSize + 4
                        clip: true
                        onTextChanged: { launcher.query = text; list.currentIndex = 0; }
                        Keys.onEscapePressed: launcher.close()
                        Keys.onDownPressed: list.incrementCurrentIndex()
                        Keys.onUpPressed: list.decrementCurrentIndex()
                        Keys.onReturnPressed: launcher.launch(launcher.results[list.currentIndex])
                        Text {
                            visible: search.text.length === 0
                            text: popup.tr("searchapps")
                            color: Qt.alpha(popup.fg, 0.4)
                            font.pixelSize: popup.fontSize + 4
                        }
                    }
                }
            }

            ListView {
                id: list
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 2
                model: launcher.results
                highlightMoveDuration: popup.ms

                delegate: Rectangle {
                    id: row
                    required property var modelData
                    required property int index
                    width: list.width
                    implicitHeight: 52
                    radius: 12
                    readonly property bool current: ListView.isCurrentItem
                    color: current ? Qt.alpha(popup.accent, 0.18) : (hover.containsMouse ? Qt.alpha(popup.fg, 0.07) : "transparent")
                    Behavior on color { ColorAnimation { duration: 120 } }
                    opacity: 0
                    Component.onCompleted: opacity = 1
                    Behavior on opacity { NumberAnimation { duration: popup.ms } }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 12
                        spacing: 12
                        Item {
                            implicitWidth: 34
                            implicitHeight: 34
                            Image {
                                id: appIcon
                                anchors.fill: parent
                                source: row.modelData.icon ? Quickshell.iconPath(row.modelData.icon, true) : ""
                                sourceSize: Qt.size(64, 64)
                                asynchronous: true
                                scale: row.current ? 1.08 : 1.0
                                Behavior on scale { NumberAnimation { duration: popup.ms; easing.type: popup.easing } }
                            }
                            Text {
                                anchors.centerIn: parent
                                visible: appIcon.status !== Image.Ready
                                text: (row.modelData.name || "?").charAt(0).toUpperCase()
                                color: popup.accent
                                font.pixelSize: popup.fontSize + 6
                                font.bold: true
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 1
                            Text {
                                Layout.fillWidth: true
                                text: row.modelData.name
                                color: popup.fg
                                font.pixelSize: popup.fontSize + 2
                                font.bold: row.current
                                elide: Text.ElideRight
                            }
                            Text {
                                Layout.fillWidth: true
                                visible: text.length > 0
                                text: row.modelData.comment || row.modelData.genericName || ""
                                color: Qt.alpha(popup.fg, 0.5)
                                font.pixelSize: popup.fontSize
                                elide: Text.ElideRight
                            }
                        }
                    }
                    MouseArea {
                        id: hover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onEntered: list.currentIndex = row.index
                        onClicked: launcher.launch(row.modelData)
                    }
                }

                Text {
                    anchors.centerIn: parent
                    visible: launcher.results.length === 0
                    text: popup.tr("noapps")
                    color: Qt.alpha(popup.fg, 0.45)
                    font.pixelSize: popup.fontSize + 1
                }
            }

            Text {
                text: popup.tr("launchhint")
                color: Qt.alpha(popup.fg, 0.4)
                font.pixelSize: popup.fontSize - 1
            }
        }
    }
}
"###,
    );
    ensure_zenith_panel(
        &dir.join("Notifications.qml"),
        r###"// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Io
import Quickshell.Services.Notifications

// Meldingen: pop-ups rechtsboven + paneel met geschiedenis, niet-storen en alles wissen.
// Werkt alleen als geen andere meldingendienst (swaync, dunst, mako, serpantinum) draait.
Scope {
    id: nc

    property bool dnd: false
    property var popups: []
    readonly property var history: server.trackedNotifications.values

    function hidePopup(n) { nc.popups = nc.popups.filter(x => x !== n); }
    function clearAll() {
        var all = nc.history.slice();
        for (var i = 0; i < all.length; i++) all[i].dismiss();
        nc.popups = [];
    }

    NotificationServer {
        id: server
        keepOnReload: true
        actionsSupported: true
        imageSupported: true
        bodySupported: true
        // Platte tekst: apps bepalen de inhoud, dus geen opmaak/links interpreteren.
        bodyMarkupSupported: false
        persistenceSupported: true
        onNotification: n => {
            n.tracked = true;
            if (!nc.dnd || n.urgency === NotificationUrgency.Critical) nc.popups = nc.popups.concat([n]).slice(-4);
        }
    }

    IpcHandler {
        target: "notifications"
        function toggle() { popup.active = !popup.active; }
        function open() { popup.active = true; }
        function close() { popup.active = false; }
        function toggleDnd() { nc.dnd = !nc.dnd; }
        function clear() { nc.clearAll(); }
    }

    // Eén melding als kaart (gebruikt door pop-ups én het paneel)
    component NotifCard: Rectangle {
        id: cardRoot
        required property var notif
        // Thema van het paneel (inline-components zien de ids van het bestand niet)
        required property var theme
        property bool compact: false
        signal closeRequested()
        implicitHeight: content.implicitHeight + 24
        radius: 14
        color: Qt.alpha(cardRoot.theme.surface, 0.9)
        border.width: 1
        border.color: notif && notif.urgency === NotificationUrgency.Critical ? "#f38ba8" : Qt.alpha(cardRoot.theme.border, 0.7)

        RowLayout {
            id: content
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 12
            spacing: 12

            Item {
                Layout.alignment: Qt.AlignTop
                implicitWidth: 36
                implicitHeight: 36
                Image {
                    id: nIcon
                    anchors.fill: parent
                    source: !cardRoot.notif ? "" : (cardRoot.notif.image || (cardRoot.notif.appIcon ? Quickshell.iconPath(cardRoot.notif.appIcon, true) : ""))
                    sourceSize: Qt.size(72, 72)
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                }
                Text {
                    anchors.centerIn: parent
                    visible: nIcon.status !== Image.Ready
                    text: "🔔"
                    font.pixelSize: 20
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                RowLayout {
                    Layout.fillWidth: true
                    Text {
                        Layout.fillWidth: true
                        text: cardRoot.notif ? (cardRoot.notif.appName || "") : ""
                        color: Qt.alpha(cardRoot.theme.fg, 0.5)
                        font.pixelSize: cardRoot.theme.fontSize - 1
                        elide: Text.ElideRight
                    }
                    Text {
                        text: "✕"
                        color: Qt.alpha(cardRoot.theme.fg, xArea.containsMouse ? 0.9 : 0.45)
                        font.pixelSize: cardRoot.theme.fontSize
                        MouseArea {
                            id: xArea
                            anchors.fill: parent
                            anchors.margins: -6
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: cardRoot.closeRequested()
                        }
                    }
                }
                Text {
                    Layout.fillWidth: true
                    text: cardRoot.notif ? cardRoot.notif.summary : ""
                    textFormat: Text.PlainText
                    color: cardRoot.theme.fg
                    font.pixelSize: cardRoot.theme.fontSize + 2
                    font.bold: true
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                }
                Text {
                    Layout.fillWidth: true
                    visible: text.length > 0
                    text: cardRoot.notif ? cardRoot.notif.body : ""
                    textFormat: Text.PlainText
                    color: Qt.alpha(cardRoot.theme.fg, 0.75)
                    font.pixelSize: cardRoot.theme.fontSize + 1
                    wrapMode: Text.Wrap
                    maximumLineCount: cardRoot.compact ? 2 : 5
                    elide: Text.ElideRight
                }
                // Actieknoppen van de app
                Flow {
                    Layout.fillWidth: true
                    Layout.topMargin: 4
                    spacing: 6
                    visible: cardRoot.notif && cardRoot.notif.actions.length > 0
                    Repeater {
                        model: cardRoot.notif ? cardRoot.notif.actions : []
                        Rectangle {
                            required property var modelData
                            implicitWidth: actText.implicitWidth + 20
                            implicitHeight: 26
                            radius: 8
                            color: actArea.containsMouse ? Qt.alpha(cardRoot.theme.accent, 0.3) : Qt.alpha(cardRoot.theme.accent, 0.15)
                            Behavior on color { ColorAnimation { duration: 120 } }
                            Text { id: actText; anchors.centerIn: parent; text: parent.modelData.text; color: cardRoot.theme.fg; font.pixelSize: cardRoot.theme.fontSize }
                            MouseArea {
                                id: actArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: parent.modelData.invoke()
                            }
                        }
                    }
                }
            }
        }
    }

    // ===== Pop-ups rechtsboven =====
    PanelWindow {
        visible: nc.popups.length > 0
        color: "transparent"
        anchors { top: true; right: true }
        margins { top: 52; right: 12 }
        implicitWidth: 380
        implicitHeight: Math.max(1, toastCol.implicitHeight)
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        exclusionMode: ExclusionMode.Ignore

        Column {
            id: toastCol
            width: parent.width
            spacing: 8
            Repeater {
                model: nc.popups
                NotifCard {
                    id: toast
                    required property var modelData
                    notif: modelData
                    theme: popup
                    compact: true
                    width: toastCol.width
                    onCloseRequested: nc.hidePopup(modelData)

                    // Inglijden van rechts
                    opacity: 0
                    transform: Translate {
                        id: slide
                        x: 40
                        Behavior on x { NumberAnimation { duration: popup.ms; easing.type: popup.easing } }
                    }
                    Component.onCompleted: { opacity = 1; slide.x = 0; }
                    Behavior on opacity { NumberAnimation { duration: popup.ms } }

                    Timer {
                        // Kritieke meldingen blijven staan tot je ze sluit
                        running: toast.modelData && toast.modelData.urgency !== NotificationUrgency.Critical
                        interval: toast.modelData && toast.modelData.expireTimeout > 0 ? toast.modelData.expireTimeout * 1000 : 5000
                        onTriggered: nc.hidePopup(toast.modelData)
                    }
                    MouseArea {
                        anchors.fill: parent
                        z: -1
                        onClicked: nc.hidePopup(toast.modelData)
                    }
                }
            }
        }
        Behavior on implicitHeight { NumberAnimation { duration: popup.ms; easing.type: popup.easing } }
    }

    // ===== Meldingenpaneel =====
    ZenithPopup {
        id: popup
        title: popup.tr("notifications")
        icon: "🔔"
        position: "top-right"
        panelWidth: 400
        panelHeight: 560

        ColumnLayout {
            anchors.fill: parent
            spacing: 10

            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                // Niet storen
                Rectangle {
                    implicitWidth: dndRow.implicitWidth + 20
                    implicitHeight: 30
                    radius: 10
                    color: nc.dnd ? Qt.alpha(popup.accent, 0.25) : Qt.alpha(popup.fg, 0.06)
                    border.width: nc.dnd ? 1 : 0
                    border.color: popup.accent
                    Behavior on color { ColorAnimation { duration: popup.ms } }
                    RowLayout {
                        id: dndRow
                        anchors.centerIn: parent
                        spacing: 6
                        Text { text: nc.dnd ? "🔕" : "🔔"; font.pixelSize: popup.fontSize + 1 }
                        Text { text: popup.tr("dnd"); color: popup.fg; font.pixelSize: popup.fontSize }
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: nc.dnd = !nc.dnd
                    }
                }
                Item { Layout.fillWidth: true }
                Rectangle {
                    visible: nc.history.length > 0
                    implicitWidth: clearText.implicitWidth + 20
                    implicitHeight: 30
                    radius: 10
                    color: clearArea.containsMouse ? Qt.alpha("#f38ba8", 0.22) : Qt.alpha(popup.fg, 0.06)
                    Behavior on color { ColorAnimation { duration: 120 } }
                    Text { id: clearText; anchors.centerIn: parent; text: popup.tr("wipe"); color: popup.fg; font.pixelSize: popup.fontSize }
                    MouseArea {
                        id: clearArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: nc.clearAll()
                    }
                }
            }

            ListView {
                id: hist
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 8
                // Nieuwste bovenaan
                model: nc.history.slice().reverse()
                add: Transition { NumberAnimation { property: "opacity"; from: 0; to: 1; duration: popup.ms } }
                remove: Transition { NumberAnimation { property: "opacity"; to: 0; duration: popup.ms } }
                delegate: NotifCard {
                    required property var modelData
                    notif: modelData
                    theme: popup
                    width: hist.width
                    onCloseRequested: modelData.dismiss()
                }

                ColumnLayout {
                    anchors.centerIn: parent
                    visible: nc.history.length === 0
                    spacing: 6
                    Text { Layout.alignment: Qt.AlignHCenter; text: "🔕"; font.pixelSize: 40; opacity: 0.6 }
                    Text {
                        Layout.alignment: Qt.AlignHCenter
                        text: popup.tr("nonotif")
                        color: Qt.alpha(popup.fg, 0.5)
                        font.pixelSize: popup.fontSize + 1
                    }
                }
            }
        }
    }
}
"###,
    );
    ensure_zenith_panel(
        &dir.join("ControlCenter.qml"),
        r###"// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Io

Scope {
    id: ccScope

    FileView {
        id: shellConfigFile
        path: Quickshell.env("HOME") + "/.config/quickshell/zenith-shell.json"
        watchChanges: true
    }

    readonly property var shellConfig: {
        try {
            var txt = shellConfigFile.text().trim();
            if (txt.length > 0) return JSON.parse(txt);
        } catch (e) {}
        return {};
    }

    readonly property var ccConfig: ccScope.shellConfig.control_center || {}
    readonly property var customScripts: ccScope.shellConfig.custom_scripts || []
    readonly property var icons: ccScope.shellConfig.custom_icons || {}
    readonly property bool isCcEnabled: ccScope.ccConfig.enabled !== undefined ? ccScope.ccConfig.enabled : true
    readonly property int ccWidth: ccScope.ccConfig.width || 380
    readonly property int ccMaxHeight: ccScope.ccConfig.max_height || 600
    readonly property string ccPosition: ccScope.ccConfig.position || "top-right"
    readonly property int ccRadius: ccScope.ccConfig.border_radius !== undefined ? ccScope.ccConfig.border_radius : 16
    readonly property real ccOpacity: ccScope.ccConfig.opacity !== undefined ? ccScope.ccConfig.opacity : 0.92
    readonly property color bgColor: (ccScope.shellConfig.styling && ccScope.shellConfig.styling.background) || "#1e1e2e"
    readonly property color borderColor: (ccScope.shellConfig.styling && ccScope.shellConfig.styling.border_color) || "#45475a"
    readonly property color textColor: (ccScope.shellConfig.styling && ccScope.shellConfig.styling.text_color) || "#cdd6f4"

    property bool isCcActive: false

    // Animaties volgen Zenith → Statusbalk → Animaties
    readonly property var an: ccScope.shellConfig.animations || {}
    readonly property int ms: ccScope.an.duration_ms !== undefined ? Math.max(ccScope.an.duration_ms, 120) : 240
    readonly property int easing: {
        var e = Easing[ccScope.an.curve || "OutBack"];
        return e !== undefined ? e : Easing.OutBack;
    }

    // Snelle acties; afmelden/herstarten/uitschakelen vragen een tweede klik binnen 3 s
    property string confirmAction: ""
    readonly property string lang: ccScope.shellConfig.language || "nl"
    readonly property var labels: ({
        nl: { lock: "Vergrendel", settings: "Zenith", logout: "Afmelden", reboot: "Herstart", poweroff: "Uit", sure: "Zeker?" },
        en: { lock: "Lock", settings: "Zenith", logout: "Log out", reboot: "Restart", poweroff: "Power off", sure: "Sure?" },
        de: { lock: "Sperren", settings: "Zenith", logout: "Abmelden", reboot: "Neustart", poweroff: "Aus", sure: "Sicher?" },
        es: { lock: "Bloquear", settings: "Zenith", logout: "Salir", reboot: "Reiniciar", poweroff: "Apagar", sure: "¿Seguro?" }
    })
    function label(k) { return (ccScope.labels[ccScope.lang] || ccScope.labels.nl)[k]; }
    Timer { id: confirmReset; interval: 3000; onTriggered: ccScope.confirmAction = "" }
    Process { id: actionProc }
    function runAction(a) {
        if (a.confirm && ccScope.confirmAction !== a.id) {
            ccScope.confirmAction = a.id;
            confirmReset.restart();
            return;
        }
        ccScope.confirmAction = "";
        ccScope.isCcActive = false;
        actionProc.command = ["bash", "-c", a.cmd];
        actionProc.running = true;
    }

    // Hyprland / Quickshell IPC Handler
    IpcHandler {
        target: "controlCenter"
        function toggle() {
            if (ccScope.isCcEnabled) {
                ccScope.isCcActive = !ccScope.isCcActive;
            }
        }
        function open() {
            if (ccScope.isCcEnabled) ccScope.isCcActive = true;
        }
        function close() {
            ccScope.isCcActive = false;
        }
    }

    // Dismiss Overlay: klik buiten paneel sluit Control Center
    PanelWindow {
        id: dismissOverlay
        visible: ccScope.isCcActive
        color: "transparent"
        anchors {
            top: true
            bottom: true
            left: true
            right: true
        }
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        MouseArea {
            anchors.fill: parent
            onClicked: ccScope.isCcActive = false
        }
    }

    // Hoofdvenster Control Center
    PanelWindow {
        id: ccWindow
        visible: ccScope.isCcActive || mainBg.opacity > 0.01
        color: "transparent"
        implicitWidth: ccScope.ccWidth
        implicitHeight: Math.min(contentCol.implicitHeight + 36, ccScope.ccMaxHeight)

        anchors {
            top: ccScope.ccPosition.indexOf("top") !== -1
            bottom: ccScope.ccPosition.indexOf("bottom") !== -1
            right: ccScope.ccPosition.indexOf("right") !== -1
            left: ccScope.ccPosition.indexOf("left") !== -1
        }
        margins {
            top: 48
            bottom: 48
            left: 16
            right: 16
        }
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        Rectangle {
            id: mainBg
            anchors.fill: parent
            radius: ccScope.ccRadius
            color: Qt.alpha(ccScope.bgColor, ccScope.ccOpacity)
            border.width: 1
            border.color: ccScope.borderColor

            opacity: ccScope.isCcActive ? 1.0 : 0.0
            scale: ccScope.isCcActive ? 1.0 : 0.90
            transformOrigin: {
                if (ccScope.ccPosition === "top-right") return Item.TopRight;
                if (ccScope.ccPosition === "top-left") return Item.TopLeft;
                if (ccScope.ccPosition === "bottom-right") return Item.BottomRight;
                if (ccScope.ccPosition === "bottom-left") return Item.BottomLeft;
                return Item.Center;
            }

            Behavior on opacity {
                NumberAnimation { duration: Math.max(140, ccScope.ms * 0.8); easing.type: Easing.OutCubic }
            }
            Behavior on scale {
                NumberAnimation { duration: ccScope.ms; easing.type: ccScope.easing }
            }

            Flickable {
                id: flick
                anchors.fill: parent
                anchors.margins: 14
                contentHeight: contentCol.implicitHeight
                clip: true

                ColumnLayout {
                    id: contentCol
                    width: flick.width
                    spacing: 10

                    // Header
                    RowLayout {
                        Layout.fillWidth: true
                        Text {
                            text: "Control Center"
                            font.bold: true
                            font.pixelSize: 13
                            color: ccScope.textColor
                        }
                        Item { Layout.fillWidth: true }
                        Rectangle {
                            width: 22
                            height: 22
                            radius: 11
                            color: Qt.alpha(ccScope.textColor, 0.12)
                            Text {
                                anchors.centerIn: parent
                                text: (ccScope.icons && ccScope.icons.close) || "✕"
                                font.pixelSize: 10
                                color: ccScope.textColor
                            }
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: ccScope.isCcActive = false
                            }
                        }
                    }

                    // Snelle acties
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 6
                        Repeater {
                            model: [
                                { id: "lock", icon: "🔒", confirm: false, cmd: "quickshell -p \"$HOME/.config/quickshell/shell.qml\" ipc call lockscreen lock" },
                                { id: "settings", icon: "⚙️", confirm: false, cmd: "zenith" },
                                { id: "logout", icon: "🚪", confirm: true, cmd: "hyprctl dispatch 'hl.dsp.exit()' 2>/dev/null | grep -q ok || hyprctl dispatch exit" },
                                { id: "reboot", icon: "🔄", confirm: true, cmd: "systemctl reboot" },
                                { id: "poweroff", icon: "⏻", confirm: true, cmd: "systemctl poweroff" }
                            ]
                            Rectangle {
                                id: actBtn
                                required property var modelData
                                readonly property bool confirming: ccScope.confirmAction === modelData.id
                                Layout.fillWidth: true
                                implicitHeight: 54
                                radius: 12
                                color: confirming ? Qt.alpha("#f38ba8", 0.28) : (actArea.containsMouse ? Qt.alpha(ccScope.textColor, 0.12) : Qt.alpha(ccScope.textColor, 0.05))
                                border.width: confirming ? 1 : 0
                                border.color: "#f38ba8"
                                scale: actArea.pressed ? 0.94 : 1.0
                                Behavior on color { ColorAnimation { duration: 140 } }
                                Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutBack } }
                                ColumnLayout {
                                    anchors.centerIn: parent
                                    spacing: 2
                                    Text { Layout.alignment: Qt.AlignHCenter; text: actBtn.modelData.icon; font.pixelSize: 16 }
                                    Text {
                                        Layout.alignment: Qt.AlignHCenter
                                        text: actBtn.confirming ? ccScope.label("sure") : ccScope.label(actBtn.modelData.id)
                                        color: ccScope.textColor
                                        font.pixelSize: 9
                                        font.bold: actBtn.confirming
                                    }
                                }
                                MouseArea {
                                    id: actArea
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: ccScope.runAction(actBtn.modelData)
                                }
                            }
                        }
                    }

                    // Dynamische Repeater & Loader voor alle actieve kaarten
                    Repeater {
                        model: ccScope.ccConfig.cards || []
                        Loader {
                            id: cardLoader
                            required property var modelData
                            Layout.fillWidth: true
                            source: {
                                if (typeof modelData === "string" && modelData.indexOf("script:") === 0) {
                                    return Quickshell.env("HOME") + "/.config/quickshell/cards/script_card.qml";
                                }
                                return Quickshell.env("HOME") + "/.config/quickshell/cards/" + modelData + ".qml";
                            }
                            onLoaded: {
                                if (typeof modelData === "string" && modelData.indexOf("script:") === 0 && item) {
                                    var sId = modelData.substring(7);
                                    for (var i = 0; i < ccScope.customScripts.length; i++) {
                                        var s = ccScope.customScripts[i];
                                        if (s.id === sId) {
                                            item.scriptCommand = s.command || "";
                                            item.intervalSec = s.interval_seconds || 10;
                                            item.scriptIcon = s.icon || ((ccScope.icons && ccScope.icons.script) || "💻");
                                            item.onClickCmd = s.on_click || "";
                                            item.cardName = s.name || "Custom Script";
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
"###,
    );
    ensure_zenith_panel(
        &dir.join("Lockscreen.qml"),
        r###"// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import Quickshell
import Quickshell.Wayland
import Quickshell.Io
import Quickshell.Services.Pam
import Quickshell.Services.Mpris

// Zenith-lockscreen: echte sessie-lock (ext-session-lock) met PAM.
// Veiligheid: het wachtwoord gaat alleen naar PAM (nooit naar een proces of log),
// en vergrendelen weigert als de PAM-config ontbreekt, zodat je nooit buitengesloten raakt.
Scope {
    id: ls

    readonly property string pamService: "quickshell"
    property bool isLocked: false
    property bool isPreview: false
    property bool busy: false
    property bool unlocking: false
    property int failCount: 0
    property string statusText: ""
    property string pendingPass: ""

    FileView {
        id: cfgFile
        path: Quickshell.env("HOME") + "/.config/quickshell/zenith-shell.json"
        watchChanges: true
    }
    readonly property var cfg: {
        try {
            var t = cfgFile.text().trim();
            if (t.length > 0) return JSON.parse(t);
        } catch (e) {}
        return {};
    }
    readonly property var lc: ls.cfg.lockscreen || {}
    readonly property var st: ls.cfg.styling || {}
    readonly property var an: ls.cfg.animations || {}
    readonly property color fg: ls.st.text_color || "#cdd6f4"
    readonly property color accent: ls.st.accent || "#89b4fa"
    readonly property color surface: ls.st.pill_bg || "#181825"
    readonly property color border: ls.st.border_color || "#45475a"
    readonly property int ms: ls.an.duration_ms !== undefined ? Math.max(ls.an.duration_ms, 120) : 250
    readonly property int easing: {
        var e = Easing[ls.an.curve || "OutCubic"];
        return e !== undefined ? e : Easing.OutCubic;
    }
    readonly property string lang: ls.cfg.language || "nl"
    readonly property var locale: Qt.locale(({ nl: "nl_NL", en: "en_GB", de: "de_DE", es: "es_ES" })[ls.lang] || "nl_NL")
    readonly property var dict: ({
        nl: { password: "Wachtwoord", checking: "Controleren…", wrong: "Onjuist wachtwoord", welcome: "Welkom terug", preview: "Voorbeeld — Esc om te sluiten" },
        en: { password: "Password", checking: "Checking…", wrong: "Wrong password", welcome: "Welcome back", preview: "Preview — press Esc to close" },
        de: { password: "Passwort", checking: "Wird geprüft…", wrong: "Falsches Passwort", welcome: "Willkommen zurück", preview: "Vorschau — Esc zum Schließen" },
        es: { password: "Contraseña", checking: "Comprobando…", wrong: "Contraseña incorrecta", welcome: "Bienvenido de nuevo", preview: "Vista previa — Esc para cerrar" }
    })
    function tr(k) { return (ls.dict[ls.lang] || ls.dict.nl)[k]; }
    readonly property var cards: ls.lc.cards || ["clock", "avatar", "auth", "mpris", "battery_network"]
    function shows(card) { return ls.cards.indexOf(card) !== -1 || card === "auth"; }

    // Bestaat de PAM-config? Zonder kan niemand ontgrendelen, dus dan niet vergrendelen.
    FileView {
        id: pamCfg
        path: "/etc/pam.d/" + ls.pamService
    }

    IpcHandler {
        target: "lockscreen"
        function lock() { ls.lock(); }
        function preview() { ls.isPreview = !ls.isPreview; }
        // Bewust geen unlock(): ontgrendelen kan alleen met je wachtwoord.
    }

    function lock() {
        if (ls.lc.enabled === false) return;
        if (pamCfg.text().length === 0) {
            console.warn("Zenith-lockscreen: /etc/pam.d/" + ls.pamService + " ontbreekt; niet vergrendeld.");
            return;
        }
        ls.statusText = "";
        ls.unlocking = false;
        ls.isLocked = true;
    }

    function submit(pass) {
        if (ls.busy || pass.length === 0) return;
        ls.pendingPass = pass;
        ls.busy = true;
        ls.statusText = ls.tr("checking");
        if (!pam.start()) {
            ls.pendingPass = "";
            ls.busy = false;
            ls.statusText = "PAM kon niet starten";
            ls.failCount++;
        }
    }

    function succeeded() {
        ls.busy = false;
        ls.statusText = "";
        if (ls.isPreview && !ls.isLocked) {
            ls.isPreview = false;
            return;
        }
        // Eerst uitfaden, dan pas de sessie vrijgeven
        ls.unlocking = true;
        unlockTimer.restart();
    }

    Timer {
        id: unlockTimer
        interval: ls.ms
        onTriggered: { ls.isLocked = false; ls.unlocking = false; }
    }

    PamContext {
        id: pam
        config: ls.pamService
        user: Quickshell.env("USER") || ""
        onResponseRequiredChanged: {
            if (responseRequired && ls.pendingPass.length > 0) {
                respond(ls.pendingPass);
                ls.pendingPass = "";
            }
        }
        onCompleted: result => {
            ls.pendingPass = "";
            if (result === PamResult.Success) {
                ls.succeeded();
            } else {
                ls.busy = false;
                ls.statusText = ls.tr("wrong");
                ls.failCount++;
            }
        }
        onError: err => {
            ls.pendingPass = "";
            ls.busy = false;
            ls.statusText = "PAM: " + err;
            ls.failCount++;
        }
    }

    // Echte vergrendeling: één oppervlak per scherm
    WlSessionLock {
        locked: ls.isLocked
        surface: Component {
            WlSessionLockSurface {
                color: "black"
                LockView { anchors.fill: parent }
            }
        }
    }

    // Veilige voorbeeldmodus (gewoon venster, Esc sluit)
    PanelWindow {
        visible: ls.isPreview && !ls.isLocked
        color: "black"
        anchors { top: true; bottom: true; left: true; right: true }
        exclusionMode: ExclusionMode.Ignore
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
        LockView { anchors.fill: parent; preview: true }
    }

    component LockView: Item {
        id: view
        property bool preview: false
        property date now: new Date()

        Timer { interval: 1000; running: true; repeat: true; onTriggered: view.now = new Date() }

        // Achtergrond: wallpaper (optioneel vervaagd) of effen zwart
        readonly property string mode: ls.lc.background_mode || "wallpaper_blur"
        Image {
            id: bg
            anchors.fill: parent
            visible: false
            source: view.mode === "solid_black" ? ""
                  : view.mode === "custom_image" && ls.lc.custom_image_path ? "file://" + ls.lc.custom_image_path
                  : ls.cfg.wallpaper_path ? "file://" + ls.cfg.wallpaper_path : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
        }
        MultiEffect {
            anchors.fill: parent
            source: bg
            visible: bg.status === Image.Ready
            blurEnabled: view.mode === "wallpaper_blur"
            blurMax: 64
            blur: Math.min(1, (ls.lc.blur_radius !== undefined ? ls.lc.blur_radius : 24) / 40)
            autoPaddingEnabled: false
        }
        Rectangle {
            anchors.fill: parent
            color: "black"
            opacity: view.mode === "solid_black" ? 1 : (ls.lc.dim_opacity !== undefined ? ls.lc.dim_opacity : 0.4)
        }

        // Alle inhoud fadet/schaalt in bij vergrendelen en uit bij ontgrendelen
        Item {
            id: content
            anchors.fill: parent
            opacity: ls.unlocking ? 0 : 1
            scale: ls.unlocking ? 1.04 : 1
            Behavior on opacity { NumberAnimation { duration: ls.ms; easing.type: Easing.OutCubic } }
            Behavior on scale { NumberAnimation { duration: ls.ms; easing.type: ls.easing } }
            Component.onCompleted: { opacity = 0; opacity = Qt.binding(() => ls.unlocking ? 0 : 1); }

            readonly property string preset: ls.lc.layout_preset || "centered"

            GridLayout {
                columns: content.preset === "split" ? 2 : 1
                columnSpacing: 120
                rowSpacing: 28
                anchors.verticalCenter: parent.verticalCenter
                anchors.horizontalCenter: content.preset === "left_aligned" ? undefined : parent.horizontalCenter
                anchors.left: content.preset === "left_aligned" ? parent.left : undefined
                anchors.leftMargin: 110

                // Klok, datum, media
                ColumnLayout {
                    spacing: 18
                    Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                    ColumnLayout {
                        visible: ls.shows("clock")
                        spacing: 2
                        Text {
                            Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                            text: Qt.formatTime(view.now, ls.lc.clock_format || "HH:mm")
                            color: ls.fg
                            font.pixelSize: ls.lc.clock_font_size || 96
                            font.family: ls.lc.clock_font_family || "JetBrains Mono"
                            font.weight: Font.Light
                        }
                        Text {
                            Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                            text: ls.locale.toString(view.now, "dddd d MMMM")
                            color: ls.accent
                            font.pixelSize: Math.max(16, (ls.lc.clock_font_size || 96) * 0.22)
                            font.capitalization: Font.Capitalize
                        }
                    }

                    // Media (alleen als er iets speelt of gepauzeerd is)
                    Rectangle {
                        id: mediaCard
                        readonly property var player: Mpris.players.values.length > 0 ? Mpris.players.values[0] : null
                        visible: ls.shows("mpris") && player !== null
                        Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                        implicitWidth: 340
                        implicitHeight: 72
                        radius: 18
                        color: Qt.alpha(ls.surface, 0.7)
                        border.width: 1
                        border.color: Qt.alpha(ls.border, 0.6)
                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 12
                            Rectangle {
                                implicitWidth: 48; implicitHeight: 48; radius: 12; clip: true
                                color: Qt.alpha(ls.fg, 0.08)
                                Image { anchors.fill: parent; source: mediaCard.player ? mediaCard.player.trackArtUrl : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true }
                            }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 0
                                Text { Layout.fillWidth: true; text: mediaCard.player ? mediaCard.player.trackTitle : ""; color: ls.fg; font.bold: true; elide: Text.ElideRight }
                                Text { Layout.fillWidth: true; text: mediaCard.player ? mediaCard.player.trackArtist : ""; color: Qt.alpha(ls.fg, 0.6); elide: Text.ElideRight }
                            }
                            Text {
                                text: mediaCard.player && mediaCard.player.isPlaying ? "⏸" : "▶"
                                color: ls.fg
                                font.pixelSize: 20
                                MouseArea {
                                    anchors.fill: parent
                                    anchors.margins: -8
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: if (mediaCard.player) mediaCard.player.togglePlaying()
                                }
                            }
                        }
                    }
                }

                // Gebruiker, wachtwoord, status
                ColumnLayout {
                    spacing: 16
                    Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft

                    ColumnLayout {
                        visible: ls.shows("avatar")
                        spacing: 10
                        Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                        Rectangle {
                            Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                            implicitWidth: 88; implicitHeight: 88; radius: 44
                            color: Qt.alpha(ls.accent, 0.18)
                            border.width: 2
                            border.color: ls.accent
                            clip: true
                            Image {
                                id: avatar
                                anchors.fill: parent
                                anchors.margins: 2
                                source: ls.lc.user_avatar_path ? "file://" + ls.lc.user_avatar_path : "file://" + Quickshell.env("HOME") + "/.face"
                                fillMode: Image.PreserveAspectCrop
                                asynchronous: true
                                layer.enabled: true
                                layer.effect: MultiEffect { maskEnabled: true; maskSource: avatarMask }
                            }
                            Rectangle { id: avatarMask; anchors.fill: avatar; radius: width / 2; visible: false; layer.enabled: true }
                            Text {
                                anchors.centerIn: parent
                                visible: avatar.status !== Image.Ready
                                text: (Quickshell.env("USER") || "?").charAt(0).toUpperCase()
                                color: ls.accent
                                font.pixelSize: 36
                                font.bold: true
                            }
                        }
                        Text {
                            Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                            text: (ls.lc.custom_greeting && ls.lc.custom_greeting.length > 0 ? ls.lc.custom_greeting : ls.tr("welcome") + ", {user}").replace("{user}", Quickshell.env("USER") || "")
                            color: ls.fg
                            font.pixelSize: 18
                        }
                    }

                    // Wachtwoordveld met schud-animatie bij een fout
                    Rectangle {
                        id: field
                        Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                        implicitWidth: 320
                        implicitHeight: 52
                        radius: 26
                        color: Qt.alpha(ls.surface, 0.75)
                        border.width: input.activeFocus ? 2 : 1
                        border.color: ls.statusText === ls.tr("wrong") ? "#f38ba8" : (input.activeFocus ? ls.accent : Qt.alpha(ls.border, 0.7))
                        Behavior on border.color { ColorAnimation { duration: 150 } }
                        transform: Translate { id: shake }

                        SequentialAnimation {
                            id: shakeAnim
                            loops: 2
                            NumberAnimation { target: shake; property: "x"; to: -12; duration: 45 }
                            NumberAnimation { target: shake; property: "x"; to: 12; duration: 90 }
                            NumberAnimation { target: shake; property: "x"; to: 0; duration: 45 }
                        }
                        Connections {
                            target: ls
                            function onFailCountChanged() { shakeAnim.restart(); input.text = ""; }
                        }

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 20
                            anchors.rightMargin: 20
                            spacing: 10
                            Text { text: ls.busy ? "⏳" : "🔒"; font.pixelSize: 16 }
                            TextInput {
                                id: input
                                Layout.fillWidth: true
                                echoMode: TextInput.Password
                                passwordCharacter: "●"
                                color: ls.fg
                                font.pixelSize: 16
                                font.letterSpacing: 2
                                clip: true
                                enabled: !ls.busy
                                focus: true
                                Component.onCompleted: forceActiveFocus()
                                Keys.onReturnPressed: ls.submit(text)
                                Keys.onEnterPressed: ls.submit(text)
                                Keys.onEscapePressed: { if (view.preview) ls.isPreview = false; else text = ""; }
                                Text {
                                    visible: input.text.length === 0
                                    text: ls.tr("password")
                                    color: Qt.alpha(ls.fg, 0.4)
                                    font.pixelSize: 16
                                }
                            }
                        }
                    }

                    Text {
                        Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                        visible: ls.statusText.length > 0
                        text: ls.statusText
                        color: ls.statusText === ls.tr("checking") ? Qt.alpha(ls.fg, 0.6) : "#f38ba8"
                        font.pixelSize: 13
                    }

                    // Batterij (alleen op apparaten met een accu)
                    Text {
                        id: battery
                        property string level: ""
                        Layout.alignment: content.preset === "centered" ? Qt.AlignHCenter : Qt.AlignLeft
                        visible: ls.shows("battery_network") && level.length > 0
                        text: level
                        color: Qt.alpha(ls.fg, 0.7)
                        font.pixelSize: 13
                        Process {
                            id: batProc
                            command: ["bash", "-c", "for b in /sys/class/power_supply/BAT*; do [ -r \"$b/capacity\" ] && echo \"$(cat \"$b/capacity\") $(cat \"$b/status\")\" && break; done"]
                            stdout: StdioCollector {
                                onStreamFinished: {
                                    var p = this.text.trim().split(" ");
                                    battery.level = p.length >= 2 ? (p[1] === "Charging" ? "⚡ " : "🔋 ") + p[0] + "%" : "";
                                }
                            }
                        }
                        Timer { interval: 30000; running: true; repeat: true; triggeredOnStart: true; onTriggered: batProc.running = true }
                    }
                }
            }

            // Voorbeeld-badge
            Rectangle {
                visible: view.preview
                anchors.top: parent.top
                anchors.topMargin: 28
                anchors.horizontalCenter: parent.horizontalCenter
                implicitWidth: badge.implicitWidth + 28
                implicitHeight: 34
                radius: 17
                color: Qt.alpha(ls.surface, 0.9)
                border.width: 1
                border.color: ls.accent
                Text { id: badge; anchors.centerIn: parent; text: "👁 " + ls.tr("preview"); color: ls.fg; font.pixelSize: 13; font.bold: true }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: ls.isPreview = false }
            }
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("Osd.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Io

Scope {
    id: osdScope

    FileView {
        id: shellConfigFile
        path: Quickshell.env("HOME") + "/.config/quickshell/zenith-shell.json"
        watchChanges: true
    }

    readonly property var shellConfig: {
        try {
            var txt = shellConfigFile.text().trim();
            if (txt.length > 0) return JSON.parse(txt);
        } catch (e) {}
        return {};
    }

    readonly property var osdConfig: (osdScope.shellConfig && osdScope.shellConfig.osd) || {}
    readonly property bool isOsdEnabled: osdScope.osdConfig.enabled !== undefined ? osdScope.osdConfig.enabled : true
    readonly property string osdPos: osdScope.osdConfig.position || "bottom"
    readonly property string osdOrientation: osdScope.osdConfig.orientation || "horizontal"
    readonly property int osdWidth: osdScope.osdConfig.width || 260
    readonly property int osdHeight: osdScope.osdConfig.height || 48
    readonly property int osdTimeout: osdScope.osdConfig.timeout_ms || 2000
    readonly property bool showPercent: osdScope.osdConfig.show_percentage !== undefined ? osdScope.osdConfig.show_percentage : true
    readonly property bool showIcon: osdScope.osdConfig.show_icon !== undefined ? osdScope.osdConfig.show_icon : true
    readonly property color bgColor: (osdScope.shellConfig.styling && osdScope.shellConfig.styling.background) || "#1e1e2e"
    readonly property color accentColor: (osdScope.shellConfig.styling && osdScope.shellConfig.styling.accent) || "#89b4fa"
    readonly property color textColor: (osdScope.shellConfig.styling && osdScope.shellConfig.styling.text_color) || "#cdd6f4"

    property string currentIcon: "🔊"
    property string currentText: "50%"
    property real currentProgress: 0.5
    property real lastVol: -1.0
    property real lastBright: -1.0
    property bool isOsdActive: false

    // IPC Handler voor OSD aanroepen
    IpcHandler {
        target: "osd"
        function popup(icon: string, text: string, val: real) {
            if (!osdScope.isOsdEnabled) return;
            osdScope.currentIcon = icon || "🔊";
            osdScope.currentText = text || "";
            osdScope.currentProgress = Math.max(0.0, Math.min(1.0, val || 0.0));
            osdScope.isOsdActive = true;
            hideTimer.restart();
        }
        function triggerVolume() { volWatcher.running = true; }
        function triggerBrightness() { brightWatcher.running = true; }
    }

    Timer {
        id: hideTimer
        interval: osdScope.osdTimeout
        onTriggered: osdScope.isOsdActive = false
    }

    // WirePlumber watcher
    Process {
        id: volWatcher
        command: ["bash", "-c", "wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null || echo 'Volume: 0.50'"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var s = data.trim();
                var isMuted = s.indexOf("[MUTED]") !== -1;
                var match = s.match(/Volume:\s+([0-9.]+)/);
                if (match && match[1]) {
                    var v = parseFloat(match[1]);
                    if (osdScope.lastVol >= 0.0 && Math.abs(osdScope.lastVol - v) > 0.005) {
                        osdScope.popup(isMuted ? "🔇" : (v > 0.5 ? "🔊" : "🔉"), isMuted ? "Gedempt" : Math.round(v * 100) + "%", v);
                    }
                    osdScope.lastVol = v;
                }
            }
        }
    }
    Timer { interval: 600; running: osdScope.isOsdEnabled; repeat: true; onTriggered: volWatcher.running = true }

    // Brightnessctl watcher
    Process {
        id: brightWatcher
        command: ["bash", "-c", "brightnessctl -m 2>/dev/null | cut -d, -f4 | tr -d '%'"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var val = parseInt(data.trim());
                if (!isNaN(val)) {
                    var b = val / 100.0;
                    if (osdScope.lastBright >= 0.0 && Math.abs(osdScope.lastBright - b) > 0.01) {
                        osdScope.popup("☀️", val + "%", b);
                    }
                    osdScope.lastBright = b;
                }
            }
        }
    }
    Timer { interval: 800; running: osdScope.isOsdEnabled; repeat: true; onTriggered: brightWatcher.running = true }

    // OSD Floating Layer Window
    PanelWindow {
        id: osdWin
        visible: osdScope.isOsdActive || osdContainer.opacity > 0.01
        color: "transparent"
        implicitWidth: osdScope.osdOrientation === "vertical" ? osdScope.osdHeight : osdScope.osdWidth
        implicitHeight: osdScope.osdOrientation === "vertical" ? osdScope.osdWidth : osdScope.osdHeight

        anchors {
            top: osdScope.osdPos.indexOf("top") !== -1
            bottom: osdScope.osdPos.indexOf("bottom") !== -1
            right: osdScope.osdPos.indexOf("right") !== -1
            left: osdScope.osdPos.indexOf("left") !== -1
        }
        margins {
            top: 40
            bottom: 40
            left: 30
            right: 30
        }
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        Rectangle {
            id: osdContainer
            anchors.fill: parent
            radius: parent.height / 2
            color: Qt.alpha(osdScope.bgColor, 0.94)
            border.width: 1
            border.color: Qt.alpha(osdScope.accentColor, 0.4)

            opacity: osdScope.isOsdActive ? 1.0 : 0.0
            scale: osdScope.isOsdActive ? 1.0 : 0.90

            Behavior on opacity {
                NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
            }
            Behavior on scale {
                NumberAnimation { duration: 240; easing.type: Easing.OutBack; easing.overshoot: 1.25 }
            }

            // Horizontale Layout
            RowLayout {
                visible: osdScope.osdOrientation !== "vertical"
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 16
                spacing: 12

                Text {
                    visible: osdScope.showIcon
                    text: osdScope.currentIcon
                    font.pixelSize: 16
                    color: osdScope.accentColor
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 8
                    radius: 4
                    color: "#313244"
                    Rectangle {
                        height: parent.height
                        width: parent.width * osdScope.currentProgress
                        radius: 4
                        color: osdScope.accentColor
                        Behavior on width {
                            NumberAnimation { duration: 140; easing.type: Easing.OutCubic }
                        }
                    }
                }

                Text {
                    visible: osdScope.showPercent
                    text: osdScope.currentText
                    font.bold: true
                    font.pixelSize: 11
                    color: osdScope.textColor
                }
            }

            // Verticale Layout
            ColumnLayout {
                visible: osdScope.osdOrientation === "vertical"
                anchors.fill: parent
                anchors.topMargin: 14
                anchors.bottomMargin: 14
                spacing: 10

                Text {
                    visible: osdScope.showIcon
                    text: osdScope.currentIcon
                    font.pixelSize: 16
                    Layout.alignment: Qt.AlignHCenter
                    color: osdScope.accentColor
                }

                Rectangle {
                    Layout.fillHeight: true
                    width: 8
                    radius: 4
                    color: "#313244"
                    Layout.alignment: Qt.AlignHCenter
                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: parent.height * osdScope.currentProgress
                        radius: 4
                        color: osdScope.accentColor
                        Behavior on height {
                            NumberAnimation { duration: 140; easing.type: Easing.OutCubic }
                        }
                    }
                }

                Text {
                    visible: osdScope.showPercent
                    text: osdScope.currentText
                    font.bold: true
                    font.pixelSize: 10
                    Layout.alignment: Qt.AlignHCenter
                    color: osdScope.textColor
                }
            }
        }
    }
}
"###,
    );
}

fn ensure_default_cards(dir: &Path) {
    ensure_file_exists(
        &dir.join("wifi_toggle.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 52
    radius: 10
    color: isEnabled ? Qt.alpha("#89b4fa", 0.18) : Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: isEnabled ? "#89b4fa" : "#45475a"

    property bool isEnabled: false
    property string ssid: "Wi-Fi"

    Process {
        id: wifiStatus
        command: ["bash", "-c", "nmcli radio wifi && nmcli -t -f active,ssid dev wifi | grep '^yes' | cut -d: -f2"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var lines = data.trim().split("\n");
                if (lines.length > 0) cardRoot.isEnabled = (lines[0].trim() === "enabled");
                if (lines.length > 1 && lines[1].trim().length > 0) cardRoot.ssid = lines[1].trim();
                else cardRoot.ssid = cardRoot.isEnabled ? "Verbonden" : "Uitgeschakeld";
            }
        }
    }
    Timer { interval: 4000; running: true; repeat: true; onTriggered: wifiStatus.running = true }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        Text {
            text: cardRoot.isEnabled ? "📶" : "󰤭"
            font.pixelSize: 18
            color: cardRoot.isEnabled ? "#89b4fa" : "#a6adc8"
        }

        ColumnLayout {
            spacing: 2
            Text {
                text: "Wi-Fi"
                font.bold: true
                font.pixelSize: 12
                color: "#cdd6f4"
            }
            Text {
                text: cardRoot.ssid
                font.pixelSize: 10
                color: "#a6adc8"
                elide: Text.ElideRight
                Layout.maximumWidth: 200
            }
        }

        Item { Layout.fillWidth: true }

        Rectangle {
            width: 38
            height: 20
            radius: 10
            color: cardRoot.isEnabled ? "#89b4fa" : "#45475a"
            Rectangle {
                width: 16
                height: 16
                radius: 8
                color: "#1e1e2e"
                anchors.verticalCenter: parent.verticalCenter
                x: cardRoot.isEnabled ? 20 : 2
                Behavior on x { NumberAnimation { duration: 150 } }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            var cmd = cardRoot.isEnabled ? "nmcli radio wifi off" : "nmcli radio wifi on";
            toggleProc.command = ["bash", "-c", cmd];
            toggleProc.running = true;
            cardRoot.isEnabled = !cardRoot.isEnabled;
        }
    }
    Process { id: toggleProc; onExited: wifiStatus.running = true }
}
"###,
    );

    ensure_file_exists(
        &dir.join("bluetooth_toggle.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 52
    radius: 10
    color: isEnabled ? Qt.alpha("#89b4fa", 0.18) : Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: isEnabled ? "#89b4fa" : "#45475a"

    property bool isEnabled: false
    property string deviceName: "Bluetooth"

    Process {
        id: btStatus
        command: ["bash", "-c", "bluetoothctl show | grep 'Powered: yes' >/dev/null && echo 'on' || echo 'off'; bluetoothctl info 2>/dev/null | grep 'Name:' | cut -d' ' -f2-"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var lines = data.trim().split("\n");
                if (lines.length > 0) cardRoot.isEnabled = (lines[0].trim() === "on");
                if (lines.length > 1 && lines[1].trim().length > 0) cardRoot.deviceName = lines[1].trim();
                else cardRoot.deviceName = cardRoot.isEnabled ? "Ingeschakeld" : "Uitgeschakeld";
            }
        }
    }
    Timer { interval: 4000; running: true; repeat: true; onTriggered: btStatus.running = true }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        Text {
            text: "ᛒ"
            font.pixelSize: 18
            color: cardRoot.isEnabled ? "#89b4fa" : "#a6adc8"
        }

        ColumnLayout {
            spacing: 2
            Text {
                text: "Bluetooth"
                font.bold: true
                font.pixelSize: 12
                color: "#cdd6f4"
            }
            Text {
                text: cardRoot.deviceName
                font.pixelSize: 10
                color: "#a6adc8"
                elide: Text.ElideRight
                Layout.maximumWidth: 200
            }
        }

        Item { Layout.fillWidth: true }

        Rectangle {
            width: 38
            height: 20
            radius: 10
            color: cardRoot.isEnabled ? "#89b4fa" : "#45475a"
            Rectangle {
                width: 16
                height: 16
                radius: 8
                color: "#1e1e2e"
                anchors.verticalCenter: parent.verticalCenter
                x: cardRoot.isEnabled ? 20 : 2
                Behavior on x { NumberAnimation { duration: 150 } }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            var cmd = cardRoot.isEnabled ? "bluetoothctl power off" : "bluetoothctl power on";
            toggleProc.command = ["bash", "-c", cmd];
            toggleProc.running = true;
            cardRoot.isEnabled = !cardRoot.isEnabled;
        }
    }
    Process { id: toggleProc; onExited: btStatus.running = true }
}
"###,
    );

    ensure_file_exists(
        &dir.join("dnd_toggle.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 52
    radius: 10
    color: isDnd ? Qt.alpha("#f38ba8", 0.18) : Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: isDnd ? "#f38ba8" : "#45475a"

    property bool isDnd: false

    Process {
        id: dndStatus
        command: ["bash", "-c", "dunstctl is-paused 2>/dev/null || echo 'false'"]
        running: true
        stdout: SplitParser {
            onRead: data => { cardRoot.isDnd = (data.trim() === "true"); }
        }
    }
    Timer { interval: 3000; running: true; repeat: true; onTriggered: dndStatus.running = true }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        Text {
            text: cardRoot.isDnd ? "🔕" : "🔔"
            font.pixelSize: 18
            color: cardRoot.isDnd ? "#f38ba8" : "#cdd6f4"
        }

        ColumnLayout {
            spacing: 2
            Text {
                text: "Niet Storen (DND)"
                font.bold: true
                font.pixelSize: 12
                color: "#cdd6f4"
            }
            Text {
                text: cardRoot.isDnd ? "Notificaties gedempt" : "Notificaties actief"
                font.pixelSize: 10
                color: "#a6adc8"
            }
        }

        Item { Layout.fillWidth: true }

        Rectangle {
            width: 38
            height: 20
            radius: 10
            color: cardRoot.isDnd ? "#f38ba8" : "#45475a"
            Rectangle {
                width: 16
                height: 16
                radius: 8
                color: "#1e1e2e"
                anchors.verticalCenter: parent.verticalCenter
                x: cardRoot.isDnd ? 20 : 2
                Behavior on x { NumberAnimation { duration: 150 } }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            toggleProc.command = ["bash", "-c", "dunstctl set-paused toggle"];
            toggleProc.running = true;
            cardRoot.isDnd = !cardRoot.isDnd;
        }
    }
    Process { id: toggleProc; onExited: dndStatus.running = true }
}
"###,
    );

    ensure_file_exists(
        &dir.join("nightlight_toggle.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 52
    radius: 10
    color: isNight ? Qt.alpha("#fab387", 0.18) : Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: isNight ? "#fab387" : "#45475a"

    property bool isNight: false

    Process {
        id: nightStatus
        command: ["bash", "-c", "hyprshade current 2>/dev/null || echo 'off'"]
        running: true
        stdout: SplitParser {
            onRead: data => { cardRoot.isNight = (data.trim().length > 0 && data.trim() !== "off"); }
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        Text {
            text: "🌙"
            font.pixelSize: 18
            color: cardRoot.isNight ? "#fab387" : "#cdd6f4"
        }

        ColumnLayout {
            spacing: 2
            Text {
                text: "Nachtmodus"
                font.bold: true
                font.pixelSize: 12
                color: "#cdd6f4"
            }
            Text {
                text: cardRoot.isNight ? "Blauwfilter actief" : "Standaard kleuren"
                font.pixelSize: 10
                color: "#a6adc8"
            }
        }

        Item { Layout.fillWidth: true }

        Rectangle {
            width: 38
            height: 20
            radius: 10
            color: cardRoot.isNight ? "#fab387" : "#45475a"
            Rectangle {
                width: 16
                height: 16
                radius: 8
                color: "#1e1e2e"
                anchors.verticalCenter: parent.verticalCenter
                x: cardRoot.isNight ? 20 : 2
                Behavior on x { NumberAnimation { duration: 150 } }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            var cmd = cardRoot.isNight ? "hyprshade off 2>/dev/null || true" : "hyprshade on blue-light-filter 2>/dev/null || true";
            toggleProc.command = ["bash", "-c", cmd];
            toggleProc.running = true;
            cardRoot.isNight = !cardRoot.isNight;
        }
    }
    Process { id: toggleProc }
}
"###,
    );

    ensure_file_exists(
        &dir.join("volume_slider.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 64
    radius: 10
    color: Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: "#45475a"

    property real volumeLevel: 0.5
    property bool isMuted: false

    Process {
        id: volProc
        command: ["bash", "-c", "wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null || echo 'Volume: 0.50'"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var s = data.trim();
                cardRoot.isMuted = s.indexOf("[MUTED]") !== -1;
                var match = s.match(/Volume:\s+([0-9.]+)/);
                if (match && match[1]) {
                    cardRoot.volumeLevel = Math.min(1.0, parseFloat(match[1]));
                }
            }
        }
    }
    Timer { interval: 2000; running: true; repeat: true; onTriggered: volProc.running = true }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 6

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: cardRoot.isMuted ? "🔇" : (cardRoot.volumeLevel > 0.5 ? "🔊" : "🔉")
                font.pixelSize: 14
                color: cardRoot.isMuted ? "#f38ba8" : "#89b4fa"
            }
            Text {
                text: "Luidspreker Volume"
                font.bold: true
                font.pixelSize: 11
                color: "#cdd6f4"
            }
            Item { Layout.fillWidth: true }
            Text {
                text: cardRoot.isMuted ? "Gedempt" : Math.round(cardRoot.volumeLevel * 100) + "%"
                font.pixelSize: 11
                font.bold: true
                color: cardRoot.isMuted ? "#f38ba8" : "#89b4fa"
            }
        }

        // Custom Slider Track
        Rectangle {
            id: track
            Layout.fillWidth: true
            height: 12
            radius: 6
            color: "#313244"

            Rectangle {
                height: parent.height
                width: parent.width * cardRoot.volumeLevel
                radius: 6
                color: cardRoot.isMuted ? "#585b70" : "#89b4fa"
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                preventStealing: true
                function updatePos(mouse) {
                    var newLvl = Math.max(0.0, Math.min(1.0, mouse.x / track.width));
                    cardRoot.volumeLevel = newLvl;
                    setProc.command = ["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", newLvl.toFixed(2)];
                    setProc.running = true;
                }
                onPressed: mouse => updatePos(mouse)
                onPositionChanged: mouse => updatePos(mouse)
            }
        }
    }
    Process { id: setProc }
}
"###,
    );

    ensure_file_exists(
        &dir.join("mic_slider.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 64
    radius: 10
    color: Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: "#45475a"

    property real micLevel: 0.5
    property bool isMuted: false

    Process {
        id: micProc
        command: ["bash", "-c", "wpctl get-volume @DEFAULT_AUDIO_SOURCE@ 2>/dev/null || echo 'Volume: 0.50'"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var s = data.trim();
                cardRoot.isMuted = s.indexOf("[MUTED]") !== -1;
                var match = s.match(/Volume:\s+([0-9.]+)/);
                if (match && match[1]) {
                    cardRoot.micLevel = Math.min(1.0, parseFloat(match[1]));
                }
            }
        }
    }
    Timer { interval: 2500; running: true; repeat: true; onTriggered: micProc.running = true }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 6

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: cardRoot.isMuted ? "🎙️❌" : "🎙️"
                font.pixelSize: 14
                color: cardRoot.isMuted ? "#f38ba8" : "#a6e3a1"
            }
            Text {
                text: "Microfoon Ingang"
                font.bold: true
                font.pixelSize: 11
                color: "#cdd6f4"
            }
            Item { Layout.fillWidth: true }
            Text {
                text: cardRoot.isMuted ? "Gedempt" : Math.round(cardRoot.micLevel * 100) + "%"
                font.pixelSize: 11
                font.bold: true
                color: cardRoot.isMuted ? "#f38ba8" : "#a6e3a1"
            }
        }

        // Custom Slider Track
        Rectangle {
            id: track
            Layout.fillWidth: true
            height: 12
            radius: 6
            color: "#313244"

            Rectangle {
                height: parent.height
                width: parent.width * cardRoot.micLevel
                radius: 6
                color: cardRoot.isMuted ? "#585b70" : "#a6e3a1"
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                preventStealing: true
                function updatePos(mouse) {
                    var newLvl = Math.max(0.0, Math.min(1.0, mouse.x / track.width));
                    cardRoot.micLevel = newLvl;
                    setProc.command = ["wpctl", "set-volume", "@DEFAULT_AUDIO_SOURCE@", newLvl.toFixed(2)];
                    setProc.running = true;
                }
                onPressed: mouse => updatePos(mouse)
                onPositionChanged: mouse => updatePos(mouse)
            }
        }
    }
    Process { id: setProc }
}
"###,
    );

    ensure_file_exists(
        &dir.join("brightness_slider.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 64
    radius: 10
    color: Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: "#45475a"

    property real brightnessLevel: 0.5

    Process {
        id: brightProc
        command: ["bash", "-c", "brightnessctl -m 2>/dev/null | cut -d, -f4 | tr -d '%'"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var val = parseInt(data.trim());
                if (!isNaN(val)) cardRoot.brightnessLevel = Math.max(0.05, Math.min(1.0, val / 100.0));
            }
        }
    }
    Timer { interval: 3000; running: true; repeat: true; onTriggered: brightProc.running = true }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 6

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "☀️"
                font.pixelSize: 14
                color: "#f9e2af"
            }
            Text {
                text: "Schermhelderheid"
                font.bold: true
                font.pixelSize: 11
                color: "#cdd6f4"
            }
            Item { Layout.fillWidth: true }
            Text {
                text: Math.round(cardRoot.brightnessLevel * 100) + "%"
                font.pixelSize: 11
                font.bold: true
                color: "#f9e2af"
            }
        }

        // Custom Slider Track
        Rectangle {
            id: track
            Layout.fillWidth: true
            height: 12
            radius: 6
            color: "#313244"

            Rectangle {
                height: parent.height
                width: parent.width * cardRoot.brightnessLevel
                radius: 6
                color: "#f9e2af"
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                preventStealing: true
                function updatePos(mouse) {
                    var newLvl = Math.max(0.05, Math.min(1.0, mouse.x / track.width));
                    cardRoot.brightnessLevel = newLvl;
                    setProc.command = ["brightnessctl", "s", Math.round(newLvl * 100) + "%"];
                    setProc.running = true;
                }
                onPressed: mouse => updatePos(mouse)
                onPositionChanged: mouse => updatePos(mouse)
            }
        }
    }
    Process { id: setProc }
}
"###,
    );

    ensure_file_exists(
        &dir.join("mpris_card.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 86
    radius: 12
    color: Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: "#45475a"

    property string title: "Geen media actief"
    property string artist: "Antigravity Zenith"
    property string status: "Stopped"

    Process {
        id: mprisProc
        command: ["bash", "-c", "playerctl metadata --format '{{title}};;{{artist}};;{{status}}' 2>/dev/null || echo 'Geen media;;-- ;;Stopped'"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var p = data.trim().split(";;");
                if (p.length >= 3) {
                    cardRoot.title = p[0].length > 32 ? p[0].substring(0, 30) + "..." : p[0];
                    cardRoot.artist = p[1].length > 25 ? p[1].substring(0, 23) + "..." : p[1];
                    cardRoot.status = p[2];
                }
            }
        }
    }
    Timer { interval: 1500; running: true; repeat: true; onTriggered: mprisProc.running = true }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 12

        Rectangle {
            width: 48
            height: 48
            radius: 8
            color: Qt.alpha("#89b4fa", 0.2)
            Text {
                anchors.centerIn: parent
                text: "🎵"
                font.pixelSize: 22
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            Text {
                text: cardRoot.title
                font.bold: true
                font.pixelSize: 12
                color: "#cdd6f4"
                elide: Text.ElideRight
            }
            Text {
                text: cardRoot.artist
                font.pixelSize: 10
                color: "#a6adc8"
                elide: Text.ElideRight
            }
            RowLayout {
                spacing: 14
                Text {
                    text: "⏮"
                    font.pixelSize: 14
                    color: "#cdd6f4"
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { cmdProc.command = ["playerctl", "previous"]; cmdProc.running = true; }
                    }
                }
                Text {
                    text: cardRoot.status === "Playing" ? "⏸" : "▶"
                    font.pixelSize: 16
                    color: "#89b4fa"
                    font.bold: true
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { cmdProc.command = ["playerctl", "play-pause"]; cmdProc.running = true; }
                    }
                }
                Text {
                    text: "⏭"
                    font.pixelSize: 14
                    color: "#cdd6f4"
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { cmdProc.command = ["playerctl", "next"]; cmdProc.running = true; }
                    }
                }
            }
        }
    }
    Process { id: cmdProc; onExited: mprisProc.running = true }
}
"###,
    );

    ensure_file_exists(
        &dir.join("battery_card.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 56
    radius: 10
    color: Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: "#45475a"

    property int percent: 100
    property string status: "Discharging"

    Process {
        id: batProc
        command: ["bash", "-c", "cat /sys/class/power_supply/BAT*/capacity 2>/dev/null | head -n1; cat /sys/class/power_supply/BAT*/status 2>/dev/null | head -n1"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                var lines = data.trim().split("\n");
                if (lines.length > 0 && lines[0].length > 0) {
                    var p = parseInt(lines[0]);
                    if (!isNaN(p)) cardRoot.percent = p;
                }
                if (lines.length > 1 && lines[1].length > 0) {
                    cardRoot.status = lines[1].trim();
                }
            }
        }
    }
    Timer { interval: 8000; running: true; repeat: true; onTriggered: batProc.running = true }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        Text {
            text: cardRoot.status === "Charging" ? "⚡" : (cardRoot.percent > 20 ? "🔋" : "🪫")
            font.pixelSize: 18
            color: cardRoot.percent > 20 ? "#a6e3a1" : "#f38ba8"
        }

        ColumnLayout {
            spacing: 2
            Text {
                text: "Batterij & Energie"
                font.bold: true
                font.pixelSize: 12
                color: "#cdd6f4"
            }
            Text {
                text: cardRoot.status === "Charging" ? "Opladen (" + cardRoot.percent + "%)" : "Resterend: " + cardRoot.percent + "%"
                font.pixelSize: 10
                color: "#a6adc8"
            }
        }

        Item { Layout.fillWidth: true }

        Rectangle {
            width: 48
            height: 18
            radius: 4
            color: "#313244"
            border.width: 1
            border.color: "#45475a"
            Rectangle {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.margins: 2
                width: (parent.width - 4) * Math.min(1.0, cardRoot.percent / 100.0)
                radius: 2
                color: cardRoot.percent > 20 ? "#a6e3a1" : "#f38ba8"
            }
        }
    }
}
"###,
    );

    ensure_file_exists(
        &dir.join("power_strip.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 48
    radius: 10
    color: Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: "#45475a"

    RowLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 8

        component PowerButton: Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: 8
            color: btnArea.containsMouse ? Qt.alpha("#89b4fa", 0.25) : Qt.alpha("#cdd6f4", 0.06)
            property string icon: ""
            property string tip: ""
            property var action: null

            Text {
                anchors.centerIn: parent
                text: parent.icon
                font.pixelSize: 14
            }
            MouseArea {
                id: btnArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: if (action) action()
            }
        }

        PowerButton {
            icon: "🔒"
            tip: "Vergrendel scherm"
            action: () => { pProc.command = ["bash", "-c", "hyprlock || swaylock || loginctl lock-session"]; pProc.running = true; }
        }
        PowerButton {
            icon: "💤"
            tip: "Slaapstand"
            action: () => { pProc.command = ["systemctl", "suspend"]; pProc.running = true; }
        }
        PowerButton {
            icon: "🚪"
            tip: "Afmelden"
            action: () => { pProc.command = ["hyprctl", "dispatch", "exit"]; pProc.running = true; }
        }
        PowerButton {
            icon: "🔄"
            tip: "Herstarten"
            action: () => { pProc.command = ["systemctl", "reboot"]; pProc.running = true; }
        }
        PowerButton {
            icon: "⏻"
            tip: "Afsluiten"
            action: () => { pProc.command = ["systemctl", "poweroff"]; pProc.running = true; }
        }
    }
    Process { id: pProc }
}
"###,
    );

    ensure_file_exists(
        &dir.join("script_card.qml"),
        r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Rectangle {
    id: cardRoot
    Layout.fillWidth: true
    implicitHeight: 52
    radius: 10
    color: Qt.alpha("#cdd6f4", 0.08)
    border.width: 1
    border.color: "#45475a"

    property string scriptCommand: ""
    property int intervalSec: 10
    property string scriptIcon: "💻"
    property string onClickCmd: ""
    property string cardName: "Custom Script"
    property string liveOutput: "Laden..."

    Process {
        id: proc
        command: ["bash", "-c", cardRoot.scriptCommand]
        running: cardRoot.scriptCommand.length > 0
        stdout: SplitParser {
            onRead: data => { cardRoot.liveOutput = data.trim(); }
        }
    }
    Timer {
        interval: Math.max(1000, cardRoot.intervalSec * 1000)
        running: cardRoot.scriptCommand.length > 0
        repeat: true
        onTriggered: proc.running = true
    }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        Text {
            text: cardRoot.scriptIcon
            font.pixelSize: 18
        }

        ColumnLayout {
            spacing: 2
            Text {
                text: cardRoot.cardName
                font.bold: true
                font.pixelSize: 12
                color: "#cdd6f4"
            }
            Text {
                text: cardRoot.liveOutput
                font.pixelSize: 10
                color: "#89b4fa"
                elide: Text.ElideRight
                Layout.maximumWidth: 260
            }
        }

        Item { Layout.fillWidth: true }

        Text {
            visible: cardRoot.onClickCmd.length > 0
            text: "➔"
            font.pixelSize: 12
            color: "#6c7086"
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: cardRoot.onClickCmd.length > 0 ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: {
            if (cardRoot.onClickCmd.length > 0) {
                clickProc.command = ["bash", "-c", cardRoot.onClickCmd];
                clickProc.running = true;
            }
        }
    }
    Process { id: clickProc }
}
"###,
    );
}

fn ensure_waybar_config(home: &Path) {
    let wb_dir = home.join(".config/waybar");
    let config_path = wb_dir.join("config");

    if !config_path.exists() {
        let _ = create_dir_all(&wb_dir);
        let default_config = r#"{
    "layer": "top",
    "position": "top",
    "height": 32,
    "modules-left": ["hyprland/workspaces"],
    "modules-center": ["clock"],
    "modules-right": ["cpu", "battery", "pulseaudio", "bluetooth", "network", "tray"],
    "clock": {
        "format": "{:%a %d %b  %H:%M}",
        "tooltip-format": "{:%A %d %B %Y  %H:%M:%S}"
    },
    "battery": {
        "format": "{icon}  {capacity}%",
        "format-charging": "⚡ {capacity}%",
        "format-icons": ["🪫", "🪫", "🔋", "🔋", "🔋"],
        "interval": 30
    },
    "cpu": {
        "format": "🖥 {usage}%",
        "interval": 3
    },
    "pulseaudio": {
        "format": "🔊 {volume}%",
        "format-muted": "🔇 muted",
        "on-click": "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"
    },
    "bluetooth": {
        "format": " {status}",
        "format-connected": " {device_alias}",
        "format-disabled": " uit",
        "on-click": "bluetoothctl power toggle"
    },
    "network": {
        "format-wifi": "🌐 {essid}",
        "format-ethernet": "🌐 Ethernet",
        "format-disconnected": "🌐 –",
        "tooltip-format": "{ifname}: {ipaddr}"
    },
    "tray": {
        "spacing": 8
    }
}
"#;
        if let Ok(mut f) = File::create(&config_path) {
            let _ = f.write_all(default_config.as_bytes());
        }
    }

    // Garandeer ook een basis style.css
    let style_path = wb_dir.join("style.css");
    if !style_path.exists() {
        let default_style = r#"@import "zenith-style.css";

* {
    font-family: "JetBrainsMono Nerd Font", "Noto Sans", sans-serif;
    font-size: 13px;
}

window#waybar {
    background-color: rgba(30, 30, 46, 0.90);
    color: #cdd6f4;
}

#workspaces button {
    padding: 0 5px;
    color: #cdd6f4;
    border-radius: 6px;
}

#workspaces button.active {
    background-color: rgba(137, 180, 250, 0.25);
    color: #89b4fa;
}

#clock, #battery, #cpu, #pulseaudio, #bluetooth, #network, #tray {
    padding: 0 10px;
}
"#;
        if let Ok(mut f) = File::create(&style_path) {
            let _ = f.write_all(default_style.as_bytes());
        }
    }
}

fn ensure_hyprland_source(home: &Path) {
    let hypr_dir = home.join(".config/hypr");
    let _ = create_dir_all(&hypr_dir);
    let hypr_conf = hypr_dir.join("hyprland.conf");
    let zenith_conf = hypr_dir.join("zenith.conf");

    if !zenith_conf.exists() {
        let initial_cfg = crate::backend::config::load_config();
        crate::backend::config::save_config(&initial_cfg);
    } else if let Ok(content) = read_to_string(&zenith_conf) {
        if !content.contains("org.zenith.control") {
            let initial_cfg = crate::backend::config::load_config();
            crate::backend::config::save_config(&initial_cfg);
        }
    }

    // Hyprland >= 0.55 met Lua-config: zenith.lua laden via pcall, zodat een fout erin
    // nooit de rest van de gebruikersconfig breekt.
    let hypr_lua = hypr_dir.join("hyprland.lua");
    if hypr_lua.exists() {
        // Ontbrekend of verouderd (laadt de sneltoetsen nog niet): opnieuw genereren.
        let up_to_date = read_to_string(hypr_dir.join("zenith.lua")).map(|c| c.contains("zenith-keybinds.lua")).unwrap_or(false);
        if !up_to_date {
            crate::backend::config::save_config(&crate::backend::config::load_config());
        }
        if let Ok(content) = read_to_string(&hypr_lua) {
            if !content.contains("zenith.lua") {
                if let Ok(mut file) = OpenOptions::new().append(true).open(&hypr_lua) {
                    let _ = writeln!(file, "\n-- Injected by Zenith Control\npcall(dofile, os.getenv(\"HOME\") .. \"/.config/hypr/zenith.lua\")");
                }
            }
        }
    }

    let source_line = "source = ~/.config/hypr/zenith.conf";
    if hypr_conf.exists() {
        if let Ok(content) = read_to_string(&hypr_conf) {
            if !content.contains(source_line) {
                if let Ok(mut file) = OpenOptions::new().append(true).open(&hypr_conf) {
                    let _ = writeln!(file, "\n# Injected by Zenith Control\n{}", source_line);
                }
            }
        }
    } else {
        let default_hypr = format!(
            "# Hyprland Configuration with Zenith Control\n\
            $mainMod = SUPER\n\
            bind = $mainMod, Q, exec, kitty\n\
            bind = $mainMod, C, killactive,\n\
            bind = $mainMod, M, exit,\n\
            bind = $mainMod, V, togglefloating,\n\
            bind = $mainMod, R, exec, rofi -show drun || wofi --show drun\n\n\
            # Injected by Zenith Control\n\
            {}\n",
            source_line
        );
        let _ = std::fs::write(&hypr_conf, default_hypr);
    }
}

fn ensure_kitty_import(home: &Path) {
    let kitty_conf = home.join(".config/kitty/kitty.conf");
    if kitty_conf.exists() {
        if let Ok(content) = read_to_string(&kitty_conf) {
            let import_line = "include ./zenith-theme.conf";
            if !content.contains(import_line) {
                if let Ok(mut file) = OpenOptions::new().append(true).open(&kitty_conf) {
                    let _ = writeln!(file, "\n# Injected by Zenith Control\n{}", import_line);
                }
            }
        }
    }
}

fn ensure_waybar_import(home: &Path) {
    let waybar_style = home.join(".config/waybar/style.css");
    if waybar_style.exists() {
        if let Ok(content) = read_to_string(&waybar_style) {
            let import_line = "@import \"zenith-style.css\";";
            if !content.contains(import_line) {
                let new_content = format!("{}\n{}", import_line, content);
                let _ = std::fs::write(&waybar_style, new_content);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ensure_zenith_panel;

    #[test]
    fn zenith_panels_update_safely() {
        let dir = std::env::temp_dir().join(format!("zenith-panels-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("Media.qml");

        ensure_zenith_panel(&p, "// zenith-panel v1");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "// zenith-panel v1");

        // Nieuwere Zenith-versie: bijwerken met back-up
        ensure_zenith_panel(&p, "// zenith-panel v2");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "// zenith-panel v2");
        assert_eq!(std::fs::read_to_string(dir.join("Media.qml.bak")).unwrap(), "// zenith-panel v1");

        // Oudere versie zonder markering maar met ZenithPopup telt ook als van Zenith
        std::fs::write(&p, "ZenithPopup { }").unwrap();
        ensure_zenith_panel(&p, "// zenith-panel v3");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "// zenith-panel v3");

        // Oude ZenithPopup.qml (zonder markering) wordt herkend aan zijn kopregel
        let popup = dir.join("ZenithPopup.qml");
        std::fs::write(&popup, "// Gedeeld pop-uppaneel van Zenith: oud").unwrap();
        ensure_zenith_panel(&popup, "// zenith-panel nieuw");
        assert_eq!(std::fs::read_to_string(&popup).unwrap(), "// zenith-panel nieuw");

        // Oude lockscreen met onveilige wachtwoord-terugval wordt vervangen
        let lock = dir.join("Lockscreen.qml");
        std::fs::write(&lock, "// Native PAM authenticatie\nsu -c").unwrap();
        ensure_zenith_panel(&lock, "// zenith-panel lock");
        assert_eq!(std::fs::read_to_string(&lock).unwrap(), "// zenith-panel lock");

        // Eigen bestand van de gebruiker: nooit overschrijven
        std::fs::write(&p, "Rectangle { color: \"red\" }").unwrap();
        ensure_zenith_panel(&p, "// zenith-panel v4");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "Rectangle { color: \"red\" }");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
