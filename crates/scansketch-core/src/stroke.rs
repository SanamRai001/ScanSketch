use serde::Serialize;

/// One editable, renderer-independent pencil segment. Coordinates are in
/// working-image pixels. Drawing order is the order in Sketch::strokes.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Stroke {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub width: f32,
    pub opacity: f32,
}

/// Source of truth for the generated result; PNG is derived from these records.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Sketch {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub strokes: Vec<Stroke>,
}
