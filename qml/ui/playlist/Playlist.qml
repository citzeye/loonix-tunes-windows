/* --- loonixtunesv2/qml/ui/playlist/Playlist.qml | Generic (Dumb Canvas) --- */

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

Rectangle {
    id: playlistRoot
    Layout.fillWidth: true
    Layout.fillHeight: true
    color: 'transparent'

    Rectangle { height: 8; anchors.top: parent.top; anchors.left: parent.left; anchors.right: parent.right; color: theme.colormap.bgmain }
    Rectangle { height: 8; anchors.bottom: parent.bottom; anchors.bottomMargin: 8; anchors.left: parent.left; anchors.right: parent.right; color: theme.colormap.bgmain }
    Rectangle { width: 8; anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom; color: theme.colormap.bgoverlay }
    Rectangle { width: 8; anchors.right: parent.right; anchors.top: parent.top; anchors.bottom: parent.bottom; color: theme.colormap.bgoverlay }
    Rectangle { height: 8; anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.right: parent.right; color: theme.colormap.bgoverlay }

    ListView {
        id: playlistView
        anchors.fill: parent
        anchors.leftMargin: 16
        anchors.rightMargin: 8
        anchors.topMargin: 8
        anchors.bottomMargin: 16
        clip: true
        model: musicModel

        ScrollBar.vertical: ScrollBar {
            id: vBar
            width: 6
            policy: ScrollBar.AsNeeded
            background: Rectangle { color: "transparent" }
            contentItem: Rectangle {
                implicitWidth: 6
                implicitHeight: Math.max(30, playlistView.height * playlistView.visibleArea.heightRatio)
                radius: 3
                color: vBar.pressed ? theme.colormap.playeraccent : vBar.hovered ? theme.colormap.headerhover : theme.colormap.playeraccent
                opacity: vBar.active ? 1.0 : 0.5
                Behavior on color { ColorAnimation { duration: 150 } }
                Behavior on opacity { NumberAnimation { duration: 150 } }
            }
        }

        Rectangle {
            anchors.centerIn: parent
            width: parent.width - 40
            height: 80
            color: 'transparent'
            visible: playlistView.count === 0
            Column {
                anchors.centerIn: parent
                spacing: 8
                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: 'No music found'
                    font.family: kodeMono.name
                    font.pixelSize: 14
                    color: theme.colormap.playersubtext
                }
            }
        }

        Component.onCompleted: {
            musicModel.current_index_changed.connect(function() {
                if (musicModel.current_index >= 0) {
                    playlistView.positionViewAtIndex(musicModel.current_index, ListView.Center)
                }
            })
        }

        delegate: Component {
            id: playlistDelegate
            Rectangle {
                width: playlistView.width
                height: 26
                color: 'transparent'
                property bool isPlayingNow: !model.is_folder && model.name === musicModel.current_title
                property bool isHovered: false
                property bool hasParentFolder: !!model.parent_folder && model.parent_folder !== ''

                // Indentasi - 15px kalau ada parent_folder
                anchors.leftMargin: hasParentFolder ? 15 : 0
                anchors.rightMargin: 6

                // Left Border - menunjukkan hierarchy folder
                Rectangle {
                    width: 2
                    height: 20
                    visible: hasParentFolder
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    color: theme.colormap.playlistfolder
                }

                Text {
                    id: playlistIcon
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    text: isPlayingNow ? '󰶻' : model.is_folder ? '󱍙' : '󰽷'
                    font.family: symbols.name
                    color: isPlayingNow || isHovered || root.rightClickedIndex === index ? theme.colormap.playlistactive : theme.colormap.playlisticon
                    font.pixelSize: model.is_folder ? 20 : 14
                    leftPadding: 6
                }

                Text {
                    text: model.name
                    color: {
                        if (isPlayingNow || isHovered || root.rightClickedIndex === index) return theme.colormap.playlistactive
                        if (model.is_folder) return theme.colormap.playlistfolder
                        return theme.colormap.playlisttext
                    }
                    font.family: kodeMono.name
                    font.pixelSize: model.is_folder ? 14 : 13
                    font.bold: isPlayingNow
                    elide: Text.ElideRight
                    anchors.left: playlistIcon.right
                    leftPadding: 6
                    anchors.right: parent.right
                    anchors.rightMargin: 4
                    anchors.verticalCenter: parent.verticalCenter
                }

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onEntered: parent.isHovered = true
                    onExited: parent.isHovered = false
                    onClicked: function(mouse) {
                        if (mouse.button === Qt.LeftButton) {
                            if (model.is_folder) {
                                musicModel.toggle_folder(index)
                            } else {
                                musicModel.play_at(index)
                            }
                        } else if (mouse.button === Qt.RightButton) {
                            parent.isHovered = false
                            root.popupMenuVisible = false
                            root.tabContextMenuVisible = false
                            root.externalFilesContextMenuVisible = false
                            root.rightClickedIndex = index
                            root.playlistContextItemIndex = index
                            root.playlistContextItemName = String(model.name || "")
                            root.playlistContextItemPath = String(model.path || "")
                            root.playlistContextIsFolder = Boolean(model.is_folder)
                            var menuHeight = 170
                            var menuWidth = 170
                            var bottomPos = parent.mapToItem(null, 0, parent.height)
                            var topPos = parent.mapToItem(null, 0, 0)
                            var spaceBelow = root.height - bottomPos.y
                            var spaceAbove = topPos.y
                            if (spaceBelow >= menuHeight) {
                                root.playlistContextMenuY = bottomPos.y | 0
                            } else if (spaceAbove >= menuHeight) {
                                root.playlistContextMenuY = (topPos.y - menuHeight) | 0
                            } else {
                                root.playlistContextMenuY = bottomPos.y | 0
                            }
                            if (bottomPos.x + menuWidth > root.width) {
                                root.playlistContextMenuX = (root.width - menuWidth) | 0
                            } else {
                                root.playlistContextMenuX = bottomPos.x | 0
                            }
                            root.playlistContextMenuVisible = true
                        }
                    }
                    onDoubleClicked: function(mouse) {
                        if (mouse.button === Qt.LeftButton && model.is_folder) {
                            musicModel.switch_to_folder(model.path)
                            root.playlistSource = "qrc:/qml/ui/playlist/Playlist.qml"
                        }
                    }
                    onPressAndHold: function(mouse) {
                        root.popupX = mouse.x
                        root.popupY = mouse.y
                        root.popupMenuVisible = true
                    }
                }
            }
        }
    }
}
