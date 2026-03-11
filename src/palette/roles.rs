use crate::models::color::Color;
use crate::models::theme::ThemeMode;

pub struct ThemePalette {
    pub background: Color,
    pub surface: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub accent2: Color,
    pub highlight: Color,
    pub text: Color,
}

pub fn assign_roles(colors: Vec<Color>, theme: ThemeMode) -> ThemePalette {
    let mut entries = if colors.is_empty() {
        vec![Color { r: 0, g: 0, b: 0 }]
    } else {
        colors
    };

    while entries.len() < 8 {
        let next = entries[entries.len() % entries.len()];
        entries.push(next);
    }

    let mut used = vec![false; entries.len()];
    let metrics: Vec<(f32, f32)> = entries
        .iter()
        .map(|color| {
            let luminance = luminance(*color);
            let (_, saturation, _) = rgb_to_hsl(*color);
            (luminance, saturation)
        })
        .collect();

    let background_idx = match theme {
        ThemeMode::Dark => arg_min(&metrics, |m| m.0),
        ThemeMode::Light => arg_max(&metrics, |m| m.0),
    };
    used[background_idx] = true;

    let background_l = metrics[background_idx].0;
    let surface_idx = arg_best_unused(&used, &metrics, |m| (m.0 - background_l).abs());
    used[surface_idx] = true;

    let primary_idx = arg_best_unused(&used, &metrics, |m| -m.1);
    used[primary_idx] = true;

    let secondary_idx = arg_best_unused(&used, &metrics, |m| -m.1);
    used[secondary_idx] = true;

    let accent_idx = arg_best_unused(&used, &metrics, |m| -(m.1 * 0.7 + (m.0 / 255.0) * 0.3));
    used[accent_idx] = true;

    let accent2_idx = arg_best_unused(&used, &metrics, |m| -(m.1 * 0.65 + (m.0 / 255.0) * 0.35));
    used[accent2_idx] = true;

    let highlight_idx = arg_best_unused(&used, &metrics, |m| -(m.1 * 0.4 + (m.0 / 255.0) * 0.6));
    used[highlight_idx] = true;

    let text_idx = arg_best_unused(&used, &metrics, |m| -(m.0 - background_l).abs());

    ThemePalette {
        background: entries[background_idx],
        surface: entries[surface_idx],
        primary: entries[primary_idx],
        secondary: entries[secondary_idx],
        accent: entries[accent_idx],
        accent2: entries[accent2_idx],
        highlight: entries[highlight_idx],
        text: entries[text_idx],
    }
}

fn arg_min<F>(items: &[(f32, f32)], key: F) -> usize
where
    F: Fn(&(f32, f32)) -> f32,
{
    let mut best_idx = 0usize;
    let mut best = key(&items[0]);
    for (idx, item) in items.iter().enumerate().skip(1) {
        let value = key(item);
        if value < best {
            best = value;
            best_idx = idx;
        }
    }
    best_idx
}

fn arg_max<F>(items: &[(f32, f32)], key: F) -> usize
where
    F: Fn(&(f32, f32)) -> f32,
{
    let mut best_idx = 0usize;
    let mut best = key(&items[0]);
    for (idx, item) in items.iter().enumerate().skip(1) {
        let value = key(item);
        if value > best {
            best = value;
            best_idx = idx;
        }
    }
    best_idx
}

fn arg_best_unused<F>(used: &[bool], items: &[(f32, f32)], key: F) -> usize
where
    F: Fn(&(f32, f32)) -> f32,
{
    let mut best_idx = used.iter().position(|taken| !taken).unwrap_or(0);
    let mut best = key(&items[best_idx]);

    for (idx, item) in items.iter().enumerate() {
        if used[idx] {
            continue;
        }

        let value = key(item);
        if value < best {
            best = value;
            best_idx = idx;
        }
    }

    best_idx
}

fn luminance(color: Color) -> f32 {
    0.2126 * color.r as f32 + 0.7152 * color.g as f32 + 0.0722 * color.b as f32
}

fn rgb_to_hsl(color: Color) -> (f32, f32, f32) {
    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let lightness = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, lightness);
    }

    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue_base = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };

    (hue_base * 60.0, saturation, lightness)
}
