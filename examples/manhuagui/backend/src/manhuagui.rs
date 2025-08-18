use std::fmt::Display;

/// This code is based on the Manhuagui extension for Tachiyomi.
/// Source: <https://github.com/keiyoushi/extensions-source/blob/main/src/zh/manhuagui/src/eu/kanade/tachiyomi/extension/zh/manhuagui/Manhuagui.ktb>
use anyhow::{Context, bail};
use regex::Regex;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue, REFERER, USER_AGENT},
};
use scraper::Html;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{MangaBackend, MangaStatus, Page, SChapter, SManga};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manhuagui {
    pub name: String,
    pub lang: String,
    base_url: String,
    image_server: [String; 2],

    #[serde(skip, default = "deserialize_client")]
    client: Client,
}

fn deserialize_client() -> Client {
    Manhuagui::build_client(Preferences::default()).unwrap()
}

impl Display for Manhuagui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Manhuagui")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Preferences {
    pub show_r18: bool,
    pub show_zh_hant_website: bool,
    pub use_mirror_url: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            show_r18: true,
            show_zh_hant_website: true,
            use_mirror_url: false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Comic {
    bname: String,
    bpic: String,
    files: Option<Vec<String>>,
    path: String,
    len: u32,
    block_cc: String,
    sl: Option<Sl>,
}

#[derive(Debug, Deserialize)]
struct Sl {
    e: u32,
    m: String,
}

#[async_trait::async_trait]
#[typetag::serde]
impl MangaBackend for Manhuagui {
    fn client(&self) -> Client {
        self.client.clone()
    }
    async fn search_by_id(&self, id: &str) -> anyhow::Result<SManga> {
        let url = format!("{}/comic/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let manga = Self::smanga_creation(&document, format!("/comic/{id}/"));
        Ok(manga)
    }

    async fn fetch_chapters(&self, manga: &SManga) -> anyhow::Result<Vec<SChapter>> {
        let manga_url = self.chapter_url(manga);
        let response = self.client.get(manga_url).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let mut chapters = Vec::new();

        let viewstate_selector = Selector::parse("#__VIEWSTATE");
        let erroraudit_show_selector = Selector::parse("#erroraudit_show");
        if let Some(hidden_encrypted_chapter_list) = document.select(&viewstate_selector).next() {
            if let Some(val) = hidden_encrypted_chapter_list.value().attr("value") {
                let js_decode_func = Self::JS_DECODE_FUNC;
                let decoded_hidden_chapter_list = quick_js::Context::new()?
                    .eval(&format!(
                        "{js_decode_func}LZString.decompressFromBase64('{val}');",
                    ))?
                    .as_str()
                    .context("not string")?
                    .to_owned();

                let hidden_chapter_list = Html::parse_fragment(&decoded_hidden_chapter_list);

                if let Some(error_audit_show) = document.select(&erroraudit_show_selector).next() {
                    // document = document.replacen(error_audit_show, &hidden_chapter_list, 1);
                    // TODO: Implement a proper way to replace nodes in the document
                    println!("R18 content detected, but node replacement is not implemented yet.");
                }
            }
        }

        let latest_chapter_selector =
            Selector::parse("div.book-detail > ul.detail-list > li.status > span > a.blue");
        let latest_chapter_href = document
            .select(&latest_chapter_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(String::from);

        let ch_num_regex = Regex::new(r"\d+").unwrap();

        let section_list_selector = Selector::parse("[id^=chapter-list-]");
        for section in document.select(&section_list_selector) {
            let page_list_selector = Selector::parse("ul");
            let page_list = section.select(&page_list_selector).collect::<Vec<_>>();

            for page in page_list.iter().rev() {
                let chapter_list_selector = Selector::parse("li > a.status0");
                for chapter_link in page.select(&chapter_list_selector) {
                    let url = chapter_link.value().attr("href").unwrap().to_string();
                    let name = chapter_link.value().attr("title").map_or_else(
                        || {
                            chapter_link
                                .select(&Selector::parse("span"))
                                .next()
                                .unwrap()
                                .text()
                                .next()
                                .unwrap()
                                .trim()
                                .to_string()
                        },
                        |title| title.trim().to_owned(),
                    );

                    let chapter_number = ch_num_regex
                        .find(&name)
                        .and_then(|m| m.as_str().parse::<f32>().ok())
                        .unwrap_or(-1.0);

                    // mutable because post date_upload usage
                    let mut current_chapter = SChapter {
                        url,
                        name,
                        chapter_number,
                        date_upload: None,
                    };

                    if let Some(ref latest_href) = latest_chapter_href
                        && current_chapter.url == *latest_href
                    {
                        let date_selector = Selector::parse(
                            "div.book-detail > ul.detail-list > li.status > span > span.red",
                        );
                        if let Some(date_element) = document.select(&date_selector).next_back() {
                            current_chapter.date_upload =
                                Some(Self::parse_date(date_element.text().next().unwrap()));
                        }
                    }
                    chapters.push(current_chapter);
                }
            }
        }

        Ok(chapters)
    }

    async fn fetch_pages(&self, chapter: &SChapter) -> anyhow::Result<Vec<Page>> {
        let manga_url = self.page_url(chapter);
        let response = self.client.get(manga_url).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let erroraudit_show_selector = Selector::parse("#erroraudit_show");
        if document.select(&erroraudit_show_selector).next().is_some() {
            bail!("R18作品显示开关未开启或未生效");
        }

        let re = Regex::new(r#"window\[".*?"\](\(.*\)\s*\{[\s\S]+\}\s*\(.*\))"#).unwrap();
        let re2 = Regex::new(r"\{.*\}").unwrap();

        let js_decode_func = Self::JS_DECODE_FUNC;

        let img_code = re
            .captures(&body)
            .and_then(|cap| cap.get(1))
            .map(|mat| mat.as_str())
            .context("Failed to find image code")?;

        let img_decode = quick_js::Context::new()?
            .eval(&format!("{js_decode_func}{img_code}"))?
            .as_str()
            .context("not string")?
            .to_owned();

        let img_json_str = re2
            .captures(&img_decode)
            .and_then(|cap| cap.get(0))
            .map(|mat| mat.as_str())
            .context("Failed to find image JSON string")?;

        let image_json: Comic = serde_json::from_str(img_json_str)?;

        let pages = image_json
            .files
            .unwrap_or_default()
            .iter()
            .enumerate()
            .map(|(i, img_str)| {
                let imgurl = format!(
                    "{}{}{}?e={}&m={}",
                    self.image_server[0],
                    image_json.path,
                    img_str,
                    image_json.sl.as_ref().map_or(0, |sl| sl.e),
                    image_json.sl.as_ref().map_or("", |sl| sl.m.as_str())
                );
                Page {
                    index: i,
                    url: String::new(),
                    image_url: imgurl,
                }
            })
            .collect();

        Ok(pages)
    }
}

// should only be used for craetion of constly parsed values
struct Selector;

impl Selector {
    #[must_use]
    pub fn parse(s: &str) -> scraper::Selector {
        scraper::Selector::parse(s)
            .map_err(|x| anyhow::anyhow!("{x:#?}"))
            .expect("invalid input")
    }
}
impl Manhuagui {
    const JS_DECODE_FUNC: &'static str = r#"
        var LZString=(function(){var f=String.fromCharCode;var keyStrBase64="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";var baseReverseDic={};function getBaseValue(alphabet,character){if(!baseReverseDic[alphabet]){baseReverseDic[alphabet]={};for(var i=0;i<alphabet.length;i++){baseReverseDic[alphabet][alphabet.charAt(i)]=i}}return baseReverseDic[alphabet][character]}var LZString={decompressFromBase64:function(input){if(input==null)return"";if(input=="")return null;return LZString._0(input.length,32,function(index){return getBaseValue(keyStrBase64,input.charAt(index))})},_0:function(length,resetValue,getNextValue){var dictionary=[],next,enlargeIn=4,dictSize=4,numBits=3,entry="",result=[],i,w,bits,resb,maxpower,power,c,data={val:getNextValue(0),position:resetValue,index:1};for(i=0;i<3;i+=1){dictionary[i]=i}bits=0;maxpower=Math.pow(2,2);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}switch(next=bits){case 0:bits=0;maxpower=Math.pow(2,8);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}c=f(bits);break;case 1:bits=0;maxpower=Math.pow(2,16);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}c=f(bits);break;case 2:return""}dictionary[3]=c;w=c;result.push(c);while(true){if(data.index>length){return""}bits=0;maxpower=Math.pow(2,numBits);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}switch(c=bits){case 0:bits=0;maxpower=Math.pow(2,8);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}dictionary[dictSize++]=f(bits);c=dictSize-1;enlargeIn--;break;case 1:bits=0;maxpower=Math.pow(2,16);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}dictionary[dictSize++]=f(bits);c=dictSize-1;enlargeIn--;break;case 2:return result.join('')}if(enlargeIn==0){enlargeIn=Math.pow(2,numBits);numBits++}if(dictionary[c]){entry=dictionary[c]}else{if(c===dictSize){entry=w+w.charAt(0)}else{return null}}result.push(entry);dictionary[dictSize++]=w+entry.charAt(0);enlargeIn--;w=entry;if(enlargeIn==0){enlargeIn=Math.pow(2,numBits);numBits++}}}};return LZString})();String.prototype.splic=function(f){return LZString.decompressFromBase64(this).split(f)};
        "#;
    pub fn new(preferences: Preferences) -> anyhow::Result<Self> {
        let base_host = if preferences.use_mirror_url {
            "mhgui.com"
        } else {
            "manhuagui.com"
        }
        .to_owned();

        let base_url = if preferences.show_zh_hant_website {
            format!("https://tw.{base_host}")
        } else {
            format!("https://www.{base_host}")
        };

        let image_server = [
            "https://i.hamreus.com".to_string(),
            "https://cf.hamreus.com".to_string(),
        ];

        let client = Self::build_client(preferences)?;

        Ok(Self {
            name: String::from("漫画柜"),
            lang: String::from("zh"),
            base_url,
            image_server,
            client,
        })
    }

    fn build_client(preferences: Preferences) -> anyhow::Result<Client> {
        let base_host = if preferences.use_mirror_url {
            "mhgui.com"
        } else {
            "manhuagui.com"
        }
        .to_owned();

        let base_url = if preferences.show_zh_hant_website {
            format!("https://tw.{base_host}")
        } else {
            format!("https://www.{base_host}")
        };
        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_str(&base_url)?);
        headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/84.0.4147.105 Safari/537.36",
        ),
    );
        if preferences.show_r18 {
            headers.insert("Cookie", HeaderValue::from_static("isAdult=1"));
        }

        let client = Client::builder().default_headers(headers).build()?;
        Ok(client)
    }

    pub fn smanga_creation(document: &Html, url: impl Into<String>) -> SManga {
        let title_selector = Selector::parse("div.book-title > h1:nth-child(1)");
        let description_selector = Selector::parse("div#intro-all");
        let thumbnail_selector = Selector::parse("p.hcover > img");
        let author_selector = Selector::parse("span > strong");
        let genre_selector = Selector::parse("span > strong");
        let date_selector = Selector::parse("span > strong");
        let status_selector =
            Selector::parse("div.book-detail > ul.detail-list > li.status > span > span");

        let title = document
            .select(&title_selector)
            .next()
            .map_or(String::new(), |el| {
                el.text().collect::<String>().trim().to_owned()
            });
        let description = document
            .select(&description_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned());
        let thumbnail_url = document
            .select(&thumbnail_selector)
            .next()
            .and_then(|el| el.value().attr("src").map(String::from));

        let span_matcher =
            |selector, filterer: fn(String) -> bool, element_filterer: fn(&str) -> bool| {
                document
                    .select(&selector)
                    .filter(|x| {
                        let x = x.text().collect::<String>();
                        filterer(x)
                    })
                    .filter_map(|strong| {
                        Some(
                            strong
                                .parent()?
                                .children()
                                .filter(move |x| x.id() != strong.id()),
                        )
                    })
                    .flatten()
                    .filter_map(|x| x.value().as_element().map(|y| (x.id(), y)))
                    .filter(|(_, x)| element_filterer(x.name()))
                    .filter_map(|(id, _)| document.tree.get(id)?.first_child()?.value().as_text())
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
            };

        let author = Some(
            span_matcher(
                author_selector,
                |x| x.contains("漫画作者") || x.contains("漫畫作者"),
                |x| x == "a",
            )
            .join(", ")
            .trim()
            .to_owned(),
        );
        let genre = Some(
            span_matcher(
                genre_selector,
                |x| x.contains("漫画剧情") || x.contains("漫畫劇情"),
                |x| x == "a",
            )
            .join(", ")
            .trim()
            .to_owned(),
        );

        let last_updated_time = span_matcher(
            date_selector,
            |x| x.contains("漫畫狀態") || x.contains("漫画状态"),
            |x| x == "span",
        )
        .get(1)
        .cloned()
        .unwrap_or_default();

        let status = match document
            .select(&status_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .as_deref()
        {
            Some("连载中" | "連載中") => MangaStatus::Ongoing,
            Some("已完结" | "已完結") => MangaStatus::Completed,
            _ => MangaStatus::Unknown,
        };
        SManga {
            url: url.into(),
            title,
            thumbnail_url,
            author,
            description,
            genre,
            status,
            last_updated_time,
        }
    }
    #[must_use]
    pub fn chapter_url(&self, api: &SManga) -> String {
        format!("{}{}", self.base_url, api.url)
    }

    #[must_use]
    pub fn page_url(&self, api: &SChapter) -> String {
        format!("{}{}", self.base_url, api.url)
    }

    fn parse_date(date_str: &str) -> i64 {
        use chrono::NaiveDate;

        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let datetime = date.and_hms_opt(0, 0, 0).unwrap();
            let timestamp = {
                let this = &datetime;
                this.and_utc().timestamp()
            };
            if timestamp >= 0 {
                return timestamp * 1000;
            }
        }
        0
    }
}
