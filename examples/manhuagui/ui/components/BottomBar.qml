import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3
import "../definitions"

RowLayout {
    id: layout
    Layout.fillWidth: true
    Layout.fillHeight: false
    Layout.preferredHeight: 150
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
                    text: StateManager.mangaName
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
                            onClicked: () => BackendController.sendMessage(4, "")
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
                            onClicked: () => BackendController.sendMessage(5, "")
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
                                text: `${StateManager.currPage}/${StateManager.totalPage}`
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
                                text: `${StateManager.currChpt}/${StateManager.totalChpt}`
                            }
                        }
                    }
                }
            }
        }
    }
}

