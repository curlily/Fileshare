use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use serde::{Deserialize, Serialize};
use anyhow::Context;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)] // <- important
pub struct Config {
    pub server: ServerConfig,
    pub base_directory: PathBuf,
    pub meta_directory: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 8080,
            },
            base_directory: ".".into(),
            meta_directory: ".".into(),
        }
    }
}

pub fn load_or_create_config(path: impl AsRef<Path>) -> anyhow::Result<Config> {

    let path = path.as_ref();

    if path.exists() {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file {:?}", path))?;

        let config: Config = toml::from_str(&contents)
            .context("Failed to parse TOML config")?;

        Ok(config)
    } else {

        write_default_config(path)
            .with_context(|| format!("Failed to write default config to {:?}", path))?;

        println!("Created default config file - please review it.");
        exit(0)
    }
}

fn write_default_config(path: &Path) -> std::io::Result<()> {
    let default = r#"
# Base directory for file browser
base_directory = 'C:\Users\Public\Documents'

[server]
host = "127.0.0.1"
port = 8080
"#;

    fs::write(path, default.trim_start())
}