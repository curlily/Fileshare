use std::fs;
use std::path::Path;
use anyhow::Context;
use crate::structs::MetaFile;

pub fn load_or_create_meta(path: impl AsRef<Path>) -> anyhow::Result<MetaFile> {

    let path = path.as_ref();

    if !path.exists() {
        write_default_meta(path)
            .with_context(|| format!("Failed to write default config to {:?}", path))?;

        println!("Created default meta file");
    }

    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read meta file {:?}", path))?;

    let meta: MetaFile = toml::from_str(&contents)
        .context("Failed to parse TOML meta")?;

    Ok(meta)
}

fn write_default_meta(path: &Path) -> std::io::Result<()> {
    let default = r#"
[files]

# ["files.example.txt"]
# hidden = true
# tokens = []

# ["files.directory/example.txt"]
# password_hash = ""
"#;

    fs::write(path, default.trim_start())
}