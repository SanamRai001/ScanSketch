use image::RgbaImage;

/// Target darkness in linear light, including alpha compositing over white.
///
/// This is only an initial objective for later experiments; it is not a claim
/// that linear luminance alone is perceptually optimal for sketch artwork.
pub fn darkness_map(source: &RgbaImage) -> Vec<f32> {
    source
        .pixels()
        .map(|pixel| {
            let alpha = pixel[3] as f32 / 255.0;
            let r = over_white(pixel[0], alpha);
            let g = over_white(pixel[1], alpha);
            let b = over_white(pixel[2], alpha);
            (1.0 - (0.2126 * r + 0.7152 * g + 0.0722 * b)).clamp(0.0, 1.0)
        })
        .collect()
}

fn over_white(channel: u8, alpha: f32) -> f32 {
    let v = channel as f32 / 255.0;
    let linear = if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    };
    alpha * linear + (1.0 - alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn luminance_handles_black_white_and_transparency() {
        let mut img = RgbaImage::new(3, 1);
        img.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        img.put_pixel(1, 0, Rgba([0, 0, 0, 255]));
        img.put_pixel(2, 0, Rgba([0, 0, 0, 0]));
        let d = darkness_map(&img);
        assert!(d[0].abs() < 0.00001);
        assert!((d[1] - 1.0).abs() < 0.00001);
        assert!(d[2].abs() < 0.00001);
    }
}
