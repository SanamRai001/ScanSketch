use tiny_skia::{Color, LineCap, Paint, PathBuilder, Pixmap, Stroke as Pen, Transform};
use crate::stroke::Sketch;

/// Replays exactly the committed stroke list over opaque white paper.
pub fn render_sketch(sketch: &Sketch) -> Result<Pixmap, String> {
    if sketch.width == 0 || sketch.height == 0 || sketch.width > 1024 || sketch.height > 1024 {
        return Err("invalid preview dimensions".into());
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

        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, (mark.opacity * 255.0).round() as u8);
        paint.anti_alias = true;
        let mut pen = Pen::default();
        pen.width = mark.width;
        pen.line_cap = LineCap::Round;
        canvas.stroke_path(&path, &paint, &pen, Transform::identity(), None);
    }
    Ok(canvas)
}
