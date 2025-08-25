use std::fmt::Display;
use std::fs::File;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::BufReader;
use std::mem::MaybeUninit;
use std::path::PathBuf;

use anyhow::Context;
use epub::doc::EpubDoc;
use futures::FutureExt;
use reqwest::Client;
use scraper::Html;
use scraper::Selector;
// use futures::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use smol::fs::create_dir;
use smol::fs::create_dir_all;
use smol::fs::read_dir;
use smol::stream::StreamExt;

use crate::ImageUrl;
use crate::MangaBackend;
use crate::Page;
use crate::SChapter;
use crate::SManga;

#[derive(Debug, Deserialize, Serialize)]
pub struct Epub {
    base: PathBuf,
}

impl Epub {
    // base/[id]/[chapter-name]/[book-name](to id)
    pub async fn create_s_manga(&self, id: &str) -> anyhow::Result<SManga> {
        let iter = read_dir(&self.base).await?;
        let id_path = iter
            .map(|x| anyhow::Ok(x?.path()))
            .find(|x| {
                x.as_ref().is_ok_and(|x| {
                    let x = x
                        .file_name()
                        .context("no filename?")
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    x == id
                })
            })
            .await
            .context("can't find it")??;
        let first_chapter = read_dir(id_path)
            .await?
            .next()
            .await
            .context("no first chapter")??;
        let path = read_dir(first_chapter.path())
            .await?
            .next()
            .await
            .context("no first book")??
            .path();

        let epub = EpubDoc::new(&path).unwrap();
        let title = epub
            .metadata
            .get("title")
            .and_then(|x| x.first().cloned())
            .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string());
        let author = epub
            .metadata
            .get("creator")
            .and_then(|x| x.first().cloned());
        let date = epub.metadata.get("date").and_then(|x| x.first().cloned());
        Ok(SManga {
            url: ImageUrl::new_epub(path),
            title,
            thumbnail_url: None,
            author,
            description: None,
            genre: None,
            status: crate::MangaStatus::Unknown,
            last_updated_time: date.unwrap_or_default(),
        })
    }
    // base/[id]/[chapter-name]/[book-name]
    pub async fn create_s_chapters(manga: &SManga) -> anyhow::Result<Vec<SChapter>> {
        let ImageUrl::LocalEpub {
            epub_path: path, ..
        } = &manga.url
        else {
            unreachable!()
        };

        let id_path = path.parent().unwrap().parent().unwrap();
        let chapter_number = std::fs::read_dir(id_path)?.count();
        let iter = read_dir(id_path).await?;
        let p = iter
            .then(|x| async move {
                let chapter_name = x?.path();
                let path = read_dir(chapter_name.clone())
                    .await?
                    .next()
                    .await
                    .context("no first book")??
                    .path();
                anyhow::Ok(SChapter {
                    name: chapter_name
                        .file_name()
                        .context("plz don't be empty")?
                        .to_string_lossy()
                        .to_string(),
                    url: ImageUrl::new_epub(path),
                    chapter_number: chapter_number as f32,
                    date_upload: None,
                })
            })
            .collect::<Vec<_>>()
            .await;
        let p = p.into_iter().collect::<anyhow::Result<Vec<_>>>()?;
        Ok(p)
    }
    pub async fn fetch_pages(base: PathBuf, chapter: &SChapter) -> anyhow::Result<Vec<Page>> {
        let ImageUrl::LocalEpub {
            epub_path: path, ..
        } = &chapter.url
        else {
            unreachable!()
        };

        let mut epub = EpubDoc::new(path).unwrap();
        let mut v = vec![];
        for i in 0..epub.get_num_pages() {
            let id = epub.spine.get(i).cloned().map(|i| i.idref).unwrap();
            let (bytes, _) = epub.get_resource(&id).unwrap();

            let html = String::from_utf8(bytes).unwrap();
            let img_path = {
                let html = Html::parse_document(&html);
                html.select(&Selector::parse("img").unwrap())
                    .next()
                    .unwrap()
                    .attr("src")
                    .unwrap()
                    .to_owned()
            };
            let img_path = img_path.strip_prefix("../").unwrap_or(&img_path);
            v.push(Page {
                index: i,
                url: String::new(),
                image_url: ImageUrl::LocalEpub {
                    epub_path: path.to_path_buf(),
                    img_path: Some(img_path.to_string()),
                },
            });
        }
        dbg!("finished page fetching for epub");
        Ok(v)
    }
    pub fn fetch_img(&self, path: PathBuf, epub_img_path: &str) -> anyhow::Result<Vec<u8>> {
        let mut epub = EpubDoc::new(path).unwrap();
        epub.get_resource_by_path(epub_img_path)
            .context("resource does not exist")
    }
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { base: path }
    }
}

impl Display for Epub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Epub")
    }
}

impl Default for Epub {
    fn default() -> Self {
        Self::new(
            std::env::home_dir()
                .context("no home dir")
                .unwrap()
                .join("mangarr-books"),
        )
    }
}

impl MangaBackend for Epub {
    async fn search_by_id(&self, id: &str) -> anyhow::Result<SManga> {
        self.create_s_manga(id).await
    }

    async fn fetch_chapters(&self, manga: &SManga) -> anyhow::Result<Vec<SChapter>> {
        let mut p = Self::create_s_chapters(manga).await?;
        p.sort_by_key(|x| x.name.clone());
        Ok(p)
    }

    async fn fetch_pages(&self, chapter: &SChapter) -> anyhow::Result<Vec<Page>> {
        Self::fetch_pages(self.base.clone(), chapter).await
    }

    fn client(&self) -> std::option::Option<reqwest::Client> {
        Client::default().into()
    }
}
