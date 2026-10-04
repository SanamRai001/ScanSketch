# Research, Prior Art and Attribution

This document records existing ideas and separates published work from ScanSketch hypotheses. External links are research references, not dependencies already imported.

## Principal inspiration: Primitive

Michael Fogleman, [Primitive](https://github.com/fogleman/primitive) (2016, MIT License), rebuilds a source image by iteratively adding an optimized geometric primitive. Its implementation includes candidate search, bounded hill climbing, multiple workers/restarts, best-fit color estimates and affected-pixel/partial difference scoring.

Relevant source reading:
- [model.go](https://github.com/fogleman/primitive/blob/master/primitive/model.go): committed shape and model steps.
- [worker.go](https://github.com/fogleman/primitive/blob/master/primitive/worker.go): candidate search.
- [optimize.go](https://github.com/fogleman/primitive/blob/master/primitive/optimize.go): hill climbing.
- [core.go](https://github.com/fogleman/primitive/blob/master/primitive/core.go): local raster scoring/compositing.
- [quadratic.go](https://github.com/fogleman/primitive/blob/master/primitive/quadratic.go): quadratic path as a supported stroke-like shape.

Adapt **principles**, not the polygon aesthetics: our intended unit is an expressive pencil stroke chosen by a top-to-bottom guided scan and evaluated using sketch-specific constraints. Primitive's internal scanline rasterization converts geometric coverage to rows; it is not itself the creative row-scanning method proposed for ScanSketch.

Before copying any Primitive implementation, retain its own copyright and MIT license text in a suitable notice. The ScanSketch LICENSE covers our original material and does not erase upstream obligations.

## Other relevant background

- [Potrace](https://potrace.sourceforge.net/): established raster tracing, particularly for bitmap silhouettes; useful as a distinct contour-focused baseline.
- [VTracer](https://github.com/visioncortex/vtracer): color raster-to-vector tracing; study its goals rather than assuming it creates pencil sketches.
- [DiffVG](https://github.com/BachiLi/diffvg): differentiable vector graphics rendering and optimization; worth investigating later if simple local search reaches limits.
- Classical image processing: Sobel/Canny-style edge detection, image gradients, thresholding, antialiasing and sRGB/linear-light handling.
- Stroke/engraving/halftone art is nearby prior art; compare outcomes carefully rather than claiming row scanning or image-to-sketch is a new invention.

## Questions to validate

1. Can short horizontal scanline strokes represent tone while remaining visually sketch-like?
2. Does adding Primitive-inspired stroke search improve results against a deterministic no-search baseline at equal budgets?
3. Which score best balances tone, highlights, important boundaries and line aesthetics?
4. Can a second pass preserve the scanline visual identity?
5. Do true vector paths export with comparable preview geometry and acceptable file size?
6. Is partial-image score updating correct for each selected objective, including edge neighborhoods?
7. At which image/candidate sizes does optimization become too slow on modest CPU-only hardware?

## Research ethics and reproducibility

Credit source authors, cite any incorporated ideas/code, keep provenance and redistribution rights for fixtures, disclose whether visuals are generated or merely reference examples, report failures as well as best results, and avoid unsupported novelty claims.
