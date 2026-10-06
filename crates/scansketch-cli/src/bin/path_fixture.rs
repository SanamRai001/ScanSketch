use std::{error::Error, fs, path::PathBuf};

use clap::Parser;
use scansketch_core::{render_sketch, PathPoint, PathStroke, Sketch, StrokeRole};

#[derive(Parser, Debug)]
#[command(about = "Write a deterministic P4-A.0 logical path fixture")]
struct Args {
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    strokes: PathBuf,
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    for path in [&args.output, &args.strokes] {
        if path.exists() {
            return Err(format!(
                "output already exists; use a new path to preserve earlier experiments: {}",
                path.display()
            )
            .into());
        }
    }

    let sketch = Sketch {
        width: 64,
        height: 64,
        seed: 42,
        strokes: Vec::new(),
        paths: vec![PathStroke {
            points: vec![
                PathPoint { x: 5.0, y: 44.0 },
                PathPoint { x: 10.0, y: 32.0 },
                PathPoint { x: 17.0, y: 21.0 },
                PathPoint { x: 27.0, y: 14.0 },
                PathPoint { x: 38.0, y: 15.0 },
                PathPoint { x: 48.0, y: 24.0 },
                PathPoint { x: 57.0, y: 42.0 },
            ],
            width: 1.5,
            opacity: 0.82,
            role: StrokeRole::Gesture,
        }],
    };

    let preview = render_sketch(&sketch).map_err(std::io::Error::other)?;
    preview.save_png(&args.output)?;
    fs::write(&args.strokes, serde_json::to_vec_pretty(&sketch)?)?;

    println!(
        "P4-A.0 path fixture: {} logical mark | {} points | {:.2}px path | saved {}",
        sketch.logical_mark_count(),
        sketch.paths[0].points.len(),
        sketch.paths[0].path_length_px(),
        args.output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ScanSketch P4-A.0 fixture error: {error}");
        std::process::exit(1);
    }
}
