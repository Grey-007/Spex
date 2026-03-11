use std::env;
use std::path::Path;

use crate::template::config::TemplateConfig;

pub fn check_hooks(config: &TemplateConfig) -> usize {
    let mut issues = 0usize;

    if config.hooks.commands.is_empty() {
        println!("[OK] Hooks configured (none)");
        return issues;
    }

    if !shell_exists("sh") {
        println!("[ERROR] Shell executable not found: sh");
        issues += 1;
    }

    for (idx, command) in config.hooks.commands.iter().enumerate() {
        if command.trim().is_empty() {
            println!("[ERROR] Hook command is empty at index {idx}");
            issues += 1;
        }
    }

    if issues == 0 {
        println!("[OK] Hooks configured");
    }

    issues
}

fn shell_exists(name: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&paths).any(|entry| Path::new(&entry).join(name).exists())
}
