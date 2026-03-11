pub mod css;
pub mod json;
pub mod terminal;

use std::io;
use std::path::PathBuf;

pub fn export_file_path(filename: &str) -> io::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME environment variable not set"))?;

    let export_dir = PathBuf::from(home).join(".config").join("spex").join("export");
    std::fs::create_dir_all(&export_dir)?;

    Ok(export_dir.join(filename))
}
