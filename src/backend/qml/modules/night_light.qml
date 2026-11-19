import QtQuick
import "../components"
import Quickshell
import Quickshell.Io

// Nachtlicht: hyprshade (zoals de Control Center-kaart), anders hyprsunset.
Item {
    id: nightMod
    implicitWidth: nightText.implicitWidth
    implicitHeight: 22
    property bool on: false

    Process {
        id: stateProc
        command: ["bash", "-c", "if command -v hyprshade >/dev/null; then hyprshade current; else pgrep -x hyprsunset >/dev/null && echo on; fi"]
        running: true
        stdout: StdioCollector { onStreamFinished: nightMod.on = this.text.trim().length > 0 }
    }
    Process { id: toggleProc; onExited: stateProc.running = true }

    ZIcon {
        id: nightText
        anchors.centerIn: parent
        name: nightMod.on ? "nightlight" : "sunny"
        color: nightMod.on ? "#fab387" : (root.fgColor || "#cdd6f4")
        size: (root.userFontSize || 11) + 4
    }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            toggleProc.command = ["bash", "-c", nightMod.on
                ? "hyprshade off 2>/dev/null || pkill -x hyprsunset"
                : "if command -v hyprshade >/dev/null; then hyprshade on blue-light-filter; else setsid -f hyprsunset -t 4000 >/dev/null 2>&1; fi"];
            toggleProc.running = true;
        }
    }
}
