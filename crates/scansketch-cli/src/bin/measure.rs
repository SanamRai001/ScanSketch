//! P2-C v1 local evaluation command; renderer-independent, no network I/O.
//! Uses the same decode and Triangle resize policy as the native renderer.
use clap::Parser;
use image::{imageops::FilterType, DynamicImage, ImageFormat, ImageReader, Limits};
use scansketch_core::{measure_sketch, Sketch};
use std::{error::Error, fs, io, path::{Path, PathBuf}};

#[derive(Parser, Debug)]
#[command(name = "scansketch-measure", version, about = "Measure source-vs-preview tone, white ink, structure proxy and actual stroke complexity")]
struct Args {
    /// Original PNG/JPEG used to render the sketch.
    #[arg(long)]
    source: PathBuf,
    /// Actual rendered white-paper PNG.
    #[arg(long)]
    preview: PathBuf,
    /// Ordered strokes JSON exported by ScanSketch.
    #[arg(long)]
    strokes: PathBuf,
    /// EXACT same max side passed to the rendering command, 32..=1024.
    #[arg(long, default_value_t = 512)]
    max_size: u32,
    /// Optional new JSON report file. Existing files are never overwritten.
    #[arg(long)]
    report: Option<PathBuf>,
}

fn decode(path: &Path, source: bool) -> Result<DynamicImage, Box<dyn Error>> {
    if fs::metadata(path)?.len() > 16 * 1024 * 1024 {
        return Err(format!("image exceeds 16 MiB compressed limit: {}", path.display()).into());
    }
    let mut reader = ImageReader::open(path)?.with_guessed_format()?;
    if !matches!(reader.format(), Some(ImageFormat::Png | ImageFormat::Jpeg))
        || (!source && reader.format() != Some(ImageFormat::Png))
    {
        return Err(if source {
            "source must be PNG or JPEG"
        } else {
            "preview must be PNG"
        }.into());
    }
    let mut limits = Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    Ok(reader.decode()?)
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if !(32..=1024).contains(&args.max_size) {
        return Err("--max-size must be within 32..=1024".into());
    }
    if let Some(report) = &args.report {
        if !report.extension().and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        {
            return Err("--report must have a .json extension".into());
        }
        if report.exists() {
            return Err("report already exists; use another filename to preserve earlier experiments".into());
        }
        for input in [&args.source, &args.preview, &args.strokes] {
            if report == input {
                return Err("--report cannot point to an input file".into());
            }
        }
    }

    let source = decode(&args.source, true)?;
    // Mirror scansketch/src/main.rs preprocessing EXACTLY. Triangle resize is
    // used only when a dimension exceeds max_size; aspect ratio is preserved.
    let working = if source.width() > args.max_size || source.height() > args.max_size {
        source.resize(args.max_size, args.max_size, FilterType::Triangle)
    } else {
        source
    }.to_rgba8();
    let preview = decode(&args.preview, false)?.to_rgba8();

    if fs::metadata(&args.strokes)?.len() > 64 * 1024 * 1024 {
        return Err("stroke JSON exceeds the 64 MiB research-input limit".into());
    }
    let stroke_bytes = fs::read(&args.strokes)?;
    let sketch: Sketch = serde_json::from_slice(&stroke_bytes)?;
    let report = measure_sketch(&working, &preview, &sketch).map_err(io::Error::other)?;
    let json = serde_json::to_string_pretty(&report)?;
    if let Some(output) = args.report {
        fs::write(&output, json.as_bytes())?;
        eprintln!("Saved metrics: {}", output.display());
    }
    println!("{json}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ScanSketch measurement error: {error}");
        std::process::exit(1);
    }
}
