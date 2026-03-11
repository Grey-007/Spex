use std::error::Error;

use crate::models::pixel::Pixel;

/// Loads an image from disk, converts it to RGB8, and returns all pixels.
pub fn load_image(path: &str) -> Result<Vec<Pixel>, Box<dyn Error>> {
    let img = ::image::open(path)?;
    let rgb_img = img.to_rgb8();

    let pixels = rgb_img
        .pixels()
        .map(|pixel| Pixel {
            r: pixel[0],
            g: pixel[1],
            b: pixel[2],
        })
        .collect();

    Ok(pixels)
}
