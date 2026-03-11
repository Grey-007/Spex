mod image;
mod models;

use std::env;
use std::error::Error;

use ::image::GenericImageView;

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

    println!("Image loaded");
    println!("Width: {width}");
    println!("Height: {height}");
    println!("Total pixel count: {}", pixels.len());

    Ok(())
}
