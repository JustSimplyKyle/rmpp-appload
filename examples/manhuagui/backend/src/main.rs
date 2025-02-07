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
use appload_client::{start, AppLoadBackend, BackendReplier, Message, MSG_SYSTEM_NEW_COORDINATOR};
use async_trait::async_trait;
use backend::{
    manhuagui::{Manhuagui, Preferences},
    nhentai::NHentai,
    MangaBackend, Page, SChapter, SManga,
};
use futures_util::StreamExt;
use image::{
    codecs::png::PngEncoder, imageops::FilterType, DynamicImage, GenericImageView, ImageReader,
    Rgba,
};
use reqwest::Client;
use tokio::{io::AsyncWriteExt, sync::Mutex, task::JoinHandle};

#[tokio::main]
async fn main() {
    start(&mut MyBackend::default())
        .await
        .expect("backend failing to start. please cry");
}

type Backend<'a> = &'a SupportedBackends;

#[derive(Debug, Clone)]
enum SupportedBackends {
    NHentai(NHentai),
    Manhuagui(Manhuagui),
}

// clone of `MangaBackend` to get around object safety issues surrounding clone
impl SupportedBackends {
    fn client(&self) -> Client {
        match self {
            Self::NHentai(x) => x.client(),
            Self::Manhuagui(x) => x.client(),
        }
    }
    async fn search_by_id(&self, id: &str) -> anyhow::Result<SManga> {
        match self {
            Self::NHentai(x) => x.search_by_id(id).await,
            Self::Manhuagui(x) => x.search_by_id(id).await,
        }
    }

    async fn fetch_chapters(&self, manga: &SManga) -> anyhow::Result<Vec<SChapter>> {
        match self {
            Self::NHentai(x) => x.fetch_chapters(manga).await,
            Self::Manhuagui(x) => x.fetch_chapters(manga).await,
        }
    }

    async fn fetch_pages(&self, chapter: &SChapter) -> anyhow::Result<Vec<Page>> {
        match self {
            Self::NHentai(x) => x.fetch_pages(chapter).await,
            Self::Manhuagui(x) => x.fetch_pages(chapter).await,
        }
    }
}

struct MyBackend {
    api: SupportedBackends,
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
    ConfirmMangaSearch,
    SelectBackend(SupportedBackends),
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
            10 => Self::ConfirmMangaSearch,
            11 => Self::SelectBackend(match &*message.contents {
                "NHentai" => {
                    SupportedBackends::NHentai(NHentai::new("zh-tw".into(), "zh-tw".into(), true)?)
                }
                "Manhuagui" => {
                    SupportedBackends::Manhuagui(Manhuagui::new(Preferences::default())?)
                }
                _ => bail!("Unsupported backend."),
            }),
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
            Self::BackendImage => (101, None),
            Self::Error(s) => (1000, Some(s)),
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
                let api = &self.api;
                functionality.send_typed_message(SendMessage::status("starts downloading"))?;

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
            RecvMessage::ConfirmMangaSearch => {
                let State::Search { search: manga, .. } = &mut self.state else {
                    bail!("impossible state reached");
                };
                self.active = std::mem::take(manga);
                self.state = State::Reading;
            }
            RecvMessage::SelectBackend(supported_backend) => {
                use SupportedBackends as SBC;
                let is_different = matches!(
                    (&self.api, &supported_backend),
                    (SBC::NHentai(_), SBC::Manhuagui(_)) | (SBC::Manhuagui(_), SBC::NHentai(_))
                );
                if is_different {
                    self.api = supported_backend;
                    self.active = MangaReader::default();
                    if let State::Search { search: manga, .. } = &mut self.state {
                        std::mem::take(manga);
                    }
                }
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

    async fn react_to_state(
        &mut self,
        functionality: &BackendReplier,
    ) -> Result<(), anyhow::Error> {
        match self.state {
            State::Idleing => {}
            State::Reading => {
                self.active.send_page_information(*functionality)?;

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
            State::Search {
                ref mut search,
                confirm,
            } => {
                if confirm {
                    let manga = std::mem::take(search);
                    self.active = *manga;
                    self.state = State::Reading;
                    Box::pin(self.react_to_state(functionality)).await?;
                    return Ok(());
                }
                search.send_details(*functionality)?;
                search.send_page_information(*functionality)?;
                search.save_to_disk(&self.api, 0).await?;
                functionality.send_typed_message(SendMessage::MangaPreview(
                    search.get_url_with_path(0)?.1,
                ))?;
            }
        };
        Ok(())
    }
}

enum State {
    Idleing,
    ChapterList {
        output: String,
    },
    Reading,
    Search {
        search: Box<MangaReader>,
        confirm: bool,
    },
}
#[derive(Clone, Default, Debug)]
struct MangaReader {
    details: SManga,
    chapters: Vec<SChapter>,
    chapter: usize,
    pages: Vec<String>,
    page: usize,
}

impl MangaReader {
    pub fn send_details(&self, functionality: BackendReplier) -> anyhow::Result<()> {
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
    pub fn send_page_information(&self, functionality: BackendReplier) -> anyhow::Result<()> {
        functionality.send_typed_message(SendMessage::ActivePageNumber(self.page + 1))?;

        functionality.send_typed_message(SendMessage::TotalPageSize(self.pages.len()))?;
        functionality.send_typed_message(SendMessage::ActiveChapterNumber(self.chapter + 1))?;
        functionality.send_typed_message(SendMessage::TotalChapterSize(self.chapters.len()))?;
        Ok(())
    }
    pub async fn next_chapter(&mut self, api: Backend<'_>) -> anyhow::Result<()> {
        self.chapter += 1;
        self.update_chapter(api).await
    }
    pub async fn prev_chapter(&mut self, api: Backend<'_>) -> anyhow::Result<()> {
        self.chapter -= 1;
        self.update_chapter(api).await
    }
    async fn update_chapter(&mut self, api: Backend<'_>) -> anyhow::Result<()> {
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
    pub async fn save_to_disk(&self, api: Backend<'_>, page: usize) -> anyhow::Result<()> {
        if !PathBuf::from("/tmp/mangarr").exists() {
            create_dir("/tmp/mangarr/")?;
        }
        if !PathBuf::from("/tmp/mangarr/preview").exists() {
            create_dir("/tmp/mangarr/preview")?;
        }

        let (url, ps) = self.get_url_with_path(page)?;

        if !ps.exists() {
            let image = Self::save_page(url, api, &ps).await?;
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
        api: Backend<'_>,
        path: impl AsRef<Path> + Send,
    ) -> anyhow::Result<Option<DynamicImage>> {
        let path = path.as_ref();
        if !path.exists() {
            let bytes = api
                .client()
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

impl Default for MyBackend {
    fn default() -> Self {
        Self {
            api: SupportedBackends::Manhuagui(
                Manhuagui::new(Preferences::default()).expect("invalid"),
            ),
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
                .send_message(1000, &format!("error: {err:#?}"))
                .expect("can't send message");
        }
    }
}
