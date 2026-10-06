//! P4-A.1: sparse long structural / gestural path tracing.
//!
//! This is intentionally separate from P2/P3 selection experiments. Long
//! paths are proposed from source edge corridors, integrated along the
//! multiscale tangent field, and stored as first-class PathStroke records.

use image::RgbaImage;
use serde::Serialize;

use crate::analysis::darkness_map;
use crate::directional::{Direction, OrientationField};
use crate::metrics::edge_strength_map;
use crate::scanline::{generate_sketch, SketchOptions};
use crate::stroke::{PathPoint, PathStroke, Sketch, StrokeRole};

const SEED_CELL: usize = 6;
const MIN_SEED_EDGE: f32 = 0.16;
const MIN_TRACE_EDGE: f32 = 0.095;
const MIN_COHERENCE: f32 = 0.42;
const STEP_PX: f32 = 2.0;
const MAX_HALF_LENGTH_PX: f32 = 48.0;
const MIN_PATH_LENGTH_PX: f64 = 24.0;
const MAX_PATH_LENGTH_PX: f64 = 100.0;
const COVER_RADIUS: usize = 6;
const SNAP_RADIUS: isize = 2;
const MIN_TURN_DOT: f32 = 0.68;

#[derive(Clone, Debug, Default, Serialize)]
pub struct LongPathStats {
    pub seed_candidates: usize,
    pub accepted_paths: usize,
    pub rejected_short: usize,
    pub rejected_support: usize,
    pub baseline_segment_count: usize,
    pub total_path_length_px: f64,
    pub mean_path_length_px: f64,
    pub max_path_length_px: f64,
}

#[derive(Clone, Copy, Debug)]
struct Seed {
    x: usize,
    y: usize,
    score: f32,
}

fn direction_quality(direction: Direction) -> f32 {
    let strength = direction.energy / (direction.energy + 0.03);
    (direction.coherence * strength).clamp(0.0, 1.0)
}

fn best_direction(field: &OrientationField, x: usize, y: usize) -> Option<Direction> {
    let (fine, coarse) = field.components(x, y);
    let mut best = None;
    let mut best_quality = 0.0_f32;
    for direction in [coarse, fine] {
        if direction.coherence < MIN_COHERENCE || direction.energy < 0.0015 {
            continue;
        }
        let quality = direction_quality(direction);
        if quality > best_quality {
            best_quality = quality;
            best = Some(direction);
        }
    }
    best
}

fn collect_seeds(
    edge: &[f32],
    field: &OrientationField,
    width: usize,
    height: usize,
) -> Vec<Seed> {
    let mut seeds = Vec::new();
    if width < 5 || height < 5 {
        return seeds;
    }

    for top in (1..height - 1).step_by(SEED_CELL) {
        for left in (1..width - 1).step_by(SEED_CELL) {
            let bottom = (top + SEED_CELL).min(height - 1);
            let right = (left + SEED_CELL).min(width - 1);
            let mut best: Option<Seed> = None;
            for y in top..bottom {
                for x in left..right {
                    let e = edge[y * width + x];
                    if e < MIN_SEED_EDGE {
                        continue;
                    }
                    let Some(direction) = best_direction(field, x, y) else {
                        continue;
                    };
                    let score = e * (0.55 + 0.45 * direction_quality(direction));
                    if best.is_none_or(|seed| score > seed.score) {
                        best = Some(Seed { x, y, score });
                    }
                }
            }
            if let Some(seed) = best {
                seeds.push(seed);
            }
        }
    }

    seeds.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.y.cmp(&b.y))
            .then_with(|| a.x.cmp(&b.x))
    });
    seeds
}

fn align(direction: Direction, hx: f32, hy: f32) -> (f32, f32) {
    let mut tx = direction.tx;
    let mut ty = direction.ty;
    if tx * hx + ty * hy < 0.0 {
        tx = -tx;
        ty = -ty;
    }
    (tx, ty)
}

fn normalize(x: f32, y: f32) -> Option<(f32, f32)> {
    let length = x.hypot(y);
    (length > 0.0001).then(|| (x / length, y / length))
}

fn snap_to_edge(
    proposed_x: f32,
    proposed_y: f32,
    edge: &[f32],
    field: &OrientationField,
    width: usize,
    height: usize,
    prev_hx: f32,
    prev_hy: f32,
) -> Option<(f32, f32, f32, f32)> {
    if proposed_x < 1.0
        || proposed_y < 1.0
        || proposed_x >= (width - 1) as f32
        || proposed_y >= (height - 1) as f32
    {
        return None;
    }

    let px = proposed_x.round() as isize;
    let py = proposed_y.round() as isize;
    let mut best = None;
    let mut best_score = f32::NEG_INFINITY;

    for oy in -SNAP_RADIUS..=SNAP_RADIUS {
        for ox in -SNAP_RADIUS..=SNAP_RADIUS {
            let x = px + ox;
            let y = py + oy;
            if x <= 0 || y <= 0 || x >= width as isize - 1 || y >= height as isize - 1 {
                continue;
            }
            let xu = x as usize;
            let yu = y as usize;
            let e = edge[yu * width + xu];
            if e < MIN_TRACE_EDGE {
                continue;
            }
            let Some(direction) = best_direction(field, xu, yu) else {
                continue;
            };
            let (tx, ty) = align(direction, prev_hx, prev_hy);
            let turn = tx * prev_hx + ty * prev_hy;
            if turn < MIN_TURN_DOT {
                continue;
            }
            let dx = x as f32 - proposed_x;
            let dy = y as f32 - proposed_y;
            let distance = dx.hypot(dy);
            let score = e * (0.70 + 0.30 * direction_quality(direction))
                + 0.10 * turn
                - 0.035 * distance;
            if score > best_score {
                best_score = score;
                best = Some((x as f32 + 0.5, y as f32 + 0.5, tx, ty));
            }
        }
    }
    best
}

fn trace_half(
    seed: PathPoint,
    initial_hx: f32,
    initial_hy: f32,
    edge: &[f32],
    field: &OrientationField,
    width: usize,
    height: usize,
) -> Vec<PathPoint> {
    let mut points = vec![seed];
    let mut current = seed;
    let mut hx = initial_hx;
    let mut hy = initial_hy;
    let mut travelled = 0.0_f32;

    while travelled + STEP_PX <= MAX_HALF_LENGTH_PX {
        let proposed_x = current.x + hx * STEP_PX;
        let proposed_y = current.y + hy * STEP_PX;
        let Some((nx, ny, tx, ty)) = snap_to_edge(
            proposed_x,
            proposed_y,
            edge,
            field,
            width,
            height,
            hx,
            hy,
        ) else {
            break;
        };

        let dx = nx - current.x;
        let dy = ny - current.y;
        let step = dx.hypot(dy);
        if step < 0.55 || step > 4.5 {
            break;
        }

        // Avoid tight loops caused by snapping around a small high-contrast blob.
        if points
            .iter()
            .take(points.len().saturating_sub(4))
            .any(|p| (p.x - nx).hypot(p.y - ny) < 1.4)
        {
            break;
        }

        current = PathPoint { x: nx, y: ny };
        points.push(current);
        travelled += step;

        let Some((blended_x, blended_y)) = normalize(hx * 0.35 + tx * 0.65, hy * 0.35 + ty * 0.65)
        else {
            break;
        };
        hx = blended_x;
        hy = blended_y;
    }

    points
}

fn path_edge_support(
    points: &[PathPoint],
    edge: &[f32],
    width: usize,
    height: usize,
) -> Option<(f32, f32)> {
    if points.len() < 2 {
        return None;
    }

    let mut sum = 0.0_f32;
    let mut samples = 0_usize;
    let mut supported = 0_usize;

    for pair in points.windows(2) {
        let dx = pair[1].x - pair[0].x;
        let dy = pair[1].y - pair[0].y;
        let length = dx.hypot(dy);
        let steps = (length / 0.75).ceil().max(1.0) as usize;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = pair[0].x + dx * t;
            let y = pair[0].y + dy * t;
            if x < 0.0 || y < 0.0 || x >= width as f32 || y >= height as f32 {
                return None;
            }
            let xi = x.floor() as usize;
            let yi = y.floor() as usize;
            let value = edge[yi * width + xi];
            sum += value;
            samples += 1;
            if value >= MIN_TRACE_EDGE {
                supported += 1;
            }
        }
    }

    if samples == 0 {
        return None;
    }
    let mean = sum / samples as f32;
    let fraction = supported as f32 / samples as f32;
    (mean >= 0.12 && fraction >= 0.72).then_some((mean, fraction))
}

fn mark_covered(
    covered: &mut [bool],
    width: usize,
    height: usize,
    points: &[PathPoint],
) {
    for point in points {
        let cx = point.x.floor() as usize;
        let cy = point.y.floor() as usize;
        for y in cy.saturating_sub(COVER_RADIUS)..=(cy + COVER_RADIUS).min(height - 1) {
            for x in cx.saturating_sub(COVER_RADIUS)..=(cx + COVER_RADIUS).min(width - 1) {
                let dx = x.abs_diff(cx);
                let dy = y.abs_diff(cy);
                if dx * dx + dy * dy <= COVER_RADIUS * COVER_RADIUS {
                    covered[y * width + x] = true;
                }
            }
        }
    }
}

fn trace_path(
    seed: Seed,
    edge: &[f32],
    field: &OrientationField,
    width: usize,
    height: usize,
) -> Option<(PathStroke, f32)> {
    let direction = best_direction(field, seed.x, seed.y)?;
    let (tx, ty) = normalize(direction.tx, direction.ty)?;
    let seed_point = PathPoint {
        x: seed.x as f32 + 0.5,
        y: seed.y as f32 + 0.5,
    };

    let backward = trace_half(seed_point, -tx, -ty, edge, field, width, height);
    let forward = trace_half(seed_point, tx, ty, edge, field, width, height);

    let mut points = backward;
    points.reverse();
    points.extend(forward.into_iter().skip(1));
    if points.len() < 4 {
        return None;
    }

    let provisional = PathStroke {
        points,
        width: 1.0,
        opacity: 0.7,
        role: StrokeRole::Gesture,
    };
    let length = provisional.path_length_px();
    if !(MIN_PATH_LENGTH_PX..=MAX_PATH_LENGTH_PX).contains(&length) {
        return None;
    }

    let (mean_edge, _) = path_edge_support(&provisional.points, edge, width, height)?;
    let width_mark = (0.72 + 0.38 * mean_edge).clamp(0.72, 1.18);
    let opacity = (0.48 + 0.34 * mean_edge).clamp(0.50, 0.82);

    Some((
        PathStroke {
            width: width_mark,
            opacity,
            ..provisional
        },
        mean_edge,
    ))
}

fn max_paths_for_canvas(width: usize, height: usize) -> usize {
    ((width * height) / (72 * 72)).clamp(4, 56)
}

fn generate_paths(source: &RgbaImage) -> (Vec<PathStroke>, LongPathStats) {
    let (width_u32, height_u32) = source.dimensions();
    let width = width_u32 as usize;
    let height = height_u32 as usize;
    let darkness = darkness_map(source);
    let edge = edge_strength_map(&darkness, width, height);
    let field = OrientationField::new(&darkness, width, height);
    let seeds = collect_seeds(&edge, &field, width, height);
    let seed_count = seeds.len();

    let mut covered = vec![false; width * height];
    let mut accepted: Vec<(PathStroke, f32)> = Vec::new();
    let mut rejected_short = 0_usize;
    let mut rejected_support = 0_usize;
    let max_paths = max_paths_for_canvas(width, height);

    for seed in seeds {
        if accepted.len() >= max_paths {
            break;
        }
        if covered[seed.y * width + seed.x] {
            continue;
        }

        match trace_path(seed, &edge, &field, width, height) {
            Some((path, mean_edge)) => {
                mark_covered(&mut covered, width, height, &path.points);
                accepted.push((path, mean_edge));
            }
            None => {
                // Split diagnostics approximately: seeds on real edges that fail
                // normally do so because their trace is too short or corridor
                // support collapses. Exact rejection taxonomy is not a quality metric.
                if edge[seed.y * width + seed.x] >= MIN_SEED_EDGE {
                    rejected_short += 1;
                } else {
                    rejected_support += 1;
                }
            }
        }
    }

    let mut paths: Vec<PathStroke> = accepted.into_iter().map(|(path, _)| path).collect();
    paths.sort_by(|a, b| {
        let ay = a.points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let by = b.points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let ax = a.points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let bx = b.points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        ay.total_cmp(&by).then_with(|| ax.total_cmp(&bx))
    });

    let total_path_length_px: f64 = paths.iter().map(PathStroke::path_length_px).sum();
    let max_path_length_px = paths
        .iter()
        .map(PathStroke::path_length_px)
        .fold(0.0_f64, f64::max);
    let accepted_paths = paths.len();

    (
        paths,
        LongPathStats {
            seed_candidates: seed_count,
            accepted_paths,
            rejected_short,
            rejected_support,
            baseline_segment_count: 0,
            total_path_length_px,
            mean_path_length_px: if accepted_paths > 0 {
                total_path_length_px / accepted_paths as f64
            } else {
                0.0
            },
            max_path_length_px,
        },
    )
}

/// Generate P4-A.1 long structural paths.
///
/// When include_baseline is true, the exact frozen P2-B.1 straight-stroke
/// sketch is preserved and paths are added in the separate logical path layer.
/// When false, the exact same paths are returned with no straight segments so
/// the gesture layer can be inspected directly.
pub fn generate_long_structural_sketch_with_stats(
    source: &RgbaImage,
    options: &SketchOptions,
    include_baseline: bool,
) -> Result<(Sketch, LongPathStats), String> {
    // generate_sketch remains the authoritative option/dimension validation and
    // supplies the exact historical baseline when requested.
    let baseline = generate_sketch(source, options)?;
    let (paths, mut stats) = generate_paths(source);
    stats.baseline_segment_count = baseline.strokes.len();

    let sketch = Sketch {
        width: baseline.width,
        height: baseline.height,
        seed: baseline.seed,
        strokes: if include_baseline {
            baseline.strokes
        } else {
            Vec::new()
        },
        paths,
    };

    if sketch.logical_mark_count() > 500_000 {
        return Err("P4-A.1 logical mark limit exceeded".into());
    }
    Ok((sketch, stats))
}

pub fn generate_long_structural_sketch(
    source: &RgbaImage,
    options: &SketchOptions,
    include_baseline: bool,
) -> Result<Sketch, String> {
    generate_long_structural_sketch_with_stats(source, options, include_baseline)
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
    fn white_source_produces_no_long_paths() {
        let source = white(64, 64);
        let (sketch, stats) =
            generate_long_structural_sketch_with_stats(&source, &SketchOptions::default(), false)
                .unwrap();
        assert!(sketch.paths.is_empty());
        assert_eq!(stats.accepted_paths, 0);
    }

    #[test]
    fn long_vertical_step_yields_long_gesture_path() {
        let mut source = white(96, 96);
        for y in 0..96 {
            for x in 48..96 {
                source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let (sketch, stats) =
            generate_long_structural_sketch_with_stats(&source, &SketchOptions::default(), false)
                .unwrap();
        assert!(!sketch.paths.is_empty(), "step edge should trace at least one path");
        assert!(stats.max_path_length_px >= 40.0, "{stats:?}");
        assert!(sketch
            .paths
            .iter()
            .all(|path| path.path_length_px() >= MIN_PATH_LENGTH_PX));
        assert!(sketch.paths.iter().all(|path| path.role == StrokeRole::Gesture));
    }

    #[test]
    fn disk_boundary_produces_curved_long_path() {
        let mut source = white(96, 96);
        let cx = 48_i32;
        let cy = 48_i32;
        for y in 0..96_i32 {
            for x in 0..96_i32 {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= 26 * 26 {
                    source.put_pixel(x as u32, y as u32, Rgba([20, 20, 20, 255]));
                }
            }
        }
        let (sketch, _) =
            generate_long_structural_sketch_with_stats(&source, &SketchOptions::default(), false)
                .unwrap();
        let curved = sketch.paths.iter().any(|path| {
            let min_x = path.points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = path.points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
            let min_y = path.points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_y = path.points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
            max_x - min_x > 8.0 && max_y - min_y > 8.0 && path.path_length_px() >= 24.0
        });
        assert!(curved, "disk should yield at least one genuinely curved gesture");
    }

    #[test]
    fn overlay_preserves_exact_p2b1_segments_and_paths_only_matches_paths() {
        let mut source = white(96, 96);
        for y in 0..96 {
            for x in 48..96 {
                source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let options = SketchOptions::default();
        let baseline = generate_sketch(&source, &options).unwrap();
        let (overlay, _) =
            generate_long_structural_sketch_with_stats(&source, &options, true).unwrap();
        let (paths_only, _) =
            generate_long_structural_sketch_with_stats(&source, &options, false).unwrap();

        assert_eq!(overlay.strokes, baseline.strokes);
        assert_eq!(overlay.paths, paths_only.paths);
        assert!(paths_only.strokes.is_empty());
        assert!(!overlay.paths.is_empty());
    }

    #[test]
    fn long_path_generation_is_deterministic() {
        let mut source = white(96, 96);
        for y in 20..76 {
            for x in 20..76 {
                source.put_pixel(x, y, Rgba([25, 25, 25, 255]));
            }
        }
        let options = SketchOptions::default();
        let a = generate_long_structural_sketch(&source, &options, false).unwrap();
        let b = generate_long_structural_sketch(&source, &options, false).unwrap();
        assert_eq!(a, b);
    }
}
