import QtQuick
import "../components"
import Quickshell
import Quickshell.Io
import Quickshell.Hyprland

// Actieve toetsenbordindeling; klik = volgende indeling.
Item {
    id: kbMod
    implicitWidth: kbText.implicitWidth
    implicitHeight: 22
    property string layout: ""

    Process {
        command: ["hyprctl", "devices", "-j"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    var kbs = JSON.parse(this.text).keyboards;
                    var main = kbs.find(k => k.main) || kbs[0];
                    if (main) kbMod.layout = main.active_keymap;
                } catch (e) {}
            }
        }
    }
    Connections {
        target: Hyprland
        function onRawEvent(event) {
            if (event.name === "activelayout") kbMod.layout = event.data.split(",").slice(1).join(",");
        }
    }
    Process { id: nextProc; command: ["hyprctl", "switchxkblayout", "all", "next"] }

    ZLabel {
        id: kbText
        anchors.centerIn: parent
        name: "keyboard"
        text: kbMod.layout.substring(0, 2).toUpperCase()
        color: root.fgColor || "#cdd6f4"
        pixelSize: root.userFontSize || 11
    }
    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: nextProc.running = true }
}
