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
    }

    // Helper component voor het dynamisch inladen van modules (standaard of script)
    component ModuleLoader: Loader {
        id: mLoader
        required property var modelData

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
}
"###;
    if shell_qml.exists() {
        if let Ok(content) = read_to_string(&shell_qml) {
            // Als het bestaande bestand verouderd is (mist ModuleLoader, FileView, Scope of Panels)
            if !content.contains("ModuleLoader") || !content.contains("FileView") || !content.contains("Panels.ControlCenter") || !content.contains("Scope") {
                let backup = qs_dir.join("shell.qml.old");
                let _ = std::fs::copy(&shell_qml, &backup);
                let _ = std::fs::write(&shell_qml, modular_shell_qml);
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
    ensure_file_exists(
        &dir.join("ControlCenter.qml"),
        r###"import QtQuick
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
                NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
            }
            Behavior on scale {
                NumberAnimation { duration: 260; easing.type: Easing.OutBack; easing.overshoot: 1.18 }
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
                                text: "✕"
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
                                            item.scriptIcon = s.icon || "💻";
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