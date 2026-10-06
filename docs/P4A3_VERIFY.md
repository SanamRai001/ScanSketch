# P4-A.3 — Residual Hatching Verification

**Status:** engineering gate passed on GitHub Actions: **85/85 Rust tests**, every inherited P2/P3/P4 smoke, and the dedicated residual-hatching proof. Real chair/mug/plant visual gate remains. [CI run 37495188329](https://github.com/SanamRai001/ScanSketch/actions/runs/37495188329).

## Controlled change

P4-A.3 freezes the accepted P4-A.2 structural hierarchy:

- exact Gesture paths;
- exact Form paths.

It removes the old full P2-B.1 underlay from the P4 candidate and adds a new sparse residual `Hatch` layer.

The final P4-A.3 candidate contains **zero legacy straight Stroke records**.

## Composition

```text
Gesture
→ Form
→ render structure
→ compute positive source-vs-structure darkness residual
→ compress residual tone
→ protect Gesture/Form corridors
→ generate sparse short Hatch paths
```

## Initial frozen Hatch policy

- residual floor: 0.06;
- residual gain: 0.58;
- Hatch threshold: 0.10;
- 5px tonal bands rather than P2-B.1's 3px bands;
- 28px sampling segments;
- one Hatch layer only;
- Hatch length 2.0–9.5px;
- lighter/thinner than structural paths;
- deterministic independent RNG;
- maximum Hatch count = 55% of P2-B.1 control segment count;
- Gesture corridor protection radius 3px;
- Form corridor protection radius 2px.

The hard count ceiling guarantees that this phase cannot silently reconstruct the old full scan field.

## CLI modes

`--residual-hatching`
- exact P4-A.2 Gesture + Form;
- sparse residual Hatch;
- zero legacy segments.

`--residual-hatching-only`
- exact same Hatch list;
- no Gesture/Form;
- zero legacy segments.

## Proof script

`scripts/run-p4a3-ab.ps1` writes:

- `p2b1.png` — historical control;
- `p4a2-paths.png` — frozen Gesture+Form;
- `p4a3-final.png` — new P4 candidate;
- `p4a3-hatches.png` — Hatch diagnostics;
- JSON and measurement reports;
- `summary.json`.

It asserts:

- P4-A.3 has zero old straight segments;
- P4-A.2 Gesture/Form list is byte/value exact inside P4-A.3;
- final Hatch list equals Hatch-only mode exactly;
- role counts match reported stats;
- Hatch count stays inside the sparse budget;
- Hatch count reduction is at least 40% versus P2-B.1 when a control exists.

## Core tests

Rust tests require:

- white source → no Hatch marks;
- dark field → sparse Hatches and zero legacy segments;
- P4-A.2 hierarchy exact preservation;
- Hatch-only parity;
- Hatch paths remain 2-point and 2–9.5px;
- deterministic generation.

## Engineering smoke

The rights-clear `form-detail.png` fixture must prove that a source with known Gesture+Form hierarchy can also receive residual hatching without changing those structural paths.

CI requires:
- full workspace tests green;
- exact structure preservation;
- zero legacy P4-A.3 segments;
- nonzero Hatch layer;
- Hatch count >=40% lower than P2-B.1;
- Hatch-only parity.

## Real image gate

Reuse the same chair / mug / plant sources.

Primary visual comparison:

```text
P2-B.1 control
P4-A.2 Gesture + Form only
P4-A.3 Gesture + Form + residual Hatch
P4-A.3 Hatch only
```

The principal question is not RMSE:

> Does P4-A.3 read as purposeful structural drawing with supporting tone, instead of a scanline field?

Desired:
- structure remains visually dominant;
- enough Hatch tone remains to describe dark mass;
- more paper is visible;
- Hatch bands no longer dominate the image;
- no structural path is buried by hatching;
- object remains recognizable.

## Metric interpretation

P4-A.3 is explicitly allowed to worsen tone RMSE versus P2-B.1.

Important numeric evidence:
- Hatch count reduction;
- role counts;
- structure-vs-Hatch path-length share;
- white-region behavior;
- edge F1 as secondary structural evidence.

Do not reject a visually more human sketch merely because it reconstructs grayscale tone less literally.

## Exit

If chair/mug/plant retain recognizability while scanline dominance materially falls, proceed toward P4-G human-likeness validation.

If density is good but mostly-horizontal Hatch orientation still looks mechanical, open a **small P4-A.3.x form-aware Hatch direction experiment** without changing Gesture/Form or Hatch count.


## First engineering result

On the rights-clear 64×64 `form-detail` fixture:

- frozen Gestures: **4**;
- frozen Forms: **1**;
- residual Hatches: **42**;
- P2-B.1 control segments: **161**;
- Hatch budget ceiling: **88**;
- Hatch-count reduction: **73.91%**;
- residual candidate pixels: **1676**;
- mean Hatch length: **4.57px**;
- structure path-length share: **49.32%**;
- Hatch path-length share: **50.68%**;
- legacy straight segments in P4-A.3 final: **0**;
- final Hatch list exactly equals Hatch-only mode.

This clears the engineering gate by a wide margin. The important next question is visual: does the same composition look more human and less scanline-dominated on chair, mug and plant?
