import QtQuick
import "../components"
import Quickshell
import Quickshell.Wayland

// Houd het scherm wakker (blokkeert idle/vergrendelen zolang hij aan staat).
Item {
    id: awakeMod
    implicitWidth: awakeText.implicitWidth
    implicitHeight: 22
    property bool awake: false

    IdleInhibitor { window: root; enabled: awakeMod.awake }

    ZIcon {
        id: awakeText
        anchors.centerIn: parent
        name: awakeMod.awake ? "awake" : "idle"
        color: awakeMod.awake ? (root.themeAccent || "#89b4fa") : (root.fgColor || "#cdd6f4")
        size: (root.userFontSize || 11) + 4
    }
    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: awakeMod.awake = !awakeMod.awake }
}
