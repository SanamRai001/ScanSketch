# P3-A.2.3 — Deficit-Driven Proposal Verification

**Status:** implementation passed GitHub CI: **62/62 Rust tests**, all inherited P2-C/P3 checks, and the P3-A.2.3 deficit-proposal smoke. Real-image quality remains pending. [CI run 37348529191](https://github.com/SanamRai001/ScanSketch/actions/runs/37348529191).

## Controlled change

P3-A.2.3 changes one variable relative to P3-A.2.2:

```text
candidate proposal signal:
    P3-A.2.2 = positive tonal residual
    P3-A.2.3 = missing-edge residual
```

Frozen:
- P2-B.1 baseline;
- exact P2 tonal prefix;
- exact total accepted stroke count;
- short-stroke geometry;
- source tensor/tangent direction;
- source-support and protected-white checks;
- spacing + 32×32 tile fairness;
- P3-A.2.2 missing-structure utility;
- 15% + 0.001 replacement margin;
- 40% replacement cap;
- in-place structural slot substitution.

This is a proposal-coverage experiment, not a threshold/budget increase.

## Engineering gates

CI must verify:
- exact tonal prefix parity;
- exact P2-B.1 total count;
- replacements + retained == structural budget;
- replacements <= frozen cap;
- changed structural slots == replacements;
- nonzero deficit-driven candidate pool on the step fixture;
- deterministic output;
- all historical modes/tests remain green.

## Windows portrait A/B after green CI

```powershell
cd D:\Projects\ScanSketch
git fetch origin
git status --short
git switch --track origin/feat/p3a23-deficit-driven-proposals

$env:CARGO_BUILD_JOBS = "2"

cargo check --workspace
cargo test --workspace

powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3a23-ab.ps1" -InputImage ".\sample.jpg" -OutputDir "experiments/local/p3a23/portrait-a" -MaxSize 512 -Seed 42 -RightsConfirmed
```

If the branch already exists locally:

```powershell
git switch feat/p3a23-deficit-driven-proposals
git pull --ff-only
```

Open:

```powershell
Start-Process ".\experiments\local\p3a23\portrait-a\p2b1.png"
Start-Process ".\experiments\local\p3a23\portrait-a\p3a23.png"
```

Print:

```powershell
$s = Get-Content ".\experiments\local\p3a23\portrait-a\summary.json" -Raw | ConvertFrom-Json

"P3-A.2.3 mix: tone=$($s.deficit_proposal_stats.tone_count); budget=$($s.deficit_proposal_stats.structural_budget); max=$($s.deficit_proposal_stats.max_replacements); replacements=$($s.deficit_proposal_stats.replacements); retained=$($s.deficit_proposal_stats.baseline_retained); candidates=$($s.deficit_proposal_stats.candidate_count); mean-missing=$($s.deficit_proposal_stats.mean_missing)"
"Changed structural slots: $($s.changed_structural_slots)"

$s.fixtures |
    Select-Object mode,strokes,path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1,highlight_ink |
    Format-Table -AutoSize
```

## What to inspect visually

The hypothesis is about **where changed strokes move**.

Compare P3-A.2.3 against P2-B.1 and, mentally, against the prior P3-A.2.1/P3-A.2.2 outcomes:

- are changes less concentrated in easy outer hair/silhouette?
- do underrepresented interior edges gain useful marks?
- does the face/object structure read more clearly without semantic rules?
- is tonal mass still essentially frozen?
- does white paper remain clean?

Aggregate edge F1 is important but insufficient.

## Acceptance

Promising if:
- exact invariants hold;
- changed structure visibly reaches underrepresented regions;
- edge F1 and/or human readability improve without material tonal/midtone regression;
- no new white contamination;
- intervention remains selective.

Then repeat on a **genuine permission-cleared nonportrait** before promoting.

Reject if the candidate pool still mostly targets already-obvious structure, even if a global edge metric rises.


## First CI observation

On the 64×64 step fixture:

- total strokes: **193 vs 193**;
- exact tonal prefix: **173**;
- structural budget: **20**;
- maximum replacements: **8**;
- replacements made: **6**;
- baseline structural strokes retained: **14**;
- deficit-driven candidate pool: **26**;
- P3-A.2.2 tone-residual candidate pool on the same fixture: **55**;
- weakest baseline utility: **0.120863**;
- strongest deficit candidate utility: **0.356100**;
- mean missing-edge evidence at accepted replacements: **0.127317** (P3-A.2.2: 0.126235);
- rounded edge F1: **~0.81** for P3-A.2.3 on this engineering fixture.

The important engineering signal is not the rounded metric itself: P3-A.2.3 produced a **smaller, more deficit-focused proposal pool** while preserving the same replacement count and every frozen invariant.
