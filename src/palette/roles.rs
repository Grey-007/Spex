use crate::color_utils::{delta_e, layer_background, luminance, saturation, tint_background};
use crate::models::color::Color;
use crate::models::theme::ThemeMode;

pub struct ThemePalette {
    pub background: Color,
    pub surface: Color,
    pub surface_container: Color,
    pub surface_high: Color,
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

    let mut used = vec![false; entries.len()];

    let background_idx = 0;
    used[background_idx] = true;
    let background = tint_background(entries[background_idx].color, theme);
    let surface = layer_background(background, theme, 8.0);
    let surface_container = layer_background(background, theme, 12.0);
    let surface_high = layer_background(background, theme, 18.0);

    let ranked_saturation = saturation_ranked_indices(&entries);
    let primary_idx = select_primary_index(&entries, &ranked_saturation, &used, background)
        .or_else(|| select_ranked_unused(&ranked_saturation, &used))
        .unwrap_or(background_idx);
    if primary_idx < used.len() {
        used[primary_idx] = true;
    }

    let accent_idx = select_accent_index(
        &entries,
        &ranked_saturation,
        &used,
        background,
        entries[primary_idx].color,
    )
    .or_else(|| select_ranked_unused(&ranked_saturation, &used))
    .unwrap_or(primary_idx);
    if accent_idx < used.len() {
        used[accent_idx] = true;
    }

    let secondary_idx = select_secondary_index(
        &entries,
        &ranked_saturation,
        &used,
        background,
        entries[primary_idx].color,
        entries[accent_idx].color,
    )
    .or_else(|| select_ranked_unused(&ranked_saturation, &used))
    .unwrap_or(accent_idx);
    if secondary_idx < used.len() {
        used[secondary_idx] = true;
    }

    let accent2_idx = select_accent2_index(
        &entries,
        &ranked_saturation,
        &used,
        background,
        entries[primary_idx].color,
        entries[accent_idx].color,
        entries[secondary_idx].color,
    )
    .or_else(|| select_ranked_unused(&ranked_saturation, &used))
    .unwrap_or(secondary_idx);
    if accent2_idx < used.len() {
        used[accent2_idx] = true;
    }

    let text_idx = select_highest_contrast_index(&entries, background, &used).unwrap_or_else(|| {
        select_highest_contrast_index(&entries, background, &vec![false; entries.len()])
            .unwrap_or(background_idx)
    });
    if text_idx < used.len() {
        used[text_idx] = true;
    }

    let highlight_idx =
        select_highest_contrast_index(&entries, background, &used).unwrap_or(text_idx);

    ThemePalette {
        background,
        surface,
        surface_container,
        surface_high,
        primary: entries[primary_idx].color,
        secondary: entries[secondary_idx].color,
        accent: entries[accent_idx].color,
        accent2: entries[accent2_idx].color,
        highlight: entries[highlight_idx].color,
        text: entries[text_idx].color,
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

fn saturation_ranked_indices(entries: &[ColorEntry]) -> Vec<usize> {
    let mut ranked: Vec<usize> = entries.iter().enumerate().map(|(idx, _)| idx).collect();

    ranked.sort_by(|a, b| {
        entries[*b]
            .saturation
            .total_cmp(&entries[*a].saturation)
            .then_with(|| entries[*a].luminance.total_cmp(&entries[*b].luminance))
    });

    ranked
}

fn select_ranked_unused(ranked: &[usize], used: &[bool]) -> Option<usize> {
    ranked.iter().copied().find(|idx| !used[*idx])
}

fn select_primary_index(
    entries: &[ColorEntry],
    ranked: &[usize],
    used: &[bool],
    background: Color,
) -> Option<usize> {
    ranked
        .iter()
        .copied()
        .find(|idx| !used[*idx] && delta_e(entries[*idx].color, background) > 20.0)
}

fn select_accent_index(
    entries: &[ColorEntry],
    ranked: &[usize],
    used: &[bool],
    background: Color,
    primary: Color,
) -> Option<usize> {
    ranked.iter().copied().find(|idx| {
        !used[*idx]
            && delta_e(entries[*idx].color, background) > 15.0
            && delta_e(entries[*idx].color, primary) > 15.0
    })
}

fn select_secondary_index(
    entries: &[ColorEntry],
    ranked: &[usize],
    used: &[bool],
    background: Color,
    primary: Color,
    accent: Color,
) -> Option<usize> {
    ranked.iter().copied().find(|idx| {
        !used[*idx]
            && delta_e(entries[*idx].color, background) > 12.0
            && delta_e(entries[*idx].color, primary) > 10.0
            && delta_e(entries[*idx].color, accent) > 8.0
    })
}

fn select_accent2_index(
    entries: &[ColorEntry],
    ranked: &[usize],
    used: &[bool],
    background: Color,
    primary: Color,
    accent: Color,
    secondary: Color,
) -> Option<usize> {
    ranked.iter().copied().find(|idx| {
        !used[*idx]
            && delta_e(entries[*idx].color, background) > 10.0
            && delta_e(entries[*idx].color, primary) > 8.0
            && delta_e(entries[*idx].color, accent) > 8.0
            && delta_e(entries[*idx].color, secondary) > 6.0
    })
}

fn select_highest_contrast_index(
    entries: &[ColorEntry],
    background: Color,
    used: &[bool],
) -> Option<usize> {
    let background_luminance = luminance(background);

    entries
        .iter()
        .enumerate()
        .filter(|(idx, _)| !used[*idx])
        .max_by(|(_, a), (_, b)| {
            contrast_ratio(a.luminance, background_luminance)
                .total_cmp(&contrast_ratio(b.luminance, background_luminance))
        })
        .map(|(idx, _)| idx)
}

fn contrast_ratio(a_luminance: f32, b_luminance: f32) -> f32 {
    let a = a_luminance / 255.0;
    let b = b_luminance / 255.0;
    let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
    (lighter + 0.05) / (darker + 0.05)
}
