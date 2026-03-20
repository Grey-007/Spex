pub use crate::color_utils::LabColor;
use crate::color_utils::{lab_to_rgb, rgb_to_lab};
use crate::models::color::Color;
use crate::models::pixel::Pixel;

const MAX_ITERATIONS: usize = 25;
const CONVERGENCE_EPSILON: f32 = 0.001;

#[derive(Debug, Clone)]
pub struct LabCluster {
    pub lab: LabColor,
    pub color: Color,
    pub size: usize,
}

pub fn extract_clusters(pixels: &[Pixel], k: usize) -> Vec<LabCluster> {
    if pixels.is_empty() || k == 0 {
        return Vec::new();
    }

    let points: Vec<LabColor> = pixels.iter().map(pixel_to_lab).collect();
    let cluster_count = k.min(points.len());
    let mut centers = init_centers(&points, cluster_count);
    let mut assignments = vec![0usize; points.len()];

    for _ in 0..MAX_ITERATIONS {
        for (idx, point) in points.iter().enumerate() {
            assignments[idx] = nearest_center(point, &centers);
        }

        let mut sums = vec![
            LabColor {
                l: 0.0,
                a: 0.0,
                b: 0.0,
            };
            cluster_count
        ];
        let mut counts = vec![0usize; cluster_count];

        for (point, &cluster_idx) in points.iter().zip(assignments.iter()) {
            sums[cluster_idx].l += point.l;
            sums[cluster_idx].a += point.a;
            sums[cluster_idx].b += point.b;
            counts[cluster_idx] += 1;
        }

        let mut max_shift = 0.0_f32;

        for center_idx in 0..cluster_count {
            if counts[center_idx] == 0 {
                continue;
            }

            let count = counts[center_idx] as f32;
            let new_center = LabColor {
                l: sums[center_idx].l / count,
                a: sums[center_idx].a / count,
                b: sums[center_idx].b / count,
            };

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

    centers
        .into_iter()
        .zip(cluster_sizes)
        .filter(|(_, size)| *size > 0)
        .map(|(lab, size)| LabCluster {
            color: lab_to_rgb(lab),
            lab,
            size,
        })
        .collect()
}

fn init_centers(points: &[LabColor], k: usize) -> Vec<LabColor> {
    (0..k)
        .map(|i| {
            let idx = i * points.len() / k;
            points[idx]
        })
        .collect()
}

fn nearest_center(point: &LabColor, centers: &[LabColor]) -> usize {
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

fn squared_distance(a: &LabColor, b: &LabColor) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    dl * dl + da * da + db * db
}

fn pixel_to_lab(pixel: &Pixel) -> LabColor {
    rgb_to_lab(Color {
        r: pixel.r,
        g: pixel.g,
        b: pixel.b,
    })
}
