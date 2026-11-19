import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Widgets

// Open vensters als app-iconen: klik = focus, middelklik = sluiten.
RowLayout {
    id: taskMod
    spacing: 4

    Repeater {
        model: ToplevelManager.toplevels
        Rectangle {
            id: task
            required property var modelData
            readonly property var entry: DesktopEntries.heuristicLookup(modelData.appId)
            implicitWidth: 28; implicitHeight: 24; radius: 7
            color: modelData.activated ? Qt.alpha(root.themeAccent || "#89b4fa", 0.25)
                 : (taskHover.containsMouse ? Qt.alpha(root.fgColor || "#cdd6f4", 0.10) : "transparent")
            Behavior on color { ColorAnimation { duration: 150 } }

            IconImage {
                anchors.centerIn: parent
                implicitSize: 18
                source: Quickshell.iconPath(task.entry ? task.entry.icon : task.modelData.appId, "application-x-executable")
            }
            Rectangle {
                anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter
                width: task.modelData.activated ? 12 : 4; height: 2; radius: 1
                color: root.themeAccent || "#89b4fa"
                Behavior on width { NumberAnimation { duration: 150 } }
            }
            MouseArea {
                id: taskHover
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton
                cursorShape: Qt.PointingHandCursor
                onClicked: mouse => mouse.button === Qt.MiddleButton ? task.modelData.close() : task.modelData.activate()
            }
        }
    }
}
