use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::crypto::verify_password;
use crate::structs::{AppState, FileAccessParameters};
use crate::filesystem::{list_directory, resolve_safe_path, stream_file};
use crate::token::validate_token;

pub async fn handle_root(
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {

    let entries = list_directory(state.config.base_directory.as_ref(), state.clone()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(entries).into_response())
}

pub async fn handle_files(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(params): Query<FileAccessParameters>,
) -> Result<Response, StatusCode> {

    let path = path.trim_start_matches('/');

    let base_dir = &state.config.base_directory;
    let resolved = resolve_safe_path(base_dir.as_ref(), &path)?;

    let user_file_meta = state.meta.read().unwrap().files.get(path).cloned().unwrap_or_default();

    // Check hidden access
    if user_file_meta.hidden {
        let token = params.token.as_ref().ok_or(StatusCode::FORBIDDEN)?;
        if !validate_token(token, &user_file_meta.tokens) {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // If it's a file, check password
    if resolved.is_file() {
        if let Some(hash) = &user_file_meta.password_hash {
            let password = params.password.ok_or(StatusCode::UNAUTHORIZED)?;

            if !verify_password(&*password, hash) {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }

        // Return the file contents or redirect to download
        return stream_file(resolved).await;
    }

    // It's a directory -> list entries
    let entries = list_directory(resolved.as_ref(), state).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(entries).into_response())
}