import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3
import net.asivery.AppLoad 1.0

Rectangle {
    anchors.fill: parent
    property string manga_id: ""
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
                    chptSeparater.text = "/"
                    chptStarter.text = "chapter:"
                    break;
                case 7:
                    totalChpt.text = `${contents}`
                    chptSeparater.text = "/"
                    chptStarter.text = "chapter:"
                    break;
            }
        }
    }
    ColumnLayout {
        id: view
        anchors.fill: parent

        Image {
            id: backendImage
            Layout.fillWidth: true
            Layout.fillHeight: true
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
        ColumnLayout {
            id: chapterList
            visible: false
            Layout.fillWidth: true
            Layout.fillHeight: true
            Rectangle {
                Layout.fillWidth: true
                height: 60
                border.width: 2
                border.color: "black"
                Text {
                    font.pointSize: 24
                    text: "baby2"
                }
            }
            Rectangle {
                Layout.fillWidth: true
                height: 60
                border.width: 2
                border.color: "black"
                Text {
                    font.pointSize: 24
                    text: "baby3"
                }
            }
            Rectangle {
                Layout.fillWidth: true
                height: 60
                border.width: 2
                border.color: "black"
                Text {
                    font.pointSize: 24
                    text: "baby4"
                }
            }
        }
        Rectangle {
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
                width: content.implicitWidth + 30
                height: content.implicitHeight + 10
                border.width: 2
                border.color: "black"
                ColumnLayout {
                    id: content
                    spacing: 10
                    anchors.centerIn: parent
                    anchors.right: parent.right
                    Button {
                        font.pointSize: 24
                        onClicked: () => {
                            backendImage.visible = false;
                            chapterList.visible = true;
                        }
                        text: "Change chapter"
                    }
                    Button {
                        font.pointSize: 24
                        text: "Change page"
                    }
                    Button {
                        font.pointSize: 24
                        text: "Go back to settings"
                    }
                    Button {
                        font.pointSize: 24
                        text: "Go back to bookshelf"
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
                    onClicked: () => appload.sendMessage(99, "")
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
