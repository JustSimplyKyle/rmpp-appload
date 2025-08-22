import QtQuick 2.5

QtObject {
    pragma Singleton

    property string manga_id: ""
    property var chapters: []
    property string activePage: "mangaReading" 
    property string activeBackend: "Manhuagui"
    property var pages: ({})
    property var bookshelf: ({})
    property int currPage: 0
    property int totalPage: 0
    property int currChpt: 0
    property int totalChpt: 0
    property string currentImageUrl: ""
    property string mangaTitle: ""
    property string mangaAuthor: ""
    property string mangaDescription: ""
    property string mangaDate: ""
    property string errorMessage: ""

    Component.onCompleted: {
        pages = new Map();
        bookshelf = new Map();
    }
}
