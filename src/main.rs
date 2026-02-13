use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Context;
use axum::Router;
use axum::routing::get;
use tower_http::services::{ServeDir, ServeFile};

mod handlers;
mod config;
mod services;
pub mod structs;

use config::load_or_create_config;
use crate::config::Config;

pub struct AppState {
    config: Config,
}

#[tokio::main]
async fn main() {

    let mut config = load_or_create_config("Config.toml")
        .context("Loading config file")
        .unwrap();

    config.base_directory = PathBuf::from(&config.base_directory)
        .canonicalize()
        .expect("Invalid base directory")
        .display()
        .to_string();

    let address = format!("{}:{}", config.server.host, config.server.port.to_string());

    // build our application with a single route
    let app = Router::new()
        // API
        .route("/api/files", get(handlers::handle_root))
        .route("/api/files{*path}", get(handlers::handle_files))
        // Frontend (catch-all)
        .fallback_service(
            ServeDir::new("client")
                .fallback(ServeFile::new("client/index.html"))
        )
        .with_state(Arc::new(AppState { config }));

    // run our app with hyper, listening globally on configured address
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("Server started successfully at {:?}", &address);
    axum::serve(listener, app).await.unwrap();
}