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
    pub colors: Vec<Color>,
}

#[derive(Clone, Copy)]
struct ColorEntry {
    color: Color,
    luminance: f32,
    saturation: f32,
}

pub fn assign_roles(colors: Vec<Color>, theme: ThemeMode) -> ThemePalette {
    let mut entries = build_entries(colors);
    sort_by_theme_luminance(&mut entries, theme);
    let ordered_palette: Vec<Color> = entries.iter().map(|entry| entry.color).collect();

    while entries.len() < 8 {
        let next = entries[entries.len() % entries.len()];
        entries.push(next);
    }

    let background = entries[0];
    let surface = select_surface(&entries, background);

    let average_saturation =
        entries.iter().map(|entry| entry.saturation).sum::<f32>() / entries.len() as f32;
    let should_boost_saturation = average_saturation < 0.18;

    let primary = boost_if_grayscale(
        select_saturated_color(&entries, background, 0),
        should_boost_saturation,
    );
    let secondary = boost_if_grayscale(
        select_saturated_color(&entries, background, 1),
        should_boost_saturation,
    );
    let accent = boost_if_grayscale(
        select_saturated_color(&entries, background, 1),
        should_boost_saturation,
    );
    let accent2 = boost_if_grayscale(
        select_saturated_color(&entries, background, 2),
        should_boost_saturation,
    );
    let highlight = entries
        .iter()
        .copied()
        .max_by(|a, b| {
            contrast_against(*a, background).total_cmp(&contrast_against(*b, background))
        })
        .unwrap_or(background)
        .color;

    let text = if background.luminance_normalized() < 0.5 {
        entries
            .iter()
            .copied()
            .max_by(|a, b| a.luminance.total_cmp(&b.luminance))
            .unwrap_or(background)
            .color
    } else {
        entries
            .iter()
            .copied()
            .min_by(|a, b| a.luminance.total_cmp(&b.luminance))
            .unwrap_or(background)
            .color
    };

    ThemePalette {
        background: background.color,
        surface: surface.color,
        primary,
        secondary,
        accent,
        accent2,
        highlight,
        text,
        colors: ordered_palette,
    }
}

fn build_entries(colors: Vec<Color>) -> Vec<ColorEntry> {
    let palette = if colors.is_empty() {
        vec![Color { r: 0, g: 0, b: 0 }]
    } else {
        colors
    };

    palette
        .into_iter()
        .map(|color| ColorEntry {
            color,
            luminance: luminance(color),
            saturation: saturation(color),
        })
        .collect()
}

fn sort_by_theme_luminance(entries: &mut [ColorEntry], theme: ThemeMode) {
    entries.sort_by(|a, b| match theme {
        ThemeMode::Dark => a.luminance.total_cmp(&b.luminance),
        ThemeMode::Light => b.luminance.total_cmp(&a.luminance),
    });
}

fn select_surface(entries: &[ColorEntry], background: ColorEntry) -> ColorEntry {
    entries
        .iter()
        .copied()
        .skip(1)
        .min_by(|a, b| {
            background_distance(*a, background).total_cmp(&background_distance(*b, background))
        })
        .unwrap_or(background)
}

fn select_saturated_color(entries: &[ColorEntry], background: ColorEntry, rank: usize) -> Color {
    let mut candidates: Vec<ColorEntry> = entries
        .iter()
        .copied()
        .filter(|entry| background_distance(*entry, background) >= 32.0)
        .collect();

    if candidates.is_empty() {
        candidates = entries.to_vec();
    }

    candidates.sort_by(|a, b| {
        b.saturation.total_cmp(&a.saturation).then_with(|| {
            contrast_against(*b, background).total_cmp(&contrast_against(*a, background))
        })
    });

    candidates.get(rank).copied().unwrap_or(candidates[0]).color
}

fn background_distance(entry: ColorEntry, background: ColorEntry) -> f32 {
    (entry.luminance - background.luminance).abs()
}

fn contrast_against(entry: ColorEntry, background: ColorEntry) -> f32 {
    background_distance(entry, background) + entry.saturation * 32.0
}

fn boost_if_grayscale(color: Color, should_boost: bool) -> Color {
    if !should_boost {
        return color;
    }

    let (h, s, l) = rgb_to_hsl(color);
    let boosted = (s + 0.18).clamp(0.22, 0.55);
    hsl_to_rgb(h, boosted, l)
}

fn luminance(color: Color) -> f32 {
    0.2126 * color.r as f32 + 0.7152 * color.g as f32 + 0.0722 * color.b as f32
}

fn saturation(color: Color) -> f32 {
    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    if max == 0.0 { 0.0 } else { (max - min) / max }
}

impl ColorEntry {
    fn luminance_normalized(self) -> f32 {
        self.luminance / 255.0
    }
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

    let sat = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    } * 60.0;

    (hue, sat, lightness)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    if s == 0.0 {
        let gray = (l * 255.0).round() as u8;
        return Color {
            r: gray,
            g: gray,
            b: gray,
        };
    }

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = (h / 60.0).rem_euclid(6.0);
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = l - c / 2.0;
    Color {
        r: ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        g: ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        b: ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    }
}
