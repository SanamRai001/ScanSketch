//! ScanSketch's native, UI-independent reconstruction baseline.
//!
//! P1 deliberately contains no optimized search, erasure, paper effects or AI.
//! The output is a sequence of strokes; a PNG is only a rendering of that data.

mod analysis;
mod render;
mod scanline;
mod stroke;

pub use analysis::darkness_map;
pub use render::render_sketch;
pub use scanline::{generate_sketch, SketchOptions};
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
}
