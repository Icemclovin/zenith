// zenith-panel: wordt door Zenith bijgewerkt (vorige versie blijft bewaard als .bak)
import QtQuick

// Icoon uit de Nerd Font (Material Design-set). Animaties staan aan/uit via
// Zenith → Statusbalk → Animaties → "Iconen animeren".
Text {
    id: ic

    /// Naam uit ZSettings (aanpasbaar in Zenith → Iconen) of een vaste glyph via `icon`.
    property string name: ""
    property string icon: ""
    property real size: 14
    property bool animated: ZSettings.iconAnim
    /// Licht vergroten, bv. gekoppeld aan een MouseArea.containsMouse.
    property bool hovered: false
    /// Blijft draaien, bv. tijdens laden.
    property bool spinning: false

    text: icon !== "" ? icon : ZSettings.glyph(name)
    font.family: "JetBrainsMono Nerd Font Propo"
    font.pixelSize: size
    horizontalAlignment: Text.AlignHCenter
    verticalAlignment: Text.AlignVCenter

    HoverHandler { id: hover }
    scale: (hovered || hover.hovered) && animated ? 1.18 : 1.0
    Behavior on scale { enabled: ic.animated; NumberAnimation { duration: 180; easing.type: Easing.OutBack } }
    Behavior on color { enabled: ic.animated; ColorAnimation { duration: 220 } }

    // "Pop" als het icoon wisselt (bv. gedempt ↔ aan), los van de hover-schaal.
    transform: Scale { id: pop; origin.x: ic.width / 2; origin.y: ic.height / 2 }
    onTextChanged: if (animated) popAnim.restart()
    ParallelAnimation {
        id: popAnim
        NumberAnimation { target: pop; property: "xScale"; from: 0.55; to: 1; duration: 320; easing.type: Easing.OutBack }
        NumberAnimation { target: pop; property: "yScale"; from: 0.55; to: 1; duration: 320; easing.type: Easing.OutBack }
        NumberAnimation { target: ic; property: "opacity"; from: 0.3; to: 1; duration: 220 }
    }

    RotationAnimator on rotation {
        running: ic.spinning && ic.animated
        from: 0; to: 360; duration: 1000; loops: Animation.Infinite
        onRunningChanged: if (!running) ic.rotation = 0
    }
}
