import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3
import net.asivery.AppLoad 1.0


Rectangle {
    anchors.fill: parent
    id: base
    property string manga_id: ""
    property var chapters: []
    property string activePage: "mangaReading"
    property var pages

    Component.onCompleted: {
        pages = new Map();
    }

    // This is an endpoint. There can be multiple endpoints throughout one application
    // All endpoints will get all messages sent from the backend
    AppLoad {
        id: appload
        applicationID: "manhuagui"
        onMessageReceived: (type, contents) => {
            switch(type){
                case 101:
                    // const s = pages.get(currChpt.text).get(parseInt(currPage.text)-1).pageUrl;
                    const chapter = pages.get(currChpt.text);
                    if(!chapter) {
                        backendImage.source = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    const pagef = chapter.get(parseInt(currPage.text) - 1);
                    if(!pagef) {
                        backendImage.source = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    const url = pagef.pageUrl;
                    if(!url) {
                        backendImage.source = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    backendImage.source = url;
                    break;
                case 11:
                    stat.text = `${contents}`
                    break;
                case 2:
                    break;
                case 4:
                    currPage.text = `${contents}`
                    pageSeparater.text = "/"
                    pageStarter.text = "page:"
                    break;
                case 5:
                    totalPage.text = `${contents}`
                    pageSeparater.text = "/"
                    pageStarter.text = "page:"
                    break;
                case 6:
                    currChpt.text = `${contents}`

                    if(!pages.get(currChpt.text)) {
                        const obj = Qt.createQmlObject(`
                            import QtQuick
                            ListModel {}
                        `, base, "myDynamicSnippet");
                        pages.set(currChpt.text, obj);
                    }
                    chptSeparater.text = "/"
                    chptStarter.text = "chapter:"
                    break;
                case 7:
                    totalChpt.text = `${contents}`
                    chptSeparater.text = "/"
                    chptStarter.text = "chapter:"
                    break;
                case 8:
                    chapters = contents.split('\n')
                    chapterList.model = chapters.length
                    break;
                case 9:
                    const targetChpt = contents;
                    let pageModel = pages.get(targetChpt);
                    for(let i = 0; i < parseInt(totalPage.text); i++) {
                        const page = pageModel.get(i);
                        if(!page) {
                            pageModel.append({ "pageUrl": "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png" });
                        }
                    }
                    if (parseInt(targetChpt) === parseInt(currChpt.text)) {
                        pageList.model = pageModel;
                        pageList.forceLayout();
                    }
                    break;
                case 10:
                    const arr = contents.split('\n');
                    const targetChapter = arr[0];
                    const targetPage = arr[1];
                    const path = arr[2];
                    const ageModel = pages.get(targetChapter);
                    ageModel.get(targetPage).pageUrl = path;
                    if(parseInt(targetChapter) === parseInt(currChpt.text)) {
                        if(parseInt(targetPage) + 1 === parseInt(currPage.text)) {
                            backendImage.source = path;
                        }
                        pageList.model = ageModel;
                        pageList.forceLayout();
                        // pageList.model.get(targetPage).pageUrl = path;
                    }
            }
        }
    }

    Popup {
        id: popup
        x: 0
        // y: view.height - popup.height - 80
        y: view.height - popup.height
        closePolicy: Popup.CloseOnPressOutside
        width: parent.width
        height: 120
        Rectangle {
            anchors.fill: parent
            height: parent.height

            ColumnLayout {
                anchors.fill: parent
                Layout.alignment: Qt.AlignVCenter | Qt.AlignHCenter
                Text {
                    font.pointSize: 36
                    text: slider.value
                }
                Slider {
                    id: slider
                    Layout.preferredWidth: parent.width
                    snapMode: Slider.SnapAlways
                    // live: false
                    Component.onCompleted: {
                        slider.handle.implicitWidth = 36;
                        slider.handle.implicitHeight = 36;
                        slider.handle.radius = 26;
                        slider.handle.color = slider.pressed ? "#f0f0f0" : "#f6f6f6";
                    }
                    onMoved: () => {
                        appload.sendMessage(9, slider.value - 1);
                    }
                    stepSize: 1
                    from: 1
                    value: 1
                    to: parseInt(totalPage.text)
                }
            }
        }
    }

    Rectangle {
        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
        }
        height: 80
        SwipeDetection {
            id: swipew
            anchors.fill: parent
            onReleased: () => {
                if(swipew.swipeUp) {
                    popup.open();
                } else if (swipew.swipeDown) {
                    popup.close();
                }
            }
        }
    }

    Rectangle {
        z: 1
        anchors.top: view.top
        anchors.right: view.right
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
                            activePage = "chapterList"
                            appload.sendMessage(6, "")
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
                            activePage = "pageList"
                            // appload.sendMessage(8, "")
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
                    Text {
                        font.pointSize: 24
                        text: "Change chapter"
                    }
                }
                Rectangle {
                    width: 300
                    height: 45
                    border.width: 2
                    border.color: "black"
                    Text {
                        font.pointSize: 24
                        text: "Change chapter"
                    }
                }
            }
        }
    }

    RowLayout {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: parent.height - layout.implicitHeight
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
    

    ColumnLayout {
        id: view
        clip: true
        anchors.fill: parent

        Image {
            id: backendImage
            visible: activePage === "mangaReading"
            Layout.preferredHeight: backendImage.visible ? null : 0
            Layout.fillWidth: backendImage.visible ? true : false
            Layout.fillHeight: backendImage.visible ? true : false
            fillMode: Image.PreserveAspectFit
            RowLayout {
                anchors.fill: parent
                MouseArea {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    onClicked: () =>  appload.sendMessage(3, "") 
                }
                MouseArea {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    onClicked: () => appload.sendMessage(2, "")
                }
            }
        }
        GridView {
            id: chapterList
            visible: activePage === "chapterList"
            Layout.fillWidth: chapterList.visible ? true : false
            Layout.fillHeight: chapterList.visible ? true: false
            Layout.preferredHeight: chapterList.visible ? null : 0
            cellWidth: view.width
            cellHeight: 100
            model: 0
            header: Rectangle {
                width: view.width
                height: 80
                Text {
                    anchors.centerIn: parent
                    font.pointSize: 24
                    text: "Chapter Selection"
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
                            backendImage.source = "";
                            appload.sendMessage(7, index);
                            activePage = "mangaReading"
                        }
                    }
                    Text {
                        font.pointSize: 36
                        text: chapters[index]
                    }
                }
            }
        }
        GridView {
            id: pageList
            visible: activePage === "pageList"
            Layout.fillWidth: pageList.visible ? true : false
            Layout.fillHeight: pageList.visible ? true: false
            Layout.preferredHeight: pageList.visible ? null : 0
            cellWidth: view.width / 4.
            cellHeight: 660
            clip: true

            interactive: false
            highlightMoveDuration: 0

            model: pages.get(currChpt.text)

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
                        console.log("upupup");
                        pageList.scrollDownOneRow();
                    } else if (listScroll.swipeDown) {
                        console.log("downdowndown");
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
                            source: pageUrl
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

        RowLayout {
            id: layout
            Layout.fillWidth: true

            Rectangle {
                border.width: 2
                border.color: "black"
                width: 100
                height: 100
                 Text {
                    text: "Close"
                    font.pointSize: 24
                    anchors.centerIn: parent
                }

                MouseArea {
                    anchors.fill: parent
                    // onClicked: () => popup.open()
                    onClicked: () => appload.sendMessage(99, "")
                }
            }

            Rectangle {
                id: button
                border.width: 2
                border.color: "black"
                width: 400
                Layout.preferredHeight: numpad.visible ? 600 : 60
                
                MouseArea {
                    anchors.top: parent.top
                    anchors.left: parent.left
                    height: 60
                    width: 600
                    onClicked: () => numpad.visible = !numpad.visible
                }

                ColumnLayout {
                    anchors.fill: parent
                    Rectangle {
                        width: 300
                        height: 60
                        color: "transparent"
                        Text {
                            font.pointSize: 24
                            text: "mangid: " + manga_id
                        }
                    }
                    GridLayout {
                        id: numpad
                        visible: true
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        rowSpacing: -1
                        columnSpacing: -1
                        columns: 3
                        rows: 4 

                        // Number buttons (1-9)
                        Repeater {
                            model: 9
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: "black"
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: () => manga_id += String(index + 1)
                                }
                                Text {
                                    anchors.centerIn: parent
                                    font.pointSize: 24
                                    text: String(index + 1)
                                }
                            }
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            border.width: 2
                            border.color: "black"
                            MouseArea {
                                anchors.fill: parent
                                onClicked: () => manga_id += String(0)
                            }
                            Text {
                                anchors.centerIn: parent
                                font.pointSize: 24
                                text: String(0)
                            }
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            border.width: 2
                            border.color: "black"
                            MouseArea {
                                anchors.fill: parent
                                onClicked: () => { appload.sendMessage(1, "id:" + manga_id); manga_id = ""; numpad.visible = false }
                            }
                            Text {
                                anchors.centerIn: parent
                                font.pointSize: 24
                                text: "Enter!"
                            }
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            border.width: 2
                            border.color: "black"
                            MouseArea {
                                anchors.fill: parent
                                onClicked: () => manga_id = ""
                            }
                            Text {
                                anchors.centerIn: parent
                                font.pointSize: 24
                                text: "Clear!"
                            }
                        }
                    }
                }
            }

            Rectangle {
                id: prev
                border.width: 2
                border.color: "black"
                width: 300
                height: 150

                Text {
                    text: "Prev chapter"
                    font.pointSize: 24
                    anchors.centerIn: parent
                }
                MouseArea {
                    anchors.fill: parent
                    onClicked: () => appload.sendMessage(4, "")
                }
            }


            Rectangle {
                id: next
                border.width: 2
                border.color: "black"
                width: 300
                height: 150

                Text {
                    text: "Next chapter"
                    font.pointSize: 24
                    anchors.centerIn: parent
                }

                MouseArea {
                    anchors.fill: parent
                    onClicked: () => appload.sendMessage(5, "")
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Text {
                    id: stat
                    font.pointSize: 24
                }

                RowLayout {
                    Layout.fillWidth: true
                    Text {
                        id: pageStarter
                        font.pointSize: 24
                    }
                    
                    Text {
                        id: currPage
                        font.pointSize: 24
                    }

                    Text {
                        id: pageSeparater
                        font.pointSize: 24
                    }

                    Text {
                        id: totalPage
                        font.pointSize: 24
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    Text {
                        id: chptStarter
                        font.pointSize: 24
                    }
                    
                    Text {
                        id: currChpt
                        font.pointSize: 24
                    }

                    Text {
                        id: chptSeparater
                        font.pointSize: 24
                    }

                    Text {
                        id: totalChpt
                        font.pointSize: 24
                    }
                }
            }
        }
    }
}
