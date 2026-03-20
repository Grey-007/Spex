use crate::models::color::Color;

pub fn lighten(color: Color, percent: f32) -> Color {
    let (h, s, l) = rgb_to_hsl(color);
    let next_l = (l + percent / 100.0).clamp(0.0, 1.0);
    hsl_to_rgb(h, s, next_l)
}

pub fn darken(color: Color, percent: f32) -> Color {
    let (h, s, l) = rgb_to_hsl(color);
    let next_l = (l - percent / 100.0).clamp(0.0, 1.0);
    hsl_to_rgb(h, s, next_l)
}

pub fn desaturate(color: Color, factor: f32) -> Color {
    let (h, s, l) = rgb_to_hsl(color);
    let next_s = (s * (1.0 - factor)).clamp(0.0, 1.0);
    hsl_to_rgb(h, next_s, l)
}

pub fn rotate_hue(color: Color, degrees: f32) -> Color {
    let (h, s, l) = rgb_to_hsl(color);
    hsl_to_rgb((h + degrees).rem_euclid(360.0), s, l)
}

pub fn rgb_to_hsl(color: Color) -> (f32, f32, f32) {
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

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
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
