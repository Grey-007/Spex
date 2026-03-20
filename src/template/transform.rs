use crate::color_engine::derive::{darken, lighten, rgb_to_hsl};
use crate::color_engine::engine::{build_tokens, infer_theme_from_palette};
use crate::color_utils::delta_e;
use crate::models::color::Color;
use crate::palette::roles::ThemePalette;

const MIN_BACKGROUND_DELTA_E: f32 = 8.0;

pub fn resolve_token(token: &str, palette: &ThemePalette, debug_theme: bool) -> Option<String> {
    if token.starts_with("colors.") {
        if let Some(resolution) = resolve_color_token(token, palette) {
            if debug_theme {
                println!("Template variable: {token}");
                println!("Resolved role: {}", resolution.resolved_role);
                println!("Color: {}", to_hex(resolution.color));
            }
            return Some(resolution.formatted);
        }
    }

    if let Some(color) = get_color(token, palette) {
        return Some(to_hex(color));
    }

    if let Some(base) = token.strip_suffix("_rgb") {
        return get_color(base, palette)
            .map(|color| format!("{}, {}, {}", color.r, color.g, color.b));
    }

    if let Some((base, arg)) = split_call(token, "_rgba(") {
        let alpha = arg.parse::<f32>().ok()?.clamp(0.0, 1.0);
        return get_color(base, palette)
            .map(|color| format!("rgba({}, {}, {}, {:.2})", color.r, color.g, color.b, alpha));
    }

    if let Some(base) = token.strip_suffix("_hsl") {
        return get_color(base, palette).map(|color| {
            let (h, s, l) = rgb_to_hsl(color);
            format!("hsl({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, l * 100.0)
        });
    }

    if let Some((base, arg)) = split_call(token, "_lighten(") {
        let amount = arg.parse::<f32>().ok()?;
        return get_color(base, palette).map(|color| to_hex(lighten(color, amount)));
    }

    if let Some((base, arg)) = split_call(token, "_darken(") {
        let amount = arg.parse::<f32>().ok()?;
        return get_color(base, palette).map(|color| to_hex(darken(color, amount)));
    }

    None
}

struct ColorTokenResolution {
    resolved_role: String,
    color: Color,
    formatted: String,
}

fn resolve_color_token(token: &str, palette: &ThemePalette) -> Option<ColorTokenResolution> {
    let path = token.strip_prefix("colors.")?;
    let (role, format) = split_role_and_format(path)?;

    if let Some((resolved_role, role_color)) = resolve_theme_color(role, palette) {
        let adjusted = enforce_background_separation(&resolved_role, role_color, palette);
        return format_color(adjusted, format).map(|formatted| ColorTokenResolution {
            resolved_role,
            color: adjusted,
            formatted,
        });
    }

    let inferred_theme = infer_theme_from_palette(&palette.colors);
    let tokens = build_tokens(palette.colors.clone(), inferred_theme);
    let role_color = tokens.colors.get(role).copied()?;
    let adjusted = enforce_background_separation(role, role_color, palette);
    format_color(adjusted, format).map(|formatted| ColorTokenResolution {
        resolved_role: role.to_string(),
        color: adjusted,
        formatted,
    })
}

fn split_role_and_format(path: &str) -> Option<(&str, &str)> {
    if let Some(split) = path.rsplit_once(".default.") {
        return Some(split);
    }

    path.rsplit_once('.')
}

fn resolve_theme_color(role: &str, palette: &ThemePalette) -> Option<(String, Color)> {
    if let Some(color) = exact_theme_color(role, palette) {
        return Some((role.to_string(), color));
    }

    let fallback_role = fallback_role(role)?;
    exact_theme_color(fallback_role, palette).map(|color| (fallback_role.to_string(), color))
}

fn exact_theme_color(role: &str, palette: &ThemePalette) -> Option<Color> {
    match role {
        "background" => Some(palette.background),
        "surface" => Some(palette.surface),
        "surface_container" => Some(palette.surface_container),
        "surface_high" | "surface_container_high" => Some(palette.surface_high),
        "primary" => Some(palette.primary),
        "secondary" => Some(palette.secondary),
        "accent" => Some(palette.accent),
        "accent2" => Some(palette.accent2),
        "highlight" => Some(palette.highlight),
        "text" => Some(palette.text),
        _ => None,
    }
}

fn fallback_role(role: &str) -> Option<&'static str> {
    match role {
        "accent2" => Some("accent"),
        "surface" => Some("background"),
        "secondary" => Some("primary"),
        _ => None,
    }
}

fn format_color(color: Color, format: &str) -> Option<String> {
    match format {
        "hex" => Some(to_hex(color)),
        "rgb" => Some(format!("{}, {}, {}", color.r, color.g, color.b)),
        "rgba" => Some(format!("rgba({}, {}, {}, 1.00)", color.r, color.g, color.b)),
        "hsl" => {
            let (h, s, l) = rgb_to_hsl(color);
            Some(format!(
                "hsl({:.0}, {:.0}%, {:.0}%)",
                h,
                s * 100.0,
                l * 100.0
            ))
        }
        _ if format.starts_with("rgba(") && format.ends_with(')') => {
            let alpha = &format["rgba(".len()..format.len() - 1];
            let alpha = alpha.parse::<f32>().ok()?.clamp(0.0, 1.0);
            Some(format!(
                "rgba({}, {}, {}, {:.2})",
                color.r, color.g, color.b, alpha
            ))
        }
        _ => None,
    }
}

fn enforce_background_separation(role: &str, color: Color, palette: &ThemePalette) -> Color {
    if role == "background" {
        return color;
    }

    let background = palette.background;
    let distance = delta_e(color, background);
    if distance >= MIN_BACKGROUND_DELTA_E {
        return color;
    }

    palette
        .colors
        .iter()
        .copied()
        .filter(|candidate| *candidate != color)
        .filter(|candidate| delta_e(*candidate, background) >= MIN_BACKGROUND_DELTA_E)
        .min_by(|a, b| delta_e(*a, color).total_cmp(&delta_e(*b, color)))
        .unwrap_or(color)
}
fn split_call<'a>(token: &'a str, marker: &str) -> Option<(&'a str, &'a str)> {
    let start = token.find(marker)?;
    if !token.ends_with(')') {
        return None;
    }

    let base = &token[..start];
    let arg_start = start + marker.len();
    let arg = &token[arg_start..token.len() - 1];
    Some((base, arg))
}

fn get_color(name: &str, palette: &ThemePalette) -> Option<Color> {
    match name {
        "background" => Some(palette.background),
        "surface" => Some(palette.surface),
        "surface_container" => Some(palette.surface_container),
        "surface_high" | "surface_container_high" => Some(palette.surface_high),
        "primary" => Some(palette.primary),
        "secondary" => Some(palette.secondary),
        "accent" => Some(palette.accent),
        "accent2" => Some(palette.accent2),
        "highlight" => Some(palette.highlight),
        "text" => Some(palette.text),
        _ => {
            let index = name.strip_prefix("color")?.parse::<usize>().ok()?;
            palette.colors.get(index).copied()
        }
    }
}

fn to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

#[cfg(test)]
mod tests {
    use super::resolve_token;
    use crate::models::color::Color;
    use crate::palette::roles::ThemePalette;

    #[test]
    fn colors_surface_hex_uses_theme_surface_role() {
        let palette = sample_palette();
        let resolved = resolve_token("colors.surface.hex", &palette, false).unwrap();
        assert_eq!(resolved, "#102030");
    }

    #[test]
    fn colors_secondary_hex_uses_theme_secondary_role() {
        let palette = sample_palette();
        let resolved = resolve_token("colors.secondary.hex", &palette, false).unwrap();
        assert_eq!(resolved, "#506070");
    }

    #[test]
    fn colors_surface_hex_adjusts_if_too_close_to_background() {
        let mut palette = sample_palette();
        palette.background = Color {
            r: 10,
            g: 10,
            b: 10,
        };
        palette.surface = Color {
            r: 11,
            g: 11,
            b: 11,
        };
        palette.colors = vec![
            palette.background,
            palette.surface,
            Color {
                r: 24,
                g: 24,
                b: 24,
            },
            Color {
                r: 60,
                g: 65,
                b: 75,
            },
        ];

        let resolved = resolve_token("colors.surface.hex", &palette, false).unwrap();
        assert_eq!(resolved, "#3C414B");
    }

    fn sample_palette() -> ThemePalette {
        ThemePalette {
            background: Color { r: 5, g: 8, b: 14 },
            surface: Color {
                r: 16,
                g: 32,
                b: 48,
            },
            surface_container: Color {
                r: 24,
                g: 40,
                b: 56,
            },
            surface_high: Color {
                r: 36,
                g: 52,
                b: 68,
            },
            primary: Color {
                r: 32,
                g: 64,
                b: 96,
            },
            secondary: Color {
                r: 80,
                g: 96,
                b: 112,
            },
            accent: Color {
                r: 120,
                g: 80,
                b: 40,
            },
            accent2: Color {
                r: 140,
                g: 110,
                b: 75,
            },
            highlight: Color {
                r: 220,
                g: 180,
                b: 120,
            },
            text: Color {
                r: 230,
                g: 235,
                b: 240,
            },
            colors: vec![
                Color { r: 5, g: 8, b: 14 },
                Color {
                    r: 16,
                    g: 32,
                    b: 48,
                },
                Color {
                    r: 32,
                    g: 64,
                    b: 96,
                },
                Color {
                    r: 80,
                    g: 96,
                    b: 112,
                },
                Color {
                    r: 120,
                    g: 80,
                    b: 40,
                },
                Color {
                    r: 140,
                    g: 110,
                    b: 75,
                },
                Color {
                    r: 220,
                    g: 180,
                    b: 120,
                },
                Color {
                    r: 230,
                    g: 235,
                    b: 240,
                },
            ],
        }
    }
}
