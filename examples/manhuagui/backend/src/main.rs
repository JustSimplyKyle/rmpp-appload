use std::{
    cell::LazyCell,
    fs::{create_dir, remove_dir, remove_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    path::{Path, PathBuf},
    process::exit,
};

use anyhow::{bail, Context};
use appload_client::{start, AppLoadBackend, BackendReplier, Message, MSG_SYSTEM_NEW_COORDINATOR};
use async_trait::async_trait;
use backend::{Manhuagui, Preferences, SChapter};
use image::ImageReader;

#[tokio::main]
async fn main() {
    start(&mut MyBackend::default()).await.unwrap();
}

struct MyBackend {
    api: LazyCell<Manhuagui>,
    state: State,
}

impl MyBackend {
    #[allow(clippy::too_many_lines)]
    async fn handle_message(
        &mut self,
        functionality: &BackendReplier,
        message: Message,
    ) -> anyhow::Result<()> {
        match message.msg_type {
            MSG_SYSTEM_NEW_COORDINATOR => {
                functionality.send_message(11, "connected frontend")?;

                println!("A frontend has connected");
            }
            1 => {
                let api = &*self.api;

                functionality.send_message(11, "starts downloading!")?;

                let search_term = message.contents;

                let body = api
                    .fetch_search_manga(0, &search_term, vec![])
                    .await?
                    .mangas
                    .into_iter()
                    .next()
                    .context("empty")?;

                functionality.send_message(11, "body finished!")?;

                let chapters = api
                    .chapter_list_parse(&body)
                    .await?
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>();

                let first_chapter = chapters.first().context("empty")?;

                functionality.send_message(11, "chapter finished!")?;

                let result = api
                    .page_list_parse(first_chapter)
                    .await?
                    .into_iter()
                    .map(|x| x.image_url)
                    .collect::<Vec<_>>();

                functionality.send_message(11, "page list downloadde")?;

                self.state = State::Reading(MangaReader {
                    pages: result,
                    page: 0,
                    chapters,
                    chapter: 0,
                });
            }
            2 => {
                let State::Reading(manga) = &mut self.state else {
                    return Ok(());
                };
                manga.page += 1;
                if manga.pages.len() == manga.page {
                    manga.next_chapter(&self.api).await?;
                }
            }
            3 => {
                let State::Reading(manga) = &mut self.state else {
                    return Ok(());
                };
                if manga.page == 0 {
                    manga.prev_chapter(&self.api).await?;
                } else {
                    manga.page -= 1;
                }
            }
            4 => {
                let State::Reading(manga) = &mut self.state else {
                    bail!("invalid state, should be reading");
                };
                manga.prev_chapter(&self.api).await?;
            }
            5 => {
                let State::Reading(manga) = &mut self.state else {
                    bail!("invalid state, should be reading");
                };
                manga.next_chapter(&self.api).await?;
            }
            6 => {
                let State::Reading(manga) = &mut self.state else {
                    bail!("invalid state, should be reading");
                };
                let manga = std::mem::take(manga);

                let output = manga
                    .chapters
                    .iter()
                    .map(|x| &*x.name)
                    .collect::<Vec<&str>>()
                    .join("\n");

                self.state = State::ChapterList { output, manga };
            }
            7 => {
                let State::ChapterList { manga, .. } = &mut self.state else {
                    bail!("invalid state, should be reading");
                };
                let mut manga = std::mem::take(manga);
                let index = message.contents.parse::<usize>()?;
                manga.chapter = index;
                self.state = State::Reading(manga);
            }
            99 => {
                if PathBuf::from("/tmp/mangarr").exists() {
                    remove_dir_all("/tmp/mangarr")?;
                }
                exit(0);
            }
            _ => bail!("Unknown message received."),
        };
        match &self.state {
            State::Idleing => {}
            State::Reading(manga_reader) => {
                functionality.send_message(4, &(manga_reader.page + 1).to_string())?;
                functionality.send_message(5, &manga_reader.pages.len().to_string())?;
                functionality.send_message(6, &(manga_reader.chapter + 1).to_string())?;
                functionality.send_message(7, &manga_reader.chapters.len().to_string())?;
                manga_reader.display(&self.api, functionality).await?;
            }
            State::ChapterList {
                output: chapters, ..
            } => {
                functionality.send_message(8, chapters)?;
            }
        }
        Ok(())
    }
}

enum State {
    Idleing,
    ChapterList { output: String, manga: MangaReader },
    Reading(MangaReader),
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
            functionality.send_message(11, "downloading image")?;
            Self::save_page(url, api, &ps).await?;
            functionality.send_message(11, "finish downloading")?;
        }

        let p = self
            .pages
            .iter()
            .enumerate()
            .skip(self.page + 1)
            .take(5)
            .flat_map(|(u, _)| self.get_url_with_path(u))
            .map(|(x, y)| (x.to_owned(), y));

        for (url, path) in p {
            if !path.exists() {
                functionality.send_message(11, "prefetching image")?;
                let api = api.clone();
                tokio::spawn(async move {
                    Self::save_page(&url, &api.clone(), path).await.unwrap();
                });
                functionality.send_message(11, "finish downloading")?;
            }
        }

        functionality.send_message(101, &format!("file:{}", ps.display()))?;
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

            ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()?
                .decode()?
                .save_with_format(path, image::ImageFormat::Png)?;
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
            api: LazyCell::new(|| Manhuagui::new(Preferences::default()).unwrap()),
            state: Default::default(),
        }
    }
}

#[async_trait]
impl AppLoadBackend for MyBackend {
    async fn handle_message(&mut self, functionality: &BackendReplier, message: Message) {
        let v = self.handle_message(functionality, message);

        let status = match v.await {
            Ok(()) => "status: success".to_owned(),
            Err(x) => format!("error: {x:#?}"),
        };

        functionality
            .send_message(11, &status)
            .expect("can't send message");
    }
}
