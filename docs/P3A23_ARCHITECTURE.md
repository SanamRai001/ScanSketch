# P3-A.2.3 — Deficit-Driven Candidate Proposals

**Status:** architecture frozen before implementation on `feat/p3a23-deficit-driven-proposals`; implementation now passes the engineering CI gate. P3-A.2.2 is preserved as a rejected scoring-only experiment. P2-B.1 remains the default baseline.

## Motivation

P3-A.2.2 changed only structural scoring to reward source edges missing from the frozen P2-B.1 preview. On the first portrait, edge F1 regressed ~0.99% and weak interior structure remained visually underrepresented.

That result isolates a likely bottleneck:

> **A selector cannot recover a missing structure if the candidate generator never proposes a stroke there.**

The P3-A.2/P3-A.2.1/P3-A.2.2 proposal pool is still anchored by positive tonal residual. Missing-but-important edges in lighter or already-tonally-covered regions can therefore be absent from the pool even when the later missing-edge utility would value them.

## Single changed variable

Relative to P3-A.2.2, freeze everything except the signal used to place candidate anchors.

Frozen:
- P2-B.1 baseline;
- exact P2 tonal prefix;
- exact total stroke count;
- 40% maximum structural replacement cap;
- 15% + 0.001 replacement margin;
- P3-A.2 short-stroke geometry;
- multiscale tensor direction;
- source-support / protected-white checks;
- spacing and 32×32 tile fairness;
- P3-A.2.2 baseline-vs-candidate missing-structure utility;
- in-place structural slot substitution.

Changed:
- candidate proposal signal: **missing-edge residual instead of positive tonal residual**.

## Candidate proposal map

Use the already-computed:

```text
missing_edge = max(source_edge_strength - baseline_preview_edge_strength, 0)
```

The existing grid/local-search proposal machinery receives `missing_edge` as its candidate signal.

Therefore the generator still:
- searches a small deterministic neighborhood around grid cells;
- requires source-dark support;
- obtains tangent orientation from the same source tensor;
- creates the same short straight pencil geometry;
- applies the same full-stroke source-support check;
- keeps deterministic RNG and tile/spacing control.

But a candidate can now be born because **structure is missing**, even when positive tonal residual is not the strongest local signal.

## Important non-change

P3-A.2.3 does **not**:
- increase the structural budget;
- lower the selective replacement threshold;
- add semantic face/feature rules;
- change tensor thresholds;
- add curves or erasure;
- introduce P3-B render-search optimization.

The experiment is specifically about proposal coverage.

## Scoring and replacement

Once proposed, candidates are evaluated exactly as P3-A.2.2:

- sample missing-edge evidence along the candidate;
- include positive tonal residual as a secondary term;
- include source darkness;
- include alignment with source tensor direction;
- compare against baseline P2-B.1 contour utility;
- replace only if `hybrid >= baseline * 1.15 + 0.001`;
- replace at most 40% of structural slots;
- substitute into the original contour indices.

Zero replacements remains a valid result.

## Required statistics

Record:
- tone count;
- structural budget;
- candidate count;
- max replacements;
- replacements;
- baseline retained;
- weakest baseline utility;
- strongest candidate utility;
- mean missing-edge evidence at accepted replacements;
- changed structural slot count.

For research comparison, retain P3-A.2.2's candidate count/result so we can see whether deficit-driven proposal origin meaningfully changes pool coverage.

## Engineering gates

- exact tonal prefix parity;
- exact total count parity;
- replacement cap respected;
- changed slots == replacements;
- deterministic output;
- no-contours mode remains exact P2-A;
- protected white-channel tests remain green;
- all historical P3 modes remain reproducible.

## Quality gate

First run on the same private portrait.

The crucial question is spatial, not only aggregate:
- do changed strokes move into weak interior structure?
- do glasses/nose/lips or analogous nonsemantic underrepresented edges receive useful marks?
- does edge F1 recover or improve beyond P3-A.2.1?
- do tone/midtone/dark/white metrics remain close to P2-B.1?

Then run a genuine permission-cleared nonportrait image before claiming generality.

If the candidate pool still concentrates around easy outer/hair structure, the next step should reconsider spatial allocation or candidate geometry—not loosen the budget controls.
