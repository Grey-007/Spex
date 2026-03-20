use crate::color_engine::derive::{darken, hsl_to_rgb, lighten, rgb_to_hsl};
use crate::color_utils::{
    LabColor, contrast_ratio, delta_e, luminance, rgb_to_lab, saturation, tint_background,
};
use crate::models::color::Color;
use crate::models::theme::ThemeMode;

const SURFACE_MIN_DELTA_E: f32 = 8.0;
const PRIMARY_MIN_DELTA_E_BG: f32 = 20.0;
const SECONDARY_MIN_DELTA_E_BG: f32 = 15.0;
const SECONDARY_MIN_DELTA_E_PRIMARY: f32 = 12.0;
const ACCENT_MIN_DELTA_E: f32 = 10.0;
const MIN_TEXT_CONTRAST_RATIO: f32 = 4.5;
const ROLE_FALLBACK_MIN_DELTA_E: f32 = 8.0;
const MIN_THEME_GAP: f32 = 2.0;
const MIN_LAB_LIGHTNESS: f32 = 18.0;
const MAX_LAB_LIGHTNESS: f32 = 92.0;

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
    lab: LabColor,
    saturation: f32,
    theme_depth: f32,
}

pub fn assign_roles(colors: Vec<Color>, theme: ThemeMode) -> ThemePalette {
    let mut entries = build_entries(colors, theme);
    sort_by_theme_depth(&mut entries);
    let ordered_palette: Vec<Color> = entries.iter().map(|entry| entry.color).collect();

    let mut used = vec![false; entries.len()];
    let background_idx = 0;
    used[background_idx] = true;

    let background = tint_background(entries[background_idx].color, theme);

    let surface_idx = select_surface_index(&entries, background, theme, &used);
    if let Some(idx) = surface_idx {
        used[idx] = true;
    }
    let surface_seed = surface_idx
        .map(|idx| entries[idx].color)
        .unwrap_or_else(|| step_away_from_background(background, theme, 8.0));
    let surface = refine_role_color(
        surface_seed,
        theme,
        theme_depth(background, theme) + 0.5,
        f32::MAX,
        &[background],
        SURFACE_MIN_DELTA_E,
        None,
        0.0,
        false,
    );

    let surface_container = refine_surface_layer(
        step_away_from_background(surface, theme, 4.0),
        theme,
        background,
        surface,
    );
    let surface_high = refine_surface_layer(
        step_away_from_background(surface_container, theme, 6.0),
        theme,
        surface,
        surface_container,
    );

    let text_idx = select_text_index(&entries, background, &used);
    if let Some(idx) = text_idx {
        used[idx] = true;
    }
    let text_seed = text_idx
        .map(|idx| entries[idx].color)
        .unwrap_or_else(|| step_away_from_background(background, theme, 52.0));
    let text = refine_role_color(
        text_seed,
        theme,
        theme_depth(surface_high, theme) + MIN_THEME_GAP,
        f32::MAX,
        &[background, surface, surface_container, surface_high],
        ROLE_FALLBACK_MIN_DELTA_E,
        Some(background),
        MIN_TEXT_CONTRAST_RATIO,
        false,
    );

    let accent_min_depth = theme_depth(surface_high, theme) + MIN_THEME_GAP;
    let accent_max_depth = (theme_depth(text, theme) - MIN_THEME_GAP).max(accent_min_depth + 0.5);
    let ranked_saturation = saturation_ranked_indices(&entries);

    let primary_idx = select_primary_index(
        &entries,
        &ranked_saturation,
        &used,
        background,
        accent_min_depth,
        accent_max_depth,
    );
    if let Some(idx) = primary_idx {
        used[idx] = true;
    }
    let primary_seed = primary_idx
        .map(|idx| entries[idx].color)
        .or_else(|| fallback_ranked_color(&entries, &ranked_saturation, &used))
        .unwrap_or(surface);
    let primary = refine_role_color(
        primary_seed,
        theme,
        accent_min_depth,
        accent_max_depth,
        &[background, surface, surface_container, surface_high, text],
        ACCENT_MIN_DELTA_E,
        None,
        0.0,
        true,
    );

    let secondary_idx = select_secondary_index(
        &entries,
        &ranked_saturation,
        &used,
        background,
        primary,
        accent_min_depth,
        accent_max_depth,
    );
    if let Some(idx) = secondary_idx {
        used[idx] = true;
    }
    let secondary_seed = secondary_idx
        .map(|idx| entries[idx].color)
        .or_else(|| fallback_ranked_color(&entries, &ranked_saturation, &used))
        .unwrap_or(primary);
    let secondary = refine_role_color(
        secondary_seed,
        theme,
        accent_min_depth,
        accent_max_depth,
        &[
            background,
            surface,
            surface_container,
            surface_high,
            primary,
            text,
        ],
        ACCENT_MIN_DELTA_E,
        None,
        0.0,
        true,
    );

    let accent_idx = select_accent_index(
        &entries,
        &ranked_saturation,
        &used,
        background,
        &[primary, secondary],
        accent_min_depth,
        accent_max_depth,
    );
    if let Some(idx) = accent_idx {
        used[idx] = true;
    }
    let accent_seed = accent_idx
        .map(|idx| entries[idx].color)
        .or_else(|| fallback_ranked_color(&entries, &ranked_saturation, &used))
        .unwrap_or(secondary);
    let accent = refine_role_color(
        accent_seed,
        theme,
        accent_min_depth,
        accent_max_depth,
        &[
            background,
            surface,
            surface_container,
            surface_high,
            primary,
            secondary,
            text,
        ],
        ACCENT_MIN_DELTA_E,
        None,
        0.0,
        true,
    );

    let accent2_idx = select_accent_index(
        &entries,
        &ranked_saturation,
        &used,
        background,
        &[primary, secondary, accent],
        accent_min_depth,
        accent_max_depth,
    );
    if let Some(idx) = accent2_idx {
        used[idx] = true;
    }
    let accent2_seed = accent2_idx
        .map(|idx| entries[idx].color)
        .or_else(|| fallback_ranked_color(&entries, &ranked_saturation, &used))
        .unwrap_or(accent);
    let accent2 = refine_role_color(
        accent2_seed,
        theme,
        accent_min_depth,
        accent_max_depth,
        &[
            background,
            surface,
            surface_container,
            surface_high,
            primary,
            secondary,
            accent,
            text,
        ],
        ACCENT_MIN_DELTA_E,
        None,
        0.0,
        true,
    );

    let highlight_idx = select_highlight_index(
        &entries,
        &used,
        background,
        primary,
        accent_min_depth,
        accent_max_depth,
    );
    let highlight_seed = highlight_idx
        .map(|idx| entries[idx].color)
        .or_else(|| {
            [
                accent2_idx,
                accent_idx,
                secondary_idx,
                primary_idx,
                text_idx,
                surface_idx,
            ]
            .into_iter()
            .flatten()
            .map(|idx| entries[idx].color)
            .next()
        })
        .unwrap_or(accent2);
    let highlight = refine_role_color(
        highlight_seed,
        theme,
        accent_min_depth,
        accent_max_depth,
        &[
            background,
            surface,
            surface_container,
            surface_high,
            primary,
            secondary,
            accent,
            accent2,
            text,
        ],
        ACCENT_MIN_DELTA_E,
        None,
        0.0,
        true,
    );

    ThemePalette {
        background,
        surface,
        surface_container,
        surface_high,
        primary,
        secondary,
        accent,
        accent2,
        highlight,
        text,
        colors: ordered_palette,
    }
}

fn build_entries(colors: Vec<Color>, theme: ThemeMode) -> Vec<ColorEntry> {
    let palette = if colors.is_empty() {
        vec![Color { r: 0, g: 0, b: 0 }]
    } else {
        colors
    };

    palette
        .into_iter()
        .map(|color| ColorEntry {
            color,
            lab: rgb_to_lab(color),
            saturation: saturation(color),
            theme_depth: theme_depth(color, theme),
        })
        .collect()
}

fn sort_by_theme_depth(entries: &mut [ColorEntry]) {
    entries.sort_by(|a, b| {
        a.theme_depth
            .total_cmp(&b.theme_depth)
            .then_with(|| a.saturation.total_cmp(&b.saturation))
    });
}

fn theme_depth(color: Color, theme: ThemeMode) -> f32 {
    match theme {
        ThemeMode::Dark => luminance(color),
        ThemeMode::Light => 255.0 - luminance(color),
    }
}

fn saturation_ranked_indices(entries: &[ColorEntry]) -> Vec<usize> {
    let mut ranked: Vec<usize> = entries.iter().enumerate().map(|(idx, _)| idx).collect();
    ranked.sort_by(|a, b| {
        entries[*b]
            .saturation
            .total_cmp(&entries[*a].saturation)
            .then_with(|| entries[*b].lab.l.total_cmp(&entries[*a].lab.l))
    });
    ranked
}

fn select_surface_index(
    entries: &[ColorEntry],
    background: Color,
    theme: ThemeMode,
    used: &[bool],
) -> Option<usize> {
    let background_depth = theme_depth(background, theme);
    let mut ranked: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter_map(|(idx, _)| (!used[idx]).then_some(idx))
        .collect();

    ranked.sort_by(|a, b| {
        let a_distance = (entries[*a].theme_depth - background_depth).abs();
        let b_distance = (entries[*b].theme_depth - background_depth).abs();
        a_distance
            .total_cmp(&b_distance)
            .then_with(|| entries[*b].saturation.total_cmp(&entries[*a].saturation))
    });

    ranked
        .into_iter()
        .find(|idx| delta_e(entries[*idx].color, background) > SURFACE_MIN_DELTA_E)
}

fn select_text_index(entries: &[ColorEntry], background: Color, used: &[bool]) -> Option<usize> {
    entries
        .iter()
        .enumerate()
        .filter(|(idx, _)| !used[*idx])
        .max_by(|(_, a), (_, b)| {
            contrast_ratio(a.color, background)
                .total_cmp(&contrast_ratio(b.color, background))
                .then_with(|| a.theme_depth.total_cmp(&b.theme_depth))
        })
        .map(|(idx, _)| idx)
}

fn select_primary_index(
    entries: &[ColorEntry],
    ranked: &[usize],
    used: &[bool],
    background: Color,
    min_depth: f32,
    max_depth: f32,
) -> Option<usize> {
    select_saturated_index(ranked, used, |idx| {
        let entry = entries[idx];
        delta_e(entry.color, background) > PRIMARY_MIN_DELTA_E_BG
            && in_theme_band(entry.theme_depth, min_depth, max_depth)
            && in_lab_band(entry.lab.l)
    })
}

fn select_secondary_index(
    entries: &[ColorEntry],
    ranked: &[usize],
    used: &[bool],
    background: Color,
    primary: Color,
    min_depth: f32,
    max_depth: f32,
) -> Option<usize> {
    select_saturated_index(ranked, used, |idx| {
        let entry = entries[idx];
        delta_e(entry.color, background) > SECONDARY_MIN_DELTA_E_BG
            && delta_e(entry.color, primary) > SECONDARY_MIN_DELTA_E_PRIMARY
            && in_theme_band(entry.theme_depth, min_depth, max_depth)
    })
}

fn select_accent_index(
    entries: &[ColorEntry],
    ranked: &[usize],
    used: &[bool],
    background: Color,
    related: &[Color],
    min_depth: f32,
    max_depth: f32,
) -> Option<usize> {
    select_saturated_index(ranked, used, |idx| {
        let entry = entries[idx];
        delta_e(entry.color, background) > ACCENT_MIN_DELTA_E
            && in_theme_band(entry.theme_depth, min_depth, max_depth)
            && related
                .iter()
                .all(|color| delta_e(entry.color, *color) > ACCENT_MIN_DELTA_E)
    })
}

fn select_highlight_index(
    entries: &[ColorEntry],
    used: &[bool],
    background: Color,
    primary: Color,
    min_depth: f32,
    max_depth: f32,
) -> Option<usize> {
    entries
        .iter()
        .enumerate()
        .filter(|(idx, entry)| {
            !used[*idx]
                && in_theme_band(entry.theme_depth, min_depth, max_depth)
                && delta_e(entry.color, background) > ACCENT_MIN_DELTA_E
        })
        .max_by(|(_, a), (_, b)| {
            highlight_score(a.color, background, primary)
                .total_cmp(&highlight_score(b.color, background, primary))
        })
        .map(|(idx, _)| idx)
}

fn select_saturated_index(
    ranked: &[usize],
    used: &[bool],
    predicate: impl Fn(usize) -> bool,
) -> Option<usize> {
    ranked
        .iter()
        .copied()
        .find(|idx| !used[*idx] && predicate(*idx))
}

fn fallback_ranked_color(entries: &[ColorEntry], ranked: &[usize], used: &[bool]) -> Option<Color> {
    ranked
        .iter()
        .copied()
        .find(|idx| !used[*idx])
        .map(|idx| entries[idx].color)
}

fn highlight_score(color: Color, background: Color, primary: Color) -> f32 {
    delta_e(color, background) + delta_e(color, primary) * 0.85 + saturation(color) * 12.0
}

fn refine_surface_layer(seed: Color, theme: ThemeMode, floor: Color, previous: Color) -> Color {
    let min_depth = theme_depth(previous, theme) + 0.5;
    refine_role_color(
        seed,
        theme,
        min_depth,
        f32::MAX,
        &[floor, previous],
        4.0,
        None,
        0.0,
        false,
    )
}

fn refine_role_color(
    seed: Color,
    theme: ThemeMode,
    min_depth: f32,
    max_depth: f32,
    related: &[Color],
    min_delta_e: f32,
    contrast_against: Option<Color>,
    min_contrast_ratio: f32,
    boost_saturation: bool,
) -> Color {
    let mut color = seed;

    for attempt in 0..10 {
        let depth = theme_depth(color, theme);
        let unique = related
            .iter()
            .all(|other| delta_e(color, *other) >= min_delta_e);
        let contrast_ok = contrast_against
            .map(|target| contrast_ratio(color, target) >= min_contrast_ratio)
            .unwrap_or(true);

        if depth > min_depth && depth < max_depth && unique && contrast_ok {
            return color;
        }

        if depth <= min_depth || !contrast_ok {
            color = step_away_from_background(color, theme, 4.0 + attempt as f32 * 1.8);
        } else if depth >= max_depth {
            color = step_toward_background(color, theme, 3.0 + attempt as f32 * 1.4);
        }

        let (hue, sat, lightness) = rgb_to_hsl(color);
        let sat = if boost_saturation {
            (sat * (1.08 + attempt as f32 * 0.03) + 0.015).clamp(0.0, 0.95)
        } else {
            sat
        };
        let lightness = if unique {
            lightness
        } else {
            match theme {
                ThemeMode::Dark => (lightness + 0.01 * (attempt as f32 + 1.0)).clamp(0.0, 0.96),
                ThemeMode::Light => (lightness - 0.01 * (attempt as f32 + 1.0)).clamp(0.04, 1.0),
            }
        };

        color = hsl_to_rgb(hue, sat, lightness);
    }

    color
}

fn in_theme_band(depth: f32, min_depth: f32, max_depth: f32) -> bool {
    depth > min_depth && depth < max_depth
}

fn in_lab_band(lightness: f32) -> bool {
    (MIN_LAB_LIGHTNESS..=MAX_LAB_LIGHTNESS).contains(&lightness)
}

fn step_away_from_background(color: Color, theme: ThemeMode, amount: f32) -> Color {
    match theme {
        ThemeMode::Dark => lighten(color, amount),
        ThemeMode::Light => darken(color, amount),
    }
}

fn step_toward_background(color: Color, theme: ThemeMode, amount: f32) -> Color {
    match theme {
        ThemeMode::Dark => darken(color, amount),
        ThemeMode::Light => lighten(color, amount),
    }
}

#[cfg(test)]
mod tests {
    use super::{assign_roles, theme_depth};
    use crate::color_utils::{contrast_ratio, delta_e};
    use crate::models::color::Color;
    use crate::models::theme::ThemeMode;

    #[test]
    fn dark_theme_keeps_roles_in_expected_order() {
        let theme = assign_roles(sample_palette(), ThemeMode::Dark);
        assert!(
            theme_depth(theme.background, ThemeMode::Dark)
                < theme_depth(theme.surface, ThemeMode::Dark)
        );
        assert!(
            theme_depth(theme.surface_high, ThemeMode::Dark)
                < theme_depth(theme.primary, ThemeMode::Dark)
        );
        assert!(
            theme_depth(theme.primary, ThemeMode::Dark) < theme_depth(theme.text, ThemeMode::Dark)
        );
        assert!(delta_e(theme.background, theme.surface) > 8.0);
        assert!(contrast_ratio(theme.text, theme.background) >= 4.5);
    }

    #[test]
    fn light_theme_keeps_roles_in_expected_order() {
        let theme = assign_roles(sample_palette(), ThemeMode::Light);
        assert!(
            theme_depth(theme.background, ThemeMode::Light)
                < theme_depth(theme.surface, ThemeMode::Light)
        );
        assert!(
            theme_depth(theme.surface_high, ThemeMode::Light)
                < theme_depth(theme.accent, ThemeMode::Light)
        );
        assert!(
            theme_depth(theme.accent, ThemeMode::Light) < theme_depth(theme.text, ThemeMode::Light)
        );
        assert!(contrast_ratio(theme.text, theme.background) >= 4.5);
    }

    #[test]
    fn low_distinct_palette_still_produces_unique_roles() {
        let theme = assign_roles(
            vec![
                Color {
                    r: 28,
                    g: 30,
                    b: 35,
                },
                Color {
                    r: 42,
                    g: 46,
                    b: 54,
                },
                Color {
                    r: 54,
                    g: 58,
                    b: 68,
                },
                Color {
                    r: 78,
                    g: 86,
                    b: 102,
                },
            ],
            ThemeMode::Dark,
        );

        assert_ne!(theme.primary, theme.secondary);
        assert_ne!(theme.secondary, theme.accent);
        assert_ne!(theme.accent, theme.accent2);
        assert!(delta_e(theme.primary, theme.secondary) >= 8.0);
    }

    fn sample_palette() -> Vec<Color> {
        vec![
            Color {
                r: 10,
                g: 18,
                b: 24,
            },
            Color {
                r: 28,
                g: 38,
                b: 52,
            },
            Color {
                r: 52,
                g: 72,
                b: 92,
            },
            Color {
                r: 92,
                g: 126,
                b: 168,
            },
            Color {
                r: 199,
                g: 105,
                b: 159,
            },
            Color {
                r: 215,
                g: 157,
                b: 92,
            },
            Color {
                r: 108,
                g: 186,
                b: 156,
            },
            Color {
                r: 236,
                g: 240,
                b: 245,
            },
        ]
    }
}
