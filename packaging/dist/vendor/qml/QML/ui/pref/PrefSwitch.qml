/* --- loonixtunesv2/qml/ui/pref/PrefSwitch.qml | PrefSwitch --- */

import QtQuick
import QtQuick.Layouts

RowLayout {
    id: switchRoot
    property string label: "Pref Name"
    property string description: ""
    property bool checked: false
    signal toggled()

    spacing: 15

    ColumnLayout {
        Layout.fillWidth: true
        Layout.minimumWidth: 0
        spacing: 2
        Text {
            text: label
            color: theme.colormap["playlisttext"]
            font.family: kodeMono.name
            font.pixelSize: 13
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            Layout.minimumWidth: 0
        }
        Text {
            text: description
            color: theme.colormap["playersubtext"]
            font.family: kodeMono.name
            font.pixelSize: 11
            visible: description !== ""
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            Layout.minimumWidth: 0
        }
    }

    // Custom Switch UI - align with top label
    Rectangle {
        Layout.alignment: Qt.AlignTop
        Layout.preferredWidth: 24 
        Layout.preferredHeight: 16
        radius: 8
        color: checked ? theme.colormap["playeraccent"] : theme.colormap["graysolid"]

        Rectangle {
            width: 12; height: 12
            radius: 6
            color: theme.colormap["bgmain"]
            y: 2
            x: checked ? parent.width - width - 2 : 2
            Behavior on x { NumberAnimation { duration: 150; easing.type: Easing.InOutQuad } }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            // JANGAN checked = !checked di UI, biarin state-nya dikendalikan 100% oleh Rust (musicModel)
            onClicked: { checked = !checked; toggled() }
        }
    }
}
