use std::collections::HashMap;

use crate::models::color::Color;
use crate::models::theme::ThemeMode;

use super::derive::{darken, desaturate, lighten, luminance, rotate_hue, saturation};
use super::roles::*;

pub struct SpexColorTokens {
    pub colors: HashMap<String, Color>,
}

pub fn build_tokens(palette: Vec<Color>, theme: ThemeMode) -> SpexColorTokens {
    let palette = if palette.is_empty() {
        vec![Color { r: 0, g: 0, b: 0 }]
    } else {
        palette
    };

    let mut colors = HashMap::new();

    let background = select_background(&palette, theme);
    let surface = background;
    let primary = select_by_saturation(&palette, 0);
    let secondary = select_by_saturation(&palette, 1);
    let tertiary = rotate_hue(primary, 35.0);
    let outline = desaturate(surface, 0.65);
    let outline_variant = lighten(outline, 12.0);
    let border = darken(desaturate(surface, 0.5), 8.0);
    let highlight = select_highlight(&palette);
    let selection = lighten(primary, 14.0);

    colors.insert(ROLE_BACKGROUND.to_string(), background);
    colors.insert(ROLE_SURFACE.to_string(), surface);
    colors.insert(ROLE_PRIMARY.to_string(), primary);
    colors.insert(ROLE_SECONDARY.to_string(), secondary);
    colors.insert(ROLE_TERTIARY.to_string(), tertiary);
    colors.insert(ROLE_OUTLINE.to_string(), outline);
    colors.insert(ROLE_OUTLINE_VARIANT.to_string(), outline_variant);
    colors.insert(ROLE_BORDER.to_string(), border);
    colors.insert(
        ROLE_SURFACE_CONTAINER_LOW.to_string(),
        lighten(surface, container_amount(theme, 4.0)),
    );
    colors.insert(
        ROLE_SURFACE_CONTAINER.to_string(),
        lighten(surface, container_amount(theme, 8.0)),
    );
    colors.insert(
        ROLE_SURFACE_CONTAINER_HIGH.to_string(),
        lighten(surface, container_amount(theme, 12.0)),
    );
    colors.insert(
        ROLE_SURFACE_CONTAINER_HIGHEST.to_string(),
        lighten(surface, container_amount(theme, 16.0)),
    );
    colors.insert(ROLE_HIGHLIGHT.to_string(), highlight);
    colors.insert(ROLE_SELECTION.to_string(), selection);

    SpexColorTokens { colors }
}

pub fn infer_theme_from_palette(palette: &[Color]) -> ThemeMode {
    if palette.is_empty() {
        return ThemeMode::Dark;
    }

    let avg_luminance = palette.iter().map(|c| luminance(*c)).sum::<f32>() / palette.len() as f32;
    if avg_luminance < 128.0 {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    }
}

fn select_background(palette: &[Color], theme: ThemeMode) -> Color {
    match theme {
        ThemeMode::Dark => palette
            .iter()
            .copied()
            .min_by(|a, b| luminance(*a).total_cmp(&luminance(*b)))
            .unwrap_or(Color { r: 0, g: 0, b: 0 }),
        ThemeMode::Light => palette
            .iter()
            .copied()
            .max_by(|a, b| luminance(*a).total_cmp(&luminance(*b)))
            .unwrap_or(Color {
                r: 255,
                g: 255,
                b: 255,
            }),
    }
}

fn select_by_saturation(palette: &[Color], rank: usize) -> Color {
    let mut sorted = palette.to_vec();
    sorted.sort_by(|a, b| saturation(*b).total_cmp(&saturation(*a)));
    sorted
        .get(rank)
        .copied()
        .unwrap_or(*sorted.first().unwrap_or(&Color { r: 0, g: 0, b: 0 }))
}

fn select_highlight(palette: &[Color]) -> Color {
    palette
        .iter()
        .copied()
        .max_by(|a, b| luminance(*a).total_cmp(&luminance(*b)))
        .unwrap_or(Color {
            r: 255,
            g: 255,
            b: 255,
        })
}

fn container_amount(theme: ThemeMode, amount: f32) -> f32 {
    match theme {
        ThemeMode::Dark => amount,
        ThemeMode::Light => -amount * 0.8,
    }
}
