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

/// Commit one continuous dark-column run as actual ink, never a filled gray block.
/// Margins account for round stroke caps so a white neighboring column stays blank.
fn append_run(
    strokes: &mut Vec<Stroke>,
    rng: &mut ChaCha8Rng,
    options: &SketchOptions,
    top: u32,
    band_height: u32,
    start_column: u32,
    end_column: u32,
    average_darkness: f32,
) -> Result<(), String> {
    let columns = end_column - start_column;
    if columns == 0 {
        return Ok(());
    }
    let count = if average_darkness > 0.66 && band_height >= 3 {
        2
    } else {
        1
    };

    for layer in 0..count {
        let base_width = (0.42 + 0.48 * average_darkness).min(band_height as f32 * 0.55);
        // A one-pixel run must not spill significant ink into a neighboring
        // white column after round caps and antialiasing are applied.
        let width = if columns == 1 {
            base_width.min(0.65)
        } else {
            base_width
        };
        let margin = width * 0.5 + 0.04 + rng.gen_range(0.0..0.04);
        let x0 = start_column as f32 + margin;
        let x1 = end_column as f32 - margin;
        if x1 <= x0 {
            continue;
        }

        let fraction = if count == 2 {
            if layer == 0 { 0.30 } else { 0.72 }
        } else {
            0.50
        };
        let y = (top as f32
            + band_height as f32 * fraction
            + rng.gen_range(-0.10..0.10))
        .clamp(
            top as f32 + width * 0.5,
            (top + band_height) as f32 - width * 0.5,
        );
        let opacity = ((0.18 + 0.70 * average_darkness) * rng.gen_range(0.94..1.0))
            .clamp(0.0, 1.0);

        if strokes.len() >= options.max_strokes {
            return Err("stroke budget exceeded; increase --max-strokes or lower detail".into());
        }
        strokes.push(Stroke {
            x0,
            y0: y,
            x1,
            y1: y,
            width,
            opacity,
        });
    }
    Ok(())
}

/// A deterministic, deliberately simple top-to-bottom scanline baseline.
///
/// Each horizontal region is split further at bright *columns*, avoiding the
/// earlier bug where a region average could draw straight across a white gap.
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
            let mut run_start: Option<u32> = None;
            let mut run_darkness = 0.0f32;
            let mut run_columns = 0_u32;

            // Include a virtual blank column at right to flush any final run.
            for x in left..=right {
                let column_darkness = if x == right {
                    0.0
                } else {
                    let mut sum = 0.0;
                    for y in top..bottom {
                        sum += darkness[y as usize * width as usize + x as usize];
                    }
                    sum / actual_band as f32
                };

                if x < right && column_darkness > options.white_threshold {
                    if run_start.is_none() {
                        run_start = Some(x);
                    }
                    run_darkness += column_darkness;
                    run_columns += 1;
                } else if let Some(start) = run_start.take() {
                    append_run(
                        &mut strokes,
                        &mut rng,
                        options,
                        top,
                        actual_band,
                        start,
                        x,
                        run_darkness / run_columns as f32,
                    )?;
                    run_darkness = 0.0;
                    run_columns = 0;
                }
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
