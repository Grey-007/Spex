use std::error::Error;

use ::image::GenericImageView;

use crate::models::pixel::Pixel;

pub struct LoadedImage {
    pub original_width: u32,
    pub original_height: u32,
    pub processed_width: u32,
    pub processed_height: u32,
    pub pixels: Vec<Pixel>,
}

/// Loads an image from disk, resizes it to a bounded working image, and returns RGB pixels.
pub fn load_image(path: &str, max_dimension: u32) -> Result<LoadedImage, Box<dyn Error>> {
    let img = ::image::open(path)?;
    let (original_width, original_height) = img.dimensions();
    let processed = img.thumbnail(max_dimension, max_dimension);
    let (processed_width, processed_height) = processed.dimensions();
    let rgb_img = processed.to_rgb8();

    let pixels = rgb_img
        .pixels()
        .map(|pixel| Pixel {
            r: pixel[0],
            g: pixel[1],
            b: pixel[2],
        })
        .collect();

    Ok(LoadedImage {
        original_width,
        original_height,
        processed_width,
        processed_height,
        pixels,
    })
}
