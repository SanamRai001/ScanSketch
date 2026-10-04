# Project State

Last updated: **2026-10-04**. This is the **single authoritative** project progress record. The Git tree and actual verification take precedence over prose.

## Goal

Create ScanSketch: a real line/stroke-based image reconstruction engine with a top-to-bottom sampling identity. Later evaluate Primitive-inspired candidate optimization against a deterministic baseline. Do not mistake grayscale pixel painting, halftone, or filled polygons for the intended outcome.

## Repository and branches

- Repository: https://github.com/SanamRai001/ScanSketch
- `main`: initial bootstrap README commit `0e2c9100e36a6f5021640867a70a52434ea00cb0`.
- `docs/foundation`: research docs and MIT foundation at `a396bcf5cae43e9de11f05f372ee63b21802a4b1`; draft PR #1, awaiting review.
- Active feature branch: `feat/p1-rust-scanline-baseline`, based on the foundation branch.
- P1 Rust source commit: `ca0475b8fd2ae49f43c6a2e23c7bb72a706cbc39`.
- No merge, deployment or deletion performed.

## Current phase: P1 — Native Rust deterministic baseline (implementation committed; verification pending)

### Decisions

- Accepted stack: **Rust core + native CLI first**, `tiny-skia` and `image`; WASM/React only after the core is good enough. See ARCHITECTURE.md.
- No backend, database, GPU, cloud processing or mandatory Python.
- Graphite output is actual ordered stroke records on white paper; PNG is a rendering.
- Retain the original top-to-bottom scanline principle. P1 does not implement optimization, contour following, erasing or SVG.
- Erasure with optional graphite residue remains recorded in FUTURE_EXPERIMENTS.md only.

### Implemented on P1 feature branch

- Cargo workspace with `scansketch-core` and `scansketch-cli`.
- Linear-light darkness map with transparent pixels composited over white.
- Seeded deterministic, bounded horizontal-band sampling and short pencil segments.
- Light-area suppression and deeper-shadow second stroke.
- Renderer-independent serialized stroke model; tiny-skia white-paper replay.
- Local PNG/JPEG CLI, bounded decode/working image, output PNG, optional stroke JSON.
- Synthetic Rust tests for white input, transparent input, black marks, aligned half-white input, fixed-seed equality, stroke-budget rejection, invalid options/dimensions and bounds.

### Verification and limitations

- GitHub commit/ref and source presence can be inspected remotely; **this environment does not have rustc/cargo**.
- `cargo fmt`, `cargo check`, `cargo test` and an actual image run have **not** been performed. Do not claim the algorithm passes or produces aesthetically good images.
- `Cargo.lock` not yet generated: resolve and check in after first successful local Cargo build.
- P1 is intentionally rudimentary and may look striped; its purpose is to establish a reproducible baseline before adding advanced optimization.

## Risks

- Mechanical striped output, loss of thin objects, tonal underrepresentation and sampling across boundaries.
- Resizing a semitransparent image before compositing may create color fringes; test fixture before browser production use.
- Pixel fidelity and human preference are different measures; need representative licensed fixtures.
- Image decoder allocation caps are partly best effort; also bound compressed input, decoded dimensions and working image.
- Native-to-WASM portability, package MSRVs and vector/raster parity require later checks.

## Immediate next step (user-run verification gate)

Run the exact commands in **docs/P1_VERIFY.md** on Windows, share the CLI summary, compile/test failures if any, and a rendered result. Inspect whether the image is recognizable as a sketch. Fix correctness problems before calling P1 complete. Do not progress to P2 or optimize blindly.
