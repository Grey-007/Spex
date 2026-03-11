use crate::models::color::Color;

const MIN_SATURATION_THRESHOLD: f32 = 0.05;

pub fn balance_palette(colors: Vec<Color>, k: usize) -> Vec<Color> {
    if colors.is_empty() || k == 0 {
        return Vec::new();
    }

    let original = colors.clone();
    let mut scored: Vec<(Color, f32)> = colors
        .into_iter()
        .map(|color| {
            let (_, saturation, luminance) = rgb_to_hsl(color);
            let score = saturation * 0.7 + luminance * 0.3;
            (color, score)
        })
        .filter(|(color, _)| {
            let (_, saturation, _) = rgb_to_hsl(*color);
            saturation >= MIN_SATURATION_THRESHOLD
        })
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));

    if scored.len() < k {
        return fill_to_k(original, k);
    }

    scored.into_iter().take(k).map(|(color, _)| color).collect()
}

fn rgb_to_hsl(color: Color) -> (f32, f32, f32) {
    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let luminance = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, luminance);
    }

    let saturation = delta / (1.0 - (2.0 * luminance - 1.0).abs());

    let hue_base = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };

    let hue = hue_base * 60.0;
    (hue, saturation, luminance)
}

fn fill_to_k(mut colors: Vec<Color>, k: usize) -> Vec<Color> {
    if colors.is_empty() {
        return colors;
    }

    if colors.len() >= k {
        colors.truncate(k);
        return colors;
    }

    let base = colors.clone();
    let mut idx = 0usize;
    while colors.len() < k {
        colors.push(base[idx % base.len()]);
        idx += 1;
    }

    colors
}
