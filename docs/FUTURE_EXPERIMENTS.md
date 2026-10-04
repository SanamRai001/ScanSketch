# Future Experiments — Parking Lot

Ideas here are **not approved implementation scope**. Keep this list separate from docs/ROADMAP.md. Revisit only after the main engine produces consistently convincing sketches and a specific hypothesis can be measured.

## FE-01 — Natural erasure and residual graphite (deferred)

**Origin:** while discussing Primitive-inspired optimization, the possibility arose that erroneous *committed* pencil strokes might be corrected with an eraser and a small visible remnant, as happens in some handmade graphite sketches.

Two independent questions:
1. **Functional correction:** would controlled removal/revision of existing strokes improve a monotonic darkening algorithm when earlier strokes over-darken a region?
2. **Artistic residue:** would a faint remnant of genuinely erased lines make the drawing feel more natural, or merely muddy it?

Proposed future approach, not an implementation instruction:
- Start with a clean correction/undo model and validate that it reduces real errors.
- Store a revision-aware stroke history; do not fake erasing by indiscriminately painting white over other overlapping marks.
- If the clean correction is useful, experimentally retain a tiny configurable amount of graphite from actually revised strokes.
- Compare clean output, clean erasure and residue visually and numerically using identical source images.
- Include residual ink in actual final-output scoring.
- Reject the aesthetic feature if it damages recognizability or makes outputs look dirty.

**Explicitly excluded now:** intentional incorrect strokes, faux smudges, eraser animations, default residue, and any eraser tool in P1–P5.

## FE-02 — Expanded stroke vocabulary

Investigate curved/contour-guided strokes, cross-hatching and direction fields only after measuring the original horizontal-stroke baseline. Don't erase the scanline identity merely to imitate existing photo filters.

## FE-03 — Graphite and paper material effects

Potential pressure variation, broken graphite texture and optional paper grain. Must improve the perceived sketch rather than conceal poor reconstruction. Never bake effects into the stroke model if a separate renderer can handle them.

## FE-04 — Advanced optimization

Alternative objectives, differentiable rendering, or more sophisticated region scheduling only if simple seeded local search proves inadequate.

## FE-05 — Resolution enhancement

Reconstruction of missing input detail is a different problem from transforming visible input into lines. Generative or super-resolution extensions require separate goals, disclosure and evaluation.

## Promotion gate

A future idea may enter the roadmap only after:
1. The core engine reaches a subjectively satisfactory and reproducible quality baseline.
2. A concrete deficiency and success metric are documented.
3. Complexity/performance and export implications are reviewed.
4. A small controlled experiment beats the simpler baseline.
5. docs/PROJECT_STATE.md records the agreed decision.
