/* --- loonixtunesv2/qml/ui/pref/PrefDonate.qml | PrefDonate --- */
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    anchors.fill: parent

    ScrollView {
        id: donateScroll
        anchors.fill: parent
        contentWidth: availableWidth // Pakai availableWidth biar pas sama lebar ScrollView
        clip: true

        ColumnLayout {
            id: donateColumn
            // Jangan pakai width: parent.width - 20, mending pakai Layout.fillWidth
            width: donateScroll.availableWidth 
            spacing: 25
            
            // Padding atas bawah
            Layout.topMargin: 15
            Layout.bottomMargin: 20

            // Header Section
            ColumnLayout {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignHCenter // Rata tengah layoutnya
                spacing: 8

                Text {
                    text: "Keep the Engine Running"
                    Layout.alignment: Qt.AlignHCenter // Rata tengah teksnya
                    color: theme.colormap["playertitle"]
                    font.family: kodeMono.name
                    font.pixelSize: 18
                    font.bold: true
                }

                Text {
                    text: "Developing a low-latency audio engine in Rust takes a lot of time and even more coffee."
                    Layout.fillWidth: true
                    Layout.leftMargin: 20
                    Layout.rightMargin: 20
                    color: theme.colormap["playlisttext"]
                    font.family: kodeMono.name
                    font.pixelSize: 12
                    wrapMode: Text.WordWrap
                    horizontalAlignment: Text.AlignHCenter
                }
            }

            // --- SAWERIA SECTION ---
            ColumnLayout {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignHCenter
                spacing: 12

                Image {
                    Layout.preferredWidth: 200
                    Layout.preferredHeight: 200
                    Layout.alignment: Qt.AlignHCenter // WAJIB ADA INI
                    source: "qrc:/assets/images/saweriaqrcode.png"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                }

                Rectangle {
                    Layout.preferredWidth: 140
                    Layout.preferredHeight: 32
                    Layout.alignment: Qt.AlignHCenter // WAJIB ADA INI
                    radius: 4
                    color: "transparent"
                    border.color: saweriaArea.containsMouse ? theme.colormap["playeraccent"] : theme.colormap["graysolid"]
                    border.width: 1
                    
                    Text {
                        anchors.centerIn: parent
                        text: "Saweria Link"
                        color: saweriaArea.containsMouse ? theme.colormap["playeraccent"] : "white"
                        font.family: kodeMono.name
                        font.pixelSize: 11
                    }
                    MouseArea {
                        id: saweriaArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Qt.openUrlExternally("https://saweria.co/citzeye")
                    }
                }
            }

            // --- KO-FI SECTION ---
            ColumnLayout {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignHCenter
                spacing: 12

                Image {
                    Layout.preferredWidth: 200
                    Layout.preferredHeight: 200
                    Layout.alignment: Qt.AlignHCenter // WAJIB ADA INI
                    source: "qrc:/assets/images/kofiqrcode.png"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                }

                Rectangle {
                    Layout.preferredWidth: 140
                    Layout.preferredHeight: 32
                    Layout.alignment: Qt.AlignHCenter // WAJIB ADA INI
                    radius: 4
                    color: "transparent"
                    border.color: kofiArea.containsMouse ? theme.colormap["playeraccent"] : theme.colormap["graysolid"]
                    border.width: 1

                    Text {
                        anchors.centerIn: parent
                        text: "Ko-fi Link"
                        color: kofiArea.containsMouse ? theme.colormap["playeraccent"] : "white"
                        font.family: kodeMono.name
                        font.pixelSize: 11
                    }
                    MouseArea {
                        id: kofiArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Qt.openUrlExternally("https://ko-fi.com/citzeye")
                    }
                }
            }
        }
    }
}