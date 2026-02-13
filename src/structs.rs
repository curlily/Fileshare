use std::time::SystemTime;
use serde::Serialize;

#[derive(Debug)]
#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub is_dir: bool,
}