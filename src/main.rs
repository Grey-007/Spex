mod cli;
mod color_utils;
mod color_engine;
mod doctor;
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
use crate::color_utils::{
    PaletteEnhancementDebug, delta_e, dominant_hue_hint_from_pixels, enhance_palette, luminance,
    saturation,
};
use crate::doctor::run_doctor;
use crate::export::css::export_css;
use crate::export::json::export_json;
use crate::export::terminal::export_terminal;
use crate::extract::mediancut::extract_palette_mediancut;
use crate::extract::sampler::sample_pixels;
use crate::image::loader::load_image;
use crate::models::color::Color;
use crate::models::theme::ThemeMode;
use crate::palette::balance::balance_palette;
use crate::palette::hue::enforce_hue_diversity;
use crate::palette::roles::{ThemePalette, assign_roles};
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
        Some(Commands::Doctor) => {
            let ok = run_doctor(cli.config.as_deref())?;
            if !ok {
                std::process::exit(1);
            }
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
    let dominant_hue_hint = dominant_hue_hint_from_pixels(&sampled_pixels);
    let (palette, enhancement_debug) = enhance_palette(palette, dominant_hue_hint);
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
        println!("Debug theme: {}", cli.debug_theme);
        println!("Debug colors: {}", cli.debug_colors);
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

    let theme_palette = assign_roles(palette.clone(), theme);
    println!("Theme palette:");
    println!("background: {}", to_hex(theme_palette.background));
    println!("surface:    {}", to_hex(theme_palette.surface));
    println!("surface_container: {}", to_hex(theme_palette.surface_container));
    println!("surface_high:      {}", to_hex(theme_palette.surface_high));
    println!("primary:    {}", to_hex(theme_palette.primary));
    println!("secondary:  {}", to_hex(theme_palette.secondary));
    println!("accent:     {}", to_hex(theme_palette.accent));
    println!("accent2:    {}", to_hex(theme_palette.accent2));
    println!("highlight:  {}", to_hex(theme_palette.highlight));
    println!("text:       {}", to_hex(theme_palette.text));

    if cli.debug_colors {
        print_color_debug(&palette, &theme_palette, enhancement_debug);
    }

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

        run_template_engine(
            &theme_palette,
            cli.dry_run,
            cli.debug_theme,
            cli.config.as_deref(),
        )?;
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

fn theme_name(theme: ThemeMode) -> &'static str {
    match theme {
        ThemeMode::Dark => "dark",
        ThemeMode::Light => "light",
    }
}

fn to_hex(color: crate::models::color::Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

fn print_color_debug(
    palette: &[Color],
    theme_palette: &ThemePalette,
    enhancement_debug: PaletteEnhancementDebug,
) {
    println!();
    println!("Color debug:");
    println!(
        "avg_saturation: {:.3} (threshold {:.3})",
        enhancement_debug.average_saturation, enhancement_debug.saturation_threshold
    );
    println!(
        "vibrancy_boost: {} (sat x{:.2}, contrast x{:.2})",
        enhancement_debug.vibrancy_boost_applied,
        enhancement_debug.saturation_factor,
        enhancement_debug.contrast_factor
    );
    println!(
        "grayscale_injection: {}",
        enhancement_debug.grayscale_injection_applied
    );
    if let Some(hue) = enhancement_debug.dominant_hue_hint {
        println!("dominant_hue_hint: {:.1} deg", hue);
    }

    println!();
    println!("Palette metrics:");
    for (idx, color) in palette.iter().enumerate() {
        println!(
            "color{idx}: {} lum={:.1} sat={:.3} dE(bg)={:.1}",
            to_hex(*color),
            luminance(*color),
            saturation(*color),
            delta_e(*color, theme_palette.background)
        );
    }

    println!();
    println!("Final roles:");
    print_role_debug("background", theme_palette.background, theme_palette.background, None);
    print_role_debug(
        "surface",
        theme_palette.surface,
        theme_palette.background,
        Some(theme_palette.primary),
    );
    print_role_debug(
        "surface_container",
        theme_palette.surface_container,
        theme_palette.background,
        Some(theme_palette.primary),
    );
    print_role_debug(
        "surface_high",
        theme_palette.surface_high,
        theme_palette.background,
        Some(theme_palette.primary),
    );
    print_role_debug("primary", theme_palette.primary, theme_palette.background, None);
    print_role_debug(
        "secondary",
        theme_palette.secondary,
        theme_palette.background,
        Some(theme_palette.primary),
    );
    print_role_debug(
        "accent",
        theme_palette.accent,
        theme_palette.background,
        Some(theme_palette.primary),
    );
    print_role_debug(
        "accent2",
        theme_palette.accent2,
        theme_palette.background,
        Some(theme_palette.primary),
    );
    print_role_debug(
        "highlight",
        theme_palette.highlight,
        theme_palette.background,
        Some(theme_palette.primary),
    );
    print_role_debug("text", theme_palette.text, theme_palette.background, None);
}

fn print_role_debug(name: &str, color: Color, background: Color, primary: Option<Color>) {
    let mut line = format!(
        "{name}: {} lum={:.1} sat={:.3} dE(bg)={:.1}",
        to_hex(color),
        luminance(color),
        saturation(color),
        delta_e(color, background)
    );

    if let Some(primary) = primary {
        line.push_str(&format!(" dE(primary)={:.1}", delta_e(color, primary)));
    }

    println!("{line}");
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
