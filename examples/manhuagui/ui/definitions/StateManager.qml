pragma Singleton

import QtQuick 2.15

QtObject {
    enum ActivePage {
        BookshelfView,
        MangaReading,
        ChapterList,
        PageList,
        BackendSelection,
        Search
    }

    property string manga_id: ""
    property var chapters: []
    property int activePage: StateManager.ActivePage.BookshelfView
    property string activeBackend: "Manhuagui"
    property var pages: ({})
    property var bookshelf: ({})
    property int currPage: 0
    property int totalPage: 0
    property int currChpt: 0
    property int totalChpt: 0
    property string currentImageUrl: ""
    property string mangaName: ""
    property string mangaAuthor: ""
    property string mangaDescription: ""
    property string mangaDate: ""
    property string errorMessage: ""

    Component.onCompleted: {
        pages = new Map();
        bookshelf = new Map();
    }

    signal bookshelfUpdated()
    signal backendInitialized()
    signal pageViewUpdated()
    signal pagesUpdated(int chapter, int page)

    function updateOrCreatePage(chapter, page, data) {
        let map = pages.get(chapter);

        if (!map) {
            map = new Map();
            pages.set(chapter, map);
        }

        map.set(page, data);
        pagesUpdated(chapter, page);
    }

    function addBook(key, bookData) {
        bookshelf.set(key, bookData);

        bookshelfUpdated();
    }
}
