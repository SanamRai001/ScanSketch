//! Experimental P3-A.0: reorient existing tonal proposals using a multi-scale
//! structure tensor. No semantic feature detection, new ink, optimization,
//! curve primitives or extra stroke budget. Opt-in; P2-B.1 stays default.
//!
//! Tonal anchor/drawing order and count come directly from the exact P2-A
//! generator. A reliable local tangent may rotate one existing line around
//! its midpoint if its entire round-cap footprint is supported by *unsmoothed*
//! source darkness. Otherwise the original horizontal proposal is retained.
//! The unchanged P2-B.1 contour pass is appended afterwards if enabled.

use image::RgbaImage;

use crate::analysis::darkness_map;
use crate::scanline::{generate_sketch, SketchOptions};
use crate::stroke::{Sketch, Stroke};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Direction {
    pub(crate) tx: f32,
    pub(crate) ty: f32,
    pub(crate) coherence: f32,
    pub(crate) energy: f32,
}

/// Binomial kernel [1,2,1]/4 in each axis: variance = 0.5 pixel^2 per pass.
fn blur(source: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut horizontal = vec![0.0_f32; source.len()];
    let mut output = vec![0.0_f32; source.len()];
    for y in 0..height {
        for x in 0..width {
            let left = x.saturating_sub(1);
            let right = (x + 1).min(width - 1);
            horizontal[y * width + x] = (
                source[y * width + left] + 2.0 * source[y * width + x]
                    + source[y * width + right]
            ) * 0.25;
        }
    }
    for y in 0..height {
        for x in 0..width {
            let top = y.saturating_sub(1);
            let bottom = (y + 1).min(height - 1);
            output[y * width + x] = (
                horizontal[top * width + x] + 2.0 * horizontal[y * width + x]
                    + horizontal[bottom * width + x]
            ) * 0.25;
        }
    }
    output
}

fn repeated_blur(source: &[f32], width: usize, height: usize, passes: usize) -> Vec<f32> {
    let mut current = source.to_vec();
    for _ in 0..passes {
        current = blur(&current, width, height);
    }
    current
}

/// Sobel/4, followed by a locally smoothed 2x2 second-moment tensor.
/// Coherence is axial (mod 180 deg), avoiding sign flips along a pencil line.
fn tensor_directions(darkness: &[f32], width: usize, height: usize) -> Vec<Direction> {
    let length = width * height;
    let mut jxx = vec![0.0_f32; length];
    let mut jxy = vec![0.0_f32; length];
    let mut jyy = vec![0.0_f32; length];
    if width < 3 || height < 3 {
        return vec![Direction::default(); length];
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
            let index = y * width + x;
            jxx[index] = gx * gx;
            jxy[index] = gx * gy;
            jyy[index] = gy * gy;
        }
    }

    // Smooth products, not the angle: averaging axial angles directly would
    // incorrectly cancel values near 0 and 180 degrees.
    let jxx = repeated_blur(&jxx, width, height, 2);
    let jxy = repeated_blur(&jxy, width, height, 2);
    let jyy = repeated_blur(&jyy, width, height, 2);
    let mut result = vec![Direction::default(); length];
    for i in 0..length {
        let trace = jxx[i] + jyy[i];
        if trace < 0.0001 {
            continue;
        }
        let delta = jxx[i] - jyy[i];
        let coherence = (delta.hypot(2.0 * jxy[i]) / (trace + 0.000001)).clamp(0.0, 1.0);
        let normal_angle = 0.5 * (2.0 * jxy[i]).atan2(delta);
        let (sin, cos) = normal_angle.sin_cos();
        let mut tx = -sin;  // edge tangent, perpendicular to gradient normal
        let mut ty = cos;
        if tx < 0.0 {
            tx = -tx;
            ty = -ty;
        }
        result[i] = Direction { tx, ty, coherence, energy: trace };
    }
    result
}

pub(crate) struct OrientationField {
    fine: Vec<Direction>,
    coarse: Vec<Direction>,
    width: usize,
    height: usize,
}

impl OrientationField {
    pub(crate) fn new(darkness: &[f32], width: usize, height: usize) -> Self {
        // Fine ~sigma 0.71px; coarse ~sigma 2.0px at working resolution.
        let fine = repeated_blur(darkness, width, height, 1);
        let coarse = repeated_blur(darkness, width, height, 8);
        Self {
            fine: tensor_directions(&fine, width, height),
            coarse: tensor_directions(&coarse, width, height),
            width, height,
        }
    }


    /// P3-A.1 placement support: inspect the fine/coarse field directly at
    /// an image location. The returned directions retain their raw coherence
    /// and energy so the new proposal scheduler can apply its own thresholds
    /// without changing the rejected P3-A.0 `local()` behavior.
    pub(crate) fn components(&self, x: usize, y: usize) -> (Direction, Direction) {
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        let index = y * self.width + x;
        (self.fine[index], self.coarse[index])
    }

    /// Search a bounded neighborhood for the best direction at a requested
    /// scale. Unlike P3-A.0's `local()`, near-horizontal directions are valid:
    /// P3-A.1 chooses anchors first, then lets source geometry pick the tangent.
    pub(crate) fn best_near(
        &self,
        x: usize,
        y: usize,
        radius: usize,
        coarse: bool,
        min_energy: f32,
        min_coherence: f32,
    ) -> Option<Direction> {
        let mut best = None;
        let mut best_score = 0.0_f32;
        let xmax = (x + radius).min(self.width - 1);
        let ymax = (y + radius).min(self.height - 1);
        for yy in y.saturating_sub(radius)..=ymax {
            for xx in x.saturating_sub(radius)..=xmax {
                let index = yy * self.width + xx;
                let direction = if coarse { self.coarse[index] } else { self.fine[index] };
                if direction.energy < min_energy || direction.coherence < min_coherence {
                    continue;
                }
                let distance = x.abs_diff(xx) + y.abs_diff(yy);
                let strength =
                    (direction.energy / (direction.energy + 0.05)).clamp(0.0, 1.0);
                let score =
                    direction.coherence * strength / (1.0 + distance as f32 * 0.11);
                if score > best_score {
                    best_score = score;
                    best = Some(direction);
                }
            }
        }
        best
    }

    /// Stable nearby search supports a tonal midpoint a few pixels *inside*
    /// an edge without pulling marks towards unsupported white pixels.
    pub(crate) fn local(&self, x: usize, y: usize) -> Option<Direction> {
        let mut best: Option<Direction> = None;
        let mut best_score = 0.0_f32;
        let xmax = (x + 4).min(self.width - 1);
        let ymax = (y + 4).min(self.height - 1);
        for yy in y.saturating_sub(4)..=ymax {
            for xx in x.saturating_sub(4)..=xmax {
                let idx = yy * self.width + xx;
                let distance = x.abs_diff(xx) + y.abs_diff(yy);
                for (dir, min_energy, min_coherence) in [
                    (self.fine[idx], 0.012_f32, 0.64_f32),
                    (self.coarse[idx], 0.006_f32, 0.68_f32),
                ] {
                    if dir.energy < min_energy || dir.coherence < min_coherence {
                        continue;
                    }
                    // Avoid turning near-horizontal strokes for an effectively
                    // identical angle; keep these as exact baseline records.
                    if dir.ty.abs() < 0.27 {
                        continue;
                    }
                    let strength = (dir.energy / (dir.energy + 0.05)).clamp(0.0, 1.0);
                    let score = dir.coherence * strength / (1.0 + distance as f32 * 0.16);
                    if score > best_score {
                        best_score = score;
                        best = Some(dir);
                    }
                }
            }
        }
        best
    }
}

/// Conservative complete footprint check: centerline samples, projected
/// round-cap endpoints, and both lateral pen sides on UNSMOOTHED source.
/// Anti-aliased raster edge pixels can still exist close to source boundaries;
/// this constraint prevents a candidate centerline from crossing a white gap.
pub(crate) fn source_supported(
    darkness: &[f32],
    width: usize,
    height: usize,
    mark: &Stroke,
    threshold: f32,
) -> bool {
    let dx = mark.x1 - mark.x0;
    let dy = mark.y1 - mark.y0;
    let length = dx.hypot(dy);
    if length < 0.1 {
        return false;
    }
    let tx = dx / length;
    let ty = dy / length;
    let nx = -ty;
    let ny = tx;
    let radius = mark.width * 0.5 + 0.08;
    let steps = ((length + 2.0 * radius) / 0.33).ceil().max(1.0) as usize;
    for i in 0..=steps {
        let t = -radius + (length + 2.0 * radius) * i as f32 / steps as f32;
        let cx = mark.x0 + tx * t;
        let cy = mark.y0 + ty * t;
        for offset in [-radius, 0.0, radius] {
            let sx = cx + nx * offset;
            let sy = cy + ny * offset;
            if sx < 0.0 || sy < 0.0 || sx >= width as f32 || sy >= height as f32 {
                return false;
            }
            if darkness[sy.floor() as usize * width + sx.floor() as usize] <= threshold {
                return false;
            }
        }
    }
    true
}

/// Prototype mode only: preserve baseline tonal count and its index/drawing
/// order; rotate at most a bounded subset of anchors, then append the same
/// P2-B.1 source-derived contours with its own unchanged deterministic seed.
///
/// NOTE: matching stroke count and original path-length intent facilitates
/// controlled comparison but is NOT identical deposited-ink area after raster.
pub fn generate_directional_sketch(source: &RgbaImage, options: &SketchOptions) -> Result<Sketch, String> {
    // All option validation and exact baseline tone RNG live in one place.
    let mut tonal_options = options.clone();
    tonal_options.enable_contours = false;
    let mut sketch = generate_sketch(source, &tonal_options)?;
    if sketch.strokes.is_empty() {
        return Ok(sketch);
    }
    let (width, height) = (sketch.width as usize, sketch.height as usize);
    let darkness = darkness_map(source);
    let field = OrientationField::new(&darkness, width, height);

    const TILE: usize = 32;
    const PER_TILE: u8 = 12;
    let cols = width.div_ceil(TILE);
    let mut usage = vec![0_u8; cols * height.div_ceil(TILE)];
    let max_rotations = sketch.strokes.len() * 28 / 100;
    let mut rotations = 0_usize;
    let support_threshold = options.white_threshold.max(0.10);

    for mark in &mut sketch.strokes {
        if rotations >= max_rotations {
            break;
        }
        let mx = (mark.x0 + mark.x1) * 0.5;
        let my = (mark.y0 + mark.y1) * 0.5;
        let x = (mx.floor() as usize).min(width - 1);
        let y = (my.floor() as usize).min(height - 1);
        if darkness[y * width + x] <= support_threshold {
            continue;
        }
        let tile = (y / TILE) * cols + x / TILE;
        if usage[tile] >= PER_TILE {
            continue;
        }
        let Some(orientation) = field.local(x, y) else {
            continue;
        };
        let length = (mark.x1 - mark.x0).hypot(mark.y1 - mark.y0);
        let half = length * 0.5;
        let proposed = Stroke {
            x0: mx - orientation.tx * half,
            y0: my - orientation.ty * half,
            x1: mx + orientation.tx * half,
            y1: my + orientation.ty * half,
            width: mark.width,
            opacity: mark.opacity,
        };
        if source_supported(&darkness, width, height, &proposed, support_threshold) {
            *mark = proposed;
            rotations += 1;
            usage[tile] += 1;
        }
    }
    if options.enable_contours {
        crate::contour::append_contours(
            &darkness, sketch.width, sketch.height, options, &mut sketch.strokes
        )?;
    }
    Ok(sketch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{audit_white_pixels, AuditRoi};
    use image::Rgba;

    fn white(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
    }

    #[test]
    fn flat_source_has_no_direction_signal() {
        let darkness = vec![0.4; 32 * 32];
        let field = OrientationField::new(&darkness, 32, 32);
        assert!(field.local(16, 16).is_none());
    }

    #[test]
    fn vertical_boundary_has_vertical_tangent() {
        let mut source = white(64, 64);
        for y in 0..64 {
            for x in 24..64 {
                source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let field = OrientationField::new(&darkness_map(&source), 64, 64);
        let d = field.local(27, 32).expect("step should yield a coherent direction");
        assert!(d.ty.abs() > 0.7, "vertical boundary should encourage a vertical tangent");
    }

    #[test]
    fn directional_changes_geometry_not_stroke_count_or_ink_metadata() {
        let mut source = white(64, 64);
        for y in 0..64 {
            for x in 24..64 {
                source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let options = SketchOptions { enable_contours: false, ..SketchOptions::default() };
        let plain = generate_sketch(&source, &options).unwrap();
        let adjusted = generate_directional_sketch(&source, &options).unwrap();
        assert_eq!(plain.strokes.len(), adjusted.strokes.len());
        assert_eq!(plain.width, adjusted.width);
        assert_eq!(plain.seed, adjusted.seed);
        let changed: Vec<_> = plain.strokes.iter().zip(&adjusted.strokes)
            .filter(|(a,b)| a != b).collect();
        assert!(!changed.is_empty(), "vertical step must reorient eligible tonal segments");
        for (a,b) in plain.strokes.iter().zip(&adjusted.strokes) {
            assert_eq!(a.width, b.width);
            assert_eq!(a.opacity, b.opacity);
            let alen = (a.x1-a.x0).hypot(a.y1-a.y0);
            let blen = (b.x1-b.x0).hypot(b.y1-b.y0);
            assert!((alen - blen).abs() < 0.0001);
        }
        assert!(changed.iter().any(|(_,b)| (b.y1-b.y0).abs() > 2.0));
        assert_eq!(adjusted, generate_directional_sketch(&source, &options).unwrap());
    }

    #[test]
    fn exact_legacy_prefix_when_directional_not_selected() {
        // The opt-in API is separate: existing generate_sketch is unchanged.
        let mut source = white(48, 32);
        for y in 5..27 { for x in 12..36 {
            source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }}
        let options = SketchOptions::default();
        let reference = generate_sketch(&source, &options).unwrap();
        assert!(!reference.strokes.is_empty());
        let plain = generate_sketch(&source, &SketchOptions {
            enable_contours: false, ..options.clone()
        }).unwrap();
        assert_eq!(&reference.strokes[..plain.strokes.len()], plain.strokes.as_slice());
    }

    #[test]
    fn white_channel_preserved_with_optional_contours_and_directional() {
        let mut source = white(64, 64);
        for y in 8..56 { for x in 8..56 {
            if !(29..35).contains(&x) {
                source.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }}
        let opts = SketchOptions::default();
        let sketch = generate_directional_sketch(&source, &opts).unwrap();
        let render = crate::render::render_sketch(&sketch).unwrap();
        let bytes = render.data();
        let mut rgba = RgbaImage::new(64, 64);
        for (px, rendered) in rgba.pixels_mut().zip(bytes.chunks_exact(4)) {
            // tiny-skia's pixmap stores premultiplied RGBA, output white bg is opaque.
            *px = Rgba([rendered[0], rendered[1], rendered[2], rendered[3]]);
        }
        let audit = audit_white_pixels(&source, &rgba, Some(AuditRoi {
            x: 29, y: 8, width: 6, height: 48
        })).unwrap();
        assert_eq!(audit.roi_source_white_pixels, Some(288));
        assert_eq!(audit.roi_affected_white_pixels, Some(0));
    }

    #[test]
    fn blank_paper_produces_exactly_no_marks() {
        let source = white(64, 64);
        let out = generate_directional_sketch(&source, &SketchOptions::default()).unwrap();
        assert!(out.strokes.is_empty());
    }

    #[test]
    fn baseline_and_prototype_refuse_exceeded_contour_budget_consistently() {
        let mut source = white(48, 32);
        for y in 4..28 { for x in 12..36 { source.put_pixel(x,y,Rgba([0,0,0,255])); }}
        let plain = generate_sketch(&source, &SketchOptions {
            enable_contours: false, ..SketchOptions::default()
        }).unwrap();
        let opts = SketchOptions { max_strokes: plain.strokes.len(), ..SketchOptions::default() };
        assert!(generate_sketch(&source, &opts).is_err());
        assert!(generate_directional_sketch(&source, &opts).is_err());
    }
}
