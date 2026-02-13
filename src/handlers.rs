use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use crate::structs::AppState;
use crate::services::filesystem;

pub async fn handle_root(
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    println!("Handling root");
    filesystem::handle_path(state.config.base_directory.clone(), "".to_string()).await
}

pub async fn handle_files(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    let path = path.trim_start_matches('/');
    println!("Handling files: {}", path);
    filesystem::handle_path(state.config.base_directory.clone(), path.to_string()).await
}