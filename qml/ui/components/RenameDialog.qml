/* --- loonixtunesv2/qml/ui/components/RenameDialog.qml | RenameDialog --- */
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: renameDialogRoot
    visible: false
    anchors.fill: parent
    z: 9999

    property alias dialogVisible: renameDialogRoot.visible
    property string initialText: ""
    property int maxLength: 16
    signal saved(string newName)
    signal cancelled()

    Rectangle {
        anchors.fill: parent
        color: theme.colormap["overlay"]
        MouseArea {
            anchors.fill: parent
            onClicked: renameDialogRoot.cancelled()
        }
    }

    Rectangle {
        id: renameDialogBox
        anchors.centerIn: parent
        width: 240
        height: 80
        color: theme.colormap.bgmain
        border.color: theme.colormap.playeraccent
        radius: 4

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 10
            spacing: 8

            TextInput {
                id: renameInput
                Layout.fillWidth: true
                Layout.preferredHeight: 28
                font.family: kodeMono.name
                font.pixelSize: 12
                color: theme.colormap.playeraccent
                verticalAlignment: Text.AlignVCenter
                maximumLength: renameDialogRoot.maxLength
                activeFocusOnPress: true
                selectByMouse: true

                Component.onCompleted: {
                    text = renameDialogRoot.initialText
                    forceActiveFocus()
                    selectAll()
                }

                onAccepted: {
                    renameDialogRoot.saved(text)
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 16

                Text {
                    text: 'CANCEL'
                    font.family: kodeMono.name
                    font.pixelSize: 10
                    color: cancelMA.containsMouse ? theme.colormap.playlisticon : theme.colormap.tabtext
                    MouseArea {
                        id: cancelMA
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: renameDialogRoot.cancelled()
                    }
                }

                Item { Layout.fillWidth: true }

                Text {
                    text: 'SAVE'
                    font.family: kodeMono.name
                    font.pixelSize: 10
                    color: saveMA.containsMouse ? theme.colormap.playlisticon : theme.colormap.tabtext
                    MouseArea {
                        id: saveMA
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: renameDialogRoot.saved(renameInput.text)
                    }
                }
            }
        }
    }
}
