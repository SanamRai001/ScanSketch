# P4-A.3 multi-image validation pack.
# Frozen P2-B.1 control vs frozen P4-A.2 hierarchy vs P4-A.3 residual Hatch composition.
param(
    [Parameter(Mandatory = $true)][string]$InputDir,
    [Parameter(Mandatory = $true)][string]$OutputDir,
    [ValidateRange(32,1024)][int]$MaxSize = 512,
    [long]$Seed = 42,
    [switch]$RightsConfirmed,
    [switch]$NonPortraitConfirmed
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
Set-Location $root

function FromRoot([string]$path) {
    if ([System.IO.Path]::IsPathRooted($path)) { return $path }
    return Join-Path $root $path
}
function Safe-Slug([string]$name) {
    $slug = [regex]::Replace($name.ToLowerInvariant(), '[^a-z0-9]+', '-').Trim('-')
    if ([string]::IsNullOrWhiteSpace($slug)) { return "image" }
    return $slug
}
function Html([object]$value) {
    return [System.Net.WebUtility]::HtmlEncode([string]$value)
}
function Improve-Pct([double]$baseline, [double]$candidate, [bool]$HigherIsBetter) {
    if ([math]::Abs($baseline) -lt 1e-12) { return $null }
    if ($HigherIsBetter) {
        return (($candidate - $baseline) / [math]::Abs($baseline)) * 100.0
    }
    return (($baseline - $candidate) / [math]::Abs($baseline)) * 100.0
}

if ($Seed -lt 0) { throw "-Seed must be nonnegative." }
if (-not $RightsConfirmed) {
    throw "Pass -RightsConfirmed only when you may process every image in the input folder."
}
if (-not $NonPortraitConfirmed) {
    throw "Pass -NonPortraitConfirmed only when all selected sources are genuine nonportrait images."
}

$sourceDir = (Resolve-Path -LiteralPath (FromRoot $InputDir) -ErrorAction Stop).Path
$destination = FromRoot $OutputDir
if (Test-Path -LiteralPath $destination) {
    throw "Output already exists; choose a NEW -OutputDir: $destination"
}

$images = @(
    Get-ChildItem -LiteralPath $sourceDir -File |
        Where-Object { $_.Extension.ToLowerInvariant() -in @(".png",".jpg",".jpeg") } |
        Sort-Object Name
)
if ($images.Count -lt 3 -or $images.Count -gt 5) {
    throw "P4-A.3 validation requires exactly 3-5 image files; found $($images.Count). Use a dedicated folder."
}

$commit = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) {
    throw "Could not resolve Git HEAD."
}

New-Item -ItemType Directory -Path $destination -Force | Out-Null
$rows = @()
$cards = @()
$index = 0

foreach ($image in $images) {
    $index++
    $caseName = "{0:D2}-{1}" -f $index,(Safe-Slug $image.BaseName)
    $caseDir = Join-Path $destination $caseName

    Write-Host ""
    Write-Host ("=== P4-A.3 case {0}/{1}: {2} ===" -f $index,$images.Count,$image.Name)

    $caseArgs = @{
        InputImage = $image.FullName
        OutputDir = $caseDir
        MaxSize = $MaxSize
        Seed = $Seed
        RightsConfirmed = $true
    }
    & (Join-Path $root "scripts\run-p4a3-ab.ps1") @caseArgs
    if ($LASTEXITCODE -ne 0) {
        throw "P4-A.3 four-view run failed for $($image.Name)"
    }

    $s = Get-Content -LiteralPath (Join-Path $caseDir "summary.json") -Raw | ConvertFrom-Json
    if (-not $s.structure_exactly_preserved) {
        throw "P4-A.3 changed frozen hierarchy for $($image.Name)"
    }
    if (-not $s.final_hatches_equal_hatch_only) {
        throw "P4-A.3 Hatch-only parity failed for $($image.Name)"
    }

    $base = $s.fixtures | Where-Object { $_.mode -eq "p2b1" }
    $hier = $s.fixtures | Where-Object { $_.mode -eq "p4a2-paths" }
    $final = $s.fixtures | Where-Object { $_.mode -eq "p4a3-final" }
    $hatch = $s.fixtures | Where-Object { $_.mode -eq "p4a3-hatches" }
    if (-not $base -or -not $hier -or -not $final -or -not $hatch) {
        throw "Missing P4-A.3 comparison row for $($image.Name)"
    }

    if ($final.segment_count -ne 0 -or $hatch.segment_count -ne 0) {
        throw "P4-A.3 final/hatch-only contains legacy segments for $($image.Name)"
    }
    if ($s.residual_stats.control_segments -gt 0 -and $s.residual_stats.hatch_reduction -lt 0.40) {
        throw "P4-A.3 hatch reduction fell below 40% for $($image.Name)"
    }

    $row = [pscustomobject]@{
        source_name = $image.Name
        case_name = $caseName
        control_segments = [int]$s.residual_stats.control_segments
        gesture_count = [int]$s.residual_stats.gestures
        form_count = [int]$s.residual_stats.forms
        hatch_count = [int]$s.residual_stats.hatches
        hatch_budget = [int]$s.residual_stats.hatch_budget
        hatch_reduction_pct = [double]$s.residual_stats.hatch_reduction * 100.0
        hatch_mean_px = [double]$s.residual_stats.hatch_mean
        structure_share_pct = [double]$s.residual_stats.structure_share * 100.0
        hatch_share_pct = [double]$s.residual_stats.hatch_share * 100.0
        residual_pixels = [int]$s.residual_stats.residual_pixels
        hierarchy_logical_marks = [int]$hier.logical_marks
        final_logical_marks = [int]$final.logical_marks
        final_tone_improvement_vs_p2b1_pct = Improve-Pct $base.tone_rmse $final.tone_rmse $false
        final_midtone_improvement_vs_p2b1_pct = Improve-Pct $base.midtone_rmse $final.midtone_rmse $false
        final_dark_improvement_vs_p2b1_pct = Improve-Pct $base.dark_rmse $final.dark_rmse $false
        final_edge_f1_improvement_vs_p2b1_pct = Improve-Pct $base.edge_f1 $final.edge_f1 $true
        baseline_white_rmse = [double]$base.white_rmse
        final_white_rmse = [double]$final.white_rmse
        final_white_nonworse = ([double]$final.white_rmse -le [double]$base.white_rmse + 1e-12)
        final_highlight_ink = [double]$final.highlight_ink
        hierarchy_edge_f1 = [double]$hier.edge_f1
        final_edge_f1 = [double]$final.edge_f1
    }
    $rows += $row

    $baseRel = "$caseName/p2b1.png"
    $hierRel = "$caseName/p4a2-paths.png"
    $finalRel = "$caseName/p4a3-final.png"
    $hatchRel = "$caseName/p4a3-hatches.png"

    $cards += @"
<section class="case">
<h2>$(Html $image.Name)</h2>
<div class="quad">
<figure><img src="$(Html $baseRel)" alt="P2-B.1"><figcaption>P2-B.1 control</figcaption></figure>
<figure><img src="$(Html $hierRel)" alt="P4-A.2 hierarchy"><figcaption>P4-A.2 Gesture + Form only</figcaption></figure>
<figure class="focus"><img src="$(Html $finalRel)" alt="P4-A.3 final"><figcaption>P4-A.3 FINAL - judge this first</figcaption></figure>
<figure><img src="$(Html $hatchRel)" alt="P4-A.3 hatches"><figcaption>P4-A.3 Hatch only</figcaption></figure>
</div>
<table>
<tr><th>Gesture / Form / Hatch</th><td>$($row.gesture_count) / $($row.form_count) / $($row.hatch_count)</td></tr>
<tr><th>P2-B.1 segments -> Hatch marks</th><td>$($row.control_segments) -> $($row.hatch_count)</td></tr>
<tr><th>Hatch count reduction</th><td>$([math]::Round($row.hatch_reduction_pct,2))%</td></tr>
<tr><th>Mean Hatch length</th><td>$([math]::Round($row.hatch_mean_px,2))px</td></tr>
<tr><th>Structure / Hatch length share</th><td>$([math]::Round($row.structure_share_pct,1))% / $([math]::Round($row.hatch_share_pct,1))%</td></tr>
<tr><th>Final edge-F1 change vs P2-B.1</th><td>$([math]::Round([double]$row.final_edge_f1_improvement_vs_p2b1_pct,3))%</td></tr>
<tr><th>Final tone change vs P2-B.1</th><td>$([math]::Round([double]$row.final_tone_improvement_vs_p2b1_pct,3))%</td></tr>
<tr><th>White RMSE</th><td>$([math]::Round($row.baseline_white_rmse,5)) -> $([math]::Round($row.final_white_rmse,5))</td></tr>
</table>
<div class="questions">
<strong>Primary visual questions:</strong>
<ul>
<li>Does the final image read as Gesture -> Form -> supporting tone?</li>
<li>Are the old horizontal scan bands no longer the first thing you see?</li>
<li>Are Gesture/Form lines still visually clear instead of buried under hatching?</li>
<li>Do dark masses retain enough tone for the object to read?</li>
<li>Is there visibly more useful paper/white space than in P2-B.1?</li>
<li>Does the mostly-horizontal Hatch layer still look mechanically oriented even after density reduction?</li>
</ul>
</div>
</section>
"@
}

$reductionPass = @($rows | Where-Object { $_.hatch_reduction_pct -ge 40.0 }).Count
$reduction60 = @($rows | Where-Object { $_.hatch_reduction_pct -ge 60.0 }).Count
$structureMajority = @($rows | Where-Object { $_.structure_share_pct -gt 50.0 }).Count
$whiteNonWorse = @($rows | Where-Object { $_.final_white_nonworse }).Count
$edgeWins = @($rows | Where-Object { $_.final_edge_f1_improvement_vs_p2b1_pct -gt 0 }).Count

$aggregate = [pscustomobject]@{
    phase = "p4a3-real-pack"
    algorithm = "gesture-form-residual-hatch"
    git_sha = $commit
    seed = $Seed
    max_side = $MaxSize
    source_count = $rows.Count
    rights_confirmed_by_operator = $true
    nonportrait_confirmed_by_operator = $true
    counts = [pscustomobject]@{
        hatch_reduction_at_least_40pct = $reductionPass
        hatch_reduction_at_least_60pct = $reduction60
        structure_path_length_majority = $structureMajority
        final_white_nonworse_vs_p2b1 = $whiteNonWorse
        final_edge_f1_wins_vs_p2b1 = $edgeWins
    }
    means = [pscustomobject]@{
        control_segments = ($rows | Measure-Object control_segments -Average).Average
        hatch_count = ($rows | Measure-Object hatch_count -Average).Average
        hatch_reduction_pct = ($rows | Measure-Object hatch_reduction_pct -Average).Average
        hatch_mean_px = ($rows | Measure-Object hatch_mean_px -Average).Average
        structure_share_pct = ($rows | Measure-Object structure_share_pct -Average).Average
        hatch_share_pct = ($rows | Measure-Object hatch_share_pct -Average).Average
        final_edge_f1_improvement_vs_p2b1_pct = ($rows | Measure-Object final_edge_f1_improvement_vs_p2b1_pct -Average).Average
        final_tone_improvement_vs_p2b1_pct = ($rows | Measure-Object final_tone_improvement_vs_p2b1_pct -Average).Average
    }
    cases = $rows
}

$aggregatePath = Join-Path $destination "summary.json"
$aggregate | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $aggregatePath -Encoding UTF8
$rows | Export-Csv -LiteralPath (Join-Path $destination "summary.csv") -NoTypeInformation -Encoding UTF8

$html = @"
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>ScanSketch P4-A.3 Residual Hatch Review</title>
<style>
body{font-family:system-ui,sans-serif;margin:24px;background:#202020;color:#eee}
h1{margin-bottom:6px}.meta{color:#bbb;max-width:1100px}.case{margin:42px 0;padding-top:18px;border-top:1px solid #555}
.quad{display:grid;grid-template-columns:1fr 1fr;gap:14px;align-items:start}
figure{margin:0;background:#111;padding:8px}figure.focus{outline:3px solid #aaa}
img{width:100%;height:auto;display:block;background:white}
figcaption{text-align:center;padding-top:8px;font-size:14px}
table{margin-top:14px;border-collapse:collapse}th,td{border:1px solid #555;padding:6px 10px;text-align:left}
.questions{margin-top:14px;max-width:980px;color:#ddd}.questions li{margin:4px 0}
@media(max-width:900px){.quad{grid-template-columns:1fr}}
</style>
</head>
<body>
<h1>ScanSketch P4-A.3 - Gesture + Form + sparse residual Hatch</h1>
<p class="meta">Seed $(Html $Seed), max-side $(Html $MaxSize), sources $(Html $rows.Count). <strong>Judge the P4-A.3 FINAL panel first.</strong> RMSE is secondary: this phase deliberately trades literal raster reconstruction for human-like mark hierarchy.</p>
<p class="meta">>=40% Hatch reduction: $reductionPass/$($rows.Count). >=60% reduction: $reduction60/$($rows.Count). Structure path-length majority: $structureMajority/$($rows.Count). White non-worse vs P2-B.1: $whiteNonWorse/$($rows.Count).</p>
$($cards -join [Environment]::NewLine)
</body>
</html>
"@

$htmlPath = Join-Path $destination "review.html"
$html | Set-Content -LiteralPath $htmlPath -Encoding UTF8

Write-Host ""
Write-Host "=== P4-A.3 real-pack summary ==="
$rows |
    Select-Object source_name,control_segments,gesture_count,form_count,hatch_count,hatch_reduction_pct,hatch_mean_px,structure_share_pct,hatch_share_pct,final_edge_f1_improvement_vs_p2b1_pct,final_tone_improvement_vs_p2b1_pct |
    Format-Table -AutoSize
Write-Host ("Coverage: >=40% reduction={0}/{1}; >=60%={2}/{1}; structure-majority={3}/{1}; white-nonworse={4}/{1}; edge-wins={5}/{1}" -f $reductionPass,$rows.Count,$reduction60,$structureMajority,$whiteNonWorse,$edgeWins)
Write-Host "Aggregate JSON: $aggregatePath"
Write-Host "Aggregate CSV:  $(Join-Path $destination 'summary.csv')"
Write-Host "Visual review:  $htmlPath"
Write-Host "Judge P4-A.3 FINAL first. Original photos are not copied or uploaded."
