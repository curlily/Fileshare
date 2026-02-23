use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc};
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{Response};
use tokio_util::io::ReaderStream;
use crate::structs::{AppState, FileEntry};

pub fn list_directory(path: &Path, state: Arc<AppState>) -> io::Result<Vec<FileEntry>> {

    let mut entries = Vec::<FileEntry>::new();

    let relative_path = path.strip_prefix(Path::new(&state.config.base_directory)).unwrap();

    for entry in fs::read_dir(path)? {

        let entry = entry?;
        let file_meta = entry.metadata()?;
        let meta_path = &normalize_path(&format!("{}\\{}", relative_path.to_string_lossy(), entry.file_name().display()));
        let user_file_meta = state.meta.read().unwrap().get(meta_path);

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
            requires_password: user_file_meta.password_hash.is_some(),
        });
    }

    // Sort entries
    entries.sort_by_key(|e| (
        !e.is_dir,                // false (dirs) comes before true (files)
        e.name.to_lowercase(),
    ));

    Ok(entries)
}

pub async fn stream_file(path: PathBuf) -> Result<Response, StatusCode> {

    let file = tokio::fs::File::open(&path).await.map_err(|_| StatusCode::NOT_FOUND)?;

    let file_meta = file
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let size = file_meta.len();

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

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