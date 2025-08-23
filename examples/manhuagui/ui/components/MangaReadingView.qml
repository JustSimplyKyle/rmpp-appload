import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3
import "../definitions"
import "."

ColumnLayout {
    clip: true
    anchors.fill: parent

    Image {
        id: backendImage
        source: StateManager.currentImageUrl
        Layout.fillWidth: true
        Layout.fillHeight: true
        fillMode: Image.PreserveAspectFit
        RowLayout {
            anchors.top: parent.top  
            anchors.topMargin: 100.0 // allow swipe down to happen
            anchors.left: parent.left
            anchors.right: parent.right
            height: parent.height
            MouseArea {
                Layout.fillWidth: true
                Layout.fillHeight: true
                onClicked: () =>  BackendController.sendMessage(3, "") 
            }
            MouseArea {
                Layout.fillWidth: true
                Layout.fillHeight: true
                onClicked: () => BackendController.sendMessage(2, "")
            }
        }
    }

    BottomBar {
        id: bottomBar
    }
}
