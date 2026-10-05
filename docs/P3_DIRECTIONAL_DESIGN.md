# P3 Research Decision — Multi-scale, Direction-aware Strokes First

**Status: P3-A.0 (`--directional`) was implemented and rejected. P3-A.1 now implements the first independent placement/budget prototype behind `--placement-aware`; CI and real-image quality review remain pending. P2-C's genuine nonportrait gate remains open.** P2-C is a prerequisite for claims that P3 improves rendering. This proposal extends the research specification in [ALGORITHM.md](ALGORITHM.md), while retaining Primitive-inspired search as a later step rather than abandoning it.

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


## P3-A.0 result and P3-A.1 design consequence

P3-A.0 answered one narrow question: *is rotating a subset of already-generated horizontal tonal fragments enough?* On the first controlled portrait, **no**. The paired run held accepted stroke count at 12,671 and total path length effectively constant, but the directional version worsened overall, midtone and dark RMSE as well as edge F1; visual hatching remained predominantly horizontal. White RMSE improved slightly.

Therefore **do not iterate P3-A.0 by simply increasing the rotation quota or relaxing confidence/source-support thresholds**. The next P3-A.1 hypothesis should change proposal origin and allocation:

1. derive coarse/fine orientation and confidence fields as before;
2. identify eligible source-dark regions directly, rather than inheriting every P2-A horizontal anchor;
3. allocate explicit bounded budgets for coarse structure, medium/form tone and fine texture;
4. generate a new short segment at each accepted anchor, already tangent to reliable structure; fall back to P2-A only in ambiguous regions;
5. keep protected-white footprint checks on unsmoothed source;
6. preserve deterministic top-to-bottom commit order by sorting accepted proposal anchors by `(y, x, proposal_class)`;
7. compare against **frozen P2-B.1**, recording count, path length and p2c-v1 metrics, and reject if it only changes proxies without improving the preview.

P3-B search/optimization remains gated until P3-A.1 demonstrates a genuine visual win on at least a permitted portrait and a true nonportrait image.


### Activation result: why P3-A.1 should change placement, not thresholds

The portrait run reports **581 changed geometries out of 12,671 strokes**. P3-A.0 therefore exercised the direction field on a nontrivial subset of the image. Since the same run still worsened tone/midtone/dark RMSE and edge F1 and showed no clear visual gain, simply increasing the rotation quota or lowering confidence thresholds is not the preferred next experiment.

P3-A.1 should instead test whether the **anchor distribution itself** is the bottleneck: sample/allocate candidate anchors from source structure and tone regions first, then choose orientation, rather than generate horizontal scanline fragments first and rotate a minority afterward.


## Implemented P3-A.1 slice: placement before direction

The P3-A.1 branch now implements the design consequence of the P3-A.0 failure:

1. call the frozen P2-A tone pass only to obtain a target count for controlled comparison;
2. scan 5×3 source cells and choose anchors from actual source-dark pixels nearest a weighted darkness centroid;
3. use deterministic cell-local RNG for jitter/length/pressure so filtering one candidate does not perturb unrelated cells;
4. choose coarse/fine source-tangent direction when confidence is sufficient; ambiguous proposals receive only a small seeded tonal-angle prior;
5. add an offset second proposal in genuinely dark cells;
6. apply whole-stroke source-support checks against the unsmoothed darkness map;
7. select explicit initial budgets (24% coarse, 46% fine/form, remainder tonal), then fill unused quota from remaining highest-scored source-driven candidates;
8. use exact historical tonal strokes only as a final, counted sparse-input fallback;
9. restore deterministic top-to-bottom commit ordering;
10. append the unchanged P2-B.1 contour pass.

This keeps the **total accepted stroke count** comparable to frozen P2-B.1 while allowing path length, raster coverage and tonal distribution to change. Those differences must be measured, not assumed equal. See [P3-A.1 verification](P3A1_VERIFY.md).


## P3-A.1 result: direction cannot carry the tonal body

P3-A.1 deliberately changed anchor placement across the main tonal field. On the first portrait it kept total stroke count fixed at 12,671 while strongly nonhorizontal strokes increased **97→2367**. The geometry change was therefore substantial. Yet overall tone RMSE worsened **33.13%**, dark RMSE **34.44%**, midtone **6.49%**, and total path length increased **3.06%**. White RMSE improved slightly, but the picture lost broad tonal mass and became too structure/contour heavy.

**Conclusion:** source-driven direction is useful information, but it should not replace the tonal reconstruction wholesale.

### P3-A.2 design consequence

The next prototype should be **hybrid augmentation**:

1. preserve the frozen P2-A/P2-B.1 tonal body exactly;
2. determine the existing P2-B.1 structural budget (number of contour accents) for the same source;
3. build a stronger pool of multiscale source-structure candidates;
4. rank those candidates using structural confidence **and positive tonal residual after the tonal base is rendered**;
5. spend at most the existing structural budget on those candidates, with tile/regional quotas so hair/texture cannot dominate;
6. if too few qualified hybrid candidates exist, fill the remaining budget with the original P2-B.1 contour strokes, preserving the total comparison count;
7. keep strict unsmoothed source-support and protected-white checks;
8. retain deterministic ordering and expose hybrid/fallback counts;
9. measure tone, dark/midtone, edge proxy, white contamination, total path length and human visual readability;
10. reject the hybrid if it only raises edge F1 while visibly harming tonal mass.

This architecture isolates the next question cleanly: **can smarter structural accents improve a proven tonal base without asking structure to reconstruct the entire image?**


## P3-A.2 architecture freeze: structural accents, not tonal replacement

P3-A.2 keeps the P2 tonal prefix untouched. It uses the existing P2-B.1 contour count as an exact structural budget and attempts to replace those generic contour accents with multiscale source-tangent candidates **only where the rendered tonal base still has positive darkness residual**. Any unused budget falls back to the original P2-B.1 contours. This tests whether smarter structure can improve a proven tonal body at the same total stroke count. Full specification: [P3A2_ARCHITECTURE.md](P3A2_ARCHITECTURE.md).


## P3-A.2 portrait consequence: selective replacement

The hybrid architecture avoided P3-A.1's tonal collapse, but the first portrait selected 108/108 hybrid structural strokes and 0 original contour fallback. Because 2,024 valid candidates competed for only 108 slots, the policy effectively guaranteed full replacement. The resulting image was almost indistinguishable from P2-B.1 and did not improve edge F1 or midtone RMSE.

P3-A.2.1 should score the **baseline structural layer itself** under the same residual/structure evidence. A hybrid candidate should replace baseline structure only when it clears a baseline-derived utility threshold by a margin, and the experiment should impose a conservative maximum replacement fraction. This keeps the useful hybrid architecture while making the intervention genuinely selective.


## P3-A.2.1 selective replacement rule

Because the first hybrid selected 108/108 new structural strokes, the next test directly scores the baseline structural layer. P3-A.2.1 uses one comparable residual/structure/alignment utility for original P2-B.1 contours and hybrid candidates, replacing only the weakest baseline slots when the hybrid exceeds them by a relative + absolute margin. Replacements are capped at 40% and occur in-place in the structural tail. Full specification: [P3A21_ARCHITECTURE.md](P3A21_ARCHITECTURE.md).


## Implemented P3-A.2.1 slice

P3-A.2.1 now compares baseline P2-B.1 contour utility directly against hybrid candidate utility. Candidate utility and baseline utility use the same positive-residual, source-darkness and tensor-alignment terms. Strong hybrids replace weakest baseline structural slots only when they clear the frozen relative+absolute margin, with a 40% hard cap. Substitution occurs at the original contour indices, preserving every retained contour's order. [Verification](P3A21_VERIFY.md).


## P3-A.2.1 result: safe structural improvement, wrong spatial priority

The first portrait selective run replaced 43 of 108 structural slots and retained 65 original P2-B.1 contours. At exact tonal-prefix/total-count parity it improved edge F1 by **15.22%** while keeping tone/dark slightly better and white unchanged. The visual difference is still subtle because replacements cluster mostly in the upper hair/silhouette region. This implies the current residual+tensor utility rewards *strong/easy structure*, not necessarily *missing structure*.

### P3-A.2.2 design consequence

Compute a frozen baseline structural deficit map from source and P2-B.1 preview, e.g. `max(source_edge_strength - baseline_edge_strength, 0)`. Use that missing-edge residual alongside positive tonal residual and tensor alignment when scoring both baseline and candidate structural strokes. Keep the P3-A.2.1 replacement margin/cap and exact prefix/count invariants. The goal is generic underrepresented-structure recovery, not portrait semantics.


## P3-A.2.2 missing-structure residual

P3-A.2.1's aggregate edge gain is promising, but image-difference localization shows the intervention clusters in already-strong upper hair/silhouette. P3-A.2.2 therefore computes a continuous source-minus-baseline edge-strength deficit and uses that map in the shared baseline/candidate utility. Candidate generation, replacement margin and 40% cap remain unchanged so the experiment isolates spatial value assignment. [Full architecture](P3A22_ARCHITECTURE.md).


## P3-A.2.2 result: candidate availability is the bottleneck

The first portrait missing-edge scoring run did not improve the structure proxy or visible weak details: edge F1 fell from 0.288706 to 0.285843 while tone/dark were nearly unchanged and white stayed identical. Since P3-A.2.2 deliberately reused the exact P3-A.2.1 candidate generator, this is useful causal evidence: **utility cannot select a useful structural stroke that the proposal stage never generated**.

### P3-A.2.3 design consequence

Keep the selective replacement decision rule fixed, but generate anchors from spatially separated local maxima in the missing-edge residual rather than from positive tonal residual. Use the same source tensor for tangent direction, the same strict source-support check, the same tile fairness, and the same missing-structure utility for baseline-vs-candidate comparison. This tests proposal coverage as one variable without adding semantic feature detectors or extra stroke budget.


## P3-A.2.3 proposal-coverage experiment

P3-A.2.2 suggests the selector is limited by what P3-A.2's tone-residual-driven proposal stage makes available. P3-A.2.3 therefore reuses the exact short-stroke generator but feeds **missing-edge residual** into its anchor/local-search signal. Geometry, tensor direction, spacing, tile quotas, support checks, missing-structure utility, replacement margin and cap stay fixed. This isolates candidate availability as the variable. [Full architecture](P3A23_ARCHITECTURE.md).
