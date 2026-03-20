# Spex Extraction Pipeline

## Overview
Spex now uses LAB-based k-means as its default palette extractor.
The goal is to keep dominant wallpaper colors, improve perceptual clustering, and still fall back to `mediancut` when the result is low quality.

High-level flow:
1. load the source image
2. resize it to a maximum working size of `512px`
3. sample pixels for performance
4. convert sampled RGB pixels into LAB
5. run k-means clustering in LAB space
6. convert LAB centroids back to RGB
7. remove near-duplicate colors with Delta-E filtering
8. apply a restrained vibrancy pass when the palette is too dull
9. check palette quality
10. fall back to `mediancut` if k-means quality is too low

## 1. Image Preprocessing
Spex does not cluster the full original image directly.
It first resizes the image with:

```rust
let small = img.thumbnail(512, 512);
```

This preserves aspect ratio while keeping clustering fast enough for CLI use.

After resizing, Spex samples pixels using a stride that scales with image size.
This keeps the clustering workload bounded without changing the overall color distribution too aggressively.

## 2. LAB Conversion
Sampled pixels are converted from `RGB -> XYZ -> LAB`.
LAB is used because Euclidean distance in LAB is much closer to human color perception than Euclidean distance in RGB.

Each sampled point is stored as:

```rust
pub struct LabColor {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}
```

## 3. K-Means Clustering
Spex runs k-means on LAB points and also tracks the size of each cluster.

Cluster sizes matter because they reflect how common a color region is in the wallpaper.
Large clusters are prioritized so the final palette does not overfit rare accent pixels.

## 4. Delta-E Filtering
After clustering, centroids are converted back to RGB and filtered with CIE76 Delta-E.

Spex removes colors that are too close together:
- colors with `Delta-E < 10` are treated as near-duplicates
- nearby centroids are skipped in favor of the next cluster

This helps keep the palette visually distinct instead of returning multiple almost-identical shades.

## 5. Vibrancy Boost
If the extracted palette is too dull, Spex applies a restrained enhancement pass.

Trigger:
- average saturation below `0.25`

Behavior:
- saturation is increased slightly, typically in the `20-40%` range
- contrast is increased slightly
- wallpaper tone is preserved as much as possible

If the palette is almost grayscale, Spex can inject a very small hue hint based on the wallpaper's dominant color region.
This keeps dull wallpapers from collapsing into flat gray ramps without inventing unrelated colors.

## 6. Quality Check and Fallback
After the palette is generated, Spex evaluates it.

It marks the palette as low quality when:
- too many color pairs are still closer than `Delta-E < 8`
- average saturation is below `0.15`

If the requested extractor is `kmeans` and the result is low quality, Spex automatically falls back to `mediancut`.

This means:
- `kmeans` is the default extractor
- `mediancut` remains available explicitly
- `mediancut` is also the safety fallback when k-means underperforms

## 7. Sorting
The final extracted palette is sorted by luminance from dark to light:

```text
dark -> light
```

This makes previews and downstream processing stable and predictable.

## 8. Debugging
Two flags help inspect extraction behavior:

- `--debug-extractor`
  Prints requested/final extractor, LAB cluster centroids, cluster sizes, saturation, Delta-E stats, and whether fallback triggered.

- `--debug-palette`
  Prints final palette colors with luminance, saturation, and nearest Delta-E distances.

Use them together when tuning clustering behavior on difficult wallpapers.
