use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use time::OffsetDateTime;
use crate::config::Config;
use crate::crypto::hash_password;
use crate::meta::{get_meta_path, load_or_create_meta, save_meta};
use crate::structs::{FileMeta};
use crate::token::generate_token;

#[derive(Parser)]
#[command(name = "fileshare")]
#[command(about = "File sharing server with metadata control")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the HTTP server
    Start,

    /// Stop the HTTP Server
    Stop,

    /// Manage file metadata
    Meta {
        #[command(subcommand)]
        action: MetaCommand,
    },
}

#[derive(Subcommand)]
pub enum MetaCommand {
    /// Add a token to a file
    AddToken {
        path: PathBuf,
        #[arg(long)]
        expires: Option<String>,
    },

    /// Remove a token from a file
    RemoveToken {
        path: PathBuf,
        token: String,
    },

    /// List the tokens of a file
    ListToken {
        path: PathBuf,
    },

    /// Set or change a password
    SetPassword {
        path: PathBuf,
        password: String,
    },

    /// Remove password protection
    ClearPassword {
        path: PathBuf,
    },

    /// Hide or unhide a file
    Hidden {
        path: PathBuf,
        #[arg(long)]
        hidden: bool,
    },
}

pub fn handle_meta_command(config: Arc<Config>, cmd: MetaCommand) -> anyhow::Result<()> {

    match cmd {

        MetaCommand::AddToken { path, expires } => {

            let path_string = path.to_string_lossy().into_owned();
            let mut meta = load_or_create_meta(&get_meta_path(&config))?;
            let file_meta = meta.files.entry(path_string.clone()).or_insert_with(FileMeta::default);
            let expiry = expires.and_then(|s| OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339).ok());
            let token = generate_token(expiry);

            file_meta.tokens.push(token.clone());

            save_meta(&meta)?;
            println!("Token for {} created: {:?}", path.iter().last().unwrap().to_string_lossy(), token);
        }

        MetaCommand::RemoveToken { path, token } => {

            let path_string = path.to_string_lossy().into_owned();
            let mut meta = load_or_create_meta(&get_meta_path(&config))?;
            let file_meta = meta.files.entry(path_string.clone()).or_insert_with(FileMeta::default);

            file_meta.tokens.retain(|t| t.value != token);

            save_meta(&meta)?;
            println!("Token for {} removed", path.iter().last().unwrap().to_string_lossy());
        }

        MetaCommand::ListToken { path } => {

            let path_string = path.to_string_lossy().into_owned();
            let mut meta = load_or_create_meta(&get_meta_path(&config))?;
            let file_meta = meta.files.entry(path_string.clone()).or_insert_with(FileMeta::default);

            println!("Existing tokens for {}: {:?}", path.iter().last().unwrap().to_string_lossy(), file_meta.tokens);
        }

        MetaCommand::SetPassword { path, password } => {

            let mut meta = load_or_create_meta(&get_meta_path(&config))?;
            let file_meta = meta.files.entry(path.to_string_lossy().into_owned()).or_insert_with(FileMeta::default);

            file_meta.password_hash = Some(hash_password(&password).unwrap());
            save_meta(&meta)?;

            println!("Password for {} updated", path.iter().last().unwrap().to_string_lossy());
        }

        MetaCommand::ClearPassword { path } => {

            let path_string = path.to_string_lossy().into_owned();
            let mut meta = load_or_create_meta(&get_meta_path(&config))?;
            let file_meta = meta.files.entry(path_string.clone()).or_insert_with(FileMeta::default);

            file_meta.password_hash = None;

            // Retain only non default entries
            meta.files.retain(|_, meta| *meta != FileMeta::default());

            save_meta(&meta)?;

            println!("Password for {} removed", path.iter().last().unwrap().to_string_lossy());
        }

        MetaCommand::Hidden { path, hidden } => {

            let mut meta = load_or_create_meta(&get_meta_path(&config))?;
            let file_meta = meta.files.entry(path.to_string_lossy().into_owned()).or_insert_with(FileMeta::default);

            file_meta.hidden = hidden;

            save_meta(&meta)?;

            println!("{} is {}hidden", path.iter().last().unwrap().to_string_lossy(), if !hidden { "no longer" } else { "" });
        }
    }

    Ok(())
}

