use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use crate::structs::AppState;
use crate::services::filesystem;

pub async fn handle_root(
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    filesystem::handle_path(state, "".to_string()).await
}

pub async fn handle_files(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    let path = path.trim_start_matches('/');
    filesystem::handle_path(state, path.to_string()).await
}