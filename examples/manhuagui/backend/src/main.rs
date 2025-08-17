mod bookshelf;
mod manga_reader;
mod message;

use std::{collections::HashMap, fs::remove_dir_all, future::Future, path::PathBuf, process::exit};

use anyhow::{Context, bail};
use appload_client::{AppLoadBackend, Message};
use async_compat::Compat;
use async_trait::async_trait;
use backend::MangaBackend;
use bookshelf::BookShelf;
use futures::{StreamExt, stream};
use smol::Task;
use smol::future::block_on;

use crate::{
    manga_reader::{MangaReader, Page},
    message::{RecvMessage, ReplierExt, SendMessage},
};

type BackendReplier = appload_client::BackendReplier<MyBackend>;

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    smol::spawn(Compat::new(future))
}

fn main() {
    block_on(Compat::new(async {
        appload_client::AppLoad::new(MyBackend::new())
            .expect("backend failing to start. please cry")
            .run()
            .await
            .expect("backend failing to start. please cry");
    }));
}

type Backend = dyn MangaBackend;

struct MyBackend {
    bookshelf: BookShelf,
    manga: MangaReader,
    handler: Option<Task<()>>,
    state: State,
}

macro_rules! init_functionality {
    ($func:expr) => {
        let __functionality = $func;
        macro_rules! send_status {
            ($message:expr) => {
                __functionality.send_typed_message(SendMessage::status($message))
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
            handler: None,
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

                let chapters = api
                    .fetch_chapters(&body)
                    .await?
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>();

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
                manga.active.page += 1;
                if manga.pages_len() == manga.active.page {
                    manga.next_chapter().await?;
                }
            }
            RecvMessage::PrevPage => {
                let manga = &mut self.manga;
                if manga.active.page == 0 {
                    manga.prev_chapter().await?;
                } else {
                    manga.active.page -= 1;
                }
                self.state = State::Reading {};
            }
            RecvMessage::PrevChapter => {
                self.manga.prev_chapter().await?;
            }
            RecvMessage::NextChapter => {
                self.manga.next_chapter().await?;
            }
            RecvMessage::GetChapterList => {
                self.state = State::ChapterList;
            }
            RecvMessage::SelectChapter(index) => {
                self.manga.active.chapter = index;
                self.manga.update_chapter().await?;
                self.state = State::Reading;
            }
            RecvMessage::SelectPage(index) => {
                self.manga.active.page = index;
                if let Some(x) = self.handler.take() {
                    x.cancel().await;
                }

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
                // let p = manga.page;
                // for task in self.handlers.values_mut() {
                //     task.take().unwrap().cancel().await;
                // }
                // self.handlers.clear();
                self.state = State::Reading;
                self.manga.update_chapter().await?;
                self.manga.send_details(functionality)?;
            }
            RecvMessage::BookShelfView => {
                self.state = State::Bookshelf;
            }
        }
        self.react_to_state(functionality).await?;
        Ok(())
    }

    fn initiate_chapter_download(
        &mut self,
        functionality: BackendReplier,
        start: usize,
    ) -> anyhow::Result<()> {
        // let manga = self.manga.clone();

        // let chapter = manga.chapter;
        // let client = manga.api.client();

        // let handle = spawn(async move {
        //     let len = manga.pages.len();

        //     let mut iter = stream::iter(start..len)
        //         .map(|page| {
        //             let client = client.clone();
        //             let manga = manga.clone();
        //             spawn(async move {
        //                 manga.save_to_disk(client, page).await?;
        //                 anyhow::Ok(page)
        //             })
        //         })
        //         .buffered(3);

        //     while let Some(x) = iter.next().await {
        //         let page = x?;

        //         let path = manga.get_url_with_path(page)?.1;
        //         functionality.send_typed_message(SendMessage::PageModify {
        //             chapter: manga.chapter + 1,
        //             page,
        //             path,
        //         })?;
        //     }
        //     anyhow::Ok(())
        // });

        // self.handlers
        //     .entry((chapter, start))
        //     .or_insert(Some(handle));
        Ok(())
    }

    async fn react_to_state(
        &mut self,
        functionality: &BackendReplier,
    ) -> Result<(), anyhow::Error> {
        match self.state {
            State::Idleing => {}
            State::Reading => {
                self.manga.send_page_information(functionality)?;

                if let Some(bookshelf_manga) = self.bookshelf.get_mut(&self.manga) {
                    bookshelf_manga.active = self.manga.active;
                }

                self.manga
                    .save_to_disk(self.manga.active, functionality)
                    .await?;
                self.manga
                    .prefetch(self.manga.pages_len(), functionality)
                    .detach();

                functionality.send_typed_message(SendMessage::BackendImage)?;
            }
            State::ChapterList => {
                let chapters = self
                    .manga
                    .chapters
                    .iter()
                    .map(|x| &*x.name)
                    .collect::<Vec<&str>>()
                    .join("\n");

                functionality.send_typed_message(SendMessage::ChapterList(chapters))?;
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
                search.send_details(functionality)?;
                search.send_page_information(functionality)?;
                search.save_to_disk(Page::default(), functionality).await?;
                functionality.send_typed_message(SendMessage::MangaPreview(
                    search.get_url_with_path(Page::default())?.1,
                ))?;
            }
            State::Bookshelf => {
                for v in self.bookshelf.bookshelf().values() {
                    functionality.send_typed_message(SendMessage::BookshelfMangaDetails(
                        Box::new(v.clone()),
                    ))?;
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
