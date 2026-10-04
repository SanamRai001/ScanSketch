# Contributing to ScanSketch

Thank you for your interest in this early-stage research project.

## Start with the research question

Read the README, docs/VISION.md, docs/ALGORITHM.md, and docs/PROJECT_STATE.md before proposing an implementation. Prefer one focused hypothesis or improvement at a time; avoid adding framework, product, authentication, or design-system complexity before demonstrated need.

## Proposed change process

1. State the problem and why it affects sketch appearance, fidelity, performance, usability, or reproducibility.
2. Define an observable acceptance condition and a baseline comparison.
3. Use a focused branch and a small pull request; do not alter unrelated work.
4. Include input provenance and licensing, runtime environment, deterministic seed, parameter values, before/after output, and relevant measures for algorithm changes.
5. Clearly label unverified visual judgments and performance claims.
6. Update docs/PROJECT_STATE.md and any design document made stale by the change.

## Coding principles (when implementation starts)

- Make processing deterministic with a documented random seed.
- Separate image analysis, stroke representation, optimization, raster preview and export.
- Keep source images and generated output out of Git unless small, lawful, redistributable fixtures are intentionally selected.
- Never commit user-supplied private images, credentials, or large generated assets.
- Bound dimensions, candidate counts, time/memory use, and user-file parsing.
- Credit third-party ideas and preserve dependency/source licenses.

## Licensing

ScanSketch's original material is MIT licensed, with copyright attributed to Sanam Rai. By submitting a contribution, you agree that it may be distributed under the project's MIT License and affirm you have the right to contribute it; this is not a copyright assignment. Do not paste third-party code, fonts, data, or images without checking the rights and required notices.

## Security and privacy

During development use local image processing by default. Report a suspected vulnerability privately to the repository maintainer rather than publishing exploit details in an issue. Do not promise that a web implementation processes all images locally unless verified.
