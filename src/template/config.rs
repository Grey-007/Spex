use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct TemplateConfig {
    #[serde(default)]
    pub template: Vec<TemplateEntry>,
    #[serde(default)]
    pub template_dirs: TemplateDirs,
    #[serde(default)]
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateEntry {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct TemplateDirs {
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct HooksConfig {
    #[serde(default)]
    pub commands: Vec<String>,
}

pub fn get_spex_config_directory() -> PathBuf {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg_config_home.is_empty() {
            return PathBuf::from(xdg_config_home).join("spex");
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home).join(".config").join("spex");
        }
    }

    PathBuf::from(".config").join("spex")
}

pub fn get_config_file_path() -> PathBuf {
    get_spex_config_directory().join("config.toml")
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(suffix) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return PathBuf::from(home).join(suffix);
            }
        }
    }

    PathBuf::from(path)
}

pub fn load_config_from_path(config_path: Option<&Path>) -> io::Result<Option<TemplateConfig>> {
    let path = match config_path {
        Some(path) => {
            let raw = path.to_string_lossy();
            expand_tilde(&raw)
        }
        None => get_config_file_path(),
    };
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(path)?;
    let parsed: TemplateConfig = toml::from_str(&raw)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    Ok(Some(parsed))
}
