use image::RgbaImage;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::analysis::darkness_map;
use crate::stroke::{Sketch, Stroke};

/// Intentionally limited P1 settings. Image dimensions are bounded separately.
#[derive(Clone, Debug)]
pub struct SketchOptions {
    pub seed: u64,
    pub band_height: u32,
    pub segment_width: u32,
    pub white_threshold: f32,
    pub max_strokes: usize,
}

impl Default for SketchOptions {
    fn default() -> Self {
        Self {
            seed: 42,
            band_height: 3,
            segment_width: 8,
            white_threshold: 0.08,
            max_strokes: 100_000,
        }
    }
}

/// A deterministic, deliberately simple top-to-bottom scanline baseline.
/// Darker regions receive denser, stronger pencil marks; near-white stays blank.
/// No optimization, erasing, edge tracing or contour-following is included.
pub fn generate_sketch(source: &RgbaImage, options: &SketchOptions) -> Result<Sketch, String> {
    let (width, height) = source.dimensions();
    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return Err("working image dimensions must be within 1..=1024 per side".into());
    }
    if !(1..=16).contains(&options.band_height)
        || !(2..=64).contains(&options.segment_width)
        || !options.white_threshold.is_finite()
        || !(0.0..=0.5).contains(&options.white_threshold)
        || !(1..=500_000).contains(&options.max_strokes)
    {
        return Err("invalid scan options: band 1..=16, segment 2..=64, threshold 0..=0.5, stroke budget 1..=500000".into());
    }

    let darkness = darkness_map(source);
    let mut rng = ChaCha8Rng::seed_from_u64(options.seed);
    let mut strokes = Vec::new();

    for top in (0..height).step_by(options.band_height as usize) {
        let bottom = top.saturating_add(options.band_height).min(height);
        let actual_band = bottom - top;
        for left in (0..width).step_by(options.segment_width as usize) {
            let right = left.saturating_add(options.segment_width).min(width);
            if right - left < 2 {
                continue;
            }

            // Fixed-order region sampling: the same image and seed yield the
            // same accepted paths on this engine version.
            let mut total = 0.0f32;
            for y in top..bottom {
                let row_offset = y as usize * width as usize;
                for x in left..right {
                    total += darkness[row_offset + x as usize];
                }
            }
            let avg = total / ((right - left) * actual_band) as f32;
            if avg <= options.white_threshold {
                continue;
            }

            // Two separated thin marks in deep shadow rather than a wide fill.
            let count = if avg > 0.66 && actual_band >= 3 { 2 } else { 1 };
            for layer in 0..count {
                let start = left as f32 + 0.18 + rng.gen_range(0.0..0.35);
                let end = right as f32 - 0.18 - rng.gen_range(0.0..0.35);
                if end <= start {
                    continue;
                }
                let band_fraction = if count == 2 {
                    if layer == 0 { 0.30 } else { 0.72 }
                } else {
                    0.50
                };
                let y = (top as f32
                    + actual_band as f32 * band_fraction
                    + rng.gen_range(-0.12..0.12))
                    .clamp(0.35, height as f32 - 0.35);

                let opacity = ((0.18 + 0.70 * avg) * rng.gen_range(0.94..1.0)).clamp(0.0, 1.0);
                let width = (0.42 + 0.48 * avg).min(actual_band as f32 * 0.55);

                if strokes.len() >= options.max_strokes {
                    return Err("stroke budget exceeded; increase --max-strokes or lower detail".into());
                }
                strokes.push(Stroke {
                    x0: start,
                    y0: y,
                    x1: end,
                    y1: y,
                    width,
                    opacity,
                });
            }
        }
    }
    Ok(Sketch {
        width,
        height,
        seed: options.seed,
        strokes,
    })
}
