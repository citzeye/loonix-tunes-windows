/* --- loonixtunesv2/qml/ui/pref/PrefLibrary.qml | PrefLibrary --- */
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt.labs.platform

ColumnLayout {
    id: libRoot
    spacing: 32
    Layout.leftMargin: 16
    Layout.rightMargin: 16
    Layout.topMargin: 12
    Layout.fillWidth: true
    Layout.fillHeight: true

    FolderDialog {
        id: manualScanFolderPicker
        title: "Select Folder to Scan"
        onAccepted: {
            var path = manualScanFolderPicker.folder.toString();
            path = path.replace(/^(file:\/{2,3})/, "");
            if (path.endsWith('/')) {
                path = path.substring(0, path.length - 1);
            }
            musicModel.scan_folder(path);
        }
    }

    // --- GRUP 1: SETTINGS ---
    ColumnLayout {
        Layout.fillWidth: true
        spacing: 12

        PrefSwitch {
            Layout.fillWidth: true // WAJIB biar dia nge-stretch bener
            label: "Prioritize Folders"
            description: "Always show folders at the top of the list, before loose files."
            checked: !!musicModel && !!musicModel.prioritize_folders
            onToggled: musicModel.toggle_prioritize_folders()
        }

        PrefSwitch {
            Layout.fillWidth: true
            label: "Auto-Scan ~/Music"
            description: "Watch for new files added to your default music directory."
            checked: !!musicModel && !!musicModel.auto_scan_enabled
            onToggled: musicModel.toggle_auto_scan()
        }
    }

    // --- GRUP 2: MANUAL SCAN ---
    ColumnLayout {
        Layout.fillWidth: true
        spacing: 8

        Text {
            Layout.fillWidth: true
            text: "MANUAL SCAN TOOLS"
            font.family: kodeMono.name
            font.pixelSize: 10
            font.weight: Font.Bold
            color: theme.colormap["playeraccent"]
            font.letterSpacing: 1
        }

        Text {
            Layout.fillWidth: true
            text: "Scan a specific folder to load its music without adding it as a permanent tab."
            font.family: kodeMono.name
            font.pixelSize: 11
            color: theme.colormap["playersubtext"]
            wrapMode: Text.WordWrap
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: 8
            spacing: 12

            // TOMBOL: SCAN FOLDER
            Rectangle {
                // PAKAI implicitWidth + padding agar kotak menyesuaikan teks
                Layout.preferredWidth: Math.max(140, scanFolderText.implicitWidth + 24)
                Layout.preferredHeight: 32
                radius: 4
                color: scanFolderArea.containsMouse ? theme.colormap["bgoverlay"] : "transparent"
                border.color: scanFolderArea.containsMouse ? theme.colormap["playeraccent"] : theme.colormap["graysolid"]
                border.width: 1

                Text {
                    id: scanFolderText
                    anchors.centerIn: parent // Lebih aman pakai centerIn untuk tombol
                    text: "󰉏  Scan Folder..."
                    font.family: symbols.name
                    font.pixelSize: 11
                    color: scanFolderArea.containsMouse ? theme.colormap["playeraccent"] : theme.colormap["playlisttext"]
                    // Tambahan: jika tetap kepanjangan, potong dengan elide
                    elide: Text.ElideRight
                    width: parent.width - 10
                    horizontalAlignment: Text.AlignHCenter
                }

                MouseArea {
                    id: scanFolderArea
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    hoverEnabled: true
                    onClicked: manualScanFolderPicker.open()
                }
            }

            // TOMBOL: RESCAN
            Rectangle {
                Layout.preferredWidth: Math.max(150, rescanText.implicitWidth + 24)
                Layout.preferredHeight: 32
                radius: 4
                color: rescanArea.containsMouse ? theme.colormap["bgoverlay"] : "transparent"
                border.color: rescanArea.containsMouse ? theme.colormap["playeraccent"] : theme.colormap["graysolid"]
                border.width: 1

                Text {
                    id: rescanText
                    anchors.centerIn: parent
                    text: "󰑐  Rescan ~/Music"
                    font.family: symbols.name
                    font.pixelSize: 11
                    color: rescanArea.containsMouse ? theme.colormap["playeraccent"] : theme.colormap["playlisttext"]
                    elide: Text.ElideRight
                    width: parent.width - 10
                    horizontalAlignment: Text.AlignHCenter
                }

                MouseArea {
                    id: rescanArea
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    hoverEnabled: true
                    onClicked: musicModel.scan_music()
                }
            }

            Item {
                Layout.fillWidth: true
            }
        }
    }

    Item {
        Layout.fillHeight: true
    }
}
