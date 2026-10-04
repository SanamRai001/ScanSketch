# Architecture and Technology Decision

Last updated: 2026-10-04. **Decision accepted:** native-first Rust engine, browser adapter later.

## Actual P1 implementation

- Rust 2021 Cargo workspace.
- `scansketch-core`: UI-independent image-to-strokes and preview logic.
- `scansketch-cli`: local PNG/JPEG decode, working-image resize/limits, PNG preview and optional JSON stroke export.
- `image` for pixel data/decode; `tiny-skia` for CPU path rendering; seeded `rand_chacha` for reproducible slight stroke variation; `serde` for independent stroke data; `clap` for CLI parsing.
- No frontend, server, database, WebAssembly adapter, AI dependency or hosted service.
- Existing P1 code is committed, but Cargo compilation and actual visual quality remain **unverified** until the user runs documented Windows checks.

## Why native Rust first

ScanSketch spends effort in local sampling, path construction, future candidate mutation and affected-pixel scoring. A native reusable core offers controlled memory, good profiling opportunities and a potential WebAssembly build later. We are not claiming measurable superiority before benchmarking.

### Planned later, not implemented

- Edge analysis: `imageproc` only if needed; do not add unneeded dependencies in P1.
- Native benchmarks: Criterion after fixed fixtures exist.
- WASM adapter: `wasm-bindgen` exposing bounded core APIs, with early target-build portability checks.
- Browser processing: Web Worker with cancellation/progress and bounded memory.
- Frontend: Vite + React + TypeScript + Tailwind; Canvas preview; SVG serialization after raster/vector parity checks.
- Python/notebooks: strictly optional research scripts, not a second production engine.
- WebGL/GPU, multithreaded WASM, backend/API and storage: not justified at present.

## Actual source layout

```text
Cargo.toml
crates/
  scansketch-core/
    src/
      lib.rs        # public reconstruction/renderer API and correctness tests
      analysis.rs   # darkness/luminance, alpha compositing over white
      scanline.rs   # deterministic top-to-bottom bands and strokes
      stroke.rs     # renderer-independent serializable stroke records
      render.rs     # white-paper tiny-skia PNG preview
  scansketch-cli/
    src/main.rs     # bounded image decode, CLI flags and exports
docs/
  PROJECT_STATE.md  # sole live progress record
  P1_VERIFY.md
```

Only add new files/modules as the next measured phase demands.

## Core processing contract

```text
local PNG/JPEG -> bounded decoding -> optional aspect-preserving resize
    -> normalized RGBA -> target darkness
    -> deterministic band + short-stroke placement
    -> ordered Sketch { width, height, seed, strokes[] }
    -> CPU raster replay -> PNG, optional JSON stroke data
```

The ordered stroke collection is the authoritative drawing data. PNG is a replay, not the model. The baseline is not edge-guided or optimized and is not described as finished sketch quality.

## Invariants, data safety and limitations

- Input compressed file <= 16 MiB; decoder dimension limit 4096 per side and best-effort allocation limit 128 MiB.
- Working image <= 1024 pixels per side; CLI defaults to 768 maximum, preserving aspect ratio.
- White/transparent source regions composite against white and should produce no ink in the wholly white fixture.
- Fixed seed and same engine build must reproduce ordered stroke records.
- Band/segment/threshold/stroke budgets validated; limits are not a general image-parser sandbox.
- No uploaded image leaves the local CLI.
- Candidate scoring and erasing are absent, so improvements require measured later work rather than decorative features.
- JSON is for research/inspection; future backward compatibility and a versioned schema will need a decision.
- No Cargo.lock yet: pin via first local `cargo generate-lockfile`/Cargo build, then commit.
- For WebAssembly, avoid assuming native-only image/PNG features or parallelism will port automatically. Test the target early before investing in a browser adapter.

## UX when the engine is good enough

One image upload, source/result comparison, Render/Cancel, minimal default quality control, clearly labelled exports. Show progress honestly. Maintain keyboard accessibility and respect reduced-motion preferences.

## Attribution

Inspired by the optimization approach of [Primitive](https://github.com/fogleman/primitive), without importing its source code. Preserve original dependency notices when distributing binaries.
