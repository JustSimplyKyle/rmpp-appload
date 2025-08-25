mod bookshelf;
mod manga_reader;
mod message;

use std::{
    collections::HashMap, fs::remove_dir_all, future::Future, path::PathBuf, pin::Pin,
    process::exit, task::Poll,
};

use anyhow::{Context, bail};
use appload_client::{AppLoadBackend, Message};
use async_compat::Compat;
use async_trait::async_trait;
use backend::MangaBackend;
use bookshelf::BookShelf;
use futures::stream::{AbortHandle, Abortable, Aborted};
use smol::future::block_on;

use crate::{
    manga_reader::{MangaReader, Page},
    message::{RecvMessage, ReplierExt, SendMessage},
};

type BackendReplier = appload_client::BackendReplier<MyBackend>;

pub fn spawn<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static,
) -> AbortableTask<T> {
    AbortableTask::spawn(Compat::new(future))
}

pub struct AbortableTask<T> {
    handle: AbortHandle,
    task: smol::Task<Result<T, Aborted>>,
}

impl<T: Send + 'static> AbortableTask<T> {
    pub fn spawn(future: impl Future<Output = T> + Send + 'static) -> Self {
        let (handle, reg) = AbortHandle::new_pair();
        let abortable = Abortable::new(future, reg);
        let task = smol::spawn(abortable);

        Self { handle, task }
    }

    pub fn abort(&self) {
        self.handle.abort();
    }

    pub fn detach(self) {
        self.task.detach();
    }
}

impl<T> Future for AbortableTask<T> {
    type Output = Result<T, Aborted>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        Pin::new(&mut this.task).poll(cx)
    }
}

fn main() {
    unsafe { std::env::set_var("SMOL_THREADS", "4") };
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };

    block_on(Compat::new(async {
        appload_client::AppLoad::new(MyBackend::new())
            .expect("backend failing to start. please cry")
            .run()
            .await
            .expect("backend failing to start. please cry");
    }));
}

struct MyBackend {
    bookshelf: BookShelf,
    manga: MangaReader,
    handlers: HashMap<usize, AbortableTask<(usize, usize)>>,
    state: State,
}

macro_rules! init_functionality {
    ($func:expr) => {
        let __functionality = $func;
        macro_rules! send_status {
            ($message:expr) => {
                __functionality
                    .send_typed_message(SendMessage::status($message))
                    .await
            };
        }
    };
}
impl MyBackend {
    fn new() -> Self {
        Self {
            bookshelf: BookShelf::new().unwrap(),
            manga: MangaReader::new(None, None).unwrap(),
            state: State::default(),
            handlers: HashMap::new(),
        }
    }
    #[allow(clippy::too_many_lines)]
    async fn handle_message(
        &mut self,
        functionality: &BackendReplier,
        message: Message,
    ) -> anyhow::Result<()> {
        init_functionality!(&functionality);

        match RecvMessage::try_from(message)? {
            RecvMessage::Connect => {
                send_status!("connected frontend")?;

                println!("A frontend has connected");
            }
            RecvMessage::SearchManga(search_term) => {
                send_status!("starts downloading")?;

                let api = self.manga.api.clone();

                let body = api.search_by_id(&search_term).await?;

                send_status!("body finished!")?;

                let chapters = api.fetch_chapters(&body).await?;

                let first_chapter = chapters.first().context("empty")?;

                send_status!("chapter finished!")?;

                let result = api
                    .fetch_pages(first_chapter)
                    .await?
                    .into_iter()
                    .map(|x| x.image_url)
                    .collect::<Vec<_>>();

                send_status!("page list downloaded")?;

                let search = MangaReader::new(api, (body, chapters, result, Default::default()))?;

                self.state = State::Search {
                    search: Box::new(search),
                    confirm: false,
                };
            }
            RecvMessage::NextPage => {
                let manga = &mut self.manga;
                manga.current_page.page += 1;
                if manga.pages_len() == manga.current_page.page {
                    manga.next_chapter().await?;
                }
            }
            RecvMessage::PrevPage => {
                let manga = &mut self.manga;
                if manga.current_page.page == 0 {
                    manga.prev_chapter().await?;
                } else {
                    manga.current_page.page -= 1;
                }
                self.state = State::Reading;
            }
            RecvMessage::PrevChapter => {
                self.manga.prev_chapter().await?;
                self.bookshelf.insert(self.manga.clone()).await?;
                self.clear_download_handles().await;
            }
            RecvMessage::NextChapter => {
                self.manga.next_chapter().await?;
                self.bookshelf.insert(self.manga.clone()).await?;
                self.clear_download_handles().await;
            }
            RecvMessage::GetChapterList => {
                self.state = State::ChapterList;
            }
            RecvMessage::SelectChapter(index) => {
                self.manga.with_chapter_mut(|x| *x = index).await?;
                self.bookshelf.insert(self.manga.clone()).await?;
                self.clear_download_handles().await;
                self.state = State::Reading;
            }
            RecvMessage::SelectPage(index) => {
                self.manga.current_page.page = index;

                self.clear_download_handles().await;

                self.state = State::Reading;
            }
            RecvMessage::ConfirmMangaSearch => {
                let State::Search { search: manga, .. } =
                    std::mem::replace(&mut self.state, State::Reading)
                else {
                    bail!("impossible state reached");
                };
                self.manga = *manga;
            }
            RecvMessage::SelectBackend(supported_backend) => {
                let is_different = { dbg!(supported_backend.id()) != dbg!(self.manga.api.id()) };
                if is_different {
                    let mut manga_reader = MangaReader::new(None, None)?;
                    manga_reader.api = supported_backend;

                    self.manga = manga_reader;

                    if let State::Search { search: manga, .. } = &mut self.state {
                        *manga = Box::new(self.manga.clone());
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
                self.bookshelf.insert(self.manga.clone()).await?;
            }
            RecvMessage::SelectBookFromBookShelf(key) => {
                let manga = self
                    .bookshelf
                    .bookshelf()
                    .get(&key)
                    .context("somehow missing bookshelf stuff")?
                    .clone();
                self.manga = manga;
                self.state = State::Reading;
                self.manga.update_chapter().await?;
                self.manga.send_details(functionality).await?;
            }
            RecvMessage::BookShelfView => {
                self.state = State::Bookshelf;
            }
        }
        self.react_to_state(functionality).await?;
        Ok(())
    }

    async fn clear_download_handles(&mut self) {
        for x in self.handlers.values_mut() {
            x.abort();
        }
        self.handlers.clear();
        self.manga.clear_download_managear().await;
    }

    async fn react_to_state(
        &mut self,
        functionality: &BackendReplier,
    ) -> Result<(), anyhow::Error> {
        match self.state {
            State::Idleing => {}
            State::Reading => {
                self.manga.send_page_information(functionality).await?;

                self.bookshelf
                    .with_mut(
                        |manga| {
                            if let Some(bookshelf_manga) = manga {
                                bookshelf_manga.pages.clone_from(&self.manga.pages);
                                bookshelf_manga.current_page = self.manga.current_page;
                            }
                        },
                        &self.manga,
                    )
                    .await?;

                self.manga
                    .save_to_disk(self.manga.current_page, functionality)
                    .await?;

                if !self
                    .handlers
                    .contains_key(&self.manga.current_page.chapter())
                {
                    self.manga.prefetch_chapters();

                    let handle = self
                        .manga
                        .prefetch_pages(self.manga.pages_len(), functionality);
                    self.handlers
                        .insert(self.manga.current_page.chapter(), handle);
                }

                functionality
                    .send_typed_message(SendMessage::BackendImage)
                    .await?;
            }
            State::ChapterList => {
                let chapters = self
                    .manga
                    .chapters
                    .iter()
                    .map(|x| &*x.name)
                    .collect::<Vec<&str>>()
                    .join("\n");

                functionality
                    .send_typed_message(SendMessage::ChapterList(chapters))
                    .await?;
            }
            State::Search {
                ref search,
                confirm,
            } => {
                if confirm {
                    let State::Search { search: manga, .. } =
                        std::mem::replace(&mut self.state, State::Reading)
                    else {
                        bail!("impossible state reached");
                    };
                    self.manga = *manga;
                    self.state = State::Reading;
                    Box::pin(self.react_to_state(functionality)).await?;
                    return Ok(());
                }
                search.send_details(functionality).await?;
                search.send_page_information(functionality).await?;
                search.save_to_disk(Page::default(), functionality).await?;
                functionality
                    .send_typed_message(SendMessage::MangaPreview(
                        search.get_url_with_path(Page::default())?.1,
                    ))
                    .await?;
            }
            State::Bookshelf => {
                for v in self.bookshelf.bookshelf().values() {
                    functionality
                        .send_typed_message(SendMessage::BookshelfMangaDetails(Box::new(v.clone())))
                        .await?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum State {
    Idleing,
    Bookshelf,
    ChapterList,
    Reading,
    Search {
        search: Box<MangaReader>,
        confirm: bool,
    },
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
        v.await.unwrap();
        // if let Err(err) = v.await {
        // err.unwrap();
        // let err = err
        //     .chain()
        //     .enumerate()
        //     .map(|(i, x)| format!("{}:{x:#?}", i + 1))
        //     .collect::<Vec<_>>()
        //     .join("\n");
        // panic!("{err:#?}")
        // functionality
        //     .send_typed_message(SendMessage::Error(format!("error: {err:#?}")))
        //     .await
        //     .expect("can't send message");
        // }
    }
}
