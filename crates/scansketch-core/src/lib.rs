//! ScanSketch's native, UI-independent stroke reconstruction engine.
//!
//! P2-B.1 optionally reinforces coherent image-derived contours after P2-A.
//! No semantic face recognition, optimization, erasure or AI.
//! The output is a sequence of strokes; a PNG is only a rendering of that data.

mod analysis;
mod contour;
mod directional;
mod hybrid;
mod placement;
mod metrics;
mod white_audit;
mod render;
mod scanline;
mod stroke;

pub use analysis::darkness_map;
pub use render::render_sketch;
pub use metrics::{measure_sketch, EdgeMetric, Measurement, RegionMetric, StrokeMetric};
pub use white_audit::{audit_white_pixels, AffectedPixel, AuditRoi, WhiteAudit};
pub use scanline::{generate_sketch, SketchOptions};
pub use directional::generate_directional_sketch;
pub use hybrid::{generate_hybrid_sketch, generate_hybrid_sketch_with_stats, HybridStats};
pub use placement::{generate_placement_sketch, generate_placement_sketch_with_stats, PlacementStats};
pub use stroke::{Sketch, Stroke};

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba(rgba))
    }

    #[test]
    fn white_paper_is_left_untouched() {
        let image = solid(48, 24, [255, 255, 255, 255]);
        let sketch = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(sketch.strokes.is_empty());
        let preview = render_sketch(&sketch).unwrap();
        assert!(preview.data().chunks_exact(4).all(|px| px == [255, 255, 255, 255]));
    }

    #[test]
    fn transparent_black_on_white_matte_is_unmarked() {
        let image = solid(48, 24, [0, 0, 0, 0]);
        let sketch = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(sketch.strokes.is_empty());
    }

    #[test]
    fn black_source_produces_actual_visible_strokes() {
        let image = solid(48, 24, [0, 0, 0, 255]);
        let sketch = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(!sketch.strokes.is_empty());
        assert!(sketch.strokes.iter().all(|s| s.opacity > 0.0 && s.x1 > s.x0));
        let preview = render_sketch(&sketch).unwrap();
        assert!(preview.data().chunks_exact(4).any(|px| px[0] < 255));
    }

    #[test]
    fn strokes_do_not_cross_a_white_half_in_an_aligned_fixture() {
        let mut image = solid(48, 24, [255, 255, 255, 255]);
        for y in 0..24 {
            for x in 24..48 {
                image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let sketch = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(!sketch.strokes.is_empty());
        assert!(sketch.strokes.iter().all(|s| s.x0 >= 24.0 && s.x1 <= 48.0));
    }


    #[test]
    fn nonaligned_white_boundary_is_not_inked() {
        // Column 23 sits inside an 8px segment, not on its boundary.
        let mut image = solid(48, 24, [255, 255, 255, 255]);
        for y in 0..24 {
            for x in 23..48 {
                image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let sketch = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(!sketch.strokes.is_empty());
        assert!(sketch.strokes.iter().all(|s| s.x0 >= 23.0));
    }

    #[test]
    fn bright_gap_inside_dark_segment_splits_pencil_marks() {
        let mut image = solid(16, 9, [0, 0, 0, 255]);
        for y in 0..9 {
            image.put_pixel(3, y, Rgba([255, 255, 255, 255]));
            image.put_pixel(4, y, Rgba([255, 255, 255, 255]));
        }
        let sketch = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(!sketch.strokes.is_empty());
        assert!(sketch.strokes.iter().all(|s| s.x1 <= 3.0 || s.x0 >= 5.0));
    }

    #[test]
    fn identical_input_and_seed_produce_identical_stroke_records() {
        let image = solid(50, 26, [75, 85, 95, 255]);
        let opts = SketchOptions::default();
        let a = generate_sketch(&image, &opts).unwrap();
        let b = generate_sketch(&image, &opts).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn rejects_exceeded_stroke_budget_without_partial_success() {
        let image = solid(48, 24, [0, 0, 0, 255]);
        let opts = SketchOptions {
            max_strokes: 1,
            ..SketchOptions::default()
        };
        assert!(generate_sketch(&image, &opts).unwrap_err().contains("stroke budget"));
    }

    #[test]
    fn rejects_invalid_options_and_unbounded_images() {
        let image = solid(8, 8, [0, 0, 0, 255]);
        let invalid = SketchOptions {
            white_threshold: f32::NAN,
            ..SketchOptions::default()
        };
        assert!(generate_sketch(&image, &invalid).is_err());

        let oversized = solid(1025, 2, [255, 255, 255, 255]);
        assert!(generate_sketch(&oversized, &SketchOptions::default()).is_err());
    }

    #[test]
    fn generated_lines_remain_within_the_working_canvas() {
        let image = solid(31, 17, [0, 0, 0, 255]);
        let sketch = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(!sketch.strokes.is_empty());
        for s in &sketch.strokes {
            assert!(s.x0 >= 0.0 && s.x1 <= 31.0);
            assert!(s.y0 >= 0.0 && s.y0 <= 17.0);
            assert!(s.y1 >= 0.0 && s.y1 <= 17.0);
        }
    }

    #[test]
    fn p2_long_run_is_multiple_short_pencil_fragments() {
        let image = solid(48, 9, [0, 0, 0, 255]);
        let opts = SketchOptions {
            segment_width: 48,
            ..SketchOptions::default()
        };
        let sketch = generate_sketch(&image, &opts).unwrap();
        let first_band: Vec<_> = sketch.strokes.iter().filter(|s| s.y0 < 3.0).collect();
        assert!(first_band.len() >= 4);
        assert!(first_band.iter().all(|s| s.x1 - s.x0 <= 10.5));
    }

    #[test]
    fn p2_medium_tone_has_real_gaps_and_angled_fragments() {
        // Midtone has one layer, so gaps can be inspected in drawing order.
        let image = solid(48, 3, [175, 175, 175, 255]);
        let opts = SketchOptions {
            segment_width: 48,
            ..SketchOptions::default()
        };
        let sketch = generate_sketch(&image, &opts).unwrap();
        assert!(sketch.strokes.len() >= 3);
        assert!(sketch.strokes.windows(2).any(|pair| pair[1].x0 > pair[0].x1 + 0.5));
        assert!(sketch.strokes.iter().any(|s| (s.y1 - s.y0).abs() > 0.001));
    }

    #[test]
    fn p2_fragments_do_not_cross_an_internal_white_gap() {
        let mut image = solid(48, 9, [0, 0, 0, 255]);
        for y in 0..9 {
            for x in 22..26 {
                image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        let opts = SketchOptions {
            segment_width: 48,
            ..SketchOptions::default()
        };
        let sketch = generate_sketch(&image, &opts).unwrap();
        assert!(!sketch.strokes.is_empty());
        assert!(sketch.strokes.iter().all(|s| s.x1 <= 22.0 || s.x0 >= 26.0));
    }


    #[test]
    fn p2b_opt_out_preserves_the_exact_tonal_stroke_prefix() {
        let mut image = solid(48, 32, [255, 255, 255, 255]);
        for y in 6..26 {
            for x in 12..36 {
                image.put_pixel(x, y, Rgba([30, 30, 30, 255]));
            }
        }
        let no_contours = SketchOptions {
            enable_contours: false,
            ..SketchOptions::default()
        };
        let plain = generate_sketch(&image, &no_contours).unwrap();
        let edged = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(!plain.strokes.is_empty());
        assert!(edged.strokes.len() > plain.strokes.len());
        assert_eq!(&edged.strokes[..plain.strokes.len()], plain.strokes.as_slice());
        assert_eq!(edged, generate_sketch(&image, &SketchOptions::default()).unwrap());
    }

    #[test]
    fn p2b_keeps_white_gap_clear_with_contours_enabled() {
        let mut image = solid(48, 27, [0, 0, 0, 255]);
        for y in 0..27 {
            for x in 20..27 {
                image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        let sketch = generate_sketch(&image, &SketchOptions::default()).unwrap();
        assert!(!sketch.strokes.is_empty());
        assert!(sketch.strokes.iter().all(|s| {
            (s.x0 <= 20.0 && s.x1 <= 20.0) || (s.x0 >= 27.0 && s.x1 >= 27.0)
        }));
    }

    #[test]
    fn p2b_contours_respect_shared_stroke_budget() {
        let mut image = solid(48, 32, [255, 255, 255, 255]);
        for y in 4..28 {
            for x in 12..36 {
                image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let plain = generate_sketch(&image, &SketchOptions {
            enable_contours: false,
            ..SketchOptions::default()
        }).unwrap();
        assert!(!plain.strokes.is_empty());
        let constrained = SketchOptions {
            max_strokes: plain.strokes.len(),
            ..SketchOptions::default()
        };
        assert!(generate_sketch(&image, &constrained)
            .unwrap_err()
            .contains("stroke budget"));
    }

    #[test]
    fn p2b_rejects_invalid_contour_thresholds() {
        let image = solid(8, 8, [0, 0, 0, 255]);
        for value in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            let opts = SketchOptions {
                contour_threshold: value,
                ..SketchOptions::default()
            };
            assert!(generate_sketch(&image, &opts).is_err());
        }
    }

}
