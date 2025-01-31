use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue, REFERER, USER_AGENT},
    Client, Response,
};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const PREFIX_ID_SEARCH: &str = "id:";

#[derive(Debug, Clone)]
pub struct Manhuagui {
    pub name: String,
    pub lang: String,
    base_url: String,
    image_server: [&'static str; 2],
    pub client: Client,
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

#[derive(Debug)]
pub struct SManga {
    pub url: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub status: MangaStatus,
    initialized: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MangaStatus {
    Unknown,
    Ongoing,
    Completed,
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

#[derive(Debug)]
pub struct MangasPage {
    pub mangas: Vec<SManga>,
    pub has_next_page: bool,
}

#[derive(Debug)]
pub enum Filter<'a> {
    Sort(SortFilter<'a>),
    Locale(LocaleFilter<'a>),
    Genre(GenreFilter<'a>),
    Reader(ReaderFilter<'a>),
    PublishDate(PublishDateFilter<'a>),
    FirstLetter(FirstLetterFilter<'a>),
    Status(StatusFilter<'a>),
}
pub type FilterList<'a> = Vec<Filter<'a>>;

#[derive(Debug)]
pub struct SortFilter<'a> {
    state: usize,
    pair: &'a [(&'a str, &'a str)],
}

impl SortFilter<'_> {
    const fn new() -> Self {
        SortFilter {
            state: 0,
            pair: &[
                ("人气最旺", "view"),
                ("最新发布", ""),
                ("最新更新", "update"),
                ("评分最高", "rate"),
            ],
        }
    }
    const fn to_uri_part(&self) -> &str {
        self.pair[self.state].1
    }
}

#[derive(Debug)]
pub struct LocaleFilter<'a> {
    state: usize,
    pair: &'a [(&'a str, &'a str)],
}

impl LocaleFilter<'_> {
    const fn new() -> Self {
        LocaleFilter {
            state: 0,
            pair: &[
                ("全部", ""),
                ("日本", "japan"),
                ("港台", "hongkong"),
                ("其它", "other"),
                ("欧美", "europe"),
                ("内地", "china"),
                ("韩国", "korea"),
            ],
        }
    }
    const fn to_uri_part(&self) -> &str {
        self.pair[self.state].1
    }
}
#[derive(Debug)]
pub struct GenreFilter<'a> {
    state: usize,
    pair: &'a [(&'a str, &'a str)],
}

impl GenreFilter<'_> {
    const fn new() -> Self {
        GenreFilter {
            state: 0,
            pair: &[
                ("全部", ""),
                ("热血", "rexue"),
                ("冒险", "maoxian"),
                ("魔幻", "mohuan"),
                ("神鬼", "shengui"),
                ("搞笑", "gaoxiao"),
                ("萌系", "mengxi"),
                ("爱情", "aiqing"),
                ("科幻", "kehuan"),
                ("魔法", "mofa"),
                ("格斗", "gedou"),
                ("武侠", "wuxia"),
                ("机战", "jizhan"),
                ("战争", "zhanzheng"),
                ("竞技", "jingji"),
                ("体育", "tiyu"),
                ("校园", "xiaoyuan"),
                ("生活", "shenghuo"),
                ("励志", "lizhi"),
                ("历史", "lishi"),
                ("伪娘", "weiniang"),
                ("宅男", "zhainan"),
                ("腐女", "funv"),
                ("耽美", "danmei"),
                ("百合", "baihe"),
                ("后宫", "hougong"),
                ("治愈", "zhiyu"),
                ("美食", "meishi"),
                ("推理", "tuili"),
                ("悬疑", "xuanyi"),
                ("恐怖", "kongbu"),
                ("四格", "sige"),
                ("职场", "zhichang"),
                ("侦探", "zhentan"),
                ("社会", "shehui"),
                ("音乐", "yinyue"),
                ("舞蹈", "wudao"),
                ("杂志", "zazhi"),
                ("黑道", "heidao"),
            ],
        }
    }
    const fn to_uri_part(&self) -> &str {
        self.pair[self.state].1
    }
}

#[derive(Debug)]
pub struct ReaderFilter<'a> {
    state: usize,
    pair: &'a [(&'a str, &'a str)],
}

impl ReaderFilter<'_> {
    const fn new() -> Self {
        ReaderFilter {
            state: 0,
            pair: &[
                ("全部", ""),
                ("少女", "shaonv"),
                ("少年", "shaonian"),
                ("青年", "qingnian"),
                ("儿童", "ertong"),
                ("通用", "tongyong"),
            ],
        }
    }
    const fn to_uri_part(&self) -> &str {
        self.pair[self.state].1
    }
}
#[derive(Debug)]
pub struct PublishDateFilter<'a> {
    state: usize,
    pair: &'a [(&'a str, &'a str)],
}

impl PublishDateFilter<'_> {
    const fn new() -> Self {
        PublishDateFilter {
            state: 0,
            pair: &[
                ("全部", ""),
                ("2020年", "2020"),
                ("2019年", "2019"),
                ("2018年", "2018"),
                ("2017年", "2017"),
                ("2016年", "2016"),
                ("2015年", "2015"),
                ("2014年", "2014"),
                ("2013年", "2013"),
                ("2012年", "2012"),
                ("2011年", "2011"),
                ("2010年", "2010"),
                ("00年代", "200x"),
                ("90年代", "199x"),
                ("80年代", "198x"),
                ("更早", "197x"),
            ],
        }
    }
    const fn to_uri_part(&self) -> &str {
        self.pair[self.state].1
    }
}
#[derive(Debug)]
pub struct FirstLetterFilter<'a> {
    state: usize,
    pair: &'a [(&'a str, &'a str)],
}

impl FirstLetterFilter<'_> {
    const fn new() -> Self {
        FirstLetterFilter {
            state: 0,
            pair: &[
                ("全部", ""),
                ("A", "a"),
                ("B", "b"),
                ("C", "c"),
                ("D", "d"),
                ("E", "e"),
                ("F", "f"),
                ("G", "g"),
                ("H", "h"),
                ("I", "i"),
                ("J", "j"),
                ("K", "k"),
                ("L", "l"),
                ("M", "m"),
                ("N", "n"),
                ("O", "o"),
                ("P", "p"),
                ("Q", "q"),
                ("R", "r"),
                ("S", "s"),
                ("T", "t"),
                ("U", "u"),
                ("V", "v"),
                ("W", "w"),
                ("X", "x"),
                ("Y", "y"),
                ("Z", "z"),
                ("0-9", "0-9"),
            ],
        }
    }
    const fn to_uri_part(&self) -> &str {
        self.pair[self.state].1
    }
}

#[derive(Debug)]
pub struct StatusFilter<'a> {
    state: usize,
    pair: &'a [(&'a str, &'a str)],
}

impl StatusFilter<'_> {
    const fn new() -> Self {
        StatusFilter {
            state: 0,
            pair: &[("全部", ""), ("连载", "lianzai"), ("完结", "wanjie")],
        }
    }
    const fn to_uri_part(&self) -> &str {
        self.pair[self.state].1
    }
}

// Rate Limiter
#[derive(Clone, Debug)]
struct RateLimiter {
    permits_per_period: u32,
    period: Duration,
    last_request: Instant,
    permit_count: u32,
}

impl RateLimiter {
    fn new(permits_per_period: u32, period: Duration) -> Self {
        Self {
            permits_per_period,
            period,
            last_request: Instant::now(),
            permit_count: 0,
        }
    }

    async fn acquire(&mut self) {
        if self.permit_count >= self.permits_per_period {
            let elapsed = self.last_request.elapsed();
            if elapsed < self.period {
                sleep(self.period - elapsed).await;
            }
            self.permit_count = 0;
            self.last_request = Instant::now();
        }
        self.permit_count += 1;
    }
}

use anyhow::{bail, Context, Result};

// Preferences
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

impl Manhuagui {
    pub fn new(preferences: Preferences) -> Result<Self> {
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

        let image_server = ["https://i.hamreus.com", "https://cf.hamreus.com"];

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

        // let mut main_site_rate_limiter =
        //     RateLimiter::new(preferences.mainsite_ratelimit, Duration::from_secs(10));
        // let mut image_cdn_rate_limiter =
        //     RateLimiter::new(preferences.image_cdn_ratelimit, Duration::from_secs(1));

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            name: String::from("漫画柜"),
            lang: String::from("zh"),
            base_url,
            image_server,
            client,
        })
    }

    fn get_manga_url(&self, manga: &SManga) -> String {
        format!("{}{}", self.base_url, manga.url)
    }

    pub async fn popular_manga_request(&self, page: u32) -> Result<Response, reqwest::Error> {
        let url = format!("{}/list/view_p{}.html", self.base_url, page);
        self.client.get(url).send().await
    }

    pub async fn latest_updates_request(&self, page: u32) -> Result<Response, reqwest::Error> {
        let url = format!("{}/list/update_p{}.html", self.base_url, page);
        self.client.get(url).send().await
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub async fn search_manga_request(
        &self,
        page: u32,
        query: &str,
        filters: FilterList<'_>,
    ) -> Result<Response, reqwest::Error> {
        if !query.is_empty() {
            let url = format!("{}/s/{}_p{}.html", self.base_url, query, page);
            return self.client.get(url).send().await;
        }
        let params = filters
            .iter()
            .filter_map(|filter| match filter {
                Filter::Sort(_) => None,
                Filter::Locale(f) => Some(f.to_uri_part()),
                Filter::Genre(f) => Some(f.to_uri_part()),
                Filter::Reader(f) => Some(f.to_uri_part()),
                Filter::PublishDate(f) => Some(f.to_uri_part()),
                Filter::FirstLetter(f) => Some(f.to_uri_part()),
                Filter::Status(f) => Some(f.to_uri_part()),
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("_");

        let sort_order = filters
            .iter()
            .find_map(|filter| match filter {
                Filter::Sort(f) => Some(f.to_uri_part()),
                _ => None,
            })
            .unwrap_or("");

        let mut url = format!("{}/list", self.base_url);
        if !params.is_empty() {
            url.push_str(&format!("/{params}"));
        }
        url.push_str(&if sort_order.is_empty() {
            format!("/index_p{page}.html")
        } else {
            format!("/{sort_order}_p{page}.html")
        });

        self.client.get(url).send().await
    }

    pub async fn fetch_manga_details(&self, manga: &mut SManga) -> Result<()> {
        let url = self.get_manga_url(manga);
        let response = self.client.get(&url).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let bid = Regex::new(r"\d+")?
            .find(&manga.url)
            .map(|m| m.as_str().to_owned());

        if let Some(bid) = bid {
            let url = manga.url.clone();
            let base_url = self.base_url.clone();
            let client = self.client.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let post_url = format!("{base_url}/tools/submit_ajax.ashx?action=user_check_login");
                let _ = client
                    .post(&post_url)
                    .header("Referer", &url)
                    .header("X-Requested-With", "XMLHttpRequest")
                    .send()
                    .await;

                let get_url = format!("{base_url}/tools/vote.ashx?act=get&bid={bid}");
                let _ = client
                    .get(&get_url)
                    .header("Referer", &url)
                    .header("X-Requested-With", "XMLHttpRequest")
                    .send()
                    .await;
            });
        }

        Self::manga_details_parse(&document, manga)?;
        manga.initialized = true;
        Ok(())
    }

    async fn search_manga_by_id_parse(&self, id: &str) -> Result<MangasPage, anyhow::Error> {
        let url = format!("{}/comic/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let mut manga = SManga::create();
        Self::manga_details_parse(&document, &mut manga)?;
        manga.url = format!("/comic/{id}/");
        Ok(MangasPage {
            mangas: vec![manga],
            has_next_page: false,
        })
    }

    pub async fn fetch_search_manga(
        &self,
        page: u32,
        query: &str,
        filters: FilterList<'_>,
    ) -> Result<MangasPage, anyhow::Error> {
        if let Some(id) = query.strip_prefix(PREFIX_ID_SEARCH) {
            self.search_manga_by_id_parse(id).await
        } else {
            let response = self.search_manga_request(page, query, filters).await?;
            let url = response.url().to_string();
            let body = response.text().await?;
            let document = Html::parse_document(&body);
            Ok(Self::search_manga_parse(&document, &url))
        }
    }

    #[allow(clippy::unwrap_used)]
    fn search_manga_parse(document: &Html, url: &str) -> MangasPage {
        if url.contains("/s/") {
            let selector = Selector::parse("div.book-result > ul > li").unwrap();
            let next_page_selector = Selector::parse("span.current + a").unwrap();

            let mangas = document
                .select(&selector)
                .map(|element| Self::search_manga_from_element(element))
                .collect::<Vec<_>>();

            let has_next_page = document.select(&next_page_selector).next().is_some();

            MangasPage {
                mangas,
                has_next_page,
            }
        } else {
            let selector = Selector::parse("ul#contList > li").unwrap();
            let next_page_selector = Selector::parse("span.current + a").unwrap();

            let mangas = document
                .select(&selector)
                .map(|element| Self::popular_manga_from_element(element))
                .collect::<Vec<_>>();

            let has_next_page = document.select(&next_page_selector).next().is_some();

            MangasPage {
                mangas,
                has_next_page,
            }
        }
    }

    fn popular_manga_from_element(element: scraper::ElementRef<'_>) -> SManga {
        Self::manga_from_element(element)
    }
    fn manga_from_element(element: scraper::ElementRef<'_>) -> SManga {
        let mut manga = SManga::create();
        let cover_selector = Selector::parse("a.bcover").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        if let Some(cover) = element.select(&cover_selector).next() {
            manga.url = cover.value().attr("href").unwrap().to_string();
            manga.title = cover.value().attr("title").unwrap().trim().to_string();

            if let Some(img) = cover.select(&img_selector).next() {
                manga.thumbnail_url = if img.value().attr("src").is_some() {
                    img.value().attr("src").map(ToString::to_string)
                } else {
                    img.value().attr("data-src").map(ToString::to_string)
                };
            }
        }
        manga
    }

    fn search_manga_from_element(element: scraper::ElementRef<'_>) -> SManga {
        let mut manga = SManga::create();
        let detail_selector = Selector::parse("div.book-detail").unwrap();
        let title_selector = Selector::parse("dl > dt > a").unwrap();
        let img_selector = Selector::parse("div.book-cover > a.bcover > img").unwrap();

        if let Some(detail) = element.select(&detail_selector).next() {
            if let Some(a) = detail.select(&title_selector).next() {
                manga.url = a.value().attr("href").unwrap().to_string();
                manga.title = a.value().attr("title").unwrap().trim().to_string();
            }
            if let Some(img) = element.select(&img_selector).next() {
                manga.thumbnail_url = img.value().attr("src").map(ToString::to_string);
            }
        }
        manga
    }

    pub async fn chapter_list_parse(&self, manga: &SManga) -> Result<Vec<SChapter>, anyhow::Error> {
        let manga_url = manga.chapter_url(self);
        let response = self.client.get(manga_url).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let mut chapters = Vec::new();

        let viewstate_selector = Selector::parse("#__VIEWSTATE").unwrap();
        let erroraudit_show_selector = Selector::parse("#erroraudit_show").unwrap();
        if let Some(hidden_encrypted_chapter_list) = document.select(&viewstate_selector).next() {
            if let Some(val) = hidden_encrypted_chapter_list.value().attr("value") {
                let js_decode_func = r#"
            var LZString=(function(){var f=String.fromCharCode;var keyStrBase64="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";var baseReverseDic={};function getBaseValue(alphabet,character){if(!baseReverseDic[alphabet]){baseReverseDic[alphabet]={};for(var i=0;i<alphabet.length;i++){baseReverseDic[alphabet][alphabet.charAt(i)]=i}}return baseReverseDic[alphabet][character]}var LZString={decompressFromBase64:function(input){if(input==null)return"";if(input=="")return null;return LZString._0(input.length,32,function(index){return getBaseValue(keyStrBase64,input.charAt(index))})},_0:function(length,resetValue,getNextValue){var dictionary=[],next,enlargeIn=4,dictSize=4,numBits=3,entry="",result=[],i,w,bits,resb,maxpower,power,c,data={val:getNextValue(0),position:resetValue,index:1};for(i=0;i<3;i+=1){dictionary[i]=i}bits=0;maxpower=Math.pow(2,2);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}switch(next=bits){case 0:bits=0;maxpower=Math.pow(2,8);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}c=f(bits);break;case 1:bits=0;maxpower=Math.pow(2,16);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}c=f(bits);break;case 2:return""}dictionary[3]=c;w=c;result.push(c);while(true){if(data.index>length){return""}bits=0;maxpower=Math.pow(2,numBits);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}switch(c=bits){case 0:bits=0;maxpower=Math.pow(2,8);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}dictionary[dictSize++]=f(bits);c=dictSize-1;enlargeIn--;break;case 1:bits=0;maxpower=Math.pow(2,16);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}dictionary[dictSize++]=f(bits);c=dictSize-1;enlargeIn--;break;case 2:return result.join('')}if(enlargeIn==0){enlargeIn=Math.pow(2,numBits);numBits++}if(dictionary[c]){entry=dictionary[c]}else{if(c===dictSize){entry=w+w.charAt(0)}else{return null}}result.push(entry);dictionary[dictSize++]=w+entry.charAt(0);enlargeIn--;w=entry;if(enlargeIn==0){enlargeIn=Math.pow(2,numBits);numBits++}}}};return LZString})();String.prototype.splic=function(f){return LZString.decompressFromBase64(this).split(f)};
            "#;
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
            Selector::parse("div.book-detail > ul.detail-list > li.status > span > a.blue")
                .unwrap();
        let latest_chapter_href = document
            .select(&latest_chapter_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(String::from);

        let ch_num_regex = Regex::new(r"\d+").unwrap();

        let section_list_selector = Selector::parse("[id^=chapter-list-]").unwrap();
        for section in document.select(&section_list_selector) {
            let page_list_selector = Selector::parse("ul").unwrap();
            let mut page_list = section.select(&page_list_selector).collect::<Vec<_>>();
            page_list.reverse();

            for page in page_list {
                let chapter_list_selector = Selector::parse("li > a.status0").unwrap();
                for chapter_link in page.select(&chapter_list_selector) {
                    let mut current_chapter = SChapter::create();
                    current_chapter.url = chapter_link.value().attr("href").unwrap().to_string();
                    current_chapter.name = chapter_link.value().attr("title").map_or_else(
                        || {
                            chapter_link
                                .select(&Selector::parse("span").unwrap())
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

                    current_chapter.chapter_number = ch_num_regex
                        .find(&current_chapter.name)
                        .and_then(|m| m.as_str().parse::<f32>().ok())
                        .unwrap_or(-1.0);

                    if let Some(ref latest_href) = latest_chapter_href {
                        if current_chapter.url == *latest_href {
                            let date_selector = Selector::parse(
                                "div.book-detail > ul.detail-list > li.status > span > span.red",
                            )
                            .unwrap();
                            if let Some(date_element) = document.select(&date_selector).last() {
                                current_chapter.date_upload =
                                    Some(Self::parse_date(date_element.text().next().unwrap()));
                            }
                        }
                    }
                    chapters.push(current_chapter);
                }
            }
        }

        Ok(chapters)
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

    fn manga_details_parse(document: &Html, manga: &mut SManga) -> Result<()> {
        let title_selector = Selector::parse("div.book-title > h1:nth-child(1)")
            .map_err(|x| anyhow::anyhow!("{x}"))?;
        let description_selector =
            Selector::parse("div#intro-all").map_err(|x| anyhow::anyhow!("{x}"))?;
        let thumbnail_selector =
            Selector::parse("p.hcover > img").map_err(|x| anyhow::anyhow!("{x}"))?;
        let author_selector = Selector::parse("span > a").map_err(|x| anyhow::anyhow!("{x}"))?;
        let genre_selector = Selector::parse("span > a").map_err(|x| anyhow::anyhow!("{x}"))?;
        let status_selector =
            Selector::parse("div.book-detail > ul.detail-list > li.status > span > span")
                .map_err(|x| anyhow::anyhow!("{x}"))?;

        manga.title = document
            .select(&title_selector)
            .next()
            .map_or(String::new(), |el| {
                el.text().collect::<String>().trim().to_owned()
            });
        manga.description = document
            .select(&description_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned());
        manga.thumbnail_url = document
            .select(&thumbnail_selector)
            .next()
            .and_then(|el| el.value().attr("src").map(String::from));
        manga.author = Some(
            document
                .select(&author_selector)
                .map(|el| el.text().collect::<String>())
                .filter(|x| x.contains("漫画作者") || x.contains("漫画作者"))
                .collect::<Vec<String>>()
                .join(", ")
                .trim()
                .to_owned(),
        );
        manga.genre = Some(
            document
                .select(&genre_selector)
                .map(|el| el.text().collect::<String>())
                .filter(|x| x.contains("漫画剧情") || x.contains("漫畫劇情"))
                .collect::<Vec<String>>()
                .join(", ")
                .trim()
                .to_owned(),
        );
        manga.status = match document
            .select(&status_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .as_deref()
        {
            Some("连载中" | "連載中") => MangaStatus::Ongoing,
            Some("已完结" | "已完結") => MangaStatus::Completed,
            _ => MangaStatus::Unknown,
        };

        Ok(())
    }

    pub async fn page_list_parse(&self, chapter: &SChapter) -> Result<Vec<Page>, anyhow::Error> {
        let manga_url = chapter.page_url(self);
        let response = self.client.get(manga_url).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let erroraudit_show_selector = Selector::parse("#erroraudit_show").unwrap();
        if document.select(&erroraudit_show_selector).next().is_some() {
            bail!("R18作品显示开关未开启或未生效");
        }

        let re = Regex::new(r#"window\[".*?"\](\(.*\)\s*\{[\s\S]+\}\s*\(.*\))"#).unwrap();
        let re2 = Regex::new(r"\{.*\}").unwrap();

        let js_decode_func = r#"
        var LZString=(function(){var f=String.fromCharCode;var keyStrBase64="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";var baseReverseDic={};function getBaseValue(alphabet,character){if(!baseReverseDic[alphabet]){baseReverseDic[alphabet]={};for(var i=0;i<alphabet.length;i++){baseReverseDic[alphabet][alphabet.charAt(i)]=i}}return baseReverseDic[alphabet][character]}var LZString={decompressFromBase64:function(input){if(input==null)return"";if(input=="")return null;return LZString._0(input.length,32,function(index){return getBaseValue(keyStrBase64,input.charAt(index))})},_0:function(length,resetValue,getNextValue){var dictionary=[],next,enlargeIn=4,dictSize=4,numBits=3,entry="",result=[],i,w,bits,resb,maxpower,power,c,data={val:getNextValue(0),position:resetValue,index:1};for(i=0;i<3;i+=1){dictionary[i]=i}bits=0;maxpower=Math.pow(2,2);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}switch(next=bits){case 0:bits=0;maxpower=Math.pow(2,8);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}c=f(bits);break;case 1:bits=0;maxpower=Math.pow(2,16);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}c=f(bits);break;case 2:return""}dictionary[3]=c;w=c;result.push(c);while(true){if(data.index>length){return""}bits=0;maxpower=Math.pow(2,numBits);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}switch(c=bits){case 0:bits=0;maxpower=Math.pow(2,8);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}dictionary[dictSize++]=f(bits);c=dictSize-1;enlargeIn--;break;case 1:bits=0;maxpower=Math.pow(2,16);power=1;while(power!=maxpower){resb=data.val&data.position;data.position>>=1;if(data.position==0){data.position=resetValue;data.val=getNextValue(data.index++)}bits|=(resb>0?1:0)*power;power<<=1}dictionary[dictSize++]=f(bits);c=dictSize-1;enlargeIn--;break;case 2:return result.join('')}if(enlargeIn==0){enlargeIn=Math.pow(2,numBits);numBits++}if(dictionary[c]){entry=dictionary[c]}else{if(c===dictSize){entry=w+w.charAt(0)}else{return null}}result.push(entry);dictionary[dictSize++]=w+entry.charAt(0);enlargeIn--;w=entry;if(enlargeIn==0){enlargeIn=Math.pow(2,numBits);numBits++}}}};return LZString})();String.prototype.splic=function(f){return LZString.decompressFromBase64(this).split(f)};
        "#;

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

impl SManga {
    #[must_use]
    pub const fn create() -> Self {
        Self {
            url: String::new(),
            title: String::new(),
            thumbnail_url: None,
            author: None,
            description: None,
            genre: None,
            status: MangaStatus::Unknown,
            initialized: false,
        }
    }
    #[must_use]
    pub fn chapter_url(&self, api: &Manhuagui) -> String {
        format!("{}{}", api.base_url, self.url)
    }
}

impl SChapter {
    #[must_use]
    pub const fn create() -> Self {
        Self {
            url: String::new(),
            name: String::new(),
            chapter_number: -1.0,
            date_upload: None,
        }
    }
    #[must_use]
    pub fn page_url(&self, api: &Manhuagui) -> String {
        format!("{}{}", api.base_url, self.url)
    }
}

impl std::fmt::Display for MangaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Ongoing => write!(f, "Ongoing"),
            Self::Completed => write!(f, "Completed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_popular_manga_request() {
        let preferences = Preferences::default();
        let manhuagui = Manhuagui::new(preferences).unwrap();
        let response = manhuagui.popular_manga_request(1).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_latest_updates_request() {
        let preferences = Preferences::default();
        let manhuagui = Manhuagui::new(preferences).unwrap();
        let response = manhuagui.latest_updates_request(1).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_search_manga_request() {
        let preferences = Preferences::default();
        let manhuagui = Manhuagui::new(preferences).unwrap();
        let filters = vec![
            Filter::Sort(SortFilter::new()),
            Filter::Locale(LocaleFilter::new()),
            Filter::Genre(GenreFilter::new()),
            Filter::Reader(ReaderFilter::new()),
            Filter::PublishDate(PublishDateFilter::new()),
            Filter::FirstLetter(FirstLetterFilter::new()),
            Filter::Status(StatusFilter::new()),
        ];
        let response = manhuagui.search_manga_request(1, "", filters).await;
        assert!(response.is_ok());

        let response = manhuagui.search_manga_request(1, "火影", vec![]).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_manga_details() {
        let preferences = Preferences::default();
        let manhuagui = Manhuagui::new(preferences).unwrap();
        let mut manga = SManga {
            url: String::from("/comic/1769/"),
            title: String::from("BORUTO -火影新世代- -NARUTO NEXT GENERATIONS-"),
            thumbnail_url: Some(String::from("https://www.manhuagui.com/d/cover/2016/05/1769_573079f405462.jpg")),
            author: Some(String::from("岸本齐史,小太刀右京,池本干雄")),
            description: Some(String::from("历经动荡与变迁，木叶忍者村人丁兴旺，人才济济，和平时代下成长起来的孩子们迅速成长，他们是木叶新的力量。作为前代传奇忍者的子女，漩涡博人、宇智波佐良娜、三月等新生代忍者走上属于自己的忍者之路。他们的热血故事正在火热上演！")),
            genre: Some(String::from("热血,冒险,励志")),
            status: MangaStatus::Ongoing,
            initialized: false,
        };
        let result = manhuagui.fetch_manga_details(&mut manga).await;
        assert!(result.is_ok());
        assert_eq!(manga.title, "BORUTO -火影新世代- -NARUTO NEXT GENERATIONS-");
        assert!(manga.description.is_some());
        assert!(manga.thumbnail_url.is_some());
        assert!(manga.author.is_some());
        assert!(manga.genre.is_some());
        assert_eq!(manga.status, MangaStatus::Ongoing);
    }

    #[tokio::test]
    async fn test_fetch_search_manga() {
        let preferences = Preferences::default();
        let manhuagui = Manhuagui::new(preferences).unwrap();
        let filters = vec![];
        let result = manhuagui.fetch_search_manga(1, "id:50750", filters).await;
        assert!(result.is_ok(), "{result:#?}");
        let mangas_page = result.unwrap();
        assert!(!mangas_page.mangas.is_empty());
    }

    //     #[tokio::test]
    //     async fn test_chapter_list_parse() {
    //         let preferences = Preferences::default();
    //         let manhuagui = Manhuagui::new(preferences).unwrap();
    //         let result = manhuagui
    //             .chapter_list_parse("https://www.manhuagui.com/comic/32602/")
    //             .await;
    //         assert!(result.is_ok());
    //         let chapters = result.unwrap();
    //         assert!(!chapters.is_empty());
    //     }

    //     #[tokio::test]
    //     async fn test_page_list_parse() {
    //         let preferences = Preferences::default();
    //         let api = Manhuagui::new(preferences).unwrap();
    //         let result = api
    //             .page_list_parse("https://www.manhuagui.com/comic/13735/136663.html")
    //             .await;
    //         assert!(result.is_ok());
    //         let pages = result.unwrap();
    //         assert!(!pages.is_empty());
    //     }
}
