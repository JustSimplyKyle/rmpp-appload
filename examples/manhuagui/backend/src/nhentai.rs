/// Based on <https://github.com/keiyoushi/extensions-source/blob/main/src/all/nhentai/src/eu/kanade/tachiyomi/extension/all/nhentai/NHentai.kt>
use crate::{MangaBackend, MangaStatus, Page, SChapter, SManga};
use anyhow::{bail, Context, Result};
use regex::Regex;
use reqwest::{
    cookie::Jar,
    header::{HeaderMap, HeaderValue, REFERER, USER_AGENT},
    Client, Url,
};
use scraper::{Html, Selector};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{env, fmt::Display, sync::Arc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NHentai {
    pub lang: String,
    nh_lang: String,
    base_url: String,

    #[serde(skip, default = "deserialize_client")]
    client: Client,
    display_full_title: bool,
}

fn deserialize_client() -> Client {
    NHentai::build_client().unwrap()
}

impl Display for NHentai {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NHentai")
    }
}

// Structures to represent the JSON data from nHentai
#[derive(Debug, Deserialize)]
struct NHentaiTitle {
    english: Option<String>,
    japanese: Option<String>,
    pretty: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NHentaiImage {
    t: String, // Type: "j" (jpg), "p" (png), "g" (gif), "w"(webp)
    w: u32,    // Width
    h: u32,    // Height
}

#[derive(Debug, Deserialize)]
struct NHentaiImages {
    pages: Vec<NHentaiImage>,
    cover: Option<NHentaiImage>,
    thumbnail: Option<NHentaiImage>,
}
#[derive(Debug, Deserialize)]
struct NHentaiTag {
    id: u64,
    r#type: String,
    name: String,
    url: String,
    count: u64,
}

#[derive(Debug, Deserialize)]
struct NHentaiData {
    id: u64,
    media_id: String,
    title: NHentaiTitle,
    images: NHentaiImages,
    scanlator: Option<String>,
    upload_date: i64,
    tags: Vec<NHentaiTag>,
    num_pages: u32,
    num_favorites: u32,
}

impl NHentaiData {
    fn get_artists(&self) -> Option<String> {
        let artists = self
            .tags
            .iter()
            .filter(|t| t.r#type == "artist")
            .map(|a| a.name.clone())
            .collect::<Vec<String>>();
        if artists.is_empty() {
            None
        } else {
            Some(artists.join(", "))
        }
    }
    fn get_groups(&self) -> Option<String> {
        let groups = self
            .tags
            .iter()
            .filter(|t| t.r#type == "group")
            .map(|g| g.name.clone())
            .collect::<Vec<String>>();

        if groups.is_empty() {
            None
        } else {
            Some(groups.join(", "))
        }
    }
    fn get_tags_desc(&self) -> String {
        self.tags.iter().fold(String::new(), |acc, tag| {
            acc + &format!("{}: {}\n", tag.r#type, tag.name)
        })
    }

    fn get_tags(&self) -> String {
        self.tags
            .iter()
            .filter(|t| t.r#type == "tag")
            .map(|t| t.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl NHentai {
    pub fn new(lang: String, nh_lang: String, display_full_title: bool) -> Result<Self> {
        let base_url = "https://nhentai.net".to_string();
        let client = Self::build_client()?;
        Ok(Self {
            lang,
            nh_lang,
            base_url,
            client,
            display_full_title,
        })
    }

    fn build_client() -> anyhow::Result<Client> {
        let base_url = "https://nhentai.net".to_string();
        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_str(&base_url)?);

        let jar = Jar::default();

        #[allow(deprecated)]
        let home = env::home_dir().context("not present")?;

        let cf_clearance = std::fs::read_to_string(home.join(".config/mangarr/cf_token"))?;

        jar.add_cookie_str(
            &format!("cf_clearance={cf_clearance}"),
            &Url::parse(&base_url)?,
        );

        let user_agent_path = home.join(".config/mangarr/user_agent");

        let user_agent = if user_agent_path.exists() {
            let src = std::fs::read_to_string(user_agent_path)?;
            HeaderValue::from_str(&src)?
        } else {
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0",
            )
        };

        headers.insert(USER_AGENT, user_agent);

        let client = Client::builder()
            .default_headers(headers)
            .cookie_provider(Arc::new(jar))
            .build()?;

        Ok(client)
    }

    fn shorten_title(title: &str) -> String {
        let re = Regex::new(r"(\[[^]]*]|[({][^)}]*[)}])").unwrap();
        re.replace_all(title, "").trim().to_string()
    }

    // Helper function to extract and parse the JSON data from the script tag.
    fn extract_hentai_data(document: &Html) -> Result<NHentaiData> {
        let data_regex = Regex::new(r#"JSON\.parse\(\s*"(.*)"\s*\)"#).unwrap();
        let hentai_selector = Selector::parse("script").unwrap();

        let script = document
            .select(&hentai_selector)
            .map(|x| x.inner_html())
            .find(|x| x.contains("JSON.parse") && !x.contains("media_server"))
            .context("Could not find the script tag containing JSON data")?;
        let json_str = data_regex
            .captures(&script)
            .and_then(|cap| cap.get(1))
            .map(|mat| mat.as_str())
            .context("Could not extract JSON string from script tag")?;

        let unescaped_json = unescape_json(json_str)?;

        let data: NHentaiData =
            serde_json::from_str(&unescaped_json).context("Failed to deserialize JSON data")?;

        Ok(data)
    }
}

#[async_trait::async_trait]
#[typetag::serde]
impl MangaBackend for NHentai {
    fn client(&self) -> Client {
        self.client.clone()
    }
    async fn search_by_id(&self, id: &str) -> Result<SManga> {
        let search_id = id.to_owned();

        let url = format!("{}/g/{}", self.base_url, search_id);
        let client = self.client.clone();
        let display_full_title = self.display_full_title;

        let response = client.get(&url).send().await?.error_for_status()?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let data = Self::extract_hentai_data(&document)?;

        let title = if display_full_title {
            data.title.english.clone().unwrap_or_else(|| {
                data.title
                    .japanese
                    .clone()
                    .unwrap_or_else(|| data.title.pretty.clone().unwrap_or_default())
            })
        } else {
            data.title.pretty.clone().unwrap_or_else(|| {
                let eng_or_jap = data
                    .title
                    .english
                    .clone()
                    .unwrap_or_else(|| data.title.japanese.clone().unwrap_or_default());
                Self::shorten_title(&eng_or_jap)
            })
        };

        let thumbnail_selector = Selector::parse("#cover > a > img").unwrap();
        let thumbnail_url = document
            .select(&thumbnail_selector)
            .next()
            .and_then(|x| x.value().attr("data-src"))
            .map(String::from);

        let author = data.get_groups().or_else(|| data.get_artists());
        let description = format!(
            "Full English and Japanese titles:\n{}\n{}\n\nPages: {}\nFavorited by: {}\n{}",
            data.title.english.clone().unwrap_or_else(|| data
                .title
                .japanese
                .clone()
                .unwrap_or_default()),
            data.title.japanese.clone().unwrap_or_default(),
            data.images.pages.len(),
            data.num_favorites,
            data.get_tags_desc()
        );
        let genre = data.get_tags();

        Ok(SManga {
            url: format!("/g/{search_id}/"), // Relative URL
            title,
            thumbnail_url,
            status: MangaStatus::Completed,
            author,
            description: Some(description),
            genre: Some(genre),
            last_updated_time: String::new(),
        })
    }

    async fn fetch_chapters(&self, manga: &SManga) -> Result<Vec<SChapter>> {
        // Extract manga ID from the URL.  We assume the URL is in the format `/g/<id>/`.
        let manga_id = manga.url.trim_matches('/').split('/').nth(1);
        let client = self.client.clone();

        if let Some(manga_id) = manga_id {
            let url = format!("{}/g/{}", self.base_url, manga_id);
            let response = client.get(&url).send().await?.error_for_status()?;
            let body = response.text().await?;
            let document = Html::parse_document(&body);

            let data = Self::extract_hentai_data(&document)?;

            let date_upload = if data.upload_date > 0 {
                Some(data.upload_date * 1000)
            } else {
                None
            };
            Ok(vec![SChapter {
                url: manga.url.clone(),
                name: "Chapter".to_string(),
                chapter_number: 1.0, // NHentai typically has only one chapter
                date_upload,
            }])
        } else {
            bail!("Invalid manga URL format: {}", manga.url);
        }
    }

    async fn fetch_pages(&self, chapter: &SChapter) -> Result<Vec<Page>> {
        let chapter_id = chapter.url.trim_matches('/').split('/').nth(1);
        let client = self.client.clone();

        if let Some(chapter_id) = chapter_id {
            let url = format!("{}/g/{}", self.base_url, chapter_id);
            let response = client.get(&url).send().await?.error_for_status()?;

            let body = response.text().await?;
            let document = Html::parse_document(&body);

            let script_selector = Selector::parse("script").unwrap();
            let script = document
                .select(&script_selector)
                .map(|x| x.inner_html())
                .find(|x| x.contains("media_server"))
                .context("media server script not found")?;
            let re = Regex::new(r#"media_server\s*:\s*(\d+)"#).unwrap();
            let media_server = re
                .captures(&script)
                .and_then(|cap| cap.get(1))
                .map(|mat| mat.as_str())
                .context("media server number failed")?
                .to_string();

            let data = Self::extract_hentai_data(&document)?;

            let pages = data
                .images
                .pages
                .iter()
                .enumerate()
                .map(|(i, img)| {
                    let img_url = format!(
                        "https://i{}.nhentai.net/galleries/{}/{}{}",
                        media_server,
                        data.media_id,
                        i + 1,
                        match img.t.as_str() {
                            "p" => ".png",
                            "g" => ".gif",
                            "w" => ".webp",
                            _ => ".jpg", // Default to .jpg
                        }
                    );
                    Page {
                        index: i,
                        url: String::new(), // nhentai does not require this
                        image_url: img_url,
                    }
                })
                .collect();

            Ok(pages)
        } else {
            bail!("Invalid chapter URL format: {}", chapter.url);
        }
    }
}

// hack from https://stackoverflow.com/questions/76361360/how-do-i-unescaped-string-that-has-been-escaped-multiple-times-in-rust
fn unescape_json(dirty_string: &str) -> Result<String, serde_json::Error> {
    serde_json::from_str(&format!(r#""{dirty_string}""#))
}
