import QtQuick 2.5
import QtQuick.Layouts 1.3
import "../definitions"
import "."

ColumnLayout {
    anchors.fill: parent
    GridView {
        id: pageList
        Layout.fillWidth: true
        Layout.fillHeight: true
        cellWidth: pageList.width / 4.
        cellHeight: 660
        clip: true


        interactive: false
        highlightMoveDuration: 0

        model: StateManager.totalPage

        header: Rectangle {
            width: pageList.width
            height: 80
            Text {
                anchors.centerIn: parent
                font.pointSize: 24
                text: "Page Overview"
            }
        }

        property int row: 0

        function scrollDownOneRow() {
            console.log(pageList.model/4, pageList.row);
            if((Math.ceil(pageList.model/4) - 4) >= pageList.row)  {
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

        Connections {
            target: StateManager

            function onPageViewUpdated(TargetChapter, TargetPage) {
                pageList.row = StateManager.currPage/4 - 1;
                pageList.positionViewAtIndex(pageList.row*4, GridView.Beginning);
            }
        }

        

        // deal with this later
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

        delegate: Rectangle {
            width: pageList.cellWidth + 1
            height: pageList.cellHeight + 1
            border.width: 2
            border.color: "black"
            property string pageUrl: getPageUrl() 

            function getPageUrl() {
                let chapter = StateManager.pages.get(StateManager.currChpt);
                if(chapter) {
                    let page = chapter.get(index) ?? "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png";
                    return page;
                } else {
                    return "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png";
                }
            }
        
            Connections {
                target: StateManager

                function onPagesUpdated(TargetChapter, TargetPage) {
                    if(TargetChapter === StateManager.currChpt && TargetPage === index ) {
                        pageUrl = getPageUrl();
                    }
                }
            }


            MouseArea {
                anchors.fill: parent
                onClicked: () => {
                    BackendController.sendMessage(9, index);
                    StateManager.activePage = StateManager.ActivePage.MangaReading
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

    BottomBar {
        id: bottomBar
    }
}
