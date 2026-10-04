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

    let manifest = args.output_dir.join("README.txt");
    if manifest.exists() {
        return Err("fixture README already exists; refusing overwrite".into());
    }
    fs::write(manifest, "ScanSketch synthetic P2-C v1 fixtures. Generated programmatically; no third-party input images. 64x64 RGBA PNG. white=blank; transparent-black=white-matted blank; step=hard half-plane; square-white-channel=black square with internal white gap; gradient=horizontal grayscale; thin-lines=separated one-pixel dark strokes. Use identical source bytes across branches.\n")?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ScanSketch fixture error: {error}");
        std::process::exit(1);
    }
}
