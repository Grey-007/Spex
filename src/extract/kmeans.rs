use crate::models::color::Color;
use crate::models::pixel::Pixel;

const MAX_ITERATIONS: usize = 25;
const CONVERGENCE_EPSILON: f32 = 0.001;

/// Extracts `k` dominant colors using k-means clustering in LAB space.
pub fn extract_palette(pixels: &[Pixel], k: usize) -> Vec<Color> {
    let (colors, _) = extract_palette_with_sizes(pixels, k);
    colors
}

/// Extracts `k` dominant colors and cluster sizes using k-means clustering in LAB space.
pub fn extract_palette_with_sizes(pixels: &[Pixel], k: usize) -> (Vec<Color>, Vec<usize>) {
    if pixels.is_empty() || k == 0 {
        return (Vec::new(), Vec::new());
    }

    let points: Vec<[f32; 3]> = pixels.iter().map(pixel_to_lab).collect();
    let cluster_count = k.min(points.len());
    let mut centers = init_centers(&points, cluster_count);
    let mut assignments = vec![0usize; points.len()];

    for _ in 0..MAX_ITERATIONS {
        for (idx, point) in points.iter().enumerate() {
            assignments[idx] = nearest_center(point, &centers);
        }

        let mut sums = vec![[0.0_f32; 3]; cluster_count];
        let mut counts = vec![0usize; cluster_count];

        for (point, &cluster_idx) in points.iter().zip(assignments.iter()) {
            sums[cluster_idx][0] += point[0];
            sums[cluster_idx][1] += point[1];
            sums[cluster_idx][2] += point[2];
            counts[cluster_idx] += 1;
        }

        let mut max_shift = 0.0_f32;

        for center_idx in 0..cluster_count {
            if counts[center_idx] == 0 {
                continue;
            }

            let count = counts[center_idx] as f32;
            let new_center = [
                sums[center_idx][0] / count,
                sums[center_idx][1] / count,
                sums[center_idx][2] / count,
            ];

            let shift = squared_distance(&centers[center_idx], &new_center).sqrt();
            if shift > max_shift {
                max_shift = shift;
            }

            centers[center_idx] = new_center;
        }

        if max_shift < CONVERGENCE_EPSILON {
            break;
        }
    }

    for (idx, point) in points.iter().enumerate() {
        assignments[idx] = nearest_center(point, &centers);
    }

    let mut cluster_sizes = vec![0usize; cluster_count];
    for &cluster_idx in &assignments {
        cluster_sizes[cluster_idx] += 1;
    }

    let colors = centers.into_iter().map(lab_to_color).collect();
    (colors, cluster_sizes)
}

fn init_centers(points: &[[f32; 3]], k: usize) -> Vec<[f32; 3]> {
    (0..k)
        .map(|i| {
            let idx = i * points.len() / k;
            points[idx]
        })
        .collect()
}

fn nearest_center(point: &[f32; 3], centers: &[[f32; 3]]) -> usize {
    let mut best_idx = 0usize;
    let mut best_distance = squared_distance(point, &centers[0]);

    for (idx, center) in centers.iter().enumerate().skip(1) {
        let dist = squared_distance(point, center);
        if dist < best_distance {
            best_distance = dist;
            best_idx = idx;
        }
    }

    best_idx
}

fn squared_distance(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dl = a[0] - b[0];
    let da = a[1] - b[1];
    let db = a[2] - b[2];
    dl * dl + da * da + db * db
}

fn pixel_to_lab(pixel: &Pixel) -> [f32; 3] {
    let r = srgb_to_linear(pixel.r as f32 / 255.0);
    let g = srgb_to_linear(pixel.g as f32 / 255.0);
    let b = srgb_to_linear(pixel.b as f32 / 255.0);

    let x = (0.412_456_4 * r) + (0.357_576_1 * g) + (0.180_437_5 * b);
    let y = (0.212_672_9 * r) + (0.715_152_2 * g) + (0.072_175 * b);
    let z = (0.019_333_9 * r) + (0.119_192 * g) + (0.950_304_1 * b);

    xyz_to_lab(x, y, z)
}

fn lab_to_color(lab: [f32; 3]) -> Color {
    let (x, y, z) = lab_to_xyz(lab[0], lab[1], lab[2]);

    let r_linear = (3.240_454_2 * x) + (-1.537_138_5 * y) + (-0.498_531_4 * z);
    let g_linear = (-0.969_266 * x) + (1.876_010_8 * y) + (0.041_556 * z);
    let b_linear = (0.055_643_4 * x) + (-0.204_025_9 * y) + (1.057_225_2 * z);

    let r = linear_to_srgb(r_linear).clamp(0.0, 1.0);
    let g = linear_to_srgb(g_linear).clamp(0.0, 1.0);
    let b = linear_to_srgb(b_linear).clamp(0.0, 1.0);

    Color {
        r: (r * 255.0).round() as u8,
        g: (g * 255.0).round() as u8,
        b: (b * 255.0).round() as u8,
    }
}

fn xyz_to_lab(x: f32, y: f32, z: f32) -> [f32; 3] {
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

    [l, a, b]
}

fn lab_to_xyz(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    const XN: f32 = 0.950_47;
    const YN: f32 = 1.0;
    const ZN: f32 = 1.088_83;
    const EPSILON: f32 = 216.0 / 24_389.0;
    const KAPPA: f32 = 24_389.0 / 27.0;

    let fy = (l + 16.0) / 116.0;
    let fx = fy + (a / 500.0);
    let fz = fy - (b / 200.0);

    let xr = f_inv_lab(fx, EPSILON, KAPPA);
    let yr = f_inv_lab(fy, EPSILON, KAPPA);
    let zr = f_inv_lab(fz, EPSILON, KAPPA);

    (xr * XN, yr * YN, zr * ZN)
}

fn f_lab(t: f32, epsilon: f32, kappa: f32) -> f32 {
    if t > epsilon {
        t.cbrt()
    } else {
        (kappa * t + 16.0) / 116.0
    }
}

fn f_inv_lab(t: f32, epsilon: f32, kappa: f32) -> f32 {
    let t3 = t * t * t;
    if t3 > epsilon {
        t3
    } else {
        (116.0 * t - 16.0) / kappa
    }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
