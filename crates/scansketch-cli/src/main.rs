use clap::Parser;
use image::{imageops::FilterType, ImageFormat, ImageReader, Limits};
use scansketch_core::{generate_directional_sketch, generate_hybrid_sketch_with_stats, generate_placement_sketch_with_stats, generate_selective_hybrid_sketch_with_stats, generate_sketch, render_sketch, SketchOptions};
use std::{error::Error, fs, io, path::PathBuf};

/// Native tonal + optionally coherence-ranked contour sketch (P2-B.1).
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
    /// Local sampling run width in pixels, 2..=64; wider regions allow fragmented marks.
    #[arg(long, default_value_t = 24)]
    segment_width: u32,
    /// Skip segments with average darkness at or below this value, 0..=0.5.
    #[arg(long, default_value_t = 0.08)]
    white_threshold: f32,
    /// Maximum accepted strokes, 1..=500000.
    #[arg(long, default_value_t = 100_000)]
    max_strokes: usize,
    /// Draw only P2-A tonal fragments (disable structural contour reinforcement).
    #[arg(long, default_value_t = false)]
    no_contours: bool,
    /// Normalized Sobel strength cutoff for optional contours, 0..=1.
    #[arg(long, default_value_t = 0.28)]
    contour_threshold: f32,
    /// Experimental P3-A: reorient eligible tonal segments using a multiscale source tensor.
    /// Default (flag absent) retains exact P2-B.1 behavior.
    #[arg(long, default_value_t = false)]
    directional: bool,
    /// Experimental P3-A.1: build new source-driven tonal anchors and directions.
    /// Mutually exclusive with --directional; default still retains P2-B.1.
    #[arg(long, default_value_t = false)]
    placement_aware: bool,
    /// Experimental P3-A.2: keep P2 tone exact, replace its small contour budget
    /// with residual-aware structural accents where supported.
    #[arg(long, default_value_t = false)]
    hybrid_structural: bool,
    /// Experimental P3-A.2.1: selectively replace only baseline contours that
    /// are clearly weaker than residual-aware hybrid alternatives.
    #[arg(long, default_value_t = false)]
    hybrid_selective: bool,
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if !(32..=1024).contains(&args.max_size) {
        return Err("--max-size must be within 32..=1024".into());
    }
    let experiment_count =
        args.directional as u8
        + args.placement_aware as u8
        + args.hybrid_structural as u8
        + args.hybrid_selective as u8;
    if experiment_count > 1 {
        return Err("--directional, --placement-aware, --hybrid-structural and --hybrid-selective are separate experiments; choose only one".into());
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
    // Never overwrite the original user image, including equivalent path spellings.
    if args.output.exists()
        && fs::canonicalize(&args.input)? == fs::canonicalize(&args.output)?
    {
        return Err("input and output resolve to the same file; choose another output path".into());
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
        enable_contours: !args.no_contours,
        contour_threshold: args.contour_threshold,
    };
    let working_rgba = working.to_rgba8();
    let (sketch, placement_stats, hybrid_stats, selective_stats) =
        if args.hybrid_selective {
            let (sketch, stats) =
                generate_selective_hybrid_sketch_with_stats(&working_rgba, &options)
                    .map_err(io::Error::other)?;
            (sketch, None, None, Some(stats))
        } else if args.hybrid_structural {
            let (sketch, stats) =
                generate_hybrid_sketch_with_stats(&working_rgba, &options)
                    .map_err(io::Error::other)?;
            (sketch, None, Some(stats), None)
        } else if args.placement_aware {
        let (sketch, stats) =
            generate_placement_sketch_with_stats(&working_rgba, &options)
                .map_err(io::Error::other)?;
        (sketch, Some(stats), None, None)
    } else if args.directional {
        (
            generate_directional_sketch(&working_rgba, &options)
                .map_err(io::Error::other)?,
            None,
            None,
            None,
        )
    } else {
        (
            generate_sketch(&working_rgba, &options).map_err(io::Error::other)?,
            None,
            None,
            None,
        )
    };
    let preview = render_sketch(&sketch).map_err(io::Error::other)?;
    preview.save_png(&args.output)?;
    if let Some(path) = args.strokes {
        fs::write(path, serde_json::to_vec_pretty(&sketch)?)?;
    }
    println!(
        "ScanSketch {}: {}x{} | {} strokes | seed {} | saved {}",
        if args.hybrid_selective {
            "P3-A.2.1 selective hybrid prototype"
        } else if args.hybrid_structural {
            "P3-A.2 hybrid structural prototype"
        } else if args.placement_aware {
            "P3-A.1 placement-aware prototype"
        } else if args.directional {
            "P3-A.0 directional prototype"
        } else {
            "P2-B.1"
        },
        sketch.width, sketch.height, sketch.strokes.len(), sketch.seed, args.output.display()
    );
    if let Some(stats) = placement_stats {
        println!(
            "P3-A.1 placement: target tone={} | coarse={} | fine={} | tonal={} | baseline-fallback={} | candidates={}",
            stats.target_tonal_strokes,
            stats.selected_coarse,
            stats.selected_fine,
            stats.selected_tonal,
            stats.baseline_fallback,
            stats.generated_candidate_count,
        );
    }
    if let Some(stats) = hybrid_stats {
        println!(
            "P3-A.2 hybrid: tone={} | structural-budget={} | hybrid-selected={} | contour-fallback={} | candidates={}",
            stats.tone_count,
            stats.structural_budget,
            stats.hybrid_selected,
            stats.baseline_contour_fallback,
            stats.generated_hybrid_candidates,
        );
    }
    if let Some(stats) = selective_stats {
        println!(
            "P3-A.2.1 selective: tone={} | structural-budget={} | max-replacements={} | replacements={} | baseline-retained={} | candidates={} | weakest-baseline={:.6} | strongest-hybrid={:.6}",
            stats.tone_count,
            stats.structural_budget,
            stats.max_replacements,
            stats.replacements_made,
            stats.baseline_contours_retained,
            stats.generated_hybrid_candidates,
            stats.weakest_baseline_utility,
            stats.strongest_hybrid_utility,
        );
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ScanSketch error: {error}");
        std::process::exit(1);
    }
}
