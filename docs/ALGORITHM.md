# Adaptive Scanline Stroke Reconstruction (Proposed)

**Status:** forward-looking algorithm/research specification. P1–P2-B.1 already implement source darkness, broken tonal strokes and optional source-derived contours, with passing test suites and qualitative portrait trials. The candidate optimizer, multiscale direction field, analytic opacity search and scored refinement in this document remain **unimplemented**. For the next prioritized proposal, see [P3_DIRECTIONAL_DESIGN.md](P3_DIRECTIONAL_DESIGN.md).

## 1. Inputs, output and representation

Input: raster RGB/RGBA image with bounded size. Normalize orientation and transparency against a documented matte (initially white). Work on a constrained analysis resolution while retaining coordinate mapping to the original.

Output: ordered, editable stroke records over white paper. A raster preview is derived from those records; an eventual SVG export serializes their paths and styling. Do not treat gray-painted pixels as source-of-truth geometry.

First stroke family: short mostly-horizontal paths; begin with simple line segments, then evaluate quadratic curves only after the baseline is measured. Each stroke may have an ordered list of control points, width, opacity/pressure, placement, and random seed provenance.

## 2. Source analysis

Convert sRGB channels to linear-light components before computing relative luminance:

- C_linear = C_srgb / 12.92 when C_srgb <= 0.04045.
- Otherwise C_linear = ((C_srgb + 0.055) / 1.055) ^ 2.4.
- L = 0.2126 R_linear + 0.7152 G_linear + 0.0722 B_linear.
- Desired darkness D = 1 - L, clamped to [0,1].

This is a defensible starting target; test whether perceptual tone remapping is needed for pleasing sketches. Normalize color/alpha carefully; a white background should not accidentally produce gray marks.

Calculate an edge-strength map (Sobel baseline; Canny optional later) and, if justified, local gradient orientation and tangent. Minor denoising must not eliminate important contours. Clearly separate source structure from ink already rendered.

## 3. Core scanline pass

Visit sampling bands by ascending y. The sweep is a *candidate-placement strategy*, not the geometric rasterizer's internal scanline algorithm.

For each band:
1. Partition the band into small regions based on local darkness, brightness variance and candidate eligibility.
2. Skip protected highlights and empty residual areas; do not blindly mark every nonwhite pixel.
3. Propose multiple bounded short strokes, initially mostly horizontal.
4. Constrain stroke coverage to the band or a small neighborhood (limited bleed is possible).
5. Compute each candidate's local score; mutate promising candidates; accept only beneficial ones.
6. Record accepted strokes in order, and advance y.

A fixed seed and stable processing order are required for reproducibility. Avoid global random full-image candidates that undermine the top-to-bottom concept.

## 4. Ink model

Let d_i in [0,1] be current darkness at pixel i, m_i in [0,1] rasterized candidate coverage, and a in [0,1] candidate opacity. For ideal black ink:

    d'_i = d_i + a * m_i * (1 - d_i)

This compositing rule is monotonic: black ink cannot brighten an already dark region. Prevent over-darkening through candidate scoring and caps; an eraser/correction model is deliberately NOT in the current implementation scope.

For a fixed stroke coverage mask, an analytic initial opacity estimate under weighted squared tone error is:

    a* = clamp01( sum_i w_i m_i (1-d_i) (D_i-d_i)
                 / sum_i w_i [m_i (1-d_i)]^2 )

When the denominator is zero, discard the candidate. Evaluate the full objective afterward, including structure, highlight penalties and complexity.

## 5. Scoring objective

Start with a documented, reproducible scalar score, minimizing:

    E = E_tone + lambda_w E_white + lambda_s E_structure + lambda_c E_complexity

- E_tone: weighted difference between reconstructed and target darkness.
- E_white: extra penalty for ink in protected near-white target areas.
- E_structure: preserve important boundaries. Initially use edge-weighted tone residual; test gradient/contour similarity separately because a nonlocal filter changes incremental-scoring requirements.
- E_complexity: penalize excessive strokes, total path length or irregularity that does not improve recognizability.

A pure image RMSE is a baseline but not the sole definition of sketch quality. Keep perceptual/style assessment separate from a pixel error metric. Report all weights and changes; do not quietly tune per demo photo.

## 6. Primitive-inspired candidate optimization

For a proposed stroke:
1. Locally rasterize only its affected bounds with enough padding for antialiasing/scoring.
2. Estimate pressure/opacity, render temporarily, and compute delta score against the current image.
3. Mutate parameters: start/end, length, width, curvature (once available), slight orientation change and pressure.
4. Keep changes that lower score; use bounded hill climbing with multiple seeded restarts.
5. Choose the best valid candidate. Commit only if score improvement exceeds a minimum useful gain.

Start with much smaller candidate budgets than Primitive's sample defaults; measure wall time, memory and quality on actual fixtures. A pure random-search and a deterministic no-search baseline must remain available for comparison.

For additive pixelwise objectives, update only affected pixels. If the scoring includes gradient/edge operators, expand the affected neighborhood by the operator footprint or recompute the appropriate score; otherwise partial scoring is incorrect.

## 7. Bounded second pass

After the strict top-to-bottom initial sweep, calculate a residual/error map and optionally revisit insufficiently represented regions for **additional** beneficial strokes, within a total stroke/time budget. Keep contour assistance opt-in until comparison proves its value. Stop on minimum improvement, exhausted candidates or explicit limits.

No intentionally erroneous strokes, no eraser and no residual graphite in the core algorithm. The future possibility is documented separately.

## 8. Conceptual pseudocode

    target = analyze(source)
    canvas = white_canvas()
    strokes = []
    rng = seeded_random(seed)

    for band in top_to_bottom_bands(target):
        for region in eligible_regions(band, target, canvas):
            candidate_set = propose_strokes(region, rng, limits)
            winner = optimize_and_score(candidate_set, target, canvas)
            if winner and gain(winner) >= min_gain and respects_highlights(winner):
                apply_locally(canvas, winner)
                strokes.append(winner)

    if enable_refinement:
        for region in prioritized_residuals(target, canvas, limits):
            candidate = optimize_local_additive_stroke(region)
            if candidate and gain(candidate) >= min_gain:
                apply_locally(canvas, candidate)
                strokes.append(candidate)

    return strokes, render_preview(strokes)

## 9. Known failure modes and guards

- Mechanical horizontal stripes: prefer short strokes and evaluate controlled segmentation, but preserve the baseline for comparison.
- Source highlights unintentionally dirtied: higher penalties and a strict near-white threshold.
- Monotonic ink accumulation creates unrecoverable dark regions: conservative pressure and commit thresholds; erasing deferred.
- Edge lines overpower tone: compare against tone-only outputs, do not assume improvement.
- Optimization targets numeric fidelity over artistry: collect blinded subjective sketch-quality rankings.
- Nonlocal objective mis-scored as partial: padded update or correct global evaluation.
- Excess SVG paths/browser slowdown: define explicit budgets, cancellation and memory limits.
- Pixel-to-path scaling mismatch: verify raster/SVG parity before announcing vector export.
