use std::sync::Arc;
use anyhow::Context;
use clap::Parser;

mod handlers;
mod config;
pub mod structs;
mod meta;
mod token;
mod crypto;
mod cli;
mod app;
pub mod filesystem;

use crate::app::{kill_app, run_app};
use crate::cli::{handle_meta_command, Cli, Commands};
use crate::config::load_or_create_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let cli = Cli::parse();

    let mut config = load_or_create_config("Config.toml")
        .context("Loading config file")?;

    config.base_directory = config.base_directory.canonicalize()?;

    let config = Arc::new(config);

    match cli.command {
        Commands::Start => {
            run_app(config).await;
        }

        Commands::Stop => {
            kill_app()?;
        }

        Commands::Meta { action } => {
            handle_meta_command(config, action)?;
        }
    }

    Ok(())
}