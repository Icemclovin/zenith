// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick

// Icoon + tekst naast elkaar, voor balkmodules (bv. volume-icoon met "45%").
Row {
    id: zl
    property string name: ""
    property string icon: ""
    property string text: ""
    property color color: "#cdd6f4"
    property color iconColor: color
    property int pixelSize: 11
    property bool bold: false
    property bool spinning: false

    spacing: zi.text.length > 0 && zl.text.length > 0 ? 5 : 0
    ZIcon {
        id: zi
        anchors.verticalCenter: parent.verticalCenter
        name: zl.name; icon: zl.icon; color: zl.iconColor; size: zl.pixelSize + 4; spinning: zl.spinning
        visible: text.length > 0
    }
    Text {
        anchors.verticalCenter: parent.verticalCenter
        text: zl.text; color: zl.color; font.pixelSize: zl.pixelSize; font.bold: zl.bold
        visible: zl.text.length > 0
    }
}
