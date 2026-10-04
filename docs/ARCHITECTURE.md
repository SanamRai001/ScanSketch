# Preliminary Architecture

**No architecture has been implemented yet.** These are intended boundaries, to be revisited after the P1 baseline.

## Recommended prototype stack

- TypeScript for deterministic engine logic and accessible integration with a Vite browser prototype.
- Canvas 2D for initial raster preview and image sampling; avoid WebGL until profiling demonstrates a need.
- Web Worker for long-running analysis/optimization after an initial synchronous proof.
- Typed, renderer-independent stroke records. SVG serialization is a later adapter, not the core engine.
- No backend, database, accounts, GPU requirement, or paid external service for the initial prototype.

Go remains a viable later option for batch optimization (Primitive is a useful case study), but adopting Go now would add a second implementation language before we have baseline evidence.

## Boundaries (proposed)

    image input -> normalize/analyze -> image maps
                                     -> scanline candidate generator
                                     -> stroke optimizer/scorer
                                     -> stroke collection (source of truth)
                                     -> Canvas preview
                                     -> future SVG/PNG exports

Suggested modules when code begins:
- image/normalize: size, alpha/matte, luminance and optional denoising.
- analysis/maps: darkness, edge strength, optional orientation.
- strokes/model: typed paths, pressure and constraints; no UI types.
- scanline/generator: bands, eligibility, seeded proposals.
- scoring/objective: full and correct local scoring, metrics.
- optimization/search: baseline, random candidate search, bounded hill climb.
- render/canvas: compositing and preview from recorded strokes.
- export: later PNG and SVG adapters with parity tests.
- ui: minimal upload, comparison, run/cancel, simple controls.

This is a module map, **not** an instruction to create empty files upfront.

## Data and invariants

- Coordinate system, image scale and white matte must be explicit.
- Stroke records must reproduce the raster preview when replayed in order.
- A deterministic seed must reproduce candidates and stroke output under a pinned implementation.
- Candidate evaluation must not mutate the committed canvas.
- Global and local scoring must agree for the same objective, within a defined numeric tolerance.
- Do not mutate original uploaded image bytes; keep temporary data local by default.
- Bound dimensions, iteration counts, allocation sizes, runtime and SVG output size.
- Stop/cancel must leave the last committed drawing valid.

## UX constraints

Begin with a compact input/result comparison, Run/Cancel, honest progress indication, and very few controls (e.g. detail vs speed). Advanced debug maps and stroke statistics belong behind an optional panel. Maintain keyboard and screen-reader usability and respect reduced-motion settings if the drawing is animated.

## Dependency policy

Prefer modest, well-maintained packages with compatible licenses. Reimplement only the small portions of Primitive that are actually required or import with preserved MIT notice. Do not automatically vendor its Go project into a TypeScript experiment.
