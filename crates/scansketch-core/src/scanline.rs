use image::RgbaImage;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::analysis::darkness_map;
use crate::stroke::{Sketch, Stroke};

/// P2-A: preserve the deterministic scan, but draw broken pencil fragments.
/// Image dimensions and stroke budgets are bounded separately.
#[derive(Clone, Debug)]
pub struct SketchOptions {
    pub seed: u64,
    pub band_height: u32,
    pub segment_width: u32,
    pub white_threshold: f32,
    pub max_strokes: usize,
    /// Source-derived structural strokes, drawn after the P2-A tonal marks.
    pub enable_contours: bool,
    /// Minimum normalized Sobel strength for a contour candidate (0..=1).
    pub contour_threshold: f32,
}

impl Default for SketchOptions {
    fn default() -> Self {
        Self {
            seed: 42,
            band_height: 3,
            // Give each region room for several fragments instead of creating
            // an artificial boundary every eight pixels.
            segment_width: 24,
            white_threshold: 0.08,
            max_strokes: 100_000,
            enable_contours: true,
            contour_threshold: 0.28,
        }
    }
}

/// Turn one continuous eligible-column run into bounded, separated strokes.
/// Stroke-cap margins keep neighboring white columns free of ink.
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

    let base_width = (0.34 + 0.46 * average_darkness).min(band_height as f32 * 0.55);
    // Narrow one-column features need smaller round caps to preserve bright
    // neighboring columns. Use the maximum potential stroke width for margins.
    let max_width = if columns == 1 {
        base_width.min(0.60)
    } else {
        base_width
    };
    let margin = max_width * 0.5 + 0.07;
    let left = start_column as f32 + margin;
    let right = end_column as f32 - margin;
    if right - left < 0.12 {
        return Ok(());
    }

    let min_y = top as f32 + max_width * 0.5 + 0.06;
    let max_y = (top + band_height) as f32 - max_width * 0.5 - 0.06;
    // A second, independently fragmented line adds density in deep shadows.
    // In narrow scan bands there is not enough vertical space for two layers.
    let layers = if average_darkness > 0.72 && band_height >= 3 {
        2
    } else {
        1
    };

    for layer in 0..layers {
        let fraction = if layers == 2 {
            if layer == 0 { 0.32 } else { 0.68 }
        } else {
            0.50
        };

        // A different first-fragment phase in each band/layer prevents aligned
        // vertical seams. The phase is bounded for one-column runs too.
        let start_offset = rng
            .gen_range(0.0_f32..0.8_f32)
            .min((right - left) * 0.10);
        let mut cursor = left + start_offset;

        while cursor + 0.12 < right {
            // Tone affects ink strength and spacing, not a continuous gray bar.
            let requested_length =
                (3.1 + 3.8 * average_darkness + rng.gen_range(-1.1_f32..2.2_f32))
                    .clamp(2.0, 10.5);
            let x1 = (cursor + requested_length).min(right);
            let span = x1 - cursor;
            if span < 0.12 {
                break;
            }

            let width = max_width * rng.gen_range(0.84_f32..1.0_f32);
            let nominal_y = top as f32 + band_height as f32 * fraction;
            let y0 = (nominal_y + rng.gen_range(-0.16_f32..0.16_f32))
                .clamp(min_y, max_y);
            // Tiny endpoint variation gives each mark an angle while its
            // entire rounded stroke remains within the current scan band.
            let y1 = (y0 + span * rng.gen_range(-0.055_f32..0.055_f32))
                .clamp(min_y, max_y);
            let opacity = ((0.14 + 0.72 * average_darkness)
                * rng.gen_range(0.82_f32..1.0_f32))
                .clamp(0.0, 1.0);

            if strokes.len() >= options.max_strokes {
                return Err("stroke budget exceeded; increase --max-strokes or lower detail".into());
            }
            strokes.push(Stroke {
                x0: cursor,
                y0,
                x1,
                y1,
                width,
                opacity,
            });

            // Dark regions get narrower gaps; pale regions keep more paper.
            // The positive minimum also guarantees forward progress.
            let gap = (0.5
                + (1.0 - average_darkness) * 1.3
                + rng.gen_range(0.2_f32..1.4_f32))
                .clamp(0.7, 3.5);
            cursor = x1 + gap;
        }
    }

    Ok(())
}

/// Deterministic top-to-bottom sketch generation with broken tonal fragments.
///
/// Bright columns split runs before rendering. Every fragment is spatially
/// bounded to its dark run and scan band. P2-B optionally appends sparse,
/// source-derived contour marks; no candidate optimization or erasure.
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
        || !options.contour_threshold.is_finite()
        || !(0.0..=1.0).contains(&options.contour_threshold)
    {
        return Err("invalid scan options: band 1..=16, segment 2..=64, white threshold 0..=0.5, contour threshold 0..=1, stroke budget 1..=500000".into());
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

    if options.enable_contours {
        // Independent RNG: enabling this pass must not change a single P2-A
        // tonal stroke, which permits exact same-input A/B comparisons.
        crate::contour::append_contours(
            &darkness,
            width,
            height,
            options,
            &mut strokes,
        )?;
    }

    Ok(Sketch::from_strokes(width, height, options.seed, strokes))
}
