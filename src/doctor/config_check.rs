use std::io;
use std::path::{Path, PathBuf};

use crate::template::config::{TemplateConfig, expand_tilde, get_config_file_path, load_config_from_path};

pub struct ConfigCheckResult {
    pub issues: usize,
    pub config: Option<TemplateConfig>,
}

pub fn check_config(config_override: Option<&Path>) -> io::Result<ConfigCheckResult> {
    let path = resolved_config_path(config_override);
    let display = display_with_tilde(&path);

    if !path.exists() {
        println!("[WARN] Config file not found: {display}");
        return Ok(ConfigCheckResult {
            issues: 1,
            config: None,
        });
    }

    match load_config_from_path(config_override) {
        Ok(Some(config)) => {
            println!("[OK] Config file found");
            Ok(ConfigCheckResult {
                issues: 0,
                config: Some(config),
            })
        }
        Ok(None) => {
            println!("[WARN] Config file not found: {display}");
            Ok(ConfigCheckResult {
                issues: 1,
                config: None,
            })
        }
        Err(err) => {
            println!("[ERROR] Failed to parse config");
            println!("File: {display}");
            println!("Reason: {err}");
            Ok(ConfigCheckResult {
                issues: 1,
                config: None,
            })
        }
    }
}

fn resolved_config_path(config_override: Option<&Path>) -> PathBuf {
    match config_override {
        Some(path) => expand_tilde(&path.to_string_lossy()),
        None => get_config_file_path(),
    }
}

fn display_with_tilde(path: &Path) -> String {
    let raw = path.to_string_lossy().to_string();
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() && raw.starts_with(&home) {
            return raw.replacen(&home, "~", 1);
        }
    }
    raw
}
