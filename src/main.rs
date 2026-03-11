mod extract;
mod image;
mod models;
mod palette;

use std::env;
use std::error::Error;

use ::image::GenericImageView;

use crate::extract::kmeans::extract_palette_with_sizes;
use crate::extract::sampler::sample_pixels;
use crate::image::loader::load_image;
use crate::palette::filter::filter_palette;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let (path, colors) = parse_args(env::args().skip(1))?;

    // Open once for metadata that we print in this phase.
    let img = ::image::open(&path)?;
    let (width, height) = img.dimensions();

    let pixels = load_image(&path)?;
    let sampled_pixels = sample_pixels(&pixels, 100);
    let (palette, cluster_sizes) = extract_palette_with_sizes(&sampled_pixels, colors);
    let palette = filter_palette(palette, cluster_sizes);

    println!("Image loaded");
    println!("Width: {width}");
    println!("Height: {height}");
    println!("Original pixel count: {}", pixels.len());
    println!("Sampled pixel count: {}", sampled_pixels.len());
    println!();
    println!("Extracting palette ({colors} colors)...");
    println!();
    println!("Dominant colors:");
    for color in &palette {
        println!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
    }

    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<(String, usize), Box<dyn Error>>
where
    I: Iterator<Item = String>,
{
    let path = args
        .next()
        .ok_or("Usage: spex <image_path> [--colors <number>]")?;

    let mut colors = 8usize;
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
            _ => {
                return Err(format!("Unknown argument: {arg}").into());
            }
        }
    }

    Ok((path, colors))
}
