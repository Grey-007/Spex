use crate::color_engine::engine::{build_tokens, infer_theme_from_palette};
use crate::models::color::Color;
use crate::models::theme::ThemeMode;

pub fn check_color_engine() -> usize {
    let mut issues = 0usize;
    let palette = mock_palette();

    let dark_tokens = build_tokens(palette.clone(), ThemeMode::Dark);
    let light_tokens = build_tokens(palette.clone(), ThemeMode::Light);
    let inferred = infer_theme_from_palette(&palette);
    let inferred_tokens = build_tokens(palette, inferred);

    for required in ["background", "surface", "primary", "secondary"] {
        if !dark_tokens.colors.contains_key(required) {
            println!("[ERROR] Color engine missing required token in dark theme: {required}");
            issues += 1;
        }
        if !light_tokens.colors.contains_key(required) {
            println!("[ERROR] Color engine missing required token in light theme: {required}");
            issues += 1;
        }
        if !inferred_tokens.colors.contains_key(required) {
            println!("[ERROR] Color engine missing required token in inferred theme: {required}");
            issues += 1;
        }
    }

    if issues == 0 {
        println!("[OK] Color engine working");
    }

    issues
}

pub fn mock_palette() -> Vec<Color> {
    vec![
        Color {
            r: 14,
            g: 18,
            b: 50,
        },
        Color {
            r: 24,
            g: 36,
            b: 74,
        },
        Color {
            r: 45,
            g: 70,
            b: 130,
        },
        Color {
            r: 62,
            g: 99,
            b: 157,
        },
        Color {
            r: 53,
            g: 83,
            b: 134,
        },
        Color {
            r: 255,
            g: 209,
            b: 102,
        },
        Color {
            r: 240,
            g: 138,
            b: 93,
        },
        Color {
            r: 122,
            g: 215,
            b: 240,
        },
        Color {
            r: 230,
            g: 242,
            b: 255,
        },
        Color {
            r: 186,
            g: 198,
            b: 226,
        },
        Color {
            r: 80,
            g: 94,
            b: 137,
        },
        Color {
            r: 102,
            g: 125,
            b: 178,
        },
        Color {
            r: 142,
            g: 162,
            b: 209,
        },
        Color {
            r: 32,
            g: 45,
            b: 84,
        },
        Color {
            r: 171,
            g: 184,
            b: 210,
        },
        Color {
            r: 210,
            g: 217,
            b: 234,
        },
    ]
}
