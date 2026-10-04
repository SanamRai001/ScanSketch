//! Separate P2-C white-region audit. Does not change p2c-v1 metrics or renderer.
//! Counts source-white preview pixels above the identical 0.04 darkness cutoff,
//! then locates them inside a caller-specified rectangular region and within
//! 1 or 2 source-pixel steps of genuinely non-white target pixels.
use image::RgbaImage;
use serde::Serialize;
use crate::{analysis::darkness_map, metrics::WHITE_LIMIT};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AuditRoi {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl AuditRoi {
    fn contains(self, x: usize, y: usize) -> bool {
        x >= self.x as usize && x < (self.x + self.width) as usize
            && y >= self.y as usize && y < (self.y + self.height) as usize
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AffectedPixel {
    pub x: u32,
    pub y: u32,
    pub preview_darkness: f32,
    pub inside_roi: Option<bool>,
    /// A source non-white pixel is within a Chebyshev distance of one.
    pub near_source_nonwhite_1px: bool,
    /// A source non-white pixel is within a Chebyshev distance of two.
    pub near_source_nonwhite_2px: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WhiteAudit {
    pub audit_revision: &'static str,
    pub width: u32,
    pub height: u32,
    pub white_limit: f32,
    pub source_white_pixels: usize,
    pub affected_white_pixels: usize,
    pub near_source_nonwhite_1px: usize,
    pub near_source_nonwhite_2px: usize,
    /// Source-white pixels with preview ink farther than 2px from source dark.
    pub farther_than_2px: usize,
    pub roi: Option<AuditRoi>,
    pub roi_source_white_pixels: Option<usize>,
    pub roi_affected_white_pixels: Option<usize>,
    pub outside_roi_affected_white_pixels: Option<usize>,
    /// First up to 128 coordinates, stable row-major order (all 53 for this fixture).
    pub affected_coordinates: Vec<AffectedPixel>,
    pub coordinates_truncated: bool,
}

fn source_has_dark_near(source: &[f32], w: usize, h: usize, x: usize, y: usize, radius: usize) -> bool {
    (y.saturating_sub(radius)..=(y + radius).min(h - 1)).any(|yy| {
        (x.saturating_sub(radius)..=(x + radius).min(w - 1))
            .any(|xx| source[yy * w + xx] > WHITE_LIMIT)
    })
}

/// Source and preview must be at exactly the same working dimensions.
/// This audit is intentionally independent of Sketch/Stroke records: raster
/// location is determined here; vector-footprint causation is a later gate.
pub fn audit_white_pixels(
    source: &RgbaImage,
    preview: &RgbaImage,
    roi: Option<AuditRoi>,
) -> Result<WhiteAudit, String> {
    let (width, height) = source.dimensions();
    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return Err("working image must be 1..=1024 pixels per side".into());
    }
    if preview.dimensions() != (width, height) {
        return Err("source and preview dimensions must match".into());
    }
    if let Some(r) = roi {
        if r.width == 0 || r.height == 0
            || r.x.checked_add(r.width).is_none_or(|v| v > width)
            || r.y.checked_add(r.height).is_none_or(|v| v > height)
        {
            return Err("ROI must have positive size and lie entirely within the working image".into());
        }
    }
    let target = darkness_map(source);
    let rendered = darkness_map(preview);
    let (w, h) = (width as usize, height as usize);
    let mut white = 0;
    let mut affected = 0;
    let mut near1 = 0;
    let mut near2 = 0;
    let mut farther = 0;
    let mut region_white = 0;
    let mut region_affected = 0;
    let mut coords = Vec::new();

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if target[i] > WHITE_LIMIT {
                continue;
            }
            white += 1;
            let in_roi = roi.map(|rect| rect.contains(x, y));
            if in_roi == Some(true) {
                region_white += 1;
            }
            if rendered[i] <= WHITE_LIMIT {
                continue;
            }
            affected += 1;
            if in_roi == Some(true) {
                region_affected += 1;
            }
            let adjacent1 = source_has_dark_near(&target, w, h, x, y, 1);
            let adjacent2 = source_has_dark_near(&target, w, h, x, y, 2);
            if adjacent1 { near1 += 1; }
            if adjacent2 { near2 += 1; } else { farther += 1; }
            if coords.len() < 128 {
                coords.push(AffectedPixel {
                    x: x as u32, y: y as u32, preview_darkness: rendered[i],
                    inside_roi: in_roi,
                    near_source_nonwhite_1px: adjacent1,
                    near_source_nonwhite_2px: adjacent2,
                });
            }
        }
    }
    Ok(WhiteAudit {
        audit_revision: "p2c-white-audit-v1",
        width, height, white_limit: WHITE_LIMIT,
        source_white_pixels: white,
        affected_white_pixels: affected,
        near_source_nonwhite_1px: near1,
        near_source_nonwhite_2px: near2,
        farther_than_2px: farther,
        roi,
        roi_source_white_pixels: roi.map(|_| region_white),
        roi_affected_white_pixels: roi.map(|_| region_affected),
        outside_roi_affected_white_pixels: roi.map(|_| affected - region_affected),
        affected_coordinates: coords,
        coordinates_truncated: affected > 128,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn white() -> RgbaImage {
        RgbaImage::from_pixel(16, 16, Rgba([255,255,255,255]))
    }

    #[test]
    fn exactly_locates_inner_and_outer_ink_without_conflating_masks() {
        let mut source = white();
        // Black columns with one protected white column in between.
        for y in 4..12 {
            for x in 4..12 {
                if x != 7 { source.put_pixel(x, y, Rgba([0,0,0,255])); }
            }
        }
        let mut preview = source.clone();
        preview.put_pixel(7, 6, Rgba([0,0,0,255])); // inner channel, 1px from target black
        preview.put_pixel(0, 0, Rgba([100,100,100,255])); // far exterior white
        let audit = audit_white_pixels(&source, &preview, Some(AuditRoi {
            x:7,y:4,width:1,height:8
        })).unwrap();
        assert_eq!(audit.source_white_pixels, 200); // 256 - 56 black source pixels
        assert_eq!(audit.affected_white_pixels, 2);
        assert_eq!(audit.roi_source_white_pixels, Some(8));
        assert_eq!(audit.roi_affected_white_pixels, Some(1));
        assert_eq!(audit.outside_roi_affected_white_pixels, Some(1));
        assert_eq!(audit.near_source_nonwhite_1px, 1);
        assert_eq!(audit.farther_than_2px, 1);
        assert_eq!(audit.affected_coordinates[0].x, 0); // stable row-major
        assert_eq!(audit.affected_coordinates[1].x, 7);
    }

    #[test]
    fn empty_and_transparent_white_images_show_no_contamination() {
        let source=white();
        let preview=RgbaImage::from_pixel(16,16,Rgba([0,0,0,0]));
        let report=audit_white_pixels(&source,&preview,None).unwrap();
        assert_eq!(report.source_white_pixels,256);
        assert_eq!(report.affected_white_pixels,0);
        assert!(report.affected_coordinates.is_empty());
        assert_eq!(report.roi_affected_white_pixels,None);
    }

    #[test]
    fn invalid_roi_and_dimension_mismatch_fail() {
        assert!(audit_white_pixels(&white(), &white(), Some(AuditRoi{
            x:13,y:13,width:4,height:1
        })).is_err());
        assert!(audit_white_pixels(&white(), &white(), Some(AuditRoi{
            x:0,y:0,width:0,height:1
        })).is_err());
        assert!(audit_white_pixels(&white(), &RgbaImage::new(15,16), None).is_err());
    }
}
