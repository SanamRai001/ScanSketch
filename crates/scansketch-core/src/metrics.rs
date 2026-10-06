//! P2-C v1: reproducible comparisons, NOT a perceptual sketch-quality score.
//! Source and preview must already share exact dimensions and the same
//! white-matte linear-light normalization. No rendering or renderer mutation.

use image::RgbaImage;
use serde::Serialize;

use crate::analysis::darkness_map;
use crate::stroke::{Sketch, StrokeRole};

pub(crate) const WHITE_LIMIT: f32 = 0.04;
const DARK_START: f32 = 0.65;
const EDGE_THRESHOLD: f32 = 0.22;
const EDGE_TOLERANCE: usize = 2;

#[derive(Clone, Debug, Serialize)]
pub struct RegionMetric {
    pub pixels: usize,
    /// None for an absent region, rather than a misleading zero.
    pub rmse: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EdgeMetric {
    pub threshold: f32,
    pub tolerance_px: usize,
    pub source_pixels: usize,
    pub preview_pixels: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct StrokeMetric {
    pub count: usize,
    pub total_path_length_px: f64,
    pub mean_path_length_px: Option<f64>,
    pub mean_width_px: Option<f64>,
    pub median_width_px: Option<f64>,
    pub mean_opacity: Option<f64>,
    pub median_opacity: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PathMetric {
    pub count: usize,
    pub total_path_length_px: f64,
    pub mean_path_length_px: Option<f64>,
    pub mean_points_per_path: Option<f64>,
    pub mean_width_px: Option<f64>,
    pub median_width_px: Option<f64>,
    pub mean_opacity: Option<f64>,
    pub median_opacity: Option<f64>,
    pub gesture_count: usize,
    pub form_count: usize,
    pub hatch_count: usize,
    pub accent_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct MarkMetric {
    /// Straight segments + logical paths. One polyline path counts as one mark.
    pub logical_count: usize,
    pub segment_count: usize,
    pub path_count: usize,
    pub total_path_length_px: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Measurement {
    pub protocol_revision: &'static str,
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub total_pixels: usize,
    pub tone_rmse: f64,
    pub white_region: RegionMetric,
    pub midtone_region: RegionMetric,
    pub dark_region: RegionMetric,
    /// On the SOURCE white mask only: preview darkness above 0.04.
    pub unwanted_highlight_ink_fraction: Option<f64>,
    /// Mean positive (preview_darkness - target_darkness) on SOURCE white.
    pub mean_extra_highlight_darkness: Option<f64>,
    pub edges: EdgeMetric,
    /// Legacy straight-segment metrics retained for every P1-P3 report.
    pub strokes: StrokeMetric,
    /// P4 logical path metrics. Empty for all historical renderers.
    pub paths: PathMetric,
    /// Combined logical mark summary: segment records + path records.
    pub marks: MarkMetric,
}

#[derive(Default)]
struct Accumulator {
    count: usize,
    squared: f64,
}
impl Accumulator {
    fn record(&mut self, error: f64) {
        self.count += 1;
        self.squared += error * error;
    }
    fn metric(&self) -> RegionMetric {
        RegionMetric {
            pixels: self.count,
            rmse: (self.count > 0).then(|| (self.squared / self.count as f64).sqrt()),
        }
    }
}

fn median(mut values: Vec<f32>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f32::total_cmp);
    let n = values.len();
    if n % 2 == 1 {
        Some(values[n / 2] as f64)
    } else {
        Some((values[n / 2 - 1] as f64 + values[n / 2] as f64) * 0.5)
    }
}

/// One 3x3 binomial pass, then normalized Sobel magnitude.
///
/// This continuous map is the exact response family used by the P2-C binary
/// edge proxy. Keeping it here lets later research consume the same structural
/// evidence without changing the metric threshold or semantics.
pub(crate) fn edge_strength_map(
    darkness: &[f32],
    width: usize,
    height: usize,
) -> Vec<f32> {
    let mut strength = vec![0.0; darkness.len()];
    if width < 3 || height < 3 {
        return strength;
    }
    let mut horizontal = vec![0.0; darkness.len()];
    let mut blur = vec![0.0; darkness.len()];
    for y in 0..height {
        for x in 0..width {
            let l = x.saturating_sub(1);
            let r = (x + 1).min(width - 1);
            horizontal[y * width + x] =
                (darkness[y * width + l]
                    + 2.0 * darkness[y * width + x]
                    + darkness[y * width + r])
                    * 0.25;
        }
    }
    for y in 0..height {
        for x in 0..width {
            let t = y.saturating_sub(1);
            let b = (y + 1).min(height - 1);
            blur[y * width + x] =
                (horizontal[t * width + x]
                    + 2.0 * horizontal[y * width + x]
                    + horizontal[b * width + x])
                    * 0.25;
        }
    }
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let tl = blur[(y - 1) * width + x - 1];
            let tc = blur[(y - 1) * width + x];
            let tr = blur[(y - 1) * width + x + 1];
            let ml = blur[y * width + x - 1];
            let mr = blur[y * width + x + 1];
            let bl = blur[(y + 1) * width + x - 1];
            let bc = blur[(y + 1) * width + x];
            let br = blur[(y + 1) * width + x + 1];
            let gx = (-tl - 2.0 * ml - bl + tr + 2.0 * mr + br) * 0.25;
            let gy = (-tl - 2.0 * tc - tr + bl + 2.0 * bc + br) * 0.25;
            strength[y * width + x] = (gx * gx + gy * gy).sqrt().min(1.0);
        }
    }
    strength
}

/// Binary P2-C edge proxy. The threshold remains exactly unchanged.
fn edge_mask(darkness: &[f32], width: usize, height: usize) -> Vec<bool> {
    edge_strength_map(darkness, width, height)
        .into_iter()
        .map(|strength| strength >= EDGE_THRESHOLD)
        .collect()
}

fn matched(a: &[bool], other: &[bool], width: usize, height: usize) -> usize {
    let mut count = 0;
    for y in 0..height {
        for x in 0..width {
            if !a[y * width + x] {
                continue;
            }
            let found = (y.saturating_sub(EDGE_TOLERANCE)
                ..=(y + EDGE_TOLERANCE).min(height - 1))
                .any(|yy| {
                    (x.saturating_sub(EDGE_TOLERANCE)
                        ..=(x + EDGE_TOLERANCE).min(width - 1))
                        .any(|xx| other[yy * width + xx])
                });
            if found {
                count += 1;
            }
        }
    }
    count
}

fn edge_metric(source: &[f32], preview: &[f32], width: usize, height: usize) -> EdgeMetric {
    let source_map = edge_mask(source, width, height);
    let preview_map = edge_mask(preview, width, height);
    let source_pixels = source_map.iter().filter(|v| **v).count();
    let preview_pixels = preview_map.iter().filter(|v| **v).count();
    let precision = if preview_pixels == 0 {
        if source_pixels == 0 { 1.0 } else { 0.0 }
    } else {
        matched(&preview_map, &source_map, width, height) as f64 / preview_pixels as f64
    };
    let recall = if source_pixels == 0 {
        if preview_pixels == 0 { 1.0 } else { 0.0 }
    } else {
        matched(&source_map, &preview_map, width, height) as f64 / source_pixels as f64
    };
    let f1 = if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    };
    EdgeMetric {
        threshold: EDGE_THRESHOLD,
        tolerance_px: EDGE_TOLERANCE,
        source_pixels,
        preview_pixels,
        precision,
        recall,
        f1,
    }
}

/// Computes numeric diagnostic metrics; never interprets them as artistic quality.
pub fn measure_sketch(
    source: &RgbaImage,
    preview: &RgbaImage,
    sketch: &Sketch,
) -> Result<Measurement, String> {
    let (width, height) = source.dimensions();
    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return Err("working image must be 1..=1024 pixels per side".into());
    }
    if preview.dimensions() != (width, height) || (sketch.width, sketch.height) != (width, height) {
        return Err("source, preview and stroke JSON must have identical working dimensions".into());
    }
    if sketch.logical_mark_count() > 500_000 || sketch.paths.len() > 100_000 {
        return Err("stroke JSON exceeds maximum supported logical mark count".into());
    }

    let mut total_path_length_px = 0.0_f64;
    let mut width_sum = 0.0_f64;
    let mut opacity_sum = 0.0_f64;
    let mut widths = Vec::with_capacity(sketch.strokes.len());
    let mut opacities = Vec::with_capacity(sketch.strokes.len());
    for mark in &sketch.strokes {
        if ![mark.x0, mark.y0, mark.x1, mark.y1, mark.width, mark.opacity]
            .iter()
            .all(|v| v.is_finite())
            || mark.width <= 0.0
            || !(0.0..=1.0).contains(&mark.opacity)
            || mark.x0 < 0.0 || mark.x1 < 0.0
            || mark.y0 < 0.0 || mark.y1 < 0.0
            || mark.x0 > width as f32 || mark.x1 > width as f32
            || mark.y0 > height as f32 || mark.y1 > height as f32
        {
            return Err("invalid stroke geometry, width or opacity in JSON".into());
        }
        let dx = (mark.x1 - mark.x0) as f64;
        let dy = (mark.y1 - mark.y0) as f64;
        total_path_length_px += (dx * dx + dy * dy).sqrt();
        width_sum += mark.width as f64;
        opacity_sum += mark.opacity as f64;
        widths.push(mark.width);
        opacities.push(mark.opacity);
    }

    let mut path_total_length_px = 0.0_f64;
    let mut path_width_sum = 0.0_f64;
    let mut path_opacity_sum = 0.0_f64;
    let mut path_point_count = 0_usize;
    let mut path_widths = Vec::with_capacity(sketch.paths.len());
    let mut path_opacities = Vec::with_capacity(sketch.paths.len());
    let mut gesture_count = 0_usize;
    let mut form_count = 0_usize;
    let mut hatch_count = 0_usize;
    let mut accent_count = 0_usize;
    for path in &sketch.paths {
        if !path.valid_for_canvas(width, height) {
            return Err("invalid logical path geometry, width or opacity in JSON".into());
        }
        path_total_length_px += path.path_length_px();
        path_width_sum += path.width as f64;
        path_opacity_sum += path.opacity as f64;
        path_point_count += path.points.len();
        path_widths.push(path.width);
        path_opacities.push(path.opacity);
        match path.role {
            StrokeRole::Gesture => gesture_count += 1,
            StrokeRole::Form => form_count += 1,
            StrokeRole::Hatch => hatch_count += 1,
            StrokeRole::Accent => accent_count += 1,
        }
    }

    let target = darkness_map(source);
    let output = darkness_map(preview);
    let mut all = Accumulator::default();
    let mut white = Accumulator::default();
    let mut midtone = Accumulator::default();
    let mut dark = Accumulator::default();
    let mut accidental_ink = 0_usize;
    let mut extra_white_darkness = 0.0_f64;

    for (&wanted, &got) in target.iter().zip(&output) {
        let error = (wanted - got) as f64;
        all.record(error);
        if wanted <= WHITE_LIMIT {
            white.record(error);
            if got > WHITE_LIMIT {
                accidental_ink += 1;
            }
            extra_white_darkness += (got as f64 - wanted as f64).max(0.0);
        } else if wanted < DARK_START {
            midtone.record(error);
        } else {
            dark.record(error);
        }
    }
    let n = sketch.strokes.len();
    let path_n = sketch.paths.len();
    let logical_n = sketch.logical_mark_count();
    let pixels = all.count;
    let edges = edge_metric(&target, &output, width as usize, height as usize);
    Ok(Measurement {
        protocol_revision: "p2c-v1",
        width,
        height,
        seed: sketch.seed,
        total_pixels: pixels,
        tone_rmse: (all.squared / pixels as f64).sqrt(),
        white_region: white.metric(),
        midtone_region: midtone.metric(),
        dark_region: dark.metric(),
        unwanted_highlight_ink_fraction: (white.count > 0)
            .then(|| accidental_ink as f64 / white.count as f64),
        mean_extra_highlight_darkness: (white.count > 0)
            .then(|| extra_white_darkness / white.count as f64),
        edges,
        strokes: StrokeMetric {
            count: n,
            total_path_length_px,
            mean_path_length_px: (n > 0).then(|| total_path_length_px / n as f64),
            mean_width_px: (n > 0).then(|| width_sum / n as f64),
            median_width_px: median(widths),
            mean_opacity: (n > 0).then(|| opacity_sum / n as f64),
            median_opacity: median(opacities),
        },
        paths: PathMetric {
            count: path_n,
            total_path_length_px: path_total_length_px,
            mean_path_length_px: (path_n > 0)
                .then(|| path_total_length_px / path_n as f64),
            mean_points_per_path: (path_n > 0)
                .then(|| path_point_count as f64 / path_n as f64),
            mean_width_px: (path_n > 0)
                .then(|| path_width_sum / path_n as f64),
            median_width_px: median(path_widths),
            mean_opacity: (path_n > 0)
                .then(|| path_opacity_sum / path_n as f64),
            median_opacity: median(path_opacities),
            gesture_count,
            form_count,
            hatch_count,
            accent_count,
        },
        marks: MarkMetric {
            logical_count: logical_n,
            segment_count: n,
            path_count: path_n,
            total_path_length_px: total_path_length_px + path_total_length_px,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stroke::{PathPoint, PathStroke, Stroke};
    use image::{Rgba, RgbaImage};

    fn empty(w: u32, h: u32) -> Sketch {
        Sketch::from_strokes(w, h, 42, Vec::new())
    }
    fn white(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
    }

    #[test]
    fn identical_white_source_and_preview_have_zero_error_and_no_ink() {
        let image = white(32, 32);
        let result = measure_sketch(&image, &image, &empty(32, 32)).unwrap();
        assert_eq!(result.tone_rmse, 0.0);
        assert_eq!(result.white_region.pixels, 1024);
        assert_eq!(result.white_region.rmse, Some(0.0));
        assert_eq!(result.midtone_region.rmse, None);
        assert_eq!(result.unwanted_highlight_ink_fraction, Some(0.0));
        assert_eq!(result.mean_extra_highlight_darkness, Some(0.0));
        assert_eq!(result.edges.f1, 1.0); // no source edges and no output edges
        assert_eq!(result.strokes.mean_width_px, None);
    }

    #[test]
    fn transparent_black_source_is_white_on_white_matte() {
        let original = RgbaImage::from_pixel(32, 32, Rgba([0, 0, 0, 0]));
        let report = measure_sketch(&original, &white(32, 32), &empty(32, 32)).unwrap();
        assert_eq!(report.tone_rmse, 0.0);
        assert_eq!(report.unwanted_highlight_ink_fraction, Some(0.0));
    }

    #[test]
    fn ink_on_white_is_penalized_even_without_any_source_edges() {
        let original = white(32, 32);
        let mut output = original.clone();
        output.put_pixel(16, 16, Rgba([0, 0, 0, 255]));
        let report = measure_sketch(&original, &output, &empty(32, 32)).unwrap();
        assert!((report.tone_rmse - (1.0 / 1024.0_f64).sqrt()).abs() < 1e-8);
        assert_eq!(report.unwanted_highlight_ink_fraction, Some(1.0 / 1024.0));
        assert!((report.mean_extra_highlight_darkness.unwrap() - 1.0 / 1024.0).abs() < 1e-8);
    }

    #[test]
    fn matched_step_edge_has_perfect_proxy_scores() {
        let mut image = white(48, 32);
        for y in 0..32 {
            for x in 24..48 {
                image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let report = measure_sketch(&image, &image, &empty(48, 32)).unwrap();
        assert_eq!(report.tone_rmse, 0.0);
        assert!(report.edges.source_pixels > 0);
        assert_eq!(report.edges.source_pixels, report.edges.preview_pixels);
        assert_eq!(report.edges.precision, 1.0);
        assert_eq!(report.edges.recall, 1.0);
        assert_eq!(report.edges.f1, 1.0);
        assert_eq!(report.dark_region.rmse, Some(0.0));
    }

    #[test]
    fn missing_preview_edge_has_zero_recall() {
        let mut image = white(48, 32);
        for y in 0..32 {
            for x in 24..48 {
                image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let report = measure_sketch(&image, &white(48, 32), &empty(48, 32)).unwrap();
        assert!(report.edges.source_pixels > 0);
        assert_eq!(report.edges.preview_pixels, 0);
        assert_eq!(report.edges.recall, 0.0);
        assert_eq!(report.edges.f1, 0.0);
    }

    #[test]
    fn source_preview_and_stroke_dimensions_must_agree() {
        let input = white(32, 32);
        let different = white(31, 32);
        assert!(measure_sketch(&input, &different, &empty(32, 32)).is_err());
        assert!(measure_sketch(&input, &input, &empty(32, 31)).is_err());
    }

    #[test]
    fn stroke_path_length_and_ink_stats_are_from_json() {
        let image = white(32, 32);
        let sketch = Sketch::from_strokes(
            32,
            32,
            7,
            vec![Stroke {
                x0: 1.0, y0: 2.0, x1: 4.0, y1: 6.0, width: 0.8, opacity: 0.5
            }],
        );
        let metrics = measure_sketch(&image, &image, &sketch).unwrap();
        assert_eq!(metrics.seed, 7);
        assert_eq!(metrics.strokes.count, 1);
        assert!((metrics.strokes.total_path_length_px - 5.0).abs() < 1e-8);
        assert_eq!(metrics.strokes.median_width_px, Some(0.8_f32 as f64));
        assert_eq!(metrics.strokes.mean_opacity, Some(0.5));
    }

    #[test]
    fn invalid_json_stroke_geometry_is_rejected_not_silently_scored() {
        let image = white(32, 32);
        let sketch = Sketch::from_strokes(
            32,
            32,
            42,
            vec![Stroke {
                x0: 1.0, y0: 2.0, x1: 99.0, y1: 6.0, width: 0.8, opacity: 0.5
            }],
        );
        assert!(measure_sketch(&image, &image, &sketch).is_err());
    }


    #[test]
    fn logical_path_counts_as_one_mark_and_reports_polyline_length() {
        let image = white(32, 32);
        let mut sketch = Sketch::from_strokes(32, 32, 9, Vec::new());
        sketch.paths.push(PathStroke {
            points: vec![
                PathPoint { x: 1.0, y: 1.0 },
                PathPoint { x: 4.0, y: 5.0 },
                PathPoint { x: 7.0, y: 9.0 },
            ],
            width: 1.2,
            opacity: 0.6,
            role: StrokeRole::Gesture,
        });

        let metrics = measure_sketch(&image, &image, &sketch).unwrap();
        assert_eq!(metrics.strokes.count, 0);
        assert_eq!(metrics.paths.count, 1);
        assert_eq!(metrics.paths.gesture_count, 1);
        assert!((metrics.paths.total_path_length_px - 10.0).abs() < 1e-8);
        assert_eq!(metrics.marks.logical_count, 1);
        assert_eq!(metrics.marks.segment_count, 0);
        assert_eq!(metrics.marks.path_count, 1);
        assert!((metrics.marks.total_path_length_px - 10.0).abs() < 1e-8);
    }

    #[test]
    fn invalid_logical_path_is_rejected_not_silently_scored() {
        let image = white(32, 32);
        let mut sketch = Sketch::from_strokes(32, 32, 42, Vec::new());
        sketch.paths.push(PathStroke {
            points: vec![PathPoint { x: 4.0, y: 4.0 }],
            width: 1.0,
            opacity: 0.5,
            role: StrokeRole::Gesture,
        });
        assert!(measure_sketch(&image, &image, &sketch).is_err());
    }}
