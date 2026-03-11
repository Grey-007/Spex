use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use crate::color_engine::engine::{build_tokens, infer_theme_from_palette};
use crate::color_engine::format::resolve_token_path;
use crate::palette::roles::assign_roles;
use crate::template::config::{TemplateConfig, expand_tilde};
use crate::template::renderer::render;

use super::engine_check::mock_palette;

pub struct TemplateCheckResult {
    pub issues: usize,
}

pub fn check_templates(config: &TemplateConfig) -> io::Result<TemplateCheckResult> {
    let mut issues = 0usize;
    let mut files = HashSet::new();

    let dir_issues = check_template_dirs(config);
    issues += dir_issues;

    let file_issues = collect_and_check_template_files(config, &mut files)?;
    issues += file_issues;

    let syntax_issues = check_template_syntax(&files)?;
    issues += syntax_issues;

    let render_issues = check_template_rendering(&files)?;
    issues += render_issues;

    Ok(TemplateCheckResult { issues })
}

fn check_template_dirs(config: &TemplateConfig) -> usize {
    let mut issues = 0usize;

    for dir in &config.template_dirs.paths {
        let path = expand_tilde(dir);
        if !path.exists() {
            println!("[ERROR] Template directory missing: {}", path.display());
            issues += 1;
            continue;
        }

        if std::fs::read_dir(&path).is_err() {
            println!("[ERROR] Template directory not readable: {}", path.display());
            issues += 1;
        }
    }

    if issues == 0 {
        println!("[OK] Template directories valid");
    }

    issues
}

fn collect_and_check_template_files(
    config: &TemplateConfig,
    files: &mut HashSet<PathBuf>,
) -> io::Result<usize> {
    let mut issues = 0usize;

    for template in &config.template {
        let path = resolve_template_input(config, &template.input);
        if !path.exists() {
            println!("[ERROR] Template file not found: {}", path.display());
            issues += 1;
            continue;
        }

        if std::fs::File::open(&path).is_err() {
            println!("[ERROR] Template file not readable: {}", path.display());
            issues += 1;
            continue;
        }

        files.insert(path);
    }

    for dir in &config.template_dirs.paths {
        let dir_path = expand_tilde(dir);
        if !dir_path.exists() {
            continue;
        }

        for entry in std::fs::read_dir(dir_path)? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }

            if std::fs::File::open(&path).is_err() {
                println!("[ERROR] Template file not readable: {}", path.display());
                issues += 1;
                continue;
            }

            files.insert(path);
        }
    }

    if issues == 0 {
        println!("[OK] Template files valid");
    }

    Ok(issues)
}

fn check_template_syntax(files: &HashSet<PathBuf>) -> io::Result<usize> {
    let mut issues = 0usize;

    let palette = mock_palette();
    let theme = infer_theme_from_palette(&palette);
    let tokens = build_tokens(palette, theme);
    let mut known_roles: Vec<&str> = tokens.colors.keys().map(|s| s.as_str()).collect();
    known_roles.sort_unstable();

    for file in files {
        let text = std::fs::read_to_string(file)?;
        for (line_idx, line) in text.lines().enumerate() {
            for token in extract_tokens(line) {
                if is_special_token(token) {
                    continue;
                }

                if token.starts_with("colors.") && resolve_token_path(&tokens, token).is_none() {
                    issues += 1;
                    let unknown_role =
                        extract_role_name(token).unwrap_or("unknown");
                    println!("[ERROR] Invalid template token");
                    println!();
                    println!("File: {}", file.display());
                    println!("Line: {}", line_idx + 1);
                    println!();
                    println!("Unknown token: {unknown_role}");
                    let suggestions = suggest_roles(unknown_role, &known_roles);
                    if !suggestions.is_empty() {
                        println!();
                        println!("Did you mean:");
                        for suggestion in suggestions {
                            println!("{suggestion}");
                        }
                    }
                    println!();
                }
            }
        }
    }

    if issues == 0 {
        println!("[OK] Template syntax valid");
    }

    Ok(issues)
}

fn check_template_rendering(files: &HashSet<PathBuf>) -> io::Result<usize> {
    let mut issues = 0usize;
    let palette = mock_palette();
    let theme_palette = assign_roles(palette, crate::models::theme::ThemeMode::Dark);

    for file in files {
        let text = std::fs::read_to_string(file)?;
        let rendered = std::panic::catch_unwind(|| render(&text, &theme_palette));
        if rendered.is_err() {
            println!("[ERROR] Template rendering failed");
            println!("File: {}", file.display());
            issues += 1;
        }
    }

    if issues == 0 {
        println!("[OK] Template rendering test passed");
    }

    Ok(issues)
}

fn resolve_template_input(config: &TemplateConfig, input: &str) -> PathBuf {
    let expanded = expand_tilde(input);
    if expanded.exists() {
        return expanded;
    }

    for dir in &config.template_dirs.paths {
        let candidate = expand_tilde(dir).join(input);
        if candidate.exists() {
            return candidate;
        }
    }

    expanded
}

fn is_special_token(token: &str) -> bool {
    matches!(token, "#colors" | "/colors" | "index" | "value")
}

fn extract_tokens(line: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut cursor = 0usize;

    while let Some(start_rel) = line[cursor..].find("{{") {
        let start = cursor + start_rel + 2;
        if let Some(end_rel) = line[start..].find("}}") {
            let end = start + end_rel;
            tokens.push(line[start..end].trim());
            cursor = end + 2;
        } else {
            break;
        }
    }

    tokens
}

fn extract_role_name(token: &str) -> Option<&str> {
    let path = token.strip_prefix("colors.")?;
    if let Some((role, _)) = path.rsplit_once(".default.") {
        return Some(role);
    }
    path.rsplit_once('.').map(|(role, _)| role)
}

fn suggest_roles<'a>(role: &str, known_roles: &[&'a str]) -> Vec<&'a str> {
    let mut scored: Vec<(&str, usize)> = known_roles
        .iter()
        .map(|candidate| (*candidate, levenshtein(role, candidate)))
        .collect();

    scored.sort_by_key(|(_, distance)| *distance);
    scored
        .into_iter()
        .take(2)
        .map(|(candidate, _)| candidate)
        .collect()
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; b_chars.len() + 1]; a_chars.len() + 1];

    for (i, row) in dp.iter_mut().enumerate().take(a_chars.len() + 1) {
        row[0] = i;
    }
    for j in 0..=b_chars.len() {
        dp[0][j] = j;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[a_chars.len()][b_chars.len()]
}
