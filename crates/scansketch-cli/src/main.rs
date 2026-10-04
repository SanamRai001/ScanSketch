use clap::Parser;
use image::{imageops::FilterType, ImageFormat, ImageReader, Limits};
use scansketch_core::{generate_sketch, render_sketch, SketchOptions};
use std::{error::Error, fs, io, path::PathBuf};

/// A deterministic, native-first scanline pencil reconstruction prototype.
#[derive(Parser, Debug)]
#[command(name = "scansketch", version, about)]
struct Args {
    /// Input PNG/JPEG path (local files only).
    #[arg(short, long)]
    input: PathBuf,
    /// Output PNG path. The output directory must already exist.
    #[arg(short, long)]
    output: PathBuf,
    /// Optional JSON output of the actual, ordered editable stroke records.
    #[arg(long)]
    strokes: Option<PathBuf>,
    /// Maximum working size along either axis, 32..=1024.
    #[arg(long, default_value_t = 768)]
    max_size: u32,
    /// Deterministic random seed.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Horizontal band height in pixels, 1..=16.
    #[arg(long, default_value_t = 3)]
    band_height: u32,
    /// Local sampling segment width in pixels, 2..=64.
    #[arg(long, default_value_t = 8)]
    segment_width: u32,
    /// Skip segments with average darkness at or below this value, 0..=0.5.
    #[arg(long, default_value_t = 0.08)]
    white_threshold: f32,
    /// Maximum accepted strokes, 1..=500000.
    #[arg(long, default_value_t = 100_000)]
    max_strokes: usize,
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if !(32..=1024).contains(&args.max_size) {
        return Err("--max-size must be within 32..=1024".into());
    }
    if !args.output.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("png")) {
        return Err("--output must point to a .png file".into());
    }
    if let Some(ref json) = args.strokes {
        if !json.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("json")) {
            return Err("--strokes must point to a .json file".into());
        }
        if json == &args.output {
            return Err("stroke JSON and PNG output paths must differ".into());
        }
    }
    // A compressed-size guard supplements (but does not replace) decoder limits.
    if fs::metadata(&args.input)?.len() > 16 * 1024 * 1024 {
        return Err("input exceeds the 16 MiB compressed-file limit".into());
    }
    let mut reader = ImageReader::open(&args.input)?.with_guessed_format()?;
    if !matches!(reader.format(), Some(ImageFormat::Png | ImageFormat::Jpeg)) {
        return Err("only PNG and JPEG input are supported in P1".into());
    }
    let mut limits = Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let source = reader.decode()?;

    let working = if source.width() > args.max_size || source.height() > args.max_size {
        source.resize(args.max_size, args.max_size, FilterType::Triangle)
    } else {
        source
    };
    let options = SketchOptions {
        seed: args.seed,
        band_height: args.band_height,
        segment_width: args.segment_width,
        white_threshold: args.white_threshold,
        max_strokes: args.max_strokes,
    };
    let sketch = generate_sketch(&working.to_rgba8(), &options)
        .map_err(io::Error::other)?;
    let preview = render_sketch(&sketch).map_err(io::Error::other)?;
    preview.save_png(&args.output)?;
    if let Some(path) = args.strokes {
        fs::write(path, serde_json::to_vec_pretty(&sketch)?)?;
    }
    println!(
        "ScanSketch P1: {}x{} | {} strokes | seed {} | saved {}",
        sketch.width, sketch.height, sketch.strokes.len(), sketch.seed, args.output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ScanSketch error: {error}");
        std::process::exit(1);
    }
}
