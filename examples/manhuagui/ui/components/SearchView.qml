import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3

import "../definitions"

import xofm.libs.epaper as Epaper

ColumnLayout {
    anchors.fill: parent

    Image {
        id: backendImage
        source: StateManager.currentImageUrl
        Layout.fillWidth: true
        Layout.fillHeight: true
        fillMode: Image.PreserveAspectFit
        RowLayout {
            anchors.fill: parent
            MouseArea {
                Layout.fillWidth: true
                Layout.fillHeight: true
                onClicked: () =>  BackendController.sendMessage(3, "") 
            }
            MouseArea {
                Layout.fillWidth: true
                Layout.fillHeight: true
                onClicked: () => BackendController.sendMessage(2, "")
            }
        }
    }

    RowLayout {
        id: searchFooter
        Layout.fillWidth: true
        Layout.fillHeight: false
        Layout.preferredHeight: 450
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
                            onClicked: () => { StateManager.manga_id += String(index + 1); confirmManga.clicked = false }
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
                        onClicked: () => { StateManager.manga_id += String(0); confirmManga.clicked = false }
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
                        onClicked: () => { StateManager.manga_id = manga_id.slice(0, -1); confirmManga.clicked = false }
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
                            Layout.preferredWidth: searchFooter.implicitWidth 
                            Layout.fillHeight: true
                            border.width: 2
                            border.color: "black"
                            Text {
                                anchors.centerIn: parent
                                font.pointSize: 36
                                text: StateManager.activeBackend
                            }
                        }
                        Rectangle {
                            id: block
                            Layout.preferredWidth: searchFooter.implicitWidth / 3
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
                                text: StateManager.manga_id
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
                            Layout.preferredWidth: searchFooter.implicitWidth / 2
                            Layout.fillHeight: true
                            border.width: 2
                            border.color: confirmManga.clicked ? "green" : "blue"
                            MouseArea {
                                id: confirmManga
                                anchors.fill: parent
                                property bool clicked: false
                                onClicked: () => {
                                    if(clicked) {
                                        StateManager.activePage = StateManager.ActivePage.MangaReading;
                                        BackendController.sendMessage(10, "");
                                        clicked = false
                                    } else {
                                        BackendController.sendMessage(1, StateManager.manga_id);
                                        clicked = true;
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
                            Layout.preferredWidth: searchFooter.implicitWidth / 2
                            Layout.fillHeight: true
                            border.width: 2
                            border.color: "red"
                            MouseArea {
                                anchors.fill: parent
                                onClicked: () => { StateManager.manga_id = ""; backendImage.source = ""; confirmManga.clicked = false }
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
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                font.pointSize: 24
                                anchors.leftMargin: 5
                                anchors.rightMargin: 5
                                text: "Author: " + StateManager.MangaAuthor
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
                                text: StateManager.totalChpt ? "Chapters: " + StateManager.totalChpt : "Chapters: "
                            }
                        }
                        Rectangle {
                            Layout.preferredWidth: searchFooter.implicitWidth
                            Layout.fillHeight: true
                            border.width: 2
                            border.color: "black"
                            Text {
                                id: dateDisplay
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                font.pointSize: 24
                                anchors.leftMargin: 5
                                anchors.rightMargin: 5
                                text: "Date: " + StateManager.mangaDate
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
                        text: StateManager.mangaDescription
                    }
                }
            }
        }
    }
}
