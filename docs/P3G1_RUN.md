# P3-G1 — Runbook

## Prepare 3–5 local nonportrait photos

Create a dedicated folder containing only the images for this validation pack, for example:

```powershell
New-Item -ItemType Directory -Force ".\experiments\local\p3g1\sources" | Out-Null
```

Copy **3–5 permission-cleared nonportrait** PNG/JPEG files into that folder. Keep originals local/untracked.

Suggested source diversity:
- mug/bottle/tool;
- shoe/bag;
- chair/room/building detail;
- optional plant/natural object;
- optional light/midtone object.

## Run

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3g1-generalization.ps1" \
  -InputDir ".\experiments\local\p3g1\sources" \
  -OutputDir ".\experiments\local\p3g1\run-a" \
  -MaxSize 512 \
  -Seed 42 \
  -RightsConfirmed \
  -NonPortraitConfirmed
```

In Windows PowerShell, use backticks instead of the displayed line-continuation backslashes, or run as one line:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\run-p3g1-generalization.ps1" -InputDir ".\experiments\local\p3g1\sources" -OutputDir ".\experiments\local\p3g1\run-a" -MaxSize 512 -Seed 42 -RightsConfirmed -NonPortraitConfirmed
```

The output root must not already exist.

## Outputs

```text
run-a/
  01-<source>/
    p2b1.png
    p3a23.png
    p2b1-strokes.json
    p3a23-strokes.json
    p2b1-report.json
    p3a23-report.json
    summary.json
  ...
  summary.json
  summary.csv
  review.html
```

Open:

```powershell
Start-Process ".\experiments\local\p3g1\run-a\review.html"
```

The HTML is local and references local generated previews only.

## Share the aggregate evidence

Print:

```powershell
$s = Get-Content ".\experiments\local\p3g1\run-a\summary.json" -Raw | ConvertFrom-Json

$s.wins
$s.mean_changes_pct

$s.cases |
    Select-Object source_name,replacements,structural_budget,candidate_count,edge_f1_improvement_pct,tone_improvement_pct,midtone_improvement_pct,dark_improvement_pct,path_change_pct |
    Format-Table -AutoSize
```

For visual review, share generated A/B previews only if you have permission. The original source photographs do not need to be uploaded.

## Interpretation

Positive improvement percentages mean:
- lower RMSE for tone/midtone/dark/white;
- higher F1 for edges.

`path_change_pct` is just resource change, not a quality score.

Do not average your visual judgment. Review each source class individually, then make a phase decision.


## Harness status

The batch script itself is CI-verified on Windows-style PowerShell semantics in GitHub Actions: aggregate JSON, CSV and HTML were all produced successfully, and all frozen per-case invariants passed. Synthetic CI outputs are not substitutes for the required real nonportrait photographs.
