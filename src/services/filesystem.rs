use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::Json;
use axum::response::{IntoResponse, Response};
use tokio_util::io::ReaderStream;
use crate::structs::{AppState, FileEntry, MetaFile};

pub async fn handle_path(state: Arc<AppState>, path: String) -> Result<Response, StatusCode> {

    let base_dir = state.config.base_directory.clone();
    let root = Path::new(&base_dir);
    let resolved = resolve_safe_path(&root, &path)?;

    if resolved.is_dir() {
        let entries = tokio::task::spawn_blocking(move || {
            list_directory_sync(&resolved, path, &state.meta)
        })
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(entries).into_response())
    } else if resolved.is_file() {
        stream_file(resolved).await
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
pub fn list_directory_sync(absolute_path: &Path, relative_path: String, user_meta: &MetaFile) -> io::Result<Vec<FileEntry>> {

    let mut entries = Vec::<FileEntry>::new();

    for entry in fs::read_dir(absolute_path)? {

        let entry = entry?;
        let file_meta = entry.metadata()?;
        let user_file_meta = user_meta.get(&normalize_path(&format!("{}\\{}", &relative_path, entry.file_name().display())));

        // Skip if file should be hidden
        if user_file_meta.hidden { continue; }

        let is_dir = file_meta.is_dir();
        let size = if is_dir { 0 } else { file_meta.len() };

        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            size,
            created: file_meta.created().ok(),
            modified: file_meta.modified().ok(),
            is_dir,
        });
    }

    // Sort entries
    entries.sort_by_key(|e| (
        !e.is_dir,                // false (dirs) comes before true (files)
        e.name.to_lowercase(),
    ));

    Ok(entries)
}

async fn stream_file(path: PathBuf) -> Result<Response, StatusCode> {

    let file = tokio::fs::File::open(&path).await.map_err(|_| StatusCode::NOT_FOUND)?;

    let metadata = file
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let size = metadata.len();

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, size)
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\""))
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn resolve_safe_path(root: &Path, user_path: &str) -> Result<PathBuf, StatusCode> {

    let candidate = root.join(user_path);

    let canonical = candidate
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !canonical.starts_with(root) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(canonical)
}

pub fn normalize_path(input: &str) -> String {
    input
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
}