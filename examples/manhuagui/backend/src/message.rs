use std::{path::PathBuf, sync::Arc};

use anyhow::bail;
use appload_client::Message;
use backend::{
    manhuagui::{Manhuagui, Preferences},
    nhentai::NHentai,
};
use serde_json::json;

use crate::{Backend, BackendReplier, bookshelf::BookShelfKey, manga_reader::MangaReader};

#[derive(Debug)]
pub enum RecvMessage {
    Connect,
    SearchManga(String),
    NextPage,
    PrevPage,
    NextChapter,
    PrevChapter,
    GetChapterList,
    SelectChapter(usize),
    SelectPage(usize),
    ConfirmMangaSearch,
    SelectBackend(Arc<Backend>),
    SaveActiveToBookShelf,
    SelectBookFromBookShelf(BookShelfKey),
    BookShelfView,
    Quit,
}

impl TryFrom<Message> for RecvMessage {
    type Error = anyhow::Error;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let backend_from_str: fn(&str) -> Result<Arc<Backend>, anyhow::Error> = |x| {
            let x = match x {
                "NHentai" => {
                    Arc::new(NHentai::new("zh-tw".into(), "zh-tw".into(), true)?) as Arc<Backend>
                }
                "Manhuagui" => Arc::new(Manhuagui::new(Preferences::default())?) as Arc<Backend>,
                _ => bail!("Unsupported backend."),
            };
            anyhow::Ok(x)
        };
        let msg = match message.msg_type {
            appload_client::MSG_SYSTEM_NEW_COORDINATOR => Self::Connect,
            1 => Self::SearchManga(message.contents),
            2 => Self::NextPage,
            3 => Self::PrevPage,
            4 => Self::PrevChapter,
            5 => Self::NextChapter,
            6 => Self::GetChapterList,
            7 => Self::SelectChapter(message.contents.parse()?),
            9 => Self::SelectPage(message.contents.parse()?),
            10 => Self::ConfirmMangaSearch,
            11 => Self::SelectBackend(backend_from_str(&message.contents)?),
            12 => Self::SaveActiveToBookShelf,
            13 => {
                let (backend, manga_url) = message.contents.split_once("\n").unwrap();
                let backend = backend_from_str(backend)?;
                let key = BookShelfKey::new(&*backend, manga_url.to_string());
                Self::SelectBookFromBookShelf(key)
            }
            14 => Self::BookShelfView,
            99 => Self::Quit,
            _ => bail!("Unknown message received."),
        };
        Ok(msg)
    }
}

pub enum SendMessage {
    ActivePageNumber(usize),
    TotalPageSize(usize),
    ActiveChapterNumber(usize),
    TotalChapterSize(usize),
    ChapterList(String),
    /// setups the models for all pages in a chapter
    InitPagesForGivenChapter(usize),
    /// modfies a given page's path for a given chapter
    PageModify {
        chapter: usize,
        page: usize,
        path: PathBuf,
    },
    Status(String),
    MangaDescription(String),
    MangaAuthor(String),
    MangaPreview(PathBuf),
    MangaName(String),
    MangaLastUpdatedTime(String),
    BookshelfMangaDetails(Box<MangaReader>),
    Error(String),
    /// the display for the image on each active page
    BackendImage,
}

impl SendMessage {
    pub fn display(self) -> (u32, Option<String>) {
        match self {
            Self::ActivePageNumber(x) => (4, Some(x.to_string())),
            Self::TotalPageSize(x) => (5, Some(x.to_string())),
            Self::ActiveChapterNumber(x) => (6, Some(x.to_string())),
            Self::TotalChapterSize(x) => (7, Some(x.to_string())),
            Self::ChapterList(s) => (8, Some(s)),
            Self::InitPagesForGivenChapter(x) => (9, Some(x.to_string())),
            Self::PageModify {
                chapter: display_chapter,
                page,
                path,
            } => {
                let msg = format!("{}\n{}\nfile:{}", display_chapter, page, path.display());
                (10, Some(msg))
            }
            Self::Status(s) => (11, Some(s)),
            Self::MangaDescription(s) => (12, Some(s)),
            Self::MangaAuthor(s) => (13, Some(s)),
            Self::MangaPreview(path) => {
                let msg = format!("file:{}", path.display());
                (14, Some(msg))
            }
            Self::MangaName(s) => (15, Some(s)),
            Self::MangaLastUpdatedTime(s) => (16, Some(s)),
            Self::BookshelfMangaDetails(manga) => {
                let total_pages = manga.pages().len();
                let details = manga.details;
                let v = json![{
                    "url"            : details.url,
                    "title"          : details.title,
                    "backend"        : manga.api.to_string(),
                    "lastReadPage"   : (manga.current_page.page + 1).to_string(),
                    "totalPages"     : total_pages.to_string(),
                    "lastReadChapter": (manga.current_page.chapter() + 1).to_string(),
                    "totalChapters"  : manga.chapters.len().to_string(),
                    "description"    : details.description.unwrap_or_default(),
                }];

                (17, Some(v.to_string()))
            }
            Self::Error(s) => (1000, Some(s)),
            Self::BackendImage => (101, None),
        }
    }
    pub fn status(s: impl Into<String>) -> Self {
        Self::Status(s.into())
    }
}

pub trait ReplierExt {
    fn send_typed_message(&self, msg: SendMessage) -> anyhow::Result<()>;
}

impl ReplierExt for BackendReplier {
    fn send_typed_message(&self, msg: SendMessage) -> anyhow::Result<()> {
        let (msg, contents) = msg.display();
        let mut contents = contents.as_ref().map_or("placeholder", |v| v);
        if contents.is_empty() {
            println!("empty content! adding placeholder text for protection");
            contents = "placeholder";
        }
        self.send_message(msg, contents)
    }
}
