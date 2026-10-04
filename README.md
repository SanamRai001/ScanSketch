# ScanSketch

**Reconstructing images, one intelligent stroke at a time.**

ScanSketch is an early-stage, research-driven graphics project by **Sanam Rai**. Its aim is to transform raster photographs and illustrations into convincing hand-drawn-style sketches through adaptive scanline sampling, vector pencil strokes, and iterative reconstruction.

> Status: **P1 native Rust scanline baseline committed on a feature branch; compilation and visual quality await local verification.** No browser app, optimized candidate search, or published package yet.

## The idea

A virtual pencil moves through an image from top to bottom. It reads the source's visual information, keeps bright paper mostly untouched, and proposes sketch strokes where darkness and structure need representation. Instead of mechanically converting every pixel into gray or drawing random marks, the engine evaluates proposed strokes and retains those that meaningfully improve the reconstruction.

The intended result is visibly made from **lines and strokes**, not a grayscale photograph, halftone-dot filter, or collection of filled polygons.

### Proposed pipeline

1. Normalize input and calculate target luminance, darkness, and edge maps.
2. Visit horizontal sampling bands from top to bottom.
3. Identify local regions needing graphite, skipping protected highlights.
4. Generate short pencil-stroke candidates, starting with primarily horizontal strokes.
5. Render candidate locally, estimate suitable pressure, score, and optimize promising candidates.
6. Commit beneficial strokes as editable vector records.
7. Optionally perform an additive, bounded error-guided refinement pass after the first scan.
8. Preview as a raster drawing; later export recorded paths as SVG and the canvas as PNG.

The current P1 baseline implements luminance, deterministic band sampling, short mostly-horizontal strokes, replayable vector records, and PNG preview. Candidate optimization, edge guidance and refinement remain proposals. See [Algorithm](docs/ALGORITHM.md).

## Current priorities

- A clear, recognizable result built from actual strokes.
- Preserve light/white paper and identifiable details.
- A deterministic baseline before introducing costly optimization.
- Compare visual quality, stroke count, runtime, and human preference on fixed test images.
- Keep the engine independent of the UI and store actual stroke paths.
- Build a simple and accessible browser experience only after the core result is good.

### Explicitly not the MVP

- AI image generation or invented details not present in the source.
- Erasure residue, intentionally incorrect strokes, smudges, paper simulation, or other decorative realism.
- A huge settings panel, user accounts, cloud storage, payments, or deployment infrastructure.
- Claims that the output is photorealistic, lossless, or uniquely novel.

The proposed erasure/correction idea is recorded under [Future Experiments](docs/FUTURE_EXPERIMENTS.md), **not scheduled for implementation**.

## Project documentation

| File | Purpose |
| --- | --- |
| [Vision](docs/VISION.md) | Goals, audience, constraints, MVP and non-goals |
| [Algorithm](docs/ALGORITHM.md) | Source analysis, strokes, objective, optimization, refinement |
| [Architecture](docs/ARCHITECTURE.md) | Rust workspace and proposed WASM/React integration |
| [Roadmap](docs/ROADMAP.md) | Small gated phases and acceptance conditions |
| [Research](docs/RESEARCH.md) | Prior art, inspiration, attribution and open questions |
| [Experiments](docs/EXPERIMENTS.md) | Reproducible comparison methodology |
| [Future experiments](docs/FUTURE_EXPERIMENTS.md) | Ideas parked until core output is convincing |
| [Project state](docs/PROJECT_STATE.md) | Single authoritative progress, risks and next phase |
| [P1 verification](docs/P1_VERIFY.md) | Windows commands, expected results and manual visual checks |
| [Contributing](CONTRIBUTING.md) | How to propose changes and provide reproducible results |

## Inspiration and attribution

ScanSketch is conceptually inspired in part by Michael Fogleman's [Primitive](https://github.com/fogleman/primitive), particularly its one-candidate-at-a-time reconstruction, random restarts, hill climbing, and partial image error scoring. ScanSketch is **not** a fork or existing implementation of Primitive. Its intended visual primitives are pencil strokes, with a top-to-bottom scanning identity and sketch-specific quality constraints.

Read [Research](docs/RESEARCH.md) before importing or adapting third-party implementation code. Preserve applicable upstream licenses and notices.

## Getting started

Install a current stable [Rust toolchain](https://rustup.rs/) (Cargo included). **Use the P1 feature branch**, since the foundation PR has not been merged:

```powershell
git clone https://github.com/SanamRai001/ScanSketch.git
cd ScanSketch
git switch feat/p1-rust-scanline-baseline

cargo check --workspace
cargo test -p scansketch-core
cargo run -p scansketch-cli -- --help
```

Place a photo at `sample.jpg`, then create an output directory and render:

```powershell
New-Item -ItemType Directory -Force outputs | Out-Null
cargo run -p scansketch-cli -- --input .\sample.jpg --output .\outputs\sample-sketch.png --strokes .\outputs\sample-strokes.json --seed 42 --max-size 512
```

The CLI supports **local PNG/JPEG input**, white-paper PNG output, and optional ordered stroke records in JSON. Processing is capped to a 1024px working side; default maximum is 768px. This is an experimental baseline; no quality claim is made yet.

See [P1 verification](docs/P1_VERIFY.md) for exact Windows test commands and expected output. A Rust `Cargo.lock` has not been generated in this connector environment and should be checked in following the first successful local dependency resolution.

## License and ownership

Copyright (c) 2026 **Sanam Rai**. Released under the [MIT License](LICENSE).

The original creator retains copyright in their contributions. MIT grants others broad permissions, including modification and commercial reuse, subject to preservation of the required copyright and license notice; it does **not** prohibit proprietary derivatives. Contributors retain rights in their own contributions unless separately agreed. No trademark exclusivity is claimed by the MIT license.
