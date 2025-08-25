import QtQuick 2.5
import QtQuick.Window 2.2
import QtQuick.Controls 2.5
import net.asivery.AppLoad 1.0
import net.asivery.Framebuffer 1.0

FocusScope {
    id: root
    Drag.active: topbarDragHandle.drag.active && !fullscreen
    implicitWidth: 500
    implicitHeight: 700
    width: root.globalWidth / 3
    height: _height + topbar.height
    z: 100

    property int globalWidth: 0
    property int globalHeight: 0
    readonly property int minWidth: 400
    readonly property int minHeight: 700


    property alias appName: _appName.text
    property bool supportsScaling: false
    property var qtfbKey: -1
    property int appPid: -1
    property bool minimized: false
    property bool fullscreen: false
    property bool forceTopBarVisible: false
    // This is defined like that so as it doesn't interfere with minimization
    property var _height: root.globalHeight / 3
    property var beforeFullscreenData: null

    // External I/O from this component:
    signal closed
    function loadApplication(appId) {
        coord.loadApplication(appId);
    }
    function maximize() {
        if(fullscreen) {
            // Is it already maximized?
            // Restore the previous config, and bring the window back.
            root.fullscreen = false;
            root.x = beforeFullscreenData.x
            root.y = beforeFullscreenData.y
            root.width = Math.max(minWidth, beforeFullscreenData.width)
            root._height = Math.max(minHeight, beforeFullscreenData.height)
            root.minimized = beforeFullscreenData.minimized;
            root.beforeFullscreenData = null;
        } else {
            // Bring the window to fullscreen.
            root.beforeFullscreenData = {
                x: root.x,
                y: root.y,
                width: root.width,
                height: root._height,
                minimized: root.minimized
            };
            root.x = 0;
            root.y = 0;
            root.width = root.globalWidth;
            root._height = root.globalHeight;
            root.minimized = false;
            root.fullscreen = true;
        }
    }

    AppLoadLibrary {
        id: library
        onPidDied: (pid) => {
            if(pid == appPid) {
                coord.close();
                root.closed();
            }
        }
    }

    function stopUnderlying() {
        if(appPid != -1) {
            library.terminateExternal(appPid);
        }
    }

    states: [
        State {
            name: "minimized"
            when: minimized
            PropertyChanges {
                target: root
                implicitHeight: topbar.height
                height: topbar.height
            }

            PropertyChanges {
                target: mainWindowView
                visible: false
            }
        },
        State {
            name: "maximized"
            when: fullscreen

            // Do not make the window's contents "jump" when topbar reveals itself.
            PropertyChanges {
                target: mainWindowView
                height: root.globalHeight
                anchors.top: parent.top
            }
        }
    ]

    // Swiping from the top will activate the topbar in fullscreen, and do nothing in any other case
    Timer {
        id: topBarHideTimer
        interval: 3000
        repeat: false
        onTriggered: () => forceTopBarVisible = false
    }
    function swipeFromTheTop() {
        if(fullscreen && !forceTopBarVisible) {
            forceTopBarVisible = true;
            topBarHideTimer.start();
        }
    }

    MouseArea {
        id: resizeHandle
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        width: 100
        height: 100
        cursorShape: Qt.SizeFDiagCursor
        enabled: !fullscreen && !minimized
        z: 10001

        property real startX: 0
        property real startY: 0

        onPressed: {
            startX = mouse.x
            startY = mouse.y
        }

        onReleased: {
            let deltaX = mouse.x - startX
            let deltaY = mouse.y - startY

            let width = Math.max(minWidth, root.width + deltaX)
            let height = Math.max(minHeight, root.height + deltaY)
            startX = mouse.x
            startY = mouse.y

            if(!supportsScaling) {
                let scale = Math.min(height / root.globalHeight, width / root.globalWidth);

                width = root.globalWidth * scale
                height = root.globalHeight * scale + topbar.height
            }

            root.width = width;
            root._height = height;
        }
    }

    Rectangle {
        anchors.fill: parent
        color: "transparent"
        border.width: 2
        border.color: "black"
        z: 101
        visible: !topbarDragHandle.drag.active
    }

    Rectangle {
        id: topbar
        visible: !fullscreen || forceTopBarVisible
        width: parent.width
        height: visible ? 75 : 0
        color: "white"
        z: 2001

        border.width: 2
        border.color: "black"

        Text {
            id: _appName
            anchors.fill: parent
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            font.pointSize: 20
            text: "<AppLoad Window>"
        }

        MouseArea {
            id: topbarDragHandle
            anchors.fill: parent
            drag.target: root
            enabled: !fullscreen

            onClicked: () => {
                root.forceActiveFocus();
            }
        }

        Rectangle {
            id: closeButton
            width: parent.height
            height: parent.height
            anchors.right: parent.right
            border.width: 2
            border.color: "black"
            color: parent.color

            Text {
                anchors.fill: parent
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
                text: "X"
                font.pointSize: 20
            }

            MouseArea {
                anchors.fill: parent
                onClicked: () => {
                    coord.close();
                    root.closed();
                    stopUnderlying();
                }

                onPressAndHold: () => {
                    coord.terminate();
                    root.closed();
                    stopUnderlying();
                }
            }
        }
        Rectangle {
            id: maximizeButton
            width: parent.height
            height: parent.height
            anchors.right: closeButton.left
            border.width: 2
            border.color: "black"
            color: parent.color

            Text {
                anchors.fill: parent
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
                text: fullscreen ? "▭" : "□"
                font.pointSize: 20
            }

            MouseArea {
                anchors.fill: parent
                onClicked: () => {
                    maximize()
                }
            }
        }
        Rectangle {
            id: minimizeButton
            width: parent.height
            height: parent.height
            anchors.right: maximizeButton.left
            border.width: 2
            border.color: "black"
            color: parent.color

            Text {
                anchors.fill: parent
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
                text: "_"
                font.pointSize: 20
            }

            MouseArea {
                anchors.fill: parent
                onClicked: () => {
                    if(fullscreen) maximize(); // Cannot have a full screen minimized app.
                    minimized = !minimized;
                }
            }
        }

        Rectangle {
            width: parent.width
            height: 2
            anchors.bottom: parent.bottom
            border.width: 2
            border.color: "black"
        }
    }
    Rectangle {
        id: mainWindowView
        anchors.top: topbar.bottom
        height: parent.height - topbar.height
        width: parent.width
        clip: true
        visible: !topbarDragHandle.drag.active

        MouseArea {
            id: qmldiffEventBlocker
            anchors.fill: parent
        }

        FBController {
            id: windowCanvas
            anchors.fill: parent
            visible: qtfbKey != -1
            allowScaling: true
            framebufferID: qtfbKey
            focus: qtfbKey != -1

            onActiveChanged: () => {
                if(!windowCanvas.active) {
                    // Application closed.
                    root.closed();
                }
            }

            onDragDown: () => {
                swipeFromTheTop();
            }
        }

        AppLoadCoordinator {
            id: coord

            onUnloading: () => {
                if(!coord.loaded) return;
                let unloadingFunction;
                if(supportsScaling) {
                    unloadingFunction = loader.item.unloading;
                } else {
                    unloadingFunction = loaderScaled.item.unloading;
                }
                if(unloadingFunction) unloadingFunction();
                root.closed();
            }
        }

        Loader {
            id: loaderScaled
            source: coord.applicationQMLRoot
            visible: !supportsScaling
            active: coord.loaded && !supportsScaling
            anchors.centerIn: parent

            width: root.globalWidth
            height: root.globalHeight

            transform: Scale {
                origin.x: loaderScaled.width / 2
                origin.y: loaderScaled.height / 2
                xScale: Math.min(root.width / loaderScaled.width, root.height / loaderScaled.height)
                yScale: Math.min(root.width / loaderScaled.width, root.height / loaderScaled.height)
            }

            Connections {
                target: loaderScaled.item
                function onClose() {
                    coord.close();
                }
            }
        }
        Loader {
            id: loader
            source: coord.applicationQMLRoot
            visible: supportsScaling
            active: coord.loaded && supportsScaling
            anchors.fill: parent

            Connections {
                target: loader.item
                function onClose() {
                    coord.close();
                }
            }
        }
    }

    Button {
        id: qmldiffReplaceMeButton

        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 20
        z: 1600
        visible: fullscreen && !forceTopBarVisible
        text: "<Swipe>"
        onClicked: swipeFromTheTop()
    }
}
