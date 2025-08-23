import QtQuick 2.5
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.3
import "../definitions"

GridView {
    id: backendSelection
    anchors.fill: parent
    cellWidth: parent.width
    cellHeight: 100
    model: backendModel

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

    header: Rectangle {
        width: backendSelection.width
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
            width: backendSelection.width - 300
            height: backendSelection.cellHeight + 1
            border.width: 2
            border.color: "black"
            MouseArea {
                anchors.fill: parent
                onClicked: () => {
                    StateManager.activeBackend = name;
                    BackendController.sendMessage(11, name);
                    StateManager.activePage = StateManager.ActivePage.MangaReading
                    StateManager.pages = new Map();
                    StateManager.manga_id = "";
                    StateManager.totalChpt = 0;
                    StateManager.mangaDescription = "";
                    StateManager.mangaName = "";
                    StateManager.mangaDate = "";
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

