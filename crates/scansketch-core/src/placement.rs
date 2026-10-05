//! Experimental P3-A.1: choose source-driven anchors before choosing stroke
//! direction. The frozen P2-B.1 renderer remains the baseline/default.
//!
//! Design goals:
//! - use P2-A only to establish a comparable tonal STROKE COUNT, never anchors;
//! - place primary anchors directly from source-dark cells;
//! - separate coarse structure, fine/form and ambiguous tonal candidate pools;
//! - preserve source-white gaps using the same conservative unsmoothed support;
//! - restore deterministic top-to-bottom commit order after score selection;
//! - append the unchanged P2-B.1 contour pass after the new tonal field.
//!
//! This is still a straight-segment, nonsemantic, CPU-first experiment. It is
//! not candidate optimization, erasure, a face detector or a perceptual model.

use image::RgbaImage;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use crate::analysis::darkness_map;
use crate::directional::{source_supported, Direction, OrientationField};
use crate::scanline::{generate_sketch, SketchOptions};
use crate::stroke::{Sketch, Stroke};

const CELL_W: usize = 5;
const CELL_H: usize = 3;
const COARSE_PERCENT: usize = 24;
const FINE_PERCENT: usize = 46;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ProposalClass {
    Coarse,
    Fine,
    Tonal,
    BaselineFallback,
}

#[derive(Clone, Debug)]
struct Candidate {
    stroke: Stroke,
    class: ProposalClass,
    score: f32,
    anchor_x: f32,
    anchor_y: f32,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct PlacementStats {
    pub target_tonal_strokes: usize,
    pub selected_coarse: usize,
    pub selected_fine: usize,
    pub selected_tonal: usize,
    pub baseline_fallback: usize,
    pub generated_candidate_count: usize,
}

fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn cell_rng(seed: u64, x: usize, y: usize, layer: u64) -> ChaCha8Rng {
    let cell = ((y as u64) << 32) ^ x as u64 ^ layer.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    ChaCha8Rng::seed_from_u64(mix64(seed ^ cell ^ 0xA13C_7E55_12F0_918D))
}

fn quality(direction: Direction) -> f32 {
    let strength = direction.energy / (direction.energy + 0.05);
    (direction.coherence * strength).clamp(0.0, 1.0)
}

fn midpoint(stroke: &Stroke) -> (f32, f32) {
    ((stroke.x0 + stroke.x1) * 0.5, (stroke.y0 + stroke.y1) * 0.5)
}

fn candidate_cmp(a: &Candidate, b: &Candidate) -> std::cmp::Ordering {
    b.score
        .total_cmp(&a.score)
        .then_with(|| a.anchor_y.total_cmp(&b.anchor_y))
        .then_with(|| a.anchor_x.total_cmp(&b.anchor_x))
        .then_with(|| a.class.cmp(&b.class))
}

fn commit_cmp(a: &Candidate, b: &Candidate) -> std::cmp::Ordering {
    a.anchor_y
        .total_cmp(&b.anchor_y)
        .then_with(|| a.anchor_x.total_cmp(&b.anchor_x))
        .then_with(|| a.class.cmp(&b.class))
        .then_with(|| a.stroke.y1.total_cmp(&b.stroke.y1))
        .then_with(|| a.stroke.x1.total_cmp(&b.stroke.x1))
}

/// Source-driven cell anchor. A weighted centroid identifies the local dark
/// mass, then we snap to an actual source-dark pixel near it so the midpoint
/// never begins inside a white gap between two neighboring dark structures.
fn cell_anchor(
    darkness: &[f32],
    width: usize,
    height: usize,
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    threshold: f32,
) -> Option<(f32, f32, f32, f32)> {
    let mut weight = 0.0_f32;
    let mut sx = 0.0_f32;
    let mut sy = 0.0_f32;
    let mut darkness_sum = 0.0_f32;
    let mut source_pixels = 0_usize;

    for y in top..bottom.min(height) {
        for x in left..right.min(width) {
            let d = darkness[y * width + x];
            if d <= threshold {
                continue;
            }
            let w = (d - threshold).max(0.001);
            weight += w;
            sx += (x as f32 + 0.5) * w;
            sy += (y as f32 + 0.5) * w;
            darkness_sum += d;
            source_pixels += 1;
        }
    }
    if source_pixels == 0 || weight <= 0.0 {
        return None;
    }
    let cx = sx / weight;
    let cy = sy / weight;
    let mut best = None;
    let mut best_score = f32::NEG_INFINITY;
    for y in top..bottom.min(height) {
        for x in left..right.min(width) {
            let d = darkness[y * width + x];
            if d <= threshold {
                continue;
            }
            let dx = (x as f32 + 0.5) - cx;
            let dy = (y as f32 + 0.5) - cy;
            let score = d - 0.035 * (dx * dx + dy * dy);
            if score > best_score {
                best_score = score;
                best = Some((x as f32 + 0.5, y as f32 + 0.5));
            }
        }
    }
    best.map(|(x, y)| {
        (
            x,
            y,
            darkness_sum / source_pixels as f32,
            source_pixels as f32 / ((right - left) * (bottom - top)) as f32,
        )
    })
}

fn stroke_from_anchor(
    anchor_x: f32,
    anchor_y: f32,
    average_darkness: f32,
    direction: Option<Direction>,
    class: ProposalClass,
    rng: &mut ChaCha8Rng,
    secondary: bool,
) -> Stroke {
    let (mut tx, mut ty) = match direction {
        Some(d) => (d.tx, d.ty),
        None => {
            // Ambiguous zones get a hand-like low-angle prior rather than
            // perfectly aligned scan rows. It remains deterministic/seeded.
            let angle = rng.gen_range(-0.14_f32..0.14_f32);
            (angle.cos(), angle.sin())
        }
    };
    if tx < 0.0 {
        tx = -tx;
        ty = -ty;
    }

    let base_length = match class {
        ProposalClass::Coarse => 5.7 + 3.8 * average_darkness,
        ProposalClass::Fine => 3.7 + 3.0 * average_darkness,
        ProposalClass::Tonal | ProposalClass::BaselineFallback => 3.1 + 3.6 * average_darkness,
    };
    let jitter = match class {
        ProposalClass::Coarse => rng.gen_range(-0.7_f32..1.5_f32),
        ProposalClass::Fine => rng.gen_range(-0.7_f32..1.0_f32),
        _ => rng.gen_range(-0.9_f32..1.2_f32),
    };
    let mut length = (base_length + jitter).clamp(2.0, 10.8);
    if secondary {
        length *= rng.gen_range(0.72_f32..0.88_f32);
    }

    let width_base = (0.34 + 0.46 * average_darkness).clamp(0.36, 0.80);
    let width = width_base * rng.gen_range(0.84_f32..1.0_f32);
    let mut opacity =
        ((0.14 + 0.72 * average_darkness) * rng.gen_range(0.82_f32..1.0_f32))
            .clamp(0.0, 1.0);
    if secondary {
        opacity *= 0.84;
    }

    let half = length * 0.5;
    Stroke {
        x0: anchor_x - tx * half,
        y0: anchor_y - ty * half,
        x1: anchor_x + tx * half,
        y1: anchor_y + ty * half,
        width,
        opacity,
    }
}

fn push_candidate(
    pools: &mut [Vec<Candidate>; 3],
    darkness: &[f32],
    field: &OrientationField,
    width: usize,
    height: usize,
    options: &SketchOptions,
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    secondary: bool,
) {
    let threshold = options.white_threshold.max(0.10);
    let Some((base_x, base_y, average_darkness, occupancy)) =
        cell_anchor(darkness, width, height, left, top, right, bottom, threshold)
    else {
        return;
    };

    let mut rng = cell_rng(
        options.seed,
        left / CELL_W,
        top / CELL_H,
        if secondary { 1 } else { 0 },
    );

    // Position jitter is the key difference from scan-band anchors, but it is
    // small enough that the source footprint check remains authoritative.
    let mut anchor_x = base_x + rng.gen_range(-0.55_f32..0.55_f32);
    let mut anchor_y = base_y + rng.gen_range(-0.45_f32..0.45_f32);

    let px = base_x.floor().max(0.0) as usize;
    let py = base_y.floor().max(0.0) as usize;
    let (fine_here, coarse_here) = field.components(px, py);
    let coarse = if quality(coarse_here) >= 0.23 && coarse_here.coherence >= 0.62 {
        Some(coarse_here)
    } else {
        field.best_near(px, py, 9, true, 0.004, 0.62)
    };
    let fine = if quality(fine_here) >= 0.20 && fine_here.coherence >= 0.56 {
        Some(fine_here)
    } else {
        field.best_near(px, py, 4, false, 0.008, 0.56)
    };

    let (class, direction, directional_quality) = match (coarse, fine) {
        (Some(c), Some(f)) if quality(c) + 0.045 >= quality(f) => {
            (ProposalClass::Coarse, Some(c), quality(c))
        }
        (_, Some(f)) => (ProposalClass::Fine, Some(f), quality(f)),
        (Some(c), None) => (ProposalClass::Coarse, Some(c), quality(c)),
        (None, None) => (ProposalClass::Tonal, None, 0.0),
    };

    // Dark secondary layers are offset across, not along, the primary flow.
    if secondary {
        if let Some(d) = direction {
            let offset = rng.gen_range(0.65_f32..1.20_f32)
                * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
            anchor_x += -d.ty * offset;
            anchor_y += d.tx * offset;
        } else {
            anchor_y += rng.gen_range(0.60_f32..1.05_f32);
        }
    }

    let stroke = stroke_from_anchor(
        anchor_x,
        anchor_y,
        average_darkness,
        direction,
        if secondary && class == ProposalClass::Coarse {
            ProposalClass::Fine
        } else {
            class
        },
        &mut rng,
        secondary,
    );
    if !source_supported(darkness, width, height, &stroke, threshold) {
        return;
    }
    let effective_class = if secondary && class == ProposalClass::Coarse {
        ProposalClass::Fine
    } else {
        class
    };
    let score = average_darkness
        * (0.52 + directional_quality)
        * (0.72 + 0.28 * occupancy)
        * if secondary { 0.86 } else { 1.0 };
    let pool = match effective_class {
        ProposalClass::Coarse => 0,
        ProposalClass::Fine => 1,
        ProposalClass::Tonal | ProposalClass::BaselineFallback => 2,
    };
    pools[pool].push(Candidate {
        stroke,
        class: effective_class,
        score,
        anchor_x,
        anchor_y,
    });
}

fn take_best(pool: &mut Vec<Candidate>, count: usize, out: &mut Vec<Candidate>) {
    pool.sort_by(candidate_cmp);
    let take = count.min(pool.len());
    out.extend(pool.drain(..take));
}

fn build_placement_tone(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<(Vec<Stroke>, PlacementStats), String> {
    let mut baseline_options = options.clone();
    baseline_options.enable_contours = false;
    let baseline = generate_sketch(source, &baseline_options)?;
    let target = baseline.strokes.len();
    if target == 0 {
        return Ok((Vec::new(), PlacementStats::default()));
    }

    let (width, height) = source.dimensions();
    let (w, h) = (width as usize, height as usize);
    let darkness = darkness_map(source);
    let field = OrientationField::new(&darkness, w, h);
    let mut pools: [Vec<Candidate>; 3] = [Vec::new(), Vec::new(), Vec::new()];

    for top in (0..h).step_by(CELL_H) {
        let bottom = (top + CELL_H).min(h);
        for left in (0..w).step_by(CELL_W) {
            let right = (left + CELL_W).min(w);
            push_candidate(
                &mut pools, &darkness, &field, w, h, options,
                left, top, right, bottom, false,
            );
            // A second independently offset proposal replaces P2-A's rigid
            // second scanline only in genuinely dark cells.
            let mut dark_sum = 0.0_f32;
            let mut count = 0_usize;
            for y in top..bottom {
                for x in left..right {
                    let d = darkness[y * w + x];
                    if d > options.white_threshold {
                        dark_sum += d;
                        count += 1;
                    }
                }
            }
            if count > 0 && dark_sum / count as f32 >= 0.70 {
                push_candidate(
                    &mut pools, &darkness, &field, w, h, options,
                    left, top, right, bottom, true,
                );
            }
        }
    }

    let generated_candidate_count = pools.iter().map(Vec::len).sum();
    let coarse_target = target * COARSE_PERCENT / 100;
    let fine_target = target * FINE_PERCENT / 100;
    let tonal_target = target.saturating_sub(coarse_target + fine_target);

    let mut selected = Vec::with_capacity(target);
    take_best(&mut pools[0], coarse_target, &mut selected);
    take_best(&mut pools[1], fine_target, &mut selected);
    take_best(&mut pools[2], tonal_target, &mut selected);

    // Quotas are allocation guards, not reasons to waste a valid budget.
    // Fill unused slots from the strongest remaining source-driven proposals.
    let mut remainder: Vec<Candidate> = pools.into_iter().flatten().collect();
    remainder.sort_by(candidate_cmp);
    let needed = target.saturating_sub(selected.len());
    selected.extend(remainder.into_iter().take(needed));

    let source_driven = selected.len();
    if selected.len() < target {
        // Rare/sparse-input safety net. It guarantees comparable stroke count
        // without changing the frozen baseline branch. Keep these exact old
        // marks last-resort only; statistics make fallback visible to tests.
        for stroke in baseline.strokes.iter().take(target - selected.len()) {
            let (anchor_x, anchor_y) = midpoint(stroke);
            selected.push(Candidate {
                stroke: stroke.clone(),
                class: ProposalClass::BaselineFallback,
                score: 0.0,
                anchor_x,
                anchor_y,
            });
        }
    }
    if selected.len() != target {
        return Err("P3-A.1 could not satisfy the comparison tonal stroke budget".into());
    }

    selected.sort_by(commit_cmp);
    let mut stats = PlacementStats {
        target_tonal_strokes: target,
        generated_candidate_count,
        baseline_fallback: target - source_driven,
        ..PlacementStats::default()
    };
    for candidate in &selected {
        match candidate.class {
            ProposalClass::Coarse => stats.selected_coarse += 1,
            ProposalClass::Fine => stats.selected_fine += 1,
            ProposalClass::Tonal => stats.selected_tonal += 1,
            ProposalClass::BaselineFallback => {}
        }
    }
    Ok((selected.into_iter().map(|candidate| candidate.stroke).collect(), stats))
}

pub fn generate_placement_sketch_with_stats(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<(Sketch, PlacementStats), String> {
    let (width, height) = source.dimensions();
    // Reuse baseline validation before any new work. This deliberately does
    // not reuse baseline anchors.
    let mut validation = options.clone();
    validation.enable_contours = false;
    let _ = generate_sketch(source, &validation)?;

    let darkness = darkness_map(source);
    let (mut strokes, stats) = build_placement_tone(source, options)?;
    if options.enable_contours {
        crate::contour::append_contours(&darkness, width, height, options, &mut strokes)?;
    }
    Ok((Sketch { width, height, seed: options.seed, strokes }, stats))
}

pub fn generate_placement_sketch(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<Sketch, String> {
    generate_placement_sketch_with_stats(source, options).map(|(sketch, _)| sketch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{audit_white_pixels, render_sketch, AuditRoi};
    use image::Rgba;

    fn white(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
    }

    fn vertical_step() -> RgbaImage {
        let mut source = white(64, 64);
        for y in 0..64 {
            for x in 24..64 {
                source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        source
    }

    #[test]
    fn blank_paper_stays_blank() {
        let source = white(64, 64);
        let (sketch, stats) =
            generate_placement_sketch_with_stats(&source, &SketchOptions::default()).unwrap();
        assert!(sketch.strokes.is_empty());
        assert_eq!(stats.target_tonal_strokes, 0);
        assert_eq!(stats.baseline_fallback, 0);
    }

    #[test]
    fn step_uses_same_total_budget_as_p2b1_but_new_tonal_geometry() {
        let source = vertical_step();
        let opts = SketchOptions::default();
        let baseline = generate_sketch(&source, &opts).unwrap();
        let mut tonal_opts = opts.clone();
        tonal_opts.enable_contours = false;
        let baseline_tone = generate_sketch(&source, &tonal_opts).unwrap();
        let (placed, stats) = generate_placement_sketch_with_stats(&source, &opts).unwrap();
        assert_eq!(placed.strokes.len(), baseline.strokes.len());
        assert_eq!(stats.target_tonal_strokes, baseline_tone.strokes.len());
        assert!(stats.selected_coarse + stats.selected_fine > 0);
        assert!(stats.baseline_fallback < stats.target_tonal_strokes / 5);
        let changed = baseline_tone.strokes.iter()
            .zip(placed.strokes.iter().take(baseline_tone.strokes.len()))
            .filter(|(a,b)| *a != *b)
            .count();
        assert!(changed > baseline_tone.strokes.len() / 2);
    }

    #[test]
    fn placement_is_deterministic() {
        let source = vertical_step();
        let opts = SketchOptions::default();
        let a = generate_placement_sketch_with_stats(&source, &opts).unwrap();
        let b = generate_placement_sketch_with_stats(&source, &opts).unwrap();
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.target_tonal_strokes, b.1.target_tonal_strokes);
        assert_eq!(a.1.selected_coarse, b.1.selected_coarse);
        assert_eq!(a.1.selected_fine, b.1.selected_fine);
        assert_eq!(a.1.selected_tonal, b.1.selected_tonal);
        assert_eq!(a.1.baseline_fallback, b.1.baseline_fallback);
    }

    #[test]
    fn protected_channel_remains_clear() {
        let mut source = white(64, 64);
        for y in 8..56 {
            for x in 8..56 {
                if !(29..35).contains(&x) {
                    source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                }
            }
        }
        let sketch = generate_placement_sketch(&source, &SketchOptions::default()).unwrap();
        let render = render_sketch(&sketch).unwrap();
        let mut rgba = RgbaImage::new(64, 64);
        for (pixel, bytes) in rgba.pixels_mut().zip(render.data().chunks_exact(4)) {
            *pixel = Rgba([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        let audit = audit_white_pixels(&source, &rgba, Some(AuditRoi {
            x: 29, y: 8, width: 6, height: 48
        })).unwrap();
        assert_eq!(audit.roi_source_white_pixels, Some(288));
        assert_eq!(audit.roi_affected_white_pixels, Some(0));
    }

    #[test]
    fn nonhorizontal_structure_is_generated_directly() {
        let source = vertical_step();
        let mut opts = SketchOptions::default();
        opts.enable_contours = false;
        let sketch = generate_placement_sketch(&source, &opts).unwrap();
        let nonhorizontal = sketch.strokes.iter().filter(|stroke| {
            (stroke.y1 - stroke.y0).abs() > (stroke.x1 - stroke.x0).abs() * 0.7
        }).count();
        assert!(nonhorizontal > sketch.strokes.len() / 10);
    }

    #[test]
    fn overflow_rules_still_apply_when_contours_are_enabled() {
        let source = vertical_step();
        let mut tonal = SketchOptions::default();
        tonal.enable_contours = false;
        let baseline_tone = generate_sketch(&source, &tonal).unwrap();
        let opts = SketchOptions {
            max_strokes: baseline_tone.strokes.len(),
            ..SketchOptions::default()
        };
        assert!(generate_placement_sketch(&source, &opts).is_err());
    }
}
