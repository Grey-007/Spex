use crate::color_engine::derive::{darken, lighten, rgb_to_hsl};
use crate::color_engine::engine::{build_tokens, infer_theme_from_palette};
use crate::color_engine::format::resolve_token_path;
use crate::models::color::Color;
use crate::palette::roles::ThemePalette;

pub fn resolve_token(token: &str, palette: &ThemePalette) -> Option<String> {
    if token.starts_with("colors.") {
        let inferred_theme = infer_theme_from_palette(&palette.colors);
        let tokens = build_tokens(palette.colors.clone(), inferred_theme);
        if let Some(value) = resolve_token_path(&tokens, token) {
            return Some(value);
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
