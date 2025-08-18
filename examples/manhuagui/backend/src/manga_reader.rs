use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, bail};
use backend::{
    SChapter, SManga,
    manhuagui::{Manhuagui, Preferences},
};
use futures::{StreamExt, TryStreamExt, stream};
use image::{
    DynamicImage, GenericImageView, ImageReader, Rgba, codecs::png::PngEncoder,
    imageops::FilterType,
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
    pub details: SManga,
    pub chapters: Vec<SChapter>,
    pub pages: HashMap<usize, Vec<String>>,
    pub current_page: Page,
    #[serde(skip)]
    download_manager: Arc<RwLock<DownloadManager>>,
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
        params: impl Into<Option<(SManga, Vec<SChapter>, Vec<String>, Page)>>,
    ) -> anyhow::Result<Self> {
        let api = match api.into() {
            Some(x) => x,
            None => Arc::new(Manhuagui::new(Preferences::default())?),
        };
        let manga_reader = if let Some((details, chapters, pages, active)) = params.into() {
            Self {
                api,
                details,
                chapters,
                pages: HashMap::from([(0, pages)]),
                current_page: active,
                download_manager: RwLock::new(DownloadManager::default()).into(),
            }
        } else {
            Self {
                api,
                details: Default::default(),
                chapters: Default::default(),
                pages: Default::default(),
                current_page: Default::default(),
                download_manager: RwLock::new(DownloadManager::default()).into(),
            }
        };
        Ok(manga_reader)
    }
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
        functionality
            .send_typed_message(SendMessage::ActivePageNumber(self.current_page.page + 1))?;

        functionality.send_typed_message(SendMessage::TotalPageSize(self.pages().len()))?;
        functionality.send_typed_message(SendMessage::ActiveChapterNumber(
            self.current_page.chapter + 1,
        ))?;
        functionality.send_typed_message(SendMessage::TotalChapterSize(self.chapters.len()))?;
        Ok(())
    }
    pub fn pages(&self) -> &[String] {
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
        if !self.pages.contains_key(&self.current_page.chapter) {
            let pages = self.download_pages_url(self.current_page.chapter).await?;
            self.pages.insert(self.current_page.chapter, pages);
        }
        Ok(())
    }

    async fn download_pages_url(&self, chapter: usize) -> Result<Vec<String>, anyhow::Error> {
        let s_chapter = &self.chapters[chapter];
        let pages = self
            .api
            .fetch_pages(s_chapter)
            .await?
            .into_iter()
            .map(|x| x.image_url)
            .collect::<Vec<_>>();
        Ok(pages)
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

            for x in results {
                x.unwrap();
            }
            (current, target)
        })
    }
    pub async fn clear_download_managear(&self) {
        self.download_manager.write().await.clear();
    }

    pub async fn save_to_disk(
        &self,
        page: Page,
        functionality: &BackendReplier,
    ) -> anyhow::Result<()> {
        functionality.send_typed_message(SendMessage::InitPagesForGivenChapter(
            self.current_page.chapter + 1,
        ))?;

        if !PathBuf::from("/tmp/mangarr").exists() {
            create_dir("/tmp/mangarr/").await?;
        }
        if !PathBuf::from("/tmp/mangarr/preview").exists() {
            create_dir("/tmp/mangarr/preview").await?;
        }

        let (url, path, part) = self.get_url_with_path(page)?;

        if !self.download_manager.read().await.contains(&page) && (part.exists() || !path.exists())
        {
            println!("attempt downloading page {page:#?}");
            File::create(&part).await?;

            self.download_manager.write().await.insert(page);

            let image = Self::save_page(url, self.api.client(), &path).await?;
            self.generate_scaled_version(image, page).await?;

            self.download_manager.write().await.remove(&page);

            fs::remove_file(&part).await?;
        }

        if path.exists() && !part.exists() {
            functionality.send_typed_message(SendMessage::PageModify {
                chapter: self.current_page.chapter + 1,
                page: page.page,
                path,
            })?;
        }

        Ok(())
    }
    pub async fn generate_scaled_version(
        &self,
        mut original_image: DynamicImage,
        page: Page,
    ) -> anyhow::Result<()> {
        original_image = original_image.resize(405, 660, FilterType::Lanczos3);
        let (_, mut ps, _) = self.get_url_with_path(page)?;
        ps = PathBuf::from("/tmp/mangarr/preview").join(ps.file_name().context("no")?);
        if !ps.exists() {
            let mut buf = vec![];

            let encoder = PngEncoder::new(&mut buf);

            original_image.write_with_encoder(encoder)?;

            fs::write(ps, buf).await?;
        }
        Ok(())
    }

    pub fn get_url_with_path(&self, page: Page) -> anyhow::Result<(&str, PathBuf, PathBuf)> {
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
        url: &str,
        client: Client,
        path: impl AsRef<Path> + Send,
    ) -> anyhow::Result<DynamicImage> {
        let path = path.as_ref();
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

        let mut file = fs::File::create(path).await?;

        file.write_all(&non_grayscale).await?;

        file.flush().await?;
        Ok(image)
    }
}
