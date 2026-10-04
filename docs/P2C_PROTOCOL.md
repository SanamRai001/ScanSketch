# P2-C — Reproducible Measurement Protocol

**Status: design agreed; metrics/tooling and comparisons not yet executed.** This is the next gate, not a retroactive claim that P1–P2-B.1 were benchmarked. The owner of progress remains [PROJECT_STATE.md](PROJECT_STATE.md).

## 1. What are we testing?

H1: P2-A's fragmented tonal marks improve perceived sketch texture vs P1.  
H2: P2-B/B.1 provide structure that is visually useful beyond extra dark ink.  
H3 (next prototype): a multi-scale direction field offers more structural recognition at similar stroke/ink budgets than another global edge threshold.

Treat each as falsifiable. The portrait feedback supports a provisional visual preference for P2-B.1, **not** H2/H3 proof.

## 2. Fixed fixture set and rights

Synthetic fixtures (safe to generate programmatically): white RGBA, transparent RGBA with white matte expectation, large black shape with a narrow white channel, step edge, smooth gradient and a few separated dark lines. Commit generated synthetic *specifications*, not a private input.

Locally held, permission-cleared photographs: (a) current portrait, (b) a simple nonportrait object, (c) architecture/thin edges, (d) a texture-heavy scene. Keep originals in `experiments/local/` or outside the repo; never commit personal/private images or an ambiguous stock photo. Record permission/licensing and SHA-256 of each input.

All comparisons for a fixture must use the **same original bytes**, image orientation, preprocessing, alpha compositing, working size (initially maximum side **512px**), color/linear-light convention, seed and rendering resolution. Record actual resized width/height (the first portrait produced **368×512**). Recommended seed set for stochastic robustness: `[42, 137, 2026]`. First use 42 for direct comparisons, then report median/range across all selected seeds; do not cherry-pick a single appealing output.

## 3. Two comparison lanes — do not confuse them

**Lane A, natural defaults:** run unchanged P1, P2-A, P2-B, P2-B.1 and report their actual output stroke counts/ink lengths. This answers what each version naturally produces. Keep each git SHA and full command. In P2-B/B.1 use `--no-contours` as an exact same-binary P2-A control when useful.

**Lane B, matched resource budgets:** compare at equal stroke counts **and**, when possible, comparable total path length/ink deposition. `--max-strokes` currently fails on overflow; it does NOT automatically make two algorithms use equal budgets. Implement explicit budget-aware proposal scheduling before claiming matched-budget results. Until then, show per-stroke/path-length-normalized values as descriptive only; they do **not** replace a controlled experiment.

Do not merge feature branches just to conduct a comparison: use separate checkouts or worktrees and unique output paths. Preserve existing P1/P2 screenshots. Save the exact `--help` flags with each engine SHA because option defaults changed across phases.

## 4. Proposed numeric measurements

All outputs: decode PNG onto the same opaque-white matte; compare at the same working resolution. Extract source darkness from the same *linear-light* normalization used by `analysis.rs`, and evaluate preview ink darkness with that same rule. Do not compare sRGB source values to linear target values.

| Measure | Precise definition / interpretation |
| --- | --- |
| Tone RMSE | `sqrt(mean((D_source - D_preview)^2))` over matched working pixels. Lower is generally more faithful, not necessarily more sketch-like. |
| Region tone RMSE | Compute separately for target white, midtone and dark masks using fixed thresholds; report mask pixel counts. Prevent a large white background from concealing lost facial structure. |
| Unwanted highlight ink | Fraction of pixels whose target darkness `D_source <= 0.04` but preview darkness `D_preview > 0.04`. Also report total mean extra darkness on this mask. A blank-source case must have zero ink. |
| Structure proxy | Apply **identical** modest smoothing/gradient operators to source and preview; threshold magnitudes identically, then calculate edge precision/recall/F1 with a fixed 2px matching tolerance. Record thresholds and edge-positive counts. This proxy does not recognize facial landmarks and can reward mechanical texture; never use alone. |
| Stroke complexity | Accepted stroke count `N`, sum of Euclidean path lengths, mean/median width and opacity, and (if feasible) angle distribution; calculate from JSON strokes, not from estimated PNG lines. |
| Runtime / resources | Separate source decoding, generation and PNG/JSON export where instrumented. Record total elapsed wall time, debug/release profile, CPU, OS and peak memory if actually measured. Never report invented missing timings. |
| Human quality | Blinded side-by-side rank of recognizable structure, pencil feel, highlight cleanliness and overall preference at 100% size. Use labelled, optionally cropped regions *after* ranking. Record failures/disagreement. |

The initial thresholds above are protocol defaults, not tuned using the portrait. If changed after inspecting outputs, give the experiment a new protocol/version and rerun all versions. Avoid interpreting nearly identical numeric metrics as a meaningful aesthetic difference.

## 5. Manifest and report fields

One experiment run per engine+fixture+seed+settings:

```text
experiment_id:
protocol_revision: p2c-v1
timestamp:
fixture_id / rights / source_sha256:
engine_phase / branch / git_sha:
target_max_side / actual_width / actual_height:
seed / command / all nondefault flags / contour mode:
output_png_path / sha256:
stroke_json_path / sha256:
build_profile / machine:
N / total_path_length / mean_width / mean_opacity:
tone_rmse / masked_region_rmse:
white_ink_fraction / white_extra_darkness:
edge_threshold / edge_positive_counts / edge_precision / edge_recall / edge_f1:
elapsed_ms / peak_memory_mb (only if observed):
blind_preference / recognizability_notes:
decision / limitations / next hypothesis:
```

Store local completed manifest under `experiments/local/`; check in only synthetic, shareable fixtures and anonymized summaries. Exclude `target/`, generated PNGs and private photos.

## 6. Implementation sequence and acceptance gate

1. Generate synthetic fixtures and validate resize, alpha handling, same-seed JSON identity and true-white no-ink behavior.
2. Build a small reproducible measurement command that reads source PNG/JPEG, output PNG and stroke JSON; validate it against exact synthetic cases **before** using portrait scores. Choose Rust to reuse the native darkness normalization; optional Python is acceptable only as a clearly isolated research utility.
3. Log all P1→P2-B.1 defaults with their true stroke budgets and runtime; render same fixtures/seed with clearly distinct filenames and commit SHAs.
4. Perform blinded visual comparisons, then look at proxy disagreements. Do not declare a winner from one scalar score.
5. Choose P3-A directional proposal hypothesis and its budget against P2-B.1, using the same metrics. Revisit candidate optimization only after this comparison.

**Exit:** a reproducible script/tool, at least one synthetic sanity run, a licensed nonportrait comparison, the recorded portrait assessment and a written keep/revise/reject decision. The work in this branch establishes the **protocol and architecture**, not completed numeric results.
