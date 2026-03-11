use crate::models::pixel::Pixel;

/// Returns a sampled subset by keeping every `stride`-th pixel.
pub fn sample_pixels(pixels: &[Pixel], stride: usize) -> Vec<Pixel> {
    let step = if stride == 0 { 1 } else { stride };

    pixels.iter().step_by(step).copied().collect()
}
