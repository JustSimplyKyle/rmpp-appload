import QtQuick 2.5
import net.asivery.AppLoad 1.0
import "." 

QtObject {
    pragma Singleton

    function sendMessage(type, contents) {
        appload.sendMessage(type, contents)
    }

    // The rest of the file is the same...
    AppLoad {
        id: appload
        applicationID: "manhuagui"
        onMessageReceived: (type, contents) => {
            switch(type){
                case 101:
                    const chapter = StateManager.pages.get(StateManager.currChpt);
                    if(!chapter) {
                        StateManager.currentImageUrl = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    const pagef = chapter.get(parseInt(StateManager.currPage) - 1);
                    if(!pagef) {
                        StateManager.currentImageUrl = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    const url = pagef.pageUrl;
                    if(!url) {
                        StateManager.currentImageUrl = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    
                    StateManager.currentImageUrl = url; 
                    break;
                case 4:
                    StateManager.currPage = parseInt(`${contents}`)
                case 5:
                    StateManager.totalPage = parseInt(`${contents}`)
                case 6:
                    // The set() method of Map instances adds or updates an entry in this map with a specified key and a value.
                    for(int i = 0; i<=StateManager.totalPage; i++) {
                        const map = new Map();
                        pages.set(StateManager.currChpt, { index: i, pageUrl: "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png" });
                    }
                    StateManager.currChpt = parseInt(`${contents}`)
                case 7:
                    StateManager.totalChpt = parseInt(`${contents}`)
                case 8:
                    StateManager.chapters = contents.split('\n')
                    // let chapterList.model: chapters.length
                case 9:
                    // probably unused
                    const targetChpt = parseInt(contents);
                case 10:
                    const arr = contents.split('\n');
                    const targetChapter = parseInt(arr[0]);
                    const targetPage = parseInt(arr[1]);
                    const path = arr[2];
                    if(targetChapter === StateManager.currChpt && targetPage + 1 === StateManager.currPage) {
                        StateManager.currentImageUrl = url;
                    }
            }
        }
    }
}
