import QtQuick 2.5
import QtQuick.Layouts 1.3
import ".."

GridView {
    property view_width: int
    
    id: pageList
    visible: StateManager.activePage === "pageList"
    Layout.fillWidth: pageList.visible ? true : false
    Layout.fillHeight: pageList.visible ? true: false
    Layout.preferredHeight: pageList.visible ? null : 0
    cellWidth: view_width / 4.
    cellHeight: 660
    clip: true

    interactive: false
    highlightMoveDuration: 0

    model: StateManager.pages


    header: Rectangle {
        width: view.width
        height: 80
        Text {
            anchors.centerIn: parent
            font.pointSize: 24
            text: "Page Overview"
        }
    }

    property int row: 0

    function scrollDownOneRow() {
        console.log(pageList.model.count/4, pageList.row);
        if((Math.ceil(pageList.model.count/4) - 4) >= pageList.row)  {
            pageList.row+=1;
        }
        pageList.positionViewAtIndex(pageList.row*4, GridView.Beginning);
    }
    function scrollUpOneRow() {
        if(pageList.row > 0) {
            pageList.row-=1;
        }
        pageList.positionViewAtIndex(pageList.row*4, GridView.Beginning);
    }

    SwipeDetection {
        id: listScroll
        anchors.fill: parent
        onChanged: () => {
            if(listScroll.swipeUp) {
                downstuff.visible = true;
                upstuff.visible = false;
            } else if (listScroll.swipeDown) {
                upstuff.visible = true;
                downstuff.visible = false;
            }
        }
        onReleased: () => {
            if(listScroll.swipeUp) {
                console.log("upupupt stestensten");
                pageList.scrollDownOneRow();
            } else if (listScroll.swipeDown) {
                console.log("downdowndown stestee");
                pageList.scrollUpOneRow();
            }
            downstuff.visible = false;
            upstuff.visible = false;
        }
    }

    delegate: Rectangle {
        width: pageList.cellWidth + 1
        height: pageList.cellHeight + 1
        border.width: 2
        border.color: "black"
        MouseArea {
            anchors.fill: parent
            onClicked: () => {
                backendImage.source = "";
                appload.sendMessage(9, index);
                activePage = "mangaReading"
            }
        }
        ColumnLayout {
            anchors.fill: parent
            spacing: -1
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Image {
                    anchors.fill: parent
                    fillMode: Image.PreserveAspectFit
                    source: pageUrl.includes("file:") ? `file:/tmp/mangarr/preview/${pageUrl.slice(18, -1)}g` : (pageUrl)
                    asynchronous: true
                }

                border.width: 2
                border.color: "black"
            }
            Rectangle {
                Layout.fillWidth: true
                border.width: 2
                border.color: "black"
                height: 100
                Text {
                    anchors.centerIn: parent
                    font.pointSize: 24
                    text: index + 1
                }
            }
        }
    }
}
