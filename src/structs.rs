use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};
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

    pub fn clean_tokens(&mut self) {
        for (path, meta) in &mut self.files {
            meta.tokens.retain(|t| {
                let keep = t.expires.map_or(true, |e| e > OffsetDateTime::now_utc());
                if !keep {
                    println!("Removed expired token for {}: {}", path, t);
                }
                keep
            });
        }
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


impl fmt::Display for FileMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Hidden field
        write!(f, "Hidden: {}\n", if self.hidden { "Yes" } else { "No" })?;

        // Password field
        write!(f, "Password: {}\n", if self.password_hash.is_some() { "Yes" } else { "No" })?;

        // Tokens, only if there are any
        if !self.tokens.is_empty() {
            writeln!(f, "Token ({}):", self.tokens.len())?;
            for token in &self.tokens {
                writeln!(f, "  - {}", token)?;
            }
        } else {
            writeln!(f, "Tokens: None")?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Token {
    pub value: String,
    pub expires: Option<OffsetDateTime>
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(expires) = &self.expires {
            let local_time = expires.to_offset(
                UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)
            );
            let formatted = local_time
                .format(&format_description!("[year]-[month]-[day] [hour]:[minute]"))
                .unwrap_or_else(|_| "invalid date".to_string());
            write!(f, "{} (expires: {})", self.value, formatted)
        } else {
            write!(f, "{}", self.value)
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct FileAccessParameters {
    pub(crate) token: Option<String>,
    pub(crate) password: Option<String>,
}