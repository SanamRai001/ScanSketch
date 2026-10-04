# Vision and Product Principles

## Problem

Typical grayscale filters recolor pixels rather than explain the image through drawn marks. ScanSketch asks whether a virtual pencil, guided by image information and a measurable optimization objective, can reconstruct the source as an appealing sketch made of actual strokes.

## Thesis

From a white canvas, a mostly horizontal top-to-bottom sweep proposes pencil marks only where useful. Bright regions remain mostly untouched; stroke density, placement, pressure and shape encode darker forms. Subsequent bounded refinement should improve visual recognition without sacrificing the appearance of hand-drawn lines.

## Who is it for?

Creative coders, graphics researchers, digital illustrators, designers and curious users who want to inspect or export the actual reconstructed strokes.

## Success criteria

- Clear sketch-line identity, not just monochrome recoloring, halftone dots, or filled polygons.
- Recognizably reconstructs different source categories (portrait, object, architecture and illustration).
- Retains important structure and bright negative space.
- Outputs reproducible results from a known seed and parameters.
- Demonstrates with measurements and blinded visual comparisons whether each more complex algorithm is worthwhile.
- Keep the UX minimal: image input, original/result comparison, render action, a few meaningful controls, and exports only when implemented.

## MVP scope (sequential, not everything at once)

1. Deterministic scanline-to-stroke baseline.
2. Measured improvements through candidate search, optimization, and local error scoring.
3. Only justified structural support and controlled additive refinement.
4. Actual retained vector stroke representation; raster preview first, vector export after parity tests.
5. Simple, accessible browser UI once a command/testable core can demonstrate convincing results.

## Constraints and non-goals

- Do not assume AI/ML training, paid infrastructure, or GPU hardware.
- Do not fabricate scene detail absent from the original image.
- Do not add intentional mistakes, erasing residue, smudging, paper grain, or effects simply to suggest realism.
- Do not implement accounts, collaboration, cloud storage, payments, or a large preset/settings system in the research MVP.
- Avoid claiming that output is lossless, original in the academic sense, or indistinguishable from real graphite.
- Preserve user image privacy; the intended first version runs locally in-browser, subject to actual architecture verification.

## Design principles

**Make complex things feel simple.** The complicated part is the engine, not the user's workflow. Favor sensible defaults, fast visual comparisons, progressive disclosure, keyboard-accessible controls and honest progress/cancellation reporting.
