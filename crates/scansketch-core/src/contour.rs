//! Sparse, source-derived contour accents following the tonal P2-A pass.
//!
//! The Sobel response is not semantic feature detection. Non-maximum
//! suppression, local spacing, dark-side anchoring and source support checks
//! avoid converting every contrast boundary into a heavy black outline.

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

/// Normalized Sobel gradient in the same linear-darkness domain as tonal marks.
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

/// Follow the source's dark region rather than drawing strokes across white
/// pixels. Subpixel samples also check both stroke edges along its normal.
fn dark_support(
    darkness: &[f32],
    width: usize,
    height: usize,
    mark: &Stroke,
    normal_x: f32,
    normal_y: f32,
    minimum: f32,
) -> bool {
    let length = ((mark.x1 - mark.x0).powi(2) + (mark.y1 - mark.y0).powi(2)).sqrt();
    let steps = (length / 0.45).ceil().max(1.0) as usize;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let cx = mark.x0 + (mark.x1 - mark.x0) * t;
        let cy = mark.y0 + (mark.y1 - mark.y0) * t;
        for side in [-0.5_f32, 0.0, 0.5] {
            let sx = cx + normal_x * mark.width * side;
            let sy = cy + normal_y * mark.width * side;
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

/// Append sparse short accents; existing P2-A strokes are never modified.
/// The optional pass has its own RNG stream and the same global stroke limit.
pub(crate) fn append_contours(
    darkness: &[f32],
    image_width: u32,
    image_height: u32,
    options: &SketchOptions,
    strokes: &mut Vec<Stroke>,
) -> Result<(), String> {
    let width = image_width as usize;
    let height = image_height as usize;
    if width < 3 || height < 3 {
        return Ok(());
    }
    let edge = gradient_map(darkness, width, height);
    let mut occupied = vec![false; darkness.len()];
    let mut rng = ChaCha8Rng::seed_from_u64(options.seed ^ 0xC043_7A85_91F0_2B6D);
    let minimum_ink_support = options.white_threshold.max(0.14);
    // Structural ink is a sparse accent, not a competing fully shaded layer.
    let contour_cap = ((width * height) / 18).min(12_000);
    let mut count = 0_usize;

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let index = y * width + x;
            let g = edge[index];
            if g.strength < options.contour_threshold || g.strength < 0.0001 {
                continue;
            }

            // Thin the edge in its normal direction. Asymmetric tie breaking
            // chooses one pixel from a flat two-pixel Sobel ridge.
            let (sx, sy): (isize, isize) = if g.nx.abs() >= g.ny.abs() {
                (if g.nx < 0.0 { -1 } else { 1 }, 0)
            } else {
                (0, if g.ny < 0.0 { -1 } else { 1 })
            };
            let forward = (y as isize + sy) as usize * width
                + (x as isize + sx) as usize;
            let backward = (y as isize - sy) as usize * width
                + (x as isize - sx) as usize;
            if g.strength < edge[backward].strength
                || g.strength <= edge[forward].strength
            {
                continue;
            }

            // Reject nearby accepted candidates: contour strokes should read
            // as intentional little marks, never a dense black pixel outline.
            let near = (y.saturating_sub(2)..=(y + 2).min(height - 1))
                .any(|yy| (x.saturating_sub(2)..=(x + 2).min(width - 1))
                     .any(|xx| occupied[yy * width + xx]));
            if near {
                continue;
            }

            // Move ~0.8 pixel toward the *darker* side of the source edge.
            // This keeps a white-side Sobel response from dirtying highlights.
            let cx = x as f32 + 0.5 + g.nx * 0.8;
            let cy = y as f32 + 0.5 + g.ny * 0.8;
            if darkness[cy.floor() as usize * width + cx.floor() as usize]
                < minimum_ink_support
            {
                continue;
            }

            let angle_wobble = rng.gen_range(-0.08_f32..0.08_f32);
            let raw_tx = -g.ny + g.nx * angle_wobble;
            let raw_ty = g.nx + g.ny * angle_wobble;
            let tangent_norm = (raw_tx * raw_tx + raw_ty * raw_ty).sqrt();
            let tx = raw_tx / tangent_norm;
            let ty = raw_ty / tangent_norm;
            let length = 2.3 + 2.0 * g.strength + rng.gen_range(-0.35_f32..0.35_f32);
            let half = length * 0.5;
            let mark = Stroke {
                x0: cx - tx * half,
                y0: cy - ty * half,
                x1: cx + tx * half,
                y1: cy + ty * half,
                width: (0.40 + 0.20 * g.strength) * rng.gen_range(0.91_f32..1.0_f32),
                opacity: ((0.34 + 0.40 * g.strength) * rng.gen_range(0.89_f32..1.0_f32))
                    .clamp(0.0, 1.0),
            };
            let margin = mark.width * 0.5 + 0.06;
            if [mark.x0, mark.x1].iter().any(|v| *v < margin || *v > image_width as f32 - margin)
                || [mark.y0, mark.y1].iter().any(|v| *v < margin || *v > image_height as f32 - margin)
                || !dark_support(
                    darkness,
                    width,
                    height,
                    &mark,
                    g.nx,
                    g.ny,
                    minimum_ink_support,
                )
            {
                continue;
            }

            if strokes.len() >= options.max_strokes {
                return Err("stroke budget exceeded during contour pass; increase --max-strokes or disable contours".into());
            }
            strokes.push(mark);
            occupied[index] = true;
            count += 1;
            if count >= contour_cap {
                return Ok(());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_ink_and_white_have_no_structural_gradient() {
        for value in [0.0, 0.5, 1.0] {
            let d = vec![value; 24 * 16];
            assert!(gradient_map(&d, 24, 16)
                .iter()
                .all(|g| g.strength == 0.0));
        }
    }

    #[test]
    fn vertical_step_produces_vertical_contour_tangent() {
        let mut d = vec![0.0; 24 * 16];
        for y in 0..16 {
            for x in 12..24 {
                d[y * 24 + x] = 1.0;
            }
        }
        let gradient = gradient_map(&d, 24, 16);
        let g = gradient[8 * 24 + 12];
        assert!(g.strength > 0.6);
        assert!(g.nx > 0.99);
        assert!(g.ny.abs() < 0.0001);
    }
}
