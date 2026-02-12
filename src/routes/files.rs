use std::time::SystemTime;
use axum::extract::{Path, State};
use axum::Router;
use axum::routing::get;
use serde::Serialize;
use crate::AppState;
use crate::services::filesystem;

#[derive(Debug)]
#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub is_dir: bool,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/*path", get(handle))
}

async fn handle(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> filesystem::Result {
    filesystem::handle_path(&state.config, path).await
}