// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick
import "../components"
import QtQuick.Layouts
import QtQuick.Shapes
import QtQuick.Effects
import Quickshell
import Quickshell.Wayland
import Quickshell.Io
import Quickshell.Services.Mpris

// Desktopwidgets op de achtergrondlaag. Indeling uit Zenith → Desktopwidgets (zenith-desktop.json),
// kleuren uit de balk (zenith-shell.json). Elke widget is een eigen klein venster, zodat klikken
// naast een widget gewoon bij het bureaublad aankomen.
Scope {
    id: dw

    FileView { id: deskFile; path: Quickshell.env("HOME") + "/.config/quickshell/zenith-desktop.json"; watchChanges: true; printErrors: false; onFileChanged: reload() }
    FileView { id: shellFile; path: Quickshell.env("HOME") + "/.config/quickshell/zenith-shell.json"; watchChanges: true; onFileChanged: reload() }

    function parse(f) {
        try { var t = f.text().trim(); if (t.length > 0) return JSON.parse(t); } catch (e) {}
        return {};
    }
    readonly property var desk: parse(deskFile)
    readonly property var shellCfg: parse(shellFile)
    readonly property var st: shellCfg.styling || {}
    readonly property string lang: shellCfg.language || "nl"
    readonly property color bg: st.background || "#1e1e2e"
    readonly property color fg: st.text_color || "#cdd6f4"
    readonly property color accent: st.accent || "#89b4fa"

    readonly property var items: desk.items || []
    readonly property var indices: desk.enabled === false ? [] : items.map((w, i) => i)
    function has(kind) { return indices.some(i => items[i].kind === kind); }

    // Grootte op schaal 1.0, gelijk aan KINDS in src/backend/desktop_widgets.rs.
    readonly property var sizes: ({ clock: [320, 140], system: [360, 140], media: [380, 130], weather: [260, 130] })

    property date now: new Date()
    Timer { interval: 1000; running: dw.has("clock"); repeat: true; triggeredOnStart: true; onTriggered: dw.now = new Date() }

    // ---- Systeem: CPU (delta van /proc/stat), RAM en schijf ----
    property real cpu: 0
    property real ram: 0
    property real disk: 0
    property var prevCpu: null
    Process {
        id: statProc
        command: ["bash", "-c", "head -1 /proc/stat; grep -E '^(MemTotal|MemAvailable):' /proc/meminfo; df --output=pcent / | tail -1"]
        stdout: StdioCollector {
            onStreamFinished: {
                var l = this.text.trim().split("\n");
                var c = l[0].trim().split(/\s+/).slice(1).map(Number);
                var idle = c[3] + (c[4] || 0), total = c.reduce((a, b) => a + b, 0);
                if (dw.prevCpu) {
                    var dt = total - dw.prevCpu[1];
                    if (dt > 0) dw.cpu = 1 - (idle - dw.prevCpu[0]) / dt;
                }
                dw.prevCpu = [idle, total];
                var memTotal = parseInt(l[1].split(/\s+/)[1]), memAvail = parseInt(l[2].split(/\s+/)[1]);
                if (memTotal > 0) dw.ram = 1 - memAvail / memTotal;
                dw.disk = (parseInt(l[3]) || 0) / 100;
            }
        }
    }
    Timer { interval: 2000; running: dw.has("system"); repeat: true; triggeredOnStart: true; onTriggered: statProc.running = true }

    // ---- Weer via wttr.in (geen API-sleutel nodig) ----
    property var weather: null
    readonly property string weatherQuery: (desk.weather_location || "") + dw.lang
    onWeatherQueryChanged: if (has("weather")) weatherProc.running = true
    Process {
        id: weatherProc
        command: ["curl", "-sf", "--max-time", "10", "https://wttr.in/" + encodeURIComponent(dw.desk.weather_location || "") + "?format=j1&lang=" + dw.lang]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    var j = JSON.parse(this.text), cur = j.current_condition[0];
                    dw.weather = {
                        temp: cur.temp_C, code: parseInt(cur.weatherCode),
                        desc: (cur["lang_" + dw.lang] || cur.weatherDesc)[0].value,
                        place: j.nearest_area[0].areaName[0].value
                    };
                } catch (e) {}
            }
        }
    }
    Timer { interval: 30 * 60 * 1000; running: dw.has("weather"); repeat: true; triggeredOnStart: true; onTriggered: weatherProc.running = true }
    function weatherIcon(c) {
        if (c === 113) return "sunny";
        if (c === 116) return "partly_cloudy";
        if (c === 119 || c === 122) return "cloudy";
        if ([143, 248, 260].includes(c)) return "fog";
        if ([200, 386, 389, 392, 395].includes(c)) return "storm";
        if ([179, 227, 230, 323, 326, 329, 332, 335, 338, 368, 371].includes(c)) return "snow";
        return "rain";
    }

    readonly property var player: Mpris.players.values.find(p => p.isPlaying) || Mpris.players.values[0] || null

    // ---- Vensters: per scherm, per widget ----
    Variants {
        model: Quickshell.screens
        Scope {
            id: perScreen
            required property var modelData
            Variants {
                model: dw.indices
                PanelWindow {
                    id: win
                    required property int modelData
                    readonly property var w: dw.items[modelData] || {}
                    readonly property var size: dw.sizes[w.kind] || [200, 100]
                    readonly property real s: w.scale || 1

                    screen: perScreen.modelData
                    color: "transparent"
                    WlrLayershell.layer: WlrLayer.Bottom
                    WlrLayershell.namespace: "zenith-desktop"
                    exclusionMode: ExclusionMode.Ignore
                    anchors { top: true; left: true }
                    margins {
                        left: Math.round((w.x || 0) * (screen ? screen.width : 1920))
                        top: Math.round((w.y || 0) * (screen ? screen.height : 1080))
                    }
                    implicitWidth: Math.round(size[0] * s)
                    implicitHeight: Math.round(size[1] * s)

                    Loader {
                        width: win.size[0]
                        height: win.size[1]
                        scale: win.s
                        transformOrigin: Item.TopLeft
                        sourceComponent: ({ clock: clockC, system: systemC, media: mediaC, weather: weatherC })[win.w.kind] || null
                    }
                }
            }
        }
    }

    component Card: Rectangle {
        radius: 22
        color: Qt.alpha(dw.bg, 0.72)
        border.width: 1
        border.color: Qt.alpha(dw.fg, 0.08)
    }

    component Ring: Item {
        id: ringRoot
        property real value: 0
        property string label: ""
        implicitWidth: 84
        implicitHeight: 104
        Shape {
            id: ring
            width: 84; height: 84
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeColor: Qt.alpha(dw.fg, 0.10); strokeWidth: 8; fillColor: "transparent"
                PathAngleArc { centerX: 42; centerY: 42; radiusX: 36; radiusY: 36; startAngle: 0; sweepAngle: 360 }
            }
            ShapePath {
                strokeColor: dw.accent; strokeWidth: 8; fillColor: "transparent"; capStyle: ShapePath.RoundCap
                PathAngleArc {
                    centerX: 42; centerY: 42; radiusX: 36; radiusY: 36; startAngle: -90
                    sweepAngle: 360 * Math.max(0.001, Math.min(1, ringRoot.value))
                    Behavior on sweepAngle { NumberAnimation { duration: 600; easing.type: Easing.OutCubic } }
                }
            }
            Text {
                anchors.centerIn: parent
                text: Math.round(ringRoot.value * 100) + "%"
                color: dw.fg; font.pixelSize: 18; font.weight: Font.DemiBold
            }
        }
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.top: ring.bottom; anchors.topMargin: 4
            text: ringRoot.label
            color: Qt.alpha(dw.fg, 0.6); font.pixelSize: 12; font.letterSpacing: 1
        }
    }

    Component {
        id: clockC
        Item {
            ColumnLayout {
                anchors.centerIn: parent
                spacing: -4
                layer.enabled: true
                layer.effect: MultiEffect { shadowEnabled: true; shadowBlur: 0.8; shadowOpacity: 0.55; shadowVerticalOffset: 2 }
                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: Qt.formatTime(dw.now, "HH:mm")
                    color: dw.fg; font.pixelSize: 84; font.weight: Font.Light
                }
                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: dw.now.toLocaleDateString(Qt.locale(dw.lang), "dddd d MMMM")
                    color: dw.accent; font.pixelSize: 18; font.weight: Font.Medium; font.capitalization: Font.Capitalize
                }
            }
        }
    }

    Component {
        id: systemC
        Card {
            RowLayout {
                anchors.centerIn: parent
                spacing: 26
                Ring { value: dw.cpu; label: "CPU" }
                Ring { value: dw.ram; label: "RAM" }
                Ring { value: dw.disk; label: "SCHIJF" }
            }
        }
    }

    Component {
        id: mediaC
        Card {
            id: mc
            readonly property var p: dw.player
            opacity: p ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 300 } }
            // Positie wordt niet vanzelf bijgewerkt (Quickshell-docs): elke seconde opvragen.
            Timer { interval: 1000; running: mc.p && mc.p.isPlaying; repeat: true; onTriggered: mc.p.positionChanged() }

            RowLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 14

                Item {
                    Layout.preferredWidth: 106; Layout.preferredHeight: 106
                    Rectangle { anchors.fill: parent; radius: 14; color: Qt.alpha(dw.accent, 0.18)
                        ZIcon { anchors.centerIn: parent; name: "media"; color: dw.accent; size: 44; visible: art.status !== Image.Ready } }
                    Image { id: art; anchors.fill: parent; source: mc.p ? mc.p.trackArtUrl : ""; fillMode: Image.PreserveAspectCrop; visible: false; asynchronous: true }
                    Rectangle { id: artMask; anchors.fill: parent; radius: 14; visible: false; layer.enabled: true }
                    MultiEffect { anchors.fill: parent; source: art; maskEnabled: true; maskSource: artMask; visible: art.status === Image.Ready }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Text { Layout.fillWidth: true; text: mc.p ? mc.p.trackTitle : ""; color: dw.fg; font.pixelSize: 16; font.weight: Font.DemiBold; elide: Text.ElideRight }
                    Text { Layout.fillWidth: true; text: mc.p ? mc.p.trackArtist : ""; color: Qt.alpha(dw.fg, 0.6); font.pixelSize: 13; elide: Text.ElideRight }
                    Rectangle {
                        Layout.fillWidth: true; Layout.topMargin: 6; height: 4; radius: 2; color: Qt.alpha(dw.fg, 0.12)
                        Rectangle {
                            height: parent.height; radius: 2; color: dw.accent
                            width: mc.p && mc.p.length > 0 ? parent.width * Math.min(1, mc.p.position / mc.p.length) : 0
                            Behavior on width { NumberAnimation { duration: 900 } }
                        }
                    }
                    RowLayout {
                        Layout.alignment: Qt.AlignHCenter
                        spacing: 22
                        Repeater {
                            model: [
                                { t: "media_prev", a: () => mc.p.previous(), ok: mc.p && mc.p.canGoPrevious },
                                { t: mc.p && mc.p.isPlaying ? "media_pause" : "media_play", a: () => mc.p.togglePlaying(), ok: mc.p && mc.p.canTogglePlaying },
                                { t: "media_next", a: () => mc.p.next(), ok: mc.p && mc.p.canGoNext }
                            ]
                            ZIcon {
                                required property var modelData
                                name: modelData.t
                                color: btn.containsMouse ? dw.accent : dw.fg
                                opacity: modelData.ok ? 1 : 0.35
                                hovered: btn.containsMouse
                                size: modelData.t === "media_prev" || modelData.t === "media_next" ? 24 : 30
                                MouseArea { id: btn; anchors.fill: parent; anchors.margins: -6; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                                    onClicked: if (parent.modelData.ok) parent.modelData.a() }
                            }
                        }
                    }
                }
            }
        }
    }

    Component {
        id: weatherC
        Card {
            RowLayout {
                anchors.centerIn: parent
                spacing: 16
                ZIcon { name: dw.weather ? dw.weatherIcon(dw.weather.code) : "thermometer"; color: dw.accent; size: 60 }
                ColumnLayout {
                    spacing: 0
                    Text { text: dw.weather ? dw.weather.temp + "°" : "…"; color: dw.fg; font.pixelSize: 40; font.weight: Font.DemiBold }
                    Text { text: dw.weather ? dw.weather.desc : "Weer laden"; color: Qt.alpha(dw.fg, 0.75); font.pixelSize: 13 }
                    Text { text: dw.weather ? dw.weather.place : ""; color: dw.accent; font.pixelSize: 12 }
                }
            }
        }
    }
}
