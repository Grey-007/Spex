mod extract;
mod image;
mod models;

use std::env;
use std::error::Error;

use ::image::GenericImageView;

use crate::extract::kmeans::extract_palette;
use crate::extract::sampler::sample_pixels;
use crate::image::loader::load_image;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).ok_or("Usage: spex <image_path>")?;

    // Open once for metadata that we print in this phase.
    let img = ::image::open(&path)?;
    let (width, height) = img.dimensions();

    let pixels = load_image(&path)?;
    let sampled_pixels = sample_pixels(&pixels, 100);
    let palette = extract_palette(&sampled_pixels, 8);

    println!("Image loaded");
    println!("Width: {width}");
    println!("Height: {height}");
    println!("Original pixel count: {}", pixels.len());
    println!("Sampled pixel count: {}", sampled_pixels.len());
    println!("Dominant colors:");
    for color in &palette {
        println!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
    }

    Ok(())
}
