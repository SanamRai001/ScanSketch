use tiny_skia::{
    Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke as Pen, Transform,
};

use crate::stroke::Sketch;

fn paint_and_pen(width: f32, opacity: f32) -> (Paint<'static>, Pen) {
    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 0, 0, (opacity * 255.0).round() as u8);
    paint.anti_alias = true;

    let mut pen = Pen::default();
    pen.width = width;
    pen.line_cap = LineCap::Round;
    pen.line_join = LineJoin::Round;
    (paint, pen)
}

/// Replays exactly the committed straight segments and logical path list over
/// opaque white paper.
///
/// P1-P3 sketches contain no logical paths, so their rendering remains exactly
/// the legacy straight-segment replay. P4 path strokes are rendered as one
/// continuous tiny-skia path each.
pub fn render_sketch(sketch: &Sketch) -> Result<Pixmap, String> {
    if sketch.width == 0
        || sketch.height == 0
        || sketch.width > 1024
        || sketch.height > 1024
    {
        return Err("invalid preview dimensions".into());
    }
    if sketch.logical_mark_count() > 500_000 || sketch.paths.len() > 100_000 {
        return Err("sketch mark count exceeds supported preview limit".into());
    }

    let mut canvas = Pixmap::new(sketch.width, sketch.height)
        .ok_or_else(|| "unable to allocate sketch preview".to_string())?;
    canvas.fill(Color::WHITE);

    for mark in &sketch.strokes {
        if ![mark.x0, mark.y0, mark.x1, mark.y1, mark.width, mark.opacity]
            .iter()
            .all(|v| v.is_finite())
            || mark.width <= 0.0
            || !(0.0..=1.0).contains(&mark.opacity)
        {
            return Err("invalid stroke geometry or ink pressure".into());
        }
        if mark.opacity == 0.0 {
            continue;
        }

        let mut builder = PathBuilder::new();
        builder.move_to(mark.x0, mark.y0);
        builder.line_to(mark.x1, mark.y1);
        let path = builder
            .finish()
            .ok_or_else(|| "unable to create a pencil path".to_string())?;

        let (paint, pen) = paint_and_pen(mark.width, mark.opacity);
        canvas.stroke_path(&path, &paint, &pen, Transform::identity(), None);
    }

    for mark in &sketch.paths {
        if !mark.valid_for_canvas(sketch.width, sketch.height) {
            return Err("invalid logical path geometry or ink pressure".into());
        }
        if mark.opacity == 0.0 {
            continue;
        }

        let mut points = mark.points.iter();
        let first = points
            .next()
            .ok_or_else(|| "logical path must contain at least two points".to_string())?;
        let mut builder = PathBuilder::new();
        builder.move_to(first.x, first.y);
        for point in points {
            builder.line_to(point.x, point.y);
        }
        let path = builder
            .finish()
            .ok_or_else(|| "unable to create a logical pencil path".to_string())?;

        let (paint, pen) = paint_and_pen(mark.width, mark.opacity);
        canvas.stroke_path(&path, &paint, &pen, Transform::identity(), None);
    }

    Ok(canvas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PathPoint, PathStroke, StrokeRole};

    #[test]
    fn one_logical_curved_polyline_renders_continuously() {
        let sketch = Sketch {
            width: 64,
            height: 64,
            seed: 42,
            strokes: Vec::new(),
            paths: vec![PathStroke {
                points: vec![
                    PathPoint { x: 6.0, y: 44.0 },
                    PathPoint { x: 14.0, y: 24.0 },
                    PathPoint { x: 28.0, y: 14.0 },
                    PathPoint { x: 43.0, y: 20.0 },
                    PathPoint { x: 56.0, y: 42.0 },
                ],
                width: 1.4,
                opacity: 0.85,
                role: StrokeRole::Gesture,
            }],
        };

        let preview = render_sketch(&sketch).unwrap();
        assert!(preview.data().chunks_exact(4).any(|px| px[0] < 200));
        assert_eq!(sketch.logical_mark_count(), 1);
        assert_eq!(sketch.paths.len(), 1);
    }

    #[test]
    fn invalid_single_point_logical_path_is_rejected() {
        let sketch = Sketch {
            width: 32,
            height: 32,
            seed: 42,
            strokes: Vec::new(),
            paths: vec![PathStroke {
                points: vec![PathPoint { x: 4.0, y: 4.0 }],
                width: 1.0,
                opacity: 0.5,
                role: StrokeRole::Gesture,
            }],
        };
        assert!(render_sketch(&sketch).is_err());
    }
}
