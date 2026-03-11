mod cli;
mod color_engine;
mod export;
mod extract;
mod image;
mod models;
mod palette;
mod preview;
mod template;

use std::error::Error;
use std::path::Path;

use ::image::GenericImageView;
use clap::Parser;

use crate::cli::{Cli, Commands, ExportArg, ThemeArg, print_completions};
use crate::export::css::export_css;
use crate::export::json::export_json;
use crate::export::terminal::export_terminal;
use crate::extract::mediancut::extract_palette_mediancut;
use crate::extract::sampler::sample_pixels;
use crate::image::loader::load_image;
use crate::models::theme::ThemeMode;
use crate::palette::balance::balance_palette;
use crate::palette::hue::enforce_hue_diversity;
use crate::palette::roles::assign_roles;
use crate::preview::terminal::print_palette;
use crate::template::config::{expand_tilde, get_config_file_path};
use crate::template::engine::run_template_engine;

#[derive(Debug, Clone, Copy)]
enum RunMode {
    Generate,
    PreviewOnly,
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    if cli.colors == 0 {
        return Err("Palette size must be at least 1".into());
    }

    match &cli.command {
        Some(Commands::Completions { shell }) => {
            print_completions(*shell)?;
            return Ok(());
        }
        Some(Commands::Config) => {
            print_config_info(cli.config.as_deref());
            return Ok(());
        }
        Some(Commands::Daemon) => {
            println!("Daemon mode is not implemented yet.");
            return Ok(());
        }
        Some(Commands::Generate { image }) => {
            return run_pipeline(image, &cli, RunMode::Generate);
        }
        Some(Commands::Preview { image }) => {
            return run_pipeline(image, &cli, RunMode::PreviewOnly);
        }
        None => {}
    }

    if let Some(image) = &cli.image {
        run_pipeline(image, &cli, RunMode::Generate)
    } else {
        Err("No image path provided. Use `spex --help`.".into())
    }
}

fn run_pipeline(image_path: &Path, cli: &Cli, mode: RunMode) -> Result<(), Box<dyn Error>> {
    let theme = map_theme(cli.theme);

    // Open once for metadata that we print in this phase.
    let img = ::image::open(image_path)?;
    let (width, height) = img.dimensions();

    let image_str = image_path.to_string_lossy();
    let pixels = load_image(&image_str)?;
    let sampled_pixels = sample_pixels(&pixels, 100);
    let base_palette = extract_palette_mediancut(&sampled_pixels, cli.colors);
    let balanced_palette = balance_palette(base_palette, cli.colors);
    let mut palette = enforce_hue_diversity(balanced_palette.clone(), 20.0);
    palette = refill_palette(palette, &balanced_palette, cli.colors);
    let palette = order_palette_for_theme(palette, theme);

    println!("Image loaded");
    println!("Width: {width}");
    println!("Height: {height}");
    println!("Original pixel count: {}", pixels.len());
    println!("Sampled pixel count: {}", sampled_pixels.len());
    println!();
    println!("Extracting palette ({} colors)...", cli.colors);
    println!("Theme: {}", theme_name(theme));
    if cli.verbose {
        println!("Mode: {:?}", mode);
        println!("Dry run: {}", cli.dry_run);
        println!("No preview: {}", cli.no_preview);
        if let Some(export) = cli.export {
            println!("Export format: {}", export_name(export));
        }
    }
    println!();

    if !cli.no_preview {
        print_palette(&palette);
        println!();
    }

    let theme_palette = assign_roles(palette, theme);
    println!("Theme palette:");
    println!("background: {}", to_hex(theme_palette.background));
    println!("surface:    {}", to_hex(theme_palette.surface));
    println!("primary:    {}", to_hex(theme_palette.primary));
    println!("secondary:  {}", to_hex(theme_palette.secondary));
    println!("accent:     {}", to_hex(theme_palette.accent));
    println!("accent2:    {}", to_hex(theme_palette.accent2));
    println!("highlight:  {}", to_hex(theme_palette.highlight));
    println!("text:       {}", to_hex(theme_palette.text));

    if matches!(mode, RunMode::Generate) {
        if let Some(format) = cli.export {
            let path = match format {
                ExportArg::Json => export_json(&theme_palette)?,
                ExportArg::Css => export_css(&theme_palette)?,
                ExportArg::Terminal => export_terminal(&theme_palette)?,
            };

            println!();
            println!("Exported palette to:");
            println!("{}", path.display());
        }

        run_template_engine(&theme_palette, cli.dry_run, cli.config.as_deref())?;
    }

    Ok(())
}

fn print_config_info(config_override: Option<&Path>) {
    let default_path = get_config_file_path();
    println!("Default config path:");
    println!("{}", default_path.display());

    if let Some(path) = config_override {
        println!();
        println!("Override config path:");
        println!("{}", expand_tilde(&path.to_string_lossy()).display());
    }
}

fn map_theme(theme: ThemeArg) -> ThemeMode {
    match theme {
        ThemeArg::Dark => ThemeMode::Dark,
        ThemeArg::Light => ThemeMode::Light,
    }
}

fn export_name(export: ExportArg) -> &'static str {
    match export {
        ExportArg::Json => "json",
        ExportArg::Css => "css",
        ExportArg::Terminal => "terminal",
    }
}

fn order_palette_for_theme(
    mut colors: Vec<crate::models::color::Color>,
    theme: ThemeMode,
) -> Vec<crate::models::color::Color> {
    colors.sort_by(|a, b| {
        let a_l = luminance(*a);
        let b_l = luminance(*b);
        match theme {
            ThemeMode::Dark => a_l.total_cmp(&b_l),
            ThemeMode::Light => b_l.total_cmp(&a_l),
        }
    });
    colors
}

fn luminance(color: crate::models::color::Color) -> f32 {
    0.2126 * color.r as f32 + 0.7152 * color.g as f32 + 0.0722 * color.b as f32
}

fn theme_name(theme: ThemeMode) -> &'static str {
    match theme {
        ThemeMode::Dark => "dark",
        ThemeMode::Light => "light",
    }
}

fn to_hex(color: crate::models::color::Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

fn refill_palette(
    mut filtered: Vec<crate::models::color::Color>,
    fallback: &[crate::models::color::Color],
    k: usize,
) -> Vec<crate::models::color::Color> {
    if filtered.len() >= k {
        filtered.truncate(k);
        return filtered;
    }

    for color in fallback {
        if filtered.len() >= k {
            break;
        }

        if !filtered.contains(color) {
            filtered.push(*color);
        }
    }

    if filtered.is_empty() {
        return filtered;
    }

    let mut idx = 0usize;
    while filtered.len() < k {
        let color = filtered[idx % filtered.len()];
        filtered.push(color);
        idx += 1;
    }

    filtered
}
