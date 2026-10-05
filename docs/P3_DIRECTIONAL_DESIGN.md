# P3 Research Decision — Multi-scale, Direction-aware Strokes First

**Status: P3-A.0 has an opt-in first prototype (`--directional`) on a separate feature branch; the more ambitious independent placement/budget architecture below is still *proposed*, not implemented. P2-C's genuine nonportrait gate remains open.** P2-C is a prerequisite for claims that P3 improves rendering. This proposal extends the research specification in [ALGORITHM.md](ALGORITHM.md), while retaining Primitive-inspired search as a later step rather than abandoning it.

## Why not another raw Sobel threshold?

P2-A corrected uniform bars by giving tone a less mechanical vocabulary. P2-B and P2-B.1 add source-derived contours, but the first portrait still lacks important readable shape, while hair texture takes much of the visual attention. More opacity on isolated edge marks is unlikely to change the fundamental way the main image is built.

**Choice:** first improve **where and in which direction the source-representative pencil strokes are proposed**; then optimize those proposals. This is a testable geometry change, unlike indefinitely tuning contour accent thresholds. It is not semantic eye/glasses detection, and must work beyond portraits.

## P3-A: multiscale source analysis and local orientation field

For each working pixel, derive source darkness `D`. Analyze two complementary spatial scales:

- **Coarse structure**: smooth source modestly (e.g. sigma around 2–4px at a 512px working side) to emphasize broad silhouette and long coherent boundaries.
- **Fine structure**: a lighter smoothing (e.g. ~0.8–1.5px) to retain thin architectural/object details. These are proposed experimental scales, not prevalidated magic constants.

At each scale, compute image derivatives `Gx, Gy`. Over a small local neighborhood, build a 2×2 **structure tensor**:

```text
J = [ smooth(Gx*Gx)  smooth(Gx*Gy) ]
    [ smooth(Gx*Gy)  smooth(Gy*Gy) ]
```

The principal tensor direction is the dominant **gradient normal**; a stroke that follows a coherent edge generally uses its **tangent** (rotate by 90 degrees). A useful confidence score is:

```text
coherence = sqrt((Jxx - Jyy)^2 + 4*Jxy^2) / (Jxx + Jyy + epsilon)
```

In flat/uncertain areas fall back to the stable P2-A short, mostly horizontal stroke prior. High-frequency chaotic regions must not get more marks merely because they have large gradient magnitude. The direction field is modulo 180 degrees: pencil line orientation does not intrinsically have a forward direction.

## P3-A proposal flow (keep top-to-bottom identity)

```text
source -> normalized darkness
       -> coarse and fine structure/orientation maps
       -> per-region confidence and protected-white mask

for band in increasing y:
    find eligible residual/tone regions
    allocate a bounded coarse / fine / tonal proposal budget
    if reliable orientation:
        propose a short line tangent to local structure
        (bounded angle, width, opacity, length and source support)
    else:
        propose P2-A-style broken tonal marks
    keep deterministic order; record proposal metadata for evaluation

return ordered stroke geometry, render with the existing tiny-skia engine
```

First implement as an **opt-in alternative to the current generator**, not a destructive replacement. Reuse `Stroke{x0,y0,x1,y1,width,opacity}`, renderer, seeded RNG, size limits and shared cap. Avoid implementing curves, face detectors, multiple raster engines or a new dependency until straight segments demonstrate value.

### Source-support requirements

- A candidate must remain inside canvas and must not cross protected bright gaps. Check endpoints, body and round-cap footprint against *unsmoothed* target support.
- Avoid strong tangents in low-coherence texture or regions with no structure; use the stable tonal fallback.
- Keep coarse outlines and fine textures on distinct, explicit budgets, so a detailed hair region cannot silently consume the entire structural pass.
- Do not assume improved pixelwise RMSE implies a better drawing.

## P3-B: Primitive-inspired local scoring (after P3-A gate)

Use candidate proposals from the field above, then evaluate **actual rendered improvement** on the current canvas. Retain Primitive's useful optimization principles: bounded candidate generation, local raster mask, scored mutation, multiple seeded restarts and deterministic commit. Do not copy the aesthetics of polygon reconstruction.

A starting objective is:

```text
E = tone_error + lambda_white * protected_white_ink
               + lambda_structure * structural_error
               + lambda_complexity * stroke_cost
```

Evaluate any image-gradient objective using a padded affected rectangle (gradient kernels have neighbors); do not treat nonlocal terms as independent per pixel. Use a fixed stroke budget and small candidate budget on a CPU-only laptop. Record the actual weights, score terms, accepted proposals and runtime. Enforce minimum improvement before committing additional black ink. Still no erasure or correction strokes.

### Suggested module boundaries, only after protocol gate

```text
scansketch-core/src/
  analysis.rs       # existing linear darkness normalization
  orientation.rs    # proposed multiscale gradients and tensor/confidence
  proposal.rs       # bounded P3-A direction-aware stroke candidates
  score.rs          # P3-B: scoring; absent during P3-A
  scanline.rs       # existing P2-A/B.1 reference implementation
  contour.rs        # current P2-B.1 reference/accent mode
  stroke.rs         # unchanged first-pass straight segment record
  render.rs         # unchanged raster replay
```

Keep a `--mode` or similar explicit switch in a later prototype; exact option names are not yet decided. Avoid a breaking stroke JSON change without a version/migration strategy.

## Decision gates / failure criteria

1. P2-C measurement tools and synthetic fixtures validated first.
2. At comparable accepted stroke/ink budgets, P3-A must improve at least one licensed portrait **and** one nonportrait image's recognizability or structure proxy without worse white-ink contamination.
3. If orientation field gets confused by hair, fabric or architecture, record failures and reduce confidence/scale rather than hardcoding a portrait-only fix.
4. Only then compare P3-B scored candidate placement to the *unoptimized directional proposal* on equal resources.
5. If neither beats P2-B.1 reliably, retain the tested baseline; stop or revisit the fundamental objective.

Open questions: preferred scale normalization across image sizes, tonal-vs-structure budget allocation, alignment in isotropic/noisy texture, visual loss proxy for salient facial anchors without introducing pretrained recognition, and interactive CPU performance on a modest laptop.

Further reading/prior art: structure-tensor orientation and multiscale image processing in classical vision; [Primitive (Michael Fogleman)](https://github.com/fogleman/primitive) for optimization mechanics, with appropriate license notice if code is incorporated. No imported third-party implementation is claimed here.

## Implemented first slice: P3-A.0 (opt-in only)

The prototype changes direction rather than location/budget: original top-to-bottom P2-A tonal proposals keep their index, center, original length, width and opacity; a source-supported subset may rotate around the same midpoint using a fine/coarse structure tensor. Hard confidence, source-ink footprint and tile/global quota guards prevent unbounded extra texture. Default `generate_sketch` and the optional P2-B.1 contour pass remain unchanged. Invoke `generate_directional_sketch` or CLI `--directional` for the experiment. This does **not** implement a new candidate optimization loop, a fully independent anchor placement scheduler or equalized raster-ink loss; future P3-A.1 requires measurement evidence first. See [P3-A.0 verification](P3A0_VERIFY.md).
