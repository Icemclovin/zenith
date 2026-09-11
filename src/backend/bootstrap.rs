use std::fs::{create_dir_all, read_to_string, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

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

fn ensure_file_exists(path: &PathBuf, default_content: &str) {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }
        if let Ok(mut f) = File::create(path) {
            let _ = f.write_all(default_content.as_bytes());
        }
    }
}

fn ensure_quickshell_config(home: &PathBuf) {
    let qs_dir = home.join(".config/quickshell");
    let modules_dir = qs_dir.join("modules");
    let _ = create_dir_all(&modules_dir);

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
  "custom": {
    "raw_override": false,
    "custom_qml_path": null
  },
  "custom_scripts": []
}"###;
        ensure_file_exists(&shell_json_path, default_shell_json);
    }

    // 2. Garandeer modulaire shell.qml
    let shell_qml = qs_dir.join("shell.qml");
    if !shell_qml.exists() {
        let modular_shell_qml = r###"import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Io
import Quickshell.Hyprland

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
"###;
        ensure_file_exists(&shell_qml, modular_shell_qml);
    }

    // 3. Garandeer standaard modules in ~/.config/quickshell/modules/
    ensure_default_modules(&modules_dir);
}

fn ensure_default_modules(dir: &PathBuf) {
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
            required property var modelData
            width: 26; height: 22; radius: 6

            property bool isActive: (Hyprland.focusedMonitor && Hyprland.focusedMonitor.activeWorkspace)
                ? modelData.id === Hyprland.focusedMonitor.activeWorkspace.id
                : false

            color: isActive
                ? Qt.alpha(root.themeAccent || "#89b4fa", 0.30)
                : Qt.alpha(root.fgColor || "#cdd6f4", 0.05)

            border.color: isActive
                ? (root.themeAccent || "#89b4fa")
                : (root.borderColor || "#45475a")
            border.width: 1

            Text {
                anchors.centerIn: parent
                text: modelData.name || modelData.id
                color: isActive ? (root.themeAccent || "#89b4fa") : (root.fgColor || "#cdd6f4")
                font.pixelSize: Math.max(9, (root.userFontSize || 11) - 1)
                font.bold: isActive
            }

            MouseArea {
                anchors.fill: parent
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

    Process { id: qsLauncherProc; command: ["bash", "-c", "rofi -show drun || wofi --show drun"] }

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

fn ensure_waybar_config(home: &PathBuf) {
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

fn ensure_hyprland_source(home: &PathBuf) {
    let hypr_conf = home.join(".config/hypr/hyprland.conf");
    let zenith_conf = home.join(".config/hypr/zenith.conf");

    if !zenith_conf.exists() {
        ensure_file_exists(&zenith_conf, "# Generated by Zenith Control\n");
    }

    if let Ok(content) = read_to_string(&hypr_conf) {
        let source_line = "source = ~/.config/hypr/zenith.conf";
        if !content.contains(source_line) {
            if let Ok(mut file) = OpenOptions::new().append(true).open(&hypr_conf) {
                let _ = writeln!(file, "\n# Injected by Zenith Control\n{}", source_line);
            }
        }
    }
}

fn ensure_kitty_import(home: &PathBuf) {
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

fn ensure_waybar_import(home: &PathBuf) {
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