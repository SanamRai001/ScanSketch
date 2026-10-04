# P2-C First Real-Image Measurement — Local Portrait

**Status:** first valid numeric observation from the user's Windows CLI output, **not** a controlled multi-version benchmark. Original portrait/PNG/JSON remain local; no photograph or source binary is copied to the public repository.

## Provenance

| Item | Recorded fact |
| --- | --- |
| Date | 2026-10-04 (provided user run) |
| Fixture | Local private portrait (`sample.jpg`); redistribution/license information not supplied |
| Source SHA-256 | Not yet recorded |
| Renderer/measurement branch | `feat/p2c-measurement-utility`, carrying unchanged P2-B.1 reconstruction plus metrics |
| Exact local git SHA | To record via `git rev-parse HEAD` (branch tip at first metrics PR: `2ef66f7df3bc3b93364ee7bd09ef2e7a804ff88d`) |
| Working image | 368×512 RGBA; maximum requested side 512 |
| Seed / contour setting | 42 / enabled, default `--contour-threshold 0.28` |
| Output (local only) | `outputs/p2c-portrait.png` |
| Stroke records (local only) | `outputs/p2c-portrait-strokes.json` |
| Report (local only) | `outputs/p2c-portrait-report.json` |
| Profile | Cargo `dev` (unoptimized + debuginfo); reported compile durations are **not** image-generation benchmarks |
| Original/preview/JSON hashes | Not yet supplied |

The command generated the paired image and stroke records then successfully ran `scansketch-measure` with `--max-size 512`; the report declared protocol revision `p2c-v1`.

## Observed result (verbatim numeric data, reformatted)

| Metric | Value |
| --- | ---: |
| Total pixels | 188416 |
| Overall linear-light tone RMSE | 0.39743605947970556 |
| White source-mask pixels | 30814 |
| White-region RMSE | 0.016080508609430236 |
| Midtone source-mask pixels | 16518 |
| Midtone-region RMSE | 0.30444623687925343 |
| Dark source-mask pixels | 141084 |
| Dark-region RMSE | 0.44725756214272244 |
| Unwanted-highlight-ink fraction (of SOURCE-white pixels) | 0.000259622249626793 |
| Mean extra highlight darkness | 0.0000345651849793871 |
| Edge threshold; tolerance | 0.22; 2px |
| Source edge pixels | 2477 |
| Preview edge pixels | 6130 |
| Edge precision | 0.18401305057096248 |
| Edge recall | 0.6697618086394832 |
| Edge F1 | 0.28870588594667923 |
| Stroke count | 12671 |
| Total path length (working pixels) | 80222.50718482705 |
| Mean path length (px) | 6.3311898969952685 |
| Mean/median width (px) | 0.6908477667771163 / 0.7024496793746948 |
| Mean/median opacity | 0.7143909006278335 / 0.7338904738426208 |

## Interpretation and limits

- The white-source mask has little unintended ink relative to its size. This is encouraging for background/highlight protection **on this fixture**; it does not establish subpixel/vector parity or robustness on other examples.
- The dark source mask has the largest regional RMSE, while 141084 source pixels are in that mask. The **direction** of that error is unknown: RMSE is unsigned; neither under-inking nor over-inking can be established from this report.
- The edge proxy detects **6130 preview vs 2477 source edge pixels**. Relatively low precision (0.1840) and recall (0.6698) are compatible with too many extra texture edges and incomplete meaningful structures, as seen visually. The ±2px edge matching is not semantic recognition; distinct strokes can match the same neighborhood, so this is a proxy, not a face recognizability score.
- The single P2-B.1 report is not evidence that P2-B.1 beats P2-A, P2-B or P1 numerically. Earlier visual comparisons only provisionally favored P2-B.1.
- No true generation wall-time, peak memory, blinded preference, input/output hashes or equal-resource benchmark was supplied.

## Immediate experiment

Run `--no-contours` **on the exact same P2-C binary**, with the same `sample.jpg`, max-size 512, seed 42, and export *new* paired PNG/JSON. Compare its metrics to this record; see [P2-C runbook, same-binary P2-A control](P2C_RUN.md). The prior 12,563 tonal-stroke count is only a consistency expectation.

Do not alter `p2c-v1` measurement constants during this comparison. If signed tonal bias or alternate edge thresholds are needed, version those additions separately and re-evaluate both variants.
