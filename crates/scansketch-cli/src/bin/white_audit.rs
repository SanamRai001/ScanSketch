//! Read-only pixel-level diagnostic for the p2c-v1 white-source mask.
//! No changes to generation, rendering or the original measurement format.
use clap::Parser;
use image::{imageops::FilterType, DynamicImage, ImageFormat, ImageReader, Limits};
use scansketch_core::{audit_white_pixels, AuditRoi};
use std::{error::Error, fs, io, path::{Path, PathBuf}};

fn roi_from_text(text: &str) -> Result<AuditRoi, String> {
    let values: Vec<u32> = text.split(',').map(|v| v.trim().parse::<u32>()
        .map_err(|_| "ROI must be four comma-separated nonnegative integers: x,y,width,height".to_string()))
        .collect::<Result<_,_>>()?;
    if values.len()!=4 { return Err("ROI must be x,y,width,height".into()); }
    Ok(AuditRoi { x:values[0], y:values[1], width:values[2], height:values[3] })
}

#[derive(Debug, Parser)]
#[command(name="scansketch-white-audit", version,
    about="Locate preview ink on source-white pixels; optionally count it inside a known rectangular region")]
struct Args {
    /// PNG/JPEG original source.
    #[arg(long)]
    source: PathBuf,
    /// Actual output PNG. Never use a screenshot or a preview resized by a photo viewer.
    #[arg(long)]
    preview: PathBuf,
    /// Same 32..=1024 working-image max side used for the render.
    #[arg(long, default_value_t=512)]
    max_size: u32,
    /// Optional source-image region: x,y,width,height (0-based; synthetic channel is 29,8,6,48).
    #[arg(long, value_parser=roi_from_text)]
    roi: Option<AuditRoi>,
    /// Optional new JSON audit filename (must not exist).
    #[arg(long)]
    report: Option<PathBuf>,
}

fn decode(path: &Path, source:bool) -> Result<DynamicImage,Box<dyn Error>> {
    if fs::metadata(path)?.len() > 16*1024*1024 {
        return Err(format!("image exceeds 16 MiB compressed input limit: {}",path.display()).into());
    }
    let mut reader=ImageReader::open(path)?.with_guessed_format()?;
    if !matches!(reader.format(),Some(ImageFormat::Png|ImageFormat::Jpeg))
        || (!source && reader.format()!=Some(ImageFormat::Png)) {
        return Err("source must be PNG/JPEG and preview must be PNG".into());
    }
    let mut limits=Limits::default();
    limits.max_image_width=Some(4096);
    limits.max_image_height=Some(4096);
    limits.max_alloc=Some(128*1024*1024);
    reader.limits(limits);
    Ok(reader.decode()?)
}

fn run()->Result<(),Box<dyn Error>> {
    let args=Args::parse();
    if !(32..=1024).contains(&args.max_size) {
        return Err("--max-size must be 32..=1024".into());
    }
    let source=decode(&args.source,true)?;
    let working=if source.width()>args.max_size || source.height()>args.max_size {
        source.resize(args.max_size,args.max_size,FilterType::Triangle)
    } else { source }.to_rgba8();
    let preview=decode(&args.preview,false)?.to_rgba8();
    let audit=audit_white_pixels(&working,&preview,args.roi).map_err(io::Error::other)?;
    let json=serde_json::to_string_pretty(&audit)?;
    if let Some(file)=args.report {
        if file.extension().and_then(|e|e.to_str()).is_none_or(|e|!e.eq_ignore_ascii_case("json")) {
            return Err("--report must end in .json".into());
        }
        // Atomic create-new refusal prevents silently overwriting prior runs.
        let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(&file)?;
        use std::io::Write;
        f.write_all(json.as_bytes())?;
        eprintln!("Saved white-region audit: {}",file.display());
    }
    println!("{json}");
    Ok(())
}

fn main() {
    if let Err(err)=run() {
        eprintln!("ScanSketch white-region audit error: {err}");
        std::process::exit(1);
    }
}
