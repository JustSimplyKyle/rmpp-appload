use reqwest::Client;

pub mod manhuagui;
pub mod nhentai;

#[derive(Clone, Debug)]
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

#[derive(Debug, Clone)]
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
pub trait MangaBackend: Clone {
    async fn search_by_id(&self, id: &str) -> anyhow::Result<SManga>;

    async fn fetch_chapters(&self, manga: &SManga) -> anyhow::Result<Vec<SChapter>>;

    async fn fetch_pages(&self, chapter: &SChapter) -> anyhow::Result<Vec<Page>>;

    fn client(&self) -> Client;
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MangaStatus {
    Unknown,
    Ongoing,
    Completed,
}
