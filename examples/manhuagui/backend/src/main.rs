mod bookshelf;

use std::{
    collections::HashMap,
    fs::{create_dir, remove_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
};

use anyhow::{bail, Context};
use appload_client::{AppLoadBackend, BackendReplier, Message, MSG_SYSTEM_NEW_COORDINATOR};
use async_trait::async_trait;
use backend::{
    manhuagui::{Manhuagui, Preferences},
    nhentai::NHentai,
    MangaBackend, SChapter, SManga,
};
use bookshelf::{BookShelf, BookShelfKey};
use futures_util::StreamExt;
use image::{
    codecs::png::PngEncoder, imageops::FilterType, DynamicImage, GenericImageView, ImageReader,
    Rgba,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{io::AsyncWriteExt, sync::Mutex, task::JoinHandle};

#[tokio::main]
async fn main() {
    appload_client::AppLoad::new(&mut MyBackend::default())
        .expect("backend failing to start. please cry")
        .run()
        .await
        .expect("backend failing to start. please cry");
}

type Backend = dyn MangaBackend;
#[derive(Default)]
struct MyBackend {
    //                          (chapter, page)
    handlers: Arc<Mutex<HashMap<(usize, usize), Option<JoinHandle<anyhow::Result<()>>>>>>,
    bookshelf: BookShelf,
    active: MangaReader,
    state: State,
}

#[derive(Debug)]
enum RecvMessage {
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
            MSG_SYSTEM_NEW_COORDINATOR => Self::Connect,
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

enum SendMessage {
    ActivePageNumber(usize),
    TotalPageSize(usize),
    ActiveChapterNumber(usize),
    TotalChapterSize(usize),
    ChapterList(String),
    PageListChapter(usize),
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
    BackendImage,
}

impl SendMessage {
    fn display(self) -> (u32, Option<String>) {
        match self {
            Self::ActivePageNumber(x) => (4, Some(x.to_string())),
            Self::TotalPageSize(x) => (5, Some(x.to_string())),
            Self::ActiveChapterNumber(x) => (6, Some(x.to_string())),
            Self::TotalChapterSize(x) => (7, Some(x.to_string())),
            Self::ChapterList(s) => (8, Some(s)),
            Self::PageListChapter(x) => (9, Some(x.to_string())),
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
                let details = manga.details;
                let v = json![{
                    "url"            : details.url,
                    "title"          : details.title,
                    "backend"        : manga.api.to_string(),
                    "lastReadPage"   : (manga.page + 1).to_string(),
                    "totalPages"     : manga.pages.len().to_string(),
                    "lastReadChapter": (manga.chapter + 1).to_string(),
                    "totalChapters"  : manga.chapters.len().to_string(),
                    "description"    : details.description.unwrap_or_default(),
                }];

                (17, Some(v.to_string()))
            }
            Self::Error(s) => (1000, Some(s)),
            Self::BackendImage => (101, None),
        }
    }
    fn status(s: impl Into<String>) -> Self {
        Self::Status(s.into())
    }
}

trait ReplierExt {
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

impl MyBackend {
    #[allow(clippy::too_many_lines)]
    async fn handle_message(
        &mut self,
        functionality: &BackendReplier,
        message: Message,
    ) -> anyhow::Result<()> {
        match RecvMessage::try_from(message)? {
            RecvMessage::Connect => {
                functionality.send_typed_message(SendMessage::status("connected frontend"))?;

                println!("A frontend has connected");
            }
            RecvMessage::SearchManga(search_term) => {
                functionality.send_typed_message(SendMessage::status("starts downloading"))?;

                let api = self.active.api.clone();

                let body = api.search_by_id(&search_term).await?;

                functionality.send_typed_message(SendMessage::status("body finished!"))?;

                let chapters = api
                    .fetch_chapters(&body)
                    .await?
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>();

                let first_chapter = chapters.first().context("empty")?;

                functionality.send_typed_message(SendMessage::status("chapter finished!"))?;

                let result = api
                    .fetch_pages(first_chapter)
                    .await?
                    .into_iter()
                    .map(|x| x.image_url)
                    .collect::<Vec<_>>();

                functionality.send_typed_message(SendMessage::status("page list downloaded"))?;

                let search = MangaReader {
                    api,
                    pages: result,
                    page: 0,
                    chapters,
                    chapter: 0,
                    details: body,
                };
                self.state = State::Search {
                    search: Box::new(search),
                    confirm: false,
                };
            }
            RecvMessage::NextPage => {
                let manga = &mut self.active;
                manga.page += 1;
                if manga.pages.len() == manga.page {
                    manga.next_chapter().await?;
                }
            }
            RecvMessage::PrevPage => {
                let manga = &mut self.active;
                if manga.page == 0 {
                    manga.prev_chapter().await?;
                } else {
                    manga.page -= 1;
                }
                self.state = State::Reading { start: manga.page };
            }
            RecvMessage::PrevChapter => {
                self.active.prev_chapter().await?;
            }
            RecvMessage::NextChapter => {
                self.active.next_chapter().await?;
            }
            RecvMessage::GetChapterList => {
                let output = self
                    .active
                    .chapters
                    .iter()
                    .map(|x| &*x.name)
                    .collect::<Vec<&str>>()
                    .join("\n");

                self.state = State::ChapterList { output };
            }
            RecvMessage::SelectChapter(index) => {
                self.active.chapter = index;
                self.active.update_chapter().await?;
                self.state = State::Reading { start: 0 };
            }
            RecvMessage::SelectPage(index) => {
                self.active.page = index;
                self.state = State::Reading { start: index };
            }
            RecvMessage::ConfirmMangaSearch => {
                let State::Search { search: manga, .. } =
                    std::mem::replace(&mut self.state, State::Reading { start: 0 })
                else {
                    bail!("impossible state reached");
                };
                self.active = *manga;
            }
            RecvMessage::SelectBackend(supported_backend) => {
                let is_different = { dbg!(supported_backend.id()) != dbg!(self.active.api.id()) };
                if is_different {
                    let manga_reader = MangaReader {
                        api: supported_backend.clone(),
                        ..Default::default()
                    };
                    self.active = manga_reader;
                    if let State::Search { search: manga, .. } = &mut self.state {
                        *manga = Box::new(self.active.clone());
                    }
                }
            }
            RecvMessage::Quit => {
                if PathBuf::from("/tmp/mangarr").exists() {
                    remove_dir_all("/tmp/mangarr")?;
                }
                exit(0);
            }
            RecvMessage::SaveActiveToBookShelf => {
                self.bookshelf.insert(self.active.clone()).await?;
            }
            RecvMessage::SelectBookFromBookShelf(key) => {
                let manga = self
                    .bookshelf
                    .bookshelf()
                    .get(&key)
                    .context("somehow missing bookshelf stuff")?
                    .clone();
                let p = manga.page;
                self.active = manga;
                self.state = State::Reading { start: p };
                // self.active.update_chapter().await?;
                // self.active.page = p;
                self.active.send_details(functionality)?;
            }
            RecvMessage::BookShelfView => {
                self.state = State::Bookshelf;
            }
        };
        self.react_to_state(functionality).await?;
        Ok(())
    }

    async fn initiate_chapter_download(
        &self,
        functionality: BackendReplier,
        start: usize,
    ) -> anyhow::Result<()> {
        let manga = Arc::new(self.active.clone());

        let chapter = manga.chapter;
        let client = manga.api.client();

        functionality.send_typed_message(SendMessage::PageListChapter(chapter + 1))?;

        let handle = tokio::spawn(async move {
            let len = manga.pages.len();

            let mut iter = tokio_stream::iter(start..len)
                .map(|page| {
                    let client = client.clone();
                    let manga = manga.clone();
                    tokio::spawn(async move {
                        manga.save_to_disk(client, page).await?;
                        anyhow::Ok(page)
                    })
                })
                .buffered(3);

            while let Some(x) = iter.next().await {
                let page = x??;

                let path = manga.get_url_with_path(page)?.1;
                functionality.send_typed_message(SendMessage::PageModify {
                    chapter: manga.chapter + 1,
                    page,
                    path,
                })?;
            }
            anyhow::Ok(())
        });
        self.handlers
            .lock()
            .await
            .insert((chapter, start), Some(handle));
        Ok(())
    }

    async fn react_to_state(
        &mut self,
        functionality: &BackendReplier,
    ) -> Result<(), anyhow::Error> {
        match self.state {
            State::Idleing => {}
            State::Reading { start } => {
                self.active.send_page_information(functionality)?;

                let chapter = self.active.chapter;

                self.bookshelf.insert(self.active.clone()).await?;

                for (_, v) in self
                    .handlers
                    .lock()
                    .await
                    .iter_mut()
                    .filter(|(&(c, p), _)| c != chapter || p != start)
                {
                    v.as_ref().unwrap().abort();
                    *v = None;
                }

                self.handlers.lock().await.retain(|_, y| y.is_some());

                functionality.send_typed_message(SendMessage::BackendImage)?;

                self.initiate_chapter_download(functionality.clone(), start)
                    .await?;
            }
            State::ChapterList {
                output: ref chapters,
                ..
            } => {
                functionality.send_typed_message(SendMessage::ChapterList(chapters.to_string()))?;
            }
            State::Search {
                ref search,
                confirm,
            } => {
                if confirm {
                    let State::Search { search: manga, .. } =
                        std::mem::replace(&mut self.state, State::Reading { start: 0 })
                    else {
                        bail!("impossible state reached");
                    };
                    self.active = *manga;
                    self.state = State::Reading { start: 0 };
                    Box::pin(self.react_to_state(functionality)).await?;
                    return Ok(());
                }
                search.send_details(functionality)?;
                search.send_page_information(functionality)?;
                search
                    .save_to_disk(self.active.api.client().clone(), 0)
                    .await?;
                functionality.send_typed_message(SendMessage::MangaPreview(
                    search.get_url_with_path(0)?.1,
                ))?;
            }
            State::Bookshelf => {
                for v in self.bookshelf.bookshelf().values() {
                    functionality.send_typed_message(SendMessage::BookshelfMangaDetails(
                        Box::new(v.clone()),
                    ))?;
                }
            }
        };
        Ok(())
    }
}

#[derive(Debug)]
enum State {
    Idleing,
    Bookshelf,
    ChapterList {
        output: String,
    },
    Reading {
        start: usize,
    },
    Search {
        search: Box<MangaReader>,
        confirm: bool,
    },
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct MangaReader {
    api: Arc<Backend>,
    details: SManga,
    chapters: Vec<SChapter>,
    chapter: usize,
    pages: Vec<String>,
    page: usize,
}

impl Default for MangaReader {
    fn default() -> Self {
        Self {
            api: Arc::new(Manhuagui::new(Preferences::default()).unwrap()),
            details: Default::default(),
            chapters: Default::default(),
            chapter: Default::default(),
            pages: Default::default(),
            page: Default::default(),
        }
    }
}

impl MangaReader {
    pub fn send_details(&self, functionality: &BackendReplier) -> anyhow::Result<()> {
        if let Some(description) = self.details.description.clone() {
            functionality.send_typed_message(SendMessage::MangaDescription(description))?;
        }
        if let Some(author) = self.details.author.clone() {
            functionality.send_typed_message(SendMessage::MangaAuthor(author))?;
        }
        functionality.send_typed_message(SendMessage::MangaLastUpdatedTime(
            self.details.last_updated_time.clone(),
        ))?;
        functionality.send_typed_message(SendMessage::MangaName(self.details.title.clone()))?;
        Ok(())
    }
    pub fn send_page_information(&self, functionality: &BackendReplier) -> anyhow::Result<()> {
        functionality.send_typed_message(SendMessage::ActivePageNumber(self.page + 1))?;

        functionality.send_typed_message(SendMessage::TotalPageSize(self.pages.len()))?;
        functionality.send_typed_message(SendMessage::ActiveChapterNumber(self.chapter + 1))?;
        functionality.send_typed_message(SendMessage::TotalChapterSize(self.chapters.len()))?;
        Ok(())
    }
    pub async fn next_chapter(&mut self) -> anyhow::Result<()> {
        self.chapter += 1;
        self.update_chapter().await
    }
    pub async fn prev_chapter(&mut self) -> anyhow::Result<()> {
        self.chapter -= 1;
        self.update_chapter().await
    }
    async fn update_chapter(&mut self) -> anyhow::Result<()> {
        let api = &*self.api;
        let chpt = &self.chapters[self.chapter];
        let pages = api
            .fetch_pages(chpt)
            .await?
            .into_iter()
            .map(|x| x.image_url)
            .collect::<Vec<_>>();
        self.pages = pages;
        self.page = 0;
        Ok(())
    }
    pub async fn save_to_disk(&self, client: Client, page: usize) -> anyhow::Result<()> {
        if !PathBuf::from("/tmp/mangarr").exists() {
            create_dir("/tmp/mangarr/")?;
        }
        if !PathBuf::from("/tmp/mangarr/preview").exists() {
            create_dir("/tmp/mangarr/preview")?;
        }

        let (url, ps) = self.get_url_with_path(page)?;

        if !ps.exists() {
            let image = Self::save_page(url, client, &ps).await?;
            if let Some(image) = image {
                self.generate_scaled_version(image, page).await?;
            }
        }
        Ok(())
    }
    pub async fn generate_scaled_version(
        &self,
        mut original_image: DynamicImage,
        page: usize,
    ) -> anyhow::Result<()> {
        original_image = original_image.resize(405, 660, FilterType::Lanczos3);
        let (_, mut ps) = self.get_url_with_path(page)?;
        ps = PathBuf::from("/tmp/mangarr/preview").join(ps.file_name().context("no")?);
        if !ps.exists() {
            let mut buf = vec![];

            let encoder = PngEncoder::new(&mut buf);

            original_image.write_with_encoder(encoder)?;

            tokio::fs::write(ps, buf).await?;
        }
        Ok(())
    }
    fn get_url_with_path(&self, page: usize) -> anyhow::Result<(&str, PathBuf)> {
        let Some(url) = self.pages.get(page) else {
            bail!("out of bounds");
        };

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hashed_value = hasher.finish();

        let ps = PathBuf::from(format!("/tmp/mangarr/pic{hashed_value}.png"));
        Ok((url, ps))
    }
    async fn save_page(
        url: &str,
        client: Client,
        path: impl AsRef<Path> + Send,
    ) -> anyhow::Result<Option<DynamicImage>> {
        let path = path.as_ref();
        if !path.exists() {
            let bytes = client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            let mut non_grayscale = vec![];

            let non_grayscale_encoder = PngEncoder::new(&mut non_grayscale);

            let mut image = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()?
                .decode()?;

            let is_almost_grayscale = image.pixels().all(|(_, _, Rgba([r, g, b, _]))| {
                r.abs_diff(g) < 3 && g.abs_diff(b) < 3 && r.abs_diff(b) < 3
            });

            if is_almost_grayscale {
                image = image.grayscale();
            }

            image.write_with_encoder(non_grayscale_encoder)?;

            if !path.exists() {
                let mut file = tokio::fs::File::create_new(path).await?;

                file.write_all(&non_grayscale).await?;

                file.flush().await?;
                return Ok(Some(image));
            }
        }
        Ok(None)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Idleing
    }
}

#[async_trait]
impl AppLoadBackend for MyBackend {
    async fn handle_message(&mut self, functionality: &BackendReplier, message: Message) {
        let v = self.handle_message(functionality, message);

        if let Err(err) = v.await {
            let err = err
                .chain()
                .enumerate()
                .map(|(i, x)| format!("{}:{x:#?}", i + 1))
                .collect::<Vec<_>>()
                .join("\n");
            // panic!("{err:#?}");
            functionality
                .send_typed_message(SendMessage::Error(format!("error: {err:#?}")))
                .expect("can't send message");
        }
    }
}
