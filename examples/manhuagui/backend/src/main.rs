use std::{
    collections::HashMap,
    fs::{create_dir, remove_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    path::{Path, PathBuf},
    process::exit,
    sync::{Arc, LazyLock},
};

use anyhow::{bail, Context};
use appload_client::{start, AppLoadBackend, BackendReplier, Message, MSG_SYSTEM_NEW_COORDINATOR};
use async_trait::async_trait;
use backend::{Manhuagui, Preferences, SChapter};
use futures_util::StreamExt;
use image::{codecs::png::PngEncoder, ImageReader};
use tokio::{io::AsyncWriteExt, sync::Mutex, task::JoinHandle};

#[tokio::main]
async fn main() {
    start(&mut MyBackend::default())
        .await
        .expect("backend failing to start. please cry");
}

struct MyBackend {
    api: LazyLock<Manhuagui>,
    handlers: Arc<Mutex<HashMap<usize, Option<JoinHandle<anyhow::Result<()>>>>>>,
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
    Quit,
}

impl TryFrom<Message> for RecvMessage {
    type Error = anyhow::Error;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
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
        self.send_message(msg, contents.as_ref().map_or("placeholder", |v| v))
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
                let api = &*self.api;
                functionality.send_typed_message(SendMessage::status("starts downloading"))?;

                let body = api
                    .fetch_search_manga(0, &search_term, vec![])
                    .await?
                    .mangas
                    .into_iter()
                    .next()
                    .context("empty")?;

                functionality.send_typed_message(SendMessage::status("body finished!"))?;

                let chapters = api
                    .chapter_list_parse(&body)
                    .await?
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>();

                let first_chapter = chapters.first().context("empty")?;

                functionality.send_typed_message(SendMessage::status("chapter finished!"))?;

                let result = api
                    .page_list_parse(first_chapter)
                    .await?
                    .into_iter()
                    .map(|x| x.image_url)
                    .collect::<Vec<_>>();

                functionality.send_typed_message(SendMessage::status("page list downloaded"))?;

                self.active = MangaReader {
                    pages: result,
                    page: 0,
                    chapters,
                    chapter: 0,
                };
                self.state = State::Reading;
            }
            RecvMessage::NextPage => {
                let manga = &mut self.active;
                manga.page += 1;
                if manga.pages.len() == manga.page {
                    manga.next_chapter(&self.api).await?;
                }
            }
            RecvMessage::PrevPage => {
                let manga = &mut self.active;
                if manga.page == 0 {
                    manga.prev_chapter(&self.api).await?;
                } else {
                    manga.page -= 1;
                }
            }
            RecvMessage::PrevChapter => {
                self.active.prev_chapter(&self.api).await?;
            }
            RecvMessage::NextChapter => {
                self.active.next_chapter(&self.api).await?;
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
                self.active.update_chapter(&self.api).await?;
                self.state = State::Reading;
            }
            RecvMessage::SelectPage(index) => {
                self.active.page = index;
                self.state = State::Reading;
            }
            RecvMessage::Quit => {
                if PathBuf::from("/tmp/mangarr").exists() {
                    remove_dir_all("/tmp/mangarr")?;
                }
                exit(0);
            }
        };
        self.react_to_state(functionality).await?;
        Ok(())
    }

    async fn initiate_chapter_download(&self, functionality: BackendReplier) -> anyhow::Result<()> {
        let mut handlers = self.handlers.lock().await;
        let downloading_status = handlers.entry(self.active.chapter).or_insert(None);
        let priority_page = if downloading_status
            .as_ref()
            .is_some_and(|x| !x.is_finished())
        {
            Some(self.active.page)
        } else {
            None
        };
        drop(handlers);

        let manga = Arc::new(self.active.clone());

        functionality.send_typed_message(SendMessage::PageListChapter(manga.chapter + 1))?;

        let api = self.api.clone();
        let chapter = manga.chapter;
        let handle = tokio::spawn(async move {
            if let Some(page) = priority_page {
                let manga = manga.clone();
                let api = api.clone();
                manga.save_to_disk(&api, page).await?;
                let path = manga.get_url_with_path(page)?.1;
                functionality.send_typed_message(SendMessage::PageModify {
                    chapter: manga.chapter + 1,
                    page,
                    path,
                })?;
                return Ok(());
            }

            let mut iter = tokio_stream::iter(0..manga.pages.len())
                .map(|page| {
                    let manga = manga.clone();
                    let api = api.clone();
                    tokio::spawn(async move {
                        manga.save_to_disk(&api, page).await?;
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
        if priority_page.is_none() {
            self.handlers.lock().await.insert(chapter, Some(handle));
        }
        Ok(())
    }

    async fn react_to_state(&self, functionality: &BackendReplier) -> Result<(), anyhow::Error> {
        match self.state {
            State::Idleing => {}
            State::Reading => {
                let manga_reader = &self.active;

                functionality
                    .send_typed_message(SendMessage::ActivePageNumber(manga_reader.page + 1))?;

                functionality
                    .send_typed_message(SendMessage::TotalPageSize(manga_reader.pages.len()))?;
                functionality.send_typed_message(SendMessage::ActiveChapterNumber(
                    manga_reader.chapter + 1,
                ))?;
                functionality.send_typed_message(SendMessage::TotalChapterSize(
                    manga_reader.chapters.len(),
                ))?;

                for (_, v) in self
                    .handlers
                    .lock()
                    .await
                    .iter_mut()
                    .filter(|(&k, _)| k != self.active.chapter)
                {
                    v.as_ref().unwrap().abort();
                    *v = None;
                }

                self.handlers.lock().await.retain(|_, y| y.is_some());

                functionality.send_typed_message(SendMessage::BackendImage)?;

                self.initiate_chapter_download(*functionality).await?;
            }
            State::ChapterList {
                output: ref chapters,
                ..
            } => {
                functionality.send_typed_message(SendMessage::ChapterList(chapters.to_string()))?;
            }
            State::Search => todo!(),
        };
        Ok(())
    }
}

enum State {
    Idleing,
    ChapterList { output: String },
    Reading,
    Search,
}
#[derive(Clone, Default)]
struct MangaReader {
    chapters: Vec<SChapter>,
    chapter: usize,
    pages: Vec<String>,
    page: usize,
}

impl MangaReader {
    pub async fn next_chapter(&mut self, api: &Manhuagui) -> anyhow::Result<()> {
        self.chapter += 1;
        self.update_chapter(api).await
    }
    pub async fn prev_chapter(&mut self, api: &Manhuagui) -> anyhow::Result<()> {
        self.chapter -= 1;
        self.update_chapter(api).await
    }
    async fn update_chapter(&mut self, api: &Manhuagui) -> anyhow::Result<()> {
        let chpt = &self.chapters[self.chapter];
        let pages = api
            .page_list_parse(chpt)
            .await?
            .into_iter()
            .map(|x| x.image_url)
            .collect::<Vec<_>>();
        self.pages = pages;
        self.page = 0;
        Ok(())
    }
    pub async fn save_to_disk(&self, api: &Manhuagui, page: usize) -> anyhow::Result<()> {
        if !PathBuf::from("/tmp/mangarr").exists() {
            create_dir("/tmp/mangarr/")?;
        }

        let (url, ps) = self.get_url_with_path(page)?;

        if !ps.exists() {
            Self::save_page(url, api, &ps).await?;
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
        api: &Manhuagui,
        path: impl AsRef<Path> + Send,
    ) -> anyhow::Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            let bytes = api
                .client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            let mut p = vec![];

            let encoder = PngEncoder::new(&mut p);

            ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()?
                .decode()?
                .write_with_encoder(encoder)?;

            if !path.exists() {
                let mut file = tokio::fs::File::create_new(path).await?;

                file.write_all(&p).await?;

                file.flush().await?;
            }
        }
        Ok(())
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Idleing
    }
}

impl Default for MyBackend {
    fn default() -> Self {
        Self {
            api: LazyLock::new(|| {
                Manhuagui::new(Preferences::default()).expect("internet issues baby")
            }),
            state: Default::default(),
            active: Default::default(),
            handlers: Default::default(),
            // limit: Arc::new(Semaphore::new(10)),
        }
    }
}

#[async_trait]
impl AppLoadBackend for MyBackend {
    async fn handle_message(&mut self, functionality: &BackendReplier, message: Message) {
        let v = self.handle_message(functionality, message);

        if let Err(err) = v.await {
            functionality
                .send_message(11, &format!("error: {err:#?}"))
                .expect("can't send message");
        }
    }
}
