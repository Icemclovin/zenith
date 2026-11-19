import QtQuick
import "../components"
import Quickshell
import Quickshell.Io

// Opent het meldingenpaneel van Zenith.
Item {
    implicitWidth: bell.implicitWidth
    implicitHeight: 22
    Process { id: openProc; command: ["quickshell", "-p", Quickshell.env("HOME") + "/.config/quickshell/shell.qml", "ipc", "call", "notifications", "toggle"] }
    ZIcon {
        id: bell
        anchors.centerIn: parent
        name: "dnd_off"
        color: root.fgColor || "#cdd6f4"
        size: (root.userFontSize || 11) + 4
    }
    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: openProc.running = true }
}
