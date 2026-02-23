use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::config::Config;
use crate::filesystem::normalize_path;

pub struct AppState {
    pub config: Arc<Config>,
    pub meta: RwLock<MetaFile>,
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub is_dir: bool,
    pub requires_password: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MetaFile {
    #[serde(default)]
    pub files: HashMap<String, FileMeta>,
}

impl MetaFile {
    pub fn normalize(self) -> MetaFile {
        let files = self.files.into_iter().map(|(k, v)| {
            let normalized = normalize_path(&k);
            (normalized, v)
        }).collect();

        MetaFile { files }
    }

    pub fn get(&self, path: &str) -> FileMeta {
        self.files.get(path).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct FileMeta {
    #[serde(default)]
    pub hidden: bool,

    #[serde(default)]
    pub password_hash: Option<String>,

    #[serde(default)]
    pub tokens: Vec<Token>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Token {
    pub value: String,
    pub expires: Option<OffsetDateTime>
}

#[derive(Deserialize, Debug)]
pub struct FileAccessParameters {
    pub(crate) token: Option<String>,
    pub(crate) password: Option<String>,
}