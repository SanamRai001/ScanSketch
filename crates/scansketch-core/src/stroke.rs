use serde::{Deserialize, Serialize};

/// One editable, renderer-independent pencil segment. Coordinates are in
/// working-image pixels. Drawing order is the order in Sketch::strokes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub width: f32,
    pub opacity: f32,
}

/// One control/sample point in a logical path stroke.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathPoint {
    pub x: f32,
    pub y: f32,
}

/// Generic drawing role for a logical path. It describes how the mark
/// participates in a sketch, not what object or semantic feature it depicts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrokeRole {
    Gesture,
    Form,
    Hatch,
    Accent,
}

/// One editable logical pencil path.
///
/// P4-A.0 starts with deterministic polylines rather than Beziers. A path may
/// contain many geometric segments while remaining ONE logical editable mark.
/// Width and opacity are constant for the foundation slice.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathStroke {
    pub points: Vec<PathPoint>,
    pub width: f32,
    pub opacity: f32,
    pub role: StrokeRole,
}

impl PathStroke {
    pub fn path_length_px(&self) -> f64 {
        self.points
            .windows(2)
            .map(|pair| {
                let dx = (pair[1].x - pair[0].x) as f64;
                let dy = (pair[1].y - pair[0].y) as f64;
                (dx * dx + dy * dy).sqrt()
            })
            .sum()
    }

    pub(crate) fn valid_for_canvas(&self, width: u32, height: u32) -> bool {
        self.points.len() >= 2
            && self.points.len() <= 4096
            && self.width.is_finite()
            && self.width > 0.0
            && self.opacity.is_finite()
            && (0.0..=1.0).contains(&self.opacity)
            && self.points.iter().all(|point| {
                point.x.is_finite()
                    && point.y.is_finite()
                    && point.x >= 0.0
                    && point.y >= 0.0
                    && point.x <= width as f32
                    && point.y <= height as f32
            })
    }
}

/// Source of truth for the generated result; PNG is derived from these records.
///
/// paths is a backward-compatible P4 extension. Empty path collections are
/// omitted from JSON so existing P1-P3 stroke-only exports keep the same
/// serialized shape. Old JSON without paths still deserializes exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sketch {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub strokes: Vec<Stroke>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<PathStroke>,
}

impl Sketch {
    pub fn from_strokes(width: u32, height: u32, seed: u64, strokes: Vec<Stroke>) -> Self {
        Self {
            width,
            height,
            seed,
            strokes,
            paths: Vec::new(),
        }
    }

    pub fn logical_mark_count(&self) -> usize {
        self.strokes.len() + self.paths.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_length_is_polyline_arc_length() {
        let path = PathStroke {
            points: vec![
                PathPoint { x: 1.0, y: 1.0 },
                PathPoint { x: 4.0, y: 5.0 },
                PathPoint { x: 7.0, y: 9.0 },
            ],
            width: 0.8,
            opacity: 0.5,
            role: StrokeRole::Gesture,
        };
        assert!((path.path_length_px() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn old_stroke_only_json_stays_compatible_and_omits_empty_paths() {
        let old = r#"{"width":32,"height":24,"seed":42,"strokes":[]}"#;
        let sketch: Sketch = serde_json::from_str(old).unwrap();
        assert!(sketch.paths.is_empty());

        let encoded = serde_json::to_string(&sketch).unwrap();
        assert!(!encoded.contains("paths"));
        assert_eq!(encoded, old);
    }

    #[test]
    fn path_validation_rejects_single_point_and_out_of_bounds_points() {
        let single = PathStroke {
            points: vec![PathPoint { x: 4.0, y: 4.0 }],
            width: 1.0,
            opacity: 0.5,
            role: StrokeRole::Gesture,
        };
        assert!(!single.valid_for_canvas(32, 32));

        let outside = PathStroke {
            points: vec![
                PathPoint { x: 4.0, y: 4.0 },
                PathPoint { x: 33.0, y: 4.0 },
            ],
            width: 1.0,
            opacity: 0.5,
            role: StrokeRole::Gesture,
        };
        assert!(!outside.valid_for_canvas(32, 32));
    }
}
