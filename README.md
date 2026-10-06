# ScanSketch

**Reconstructing images, one intelligent stroke at a time.**

ScanSketch is an early-stage, research-driven graphics project by **Sanam Rai**. Its aim is to transform raster photographs and illustrations into convincing hand-drawn-style sketches through adaptive scanline sampling, vector pencil strokes, and iterative reconstruction.

> Status: **P1–P2-B.1 native Rust experiments compile and pass their documented tests; P2-B.1 is the strongest visual candidate so far but the portrait still needs substantial structural improvement.** P2-C measurement protocol and a proposed P3 directional algorithm are documented. Nothing has been merged into `main`; no browser app or optimized candidate search has been shipped.

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

The current experimental branches implement luminance/white matting, seeded top-to-bottom fragment strokes, replayable vector records, PNG/JSON export, and an optional image-gradient contour accent pass (smoothed/coherence-ranked in P2-B.1). The current implementation is **not** semantic feature extraction, multi-scale directional tonal proposal generation, search-based candidate optimization, or erasing. See [Algorithm](docs/ALGORITHM.md), [P2 visual review](docs/P2_VISUAL_REVIEW.md), and the [P3 proposal](docs/P3_DIRECTIONAL_DESIGN.md).

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
| [Experiments](docs/EXPERIMENTS.md) | General reproducible comparison methodology |
| [Phase evolution](docs/PHASE_EVOLUTION.md) | Permanent P0→current algorithm progression, observed results, artifact references and decisions |
| [Phase result template](docs/PHASE_RESULT_TEMPLATE.md) | Standard evidence record to append for each future phase |
| [P2 visual review](docs/P2_VISUAL_REVIEW.md) | What actually happened in the P1–P2-B.1 portrait tests |
| [P2-C protocol](docs/P2C_PROTOCOL.md) | Fixed fixture, metrics, comparisons and acceptance gate |
| [P2-C runbook](docs/P2C_RUN.md) | Native fixture generator and measurement CLI usage |
| [P3 directional design](docs/P3_DIRECTIONAL_DESIGN.md) | Proposed multiscale geometry-first architecture |
| [P3-A.0 verification](docs/P3A0_VERIFY.md) | Rejected rotation-only tonal segment experiment and A/B evidence |
| [P3-A.1 verification](docs/P3A1_VERIFY.md) | Rejected placement-dominant proposal experiment and evidence |
| [P3-A.2 architecture](docs/P3A2_ARCHITECTURE.md) | Hybrid residual-aware structural reinforcement at the existing P2-B.1 budget |
| [P3-A.2 verification](docs/P3A2_VERIFY.md) | Exact-budget full-replacement hybrid evidence |
| [P3-A.2.1 architecture](docs/P3A21_ARCHITECTURE.md) | Selective baseline-contour vs hybrid utility replacement rule |
| [P3-A.2.1 verification](docs/P3A21_VERIFY.md) | Exact-prefix selective replacement A/B and first promising result |
| [P3-A.2.2 architecture](docs/P3A22_ARCHITECTURE.md) | Missing-baseline-edge residual while freezing the P3-A.2.1 candidate pool |
| [P3-A.2.2 verification](docs/P3A22_VERIFY.md) | Rejected scoring-only missing-structure A/B evidence |
| [P3-A.2.3 architecture](docs/P3A23_ARCHITECTURE.md) | Deficit-driven proposal coverage with frozen selective controls |
| [P3-A.2.3 verification](docs/P3A23_VERIFY.md) | Exact-budget deficit-proposal A/B and first promising portrait result |
| [P3-A.2.3 portrait result](docs/P3A23_FIRST_PORTRAIT_RESULT.md) | Same-count portrait metrics, visual interpretation and decision to generalize |
| [P3-G1 protocol](docs/P3G1_PROTOCOL.md) | Frozen P3-A.2.3 validation across 3–5 genuine nonportrait photos |
| [P3-G1 runbook](docs/P3G1_RUN.md) | One-command batch A/B, aggregate metrics and local visual review |
| [P3-G1 first result](docs/P3G1_FIRST_RESULT.md) | Chair/mug/plant generalization result and pivot to human stroke hierarchy |
| [P4-A stroke hierarchy](docs/P4A_HUMAN_STROKE_HIERARCHY.md) | Long gesture paths + medium form paths + short residual hatching architecture |
| [P4-A.0 verification](docs/P4A0_VERIFY.md) | Backward-compatible first-class path primitive, rendering and measurement gate |
| [P4-A.1 verification](docs/P4A1_VERIFY.md) | Source-driven 24–96px structural gesture tracing and paths-only visual gate |
| [Future experiments](docs/FUTURE_EXPERIMENTS.md) | Ideas parked until core output is convincing |
| [Project state](docs/PROJECT_STATE.md) | Single authoritative progress, risks and next phase |
| [P1 verification](docs/P1_VERIFY.md) | Historical baseline validation |
| [Contributing](CONTRIBUTING.md) | How to propose changes and provide reproducible results |

## Inspiration and attribution

ScanSketch is conceptually inspired in part by Michael Fogleman's [Primitive](https://github.com/fogleman/primitive), particularly its one-candidate-at-a-time reconstruction, random restarts, hill climbing, and partial image error scoring. ScanSketch is **not** a fork or existing implementation of Primitive. Its intended visual primitives are pencil strokes, with a top-to-bottom scanning identity and sketch-specific quality constraints.

Read [Research](docs/RESEARCH.md) before importing or adapting third-party implementation code. Preserve applicable upstream licenses and notices.

## Getting started with the current research renderer

Install current stable [Rust/Cargo](https://rustup.rs/) and Windows C++ MSVC Build Tools when prompted. The default branch still contains only the bootstrap README; the latest tested renderer lives on a feature branch:

```powershell
git clone https://github.com/SanamRai001/ScanSketch.git
cd ScanSketch
git switch feat/p2b1-contour-coherence
$env:CARGO_BUILD_JOBS = "2"
cargo check --workspace
cargo test --workspace
```

Put a locally permitted photograph at `sample.jpg` (never commit it), then:

```powershell
New-Item -ItemType Directory -Force outputs | Out-Null
cargo run -p scansketch-cli -- --input .\sample.jpg --output .\outputs\p2b1.png --strokes .\outputs\p2b1.json --seed 42 --max-size 512
cargo run -p scansketch-cli -- --input .\sample.jpg --output .\outputs\p2a-control.png --seed 42 --max-size 512 --no-contours
```

Inputs are local PNG/JPEG. The CLI enforces bounded decoding and a 1024px maximum working-image side. P2-B.1 defaults to optional coherence-ranked contours; `--no-contours` preserves the P2-A tonal stroke output. This is a **research prototype**, not a validated portrait product.

The P2-C documentation branch (`docs/p2c-measurement-and-p3-design`) contains the same renderer plus research plans. Historical **P3-A.0** (`feat/p3a-directional-tonal-prototype`, `--directional`) is preserved as a rejected rotation-only experiment. The current **P3-A.1** branch (`feat/p3a1-placement-aware-proposals`) adds opt-in `--placement-aware`, which creates new source-driven anchors before assigning direction while keeping frozen P2-B.1 as the default/control. It has no accepted quality win until CI and A/B review; see [verification](docs/P3A1_VERIFY.md). The first local Cargo build generated `Cargo.lock` on Windows; it has not yet been checked into the remote repository. Preserve it locally. Do not commit personal samples, `outputs/` or `target/`.

## License and ownership

Copyright (c) 2026 **Sanam Rai**. Released under the [MIT License](LICENSE).

The original creator retains copyright in their contributions. MIT grants others broad permissions, including modification and commercial reuse, subject to preservation of the required copyright and license notice; it does **not** prohibit proprietary derivatives. Contributors retain rights in their own contributions unless separately agreed. No trademark exclusivity is claimed by the MIT license.
