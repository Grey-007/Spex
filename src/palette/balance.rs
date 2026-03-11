use crate::models::color::Color;

const MIN_HUE_DISTANCE: f32 = 20.0;
const DARK_LUMINANCE_THRESHOLD: f32 = 0.2;

pub fn balance_palette(colors: Vec<Color>) -> Vec<Color> {
    if colors.is_empty() {
        return colors;
    }

    let mut entries: Vec<(Color, f32, f32, f32)> = colors
        .into_iter()
        .map(|color| {
            let (hue, saturation, luminance) = rgb_to_hsl(color);
            (color, hue, saturation, luminance)
        })
        .collect();

    entries.sort_by(|a, b| b.2.total_cmp(&a.2));

    let mut hue_filtered: Vec<(Color, f32, f32, f32)> = Vec::new();
    for entry in entries {
        let keep = hue_filtered.iter().all(|(_, existing_hue, _, _)| {
            hue_distance(entry.1, *existing_hue) >= MIN_HUE_DISTANCE
        });
        if keep {
            hue_filtered.push(entry);
        }
    }

    let dark_limit = ((hue_filtered.len() + 3) / 4).max(1);
    let mut dark_count = 0usize;
    let mut balanced = Vec::new();

    for (color, _, _, luminance) in hue_filtered {
        if luminance < DARK_LUMINANCE_THRESHOLD {
            if dark_count >= dark_limit {
                continue;
            }
            dark_count += 1;
        }

        balanced.push(color);
    }

    balanced
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
