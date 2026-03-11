use crate::models::color::Color;

pub fn enforce_hue_diversity(colors: Vec<Color>, min_distance: f32) -> Vec<Color> {
    if colors.len() < 2 {
        return colors;
    }

    let mut entries: Vec<(Color, f32, f32)> = colors
        .into_iter()
        .map(|color| {
            let (hue, saturation, _) = rgb_to_hsl(color);
            (color, hue, saturation)
        })
        .collect();

    entries.sort_by(|a, b| b.2.total_cmp(&a.2));

    let mut kept: Vec<(Color, f32, f32)> = Vec::new();
    for entry in entries {
        let too_close = kept
            .iter()
            .any(|(_, existing_hue, _)| hue_distance(entry.1, *existing_hue) < min_distance);
        if !too_close {
            kept.push(entry);
        }
    }

    kept.into_iter().map(|(color, _, _)| color).collect()
}

fn hue_distance(a: f32, b: f32) -> f32 {
    let diff = (a - b).abs();
    diff.min(360.0 - diff)
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
