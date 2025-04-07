import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3
import net.asivery.AppLoad 1.0


import xofm.libs.epaper as Epaper

Rectangle {
    anchors.fill: parent
    id: base
    property string manga_id: ""
    property var chapters: []
    property string activePage: "mangaReading"
    property string activeBackend: "Manhuagui"
    property var pages
    property var bookshelf: ({})
    property int currPage
    property int totalPage
    property int currChpt
    property int totalChpt

    Rectangle {
        anchors.fill: parent
        visible: error.text === "" ? false : true
        z: 1000
        Text {
            id: error
            text: ""
            font.pointSize: 36
        }
    }

    Component.onCompleted: {
        pages = new Map();
        bookshelf = new Map();
    }

    // This is an endpoint. There can be multiple endpoints throughout one application
    // All endpoints will get all messages sent from the backend
    AppLoad {
        id: appload
        applicationID: "manhuagui"
        onMessageReceived: (type, contents) => {
            switch(type){
                case 101:
                    const chapter = pages.get(currChpt);
                    if(!chapter) {
                        backendImage.source = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    const pagef = chapter.get(parseInt(currPage) - 1);
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
                case 1000:
                    error.text = contents;
                    break;
                case 11:
                    // stat.text = `${contents}`
                    break;
                case 2:
                    break;
                case 4:
                    currPage = parseInt(`${contents}`)
                    break;
                case 5:
                    totalPage = parseInt(`${contents}`)
                    break;
                case 6:
                    currChpt = parseInt(`${contents}`)

                    if(!pages.get(currChpt)) {
                        const obj = Qt.createQmlObject(`
                            import QtQuick
                            ListModel {}
                        `, base, "myDynamicSnippet");
                        pages.set(currChpt, obj);
                    }
                    break;
                case 7:
                    totalChpt = `${contents}`
                    break;
                case 8:
                    chapters = contents.split('\n')
                    chapterList.model = chapters.length
                    break;
                case 9:
                    const targetChpt = parseInt(contents);
                    let pageModel = pages.get(targetChpt);
                    for(let i = 0; i < totalPage; i++) {
                        const page = pageModel.get(i);
                        if(!page) {
                            pageModel.append({ "pageUrl": "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png" });
                        }
                    }
                    if (targetChpt === currChpt) {
                        pageList.model = pageModel;
                        pageList.forceLayout();
                    }
                    break;
                case 10:
                    const arr = contents.split('\n');
                    const targetChapter = parseInt(arr[0]);
                    const targetPage = parseInt(arr[1]);
                    const path = arr[2];
                    const ageModel = pages.get(targetChapter);
                    ageModel.get(targetPage).pageUrl = path;
                    if(targetChapter === currChpt) {
                        if(targetPage + 1 === currPage) {
                            backendImage.source = path;
                        }
                        pageList.model = ageModel;
                        pageList.forceLayout();
                        // pageList.model.get(targetPage).pageUrl = path;
                    }
                    break;
                case 12:
                    const s1 = contents;
                    description.text = s1;
                    break;
                case 13:
                    const s2 = contents;
                    author.name = s2;
                    break;
                case 14:
                    const s3 = contents;
                    backendImage.source = s3;
                    break;
                case 15:
                    mangaName.text = `${contents}`;
                    break;
                case 16:
                    dateDisplay.date = contents;
                    break;
                case 17:
                    const array = JSON.parse(contents);
                    const key = array.url;

                    bookshelf.set(key, array);

                    bookshelfModel.clear();


                    for(let ke = 0;ke<bookshelf.size;ke++) {
                        // console.log(key, Array.from(bookshelf.values())[ke].lastReadPage);
                        bookshelfModel.set(ke, Array.from(bookshelf.values())[ke]);
                    }

                    bookshelfView.forceLayout();
                    break;
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
                    to: totalPage
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
        anchors.left: view.left
        border.width: 2
        border.color: "red"
        width: 100
        height: 100
        MouseArea {
            anchors.fill: parent
            onClicked: () => { appload.sendMessage(99, "") }
        }
        Image {
            anchors.fill: parent
            anchors.margins: 20
            fillMode: Image.PreserveAspectFit
            source: "https://cdn-icons-png.flaticon.com/512/75/75519.png"
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
                            settings.visible = false;
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
                            settings.visible = false;
                            pageList.row = currPage/4 - 1;
                            pageList.positionViewAtIndex(pageList.row*4, GridView.Beginning);
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
                            activePage = "mangaReading"
                            settings.visible = false;
                            selectionFooter.visible = true
                            pages = new Map();
                            backendImage.source = ""
                            manga_id = "";
                            totalChpt = 0;
                            description.text = "";
                            author.name = "";
                            dateDisplay.date = "";
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
                            activePage = "backendSelection"
                            settings.visible = false;
                            selectionFooter.visible = false
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
                            appload.sendMessage(12, "");
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
                            appload.sendMessage(14, "");
                            activePage = "bookshelfView"
                            settings.visible = false;
                            selectionFooter.visible = false
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

        ListModel {
            id: bookshelfModel
        }
        GridView {
            id: bookshelfView
            visible: activePage === "bookshelfView"
            Layout.fillWidth: bookshelfView.visible ? true : false
            Layout.fillHeight: bookshelfView.visible ? true: false
            Layout.preferredHeight: bookshelfView.visible ? null : 0
            cellWidth: view.width
            cellHeight: view.height / 7
            model: bookshelfModel
            header: Rectangle {
                width: view.width
                height: 80
                Text {
                    anchors.centerIn: parent
                    font.pointSize: 24
                    text: "Bookshelf"
                }
            }

            delegate: RowLayout {
                width: bookshelfView.cellWidth
                height: bookshelfView.cellHeight
                spacing: -1
                Rectangle {
                    Layout.preferredWidth: 250
                    Layout.preferredHeight: bookshelfView.cellHeight + 1
                    border.width: 2
                    border.color: "black"
                    Text {
                        anchors.centerIn: parent
                        font.pointSize: 36
                        text: index + 1
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            appload.sendMessage(13, `${backend}\n${url}`);
                            activePage = "mangaReading"
                            pages = new Map();
                            backendImage.source = ""
                            manga_id = "";
                            totalChpt = 0;
                        }
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: bookshelfView.cellHeight + 1
                    border.width: 2
                    border.color: "black"
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            appload.sendMessage(13, `${backend}\n${url}`);
    
                            activePage = "mangaReading"
                            pages = new Map();
                            backendImage.source = ""
                            manga_id = "";
                            totalChpt = 0;
                        }
                    }
                    ColumnLayout {
                        anchors.fill: parent
                        spacing: -1
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 100
                            border.width: 2
                            border.color: "black"
                            Text {
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.leftMargin: 30
                                font.pointSize: 36
                                text: title
                            }
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            border.width: 2
                            border.color: "black"
                            RowLayout {
                                anchors.fill: parent
                                spacing: -1
                                Rectangle {
                                    Layout.preferredWidth: 300
                                    Layout.fillHeight: true
                                    border.width: 2
                                    border.color: "black"
                                    ColumnLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 30
                                        Item {
                                            Layout.alignment: Qt.AlignTop
                                            Layout.topMargin: 40
                                            Layout.fillWidth: true
                                            Layout.fillHeight: true
                                            Text {
                                                text: `Page: ${lastReadPage}/${totalPages}`
                                                font.pointSize: 24
                                            }
                                        }
                                        Item {
                                            Layout.alignment: Qt.AlignBottom
                                            Layout.bottomMargin: 30
                                            Layout.fillWidth: true
                                            Layout.fillHeight: true
                                            Text {
                                                text: `Chapter: ${lastReadChapter}/${totalChapters}`
                                                font.pointSize: 24
                                            }
                                        }
                                    }
                                }
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    border.width: 2
                                    border.color: "black"
                                    clip: true
                                    Text {
                                        anchors.left: parent.left
                                        anchors.right: parent.right
                                        anchors.leftMargin: 30
                                        anchors.topMargin: 20
                                        anchors.bottomMargin: 20
                                        wrapMode: Text.Wrap
                                        font.pointSize: 20
                                        text: description
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        ListModel {
            id: backendModel
            ListElement {
                name: "NHentai"
                desc: "Requires manual setup of the cf authenticatino token."
            }
            ListElement {
                name: "Manhuagui"
                desc: "The default backend. Works without any configuration."
            }
        }
        GridView {
            id: backendSelection
            visible: activePage === "backendSelection"
            Layout.fillWidth: backendSelection.visible ? true : false
            Layout.fillHeight: backendSelection.visible ? true: false
            Layout.preferredHeight: backendSelection.visible ? null : 0
            cellWidth: view.width
            cellHeight: 100
            model: backendModel
            header: Rectangle {
                width: view.width
                height: 80
                Text {
                    anchors.centerIn: parent
                    font.pointSize: 24
                    text: "Backend Selection"
                }
            }

            delegate: RowLayout {
                spacing: -1
                Rectangle {
                    width: 300
                    height: backendSelection.cellHeight + 1
                    border.width: 2
                    border.color: "black"
                    Text {
                        anchors.centerIn: parent
                        font.pointSize: 36
                        text: name
                    }
                }
                Rectangle {
                    width: view.width - 300
                    height: chapterList.cellHeight + 1
                    border.width: 2
                    border.color: "black"
                    MouseArea {
                        anchors.fill: parent
                        onClicked: () => {
                            activeBackend = name;
                            appload.sendMessage(11, name);
                            activePage = "mangaReading"
                            selectionFooter.visible = true
                            pages = new Map();
                            backendImage.source = ""
                            manga_id = "";
                            totalChpt = 0;
                            description.text = "";
                            author.name = "";
                            dateDisplay.date = "";
                        }
                    }
                    Text {
                        anchors.centerIn: parent
                        anchors.leftMargin: 20
                        font.pointSize: 36
                        text: desc
                    }
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

            model: pages.get(currChpt)

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
                            source: pageUrl.includes("file:") ? `file:/tmp/mangarr/preview/${pageUrl.slice(18, -1)}g` : pageUrl
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
            id: selectionFooter
            Layout.fillWidth: true
            Layout.fillHeight: false
            Layout.preferredHeight: selectionFooter.visible ? 450 : 0
            spacing: -1
            Rectangle {
                Layout.preferredWidth: 300
                Layout.fillHeight: true
                GridLayout {
                    id: numpad
                    visible: true
                    anchors.fill: parent
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
                                onClicked: () => { manga_id += String(index + 1); confirmManga.clicked = false }
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
                        Layout.columnSpan: 2
                        border.width: 2
                        border.color: "black"
                        MouseArea {
                            anchors.fill: parent
                            onClicked: () => { manga_id += String(0); confirmManga.clicked = false }
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
                            onClicked: () => { manga_id = manga_id.slice(0, -1); confirmManga.clicked = false }
                        }
                        Image {
                            anchors.fill: parent
                            anchors.margins: 20
                            fillMode: Image.PreserveAspectFit
                            clip: true
                            source: "https://cdn-icons-png.flaticon.com/512/318/318218.png"
                        }
                    }
                }
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                ColumnLayout {
                    anchors.fill: parent
                    spacing: -1
                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: false
                        Layout.preferredHeight: 450 / 4
                        RowLayout {
                            anchors.fill: parent
                            spacing: -1
                            Rectangle {
                                id: block2
                                Layout.preferredWidth: selectionFooter.implicitWidth 
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: "black"
                                Text {
                                    anchors.centerIn: parent
                                    font.pointSize: 36
                                    text: activeBackend
                                }
                            }
                            Rectangle {
                                id: block
                                Layout.preferredWidth: selectionFooter.implicitWidth / 3
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: "black"
                                Text {
                                    anchors.centerIn: parent
                                    font.pointSize: 36
                                    text: "ID"
                                }
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: "black"
                                TextInput {
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.verticalCenter: parent.verticalCenter
                                    anchors.leftMargin: 20
                                    anchors.rightMargin: 20
                                    font.pointSize: 36
                                    text: manga_id
                                    activeFocusOnPress: true
                                    onAccepted: {
                                        Qt.inputMethod.hide();
                                    }
                                    Epaper.ScreenModeItem {
                                        anchors.fill: parent
                                        visible: true
                                        mode: Epaper.ScreenModeItem.Animation
                                    }
                                }
                            }
                            Rectangle {
                                Layout.preferredWidth: selectionFooter.implicitWidth / 2
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: confirmManga.clicked ? "green" : "blue"
                                MouseArea {
                                    id: confirmManga
                                    anchors.fill: parent
                                    property bool clicked: false
                                    onClicked: () => {
                                        if(clicked) {
                                            appload.sendMessage(10, "");
                                            selectionFooter.visible = false;
                                            clicked = false
                                        } else {
                                            appload.sendMessage(1, manga_id);
                                            clicked = true;
                                            backendImage.source = ""
                                        }
                                    }
                                }
                                Image {
                                    anchors.fill: parent
                                    anchors.margins: 5
                                    fillMode: Image.PreserveAspectCrop
                                    source: "https://media.istockphoto.com/id/1133442802/vector/green-checkmark-vector-illustration.jpg?s=612x612&w=0&k=20&c=NqyVOdwANKlbJNqbXjTvEp2wIZWUKbfUbRxm9ROPk6M="
                                }
                            }
                            Rectangle {
                                Layout.preferredWidth: selectionFooter.implicitWidth / 2
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: "red"
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: () => { manga_id = ""; backendImage.source = ""; confirmManga.clicked = false }
                                }
                                Image {
                                    anchors.fill: parent
                                    fillMode: Image.PreserveAspectFit
                                    source: "https://static.thenounproject.com/png/5507757-200.png"
                                }
                            }
                        }
                    }
                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: false
                        Layout.preferredHeight: 450 / 4 / 2
                        RowLayout {
                            anchors.fill: parent
                            spacing: -1
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: "black"
                                Text {
                                    id: author
                                    property string name: ""
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.verticalCenter: parent.verticalCenter
                                    font.pointSize: 24
                                    anchors.leftMargin: 5
                                    anchors.rightMargin: 5
                                    text: "Author: " + author.name
                                }
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: "black"
                                Text {
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.verticalCenter: parent.verticalCenter
                                    font.pointSize: 24
                                    anchors.leftMargin: 5
                                    anchors.rightMargin: 5
                                    text: totalChpt ? "Chapters: " + totalChpt : "Chapters: "
                                }
                            }
                            Rectangle {
                                Layout.preferredWidth: selectionFooter.implicitWidth
                                Layout.fillHeight: true
                                border.width: 2
                                border.color: "black"
                                Text {
                                    id: dateDisplay
                                    property string date;
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.verticalCenter: parent.verticalCenter
                                    font.pointSize: 24
                                    anchors.leftMargin: 5
                                    anchors.rightMargin: 5
                                    text: "Date: " + dateDisplay.date
                                }
                            }
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        border.width: 2
                        border.color: "black"
                        Text {
                            id: description
                            anchors.fill: parent
                            anchors.margins: 20
                            font.pointSize: 24
                            wrapMode: Text.Wrap
                        }
                    }
                }
            }
        }

        RowLayout {
            id: layout
            visible: (!selectionFooter.visible) && activePage != "backendSelection" && activePage != "bookshelfView"
            Layout.fillWidth: true
            Layout.fillHeight: false
            Layout.preferredHeight: layout.visible ? 150 : 0
            spacing: -1
            Rectangle {
                id: mangaNameContainer
                Layout.fillWidth: true
                Layout.fillHeight: true
                border.width: 2
                border.color: "black"
                RowLayout {
                    anchors.fill: parent
                    Layout.alignment: Qt.AlignVCenter | Qt.AlignHCenter
                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Text {
                            id: mangaName
                            anchors.centerIn: parent
                            font.pointSize: 24
                        }
                    }
                }
            }
            Rectangle {
                id: detailContainer
                Layout.fillWidth: true
                Layout.fillHeight: true
                border.width: 2
                border.color: "black"
                RowLayout {
                    anchors.fill: parent
                    spacing: -1
                    Layout.alignment: Qt.AlignVCenter
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        border.width: 2
                        border.color: "black"
                        RowLayout {
                            anchors.fill: parent
                            spacing: -1
                            Rectangle {
                                id: prev
                                border.width: 2
                                border.color: "black"
                                Layout.fillWidth: true
                                Layout.fillHeight: true

                                ColumnLayout {
                                    anchors.fill: parent
                                    Item {
                                        Layout.leftMargin: 5
                                        Layout.topMargin: 25
                                        Layout.alignment: Qt.AlignTop
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        Text {
                                            text: "Previous"
                                            font.pointSize: 24
                                        }
                                    }
                                    Item {
                                        Layout.leftMargin: 5
                                        Layout.bottomMargin: 25
                                        Layout.alignment: Qt.AlignBottom
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        Text {
                                            text: "chapter"
                                            font.pointSize: 24
                                        }
                                    }
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
                                Layout.fillWidth: true
                                Layout.fillHeight: true

                                ColumnLayout {
                                    anchors.fill: parent
                                    Item {
                                        Layout.leftMargin: 20
                                        Layout.topMargin: 25
                                        Layout.alignment: Qt.AlignTop
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        Text {
                                            text: "Next"
                                            font.pointSize: 24
                                        }
                                    }
                                    Item {
                                        Layout.leftMargin: 20
                                        Layout.bottomMargin: 25
                                        Layout.alignment: Qt.AlignBottom
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        Text {
                                            text: "chapter"
                                            font.pointSize: 24
                                        }
                                    }
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: () => appload.sendMessage(5, "")
                                }
                            }
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        border.width: 2
                        border.color: "black"
                        ColumnLayout {
                            anchors.fill: parent
                            spacing: -1
                            Item {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                Layout.alignment: Qt.AlignTop
                                Layout.topMargin: 25
                                Layout.leftMargin: 20
                                Layout.rightMargin: 20
                                RowLayout {
                                    anchors.fill: parent
                                    Text {
                                        font.pointSize: 24
                                        text: "Page:    "
                                    }
                    
                                    Text {
                                        font.pointSize: 24
                                        text: `${currPage}/${totalPage}`
                                    }
                                }
                            }
                            Item {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                Layout.alignment: Qt.AlignBottom
                                Layout.bottomMargin: 25
                                Layout.leftMargin: 20
                                Layout.rightMargin: 20
                                RowLayout {
                                    anchors.fill: parent
                                    Text {
                                        font.pointSize: 24
                                        text: "Chapter:"
                                    }
                    
                                    Text {
                                        font.pointSize: 24
                                        text: `${currChpt}/${totalChpt}`
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
