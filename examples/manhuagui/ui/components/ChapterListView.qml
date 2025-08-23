import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3
import "../definitions"

ColumnLayout {
    id: view
    anchors.fill: parent
    GridView {
        id: chapterList
        Layout.fillWidth: true
        Layout.fillHeight: true
        cellWidth: parent.width
        cellHeight: 100
        model: StateManager.chapters

        clip: true
        interactive: false
        highlightMoveDuration: 0

        header: Rectangle {
            width: view.width
            height: 80
            Text {
                anchors.centerIn: parent
                font.pointSize: 24
                text: "Chapter Selection"
            }
        }

        property int row: 0

        function scrollDownOneRow() {
            let diff = chapterList.model - chapterList.row;
            console.log(diff);
            if(0 <= diff && diff < 19) {
                // chapterList.row+=diff;
            } else {
                chapterList.row+=5;
            }
            chapterList.positionViewAtIndex(chapterList.row, GridView.Beginning);
        }
        function scrollUpOneRow() {
            if(0 <= chapterList.row && chapterList.row <= 5) {
                chapterList.row=0;
            } else {
                chapterList.row-=5;
            }
            chapterList.positionViewAtIndex(chapterList.row, GridView.Beginning);
        }

        SwipeDetection {
            id: chapterlistScroll
            anchors.fill: parent
            onChanged: () => {
                if(chapterlistScroll.swipeUp) {
                    downstuff.visible = true;
                    upstuff.visible = false;
                } else if (chapterlistScroll.swipeDown) {
                    upstuff.visible = true;
                    downstuff.visible = false;
                }
            }
            onReleased: () => {
                if(chapterlistScroll.swipeUp) {
                    console.log("upupup");
                    chapterList.scrollDownOneRow();
                } else if (chapterlistScroll.swipeDown) {
                    console.log("downdowndown");
                    chapterList.scrollUpOneRow();
                }
                downstuff.visible = false;
                upstuff.visible = false;
            }
        }

    
        RowLayout {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: parent.height - bottomBar.implicitHeight

            z: 5
            Rectangle {
                Layout.alignment: Qt.AlignHCenter | Qt.AlignTop
                Layout.preferredWidth: 60
                Layout.preferredHeight: 60
                id: upstuff
                visible: false
                color: "black"
            }
            Rectangle {
                id: downstuff
                visible: false
                Layout.alignment: Qt.AlignHCenter | Qt.AlignBottom
                width: 60
                height: 60
                color: "black"
            }
        }

        delegate: RowLayout {
            spacing: -1
            Rectangle {
                width: 100
                height: chapterList.cellHeight + 1
                border.width: 2
                border.color: "black"
                Text {
                    anchors.centerIn: parent
                    font.pointSize: 36
                    text: index + 1
                }
            }
            Rectangle {
                width: view.width - 100
                height: chapterList.cellHeight + 1
                border.width: 2
                border.color: "black"
                MouseArea {
                    anchors.fill: parent
                    onClicked: () => {
                        BackendController.sendMessage(7, index);
                        StateManager.activePage = StateManager.ActivePage.MangaReading;
                    }
                }
                Text {
                    font.pointSize: 36
                    text: modelData
                }
            }
        }
    }
    BottomBar {
        id: bottomBar
    }
}

