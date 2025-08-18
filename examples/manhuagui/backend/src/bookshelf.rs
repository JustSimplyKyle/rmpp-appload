use std::{collections::HashMap, env, path::PathBuf};

use backend::{MangaBackend, SManga};
use serde::{Deserialize, Serialize};
use smol::fs;

use crate::manga_reader::MangaReader;

#[derive(Clone)]
pub struct BookShelf(HashMap<BookShelfKey, MangaReader>);

#[derive(Clone, Eq, Hash, PartialEq, Deserialize, Serialize, Debug, Default)]
pub struct BookShelfKey {
    pub manga_url: String,
    backend_id: String,
}

impl BookShelfKey {
    fn from_manga(manga: &MangaReader) -> Self {
        Self {
            backend_id: manga.api.id(),
            manga_url: manga.details.url.clone(),
        }
    }
    pub fn new(id: &(impl MangaBackend + ?Sized), manga_url: String) -> Self {
        Self {
            backend_id: id.id(),
            manga_url,
        }
    }
}

impl BookShelf {
    pub fn new() -> anyhow::Result<Self> {
        let path = Self::path();

        if let Some(path) = path.filter(|x| x.exists()) {
            let s = std::fs::read_to_string(&path)?;
            let x: Vec<(BookShelfKey, MangaReader)> = serde_json::from_str(&s)?;
            let x = dbg!(x.into_iter().collect());
            Ok(Self(x))
        } else {
            Ok(Self(HashMap::new()))
        }
    }

    fn path() -> Option<PathBuf> {
        let home = env::home_dir()?;

        if !home.join(".config/mangarr").exists() {
            std::fs::create_dir_all(home.join(".config/mangarr")).expect("can't create fs");
        }

        Some(home.join(".config/mangarr/bookshelf.json"))
    }

    pub async fn insert(&mut self, manga: MangaReader) -> anyhow::Result<()> {
        let key = BookShelfKey::from_manga(&manga);
        self.0.insert(key, manga);
        self.save().await?;
        Ok(())
    }
    pub async fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::path() {
            let result = self.0.iter().collect::<Vec<_>>();
            let contents = serde_json::to_string(&result)?;
            fs::write(path, contents).await?;
        }
        Ok(())
    }

    pub async fn with_mut(
        &mut self,
        f: impl FnOnce(Option<&mut MangaReader>),
        manga: &MangaReader,
    ) -> anyhow::Result<()> {
        let key = BookShelfKey::from_manga(manga);
        f(self.0.get_mut(&key));
        self.save().await
    }

    pub const fn bookshelf(&self) -> &HashMap<BookShelfKey, MangaReader> {
        &self.0
    }
}
