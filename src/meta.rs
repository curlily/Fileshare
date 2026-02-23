use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use anyhow::Context;
use notify::{Event, RecursiveMode, Watcher};
use crate::config::Config;
use crate::structs::{AppState, MetaFile};

pub fn load_or_create_meta(path: &PathBuf) -> anyhow::Result<MetaFile> {

    if !path.exists() {
        fs::write(&path, "")?;
        println!("Created Meta.toml in {}", &path.to_string_lossy());
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read meta file {:?}", &path))?;

    let meta: MetaFile = toml::from_str(&contents)
        .context("Failed to parse TOML meta")?;

    Ok(meta)
}

pub fn reload_meta(state: &AppState, path: &PathBuf) -> anyhow::Result<()> {
    let new_meta = load_or_create_meta(path)?;
    let mut guard = state.meta.write().unwrap();
    *guard = new_meta;
    println!("meta.toml reloaded");
    Ok(())
}

pub fn save_meta(meta: &MetaFile) -> anyhow::Result<()> {
    let path: PathBuf = "Meta.toml".into();
    let tmp = path.with_extension("tmp");
    let text = toml::to_string_pretty(meta)?;
    fs::write(&tmp, text)?;
    fs::rename(tmp, path)?;
    Ok(())
}

pub fn get_meta_path(config: &Config) -> PathBuf {
    config.meta_directory.join("Meta.toml")
} 

pub fn start_meta_watcher(
    state: Arc<AppState>,
) -> notify::Result<notify::RecommendedWatcher> {
    let (tx, rx) = channel();
    
    let meta_path = get_meta_path(&state.config);

    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&meta_path, RecursiveMode::NonRecursive)?;
    
    println!("Watching {} for changes...", &meta_path.to_string_lossy());

    std::thread::spawn(move || {

        let mut last_reload = Instant::now() - Duration::from_secs(1);

        for res in rx {
            match res {
                Ok(Event { .. }) => {

                    let now = Instant::now();
                    if now.duration_since(last_reload) < Duration::from_millis(200) {
                        continue; // ignore duplicate burst
                    }

                    last_reload = now;

                    if let Err(e) = reload_meta(&state, &meta_path) {
                        eprintln!("Failed to reload meta: {e}");
                    }
                }
                Err(e) => eprintln!("watch error: {e}"),
            }
        }
    });

    Ok(watcher)
}