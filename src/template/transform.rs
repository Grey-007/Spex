use crate::models::color::Color;
use crate::palette::roles::ThemePalette;

pub fn resolve_token(token: &str, palette: &ThemePalette) -> Option<String> {
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
        return get_color(base, palette).map(|color| {
            let (h, s, l) = rgb_to_hsl(color);
            let new_l = (l + amount / 100.0).clamp(0.0, 1.0);
            to_hex(hsl_to_rgb(h, s, new_l))
        });
    }

    if let Some((base, arg)) = split_call(token, "_darken(") {
        let amount = arg.parse::<f32>().ok()?;
        return get_color(base, palette).map(|color| {
            let (h, s, l) = rgb_to_hsl(color);
            let new_l = (l - amount / 100.0).clamp(0.0, 1.0);
            to_hex(hsl_to_rgb(h, s, new_l))
        });
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
