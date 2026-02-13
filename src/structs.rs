use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::services::filesystem::normalize_path;

pub struct AppState {
    pub config: Config,
    pub meta: MetaFile,
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub is_dir: bool,
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
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FileMeta {
    #[serde(default)]
    pub hidden: bool,

    #[serde(default)]
    pub password_hash: Option<String>,

    #[serde(default)]
    pub tokens: Vec<String>,
}