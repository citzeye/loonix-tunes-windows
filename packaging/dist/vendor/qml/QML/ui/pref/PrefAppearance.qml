/* --- loonixtunesv2/qml/ui/pref/PrefAppearance.qml | PrefAppearance --- */
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import "../contextmenu"

Item {
    id: prefAppearanceRoot
    property int refreshTicker: 0
    property bool playlistContextMenuVisible: false
    property bool themeInitialized: false

    Component.onCompleted: {
        if (!themeInitialized) {
            var customThemes = theme.get_custom_themes();
            if (customThemes.length === 0) {
                theme.initialize_default_theme();
            }
            themeInitialized = true;
        }
    }

    function openThemeEditorWithTarget(targetIndex) {
        root.prefThemeEditorProfileTarget = targetIndex
        root.prefThemeEditorVisible = true
    }

    Connections {
        target: theme
        function onColormapChanged() {
            refreshTicker++
        }
    }

    Flickable {
        id: appFlick
        anchors.fill: parent
        contentWidth: width
        contentHeight: appColumn.implicitHeight + 40
        clip: true
        interactive: true
        boundsBehavior: Flickable.StopAtBounds

        ScrollBar.vertical: ScrollBar {
            id: vBar
            width: 6
            policy: ScrollBar.AsNeeded
            background: Rectangle { color: "transparent" }
            contentItem: Rectangle {
                implicitWidth: 6
                implicitHeight: Math.max(30, appFlick.height * appFlick.visibleArea.heightRatio)
                radius: 3
                color: vBar.pressed ? theme.colormap.playeraccent : 
                       vBar.hovered ? theme.colormap.headerhover : 
                       theme.colormap.playeraccent
                opacity: vBar.active ? 1.0 : 0.5
                Behavior on color { ColorAnimation { duration: 150 } }
                Behavior on opacity { NumberAnimation { duration: 150 } }
            }
        }

        ColumnLayout {
            id: appColumn
            y: 10
            anchors.leftMargin: 10
            anchors.rightMargin: 10
            anchors.topMargin: 10
            anchors.bottomMargin: 10
            width: appFlick.width - 20
            spacing: 12

            // --- 1. DEFAULT THEMES ---
            ColumnLayout {
                Layout.alignment: Qt.AlignHCenter 
                spacing: 4

                Repeater {
                    model: ["Loonix", "Blue", "Green", "Monochrome", "Orange", "Pink", "Red", "Yellow"]

                    delegate: Rectangle {
                        Layout.preferredWidth: 200
                        Layout.preferredHeight: 32
                        Layout.alignment: Qt.AlignHCenter 
                        radius: 4
                        color: modelData === theme.current_theme ? theme.colormap["playeraccent"] : theme.colormap["bgoverlay"]
                        border.color: themeItemArea.containsMouse ? theme.colormap["playeraccent"] : theme.colormap["graysolid"]
                        border.width: 1

                        Behavior on color { ColorAnimation { duration: 150 } }
                        Behavior on border.color { ColorAnimation { duration: 150 } }

                        Text {
                            anchors.centerIn: parent
                            text: modelData
                            font.family: kodeMono.name
                            font.pixelSize: 12
                            color: modelData === theme.current_theme ? theme.colormap["bgmain"] : theme.colormap["playlisttext"]
                            font.bold: modelData === theme.current_theme
                        }

                        MouseArea {
                            id: themeItemArea
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            hoverEnabled: true
                            onClicked: theme.set_theme(modelData)
                        }
                    }
                }
            }

            Item { Layout.preferredHeight: 32 }

            // --- 2. CUSTOM THEMES (PRESETS) ---
            ColumnLayout {
                Layout.alignment: Qt.AlignHCenter 
                spacing: 4

                Repeater {
                    id: customThemeRepeater
                    model: theme.get_custom_themes()

                    delegate: Rectangle {
                        property int presetIndex: modelData.original_index
                        property string presetName: modelData.name

                        Layout.preferredWidth: 200
                        Layout.preferredHeight: 32
                        Layout.alignment: Qt.AlignHCenter 
                        radius: 4
                        color: presetName === theme.current_theme ? theme.colormap["playeraccent"] : theme.colormap["bgoverlay"]
                        border.color: {
                             if (prefPage.appearanceMenuVisible && prefPage.appearanceMenuIndex === presetIndex) {
                                return theme.colormap["playeraccent"]
                            }
                            if (customItemArea.containsMouse) {
                                return theme.colormap["playeraccent"]
                            }
                            return theme.colormap["graysolid"]
                        }
                        border.width: (prefPage.appearanceMenuVisible && prefPage.appearanceMenuIndex === presetIndex) ? 2 : 1

                        Behavior on color { ColorAnimation { duration: 150 } }
                        Behavior on border.color { ColorAnimation { duration: 150 } }

                        Text {
                            anchors.centerIn: parent
                            text: presetName
                            font.family: kodeMono.name
                            font.pixelSize: 12
                            color: presetName === theme.current_theme ? theme.colormap["bgmain"] : theme.colormap["playlisttext"]
                            font.bold: presetName === theme.current_theme
                        }

                        MouseArea {
                            id: customItemArea
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            hoverEnabled: true
                            acceptedButtons: Qt.LeftButton | Qt.RightButton
                            onClicked: (mouse) => {
                                if (mouse.button === Qt.RightButton) {
                                    var p = customItemArea.mapToItem(
                                        prefPage,
                                        0,
                                        customItemArea.height
                                    )
                                    prefPage.openAppearanceMenu(p.x, p.y, presetIndex)
                                } else if (mouse.button === Qt.LeftButton) {
                                    prefPage.closeAppearanceMenu()
                                    theme.set_theme(presetName)
                                }
                            }
                        }
                    }
                }
            }

            Item { Layout.preferredHeight: 32 }

            // --- 3. CREATE THEME BUTTON ---
            ColumnLayout {
                Layout.alignment: Qt.AlignHCenter 

                Rectangle {
                    Layout.preferredWidth: 200
                    Layout.preferredHeight: 32
                    radius: 4
                    color: createThemeArea.containsMouse ? theme.colormap.playeraccent : theme.colormap.bgoverlay
                    border.color: theme.colormap.playeraccent
                    Behavior on color { ColorAnimation { duration: 150 } }

                    Text {
                        anchors.centerIn: parent
                        text: 'CREATE THEME'
                        font.family: kodeMono.name
                        font.pixelSize: 12
                        font.bold: true
                        color: createThemeArea.containsMouse ? theme.colormap.bgmain : theme.colormap.playeraccent
                    }

                    MouseArea {
                        id: createThemeArea
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        hoverEnabled: true
                        onClicked: {
                            root.prefThemeEditorProfileTarget = -1
                            root.prefThemeEditorVisible = true
                        }
                    }
                }
            }
        }
    }
}