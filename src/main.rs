use std::sync::Arc;
use anyhow::Context;
use axum::{Router, routing::get};
use tower_http::services::ServeDir;

mod handlers;
mod config;
mod services;
mod routes;
mod app;
mod state;

use config::load_or_create_config;
use crate::routes::files;
use crate::state::AppState;

#[tokio::main]
async fn main() {

    let config = load_or_create_config("Config.toml")
        .context("Loading config file")
        .unwrap();

    let address = format!("{}:{}", config.server.host, config.server.port.to_string());

    // build our application with a single route
    let app = Router::new()
        // API
        .nest("/api/files", files::router())
        // Frontend (catch-all)
        .fallback_service(ServeDir::new("frontend/dist"))
        .with_state(Arc::new(AppState { config }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("Server started successfully at {:?}", &address);
    axum::serve(listener, app).await.unwrap();
}