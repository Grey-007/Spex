use crate::models::color::Color;

use super::derive::rgb_to_hsl;
use super::engine::SpexColorTokens;

pub fn resolve_token_path(tokens: &SpexColorTokens, token: &str) -> Option<String> {
    let path = token.strip_prefix("colors.")?;
    let (role, format) = split_role_and_format(path)?;
    let color = tokens.colors.get(role)?;

    match format {
        "hex" => Some(to_hex(*color)),
        "rgb" => Some(format!("{}, {}, {}", color.r, color.g, color.b)),
        "hsl" => Some(to_hsl(*color)),
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

fn split_role_and_format(path: &str) -> Option<(&str, &str)> {
    if let Some(split) = path.rsplit_once(".default.") {
        return Some(split);
    }

    path.rsplit_once('.')
}

fn to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

fn to_hsl(color: Color) -> String {
    let (h, s, l) = rgb_to_hsl(color);
    format!("hsl({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, l * 100.0)
}
