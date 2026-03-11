mod extract;
mod image;
mod models;
mod palette;
mod preview;

use std::env;
use std::error::Error;

use ::image::GenericImageView;

use crate::extract::mediancut::extract_palette_mediancut;
use crate::extract::sampler::sample_pixels;
use crate::image::loader::load_image;
use crate::models::theme::ThemeMode;
use crate::palette::balance::balance_palette;
use crate::palette::hue::enforce_hue_diversity;
use crate::preview::terminal::print_palette;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let (path, colors, theme) = parse_args(env::args().skip(1))?;

    // Open once for metadata that we print in this phase.
    let img = ::image::open(&path)?;
    let (width, height) = img.dimensions();

    let pixels = load_image(&path)?;
    let sampled_pixels = sample_pixels(&pixels, 100);
    let base_palette = extract_palette_mediancut(&sampled_pixels, colors);
    let balanced_palette = balance_palette(base_palette, colors);
    let mut palette = enforce_hue_diversity(balanced_palette.clone(), 20.0);
    palette = refill_palette(palette, &balanced_palette, colors);
    let palette = order_palette_for_theme(palette, theme);

    println!("Image loaded");
    println!("Width: {width}");
    println!("Height: {height}");
    println!("Original pixel count: {}", pixels.len());
    println!("Sampled pixel count: {}", sampled_pixels.len());
    println!();
    println!("Extracting palette ({colors} colors)...");
    println!("Theme: {}", theme_name(theme));
    println!();
    print_palette(&palette);

    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<(String, usize, ThemeMode), Box<dyn Error>>
where
    I: Iterator<Item = String>,
{
    let path = args
        .next()
        .ok_or("Usage: spex <image_path> [--colors <number>] [--theme <dark|light>]")?;

    let mut colors = 8usize;
    let mut theme = ThemeMode::Dark;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--colors" => {
                let value = args
                    .next()
                    .ok_or("Missing value for --colors. Usage: --colors <number>")?;
                colors = value.parse::<usize>()?;
                if colors == 0 {
                    return Err("Palette size must be at least 1".into());
                }
            }
            "--theme" => {
                let value = args
                    .next()
                    .ok_or("Missing value for --theme. Usage: --theme <dark|light>")?;
                theme = parse_theme(&value)?;
            }
            _ => {
                return Err(format!("Unknown argument: {arg}").into());
            }
        }
    }

    Ok((path, colors, theme))
}

fn parse_theme(value: &str) -> Result<ThemeMode, Box<dyn Error>> {
    match value {
        "dark" => Ok(ThemeMode::Dark),
        "light" => Ok(ThemeMode::Light),
        _ => Err(format!("Invalid theme '{value}'. Expected: dark or light").into()),
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
