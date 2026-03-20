use crate::color_utils::{
    PaletteEnhancementDebug, average_saturation, delta_e, dominant_hue_hint_from_pixels,
    enhance_palette, luminance, saturation,
};
use crate::extract::kmeans::{LabCluster, extract_clusters};
use crate::extract::mediancut::extract_palette_mediancut;
use crate::models::color::Color;
use crate::models::pixel::Pixel;

const DELTA_E_DEDUP_THRESHOLD: f32 = 10.0;
const LOW_QUALITY_DELTA_E_THRESHOLD: f32 = 8.0;
const LOW_QUALITY_SATURATION_THRESHOLD: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractorMethod {
    Kmeans,
    Mediancut,
}

impl ExtractorMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            ExtractorMethod::Kmeans => "kmeans",
            ExtractorMethod::Mediancut => "mediancut",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaletteQualityReport {
    pub average_saturation: f32,
    pub low_distance_pairs: usize,
    pub min_delta_e: Option<f32>,
    pub low_quality: bool,
}

#[derive(Debug, Clone)]
pub struct ExtractorReport {
    pub requested_method: ExtractorMethod,
    pub final_method: ExtractorMethod,
    pub fallback_triggered: bool,
    pub kmeans_clusters: Vec<LabCluster>,
    pub enhancement: PaletteEnhancementDebug,
    pub quality: PaletteQualityReport,
}

#[derive(Debug, Clone)]
pub struct ExtractionOutcome {
    pub palette: Vec<Color>,
    pub report: ExtractorReport,
}

pub fn extract_palette_with_fallback(
    pixels: &[Pixel],
    k: usize,
    requested_method: ExtractorMethod,
) -> ExtractionOutcome {
    if pixels.is_empty() || k == 0 {
        return ExtractionOutcome {
            palette: Vec::new(),
            report: ExtractorReport {
                requested_method,
                final_method: requested_method,
                fallback_triggered: false,
                kmeans_clusters: Vec::new(),
                enhancement: PaletteEnhancementDebug {
                    average_saturation: 0.0,
                    saturation_threshold: 0.25,
                    saturation_factor: 1.0,
                    contrast_factor: 1.0,
                    vibrancy_boost_applied: false,
                    grayscale_injection_applied: false,
                    dominant_hue_hint: None,
                },
                quality: PaletteQualityReport {
                    average_saturation: 0.0,
                    low_distance_pairs: 0,
                    min_delta_e: None,
                    low_quality: false,
                },
            },
        };
    }

    match requested_method {
        ExtractorMethod::Mediancut => {
            let (palette, enhancement) =
                finalize_palette(extract_palette_mediancut(pixels, k), pixels);
            let quality = assess_palette_quality(&palette);

            ExtractionOutcome {
                palette,
                report: ExtractorReport {
                    requested_method,
                    final_method: ExtractorMethod::Mediancut,
                    fallback_triggered: false,
                    kmeans_clusters: Vec::new(),
                    enhancement,
                    quality,
                },
            }
        }
        ExtractorMethod::Kmeans => {
            let cluster_target = k.saturating_mul(2).max(k).min(pixels.len());
            let clusters = prioritize_clusters(extract_clusters(pixels, cluster_target));
            let deduplicated = deduplicate_clusters(&clusters, k);
            let (palette, enhancement) = finalize_palette(deduplicated, pixels);
            let quality = assess_palette_quality(&palette);

            if quality.low_quality {
                let (fallback_palette, fallback_enhancement) =
                    finalize_palette(extract_palette_mediancut(pixels, k), pixels);
                let fallback_quality = assess_palette_quality(&fallback_palette);

                return ExtractionOutcome {
                    palette: fallback_palette,
                    report: ExtractorReport {
                        requested_method,
                        final_method: ExtractorMethod::Mediancut,
                        fallback_triggered: true,
                        kmeans_clusters: clusters,
                        enhancement: fallback_enhancement,
                        quality: fallback_quality,
                    },
                };
            }

            ExtractionOutcome {
                palette,
                report: ExtractorReport {
                    requested_method,
                    final_method: ExtractorMethod::Kmeans,
                    fallback_triggered: false,
                    kmeans_clusters: clusters,
                    enhancement,
                    quality,
                },
            }
        }
    }
}

fn prioritize_clusters(mut clusters: Vec<LabCluster>) -> Vec<LabCluster> {
    clusters.sort_by(|a, b| {
        b.size
            .cmp(&a.size)
            .then_with(|| saturation(b.color).total_cmp(&saturation(a.color)))
            .then_with(|| luminance(a.color).total_cmp(&luminance(b.color)))
    });
    clusters
}

fn deduplicate_clusters(clusters: &[LabCluster], k: usize) -> Vec<Color> {
    let mut palette = Vec::new();

    for cluster in clusters {
        if palette
            .iter()
            .all(|existing| delta_e(*existing, cluster.color) >= DELTA_E_DEDUP_THRESHOLD)
        {
            palette.push(cluster.color);
        }

        if palette.len() >= k {
            break;
        }
    }

    if palette.len() < k {
        for cluster in clusters {
            if palette.len() >= k {
                break;
            }

            if !palette.contains(&cluster.color) {
                palette.push(cluster.color);
            }
        }
    }

    palette.truncate(k);
    palette
}

fn finalize_palette(
    colors: Vec<Color>,
    pixels: &[Pixel],
) -> (Vec<Color>, PaletteEnhancementDebug) {
    let dominant_hue_hint = dominant_hue_hint_from_pixels(pixels);
    let (mut palette, enhancement) = enhance_palette(colors, dominant_hue_hint);
    palette.sort_by(|a, b| luminance(*a).total_cmp(&luminance(*b)));
    (palette, enhancement)
}

fn assess_palette_quality(palette: &[Color]) -> PaletteQualityReport {
    let mut low_distance_pairs = 0usize;
    let mut min_delta_e: Option<f32> = None;

    for left in 0..palette.len() {
        for right in (left + 1)..palette.len() {
            let distance = delta_e(palette[left], palette[right]);
            min_delta_e = Some(match min_delta_e {
                Some(current) => current.min(distance),
                None => distance,
            });

            if distance < LOW_QUALITY_DELTA_E_THRESHOLD {
                low_distance_pairs += 1;
            }
        }
    }

    let average_saturation = average_saturation(palette);
    let close_pair_limit = (palette.len() / 4).max(1);
    let low_quality = low_distance_pairs > close_pair_limit
        || average_saturation < LOW_QUALITY_SATURATION_THRESHOLD;

    PaletteQualityReport {
        average_saturation,
        low_distance_pairs,
        min_delta_e,
        low_quality,
    }
}

#[cfg(test)]
mod tests {
    use super::{PaletteQualityReport, assess_palette_quality, deduplicate_clusters};
    use crate::extract::kmeans::{LabCluster, LabColor};
    use crate::models::color::Color;

    #[test]
    fn deduplicate_clusters_prefers_large_distinct_clusters() {
        let clusters = vec![
            LabCluster {
                lab: LabColor {
                    l: 45.0,
                    a: 62.0,
                    b: 40.0,
                },
                color: Color {
                    r: 180,
                    g: 40,
                    b: 30,
                },
                size: 200,
            },
            LabCluster {
                lab: LabColor {
                    l: 46.0,
                    a: 60.0,
                    b: 38.0,
                },
                color: Color {
                    r: 176,
                    g: 42,
                    b: 35,
                },
                size: 180,
            },
            LabCluster {
                lab: LabColor {
                    l: 38.0,
                    a: -12.0,
                    b: -48.0,
                },
                color: Color {
                    r: 32,
                    g: 88,
                    b: 190,
                },
                size: 120,
            },
        ];

        let palette = deduplicate_clusters(&clusters, 2);
        assert_eq!(palette.len(), 2);
        assert_eq!(
            palette[0],
            Color {
                r: 180,
                g: 40,
                b: 30,
            }
        );
        assert_eq!(
            palette[1],
            Color {
                r: 32,
                g: 88,
                b: 190,
            }
        );
    }

    #[test]
    fn quality_check_flags_dull_palette() {
        let quality: PaletteQualityReport = assess_palette_quality(&[
            Color {
                r: 24,
                g: 24,
                b: 24,
            },
            Color {
                r: 48,
                g: 48,
                b: 48,
            },
            Color {
                r: 96,
                g: 96,
                b: 96,
            },
            Color {
                r: 160,
                g: 160,
                b: 160,
            },
        ]);

        assert!(quality.low_quality);
        assert!(quality.average_saturation < 0.15);
    }
}
