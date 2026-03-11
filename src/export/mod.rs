pub mod css;
pub mod json;
pub mod template;
pub mod terminal;

use std::io;
use std::path::PathBuf;

pub fn get_export_directory() -> PathBuf {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg_config_home.is_empty() {
            return PathBuf::from(xdg_config_home).join("spex").join("export");
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home)
                .join(".config")
                .join("spex")
                .join("export");
        }
    }

    PathBuf::from(".config").join("spex").join("export")
}

pub fn export_file_path(filename: &str) -> io::Result<PathBuf> {
    let export_dir = get_export_directory();
    std::fs::create_dir_all(&export_dir)?;

    Ok(export_dir.join(filename))
}
