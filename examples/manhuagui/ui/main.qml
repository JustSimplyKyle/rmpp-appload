import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3

// Import your custom components
import "components"
import "definitions" // To get StateManager and BackendController

Rectangle {
    id: base
    anchors.fill: parent

    Loader {
        id: viewLoader
        anchors.fill: parent

        source: {
            switch (StateManager.activePage) {
                case StateManager.ActivePage.MangaReading:
                    return "components/MangaReadingView.qml";
                case StateManager.ActivePage.ChapterList:
                    return "components/ChapterListView.qml";
                case StateManager.ActivePage.PageList:
                    return "components/PageListView.qml";
                case StateManager.ActivePage.BookshelfView:
                    return "components/BookshelfView.qml";
                case StateManager.ActivePage.BackendSelection:
                    return "components/BackendSelectionView.qml";
                case StateManager.ActivePage.Search:
                    return "components/SearchView.qml";
            }
        }
    }

    Rectangle {
        z: 100000
        anchors.top: parent.top
        anchors.left: parent.left
        border.width: 2
        border.color: "red"
        width: 100
        height: 100
        MouseArea {
            anchors.fill: parent
            onClicked: () => { BackendController.sendMessage(99, "") }
        }
        Image {
            anchors.fill: parent
            anchors.margins: 20
            fillMode: Image.PreserveAspectFit
            source: "https://cdn-icons-png.flaticon.com/512/75/75519.png"
        }
    }
    

    Connections {
        target: StateManager

        function onBackendInitialized(TargetChapter, TargetPage) {
            console.log("initialized backend!");
            BackendController.sendMessage(14, "")
        }
    }

    Component.onCompleted: {
        BackendController.sendMessage(69420, "");
    }


    // solution from <https://forum.qt.io/topic/62267/how-we-can-create-2-second-delay-or-wait-in-qml/7>
    Timer {
        id: timer
        running: false
        repeat: false

        property var callback

        onTriggered: callback()
    }

    function setTimeout(callback, delay)
    {
        if (timer.running) {
            console.error("nested calls to setTimeout are not supported!");
            return;
        }
        timer.callback = callback;
        // note: an interval of 0 is directly triggered, so add a little padding
        timer.interval = delay + 1;
        timer.running = true;
    }

    Rectangle {
        z: 1
        anchors.top: parent.top
        anchors.right: parent.right
        border.width: 2
        border.color: "black"
        width: 150
        height: 100
        Text {
            id: box
            text: "Settings"
            font.pointSize: 24
            anchors.centerIn: parent
        }

        MouseArea {
            anchors.fill: parent
            onClicked: () => settings.visible = !settings.visible
        }

        Rectangle {
            id: settings
            visible: false
            anchors.top: parent.bottom
            anchors.right: parent.right
            width: content.implicitWidth
            height: content.implicitHeight
            ColumnLayout {
                id: content
                spacing: -1
                anchors.centerIn: parent
                anchors.right: parent.right
                Rectangle {
                    width: 300
                    height: 45
                    border.width: 2
                    border.color: "black"
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            StateManager.activePage = StateManager.ActivePage.ChapterList;
                            settings.visible = false;
                            BackendController.sendMessage(6, "")
                        }
                    }
                    Text {
                        font.pointSize: 24
                        text: "Chapter Selection"
                    }
                }
                Rectangle {
                    width: 300
                    height: 45
                    border.width: 2
                    border.color: "black"
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            StateManager.activePage = StateManager.ActivePage.PageList
                            settings.visible = false;
                        }
                    }
                    Text {
                        font.pointSize: 24
                        text: "Page Overview"
                    }
                }
                Rectangle {
                    width: 300
                    height: 45
                    border.width: 2
                    border.color: "black"
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            StateManager.activePage = StateManager.ActivePage.Search
                            settings.visible = false;
                        }
                    }
                    Text {
                        font.pointSize: 24
                        text: "Search Manga"
                    }
                }
                Rectangle {
                    width: 300
                    height: 45
                    border.width: 2
                    border.color: "black"
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            StateManager.activePage = StateManager.ActivePage.BackendSelection
                            settings.visible = false;
                        }
                    }
                    Text {
                        font.pointSize: 24
                        text: "Backend Selection"
                    }
                }
                Rectangle {
                    width: 300
                    height: 45
                    border.width: 2
                    border.color: "black"
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            settings.visible = false;
                            BackendController.sendMessage(12, "");
                        }
                    }
                    Text {
                        font.pointSize: 24
                        text: "Add to bookshelf"
                    }
                }
                Rectangle {
                    width: 300
                    height: 45
                    border.width: 2
                    border.color: "black"
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            BackendController.sendMessage(14, "");
                            StateManager.activePage = StateManager.ActivePage.BookshelfView
                            settings.visible = false;
                        }
                    }
                    Text {
                        font.pointSize: 24
                        text: "View Bookshelf"
                    }
                }
            }
        }
    }
}
