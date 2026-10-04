# Experiments and Verification Protocol

**Status:** plan only; no experimental results yet.

## Test categories

Use a small, permission-cleared fixed fixture set:
- All-white input (should produce no unnecessary strokes).
- Simple black shape on white (geometry and highlight preservation).
- Gradient or soft shadow (tone continuity).
- Portrait (recognizable critical structure).
- Object/product photo (material and contours).
- Building/architecture (thin lines and directional structure).
- Noise/high-frequency detail (robustness against overdraw).

Keep originals, attribution/license and fixture hashes in a future fixture manifest. Never commit personal or rights-unclear user photos. Synthetic images are appropriate for the first three cases.

## Experimental comparisons

A. Baseline top-to-bottom deterministic short-stroke renderer.
B. Same stroke family and budget plus Primitive-inspired candidate optimization.
C. B plus optional structure-aware scoring and additive refinement.

Hold input size, random seed, permitted stroke count, output resolution and target image constant. Record runtime and parameters. Repeat stochastic tests over a predetermined seed set; do not report only the nicest sample.

## Proposed measurements

- Pixelwise tone MSE/RMSE (clearly identified as approximation, not artistry).
- Extra ink coverage in near-white target areas.
- Edge/structure retention proxy, defined before testing.
- Accepted stroke count, total stroke/path length and SVG size when export exists.
- Elapsed time, peak memory where measurable, candidate evaluation count.
- Human side-by-side sketch preference using the same blinded image set.

## Required correctness checks

- All-white input preserves clean paper within tolerance.
- Uniform dark input does not crash or loop indefinitely.
- Transparent pixels follow the documented matte/alpha policy.
- Fixed seed + fixed engine version reproduce ordered paths and score.
- Candidate evaluation does not mutate the committed canvas.
- Partial and full score agree for supported local objectives; padded neighborhood is considered for edge-based metrics.
- Increasing stroke count does not silently bypass bounds.
- Graceful cancellation leaves the last committed state valid.
- Future SVG and raster preview match within documented tolerance at equivalent dimensions.

## Report template

    Experiment ID / date:
    Branch and commit:
    Fixture ID, license and hash:
    Image dimensions:
    Algorithm version and seed(s):
    Parameters and stroke budget:
    Numeric results and runtime:
    Representative output links:
    Observed strengths / failures:
    Decision: retain, reject or revise
    Next hypothesis:

Do not conflate a computed score decrease with a demonstrated improvement in visual sketch quality.
