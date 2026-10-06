# P4-A.2 — First Real Medium-Form Hierarchy Result

**Date:** 2026-10-06. **Status:** chair/mug/plant real-pack complete. **P4-A.2 passes the medium-scale hierarchy gate.**

Original source photographs remain local.

## What was tested

Frozen P4-A.1 long `Gesture` paths were compared against P4-A.2:

```text
P4-A.1 = Gesture
P4-A.2 = exact same Gesture + new Form paths
```

The full old P2-B.1 hatch field was shown only as an overlay reference. It is not accepted as the final P4 composition.

## Exact real-pack evidence

| Source | Gestures | Forms | Form seeds | Gesture mean | Form mean | Form max | Form length share | paths-only edge-F1 change vs P4-A.1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| chair.jpg | 4 | 6 | 179 | 54.76px | 12.24px | 15.42px | 25.11% | **+31.596%** |
| mug.jpg | 4 | 6 | 144 | 52.47px | 16.06px | 23.65px | 31.46% | **+30.538%** |
| plant.jpg | 5 | 6 | 168 | 45.39px | 13.23px | 16.16px | 25.91% | **+23.917%** |

Aggregate:

- sources with Forms: **3/3**;
- sources with at least two Forms: **3/3**;
- hierarchy paths-only edge-F1 wins vs P4-A.1: **3/3**;
- mean Gesture count: **4.333**;
- mean Form count: **6.0**;
- mean Gesture length: **50.873px**;
- mean Form length: **13.843px**;
- mean Form path-length share: **27.493%**;
- mean paths-only edge-F1 change vs P4-A.1: **+28.684%**.

The old full-field overlay had white-RMSE non-worse on **0/3** sources. This does not invalidate the path hierarchy; it confirms that stacking Gesture+Form on top of all legacy P2-B.1 ink is not the final composition strategy.

## Visual review

The center-column hierarchy previews show the intended scale separation.

### Chair

The four long Gestures establish the major chair structure. Six medium Forms add shorter seat/back/interior transitions without replacing or fragmenting the Gesture skeleton.

### Mug

This is the clearest hierarchy example. Long Gestures establish rim/body/base structure; medium Forms add handle/body and shorter curved structure. The result is more construction-like than the P4-A.1 skeleton alone.

### Plant

Long Gestures remain sparse and dominant while six Forms add secondary leaf/stem/pot cues. The medium layer remains visibly subordinate rather than becoming texture.

Across all three:
- Form count stays small;
- Forms are substantially shorter than Gestures;
- no new dash-field explosion appears in the paths-only hierarchy;
- Gestures remain the visual backbone.

## Decision

**P4-A.2 passes. Freeze Gesture + Form geometry.**

Do not tune:
- long Gesture thresholds;
- medium Form thresholds;
- Gesture/Form counts;
- path lengths;
- edge-F1 against these three images.

The next bottleneck is now composition/tone.

## Next: P4-A.3 residual hatching rebalance

P4-A.3 must stop using the full P2-B.1 horizontal field as the tonal foundation.

New ordering:

```text
1. Gesture paths
2. Form paths
3. compute remaining tonal need
4. add sparse short Hatch marks only where residual tone warrants them
```

The P2-B.1 field remains a control only.

### Required P4-A.3 properties

- exact frozen Gesture list from P4-A.2;
- exact frozen Form list from P4-A.2;
- Hatch marks are a separate `StrokeRole::Hatch` layer;
- short hatches remain roughly in the existing 2-10px vocabulary;
- hatch count must be materially lower than P2-B.1 segment count;
- structure corridors should not be buried by hatching;
- bright paper remains largely untouched;
- no requirement to match source tone pixel-for-pixel;
- visual hierarchy matters more than reconstruction RMSE.

### Acceptance direction

A successful P4-A.3 preview should read in this order:

1. object Gesture;
2. Form construction;
3. supporting tone.

It should **not** read as horizontal scan bands with contours on top.

The old overlay's 0/3 white-nonworse result is an explicit reason to rebuild tone from the hierarchy rather than append more ink.
