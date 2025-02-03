use std::{
    collections::HashMap,
    fs::{create_dir, remove_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    path::{Path, PathBuf},
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock,
    },
    time::Duration,
};

use anyhow::{bail, Context};
use appload_client::{start, AppLoadBackend, BackendReplier, Message, MSG_SYSTEM_NEW_COORDINATOR};
use async_trait::async_trait;
use backend::{Manhuagui, Preferences, SChapter};
use futures_util::{stream::FuturesOrdered, StreamExt};
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
    handles: Arc<Mutex<FuturesOrdered<JoinHandle<anyhow::Result<usize>>>>>,
    output_pages: Arc<Mutex<Vec<String>>>,
    active: MangaReader,
    state: State,
}

#[derive(Debug)]
enum MessageType {
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

impl TryFrom<Message> for MessageType {
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
    Status = 11,
    BackendImage = 101,
    ActivePageNumber = 4,
    TotalPageSize = 5,
    ActiveChapterNumber = 6,
    TotalChapterSize = 7,
    ChapterList = 8,
    PageList = 9,
}

impl From<SendMessage> for u32 {
    fn from(val: SendMessage) -> Self {
        val as Self
    }
}

impl MyBackend {
    #[allow(clippy::too_many_lines)]
    async fn handle_message(
        &mut self,
        functionality: &BackendReplier,
        message: Message,
    ) -> anyhow::Result<()> {
        match MessageType::try_from(message)? {
            MessageType::Connect => {
                functionality.send_message(SendMessage::Status.into(), "connected frontend")?;

                println!("A frontend has connected");
            }
            MessageType::SearchManga(search_term) => {
                let api = &*self.api;

                functionality.send_message(SendMessage::Status.into(), "starts downloading!")?;

                let body = api
                    .fetch_search_manga(0, &search_term, vec![])
                    .await?
                    .mangas
                    .into_iter()
                    .next()
                    .context("empty")?;

                functionality.send_message(SendMessage::Status.into(), "body finished!")?;

                let chapters = api
                    .chapter_list_parse(&body)
                    .await?
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>();

                let first_chapter = chapters.first().context("empty")?;

                functionality.send_message(SendMessage::Status.into(), "chapter finished!")?;

                let result = api
                    .page_list_parse(first_chapter)
                    .await?
                    .into_iter()
                    .map(|x| x.image_url)
                    .collect::<Vec<_>>();

                functionality.send_message(SendMessage::Status.into(), "page list downloaded")?;

                self.active = MangaReader {
                    pages: result,
                    page: 0,
                    chapters,
                    chapter: 0,
                    chapter_downloading_status: HashMap::from_iter(vec![(
                        0,
                        Arc::new(false.into()),
                    )]),
                };
                self.state = State::Reading;
            }
            MessageType::NextPage => {
                let manga = &mut self.active;
                manga.page += 1;
                if manga.pages.len() == manga.page {
                    manga.next_chapter(&self.api).await?;
                }
            }
            MessageType::PrevPage => {
                let manga = &mut self.active;
                if manga.page == 0 {
                    manga.prev_chapter(&self.api).await?;
                } else {
                    manga.page -= 1;
                }
            }
            MessageType::PrevChapter => {
                self.active.prev_chapter(&self.api).await?;
            }
            MessageType::NextChapter => {
                self.active.next_chapter(&self.api).await?;
            }
            MessageType::GetChapterList => {
                let output = self
                    .active
                    .chapters
                    .iter()
                    .map(|x| &*x.name)
                    .collect::<Vec<&str>>()
                    .join("\n");

                self.state = State::ChapterList { output };
            }
            MessageType::SelectChapter(index) => {
                self.active.chapter = index;
                self.active.update_chapter(&self.api).await?;
                self.state = State::Reading;
            }
            MessageType::SelectPage(index) => {
                self.active.page = index;
                self.active.display(&self.api, functionality).await?;
                self.state = State::Reading;
            }
            MessageType::Quit => {
                if PathBuf::from("/tmp/mangarr").exists() {
                    remove_dir_all("/tmp/mangarr")?;
                }
                exit(0);
            }
        };
        self.react_to_state(functionality).await?;
        Ok(())
    }

    async fn initiate_chapter_download(
        &mut self,
        functionality: BackendReplier,
    ) -> anyhow::Result<()> {
        let downloading_status = self
            .active
            .chapter_downloading_status
            .entry(self.active.chapter)
            .or_default()
            .clone();
        if downloading_status.load(Ordering::Relaxed) {
            return Ok(());
        }

        let manga = Arc::new(self.active.clone());

        let handles = tokio_stream::iter(0..manga.pages.len())
            .map(|page| {
                let api = self.api.clone();
                let manga = manga.clone();
                tokio::spawn(async move {
                    manga.save_to_disk(&api, page).await?;
                    Ok(page)
                })
            })
            .collect::<Vec<_>>()
            .await;

        self.handles.lock().await.extend(handles.into_iter());
        self.output_pages = Arc::new(Mutex::new(
            (0..manga.pages.len()).map(|_| String::new()).collect(),
        ));

        let manga = self.active.clone();
        let handles = self.handles.clone();
        let output_pages = self.output_pages.clone();
        let is_running = downloading_status;
        tokio::spawn(async move {
            is_running.store(true, std::sync::atomic::Ordering::Relaxed);

            while let Some(x) = handles.lock().await.next().await {
                let page = x??;

                let path = manga.get_url_with_path(page)?.1;
                let mut output_pages = output_pages.lock().await;
                output_pages[page] = format!("file:{}", path.display());
                functionality
                    .send_message(SendMessage::PageList.into(), &output_pages.join("\n"))?;
            }
            is_running.store(false, std::sync::atomic::Ordering::Relaxed);
            anyhow::Ok(())
        });

        Ok(())
    }

    async fn react_to_state(
        &mut self,
        functionality: &BackendReplier,
    ) -> Result<(), anyhow::Error> {
        match self.state {
            State::Idleing => {}
            State::Reading => {
                let manga_reader = &self.active;
                functionality.send_message(
                    SendMessage::ActivePageNumber.into(),
                    &(manga_reader.page + 1).to_string(),
                )?;
                functionality.send_message(
                    SendMessage::TotalPageSize.into(),
                    &manga_reader.pages.len().to_string(),
                )?;
                functionality.send_message(
                    SendMessage::ActiveChapterNumber.into(),
                    &(manga_reader.chapter + 1).to_string(),
                )?;
                functionality.send_message(
                    SendMessage::TotalChapterSize.into(),
                    &manga_reader.chapters.len().to_string(),
                )?;

                manga_reader.display(&self.api, functionality).await?;

                self.initiate_chapter_download(*functionality).await?;
            }
            State::ChapterList {
                output: ref chapters,
                ..
            } => {
                functionality.send_message(SendMessage::ChapterList.into(), chapters)?;
            }
        };
        Ok(())
    }
}

enum State {
    Idleing,
    ChapterList { output: String },
    Reading,
}
#[derive(Clone, Default)]
struct MangaReader {
    chapters: Vec<SChapter>,
    chapter: usize,
    pages: Vec<String>,
    page: usize,
    chapter_downloading_status: HashMap<usize, Arc<AtomicBool>>,
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
    pub async fn display(
        &self,
        api: &Manhuagui,
        functionality: &BackendReplier,
    ) -> anyhow::Result<()> {
        if !PathBuf::from("/tmp/mangarr").exists() {
            create_dir("/tmp/mangarr/")?;
        }

        let (url, ps) = self.get_url_with_path(self.page)?;

        if !ps.exists() {
            functionality.send_message(SendMessage::Status.into(), "downloading image")?;
            Self::save_page(url, api, &ps).await?;
            functionality.send_message(SendMessage::Status.into(), "finish downloading")?;
        }

        functionality.send_message(
            SendMessage::BackendImage.into(),
            &format!("file:{}", ps.display()),
        )?;
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
            handles: Default::default(),
            active: Default::default(),
            output_pages: Default::default(),
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
