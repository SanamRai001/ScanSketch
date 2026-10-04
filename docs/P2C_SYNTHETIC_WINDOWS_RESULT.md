# P2-C — Windows synthetic A/B (full-precision results)

**Observed:** 2026-10-04, supplied PowerShell run after local `git pull --ff-only` fast-forwarded `feat/p2c-measurement-utility` to `d15ef1a73fd95cc1adcf74ad7d3b23284b56bbeb`. Both contour states used the **same binary**, source fixture bytes, maximum side 64, actual 64×64 dimensions, seed 42 and `p2c-v1` metrics. This records the user's **local** terminal outputs, separately from [the earlier CI run](P2C_SYNTHETIC_CI_RESULT.md). No private photograph was involved.

The script `scripts/run-p2c-synthetic.ps1` generated paired PNG + ordered stroke JSON + measurement JSON for each mode, six runs in total. The local SHA-256 manifest and all outputs are retained in `experiments/local/p2c-v1/results/` (Git-ignored). Their hash *values* were not included in the pasted terminal output. Rust compilation finished and all commands exited successfully.

## Exact metrics printed by Windows PowerShell

| Fixture | Mode | Stroke count | Tone RMSE ↓ | Edge F1 ↑ | Unwanted white-source ink fraction ↓ |
| --- | --- | ---: | ---: | ---: | ---: |
| Step half-plane | P2-A | 173 | 0.3249329741246735 | 0.8214285714285715 | 0.0 |
| Step half-plane | P2-B.1 | 193 | 0.3168006844265966 | 0.7894736842105263 | 0.0 |
| Square/white-channel | P2-A | 196 | 0.34122597310393504 | 0.9938837920489296 | 0.02548076923076923 |
| Square/white-channel | P2-B.1 | 236 | 0.32253577114074605 | 0.9937791601866253 | 0.02548076923076923 |
| Smooth gradient | P2-A | 290 | 0.3867534167355713 | 0.0 | 0.0 |
| Smooth gradient | P2-B.1 | 290 | 0.3867534167355713 | 0.0 | 0.0 |

### What the numbers permit us to conclude

- **Step:** +20 contours lowers tone RMSE by ~0.00813 yet decreases edge F1 by ~0.03195 (0.82143→0.78947). White-source ink remains 0.
- **White-channel shape:** +40 contours lowers tone RMSE by ~0.01869, but F1 is essentially unchanged/slightly lower (0.99388379→0.99377916). Both modes have identical white-mask-ink fraction.
- **Gradient:** both reported modes have the same count, tone RMSE, edge F1 and highlight-ink rate. These *reported metric equalities* are not by themselves byte-identity proof; compare the recorded PNG/JSON hashes separately.
- **2.5480769% white-mask ink** was reported for both square/white-channel runs. This means over the complete source-white mask (outside the square **plus** internal white channel), **not** that exactly 2.548% of the narrow channel is inked. The 64×64 fixture defines a black square over x=8..55,y=8..55 except x=29..34; thus 2080 expected source-white pixels. The measured fraction corresponds to **53 threshold-exceeding preview pixels** over that complete mask. Their coordinates and causes remain unknown until actual PNG/mask inspection.
- The gradient edge F1=0 may reflect the fixed edge proxy/threshold and should not be interpreted as a complete visual assessment. Request full individual reports to inspect source and preview edge-positive counts.
- The first same-photo portrait A/B favored contours for edge alignment, but these synthetic trials have **mixed results**. Do not declare a universal winner. This is still a *natural-output* comparison (different accepted stroke counts), not a matched-budget experiment.

## Immediate diagnostic before P3-A

1. Compare three local, synthetic, permission-safe images: `square-white-channel.png`, `results/square-white-channel-p2a.png`, `results/square-white-channel-p2b1.png`, preferably enlarged with **nearest-neighbor pixels**. Determine whether the measured 53 source-white affected pixels lie in the inner white channel, external white area near the black-square boundary, or elsewhere. Distinguish vector-footprint crossing, raster antialiasing and mask threshold behavior by examining image coordinates before proposing a fix.
2. For the gradient, inspect each `gradient-*-report.json` `edges.source_pixels` and `edges.preview_pixels`. For step/channel also inspect precision/recall, white-mask pixel count and extra highlight darkness. The compact batch table omits these fields.
3. Verify source/preview/stroke SHA-256 values from local `summary.json` without publishing a private portrait. A hash comparison can establish whether gradient outputs are byte-identical under each mode.
4. Run one **permission-cleared nonportrait** photograph at 512px/seed 42 in both modes with paired PNG/JSON + metrics, then record human visual notes. Only consider advancing the proposed P3-A after this gate; no further raw-Sobel tuning merely to improve a single proxy.

See [PHASE_EVOLUTION.md](PHASE_EVOLUTION.md), which preserves P0→current outcomes as an append-only record, and [PHASE_RESULT_TEMPLATE.md](PHASE_RESULT_TEMPLATE.md).
