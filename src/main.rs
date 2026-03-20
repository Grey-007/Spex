mod cli;
mod color_engine;
mod color_utils;
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

use clap::Parser;

use crate::cli::{Cli, Commands, ExportArg, ExtractorArg, ThemeArg, print_completions};
use crate::color_utils::{
    PaletteEnhancementDebug, contrast_ratio, delta_e, luminance, rgb_to_lab, saturation,
};
use crate::doctor::run_doctor;
use crate::export::css::export_css;
use crate::export::json::export_json;
use crate::export::terminal::export_terminal;
use crate::extract::pipeline::{ExtractionOutcome, ExtractorMethod, extract_palette_with_fallback};
use crate::extract::sampler::sample_pixels;
use crate::image::loader::load_image;
use crate::models::color::Color;
use crate::models::theme::ThemeMode;
use crate::palette::roles::{ThemePalette, assign_roles};
use crate::preview::terminal::print_palette;
use crate::template::config::{expand_tilde, get_config_file_path};
use crate::template::engine::run_template_engine;

#[derive(Debug, Clone, Copy)]
enum RunMode {
    Generate,
    PreviewOnly,
}

const MAX_IMAGE_DIMENSION: u32 = 512;
const TARGET_SAMPLED_PIXELS: usize = 40_000;

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

    let image_str = image_path.to_string_lossy();
    let loaded_image = load_image(&image_str, MAX_IMAGE_DIMENSION)?;
    let sample_stride = recommended_sample_stride(loaded_image.pixels.len());
    let sampled_pixels = sample_pixels(&loaded_image.pixels, sample_stride);
    let extraction =
        extract_palette_with_fallback(&sampled_pixels, cli.colors, map_extractor(cli.extractor));
    let palette = extraction.palette.clone();

    println!("Image loaded");
    println!("Width: {}", loaded_image.original_width);
    println!("Height: {}", loaded_image.original_height);
    println!(
        "Processed size: {}x{}",
        loaded_image.processed_width, loaded_image.processed_height
    );
    println!(
        "Original pixel count: {}",
        loaded_image.original_width as u64 * loaded_image.original_height as u64
    );
    println!("Processed pixel count: {}", loaded_image.pixels.len());
    println!("Sampled pixel count: {}", sampled_pixels.len());
    println!();
    println!("Extracting palette ({} colors)...", cli.colors);
    println!("Theme: {}", theme_name(theme));
    println!("Extractor: {}", extraction.report.final_method.as_str());
    if extraction.report.fallback_triggered {
        println!(
            "Extractor fallback: {} -> {}",
            extraction.report.requested_method.as_str(),
            extraction.report.final_method.as_str()
        );
    }
    if cli.verbose {
        println!("Mode: {:?}", mode);
        println!("Dry run: {}", cli.dry_run);
        println!(
            "Requested extractor: {}",
            map_extractor(cli.extractor).as_str()
        );
        println!("Debug theme: {}", cli.debug_theme);
        println!("Debug colors: {}", cli.debug_colors);
        println!("Debug palette: {}", cli.debug_palette);
        println!("Debug extractor: {}", cli.debug_extractor);
        println!("No preview: {}", cli.no_preview);
        println!("Sample stride: {sample_stride}");
        if let Some(export) = cli.export {
            println!("Export format: {}", export_name(export));
        }
    }
    println!();

    if !cli.no_preview {
        print_palette(&palette);
        println!();
    }

    if cli.debug_extractor {
        print_extractor_debug(&extraction, sample_stride);
    }

    if cli.debug_palette {
        print_palette_debug(&palette, &extraction);
    }

    let theme_palette = assign_roles(palette.clone(), theme);
    println!("Theme palette:");
    println!("background: {}", to_hex(theme_palette.background));
    println!("surface:    {}", to_hex(theme_palette.surface));
    println!(
        "surface_container: {}",
        to_hex(theme_palette.surface_container)
    );
    println!("surface_high:      {}", to_hex(theme_palette.surface_high));
    println!("primary:    {}", to_hex(theme_palette.primary));
    println!("secondary:  {}", to_hex(theme_palette.secondary));
    println!("accent:     {}", to_hex(theme_palette.accent));
    println!("accent2:    {}", to_hex(theme_palette.accent2));
    println!("highlight:  {}", to_hex(theme_palette.highlight));
    println!("text:       {}", to_hex(theme_palette.text));

    if cli.debug_theme {
        print_theme_debug(&theme_palette, theme);
    }

    if cli.debug_colors {
        print_color_debug(&palette, &theme_palette, extraction.report.enhancement);
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

fn map_extractor(extractor: ExtractorArg) -> ExtractorMethod {
    match extractor {
        ExtractorArg::Kmeans => ExtractorMethod::Kmeans,
        ExtractorArg::Mediancut => ExtractorMethod::Mediancut,
    }
}

fn recommended_sample_stride(pixel_count: usize) -> usize {
    if pixel_count <= TARGET_SAMPLED_PIXELS {
        1
    } else {
        pixel_count.div_ceil(TARGET_SAMPLED_PIXELS)
    }
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
    print_role_debug(
        "background",
        theme_palette.background,
        theme_palette.background,
        None,
    );
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
    print_role_debug(
        "primary",
        theme_palette.primary,
        theme_palette.background,
        None,
    );
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
    let lab = rgb_to_lab(color);
    let mut line = format!(
        "{name}: {} lab=({:.1}, {:.1}, {:.1}) lum={:.1} sat={:.3} contrast(bg)={:.2} dE(bg)={:.1}",
        to_hex(color),
        lab.l,
        lab.a,
        lab.b,
        luminance(color),
        saturation(color),
        contrast_ratio(color, background),
        delta_e(color, background)
    );

    if let Some(primary) = primary {
        line.push_str(&format!(" dE(primary)={:.1}", delta_e(color, primary)));
    }

    println!("{line}");
}

fn print_theme_debug(theme_palette: &ThemePalette, theme: ThemeMode) {
    println!();
    println!("Theme debug:");
    print_theme_role_line(
        "background",
        theme_palette.background,
        theme_palette.background,
        theme,
    );
    print_theme_role_line(
        "surface",
        theme_palette.surface,
        theme_palette.background,
        theme,
    );
    print_theme_role_line(
        "surface_container",
        theme_palette.surface_container,
        theme_palette.background,
        theme,
    );
    print_theme_role_line(
        "surface_high",
        theme_palette.surface_high,
        theme_palette.background,
        theme,
    );
    print_theme_role_line(
        "primary",
        theme_palette.primary,
        theme_palette.background,
        theme,
    );
    print_theme_role_line(
        "secondary",
        theme_palette.secondary,
        theme_palette.background,
        theme,
    );
    print_theme_role_line(
        "accent",
        theme_palette.accent,
        theme_palette.background,
        theme,
    );
    print_theme_role_line(
        "accent2",
        theme_palette.accent2,
        theme_palette.background,
        theme,
    );
    print_theme_role_line(
        "highlight",
        theme_palette.highlight,
        theme_palette.background,
        theme,
    );
    print_theme_role_line("text", theme_palette.text, theme_palette.background, theme);

    println!();
    println!("Role Delta-E:");
    let role_pairs = [
        ("background", theme_palette.background),
        ("surface", theme_palette.surface),
        ("primary", theme_palette.primary),
        ("secondary", theme_palette.secondary),
        ("accent", theme_palette.accent),
        ("accent2", theme_palette.accent2),
        ("highlight", theme_palette.highlight),
        ("text", theme_palette.text),
    ];

    for left in 0..role_pairs.len() {
        for right in (left + 1)..role_pairs.len() {
            println!(
                "{} <-> {} = {:.1}",
                role_pairs[left].0,
                role_pairs[right].0,
                delta_e(role_pairs[left].1, role_pairs[right].1)
            );
        }
    }
}

fn print_theme_role_line(name: &str, color: Color, background: Color, theme: ThemeMode) {
    let lab = rgb_to_lab(color);
    let theme_depth = match theme {
        ThemeMode::Dark => luminance(color),
        ThemeMode::Light => 255.0 - luminance(color),
    };

    println!(
        "{name}: {} lab=({:.1}, {:.1}, {:.1}) lum={:.1} sat={:.3} theme_depth={:.1} contrast(bg)={:.2} dE(bg)={:.1}",
        to_hex(color),
        lab.l,
        lab.a,
        lab.b,
        luminance(color),
        saturation(color),
        theme_depth,
        contrast_ratio(color, background),
        delta_e(color, background),
    );
}

fn print_extractor_debug(extraction: &ExtractionOutcome, sample_stride: usize) {
    println!();
    println!("Extractor debug:");
    println!(
        "requested_extractor: {}",
        extraction.report.requested_method.as_str()
    );
    println!(
        "final_extractor: {}",
        extraction.report.final_method.as_str()
    );
    println!(
        "fallback_triggered: {}",
        extraction.report.fallback_triggered
    );
    println!("sample_stride: {sample_stride}");
    println!(
        "avg_saturation: {:.3}",
        extraction.report.quality.average_saturation
    );
    println!(
        "low_distance_pairs: {}",
        extraction.report.quality.low_distance_pairs
    );
    if let Some(min_delta) = extraction.report.quality.min_delta_e {
        println!("min_delta_e: {:.2}", min_delta);
    }
    println!("low_quality: {}", extraction.report.quality.low_quality);
    println!(
        "vibrancy_boost: {}",
        extraction.report.enhancement.vibrancy_boost_applied
    );
    println!(
        "grayscale_injection: {}",
        extraction.report.enhancement.grayscale_injection_applied
    );

    if extraction.report.kmeans_clusters.is_empty() {
        println!("No LAB cluster report available for this run.");
        return;
    }

    println!();
    println!("LAB clusters:");
    for (idx, cluster) in extraction.report.kmeans_clusters.iter().enumerate() {
        println!(
            "cluster{idx}: size={} lab=({:.2}, {:.2}, {:.2}) rgb={}",
            cluster.size,
            cluster.lab.l,
            cluster.lab.a,
            cluster.lab.b,
            to_hex(cluster.color)
        );
    }
}

fn print_palette_debug(palette: &[Color], extraction: &ExtractionOutcome) {
    println!();
    println!("Palette debug:");
    println!(
        "avg_saturation: {:.3}",
        extraction.report.quality.average_saturation
    );
    println!(
        "low_distance_pairs: {}",
        extraction.report.quality.low_distance_pairs
    );
    if let Some(min_delta) = extraction.report.quality.min_delta_e {
        println!("min_delta_e: {:.2}", min_delta);
    }

    for (idx, color) in palette.iter().enumerate() {
        let nearest_delta = palette
            .iter()
            .enumerate()
            .filter(|(other_idx, _)| *other_idx != idx)
            .map(|(_, other)| delta_e(*color, *other))
            .min_by(|a, b| a.total_cmp(b));

        println!(
            "color{idx}: {} lum={:.1} sat={:.3} nearest_dE={:.2}",
            to_hex(*color),
            luminance(*color),
            saturation(*color),
            nearest_delta.unwrap_or(0.0)
        );
    }
}
