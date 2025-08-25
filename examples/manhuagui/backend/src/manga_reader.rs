use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, bail};
use backend::{
    ImageUrl, SChapter, SManga,
    manhuagui::{Manhuagui, Preferences},
};
use futures::{StreamExt, TryStreamExt, stream};
use palette::{Clamp, IntoColor, Oklch, Srgb, encoding::srgb};
use photon_rs::{
    PhotonImage, Rgb, Rgba,
    colour_spaces::{saturate_hsluv, saturate_hsv, saturate_lch},
    effects,
    helpers::save_dyn_image,
    monochrome,
    native::{open_image_from_bytes, save_image},
    transform::{SamplingFilter, resize},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use smol::{
    Task,
    fs::{self, File, create_dir},
    io::AsyncWriteExt,
    lock::RwLock,
};

use crate::{
    AbortableTask, Backend, BackendReplier,
    message::{ReplierExt, SendMessage},
    spawn,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MangaReader {
    pub api: Arc<Backend>,
    pub details: Arc<SManga>,
    pub chapters: Arc<[SChapter]>,
    pub pages: HashMap<usize, Arc<[ImageUrl]>>,
    pub current_page: Page,
    #[serde(skip)]
    download_manager: Arc<RwLock<DownloadManager>>,
    #[serde(skip)]
    chapters_manager: Arc<RwLock<HashMap<usize, Arc<[ImageUrl]>>>>,
}

#[derive(Debug, Clone, Default)]
struct DownloadManager(HashSet<Page>);

impl std::ops::DerefMut for DownloadManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for DownloadManager {
    type Target = HashSet<Page>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, Copy, PartialEq, Eq, Hash)]
pub struct Page {
    pub page: usize,
    chapter: usize,
}
impl Page {
    pub const fn chapter(&self) -> usize {
        self.chapter
    }
}

impl MangaReader {
    pub fn new(
        api: impl Into<Option<Arc<Backend>>>,
        params: impl Into<Option<(SManga, Vec<SChapter>, Vec<ImageUrl>, Page)>>,
    ) -> anyhow::Result<Self> {
        let api = match api.into() {
            Some(x) => x,
            None => Arc::new(Manhuagui::new(Preferences::default())?),
        };
        let manga_reader = if let Some((details, chapters, pages, active)) = params.into() {
            Self {
                api,
                details: details.into(),
                chapters: chapters.into(),
                pages: HashMap::from([(0, pages.into())]),
                current_page: active,
                download_manager: Default::default(),
                chapters_manager: Default::default(),
            }
        } else {
            Self {
                api,
                details: Default::default(),
                chapters: Default::default(),
                pages: Default::default(),
                current_page: Default::default(),
                download_manager: Default::default(),
                chapters_manager: Default::default(),
            }
        };
        Ok(manga_reader)
    }
    pub async fn send_details(&self, functionality: &BackendReplier) -> anyhow::Result<()> {
        if let Some(description) = self.details.description.clone() {
            functionality
                .send_typed_message(SendMessage::MangaDescription(description))
                .await?;
        }
        if let Some(author) = self.details.author.clone() {
            functionality
                .send_typed_message(SendMessage::MangaAuthor(author))
                .await?;
        }
        functionality
            .send_typed_message(SendMessage::MangaLastUpdatedTime(
                self.details.last_updated_time.clone(),
            ))
            .await?;
        functionality
            .send_typed_message(SendMessage::MangaName(self.details.title.clone()))
            .await?;
        Ok(())
    }
    pub async fn send_page_information(
        &self,
        functionality: &BackendReplier,
    ) -> anyhow::Result<()> {
        functionality
            .send_typed_message(SendMessage::ActivePageNumber(self.current_page.page + 1))
            .await?;

        functionality
            .send_typed_message(SendMessage::TotalPageSize(self.pages().len()))
            .await?;
        functionality
            .send_typed_message(SendMessage::ActiveChapterNumber(
                self.current_page.chapter + 1,
            ))
            .await?;
        functionality
            .send_typed_message(SendMessage::TotalChapterSize(self.chapters.len()))
            .await?;
        Ok(())
    }
    pub fn pages(&self) -> &[ImageUrl] {
        &self.pages[&self.current_page.chapter]
    }
    pub async fn with_chapter_mut(&mut self, f: impl Fn(&mut usize)) -> anyhow::Result<()> {
        f(&mut self.current_page.chapter);
        self.update_chapter().await?;
        self.current_page.page = 0;
        Ok(())
    }
    pub async fn next_chapter(&mut self) -> anyhow::Result<()> {
        self.with_chapter_mut(|x| *x += 1).await
    }
    pub async fn prev_chapter(&mut self) -> anyhow::Result<()> {
        self.with_chapter_mut(|x| *x -= 1).await
    }
    pub fn pages_len(&self) -> usize {
        self.pages().len()
    }
    pub async fn update_chapter(&mut self) -> anyhow::Result<()> {
        let chapter = self.current_page.chapter;
        if !self.pages.contains_key(&chapter) {
            if let Some(pages) = self.chapters_manager.read().await.get(&chapter) {
                self.pages.insert(chapter, pages.clone());
            } else {
                dbg!("chapters manager not having this shit");
                let pages = self.download_pages_url(chapter).await?;
                self.pages.insert(chapter, pages);
            }
        }
        Ok(())
    }

    pub fn prefetch_pages(
        &self,
        amount: usize,
        functionality: &BackendReplier,
    ) -> AbortableTask<(usize, usize)> {
        let current = self.current_page.page;
        let manga = self.clone();
        let functionality = functionality.clone();
        let pages_len = self.pages_len();
        let current_chapter = self.current_page.chapter;
        spawn(async move {
            let target = if current + amount >= pages_len {
                pages_len - 1
            } else {
                current + amount
            };
            let results = stream::iter(current..=target)
                .map(|page| {
                    let manga = manga.clone();
                    let f = functionality.clone();
                    spawn(async move {
                        manga
                            .save_to_disk(
                                Page {
                                    chapter: current_chapter,
                                    page,
                                },
                                &f,
                            )
                            .await
                    })
                })
                .buffered(5)
                .collect::<Vec<_>>()
                .await;

            // reason: we only discard the `Aborted` case of error
            #[allow(clippy::manual_flatten)]
            for x in results {
                if let Ok(x) = x {
                    x.expect("manga download issue");
                }
            }
            (current, target)
        })
    }

    pub fn prefetch_chapters(&self) {
        let downloaded_chapters = self.pages.keys().copied().collect::<Vec<_>>();
        let undownloaded_chapters =
            (0..self.chapters.len()).filter(move |x| !downloaded_chapters.contains(x));
        let manga = self.clone();
        let chapters_manager = self.chapters_manager.clone();

        spawn(async move {
            println!("chapter manager has started");
            let results = stream::iter(undownloaded_chapters)
                .map(|chapter| {
                    let manga = manga.clone();
                    let chapters_manager = chapters_manager.clone();
                    spawn(async move {
                        let each_page_url = manga.download_pages_url(chapter).await?;
                        chapters_manager
                            .write()
                            .await
                            .insert(chapter, each_page_url);
                        println!("finish downloading chapter {chapter}'s page urls");
                        anyhow::Ok(())
                    })
                })
                .buffered(3)
                .collect::<Vec<_>>()
                .await;

            // reason: we only discard the `Aborted` case of error
            #[allow(clippy::manual_flatten)]
            for x in results {
                if let Ok(x) = x {
                    x.expect("manga download issue");
                }
            }
        })
        .detach();
    }

    pub async fn clear_download_managear(&self) {
        self.download_manager.write().await.clear();
    }

    pub async fn save_to_disk(
        &self,
        page: Page,
        functionality: &BackendReplier,
    ) -> anyhow::Result<()> {
        functionality
            .send_typed_message(SendMessage::InitPagesForGivenChapter(
                self.current_page.chapter + 1,
            ))
            .await?;

        if !PathBuf::from("/tmp/mangarr").exists() {
            create_dir("/tmp/mangarr/").await?;
        }
        if !PathBuf::from("/tmp/mangarr/preview").exists() {
            create_dir("/tmp/mangarr/preview").await?;
        }

        let (url, path, part) = self.get_url_with_path(page)?;

        if !self.download_manager.read().await.contains(&page) && (part.exists() || !path.exists())
        {
            File::create(&part).await?;

            self.download_manager.write().await.insert(page);

            let image = Self::save_page(url, self.api.client(), &path).await?;
            self.generate_scaled_version(&image, page).await?;

            self.download_manager.write().await.remove(&page);

            fs::remove_file(&part).await?;
        }

        if path.exists() && !part.exists() {
            functionality
                .send_typed_message(SendMessage::PageModify {
                    chapter: self.current_page.chapter + 1,
                    page: page.page,
                    path,
                })
                .await?;
        }

        Ok(())
    }
    pub async fn generate_scaled_version(
        &self,
        original_image: &PhotonImage,
        page: Page,
    ) -> anyhow::Result<()> {
        let image = resize(original_image, 405, 660, SamplingFilter::Lanczos3);

        let (_, mut path, _) = self.get_url_with_path(page)?;
        path = PathBuf::from("/tmp/mangarr/preview").join(path.file_name().context("no")?);

        if !path.exists() {
            let bytes = image.get_bytes();
            smol::fs::write(path, bytes).await?;
        }
        Ok(())
    }

    async fn download_pages_url(&self, chapter: usize) -> anyhow::Result<Arc<[ImageUrl]>> {
        let s_chapter = &self.chapters[chapter];
        let pages = self
            .api
            .fetch_pages(s_chapter)
            .await?
            .into_iter()
            .map(|x| x.image_url)
            .collect::<Arc<[_]>>();
        Ok(pages)
    }

    pub fn get_url_with_path(&self, page: Page) -> anyhow::Result<(&ImageUrl, PathBuf, PathBuf)> {
        let Some(url) = self.pages().get(page.page) else {
            bail!("out of bounds");
        };

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hashed_value = hasher.finish();

        let ps = PathBuf::from(format!("/tmp/mangarr/pic{hashed_value}.png"));
        Ok((url, ps.clone(), ps.with_extension("part")))
    }

    async fn save_page(
        url: &ImageUrl,
        client: Client,
        path: impl AsRef<Path> + Send,
    ) -> anyhow::Result<PhotonImage> {
        let path = path.as_ref();
        let mut image = match url {
            ImageUrl::Web(url) => {
                let bytes = client
                    .get(url)
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes()
                    .await?;

                open_image_from_bytes(&bytes)?
            }
            ImageUrl::LocalEpub(path_buf) => {
                let bytes = smol::fs::read(path_buf).await?;
                open_image_from_bytes(&bytes)?
            }
        };
        let is_almost_grayscale = image.get_raw_pixels().chunks_exact(3).all(|x| {
            let [r, g, b] = *x else {
                unreachable!("somehow not equal to 3");
            };
            r.abs_diff(g) < 3 && g.abs_diff(b) < 3 && r.abs_diff(b) < 3
        });

        if is_almost_grayscale {
            monochrome::grayscale(&mut image);
        } else {
            saturate_hsluv(&mut image, 0.3);
        }

        let bytes = image.get_bytes();
        smol::fs::write(path, bytes).await?;

        Ok(image)
    }
}
