//! P3-A.2: hybrid structural reinforcement.
//!
//! Preserve the exact P2 tonal prefix. Spend only the structural budget that
//! frozen P2-B.1 already spends. Prefer residual-aware multiscale source
//! tangents; fill any shortfall with the original P2-B.1 contour records.
//! This keeps total stroke count exactly equal to P2-B.1 while testing whether
//! smarter structure can improve a stable tonal base.

use image::{Rgba, RgbaImage};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use crate::analysis::darkness_map;
use crate::directional::{source_supported, Direction, OrientationField};
use crate::metrics::edge_strength_map;
use crate::render::render_sketch;
use crate::scanline::{generate_sketch, SketchOptions};
use crate::stroke::{Sketch, Stroke};

const GRID: usize = 4;
const TILE: usize = 32;
const PER_TILE: u8 = 3;
const MIN_RESIDUAL: f32 = 0.025;

#[derive(Clone, Debug, Default, Serialize)]
pub struct HybridStats {
    pub tone_count: usize,
    pub structural_budget: usize,
    pub generated_hybrid_candidates: usize,
    pub hybrid_selected: usize,
    pub baseline_contour_fallback: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SelectiveHybridStats {
    pub tone_count: usize,
    pub structural_budget: usize,
    pub generated_hybrid_candidates: usize,
    pub max_replacements: usize,
    pub replacements_made: usize,
    pub baseline_contours_retained: usize,
    pub weakest_baseline_utility: f32,
    pub strongest_hybrid_utility: f32,
    pub last_replaced_baseline_utility: f32,
    pub last_replacement_hybrid_utility: f32,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct MissingStructureStats {
    pub tone_count: usize,
    pub structural_budget: usize,
    pub generated_hybrid_candidates: usize,
    pub max_replacements: usize,
    pub replacements_made: usize,
    pub baseline_contours_retained: usize,
    pub weakest_baseline_utility: f32,
    pub strongest_hybrid_utility: f32,
    pub mean_missing_edge_at_replacements: f32,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct DeficitProposalStats {
    pub tone_count: usize,
    pub structural_budget: usize,
    pub generated_deficit_candidates: usize,
    pub max_replacements: usize,
    pub replacements_made: usize,
    pub baseline_contours_retained: usize,
    pub weakest_baseline_utility: f32,
    pub strongest_candidate_utility: f32,
    pub mean_missing_edge_at_replacements: f32,
}

#[derive(Clone, Debug)]
struct Candidate {
    stroke: Stroke,
    score: f32,
    x: usize,
    y: usize,
}

fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn cell_rng(seed: u64, x: usize, y: usize) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(mix64(
        seed ^ ((y as u64) << 32) ^ x as u64 ^ 0xA2B4_9F31_0D7C_5E61,
    ))
}

fn quality(direction: Direction) -> f32 {
    let strength = direction.energy / (direction.energy + 0.05);
    (direction.coherence * strength).clamp(0.0, 1.0)
}

fn pixmap_rgba(sketch: &Sketch) -> Result<RgbaImage, String> {
    let pixmap = render_sketch(sketch)?;
    let mut image = RgbaImage::new(sketch.width, sketch.height);
    for (pixel, bytes) in image.pixels_mut().zip(pixmap.data().chunks_exact(4)) {
        *pixel = Rgba([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
    Ok(image)
}

fn best_anchor(
    darkness: &[f32],
    residual: &[f32],
    width: usize,
    height: usize,
    center_x: usize,
    center_y: usize,
    threshold: f32,
) -> Option<(usize, usize, f32)> {
    let mut best = None;
    let mut best_score = MIN_RESIDUAL;
    for y in center_y.saturating_sub(2)..=(center_y + 2).min(height - 1) {
        for x in center_x.saturating_sub(2)..=(center_x + 2).min(width - 1) {
            let index = y * width + x;
            let d = darkness[index];
            let r = residual[index];
            if d <= threshold || r <= MIN_RESIDUAL {
                continue;
            }
            let distance =
                (x.abs_diff(center_x) + y.abs_diff(center_y)) as f32;
            let score = r * (0.65 + 0.35 * d) / (1.0 + 0.08 * distance);
            if score > best_score {
                best_score = score;
                best = Some((x, y, r));
            }
        }
    }
    best
}

fn choose_direction(
    field: &OrientationField,
    x: usize,
    y: usize,
) -> Option<Direction> {
    let (fine, coarse) = field.components(x, y);
    let fq = quality(fine);
    let cq = quality(coarse);

    let direct = if cq >= 0.22 && coarse.coherence >= 0.60 && cq + 0.035 >= fq {
        Some(coarse)
    } else if fq >= 0.19 && fine.coherence >= 0.54 {
        Some(fine)
    } else {
        None
    };
    if direct.is_some() {
        return direct;
    }

    let coarse_near = field.best_near(x, y, 7, true, 0.004, 0.60);
    let fine_near = field.best_near(x, y, 4, false, 0.008, 0.54);
    match (coarse_near, fine_near) {
        (Some(c), Some(f)) if quality(c) + 0.035 >= quality(f) => Some(c),
        (_, Some(f)) => Some(f),
        (Some(c), None) => Some(c),
        _ => None,
    }
}

fn candidate_stroke(
    darkness: &[f32],
    residual: &[f32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    residual_here: f32,
    direction: Direction,
    options: &SketchOptions,
) -> Option<Candidate> {
    let mut rng = cell_rng(options.seed, x / GRID, y / GRID);
    let mut tx = direction.tx;
    let mut ty = direction.ty;
    if tx < 0.0 {
        tx = -tx;
        ty = -ty;
    }

    let d = darkness[y * width + x];
    let structural_quality = quality(direction);
    let cx = x as f32 + 0.5 + rng.gen_range(-0.22_f32..0.22_f32);
    let cy = y as f32 + 0.5 + rng.gen_range(-0.22_f32..0.22_f32);
    let desired_length = (3.0
        + 2.6 * structural_quality
        + 1.1 * residual_here
        + rng.gen_range(-0.28_f32..0.35_f32))
        .clamp(2.6, 7.2);
    let width_mark =
        (0.38 + 0.16 * d + 0.06 * structural_quality)
            * rng.gen_range(0.92_f32..1.0_f32);
    let opacity =
        ((0.30 + 0.30 * d + 0.16 * residual_here + 0.08 * structural_quality)
            * rng.gen_range(0.93_f32..1.0_f32))
            .clamp(0.28, 0.74);

    let support_threshold = options.white_threshold.max(0.12);
    let mut chosen = None;
    for scale in [1.0_f32, 0.78, 0.58] {
        let half = desired_length * scale * 0.5;
        let stroke = Stroke {
            x0: cx - tx * half,
            y0: cy - ty * half,
            x1: cx + tx * half,
            y1: cy + ty * half,
            width: width_mark,
            opacity,
        };
        if source_supported(
            darkness,
            width,
            height,
            &stroke,
            support_threshold,
        ) {
            chosen = Some(stroke);
            break;
        }
    }
    let stroke = chosen?;

    // Require residual evidence along the actual candidate body, not only
    // at its anchor. This keeps structural marks out of already-satisfied tone.
    let dx = stroke.x1 - stroke.x0;
    let dy = stroke.y1 - stroke.y0;
    let length = dx.hypot(dy).max(0.001);
    let samples = (length / 0.75).ceil().max(2.0) as usize;
    let mut residual_sum = 0.0_f32;
    let mut target_sum = 0.0_f32;
    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        let sx = stroke.x0 + dx * t;
        let sy = stroke.y0 + dy * t;
        if sx < 0.0 || sy < 0.0 || sx >= width as f32 || sy >= height as f32 {
            return None;
        }
        let index = sy.floor() as usize * width + sx.floor() as usize;
        residual_sum += residual[index];
        target_sum += darkness[index];
    }
    let count = (samples + 1) as f32;
    let mean_residual = residual_sum / count;
    let mean_target = target_sum / count;
    if mean_residual <= MIN_RESIDUAL {
        return None;
    }

    let score = mean_residual
        * (0.58 + 0.92 * structural_quality)
        * (0.70 + 0.30 * mean_target);

    Some(Candidate { stroke, score, x, y })
}

fn hybrid_candidates(
    source_darkness: &[f32],
    residual: &[f32],
    width: usize,
    height: usize,
    field: &OrientationField,
    options: &SketchOptions,
) -> Vec<Candidate> {
    let threshold = options.white_threshold.max(0.10);
    let mut out = Vec::new();
    if width < 5 || height < 5 {
        return out;
    }
    for cy in (2..height - 2).step_by(GRID) {
        for cx in (2..width - 2).step_by(GRID) {
            let Some((x, y, residual_here)) = best_anchor(
                source_darkness, residual, width, height, cx, cy, threshold
            ) else {
                continue;
            };
            let Some(direction) = choose_direction(field, x, y) else {
                continue;
            };
            if let Some(candidate) = candidate_stroke(
                source_darkness,
                residual,
                width,
                height,
                x,
                y,
                residual_here,
                direction,
                options,
            ) {
                out.push(candidate);
            }
        }
    }
    out.sort_by(|a, b| {
        b.score.total_cmp(&a.score)
            .then_with(|| a.y.cmp(&b.y))
            .then_with(|| a.x.cmp(&b.x))
    });
    out
}

fn select_spaced(
    candidates: Vec<Candidate>,
    budget: usize,
    width: usize,
    height: usize,
) -> Vec<Candidate> {
    if budget == 0 {
        return Vec::new();
    }
    let tile_columns = width.div_ceil(TILE);
    let tile_rows = height.div_ceil(TILE);
    let mut tile_usage = vec![0_u8; tile_columns * tile_rows];
    let mut occupied = vec![false; width * height];
    let mut selected = Vec::with_capacity(budget);

    for candidate in candidates {
        let tile = (candidate.y / TILE) * tile_columns + candidate.x / TILE;
        if tile_usage[tile] >= PER_TILE {
            continue;
        }
        let nearby = (candidate.y.saturating_sub(3)
            ..=(candidate.y + 3).min(height - 1))
            .any(|yy| {
                (candidate.x.saturating_sub(3)
                    ..=(candidate.x + 3).min(width - 1))
                    .any(|xx| occupied[yy * width + xx])
            });
        if nearby {
            continue;
        }
        occupied[candidate.y * width + candidate.x] = true;
        tile_usage[tile] += 1;
        selected.push(candidate);
        if selected.len() >= budget {
            break;
        }
    }
    selected.sort_by(|a, b| {
        a.y.cmp(&b.y)
            .then_with(|| a.x.cmp(&b.x))
            .then_with(|| b.score.total_cmp(&a.score))
    });
    selected
}

pub fn generate_hybrid_sketch_with_stats(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<(Sketch, HybridStats), String> {
    let (width, height) = source.dimensions();
    let mut tonal_options = options.clone();
    tonal_options.enable_contours = false;

    let tone = generate_sketch(source, &tonal_options)?;
    if !options.enable_contours {
        return Ok((
            tone.clone(),
            HybridStats {
                tone_count: tone.strokes.len(),
                ..HybridStats::default()
            },
        ));
    }

    // Frozen P2-B.1 supplies the exact structural comparison budget and
    // original contour records used as deterministic fallback.
    let baseline = generate_sketch(source, options)?;
    if baseline.strokes.len() < tone.strokes.len()
        || baseline.strokes[..tone.strokes.len()] != tone.strokes[..]
    {
        return Err("P3-A.2 invariant failed: P2-B.1 tonal prefix changed".into());
    }
    let budget = baseline.strokes.len() - tone.strokes.len();
    if budget == 0 {
        return Ok((
            tone.clone(),
            HybridStats {
                tone_count: tone.strokes.len(),
                ..HybridStats::default()
            },
        ));
    }

    let source_darkness = darkness_map(source);
    let tone_preview = pixmap_rgba(&tone)?;
    let tone_darkness = darkness_map(&tone_preview);
    let residual: Vec<f32> = source_darkness.iter()
        .zip(&tone_darkness)
        .map(|(&target, &preview)| (target - preview).max(0.0))
        .collect();

    let field = OrientationField::new(
        &source_darkness,
        width as usize,
        height as usize,
    );
    let candidates = hybrid_candidates(
        &source_darkness,
        &residual,
        width as usize,
        height as usize,
        &field,
        options,
    );
    let generated = candidates.len();
    let selected = select_spaced(
        candidates,
        budget,
        width as usize,
        height as usize,
    );

    let hybrid_count = selected.len();
    let fallback_needed = budget - hybrid_count;
    let baseline_contours = &baseline.strokes[tone.strokes.len()..];

    let mut strokes = tone.strokes.clone();
    strokes.extend(selected.into_iter().map(|candidate| candidate.stroke));
    strokes.extend(
        baseline_contours
            .iter()
            .take(fallback_needed)
            .cloned(),
    );

    if strokes.len() != baseline.strokes.len() {
        return Err("P3-A.2 failed to preserve the frozen P2-B.1 total stroke count".into());
    }
    if strokes.len() > options.max_strokes {
        return Err("stroke budget exceeded during P3-A.2 hybrid structural pass".into());
    }

    Ok((
        Sketch { width, height, seed: options.seed, strokes },
        HybridStats {
            tone_count: tone.strokes.len(),
            structural_budget: budget,
            generated_hybrid_candidates: generated,
            hybrid_selected: hybrid_count,
            baseline_contour_fallback: fallback_needed,
        },
    ))
}

pub fn generate_hybrid_sketch(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<Sketch, String> {
    generate_hybrid_sketch_with_stats(source, options).map(|(sketch, _)| sketch)
}

const SELECTIVE_MAX_PERCENT: usize = 40;
const SELECTIVE_RELATIVE_MARGIN: f32 = 1.15;
const SELECTIVE_ABSOLUTE_MARGIN: f32 = 0.001;

fn stroke_utility(
    stroke: &Stroke,
    darkness: &[f32],
    residual: &[f32],
    width: usize,
    height: usize,
    field: &OrientationField,
) -> f32 {
    let dx = stroke.x1 - stroke.x0;
    let dy = stroke.y1 - stroke.y0;
    let length = dx.hypot(dy);
    if length < 0.1 {
        return 0.0;
    }
    let samples = (length / 0.75).ceil().max(2.0) as usize;
    let mut residual_sum = 0.0_f32;
    let mut target_sum = 0.0_f32;
    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        let sx = stroke.x0 + dx * t;
        let sy = stroke.y0 + dy * t;
        if sx < 0.0 || sy < 0.0 || sx >= width as f32 || sy >= height as f32 {
            return 0.0;
        }
        let index = sy.floor() as usize * width + sx.floor() as usize;
        residual_sum += residual[index];
        target_sum += darkness[index];
    }
    let count = (samples + 1) as f32;
    let mean_residual = residual_sum / count;
    if mean_residual <= 0.0 {
        return 0.0;
    }
    let mean_target = target_sum / count;

    let mx = ((stroke.x0 + stroke.x1) * 0.5)
        .floor()
        .clamp(0.0, width.saturating_sub(1) as f32) as usize;
    let my = ((stroke.y0 + stroke.y1) * 0.5)
        .floor()
        .clamp(0.0, height.saturating_sub(1) as f32) as usize;
    let tx = dx / length;
    let ty = dy / length;
    let structural_alignment = choose_direction(field, mx, my)
        .map(|direction| {
            let alignment = (tx * direction.tx + ty * direction.ty).abs();
            quality(direction) * alignment
        })
        .unwrap_or(0.0);

    mean_residual
        * (0.58 + 0.92 * structural_alignment)
        * (0.70 + 0.30 * mean_target)
}

/// P3-A.2.1: keep the exact P2-B.1 structural tail by default and substitute
/// only baseline contour slots that are clearly weaker than a source-supported
/// hybrid candidate under the same residual/structure utility.
///
/// This is intentionally NOT global rendered optimization. The 15% + 0.001
/// margin and 40% hard cap are fixed experiment controls recorded in docs.
pub fn generate_selective_hybrid_sketch_with_stats(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<(Sketch, SelectiveHybridStats), String> {
    let (width, height) = source.dimensions();
    let mut tonal_options = options.clone();
    tonal_options.enable_contours = false;
    let tone = generate_sketch(source, &tonal_options)?;

    if !options.enable_contours {
        return Ok((
            tone.clone(),
            SelectiveHybridStats {
                tone_count: tone.strokes.len(),
                ..SelectiveHybridStats::default()
            },
        ));
    }

    let baseline = generate_sketch(source, options)?;
    if baseline.strokes.len() < tone.strokes.len()
        || baseline.strokes[..tone.strokes.len()] != tone.strokes[..]
    {
        return Err("P3-A.2.1 invariant failed: P2-B.1 tonal prefix changed".into());
    }

    let budget = baseline.strokes.len() - tone.strokes.len();
    if budget == 0 {
        return Ok((
            tone.clone(),
            SelectiveHybridStats {
                tone_count: tone.strokes.len(),
                ..SelectiveHybridStats::default()
            },
        ));
    }

    let source_darkness = darkness_map(source);
    let tone_preview = pixmap_rgba(&tone)?;
    let tone_darkness = darkness_map(&tone_preview);
    let residual: Vec<f32> = source_darkness
        .iter()
        .zip(&tone_darkness)
        .map(|(&target, &preview)| (target - preview).max(0.0))
        .collect();
    let field = OrientationField::new(
        &source_darkness,
        width as usize,
        height as usize,
    );

    let candidates = hybrid_candidates(
        &source_darkness,
        &residual,
        width as usize,
        height as usize,
        &field,
        options,
    );
    let generated = candidates.len();

    let max_replacements = ((budget * SELECTIVE_MAX_PERCENT) / 100)
        .max(1)
        .min(budget);
    let preselection_limit = (max_replacements * 3).min(budget);
    let spaced = select_spaced(
        candidates,
        preselection_limit,
        width as usize,
        height as usize,
    );

    let mut scored_hybrids: Vec<(Candidate, f32)> = spaced
        .into_iter()
        .map(|candidate| {
            let utility = stroke_utility(
                &candidate.stroke,
                &source_darkness,
                &residual,
                width as usize,
                height as usize,
                &field,
            );
            (candidate, utility)
        })
        .filter(|(_, utility)| *utility > 0.0)
        .collect();
    scored_hybrids.sort_by(|(a, au), (b, bu)| {
        bu.total_cmp(au)
            .then_with(|| a.y.cmp(&b.y))
            .then_with(|| a.x.cmp(&b.x))
    });

    let baseline_contours = &baseline.strokes[tone.strokes.len()..];
    let mut baseline_scores: Vec<(usize, f32)> = baseline_contours
        .iter()
        .enumerate()
        .map(|(index, stroke)| {
            (
                index,
                stroke_utility(
                    stroke,
                    &source_darkness,
                    &residual,
                    width as usize,
                    height as usize,
                    &field,
                ),
            )
        })
        .collect();
    baseline_scores.sort_by(|(ai, au), (bi, bu)| {
        au.total_cmp(bu).then_with(|| ai.cmp(bi))
    });

    let weakest_baseline_utility =
        baseline_scores.first().map(|(_, score)| *score).unwrap_or(0.0);
    let strongest_hybrid_utility =
        scored_hybrids.first().map(|(_, score)| *score).unwrap_or(0.0);

    let mut structural = baseline_contours.to_vec();
    let mut replacements = 0_usize;
    let mut last_baseline = 0.0_f32;
    let mut last_hybrid = 0.0_f32;

    for (candidate, hybrid_utility) in scored_hybrids {
        if replacements >= max_replacements {
            break;
        }
        let Some(&(baseline_index, baseline_utility)) =
            baseline_scores.get(replacements)
        else {
            break;
        };

        let required = baseline_utility * SELECTIVE_RELATIVE_MARGIN
            + SELECTIVE_ABSOLUTE_MARGIN;
        if hybrid_utility < required {
            // Hybrids are descending and baseline candidates are ascending.
            // If the strongest remaining hybrid cannot beat the weakest
            // remaining contour, no later pair can qualify.
            break;
        }

        structural[baseline_index] = candidate.stroke;
        replacements += 1;
        last_baseline = baseline_utility;
        last_hybrid = hybrid_utility;
    }

    let mut strokes = tone.strokes.clone();
    strokes.extend(structural);
    if strokes.len() != baseline.strokes.len() {
        return Err(
            "P3-A.2.1 failed to preserve the frozen P2-B.1 total stroke count"
                .into(),
        );
    }
    if strokes.len() > options.max_strokes {
        return Err(
            "stroke budget exceeded during P3-A.2.1 selective hybrid pass"
                .into(),
        );
    }

    Ok((
        Sketch {
            width,
            height,
            seed: options.seed,
            strokes,
        },
        SelectiveHybridStats {
            tone_count: tone.strokes.len(),
            structural_budget: budget,
            generated_hybrid_candidates: generated,
            max_replacements,
            replacements_made: replacements,
            baseline_contours_retained: budget - replacements,
            weakest_baseline_utility,
            strongest_hybrid_utility,
            last_replaced_baseline_utility: last_baseline,
            last_replacement_hybrid_utility: last_hybrid,
        },
    ))
}

pub fn generate_selective_hybrid_sketch(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<Sketch, String> {
    generate_selective_hybrid_sketch_with_stats(source, options)
        .map(|(sketch, _)| sketch)
}

fn missing_edge_residual(
    source_darkness: &[f32],
    baseline_darkness: &[f32],
    width: usize,
    height: usize,
) -> Vec<f32> {
    let source_edge = edge_strength_map(source_darkness, width, height);
    let baseline_edge = edge_strength_map(baseline_darkness, width, height);
    source_edge
        .into_iter()
        .zip(baseline_edge)
        .map(|(source, baseline)| (source - baseline).max(0.0))
        .collect()
}

fn stroke_missing_structure_utility(
    stroke: &Stroke,
    darkness: &[f32],
    tone_residual: &[f32],
    missing_edge: &[f32],
    width: usize,
    height: usize,
    field: &OrientationField,
) -> (f32, f32) {
    let dx = stroke.x1 - stroke.x0;
    let dy = stroke.y1 - stroke.y0;
    let length = dx.hypot(dy);
    if length < 0.1 {
        return (0.0, 0.0);
    }

    let samples = (length / 0.75).ceil().max(2.0) as usize;
    let mut missing_sum = 0.0_f32;
    let mut residual_sum = 0.0_f32;
    let mut target_sum = 0.0_f32;
    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        let sx = stroke.x0 + dx * t;
        let sy = stroke.y0 + dy * t;
        if sx < 0.0 || sy < 0.0 || sx >= width as f32 || sy >= height as f32 {
            return (0.0, 0.0);
        }
        let index = sy.floor() as usize * width + sx.floor() as usize;
        missing_sum += missing_edge[index];
        residual_sum += tone_residual[index];
        target_sum += darkness[index];
    }
    let count = (samples + 1) as f32;
    let mean_missing = missing_sum / count;
    let mean_residual = residual_sum / count;
    let mean_target = target_sum / count;

    // Require some actual baseline structural deficit. Pure tonal darkness
    // cannot earn a P3-A.2.2 structural replacement by itself.
    if mean_missing <= 0.008 {
        return (0.0, mean_missing);
    }

    let mx = ((stroke.x0 + stroke.x1) * 0.5)
        .floor()
        .clamp(0.0, width.saturating_sub(1) as f32) as usize;
    let my = ((stroke.y0 + stroke.y1) * 0.5)
        .floor()
        .clamp(0.0, height.saturating_sub(1) as f32) as usize;
    let tx = dx / length;
    let ty = dy / length;
    let structural_alignment = choose_direction(field, mx, my)
        .map(|direction| {
            let alignment = (tx * direction.tx + ty * direction.ty).abs();
            quality(direction) * alignment
        })
        .unwrap_or(0.0);

    let deficit = 0.78 * mean_missing + 0.22 * mean_residual.min(1.0);
    let utility = deficit
        * (0.62 + 0.88 * structural_alignment)
        * (0.78 + 0.22 * mean_target);
    (utility, mean_missing)
}

/// P3-A.2.2: selective hybrid replacement driven by structural deficit in the
/// frozen P2-B.1 preview. Candidate generation and replacement controls remain
/// identical to P3-A.2.1; only the shared utility changes.
pub fn generate_missing_structure_sketch_with_stats(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<(Sketch, MissingStructureStats), String> {
    let (width, height) = source.dimensions();
    let mut tonal_options = options.clone();
    tonal_options.enable_contours = false;
    let tone = generate_sketch(source, &tonal_options)?;

    if !options.enable_contours {
        return Ok((
            tone.clone(),
            MissingStructureStats {
                tone_count: tone.strokes.len(),
                ..MissingStructureStats::default()
            },
        ));
    }

    let baseline = generate_sketch(source, options)?;
    if baseline.strokes.len() < tone.strokes.len()
        || baseline.strokes[..tone.strokes.len()] != tone.strokes[..]
    {
        return Err(
            "P3-A.2.2 invariant failed: P2-B.1 tonal prefix changed".into(),
        );
    }

    let budget = baseline.strokes.len() - tone.strokes.len();
    if budget == 0 {
        return Ok((
            tone.clone(),
            MissingStructureStats {
                tone_count: tone.strokes.len(),
                ..MissingStructureStats::default()
            },
        ));
    }

    let source_darkness = darkness_map(source);

    let tone_preview = pixmap_rgba(&tone)?;
    let tone_darkness = darkness_map(&tone_preview);
    let tone_residual: Vec<f32> = source_darkness
        .iter()
        .zip(&tone_darkness)
        .map(|(&target, &preview)| (target - preview).max(0.0))
        .collect();

    let baseline_preview = pixmap_rgba(&baseline)?;
    let baseline_darkness = darkness_map(&baseline_preview);
    let missing_edge = missing_edge_residual(
        &source_darkness,
        &baseline_darkness,
        width as usize,
        height as usize,
    );

    let field = OrientationField::new(
        &source_darkness,
        width as usize,
        height as usize,
    );

    // Deliberately reuse the P3-A.2/P3-A.2.1 candidate generator unchanged.
    let candidates = hybrid_candidates(
        &source_darkness,
        &tone_residual,
        width as usize,
        height as usize,
        &field,
        options,
    );
    let generated = candidates.len();

    let max_replacements = ((budget * SELECTIVE_MAX_PERCENT) / 100)
        .max(1)
        .min(budget);
    let preselection_limit = (max_replacements * 3).min(budget);
    let spaced = select_spaced(
        candidates,
        preselection_limit,
        width as usize,
        height as usize,
    );

    let mut scored_hybrids: Vec<(Candidate, f32, f32)> = spaced
        .into_iter()
        .map(|candidate| {
            let (utility, missing) = stroke_missing_structure_utility(
                &candidate.stroke,
                &source_darkness,
                &tone_residual,
                &missing_edge,
                width as usize,
                height as usize,
                &field,
            );
            (candidate, utility, missing)
        })
        .filter(|(_, utility, _)| *utility > 0.0)
        .collect();
    scored_hybrids.sort_by(|(a, au, _), (b, bu, _)| {
        bu.total_cmp(au)
            .then_with(|| a.y.cmp(&b.y))
            .then_with(|| a.x.cmp(&b.x))
    });

    let baseline_contours = &baseline.strokes[tone.strokes.len()..];
    let mut baseline_scores: Vec<(usize, f32)> = baseline_contours
        .iter()
        .enumerate()
        .map(|(index, stroke)| {
            let (utility, _) = stroke_missing_structure_utility(
                stroke,
                &source_darkness,
                &tone_residual,
                &missing_edge,
                width as usize,
                height as usize,
                &field,
            );
            (index, utility)
        })
        .collect();
    baseline_scores.sort_by(|(ai, au), (bi, bu)| {
        au.total_cmp(bu).then_with(|| ai.cmp(bi))
    });

    let weakest_baseline_utility =
        baseline_scores.first().map(|(_, score)| *score).unwrap_or(0.0);
    let strongest_hybrid_utility =
        scored_hybrids.first().map(|(_, score, _)| *score).unwrap_or(0.0);

    let mut structural = baseline_contours.to_vec();
    let mut replacements = 0_usize;
    let mut replacement_missing_sum = 0.0_f32;

    for (candidate, hybrid_utility, candidate_missing) in scored_hybrids {
        if replacements >= max_replacements {
            break;
        }
        let Some(&(baseline_index, baseline_utility)) =
            baseline_scores.get(replacements)
        else {
            break;
        };
        let required = baseline_utility * SELECTIVE_RELATIVE_MARGIN
            + SELECTIVE_ABSOLUTE_MARGIN;
        if hybrid_utility < required {
            break;
        }

        structural[baseline_index] = candidate.stroke;
        replacements += 1;
        replacement_missing_sum += candidate_missing;
    }

    let mut strokes = tone.strokes.clone();
    strokes.extend(structural);
    if strokes.len() != baseline.strokes.len() {
        return Err(
            "P3-A.2.2 failed to preserve the frozen P2-B.1 total stroke count"
                .into(),
        );
    }
    if strokes.len() > options.max_strokes {
        return Err(
            "stroke budget exceeded during P3-A.2.2 missing-structure pass"
                .into(),
        );
    }

    Ok((
        Sketch {
            width,
            height,
            seed: options.seed,
            strokes,
        },
        MissingStructureStats {
            tone_count: tone.strokes.len(),
            structural_budget: budget,
            generated_hybrid_candidates: generated,
            max_replacements,
            replacements_made: replacements,
            baseline_contours_retained: budget - replacements,
            weakest_baseline_utility,
            strongest_hybrid_utility,
            mean_missing_edge_at_replacements: if replacements > 0 {
                replacement_missing_sum / replacements as f32
            } else {
                0.0
            },
        },
    ))
}

pub fn generate_missing_structure_sketch(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<Sketch, String> {
    generate_missing_structure_sketch_with_stats(source, options)
        .map(|(sketch, _)| sketch)
}

/// P3-A.2.3: keep P3-A.2.2 scoring/replacement controls frozen but drive
/// proposal anchors from the missing-edge deficit itself instead of tonal
/// residual. This isolates candidate availability as the changed variable.
pub fn generate_deficit_proposal_sketch_with_stats(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<(Sketch, DeficitProposalStats), String> {
    let (width, height) = source.dimensions();
    let mut tonal_options = options.clone();
    tonal_options.enable_contours = false;
    let tone = generate_sketch(source, &tonal_options)?;

    if !options.enable_contours {
        return Ok((
            tone.clone(),
            DeficitProposalStats {
                tone_count: tone.strokes.len(),
                ..DeficitProposalStats::default()
            },
        ));
    }

    let baseline = generate_sketch(source, options)?;
    if baseline.strokes.len() < tone.strokes.len()
        || baseline.strokes[..tone.strokes.len()] != tone.strokes[..]
    {
        return Err(
            "P3-A.2.3 invariant failed: P2-B.1 tonal prefix changed".into(),
        );
    }

    let budget = baseline.strokes.len() - tone.strokes.len();
    if budget == 0 {
        return Ok((
            tone.clone(),
            DeficitProposalStats {
                tone_count: tone.strokes.len(),
                ..DeficitProposalStats::default()
            },
        ));
    }

    let source_darkness = darkness_map(source);

    let tone_preview = pixmap_rgba(&tone)?;
    let tone_darkness = darkness_map(&tone_preview);
    let tone_residual: Vec<f32> = source_darkness
        .iter()
        .zip(&tone_darkness)
        .map(|(&target, &preview)| (target - preview).max(0.0))
        .collect();

    let baseline_preview = pixmap_rgba(&baseline)?;
    let baseline_darkness = darkness_map(&baseline_preview);
    let missing_edge = missing_edge_residual(
        &source_darkness,
        &baseline_darkness,
        width as usize,
        height as usize,
    );

    let field = OrientationField::new(
        &source_darkness,
        width as usize,
        height as usize,
    );

    // This is the single experimental change from P3-A.2.2: the exact same
    // proposal machinery receives missing-edge deficit as its anchor signal.
    let candidates = hybrid_candidates(
        &source_darkness,
        &missing_edge,
        width as usize,
        height as usize,
        &field,
        options,
    );
    let generated = candidates.len();

    let max_replacements = ((budget * SELECTIVE_MAX_PERCENT) / 100)
        .max(1)
        .min(budget);
    let preselection_limit = (max_replacements * 3).min(budget);
    let spaced = select_spaced(
        candidates,
        preselection_limit,
        width as usize,
        height as usize,
    );

    let mut scored_candidates: Vec<(Candidate, f32, f32)> = spaced
        .into_iter()
        .map(|candidate| {
            let (utility, missing) = stroke_missing_structure_utility(
                &candidate.stroke,
                &source_darkness,
                &tone_residual,
                &missing_edge,
                width as usize,
                height as usize,
                &field,
            );
            (candidate, utility, missing)
        })
        .filter(|(_, utility, _)| *utility > 0.0)
        .collect();
    scored_candidates.sort_by(|(a, au, _), (b, bu, _)| {
        bu.total_cmp(au)
            .then_with(|| a.y.cmp(&b.y))
            .then_with(|| a.x.cmp(&b.x))
    });

    let baseline_contours = &baseline.strokes[tone.strokes.len()..];
    let mut baseline_scores: Vec<(usize, f32)> = baseline_contours
        .iter()
        .enumerate()
        .map(|(index, stroke)| {
            let (utility, _) = stroke_missing_structure_utility(
                stroke,
                &source_darkness,
                &tone_residual,
                &missing_edge,
                width as usize,
                height as usize,
                &field,
            );
            (index, utility)
        })
        .collect();
    baseline_scores.sort_by(|(ai, au), (bi, bu)| {
        au.total_cmp(bu).then_with(|| ai.cmp(bi))
    });

    let weakest_baseline_utility =
        baseline_scores.first().map(|(_, score)| *score).unwrap_or(0.0);
    let strongest_candidate_utility =
        scored_candidates.first().map(|(_, score, _)| *score).unwrap_or(0.0);

    let mut structural = baseline_contours.to_vec();
    let mut replacements = 0_usize;
    let mut replacement_missing_sum = 0.0_f32;

    for (candidate, candidate_utility, candidate_missing) in scored_candidates {
        if replacements >= max_replacements {
            break;
        }
        let Some(&(baseline_index, baseline_utility)) =
            baseline_scores.get(replacements)
        else {
            break;
        };

        let required = baseline_utility * SELECTIVE_RELATIVE_MARGIN
            + SELECTIVE_ABSOLUTE_MARGIN;
        if candidate_utility < required {
            break;
        }

        structural[baseline_index] = candidate.stroke;
        replacements += 1;
        replacement_missing_sum += candidate_missing;
    }

    let mut strokes = tone.strokes.clone();
    strokes.extend(structural);
    if strokes.len() != baseline.strokes.len() {
        return Err(
            "P3-A.2.3 failed to preserve the frozen P2-B.1 total stroke count"
                .into(),
        );
    }
    if strokes.len() > options.max_strokes {
        return Err(
            "stroke budget exceeded during P3-A.2.3 deficit-proposal pass"
                .into(),
        );
    }

    Ok((
        Sketch {
            width,
            height,
            seed: options.seed,
            strokes,
        },
        DeficitProposalStats {
            tone_count: tone.strokes.len(),
            structural_budget: budget,
            generated_deficit_candidates: generated,
            max_replacements,
            replacements_made: replacements,
            baseline_contours_retained: budget - replacements,
            weakest_baseline_utility,
            strongest_candidate_utility,
            mean_missing_edge_at_replacements: if replacements > 0 {
                replacement_missing_sum / replacements as f32
            } else {
                0.0
            },
        },
    ))
}

pub fn generate_deficit_proposal_sketch(
    source: &RgbaImage,
    options: &SketchOptions,
) -> Result<Sketch, String> {
    generate_deficit_proposal_sketch_with_stats(source, options)
        .map(|(sketch, _)| sketch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{audit_white_pixels, render_sketch, AuditRoi};

    fn white(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
    }

    fn step() -> RgbaImage {
        let mut source = white(64, 64);
        for y in 0..64 {
            for x in 24..64 {
                source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        source
    }

    #[test]
    fn blank_paper_is_exactly_blank() {
        let source = white(64, 64);
        let (sketch, stats) =
            generate_hybrid_sketch_with_stats(&source, &SketchOptions::default()).unwrap();
        assert!(sketch.strokes.is_empty());
        assert_eq!(stats.tone_count, 0);
        assert_eq!(stats.structural_budget, 0);
    }

    #[test]
    fn exact_tonal_prefix_and_total_budget_match_p2b1() {
        let source = step();
        let options = SketchOptions::default();
        let baseline = generate_sketch(&source, &options).unwrap();
        let mut tonal_options = options.clone();
        tonal_options.enable_contours = false;
        let tone = generate_sketch(&source, &tonal_options).unwrap();
        let (hybrid, stats) =
            generate_hybrid_sketch_with_stats(&source, &options).unwrap();

        assert_eq!(hybrid.strokes.len(), baseline.strokes.len());
        assert_eq!(&hybrid.strokes[..tone.strokes.len()], tone.strokes.as_slice());
        assert_eq!(stats.tone_count, tone.strokes.len());
        assert_eq!(
            stats.structural_budget,
            baseline.strokes.len() - tone.strokes.len()
        );
        assert_eq!(
            stats.hybrid_selected + stats.baseline_contour_fallback,
            stats.structural_budget
        );
    }

    #[test]
    fn step_exercises_residual_aware_hybrid_selection() {
        let source = step();
        let (_, stats) =
            generate_hybrid_sketch_with_stats(&source, &SketchOptions::default()).unwrap();
        assert!(stats.structural_budget > 0);
        assert!(stats.generated_hybrid_candidates > 0);
        assert!(stats.hybrid_selected > 0);
    }

    #[test]
    fn deterministic_repeat() {
        let source = step();
        let options = SketchOptions::default();
        let a = generate_hybrid_sketch_with_stats(&source, &options).unwrap();
        let b = generate_hybrid_sketch_with_stats(&source, &options).unwrap();
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.tone_count, b.1.tone_count);
        assert_eq!(a.1.structural_budget, b.1.structural_budget);
        assert_eq!(a.1.generated_hybrid_candidates, b.1.generated_hybrid_candidates);
        assert_eq!(a.1.hybrid_selected, b.1.hybrid_selected);
        assert_eq!(a.1.baseline_contour_fallback, b.1.baseline_contour_fallback);
    }

    #[test]
    fn no_contours_returns_exact_p2a_tone() {
        let source = step();
        let options = SketchOptions {
            enable_contours: false,
            ..SketchOptions::default()
        };
        let baseline = generate_sketch(&source, &options).unwrap();
        let (hybrid, stats) = generate_hybrid_sketch_with_stats(&source, &options).unwrap();
        assert_eq!(hybrid, baseline);
        assert_eq!(stats.structural_budget, 0);
        assert_eq!(stats.hybrid_selected, 0);
    }

    #[test]
    fn missing_edge_residual_is_zero_when_source_and_preview_match() {
        let source = vec![0.0_f32; 16 * 16];
        let residual = missing_edge_residual(&source, &source, 16, 16);
        assert!(residual.iter().all(|value| *value == 0.0));
    }

    #[test]
    fn missing_structure_preserves_prefix_total_and_cap() {
        let source = step();
        let options = SketchOptions::default();
        let baseline = generate_sketch(&source, &options).unwrap();
        let mut tonal_options = options.clone();
        tonal_options.enable_contours = false;
        let tone = generate_sketch(&source, &tonal_options).unwrap();
        let (missing, stats) =
            generate_missing_structure_sketch_with_stats(&source, &options).unwrap();

        assert_eq!(missing.strokes.len(), baseline.strokes.len());
        assert_eq!(
            &missing.strokes[..tone.strokes.len()],
            tone.strokes.as_slice()
        );
        assert_eq!(
            stats.replacements_made + stats.baseline_contours_retained,
            stats.structural_budget
        );
        assert!(stats.replacements_made <= stats.max_replacements);
        assert!(stats.generated_hybrid_candidates > 0);
    }

    #[test]
    fn missing_structure_is_deterministic() {
        let source = step();
        let options = SketchOptions::default();
        let a = generate_missing_structure_sketch_with_stats(&source, &options).unwrap();
        let b = generate_missing_structure_sketch_with_stats(&source, &options).unwrap();
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.replacements_made, b.1.replacements_made);
        assert_eq!(
            a.1.baseline_contours_retained,
            b.1.baseline_contours_retained
        );
        assert_eq!(
            a.1.mean_missing_edge_at_replacements,
            b.1.mean_missing_edge_at_replacements
        );
    }

    #[test]
    fn selective_hybrid_preserves_prefix_total_and_replacement_cap() {
        let source = step();
        let options = SketchOptions::default();
        let baseline = generate_sketch(&source, &options).unwrap();
        let mut tonal_options = options.clone();
        tonal_options.enable_contours = false;
        let tone = generate_sketch(&source, &tonal_options).unwrap();
        let (selective, stats) =
            generate_selective_hybrid_sketch_with_stats(&source, &options).unwrap();

        assert_eq!(selective.strokes.len(), baseline.strokes.len());
        assert_eq!(
            &selective.strokes[..tone.strokes.len()],
            tone.strokes.as_slice()
        );
        assert_eq!(stats.tone_count, tone.strokes.len());
        assert_eq!(
            stats.replacements_made + stats.baseline_contours_retained,
            stats.structural_budget
        );
        assert!(stats.replacements_made <= stats.max_replacements);
        assert!(stats.max_replacements <= (stats.structural_budget * 40 / 100).max(1));
        assert!(stats.generated_hybrid_candidates > 0);
    }

    #[test]
    fn selective_hybrid_is_deterministic() {
        let source = step();
        let options = SketchOptions::default();
        let a = generate_selective_hybrid_sketch_with_stats(&source, &options).unwrap();
        let b = generate_selective_hybrid_sketch_with_stats(&source, &options).unwrap();
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.replacements_made, b.1.replacements_made);
        assert_eq!(
            a.1.baseline_contours_retained,
            b.1.baseline_contours_retained
        );
        assert_eq!(a.1.max_replacements, b.1.max_replacements);
    }

    #[test]
    fn selective_no_contours_is_exact_p2a() {
        let source = step();
        let options = SketchOptions {
            enable_contours: false,
            ..SketchOptions::default()
        };
        let baseline = generate_sketch(&source, &options).unwrap();
        let (selective, stats) =
            generate_selective_hybrid_sketch_with_stats(&source, &options).unwrap();
        assert_eq!(selective, baseline);
        assert_eq!(stats.structural_budget, 0);
        assert_eq!(stats.replacements_made, 0);
    }

    #[test]
    fn deficit_proposals_preserve_prefix_total_and_cap() {
        let source = step();
        let options = SketchOptions::default();
        let baseline = generate_sketch(&source, &options).unwrap();
        let mut tonal_options = options.clone();
        tonal_options.enable_contours = false;
        let tone = generate_sketch(&source, &tonal_options).unwrap();
        let (deficit, stats) =
            generate_deficit_proposal_sketch_with_stats(&source, &options).unwrap();

        assert_eq!(deficit.strokes.len(), baseline.strokes.len());
        assert_eq!(
            &deficit.strokes[..tone.strokes.len()],
            tone.strokes.as_slice()
        );
        assert_eq!(
            stats.replacements_made + stats.baseline_contours_retained,
            stats.structural_budget
        );
        assert!(stats.replacements_made <= stats.max_replacements);
        assert!(stats.generated_deficit_candidates > 0);
    }

    #[test]
    fn deficit_proposals_are_deterministic() {
        let source = step();
        let options = SketchOptions::default();
        let a =
            generate_deficit_proposal_sketch_with_stats(&source, &options).unwrap();
        let b =
            generate_deficit_proposal_sketch_with_stats(&source, &options).unwrap();
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.replacements_made, b.1.replacements_made);
        assert_eq!(
            a.1.generated_deficit_candidates,
            b.1.generated_deficit_candidates
        );
        assert_eq!(
            a.1.mean_missing_edge_at_replacements,
            b.1.mean_missing_edge_at_replacements
        );
    }

    #[test]
    fn deficit_no_contours_is_exact_p2a() {
        let source = step();
        let options = SketchOptions {
            enable_contours: false,
            ..SketchOptions::default()
        };
        let baseline = generate_sketch(&source, &options).unwrap();
        let (deficit, stats) =
            generate_deficit_proposal_sketch_with_stats(&source, &options).unwrap();
        assert_eq!(deficit, baseline);
        assert_eq!(stats.structural_budget, 0);
        assert_eq!(stats.replacements_made, 0);
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
        let sketch = generate_hybrid_sketch(&source, &SketchOptions::default()).unwrap();
        let preview = render_sketch(&sketch).unwrap();
        let mut rgba = RgbaImage::new(64, 64);
        for (pixel, bytes) in rgba.pixels_mut().zip(preview.data().chunks_exact(4)) {
            *pixel = Rgba([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        let audit = audit_white_pixels(
            &source,
            &rgba,
            Some(AuditRoi { x: 29, y: 8, width: 6, height: 48 }),
        ).unwrap();
        assert_eq!(audit.roi_source_white_pixels, Some(288));
        assert_eq!(audit.roi_affected_white_pixels, Some(0));
    }
}
