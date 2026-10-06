//! Deterministic, rights-clear 64px source fixtures for validating P2-C.
use clap::Parser;
use image::{Rgba, RgbaImage};
use std::{error::Error, fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "scansketch-fixtures", version, about = "Generate shareable synthetic P2-C image fixtures")]
struct Args {
    /// New output directory. Must not already contain fixture images.
    #[arg(long)]
    output_dir: PathBuf,
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    fs::create_dir_all(&args.output_dir)?;
    const W: u32 = 64;
    const H: u32 = 64;
    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);
    let save = |name: &str, image: &RgbaImage| -> Result<(), Box<dyn Error>> {
        let path = args.output_dir.join(name);
        if path.exists() {
            return Err(format!("fixture already exists; refusing overwrite: {}", path.display()).into());
        }
        image.save(&path)?;
        println!("Created {}", path.display());
        Ok(())
    };

    save("white.png", &RgbaImage::from_pixel(W, H, white))?;
    save("transparent-black.png", &RgbaImage::from_pixel(W, H, Rgba([0, 0, 0, 0])))?;

    let mut step = RgbaImage::from_pixel(W, H, white);
    for y in 0..H {
        for x in 32..W {
            step.put_pixel(x, y, black);
        }
    }
    save("step.png", &step)?;

    let mut channel = RgbaImage::from_pixel(W, H, white);
    for y in 8..56 {
        for x in 8..56 {
            if !(29..35).contains(&x) {
                channel.put_pixel(x, y, black);
            }
        }
    }
    save("square-white-channel.png", &channel)?;

    let mut gradient = RgbaImage::from_pixel(W, H, white);
    for y in 0..H {
        for x in 0..W {
            let v = (255.0 * (1.0 - x as f32 / (W - 1) as f32)).round() as u8;
            gradient.put_pixel(x, y, Rgba([v, v, v, 255]));
        }
    }
    save("gradient.png", &gradient)?;

    let mut thin = RgbaImage::from_pixel(W, H, white);
    for y in 6..58 {
        for &x in &[8, 20, 34, 52] {
            thin.put_pixel(x, y, black);
        }
    }
    save("thin-lines.png", &thin)?;

    // P4-A.2: a large mid-gray object boundary plus a deliberately short
    // darker internal contrast feature. The outer boundary belongs to the
    // long Gesture scale; the internal feature is shorter than the P4-A.1
    // minimum and should exercise the medium Form layer.
    let mut form_detail = RgbaImage::from_pixel(W, H, white);
    for y in 8..56 {
        for x in 8..56 {
            form_detail.put_pixel(x, y, Rgba([150, 150, 150, 255]));
        }
    }
    for y in 24..40 {
        for x in 31..34 {
            form_detail.put_pixel(x, y, Rgba([20, 20, 20, 255]));
        }
    }
    save("form-detail.png", &form_detail)?;

    let manifest = args.output_dir.join("README.txt");
    if manifest.exists() {
        return Err("fixture README already exists; refusing overwrite".into());
    }
    fs::write(manifest, "ScanSketch synthetic P2-C v1 fixtures. Generated programmatically; no third-party input images. 64x64 RGBA PNG. white=blank; transparent-black=white-matted blank; step=hard half-plane; square-white-channel=black square with internal white gap; gradient=horizontal grayscale; thin-lines=separated one-pixel dark strokes; form-detail=mid-gray object with short internal darker feature for P4-A.2 medium Form paths. Use identical source bytes across branches.\n")?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ScanSketch fixture error: {error}");
        std::process::exit(1);
    }
}
