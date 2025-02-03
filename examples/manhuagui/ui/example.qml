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
                    backendImage.source = `${contents}`
                    // backendImage
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
                    let pageModel = pages.get(currChpt.text);
                    if(pageModel !== undefined) {
                        pageModel.clear()
                    }
                    contents.split('\n').forEach(pageUrl => {
                        pageModel.append({ "pageUrl": pageUrl })
                    })
                    pageList.model = pageModel
                    pageList.forceLayout();
                    break;
            }
        }
    }

    Popup {
        id: popup
        x: 0
        y: view.height - popup.height - 80
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
                    Layout.preferredWidth: parent.width * 0.75
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
    

    ColumnLayout {
        id: view
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
                    onClicked: () => appload.sendMessage(3, "")
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
        Rectangle {
            visible: pageList.visible
            Layout.fillWidth: pageList.visible ? true : false
        }
        GridView {
            id: pageList
            visible: activePage === "pageList"
            flickableDirection: Flickable.NoFlick
            Layout.fillWidth: pageList.visible ? true : false
            Layout.fillHeight: pageList.visible ? true: false
            Layout.preferredHeight: pageList.visible ? null : 0
            cellWidth: view.width / 4.
            cellHeight: 580
            clip: true

            snapMode: GridView.SnapOneRow
            flickDeceleration: 100000
            highlightMoveDuration: 1000000

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
                    onClicked: () => popup.open()
                    // onClicked: () => appload.sendMessage(99, "")
                }
            }

            Rectangle {
                id: button
                border.width: 2
                border.color: "black"
                width: 600
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
                        columns: 3
                        rows: 4 

                        // Number buttons (1-9)
                        Repeater {
                            model: 9
                            Button {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                onClicked: () => manga_id += String(index + 1)
                                Text {
                                    anchors.centerIn: parent
                                    font.pointSize: 24
                                    text: String(index + 1)
                                }
                            }
                        }
                        Button {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            onClicked: () => manga_id += String(0)
                            Text {
                                anchors.centerIn: parent
                                font.pointSize: 24
                                text: String(0)
                            }
                        }
                        Button {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            onClicked: () => { appload.sendMessage(1, "id:" + manga_id); manga_id = ""; numpad.visible = false }
                            Text {
                                anchors.centerIn: parent
                                font.pointSize: 24
                                text: "Enter!"
                            }
                        }
                        Button {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            onClicked: () => manga_id = ""
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
