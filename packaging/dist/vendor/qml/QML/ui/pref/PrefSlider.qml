/* --- loonixtunesv2/qml/ui/pref/PrefSlider.qml | PrefSlider --- */
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: rootSlider
    implicitHeight: 24
    Layout.fillWidth: true

    property string label: "Slider"
    property string valueText: ""
    property string description: ""
    property real fromValue: 0.0
    property real toValue: 100.0
    property real stepValue: 1.0
    property real currentValue: 50.0
    property real defaultValue: 50.0
    signal moved(real value)
    signal resetToDefault()

    RowLayout {
        anchors.fill: parent
        spacing: 10

        Text {
            text: rootSlider.label
            Layout.preferredWidth: 130
            font.family: kodeMono.name
            font.pixelSize: 12
            color: theme.colormap["playlisttext"]
            elide: Text.ElideRight
        }

        Item { Layout.fillWidth: true }

        Text {
            text: rootSlider.valueText
            font.family: kodeMono.name
            font.pixelSize: 12
            color: theme.colormap["playeraccent"]
        }

        Slider {
            id: slider
            Layout.maximumWidth: 80
            Layout.minimumWidth: 40
            Layout.fillHeight: true

            from: rootSlider.fromValue
            to: rootSlider.toValue
            stepSize: rootSlider.stepValue
            value: rootSlider.currentValue
            live: true
            onMoved: rootSlider.moved(value)

            WheelHandler {
                target: slider
                acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
                orientation: Qt.Vertical
                onWheel: function(event) {
                    var step = rootSlider.stepValue
                    var delta = event.angleDelta.y > 0 ? step : -step
                    var newVal = Math.max(rootSlider.fromValue, Math.min(rootSlider.toValue, slider.value + delta))
                    slider.value = newVal
                    rootSlider.moved(newVal)
                }
            }

            background: Rectangle {
                x: slider.leftPadding
                y: slider.topPadding + slider.availableHeight / 2 - height / 2
                width: slider.availableWidth
                height: 4
                radius: 2
                color: theme.colormap["graysolid"]
                Rectangle {
                    width: slider.visualPosition * parent.width
                    height: 4
                    radius: 2
                    color: theme.colormap["playeraccent"]
                }
            }

            handle: Rectangle {
                x: slider.leftPadding + slider.visualPosition * (slider.availableWidth - 10)
                y: slider.topPadding + slider.availableHeight / 2 - 5
                width: 10
                height: 10
                radius: 5
                color: slider.pressed ? theme.colormap["playerhover"] : theme.colormap["playeraccent"]
                border.color: theme.colormap["playeraccent"]
            }
        }
    }
}