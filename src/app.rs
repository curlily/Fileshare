use std::sync::{Arc, RwLock};
use anyhow::Context;
use axum::Router;
use axum::routing::get;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use crate::config::Config;
use crate::handlers;
use crate::meta::{get_meta_path, load_or_create_meta, save_meta, start_meta_watcher};
use crate::structs::AppState;

pub async fn run_app(config: Arc<Config>) {

    std::fs::write("fileshare.pid", std::process::id().to_string()).unwrap();

    let mut meta = load_or_create_meta(&get_meta_path(&config))
        .context("Loading meta file")
        .unwrap();

    meta.clean_tokens();
    save_meta(&meta).unwrap();

    let address = format!("{}:{}", config.server.host, config.server.port.to_string());

    let state = Arc::new(AppState { config, meta: RwLock::from(meta) });

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let _meta_watcher = start_meta_watcher(state.clone()).expect("Failed to start meta watcher");
    
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
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on configured address
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("Server started successfully at {:?}", &address);
    axum::serve(listener, app).await.unwrap();
}

pub fn kill_app() -> anyhow::Result<()> {
    let pid = std::fs::read_to_string("fileshare.pid")?;
    let pid: u32 = pid.trim().parse()?;

    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;
    }

    #[cfg(windows)]
    {
        std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .spawn()?;
    }
    
    println!("Killed pid {}", &pid);
    std::fs::remove_file("fileshare.pid")?;

    Ok(())
}
