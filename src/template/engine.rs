use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::palette::roles::ThemePalette;

use super::config::{
    TemplateConfig, expand_tilde, get_spex_config_directory, load_config_from_path,
};
use super::renderer::render;

struct RenderJob {
    input: PathBuf,
    output: PathBuf,
}

pub fn run_template_engine(
    palette: &ThemePalette,
    dry_run: bool,
    debug_theme: bool,
    config_override: Option<&Path>,
) -> Result<(), Box<dyn Error>> {
    let Some(config) = load_config_from_path(config_override)? else {
        return Ok(());
    };

    let jobs = collect_jobs(&config)?;
    for job in &jobs {
        let raw_template = std::fs::read_to_string(&job.input)?;
        let rendered = render(&raw_template, palette, debug_theme);

        if dry_run {
            println!();
            println!("Dry run template output: {}", job.output.display());
            println!("{rendered}");
        } else {
            if let Some(parent) = job.output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&job.output, rendered)?;
            println!("Rendered template: {}", job.output.display());
        }
    }

    if !dry_run {
        run_hooks(&config)?;
    }

    Ok(())
}

fn collect_jobs(config: &TemplateConfig) -> io::Result<Vec<RenderJob>> {
    let mut jobs = Vec::new();
    let mut seen_inputs: HashSet<PathBuf> = HashSet::new();

    for entry in &config.template {
        let input = resolve_input_path(&entry.input, &config.template_dirs.paths)?;
        let output = expand_tilde(&entry.output);
        seen_inputs.insert(input.clone());
        jobs.push(RenderJob { input, output });
    }

    for dir in &config.template_dirs.paths {
        let dir_path = expand_tilde(dir);
        if !dir_path.exists() {
            continue;
        }

        for entry in std::fs::read_dir(&dir_path)? {
            let path = entry?.path();
            if !path.is_file() || seen_inputs.contains(&path) {
                continue;
            }

            let output = generated_output_path(&path);
            jobs.push(RenderJob {
                input: path.clone(),
                output,
            });
        }
    }

    Ok(jobs)
}

fn resolve_input_path(input: &str, template_dirs: &[String]) -> io::Result<PathBuf> {
    let expanded = expand_tilde(input);
    if expanded.exists() {
        return Ok(expanded);
    }

    for dir in template_dirs {
        let candidate = expand_tilde(dir).join(input);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Template input not found: {input}"),
    ))
}

fn generated_output_path(input: &Path) -> PathBuf {
    let fallback = "template.out";
    let file_name = input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(fallback);

    get_spex_config_directory()
        .join("export")
        .join("generated")
        .join(file_name)
}

fn run_hooks(config: &TemplateConfig) -> io::Result<()> {
    let mut failures = 0usize;

    for command in &config.hooks.commands {
        match Command::new("sh").arg("-c").arg(command).status() {
            Ok(status) if status.success() => {}
            Ok(status) => {
                failures += 1;
                eprintln!(
                    "[WARN] Hook command failed (status {:?}): {}",
                    status.code(),
                    command
                );
            }
            Err(err) => {
                failures += 1;
                eprintln!("[WARN] Hook command failed to start: {} ({err})", command);
            }
        }
    }

    if failures > 0 {
        eprintln!("[WARN] Completed hooks with {failures} failure(s).");
    }

    Ok(())
}
