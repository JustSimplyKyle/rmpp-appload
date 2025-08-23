import QtQuick 2.5

Rectangle {
    id: swipeArea
    color: "transparent" 

    property real startY: 0
    property real threshold: 100

    property bool swipeUp: false
    property bool swipeDown: false
    property bool allowSend: false

    signal changed()
    signal released()

    MouseArea {
        anchors.fill: parent
        
        acceptedButtons: Qt.LeftButton
        propagateComposedEvents: allowSend
        onPressed: (mouse) => {
            swipeArea.startY = mouse.y;
        }
        onPositionChanged: (mouse) => {
            let deltaY = mouse.y - swipeArea.startY;

            if (deltaY < -swipeArea.threshold) {
                swipeArea.swipeUp = true;
                swipeArea.swipeDown = false;
            } else if (deltaY > swipeArea.threshold) {
                swipeArea.swipeDown = true;
                swipeArea.swipeUp = false;
            }
            swipeArea.changed();
        }
        onReleased: {
            swipeArea.released();
            swipeArea.startY = 0;
            if(swipeArea.swipeUp || swipeArea.swipeDown) {
                allowSend = false;
            } else {
                allowSend = true;
            }
            swipeArea.swipeUp = false;
            swipeArea.swipeDown = false;
        }
    }
}


