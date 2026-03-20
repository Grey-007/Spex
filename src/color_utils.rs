use crate::color_engine::derive::{darken, hsl_to_rgb, lighten, rgb_to_hsl};
use crate::models::color::Color;
use crate::models::pixel::Pixel;
use crate::models::theme::ThemeMode;

pub const DULL_PALETTE_SATURATION_THRESHOLD: f32 = 0.25;

const BLACK: Color = Color { r: 0, g: 0, b: 0 };
const WHITE: Color = Color {
    r: 255,
    g: 255,
    b: 255,
};
const GRAYSCALE_AVG_SATURATION_THRESHOLD: f32 = 0.08;
const GRAYSCALE_MAX_SATURATION_THRESHOLD: f32 = 0.12;
const SURFACE_LAYER_DELTA_E_THRESHOLD: f32 = 4.0;

#[derive(Debug, Clone, Copy)]
pub struct PaletteEnhancementDebug {
    pub average_saturation: f32,
    pub saturation_threshold: f32,
    pub saturation_factor: f32,
    pub contrast_factor: f32,
    pub vibrancy_boost_applied: bool,
    pub grayscale_injection_applied: bool,
    pub dominant_hue_hint: Option<f32>,
}

pub fn mix(color: Color, target: Color, target_ratio: f32) -> Color {
    let target_ratio = target_ratio.clamp(0.0, 1.0);
    let color_ratio = 1.0 - target_ratio;

    Color {
        r: ((color.r as f32 * color_ratio) + (target.r as f32 * target_ratio)).round() as u8,
        g: ((color.g as f32 * color_ratio) + (target.g as f32 * target_ratio)).round() as u8,
        b: ((color.b as f32 * color_ratio) + (target.b as f32 * target_ratio)).round() as u8,
    }
}

pub fn luminance(color: Color) -> f32 {
    0.2126 * color.r as f32 + 0.7152 * color.g as f32 + 0.0722 * color.b as f32
}

pub fn saturation(color: Color) -> f32 {
    let (_, saturation, _) = rgb_to_hsl(color);
    saturation
}

pub fn average_saturation(colors: &[Color]) -> f32 {
    if colors.is_empty() {
        return 0.0;
    }

    colors.iter().map(|color| saturation(*color)).sum::<f32>() / colors.len() as f32
}

pub fn is_grayscale_palette(colors: &[Color]) -> bool {
    let average = average_saturation(colors);
    let max = colors
        .iter()
        .map(|color| saturation(*color))
        .fold(0.0_f32, f32::max);

    average < GRAYSCALE_AVG_SATURATION_THRESHOLD || max < GRAYSCALE_MAX_SATURATION_THRESHOLD
}

pub fn tint_background(color: Color, theme: ThemeMode) -> Color {
    match theme {
        ThemeMode::Dark => mix(color, BLACK, 0.70),
        ThemeMode::Light => mix(color, WHITE, 0.70),
    }
}

pub fn layer_background(background: Color, theme: ThemeMode, amount: f32) -> Color {
    let lightened = lighten(background, amount);
    if delta_e(lightened, background) >= SURFACE_LAYER_DELTA_E_THRESHOLD {
        return lightened;
    }

    match theme {
        ThemeMode::Dark => lightened,
        ThemeMode::Light => {
            let darkened = darken(background, amount * 0.85);
            if delta_e(darkened, background) > delta_e(lightened, background) {
                darkened
            } else {
                lightened
            }
        }
    }
}

pub fn dominant_hue_hint_from_pixels(pixels: &[Pixel]) -> Option<f32> {
    if pixels.is_empty() {
        return None;
    }

    let mut bins = [0.0_f32; 24];
    let mut r_sum = 0u64;
    let mut g_sum = 0u64;
    let mut b_sum = 0u64;

    for pixel in pixels {
        r_sum += pixel.r as u64;
        g_sum += pixel.g as u64;
        b_sum += pixel.b as u64;

        let color = Color {
            r: pixel.r,
            g: pixel.g,
            b: pixel.b,
        };
        let (hue, sat, lightness) = rgb_to_hsl(color);
        if sat < 0.05 {
            continue;
        }

        let bin = ((hue / 15.0).floor() as usize) % bins.len();
        bins[bin] += sat * (0.5 + lightness);
    }

    if let Some((idx, weight)) = bins
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
    {
        if *weight > 0.0 {
            return Some(idx as f32 * 15.0 + 7.5);
        }
    }

    let count = pixels.len() as u64;
    let average = Color {
        r: (r_sum / count) as u8,
        g: (g_sum / count) as u8,
        b: (b_sum / count) as u8,
    };
    let (hue, sat, _) = rgb_to_hsl(average);
    (sat > 0.01).then_some(hue)
}

pub fn enhance_palette(
    colors: Vec<Color>,
    dominant_hue_hint: Option<f32>,
) -> (Vec<Color>, PaletteEnhancementDebug) {
    let average_saturation = average_saturation(&colors);
    let vibrancy_boost_applied = average_saturation < DULL_PALETTE_SATURATION_THRESHOLD;
    let grayscale_palette = is_grayscale_palette(&colors);
    let strength = if vibrancy_boost_applied {
        ((DULL_PALETTE_SATURATION_THRESHOLD - average_saturation)
            / DULL_PALETTE_SATURATION_THRESHOLD)
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
    let saturation_factor = if vibrancy_boost_applied {
        1.20 + strength * 0.20
    } else {
        1.0
    };
    let contrast_factor = if vibrancy_boost_applied {
        1.04 + strength * 0.08
    } else {
        1.0
    };
    let dominant_hue_hint = dominant_hue_hint.or_else(|| dominant_hue_hint_from_colors(&colors));

    if !vibrancy_boost_applied && !grayscale_palette {
        return (
            colors,
            PaletteEnhancementDebug {
                average_saturation,
                saturation_threshold: DULL_PALETTE_SATURATION_THRESHOLD,
                saturation_factor,
                contrast_factor,
                vibrancy_boost_applied,
                grayscale_injection_applied: false,
                dominant_hue_hint,
            },
        );
    }

    let mut enhanced = Vec::with_capacity(colors.len());
    let mut grayscale_injection_applied = false;

    for (idx, color) in colors.into_iter().enumerate() {
        let (mut hue, mut sat, mut lightness) = rgb_to_hsl(color);

        if grayscale_palette {
            if let Some(hint) = dominant_hue_hint {
                let shift = if idx % 2 == 0 { -12.0 } else { 12.0 };
                hue = (hint + shift).rem_euclid(360.0);

                let saturation_floor = (0.08 + (idx % 3) as f32 * 0.03).min(0.18);
                if sat < saturation_floor {
                    sat = saturation_floor;
                    grayscale_injection_applied = true;
                }
            }
        }

        if vibrancy_boost_applied {
            let saturation_cap = if grayscale_palette { 0.28 } else { 1.0 };
            sat = (sat * saturation_factor).clamp(0.0, saturation_cap);
            lightness = contrast_lightness(lightness, contrast_factor);
        }

        enhanced.push(hsl_to_rgb(hue, sat, lightness));
    }

    (
        enhanced,
        PaletteEnhancementDebug {
            average_saturation,
            saturation_threshold: DULL_PALETTE_SATURATION_THRESHOLD,
            saturation_factor,
            contrast_factor,
            vibrancy_boost_applied,
            grayscale_injection_applied,
            dominant_hue_hint,
        },
    )
}

pub fn delta_e(a: Color, b: Color) -> f32 {
    let (al, aa, ab) = rgb_to_lab(a);
    let (bl, ba, bb) = rgb_to_lab(b);
    let dl = al - bl;
    let da = aa - ba;
    let db = ab - bb;
    (dl * dl + da * da + db * db).sqrt()
}

fn contrast_lightness(lightness: f32, factor: f32) -> f32 {
    (0.5 + (lightness - 0.5) * factor).clamp(0.0, 1.0)
}

fn dominant_hue_hint_from_colors(colors: &[Color]) -> Option<f32> {
    let mut bins = [0.0_f32; 24];

    for color in colors {
        let (hue, sat, lightness) = rgb_to_hsl(*color);
        if sat < 0.05 {
            continue;
        }

        let bin = ((hue / 15.0).floor() as usize) % bins.len();
        bins[bin] += sat * (0.5 + lightness);
    }

    bins.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .and_then(|(idx, weight)| (*weight > 0.0).then_some(idx as f32 * 15.0 + 7.5))
}

fn rgb_to_lab(color: Color) -> (f32, f32, f32) {
    let r = srgb_to_linear(color.r as f32 / 255.0);
    let g = srgb_to_linear(color.g as f32 / 255.0);
    let b = srgb_to_linear(color.b as f32 / 255.0);

    let x = (0.412_456_4 * r) + (0.357_576_1 * g) + (0.180_437_5 * b);
    let y = (0.212_672_9 * r) + (0.715_152_2 * g) + (0.072_175 * b);
    let z = (0.019_333_9 * r) + (0.119_192 * g) + (0.950_304_1 * b);

    xyz_to_lab(x, y, z)
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn xyz_to_lab(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    const XN: f32 = 0.950_47;
    const YN: f32 = 1.0;
    const ZN: f32 = 1.088_83;
    const EPSILON: f32 = 216.0 / 24_389.0;
    const KAPPA: f32 = 24_389.0 / 27.0;

    let xr = x / XN;
    let yr = y / YN;
    let zr = z / ZN;

    let fx = f_lab(xr, EPSILON, KAPPA);
    let fy = f_lab(yr, EPSILON, KAPPA);
    let fz = f_lab(zr, EPSILON, KAPPA);

    let l = (116.0 * fy) - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);

    (l, a, b)
}

fn f_lab(t: f32, epsilon: f32, kappa: f32) -> f32 {
    if t > epsilon {
        t.cbrt()
    } else {
        (kappa * t + 16.0) / 116.0
    }
}
