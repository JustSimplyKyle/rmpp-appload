import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3
import "../definitions"
GridView {
    id: bookshelfView
    anchors.fill: parent
    cellWidth: bookshelfView.width
    cellHeight: bookshelfView.height / 7

    ListModel {
        id: bookshelfModel
    }

    model: bookshelfModel

    Connections {
        target: StateManager

        function onBookshelfUpdated() {
            console.log("Bookshelf state changed, updating the view's model...");
            bookshelfModel.clear();
            const values = Array.from(StateManager.bookshelf.values());
            for (var i = 0; i < values.length; i++) {
                bookshelfModel.append(values[i]);
            }
        }
    }

    header: Rectangle {

        width: bookshelfView.width
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
                    BackendController.sendMessage(13, `${backend}\n${url}`);
                    StateManager.activePage = StateManager.ActivePage.MangaReading
                    StateManager.currentImageUrl = ""
                    // StateManager.pages = new Map();
                    StateManager.manga_id = "";
                    // totalChpt = 0;
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
                    BackendController.sendMessage(13, `${backend}\n${url}`);
                    StateManager.activePage = StateManager.ActivePage.MangaReading
                    StateManager.currentImageUrl = ""
                    // StateManager.pages = new Map();
                    StateManager.manga_id = "";
                    // totalChpt = 0;
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
