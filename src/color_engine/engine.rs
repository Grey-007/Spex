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
    let foreground = on_color(surface);

    let primary = select_by_saturation(&palette, 0);
    let primary_container = container_color(primary, theme, 22.0);
    let on_primary = on_color(primary);
    let on_primary_container = on_color(primary_container);

    let secondary = select_by_saturation(&palette, 1);
    let secondary_container = container_color(secondary, theme, 18.0);
    let on_secondary = on_color(secondary);
    let on_secondary_container = on_color(secondary_container);

    let tertiary = desaturate(rotate_hue(primary, 35.0), 0.15);
    let tertiary_container = container_color(tertiary, theme, 18.0);
    let on_tertiary = on_color(tertiary);
    let on_tertiary_container = on_color(tertiary_container);

    let error = select_error(&palette).unwrap_or_else(|| rotate_hue(primary, -32.0));
    let error_container = container_color(error, theme, 20.0);
    let on_error = on_color(error);
    let on_error_container = on_color(error_container);

    let surface_variant = desaturate(container_color(surface, theme, 10.0), 0.25);
    let surface_container_low = container_color(surface, theme, 4.0);
    let surface_container = container_color(surface, theme, 8.0);
    let surface_container_high = container_color(surface, theme, 12.0);
    let surface_container_highest = container_color(surface, theme, 16.0);

    let outline = desaturate(surface_variant, 0.6);
    let outline_variant = container_color(outline, theme, 10.0);
    let border = mix_for_border(surface_variant, theme);
    let highlight = select_highlight(&palette);
    let selection = container_color(primary, theme, 14.0);

    colors.insert(ROLE_BACKGROUND.to_string(), background);
    colors.insert(ROLE_FOREGROUND.to_string(), foreground);

    colors.insert(ROLE_PRIMARY.to_string(), primary);
    colors.insert(ROLE_PRIMARY_CONTAINER.to_string(), primary_container);
    colors.insert(ROLE_ON_PRIMARY.to_string(), on_primary);
    colors.insert(ROLE_ON_PRIMARY_CONTAINER.to_string(), on_primary_container);

    colors.insert(ROLE_SECONDARY.to_string(), secondary);
    colors.insert(ROLE_SECONDARY_CONTAINER.to_string(), secondary_container);
    colors.insert(ROLE_ON_SECONDARY.to_string(), on_secondary);
    colors.insert(
        ROLE_ON_SECONDARY_CONTAINER.to_string(),
        on_secondary_container,
    );

    colors.insert(ROLE_TERTIARY.to_string(), tertiary);
    colors.insert(ROLE_TERTIARY_CONTAINER.to_string(), tertiary_container);
    colors.insert(ROLE_ON_TERTIARY.to_string(), on_tertiary);
    colors.insert(
        ROLE_ON_TERTIARY_CONTAINER.to_string(),
        on_tertiary_container,
    );

    colors.insert(ROLE_ERROR.to_string(), error);
    colors.insert(ROLE_ERROR_CONTAINER.to_string(), error_container);
    colors.insert(ROLE_ON_ERROR.to_string(), on_error);
    colors.insert(ROLE_ON_ERROR_CONTAINER.to_string(), on_error_container);

    colors.insert(ROLE_SURFACE.to_string(), surface);
    colors.insert(ROLE_SURFACE_VARIANT.to_string(), surface_variant);
    colors.insert(ROLE_SURFACE_CONTAINER.to_string(), surface_container);
    colors.insert(
        ROLE_SURFACE_CONTAINER_LOW.to_string(),
        surface_container_low,
    );
    colors.insert(
        ROLE_SURFACE_CONTAINER_HIGH.to_string(),
        surface_container_high,
    );
    colors.insert(
        ROLE_SURFACE_CONTAINER_HIGHEST.to_string(),
        surface_container_highest,
    );

    colors.insert(ROLE_OUTLINE.to_string(), outline);
    colors.insert(ROLE_OUTLINE_VARIANT.to_string(), outline_variant);
    colors.insert(ROLE_BORDER.to_string(), border);
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

fn select_error(palette: &[Color]) -> Option<Color> {
    let mut best: Option<(Color, f32)> = None;

    for color in palette.iter().copied() {
        let score = error_score(color);
        match best {
            Some((_, best_score)) if score <= best_score => {}
            _ => best = Some((color, score)),
        }
    }

    best.map(|(color, _)| color)
}

fn error_score(color: Color) -> f32 {
    let red = color.r as f32 / 255.0;
    let green = color.g as f32 / 255.0;
    let blue = color.b as f32 / 255.0;

    red * 1.8 - green * 0.9 - blue * 0.9
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

fn container_color(color: Color, theme: ThemeMode, amount: f32) -> Color {
    match theme {
        ThemeMode::Dark => lighten(color, amount),
        ThemeMode::Light => darken(color, amount * 0.8),
    }
}

fn mix_for_border(surface_variant: Color, theme: ThemeMode) -> Color {
    match theme {
        ThemeMode::Dark => lighten(desaturate(surface_variant, 0.15), 4.0),
        ThemeMode::Light => darken(desaturate(surface_variant, 0.15), 6.0),
    }
}

fn on_color(color: Color) -> Color {
    if luminance(color) >= 145.0 {
        Color {
            r: 20,
            g: 23,
            b: 28,
        }
    } else {
        Color {
            r: 245,
            g: 247,
            b: 250,
        }
    }
}
