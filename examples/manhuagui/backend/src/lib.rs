use std::{fmt::Display, ops::Deref, path::PathBuf, str::FromStr, sync::Arc};

use manhuagui::{Manhuagui, Preferences};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{epub::Epub, nhentai::NHentai};

pub mod epub;
pub mod manhuagui;
pub mod nhentai;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct SManga {
    pub url: ImageUrl,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub status: MangaStatus,
    pub last_updated_time: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SChapter {
    pub url: ImageUrl,
    pub name: String,
    pub chapter_number: f32,
    pub date_upload: Option<i64>,
}

#[derive(Debug)]
pub struct Page {
    pub index: usize,
    pub url: String,
    pub image_url: ImageUrl,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum ImageUrl {
    Web(String),
    LocalEpub {
        epub_path: PathBuf,
        img_path: Option<String>,
    },
}
impl ImageUrl {
    pub fn new_epub(path: PathBuf) -> Self {
        Self::LocalEpub {
            epub_path: path,
            img_path: None,
        }
    }
}

impl ImageUrl {
    pub fn get_distinguisher(&self) -> String {
        match self.clone() {
            Self::Web(x) => x,
            Self::LocalEpub {
                epub_path: path, ..
            } => {
                let id_path = path.parent().unwrap().parent().unwrap();
                id_path.to_string_lossy().to_string()
            }
        }
    }
}

impl From<String> for ImageUrl {
    fn from(v: String) -> Self {
        Self::Web(v)
    }
}

pub trait MangaBackend: std::fmt::Debug + Send + Sync + Display {
    async fn search_by_id(&self, id: &str) -> anyhow::Result<SManga>;

    async fn fetch_chapters(&self, manga: &SManga) -> anyhow::Result<Vec<SChapter>>;

    async fn fetch_pages(&self, chapter: &SChapter) -> anyhow::Result<Vec<Page>>;

    fn client(&self) -> Option<Client>;

    /// # Safety
    /// Users must ensure the implementations do NOT have the same type name.(do not override)
    fn id(&self) -> String {
        core::any::type_name::<Self>().to_string()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Backend {
    Epub(Epub),
    NHentai(NHentai),
    Manhuagui(Manhuagui),
}

impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = match self {
            Self::Epub(x) => x.to_string(),
            Self::NHentai(x) => x.to_string(),
            Self::Manhuagui(x) => x.to_string(),
        };
        write!(f, "{p}")
    }
}

impl From<Manhuagui> for Backend {
    fn from(v: Manhuagui) -> Self {
        Self::Manhuagui(v)
    }
}

impl From<NHentai> for Backend {
    fn from(v: NHentai) -> Self {
        Self::NHentai(v)
    }
}

impl From<Epub> for Backend {
    fn from(v: Epub) -> Self {
        Self::Epub(v)
    }
}

impl Backend {
    pub fn id(&self) -> String {
        match self {
            Self::Epub(x) => x.id(),
            Self::NHentai(x) => x.id(),
            Self::Manhuagui(x) => x.id(),
        }
    }
    pub fn client(&self) -> Option<Client> {
        match self {
            Self::Epub(x) => x.client(),
            Self::NHentai(x) => x.client(),
            Self::Manhuagui(x) => x.client(),
        }
    }
    pub async fn search_by_id(&self, id: &str) -> Result<SManga, anyhow::Error> {
        match self {
            Self::Epub(x) => x.search_by_id(id).await,
            Self::NHentai(x) => x.search_by_id(id).await,
            Self::Manhuagui(x) => x.search_by_id(id).await,
        }
    }
    pub async fn fetch_chapters(&self, manga: &SManga) -> Result<Vec<SChapter>, anyhow::Error> {
        match self {
            Self::Epub(x) => x.fetch_chapters(manga).await,
            Self::NHentai(x) => x.fetch_chapters(manga).await,
            Self::Manhuagui(x) => x.fetch_chapters(manga).await,
        }
    }
    pub async fn fetch_pages(&self, chapter: &SChapter) -> Result<Vec<Page>, anyhow::Error> {
        match self {
            Self::Epub(x) => x.fetch_pages(chapter).await,
            Self::NHentai(x) => x.fetch_pages(chapter).await,
            Self::Manhuagui(x) => x.fetch_pages(chapter).await,
        }
    }
}

impl Default for SManga {
    fn default() -> Self {
        Self {
            url: ImageUrl::Web(String::new()),
            title: String::new(),
            thumbnail_url: None,
            author: None,
            description: None,
            genre: None,
            status: MangaStatus::Unknown,
            last_updated_time: String::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize, Serialize)]
pub enum MangaStatus {
    Unknown,
    Ongoing,
    Completed,
}
