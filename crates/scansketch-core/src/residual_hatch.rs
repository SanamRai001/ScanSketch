//! P4-A.3: sparse residual hatching after the frozen Gesture + Form hierarchy.
//!
//! P2-B.1 remains a control only. The P4-A.3 candidate contains no legacy
//! straight Stroke records: Gesture, Form and Hatch are all first-class
//! PathStroke records with explicit roles.

use image::RgbaImage;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use crate::analysis::darkness_map;
use crate::long_paths::{
    generate_hierarchical_path_sketch_with_stats, HierarchyPathStats,
};
use crate::render::render_sketch;
use crate::scanline::{generate_sketch, SketchOptions};
use crate::stroke::{PathPoint, PathStroke, Sketch, StrokeRole};

const RESIDUAL_FLOOR: f32 = 0.06;
const RESIDUAL_GAIN: f32 = 0.58;
const HATCH_WHITE_THRESHOLD: f32 = 0.10;
const HATCH_BAND_HEIGHT: usize = 5;
const HATCH_SEGMENT_WIDTH: usize = 28;
const GESTURE_PROTECT_RADIUS: usize = 3;
const FORM_PROTECT_RADIUS: usize = 2;
const HATCH_MAX_CONTROL_FRACTION: f32 = 0.55;
const HATCH_MIN_LENGTH: f32 = 2.0;
const HATCH_MAX_LENGTH: f32 = 9.5;
const HATCH_RNG_XOR: u64 = 0xA3C7_5A11_9E20_4D3B;

#[derive(Clone, Debug, Default, Serialize)]
pub struct ResidualHatchStats {
    pub gesture_count: usize,
    pub form_count: usize,
    pub hatch_count: usize,
    pub legacy_control_segments: usize,
    pub hatch_budget: usize,
    pub residual_candidate_pixels: usize,
    pub structure_total_path_length_px: f64,
    pub hatch_total_path_length_px: f64,
    pub hatch_mean_path_length_px: f64,
    pub structure_length_share: f64,
    pub hatch_length_share: f64,
    pub hatch_count_reduction_fraction_vs_control: f64,
}

fn pixmap_darkness(sketch: &Sketch) -> Result<Vec<f32>, String> {
    let pixmap = render_sketch(sketch)?;
    let image = RgbaImage::from_raw(
        sketch.width,
        sketch.height,
        pixmap.data().to_vec(),
    )
    .ok_or_else(|| "unable to convert rendered hierarchy into RGBA pixels".to_string())?;
    Ok(darkness_map(&image))
}

fn mark_corridor(
    protected: &mut [bool],
    width: usize,
    height: usize,
    path: &PathStroke,
    radius: usize,
) {
    if width == 0 || height == 0 {
        return;
    }
    for pair in path.points.windows(2) {
        let dx = pair[1].x - pair[0].x;
        let dy = pair[1].y - pair[0].y;
        let length = dx.hypot(dy);
        let steps = (length / 0.7).ceil().max(1.0) as usize;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let xf = pair[0].x + dx * t;
            let yf = pair[0].y + dy * t;
            let cx = xf.floor().clamp(0.0, (width - 1) as f32) as usize;
            let cy = yf.floor().clamp(0.0, (height - 1) as f32) as usize;
            for y in cy.saturating_sub(radius)..=(cy + radius).min(height - 1) {
                for x in cx.saturating_sub(radius)..=(cx + radius).min(width - 1) {
                    let ox = x.abs_diff(cx);
                    let oy = y.abs_diff(cy);
                    if ox * ox + oy * oy <= radius * radius {
                        protected[y * width + x] = true;
                    }
                }
            }
        }
    }
}

fn residual_need(
    source: &RgbaImage,
    hierarchy: &Sketch,
) -> Result<(Vec<f32>, usize), String> {
    let width = source.width() as usize;
    let height = source.height() as usize;
    let source_darkness = darkness_map(source);
    let structure_darkness = pixmap_darkness(hierarchy)?;
    if source_darkness.len() != structure_darkness.len() {
        return Err("P4-A.3 residual maps have mismatched dimensions".into());
    }

    let mut protected = vec![false; source_darkness.len()];
    for path in &hierarchy.paths {
        match path.role {
            StrokeRole::Gesture => {
                mark_corridor(
                    &mut protected,
                    width,
                    height,
                    path,
                    GESTURE_PROTECT_RADIUS,
                );
            }
            StrokeRole::Form => {
                mark_corridor(
                    &mut protected,
                    width,
                    height,
                    path,
                    FORM_PROTECT_RADIUS,
                );
            }
            _ => {}
        }
    }

    let mut candidate_pixels = 0_usize;
    let need = source_darkness
        .into_iter()
        .zip(structure_darkness)
        .enumerate()
        .map(|(index, (source_d, structure_d))| {
            if protected[index] {
                return 0.0;
            }
            let raw = (source_d - structure_d).max(0.0);
            let compressed = ((raw - RESIDUAL_FLOOR).max(0.0) * RESIDUAL_GAIN).clamp(0.0, 1.0);
            if compressed > HATCH_WHITE_THRESHOLD {
                candidate_pixels += 1;
            }
            compressed
        })
        .collect();

    Ok((need, candidate_pixels))
}

fn append_hatch_run(
    paths: &mut Vec<PathStroke>,
    rng: &mut ChaCha8Rng,
    top: usize,
    band_height: usize,
    start_column: usize,
    end_column: usize,
    average_need: f32,
    width: usize,
    height: usize,
    hatch_budget: usize,
) {
    if paths.len() >= hatch_budget || end_column <= start_column {
        return;
    }

    let base_width = (0.40 + 0.30 * average_need).clamp(0.40, 0.68);
    let margin = base_width * 0.5 + 0.08;
    let left = start_column as f32 + margin;
    let right = end_column as f32 - margin;
    if right - left < 0.14 {
        return;
    }

    let min_y = top as f32 + base_width * 0.5 + 0.08;
    let max_y = ((top + band_height).min(height) as f32 - base_width * 0.5 - 0.08)
        .max(min_y);
    let nominal_y = top as f32 + band_height as f32 * 0.5;
    let start_offset = rng
        .gen_range(0.0_f32..1.2_f32)
        .min((right - left) * 0.12);
    let mut cursor = left + start_offset;

    while cursor + 0.14 < right && paths.len() < hatch_budget {
        let requested_length =
            (2.9 + 3.2 * average_need + rng.gen_range(-0.9_f32..1.9_f32))
                .clamp(HATCH_MIN_LENGTH, HATCH_MAX_LENGTH);
        let x1 = (cursor + requested_length).min(right);
        let span = x1 - cursor;
        if span < HATCH_MIN_LENGTH {
            break;
        }

        let mark_width = base_width * rng.gen_range(0.86_f32..1.0_f32);
        let y0 = (nominal_y + rng.gen_range(-0.42_f32..0.42_f32))
            .clamp(min_y, max_y);
        let y1 = (y0 + span * rng.gen_range(-0.075_f32..0.075_f32))
            .clamp(min_y, max_y);
        let opacity = ((0.13 + 0.58 * average_need)
            * rng.gen_range(0.80_f32..0.98_f32))
            .clamp(0.10, 0.50);

        paths.push(PathStroke {
            points: vec![
                PathPoint { x: cursor, y: y0 },
                PathPoint { x: x1, y: y1 },
            ],
            width: mark_width,
            opacity,
            role: StrokeRole::Hatch,
        });

        let gap = (1.6
            + (1.0 - average_need) * 2.5
            + rng.gen_range(0.35_f32..1.8_f32))
            .clamp(1.8, 5.6);
        cursor = x1 + gap;
    }

    if let Some(last) = paths.last() {
        debug_assert!(last
            .points
            .iter()
            .all(|p| p.x >= 0.0 && p.x <= width as f32 && p.y >= 0.0 && p.y <= height as f32));
    }
}

fn generate_hatches(
    need: &[f32],
    width: usize,
    height: usize,
    seed: u64,
    hatch_budget: usize,
) -> Vec<PathStroke> {
    if hatch_budget == 0 || width == 0 || height == 0 {
        return Vec::new();
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ HATCH_RNG_XOR);
    let mut paths = Vec::new();

    for top in (0..height).step_by(HATCH_BAND_HEIGHT) {
        if paths.len() >= hatch_budget {
            break;
        }
        let bottom = (top + HATCH_BAND_HEIGHT).min(height);
        let actual_band = bottom - top;
        if actual_band == 0 {
            continue;
        }

        for left in (0..width).step_by(HATCH_SEGMENT_WIDTH) {
            if paths.len() >= hatch_budget {
                break;
            }
            let right = (left + HATCH_SEGMENT_WIDTH).min(width);
            let mut run_start: Option<usize> = None;
            let mut run_need = 0.0_f32;
            let mut run_columns = 0_usize;

            for x in left..=right {
                let column_need = if x == right {
                    0.0
                } else {
                    let mut sum = 0.0_f32;
                    for y in top..bottom {
                        sum += need[y * width + x];
                    }
                    sum / actual_band as f32
                };

                if x < right && column_need > HATCH_WHITE_THRESHOLD {
                    if run_start.is_none() {
                        run_start = Some(x);
                    }
                    run_need += column_need;
                    run_columns += 1;
                } else if let Some(start) = run_start.take() {
                    append_hatch_run(
                        &mut paths,
                        &mut rng,
                        top,
                        actual_band,
                        start,
                        x,
                        run_need / run_columns.max(1) as f32,
                        width,
                        height,
                        hatch_budget,
                    );
                    run_need = 0.0;
                    run_columns = 0;
                }
            }
        }
    }

    paths
}

fn structure_paths(sketch: &Sketch) -> Vec<PathStroke> {
    sketch
        .paths
        .iter()
        .filter(|path| matches!(path.role, StrokeRole::Gesture | StrokeRole::Form))
        .cloned()
        .collect()
}

/// Generate the P4-A.3 hierarchy: exact frozen Gesture + Form paths followed by
/// sparse short residual Hatch paths.
///
/// include_structure controls the diagnostic view only:
/// true = Gesture + Form + Hatch (the P4-A.3 candidate)
/// false = Hatch only, generated from the exact same frozen structure.
pub fn generate_residual_hatch_sketch_with_stats(
    source: &RgbaImage,
    options: &SketchOptions,
    include_structure: bool,
) -> Result<(Sketch, ResidualHatchStats), String> {
    let (hierarchy, _hierarchy_stats): (Sketch, HierarchyPathStats) =
        generate_hierarchical_path_sketch_with_stats(source, options, false)?;

    let control = generate_sketch(source, options)?;
    let legacy_control_segments = control.strokes.len();

    let (need, residual_candidate_pixels) = residual_need(source, &hierarchy)?;
    let hatch_budget = ((legacy_control_segments as f32 * HATCH_MAX_CONTROL_FRACTION).floor()
        as usize)
        .min(options.max_strokes.saturating_sub(hierarchy.paths.len()));

    let hatches = generate_hatches(
        &need,
        source.width() as usize,
        source.height() as usize,
        options.seed,
        hatch_budget,
    );

    let structure = structure_paths(&hierarchy);
    let gesture_count = structure
        .iter()
        .filter(|path| path.role == StrokeRole::Gesture)
        .count();
    let form_count = structure
        .iter()
        .filter(|path| path.role == StrokeRole::Form)
        .count();

    let structure_total_path_length_px: f64 = if structure.is_empty() {
        0.0
    } else {
        structure.iter().map(PathStroke::path_length_px).sum()
    };
    let hatch_total_path_length_px: f64 =
        hatches.iter().map(PathStroke::path_length_px).sum();
    let hatch_count = hatches.len();
    let combined_length = structure_total_path_length_px + hatch_total_path_length_px;

    let mut paths = if include_structure {
        structure
    } else {
        Vec::new()
    };
    paths.extend(hatches);

    let sketch = Sketch {
        width: source.width(),
        height: source.height(),
        seed: options.seed,
        strokes: Vec::new(),
        paths,
    };
    if sketch.logical_mark_count() > options.max_strokes {
        return Err("P4-A.3 logical mark budget exceeded".into());
    }

    let reduction = if legacy_control_segments > 0 {
        1.0 - hatch_count as f64 / legacy_control_segments as f64
    } else {
        1.0
    };

    Ok((
        sketch,
        ResidualHatchStats {
            gesture_count,
            form_count,
            hatch_count,
            legacy_control_segments,
            hatch_budget,
            residual_candidate_pixels,
            structure_total_path_length_px,
            hatch_total_path_length_px,
            hatch_mean_path_length_px: if hatch_count > 0 {
                hatch_total_path_length_px / hatch_count as f64
            } else {
                0.0
            },
            structure_length_share: if combined_length > 0.0 {
                structure_total_path_length_px / combined_length
            } else {
                0.0
            },
            hatch_length_share: if combined_length > 0.0 {
                hatch_total_path_length_px / combined_length
            } else {
                0.0
            },
            hatch_count_reduction_fraction_vs_control: reduction,
        },
    ))
}

pub fn generate_residual_hatch_sketch(
    source: &RgbaImage,
    options: &SketchOptions,
    include_structure: bool,
) -> Result<Sketch, String> {
    generate_residual_hatch_sketch_with_stats(source, options, include_structure)
        .map(|(sketch, _)| sketch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn white(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
    }

    #[test]
    fn white_source_produces_no_residual_hatches() {
        let source = white(64, 64);
        let (sketch, stats) =
            generate_residual_hatch_sketch_with_stats(&source, &SketchOptions::default(), true)
                .unwrap();
        assert!(sketch.paths.is_empty());
        assert_eq!(stats.hatch_count, 0);
        assert_eq!(stats.legacy_control_segments, 0);
    }

    #[test]
    fn dark_field_uses_sparse_hatches_without_legacy_segments() {
        let source = RgbaImage::from_pixel(96, 96, Rgba([25, 25, 25, 255]));
        let options = SketchOptions::default();
        let control = generate_sketch(&source, &options).unwrap();
        let (sketch, stats) =
            generate_residual_hatch_sketch_with_stats(&source, &options, true).unwrap();

        assert!(sketch.strokes.is_empty());
        assert!(stats.hatch_count > 0);
        assert!(stats.hatch_count < control.strokes.len());
        assert!(stats.hatch_count as f64 <= control.strokes.len() as f64 * 0.56);
        assert!(sketch.paths.iter().all(|path| path.role == StrokeRole::Hatch));
    }

    #[test]
    fn hierarchy_paths_are_exactly_preserved_before_hatches() {
        let mut source = white(96, 96);
        for y in 16..80 {
            for x in 16..80 {
                source.put_pixel(x, y, Rgba([100, 100, 100, 255]));
            }
        }
        for y in 39..57 {
            for x in 47..50 {
                source.put_pixel(x, y, Rgba([20, 20, 20, 255]));
            }
        }

        let options = SketchOptions::default();
        let (hierarchy, _) =
            generate_hierarchical_path_sketch_with_stats(&source, &options, false).unwrap();
        let (final_sketch, stats) =
            generate_residual_hatch_sketch_with_stats(&source, &options, true).unwrap();

        let frozen: Vec<_> = final_sketch
            .paths
            .iter()
            .filter(|path| path.role != StrokeRole::Hatch)
            .cloned()
            .collect();
        assert_eq!(frozen, hierarchy.paths);
        assert_eq!(
            stats.gesture_count + stats.form_count,
            hierarchy.paths.len()
        );
        assert!(final_sketch.strokes.is_empty());
    }

    #[test]
    fn hatch_only_mode_matches_final_hatch_layer_exactly() {
        let source = RgbaImage::from_pixel(96, 96, Rgba([55, 55, 55, 255]));
        let options = SketchOptions::default();
        let (final_sketch, _) =
            generate_residual_hatch_sketch_with_stats(&source, &options, true).unwrap();
        let (hatch_only, _) =
            generate_residual_hatch_sketch_with_stats(&source, &options, false).unwrap();

        let final_hatches: Vec<_> = final_sketch
            .paths
            .iter()
            .filter(|path| path.role == StrokeRole::Hatch)
            .cloned()
            .collect();
        assert_eq!(final_hatches, hatch_only.paths);
        assert!(hatch_only
            .paths
            .iter()
            .all(|path| path.role == StrokeRole::Hatch));
        assert!(hatch_only.strokes.is_empty());
    }

    #[test]
    fn hatch_marks_stay_short_and_two_point() {
        let source = RgbaImage::from_pixel(96, 96, Rgba([35, 35, 35, 255]));
        let (sketch, _) =
            generate_residual_hatch_sketch_with_stats(
                &source,
                &SketchOptions::default(),
                false,
            )
            .unwrap();

        assert!(!sketch.paths.is_empty());
        assert!(sketch.paths.iter().all(|path| {
            let length = path.path_length_px();
            path.role == StrokeRole::Hatch
                && path.points.len() == 2
                && length >= HATCH_MIN_LENGTH as f64 - 0.01
                && length <= HATCH_MAX_LENGTH as f64 + 0.05
        }));
    }

    #[test]
    fn residual_hatching_is_deterministic() {
        let source = RgbaImage::from_pixel(96, 96, Rgba([75, 75, 75, 255]));
        let options = SketchOptions::default();
        let a = generate_residual_hatch_sketch(&source, &options, true).unwrap();
        let b = generate_residual_hatch_sketch(&source, &options, true).unwrap();
        assert_eq!(a, b);
    }
}
