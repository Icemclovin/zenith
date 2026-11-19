import QtQuick
import "../components"
import Quickshell
import Quickshell.Io

// Microfoon: klik = dempen, scrollen = volume.
Item {
    id: micMod
    implicitWidth: micText.implicitWidth
    implicitHeight: 22
    property string level: "??%"
    property bool muted: false

    Process {
        id: micProc
        command: ["wpctl", "get-volume", "@DEFAULT_AUDIO_SOURCE@"]
        running: true
        onExited: micTimer.start()
        stdout: SplitParser {
            onRead: data => {
                var m = data.match(/Volume:\s*([\d.]+)/);
                if (m) { micMod.level = Math.round(parseFloat(m[1]) * 100) + "%"; micMod.muted = data.indexOf("[MUTED]") !== -1; }
            }
        }
    }
    Timer { id: micTimer; interval: 2000; onTriggered: micProc.running = true }
    Process { id: micCmd }
    function run(args) { micCmd.command = ["wpctl"].concat(args); micCmd.running = true; micProc.running = true; }

    ZLabel {
        id: micText
        anchors.centerIn: parent
        name: micMod.muted ? "mic_muted" : "mic"
        text: micMod.level
        color: micMod.muted ? "#6c7086" : (root.fgColor || "#cdd6f4")
        pixelSize: root.userFontSize || 11
    }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: micMod.run(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
        onWheel: wheel => micMod.run(["set-volume", "@DEFAULT_AUDIO_SOURCE@", wheel.angleDelta.y > 0 ? "5%+" : "5%-"])
    }
}
