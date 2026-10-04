//! P2-B.1: sparse, source-derived structural marks after the P2-A tonal pass.
//!
//! A small separable Gaussian reduces pixel-scale noise before Sobel. We thin
//! candidates, test tangent continuity and rank them by structural evidence,
//! instead of letting the first scanned edge claim a location. This is NOT
//! semantic recognition of facial landmarks. Ink support is checked against
//! the UNBLURRED source, preserving white gaps and highlights.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::scanline::SketchOptions;
use crate::stroke::Stroke;

#[derive(Clone, Copy, Debug, Default)]
struct Gradient {
    strength: f32,
    nx: f32,
    ny: f32,
}

#[derive(Clone, Copy, Debug)]
struct Candidate {
    index: usize,
    score: f32,
    continuity: f32,
}

/// One mild 3x3 binomial blur (separable [1,2,1]/4).
/// Sampling is clamped at the image boundary; no external imaging dependency.
fn smooth_darkness(source: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut horizontal = vec![0.0; source.len()];
    let mut blurred = vec![0.0; source.len()];
    for y in 0..height {
        for x in 0..width {
            let l = x.saturating_sub(1);
            let r = (x + 1).min(width - 1);
            horizontal[y * width + x] =
                (source[y * width + l] + 2.0 * source[y * width + x] + source[y * width + r])
                    * 0.25;
        }
    }
    for y in 0..height {
        for x in 0..width {
            let t = y.saturating_sub(1);
            let b = (y + 1).min(height - 1);
            blurred[y * width + x] =
                (horizontal[t * width + x]
                    + 2.0 * horizontal[y * width + x]
                    + horizontal[b * width + x])
                    * 0.25;
        }
    }
    blurred
}

/// Normalized Sobel response to the mildly blurred linear-light darkness map.
fn gradient_map(darkness: &[f32], width: usize, height: usize) -> Vec<Gradient> {
    let mut result = vec![Gradient::default(); darkness.len()];
    if width < 3 || height < 3 {
        return result;
    }
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let tl = darkness[(y - 1) * width + x - 1];
            let tc = darkness[(y - 1) * width + x];
            let tr = darkness[(y - 1) * width + x + 1];
            let ml = darkness[y * width + x - 1];
            let mr = darkness[y * width + x + 1];
            let bl = darkness[(y + 1) * width + x - 1];
            let bc = darkness[(y + 1) * width + x];
            let br = darkness[(y + 1) * width + x + 1];

            let gx = (-tl - 2.0 * ml - bl + tr + 2.0 * mr + br) * 0.25;
            let gy = (-tl - 2.0 * tc - tr + bl + 2.0 * bc + br) * 0.25;
            let norm = (gx * gx + gy * gy).sqrt();
            if norm > 0.0001 {
                result[y * width + x] = Gradient {
                    strength: norm.min(1.0),
                    nx: gx / norm,
                    ny: gy / norm,
                };
            }
        }
    }
    result
}

/// Nearby gradients along the edge tangent should point in a similar
/// direction. A lone high-frequency contrast speck has poor continuity.
fn tangent_continuity(edge: &[Gradient], width: usize, height: usize, x: usize, y: usize) -> f32 {
    let center = edge[y * width + x];
    let tx = -center.ny;
    let ty = center.nx;
    let mut evidence = 0.0;
    for distance in [-4.0_f32, -2.0, 2.0, 4.0] {
        let xx = (x as f32 + tx * distance).round() as isize;
        let yy = (y as f32 + ty * distance).round() as isize;
        if xx < 1 || yy < 1 || xx >= (width - 1) as isize || yy >= (height - 1) as isize {
            continue;
        }
        let near = edge[yy as usize * width + xx as usize];
        let agreement = (center.nx * near.nx + center.ny * near.ny).max(0.0);
        if near.strength >= center.strength * 0.34 && agreement > 0.72 {
            evidence += agreement * (near.strength / center.strength).min(1.0);
        }
    }
    evidence * 0.25
}

/// Thin the candidates across gradient normal, then rank by gradient strength
/// and tangent agreement. Equal scores use stable row-major coordinate order.
fn rank_candidates(
    edge: &[Gradient],
    width: usize,
    height: usize,
    threshold: f32,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for y in 2..height.saturating_sub(2) {
        for x in 2..width.saturating_sub(2) {
            let index = y * width + x;
            let g = edge[index];
            if g.strength < threshold || g.strength < 0.0001 {
                continue;
            }
            let (sx, sy): (isize, isize) = if g.nx.abs() >= g.ny.abs() {
                (if g.nx < 0.0 { -1 } else { 1 }, 0)
            } else {
                (0, if g.ny < 0.0 { -1 } else { 1 })
            };
            let ahead = (y as isize + sy) as usize * width + (x as isize + sx) as usize;
            let behind = (y as isize - sy) as usize * width + (x as isize - sx) as usize;
            if g.strength < edge[behind].strength || g.strength <= edge[ahead].strength {
                continue;
            }

            let continuity = tangent_continuity(edge, width, height, x, y);
            if continuity < 0.26 {
                continue;
            }
            candidates.push(Candidate {
                index,
                score: g.strength * (0.55 + 0.45 * continuity),
                continuity,
            });
        }
    }
    candidates.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.index.cmp(&b.index))
    });
    candidates
}

/// Source-backed mark validation. Evaluate samples along the whole line, its
/// round-cap ends and both lateral ink edges. Smoothing never authorizes ink.
fn dark_support(
    darkness: &[f32],
    width: usize,
    height: usize,
    mark: &Stroke,
    normal_x: f32,
    normal_y: f32,
    minimum: f32,
) -> bool {
    let dx = mark.x1 - mark.x0;
    let dy = mark.y1 - mark.y0;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 0.01 {
        return false;
    }
    let tx = dx / length;
    let ty = dy / length;
    let extension = mark.width * 0.5 + 0.07;
    let reach = mark.width * 0.5 + 0.09;
    let steps = ((length + 2.0 * extension) / 0.38).ceil().max(1.0) as usize;
    for step in 0..=steps {
        let distance = -extension + (length + 2.0 * extension) * step as f32 / steps as f32;
        let cx = mark.x0 + tx * distance;
        let cy = mark.y0 + ty * distance;
        for lateral in [-reach, 0.0, reach] {
            let sx = cx + normal_x * lateral;
            let sy = cy + normal_y * lateral;
            if !(0.0..width as f32).contains(&sx)
                || !(0.0..height as f32).contains(&sy)
                || darkness[sy.floor() as usize * width + sx.floor() as usize] < minimum
            {
                return false;
            }
        }
    }
    true
}

/// Append ranked, spaced structure marks. Never changes P2-A tonal strokes;
/// separate RNG means `--no-contours` produces their exact same records.
pub(crate) fn append_contours(
    darkness: &[f32],
    image_width: u32,
    image_height: u32,
    options: &SketchOptions,
    strokes: &mut Vec<Stroke>,
) -> Result<(), String> {
    let width = image_width as usize;
    let height = image_height as usize;
    if width < 5 || height < 5 {
        return Ok(());
    }

    let smooth = smooth_darkness(darkness, width, height);
    let edge = gradient_map(&smooth, width, height);
    let candidates = rank_candidates(&edge, width, height, options.contour_threshold);
    let mut occupied = vec![false; darkness.len()];
    let mut rng = ChaCha8Rng::seed_from_u64(options.seed ^ 0xC043_7A85_91F0_2B6D);
    let minimum_ink_support = options.white_threshold.max(0.14);

    // Spatial quota prevents a textured region from consuming the entire
    // optional contour pass; global budget is still shared with tonal marks.
    const TILE: usize = 32;
    const PER_TILE: u8 = 10;
    let tile_columns = width.div_ceil(TILE);
    let tile_rows = height.div_ceil(TILE);
    let mut tile_usage = vec![0_u8; tile_columns * tile_rows];
    let contour_cap = ((width * height) / 30).min(8_000);
    let mut accepted = 0_usize;

    for candidate in candidates {
        let index = candidate.index;
        let x = index % width;
        let y = index / width;
        let tile = (y / TILE) * tile_columns + (x / TILE);
        if tile_usage[tile] >= PER_TILE {
            continue;
        }
        let nearby = (y.saturating_sub(2)..=(y + 2).min(height - 1)).any(|yy| {
            (x.saturating_sub(2)..=(x + 2).min(width - 1))
                .any(|xx| occupied[yy * width + xx])
        });
        if nearby {
            continue;
        }

        let g = edge[index];
        // Direction of increasing source darkness, not the smoothed value.
        let cx = x as f32 + 0.5 + g.nx * 0.86;
        let cy = y as f32 + 0.5 + g.ny * 0.86;
        if darkness[cy.floor() as usize * width + cx.floor() as usize] < minimum_ink_support {
            continue;
        }

        let wobble = rng.gen_range(-0.045_f32..0.045_f32);
        let vx = -g.ny + g.nx * wobble;
        let vy = g.nx + g.ny * wobble;
        let norm = (vx * vx + vy * vy).sqrt();
        let tx = vx / norm;
        let ty = vy / norm;
        let width_mark = (0.45 + 0.16 * g.strength) * rng.gen_range(0.93_f32..1.0_f32);
        let opacity = ((0.44 + 0.38 * g.strength + 0.08 * candidate.continuity)
            * rng.gen_range(0.93_f32..1.0_f32))
            .clamp(0.0, 0.92);
        let desired_length = 3.2 + 1.9 * g.strength + candidate.continuity
            + rng.gen_range(-0.25_f32..0.30_f32);

        // A shorter mark can fit next to an irregular highlight even if the
        // preferred long mark cannot. Never clip only one endpoint.
        let mut chosen: Option<Stroke> = None;
        for scale in [1.0_f32, 0.72, 0.52] {
            let half = desired_length * scale * 0.5;
            let mark = Stroke {
                x0: cx - tx * half,
                y0: cy - ty * half,
                x1: cx + tx * half,
                y1: cy + ty * half,
                width: width_mark,
                opacity,
            };
            let margin = mark.width * 0.5 + 0.12;
            if [mark.x0, mark.x1]
                .iter()
                .any(|v| *v < margin || *v > image_width as f32 - margin)
                || [mark.y0, mark.y1]
                    .iter()
                    .any(|v| *v < margin || *v > image_height as f32 - margin)
            {
                continue;
            }
            if dark_support(
                darkness,
                width,
                height,
                &mark,
                g.nx,
                g.ny,
                minimum_ink_support,
            ) {
                chosen = Some(mark);
                break;
            }
        }
        let Some(mark) = chosen else {
            continue;
        };
        if strokes.len() >= options.max_strokes {
            return Err("stroke budget exceeded during contour pass; increase --max-strokes or disable contours".into());
        }
        strokes.push(mark);
        occupied[index] = true;
        tile_usage[tile] += 1;
        accepted += 1;
        if accepted >= contour_cap {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_input_stays_flat_after_smoothing_and_sobel() {
        for value in [0.0, 0.5, 1.0] {
            let d = vec![value; 24 * 16];
            let blur = smooth_darkness(&d, 24, 16);
            assert!(blur.iter().all(|v| (v - value).abs() < 0.00001));
            assert!(gradient_map(&blur, 24, 16)
                .iter()
                .all(|g| g.strength == 0.0));
        }
    }

    #[test]
    fn light_blur_reduces_isolated_dark_speck() {
        let mut d = vec![0.0; 17 * 17];
        d[8 * 17 + 8] = 1.0;
        let blurred = smooth_darkness(&d, 17, 17);
        assert!((blurred[8 * 17 + 8] - 0.25).abs() < 0.00001);
        assert!((blurred.iter().sum::<f32>() - 1.0).abs() < 0.00001);
    }

    #[test]
    fn vertical_step_has_a_continuous_vertical_contour() {
        let mut d = vec![0.0; 32 * 24];
        for y in 0..24 {
            for x in 16..32 {
                d[y * 32 + x] = 1.0;
            }
        }
        let edge = gradient_map(&smooth_darkness(&d, 32, 24), 32, 24);
        let g = edge[12 * 32 + 16];
        assert!(g.strength > 0.6);
        assert!(g.nx > 0.99);
        assert!(g.ny.abs() < 0.0001);
        let ranked = rank_candidates(&edge, 32, 24, 0.28);
        assert!(!ranked.is_empty());
        assert!(ranked.iter().any(|c| c.continuity > 0.70));
    }

    #[test]
    fn isolated_speck_gets_fewer_contour_candidates_than_a_long_edge() {
        let (width, height) = (48, 32);
        let mut isolated = vec![0.0; width * height];
        isolated[16 * width + 24] = 1.0;
        let isolated_edges = gradient_map(
            &smooth_darkness(&isolated, width, height), width, height,
        );
        let isolated_count = rank_candidates(&isolated_edges, width, height, 0.28).len();

        let mut step = vec![0.0; width * height];
        for y in 0..height {
            for x in 24..width {
                step[y * width + x] = 1.0;
            }
        }
        let step_edges = gradient_map(&smooth_darkness(&step, width, height), width, height);
        let step_count = rank_candidates(&step_edges, width, height, 0.28).len();
        assert!(step_count > isolated_count);
        assert!(step_count >= 10);
    }

    #[test]
    fn contour_candidates_rank_by_quality_then_pixel_index() {
        let mut edge = vec![Gradient::default(); 48 * 32];
        for y in 3..29 {
            for (x, strength) in [(10, 0.9), (30, 0.6)] {
                edge[y * 48 + x] = Gradient {
                    strength,
                    nx: 1.0,
                    ny: 0.0,
                };
            }
        }
        let candidates = rank_candidates(&edge, 48, 32, 0.28);
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].index % 48, 10);
        assert!(candidates.windows(2).all(|c| {
            c[0].score > c[1].score
                || (c[0].score == c[1].score && c[0].index < c[1].index)
        }));
    }
}
