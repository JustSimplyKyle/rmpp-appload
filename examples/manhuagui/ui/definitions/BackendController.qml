pragma Singleton

import QtQuick 2.5
import net.asivery.AppLoad 1.0
import "../definitions"

QtObject {
    property AppLoad appload: AppLoad {
        id: appload
        applicationID: "manhuagui"
        onMessageReceived: (type, contents) => {
            switch(type){
                case 101:
                    const chapter = StateManager.pages.get(StateManager.currChpt);
                    if(!chapter) {
                        console.log("ain't havin chapters");
                        StateManager.currentImageUrl = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    const pagef = chapter.get(parseInt(StateManager.currPage) - 1);
                    if(!pagef) {
                        console.log("ain't havin pages");
                        StateManager.currentImageUrl = "https://w7.pngwing.com/pngs/773/130/png-transparent-data-download-downloader-downloading-transfer-free-system-and-user-interface-icon-thumbnail.png"
                        break;
                    }
                    
                    StateManager.currentImageUrl = pagef; 
                    break;
                case 4:
                    StateManager.currPage = parseInt(`${contents}`)
                    break;
                case 5:
                    StateManager.totalPage = parseInt(`${contents}`)
                    break;
                case 6:
                    StateManager.currChpt = parseInt(`${contents}`)
                    break;
                case 7:
                    StateManager.totalChpt = parseInt(`${contents}`)
                    break;
                case 8:
                    StateManager.chapters = contents.split('\n')
                    break;
                    // let chapterList.model: chapters.length
                case 9:
                    // probably unused
                    const targetChpt = parseInt(contents);
                    break;
                case 10:
                    const arr = contents.split('\n');
                    const targetChapter = parseInt(arr[0]);
                    const targetPage = parseInt(arr[1]);
                    const path = arr[2];
                    if(targetChapter === StateManager.currChpt && targetPage + 1 === StateManager.currPage) {
                        StateManager.currentImageUrl = path;
                    }
                    StateManager.updateOrCreatePage(targetChapter, targetPage, path);
                    break;
                case 12:
                    StateManager.mangaDescription = `${contents}`;
                    break;
                case 13:
                    StateManager.mangaAuthor = `${contents}`;
                    break;
                case 14:
                    StateManager.currentImageUrl = `${contents}`; 
                    break;
                case 15:
                    StateManager.mangaName = `${contents}`;
                    break;
                case 16:
                    StateManager.mangaDate = `${contents}`;
                    break;
                case 17:
                    const array = JSON.parse(contents);
                    const key = array.url;
                    StateManager.addBook(key, array);
            }
        }
    }
    function sendMessage(type, contents) {
        appload.sendMessage(type, contents)
    }
}
