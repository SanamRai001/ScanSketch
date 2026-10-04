# Project State

Last updated: **2026-10-04**. This is the single authoritative progress record; actual Git history, CI and observed outputs take precedence over descriptions.

## Goal

Produce convincing image-to-sketch results using real, ordered pencil strokes on white paper. Keep a top-to-bottom tonal sampling identity and evaluate algorithmic improvements against the same input/seed. No grayscale pixel painting or assumed superiority from adding complexity.

## Repository / stacked review

- Repository: https://github.com/SanamRai001/ScanSketch
- `main`: initial bootstrap only, no merges performed.
- `docs/foundation`: draft PR #1.
- `feat/p1-rust-scanline-baseline`: P1 baseline, draft PR #2 into foundation; 11/11 core tests and workspace check passed locally.
- `feat/p2-sketch-stroke-language`: P2-A, draft PR #3 into P1. GitHub Actions workspace check and **14/14 tests passed**. Real portrait visually compared: fragments reduced continuous barcode-like horizontal bars but eyes/glasses/lips and hair still lack clear structure.
- **Active:** `feat/p2b-contour-reinforcement`: P2-B, branching from P2-A. Keep each earlier branch unchanged as comparison baseline; PR targets P2-A.

## P2-B — Optional structural contour accents

Implementation:
- Normalized Sobel gradient in existing linear-light darkness domain. The response detects image gradients, **not** eyes, glasses or named facial features semantically.
- Thin candidates via directional non-maximum suppression and skip sites near accepted contour marks.
- Place sparse short strokes tangent to gradients, with their centers nudged to the source's dark side.
- Check source support along the stroke and its lateral edges. Avoid indiscriminate hard outlines and white-highlight contamination; cap line count and enforce existing global stroke budget.
- Contour RNG separate from tonal P2-A generator, so `--no-contours` preserves exactly the old tonal stroke prefix for fair A/B comparison.
- Core `SketchOptions` adds `enable_contours` (default true) and finite `contour_threshold` (default 0.28, 0..=1). CLI adds `--no-contours` and `--contour-threshold`.
- Additional tests cover flat input, gradient direction, comparison toggle/determinism, white-gap protection, invalid settings and shared stroke budget.
- **GitHub Actions validation passed:** `cargo check --workspace`, `cargo test --workspace` (**20/20 tests**), run https://github.com/SanamRai001/ScanSketch/actions/runs/37216135188.
- No new dependencies, vector schema changes, external models, Primitive optimization, AI or erasing.

**Verification gate:** P2-B CI compilation/tests passed; the same-input no-contours/contours portrait comparison is **still pending**. Do not call this a visual improvement until those output images are inspected. An optional pass is only retained if it visibly increases recognizability without dirtying highlights or making cartoon outlines.

## Next

- Run `docs/P2B_VERIFY.md` on the same original portrait using the two modes, preserving all P1/P2-A images.
- Fix actual code/test failures and tune contour threshold if feature enhancement is too weak or hair is too dense. Assess a wider fixture set (blank/transparent, portrait, object, landscape, high-contrast geometry).
- Then P2-C: quantitative metrics with controlled budgets, fixture provenance, runtime and memory. Later P3: Primitive-inspired proposal optimization.
- Keep natural erasure and graphite residue deferred in `docs/FUTURE_EXPERIMENTS.md`.
- Locally generated `Cargo.lock` still needs to be committed after review; do not overwrite or delete local sample.jpg or stash.

## Risk notes

Sobel responds to all source gradients, including hair texture and compression/noise; there is no semantic portrait interpretation. The finite contour cap is a safeguard, not a quality objective, and current acceptance order is top-to-bottom. Source-support sampling is conservative but raster antialiasing near one-pixel highlights still requires visual inspection. More strokes can exceed small explicit budgets rather than silently truncating user-requested detail.
