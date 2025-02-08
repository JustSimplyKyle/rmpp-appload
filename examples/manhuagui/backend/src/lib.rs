use std::{fmt::Display, str::FromStr, sync::Arc};

use manhuagui::{Manhuagui, Preferences};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub mod manhuagui;
pub mod nhentai;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct SManga {
    pub url: String,
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
    pub url: String,
    pub name: String,
    pub chapter_number: f32,
    pub date_upload: Option<i64>,
}

#[derive(Debug)]
pub struct Page {
    pub index: usize,
    pub url: String,
    pub image_url: String,
}

#[async_trait::async_trait]
#[typetag::serde(tag = "type")]
pub trait MangaBackend: std::fmt::Debug + Send + Sync + Display {
    async fn search_by_id(&self, id: &str) -> anyhow::Result<SManga>;

    async fn fetch_chapters(&self, manga: &SManga) -> anyhow::Result<Vec<SChapter>>;

    async fn fetch_pages(&self, chapter: &SChapter) -> anyhow::Result<Vec<Page>>;

    fn client(&self) -> Client;

    /// # Safety
    /// Users must ensure the implementations do NOT have the same type name.(do not override)
    fn id(&self) -> String {
        core::any::type_name::<Self>().to_string()
    }
}

impl Default for SManga {
    fn default() -> Self {
        Self {
            url: String::new(),
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
