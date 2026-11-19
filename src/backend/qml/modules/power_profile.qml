import QtQuick
import "../components"
import Quickshell
import Quickshell.Io

// Energieprofiel (power-profiles-daemon); klik = volgend profiel. Verborgen zonder daemon.
Item {
    id: ppMod
    implicitWidth: ppText.implicitWidth
    implicitHeight: 22
    visible: profile !== ""
    property string profile: ""
    readonly property var order: ["power-saver", "balanced", "performance"]

    Process {
        id: getProc
        command: ["powerprofilesctl", "get"]
        running: true
        stdout: StdioCollector { onStreamFinished: ppMod.profile = this.text.trim() }
    }
    Timer { interval: 10000; running: true; repeat: true; onTriggered: getProc.running = true }
    Process { id: setProc; onExited: getProc.running = true }

    ZIcon {
        id: ppText
        anchors.centerIn: parent
        name: ppMod.profile.replace("-", "_")
        color: ppMod.profile === "performance" ? (root.themeAccent || "#89b4fa") : (root.fgColor || "#cdd6f4")
        size: (root.userFontSize || 11) + 4
    }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            var next = ppMod.order[(ppMod.order.indexOf(ppMod.profile) + 1) % ppMod.order.length];
            setProc.command = ["powerprofilesctl", "set", next];
            setProc.running = true;
        }
    }
}
