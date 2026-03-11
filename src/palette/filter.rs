use crate::models::color::Color;

/// Filters and ranks palette colors by vibrancy.
///
/// Vibrancy score:
/// `score = cluster_size * saturation`
/// where saturation comes from HSL (0.0..=1.0).
#[allow(dead_code)]
pub fn filter_palette(colors: Vec<Color>, cluster_sizes: Vec<usize>) -> Vec<Color> {
    let mut ranked: Vec<(Color, f32)> = colors
        .into_iter()
        .zip(cluster_sizes)
        .filter_map(|(color, cluster_size)| {
            let saturation = rgb_saturation(color);
            if saturation < 0.15 {
                return None;
            }

            let score = cluster_size as f32 * saturation;
            Some((color, score))
        })
        .collect();

    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked.into_iter().map(|(color, _)| color).collect()
}

#[allow(dead_code)]
fn rgb_saturation(color: Color) -> f32 {
    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta == 0.0 {
        return 0.0;
    }

    let lightness = (max + min) / 2.0;
    delta / (1.0 - (2.0 * lightness - 1.0).abs())
}
