use crate::models::color::Color;
use crate::models::pixel::Pixel;

pub fn extract_palette_mediancut(pixels: &[Pixel], k: usize) -> Vec<Color> {
    if pixels.is_empty() || k == 0 {
        return Vec::new();
    }

    let mut boxes = vec![pixels.to_vec()];

    while boxes.len() < k {
        let Some(split_idx) = next_box_to_split(&boxes) else {
            break;
        };

        let mut current_box = boxes.swap_remove(split_idx);
        let channel = dominant_channel(&current_box);
        current_box.sort_by_key(|pixel| channel_value(pixel, channel));

        let mid = current_box.len() / 2;
        if mid == 0 || mid == current_box.len() {
            boxes.push(current_box);
            break;
        }

        let right = current_box.split_off(mid);
        let left = current_box;
        boxes.push(left);
        boxes.push(right);
    }

    boxes
        .into_iter()
        .filter(|bucket| !bucket.is_empty())
        .map(average_color)
        .collect()
}

fn next_box_to_split(boxes: &[Vec<Pixel>]) -> Option<usize> {
    boxes
        .iter()
        .enumerate()
        .filter(|(_, bucket)| bucket.len() > 1)
        .max_by_key(|(_, bucket)| largest_range(bucket))
        .map(|(idx, _)| idx)
}

fn dominant_channel(pixels: &[Pixel]) -> u8 {
    let (r_range, g_range, b_range) = channel_ranges(pixels);
    if r_range >= g_range && r_range >= b_range {
        0
    } else if g_range >= r_range && g_range >= b_range {
        1
    } else {
        2
    }
}

fn largest_range(pixels: &[Pixel]) -> u8 {
    let (r_range, g_range, b_range) = channel_ranges(pixels);
    r_range.max(g_range).max(b_range)
}

fn channel_ranges(pixels: &[Pixel]) -> (u8, u8, u8) {
    let mut r_min = u8::MAX;
    let mut r_max = u8::MIN;
    let mut g_min = u8::MAX;
    let mut g_max = u8::MIN;
    let mut b_min = u8::MAX;
    let mut b_max = u8::MIN;

    for pixel in pixels {
        r_min = r_min.min(pixel.r);
        r_max = r_max.max(pixel.r);
        g_min = g_min.min(pixel.g);
        g_max = g_max.max(pixel.g);
        b_min = b_min.min(pixel.b);
        b_max = b_max.max(pixel.b);
    }

    (
        r_max.saturating_sub(r_min),
        g_max.saturating_sub(g_min),
        b_max.saturating_sub(b_min),
    )
}

fn channel_value(pixel: &Pixel, channel: u8) -> u8 {
    match channel {
        0 => pixel.r,
        1 => pixel.g,
        _ => pixel.b,
    }
}

fn average_color(pixels: Vec<Pixel>) -> Color {
    let mut r_sum = 0u64;
    let mut g_sum = 0u64;
    let mut b_sum = 0u64;

    for pixel in &pixels {
        r_sum += pixel.r as u64;
        g_sum += pixel.g as u64;
        b_sum += pixel.b as u64;
    }

    let count = pixels.len() as u64;

    Color {
        r: (r_sum / count) as u8,
        g: (g_sum / count) as u8,
        b: (b_sum / count) as u8,
    }
}
