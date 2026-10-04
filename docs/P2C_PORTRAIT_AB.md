# P2-C Portrait A/B — Tonal-only vs Coherence-ranked Contours

**Date:** 2026-10-04. **Status:** first user-supplied same-binary numeric comparison on one private portrait, not a multi-fixture, equal-budget or perceptual validation. The actual input photo and generated PNG/JSON remain local, not committed to this public repository. See [P2-C first portrait](P2C_FIRST_PORTRAIT_RESULT.md) for the full P2-B.1 source result.

## Controlled comparison

Both runs were generated on the same local `feat/p2c-measurement-utility` executable with the same `sample.jpg` bytes, `--max-size 512`, actual working resolution 368×512, seed 42, identical P2-C v1 analysis, same default stroke options and PNG/JSON export. **Only `--no-contours` changed.**

- P2-A tonal-only command produced `outputs/p2c-portrait-p2a-control.png` and paired `outputs/p2c-portrait-p2a-control-strokes.json`. Measured result saved locally to `outputs/p2c-portrait-p2a-control-report.json`.
- P2-B.1 default-contour command produced `outputs/p2c-portrait.png` and `outputs/p2c-portrait-strokes.json`. Measured result saved locally to `outputs/p2c-portrait-report.json`.
- Git exact local SHA/file hashes remain to be collected. The public branch code has P2-B.1 reconstruction unchanged from the earlier tested version.
- The earlier missing-file error was caused by trying to measure before the control PNG/JSON existed; after the user generated them first, the measurement succeeded.
- This is the **natural-output lane**: P2-B.1 adds marks. The tiny budget difference is disclosed below, not claimed to be a strictly equal-budget trial.

## Recorded numbers

| Metric | P2-A (`--no-contours`) | P2-B.1 (enabled) | Direction |
| --- | ---: | ---: | --- |
| Image dimensions | 368×512 | 368×512 | Same |
| Total pixels | 188416 | 188416 | Same |
| Overall linear-light tone RMSE (lower better) | 0.39827129133178985 | 0.39743605947970556 | B.1 lower by ~0.000835 |
| White-mask pixel count | 30814 | 30814 | Same |
| White RMSE | 0.016080508609430236 | 0.016080508609430236 | **Exactly unchanged** |
| Midtone-mask pixel count | 16518 | 16518 | Same |
| Midtone RMSE | 0.31013273632664523 | 0.30444623687925343 | B.1 lower by ~0.005686 |
| Dark-mask pixel count | 141084 | 141084 | Same |
| Dark RMSE | 0.44779205413232437 | 0.44725756214272244 | B.1 lower by ~0.000534 |
| White-mask unwanted-ink fraction | 0.000259622249626793 | 0.000259622249626793 | **Unchanged** |
| Mean extra highlight darkness | 0.0000345651849793871 | 0.0000345651849793871 | **Unchanged** |
| Fixed edge threshold/tolerance | 0.22 / 2px | 0.22 / 2px | Same |
| Source edge-positive pixels | 2477 | 2477 | Same |
| Preview edge-positive pixels | 5699 | 6130 | B.1 has +431 |
| Edge precision (higher better) | 0.1223021582733813 | 0.18401305057096248 | B.1 +0.061711 |
| Edge recall (higher better) | 0.37182075090835687 | 0.6697618086394832 | B.1 +0.297941 |
| **Edge F1 (higher better)** | **0.18406141258346656** | **0.28870588594667923** | **B.1 +0.104644, ~56.9% relative** |
| Actual strokes | 12563 | 12671 | B.1 +108 (+0.86%) |
| Total stroke-path length (px) | 79717.37483535177 | 80222.50718482705 | B.1 +505.13 (+0.63%) |
| Mean path length per stroke | 6.3454091248389535 | 6.3311898969952685 | Slightly shorter with contours |
| Mean width (px) | 0.6926141906641456 | 0.6908477667771163 | Descriptive |
| Median width (px) | 0.7029249668121338 | 0.7024496793746948 | Descriptive |
| Mean opacity | 0.71528572208001 | 0.7143909006278335 | Descriptive |
| Median opacity | 0.7344701290130615 | 0.7338904738426208 | Descriptive |

## Interpretation / decision

1. Contour-on improves this particular fixed Sobel edge-proximity proxy noticeably at a modest additional stroke/path-length cost. Improvement in edge **precision as well as recall** suggests that the marks are more often near source edges, rather than simply increasing the edge-pixel count, under this proxy's matching convention.
2. White-protection and average highlight ink did **not** deteriorate in these reports. Tonal RMSE improves only marginally overall; midtone error decreases more than dark-tone error.
3. No metric here recognizes glasses, eyes or lips. The 2px tolerant matching can credit multiple preview edge pixels against the same source edge location and favors some textures; the **56.9% relative F1 increase is not a 56.9% artistic-quality increase**. The prior visual review described a modest/incremental, not dramatic, perceptual improvement.
4. **Decision:** retain P2-B.1 as current structural baseline candidate and P2-A as tonal-only control. Do not spend another phase increasing generic Sobel opacity or thresholds.
5. **Still pending:** source/PNG/JSON SHA manifest, repeatable seeds 137/2026, one independently permission-cleared nonportrait photo, synthetic step/channel/gradient E2E checks, signed tone bias to distinguish too light/dark, explicitly equal-budget mode, generation wall-time, multi-image human review. A single same-photo default-budget trial is not enough to mark P2-C finished or claim full scientific superiority.

## Next gate

Run `scansketch-fixtures` and render/measure the **step**, **white-channel** and **gradient** cases, preserving unique paired PNG/JSON per fixture. On a permitted nonportrait object, run the same-binary `--no-contours` and default modes at equal image dimensions/seed, collect both reports. Only when the metrics and visuals hold across varied fixtures should the proposed multiscale, direction-aware P3-A prototype proceed. P3-B optimization remains separately gated.

Avoid publishing the personal source photograph, PNG/JSON, or private file hashes without permission.
