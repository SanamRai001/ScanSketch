# P4-A.2 multi-image validation pack.
# Runs frozen P4-A.1 Gestures vs P4-A.2 Gesture+Form hierarchy on 3-5 nonportrait photos.
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
    throw "P4-A.2 validation requires exactly 3-5 image files; found $($images.Count). Use a dedicated folder."
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
    Write-Host ("=== P4-A.2 case {0}/{1}: {2} ===" -f $index,$images.Count,$image.Name)

    $caseArgs = @{
        InputImage = $image.FullName
        OutputDir = $caseDir
        MaxSize = $MaxSize
        Seed = $Seed
        RightsConfirmed = $true
    }
    & (Join-Path $root "scripts\run-p4a2-ab.ps1") @caseArgs
    if ($LASTEXITCODE -ne 0) {
        throw "P4-A.2 comparison failed for $($image.Name)"
    }

    $s = Get-Content -LiteralPath (Join-Path $caseDir "summary.json") -Raw | ConvertFrom-Json
    if (-not $s.baseline_segments_exactly_preserved -or -not $s.p4a1_gestures_exactly_preserved -or -not $s.overlay_paths_equal_paths_only) {
        throw "Frozen P4-A.2 invariant failed for $($image.Name)"
    }

    $base = $s.fixtures | Where-Object { $_.mode -eq "p2b1" }
    $p4a1 = $s.fixtures | Where-Object { $_.mode -eq "p4a1-paths" }
    $p4a2 = $s.fixtures | Where-Object { $_.mode -eq "p4a2-paths" }
    $overlay = $s.fixtures | Where-Object { $_.mode -eq "p4a2-overlay" }
    if (-not $base -or -not $p4a1 -or -not $p4a2 -or -not $overlay) {
        throw "Missing P4-A.2 comparison row for $($image.Name)"
    }

    $row = [pscustomobject]@{
        source_name = $image.Name
        case_name = $caseName
        gesture_count = [int]$s.hierarchy_stats.gestures
        form_count = [int]$s.hierarchy_stats.forms
        form_seed_count = [int]$s.hierarchy_stats.form_seeds
        form_overlap_rejected = [int]$s.hierarchy_stats.form_overlap_rejected
        gesture_mean_px = [double]$s.hierarchy_stats.gesture_mean
        form_mean_px = [double]$s.hierarchy_stats.form_mean
        form_max_px = [double]$s.hierarchy_stats.form_max
        gesture_length_share = [double]$s.hierarchy_stats.gesture_share
        form_length_share = [double]$s.hierarchy_stats.form_share
        p4a1_paths_edge_f1 = [double]$p4a1.edge_f1
        p4a2_paths_edge_f1 = [double]$p4a2.edge_f1
        hierarchy_edge_f1_change_vs_p4a1_pct = Improve-Pct $p4a1.edge_f1 $p4a2.edge_f1 $true
        overlay_edge_f1_improvement_vs_p2b1_pct = Improve-Pct $base.edge_f1 $overlay.edge_f1 $true
        overlay_tone_improvement_vs_p2b1_pct = Improve-Pct $base.tone_rmse $overlay.tone_rmse $false
        overlay_midtone_improvement_vs_p2b1_pct = Improve-Pct $base.midtone_rmse $overlay.midtone_rmse $false
        overlay_dark_improvement_vs_p2b1_pct = Improve-Pct $base.dark_rmse $overlay.dark_rmse $false
        baseline_white_rmse = [double]$base.white_rmse
        overlay_white_rmse = [double]$overlay.white_rmse
        source_sha256 = $s.source_sha256
    }
    $rows += $row

    $p4a1Rel = "$caseName/p4a1-paths.png"
    $p4a2Rel = "$caseName/p4a2-paths.png"
    $overlayRel = "$caseName/p4a2-overlay.png"

    $cards += @"
<section class="case">
<h2>$(Html $image.Name)</h2>
<div class="triple">
<figure><img src="$(Html $p4a1Rel)" alt="P4-A.1 Gestures only"><figcaption>P4-A.1 - Gestures only</figcaption></figure>
<figure class="focus"><img src="$(Html $p4a2Rel)" alt="P4-A.2 Gesture and Form paths"><figcaption>P4-A.2 - Gestures + Forms (inspect this first)</figcaption></figure>
<figure><img src="$(Html $overlayRel)" alt="P4-A.2 hierarchy overlay"><figcaption>P4-A.2 - hierarchy overlay</figcaption></figure>
</div>
<table>
<tr><th>Gesture paths</th><td>$($row.gesture_count)</td></tr>
<tr><th>New Form paths</th><td>$($row.form_count)</td></tr>
<tr><th>Form seeds</th><td>$($row.form_seed_count)</td></tr>
<tr><th>Gesture mean length</th><td>$([math]::Round($row.gesture_mean_px,2))px</td></tr>
<tr><th>Form mean / max length</th><td>$([math]::Round($row.form_mean_px,2))px / $([math]::Round($row.form_max_px,2))px</td></tr>
<tr><th>Form path-length share</th><td>$([math]::Round(100.0*$row.form_length_share,2))%</td></tr>
<tr><th>Paths-only edge-F1 change</th><td>$([math]::Round([double]$row.hierarchy_edge_f1_change_vs_p4a1_pct,3))%</td></tr>
</table>
<div class="questions">
<strong>Judge the middle-scale delta:</strong>
<ul>
<li>Do Form paths add meaningful interior/object structure that P4-A.1 missed?</li>
<li>Do they help seat/slat/rim/handle/leaf/pot form rather than merely duplicate silhouette?</li>
<li>Are the Form marks visibly shorter and subordinate to Gestures?</li>
<li>Do they stay coherent and source-supported?</li>
<li>Does the hierarchy look more like purposeful construction strokes without becoming cluttered?</li>
</ul>
</div>
</section>
"@
}

$withForms = @($rows | Where-Object { $_.form_count -gt 0 }).Count
$withMultipleForms = @($rows | Where-Object { $_.form_count -ge 2 }).Count
$pathEdgeWins = @($rows | Where-Object { $_.hierarchy_edge_f1_change_vs_p4a1_pct -gt 0 }).Count
$overlayEdgeWins = @($rows | Where-Object { $_.overlay_edge_f1_improvement_vs_p2b1_pct -gt 0 }).Count
$whiteNonWorse = @($rows | Where-Object { $_.overlay_white_rmse -le $_.baseline_white_rmse + 1e-12 }).Count

$aggregate = [pscustomobject]@{
    phase = "p4a2-real-pack"
    algorithm = "gesture-plus-medium-form-hierarchy"
    git_sha = $commit
    seed = $Seed
    max_side = $MaxSize
    source_count = $rows.Count
    rights_confirmed_by_operator = $true
    nonportrait_confirmed_by_operator = $true
    counts = [pscustomobject]@{
        sources_with_forms = $withForms
        sources_with_at_least_two_forms = $withMultipleForms
        hierarchy_paths_edge_f1_wins_vs_p4a1 = $pathEdgeWins
        overlay_edge_f1_wins_vs_p2b1 = $overlayEdgeWins
        overlay_white_nonworse = $whiteNonWorse
    }
    means = [pscustomobject]@{
        gesture_count = ($rows | Measure-Object gesture_count -Average).Average
        form_count = ($rows | Measure-Object form_count -Average).Average
        gesture_mean_px = ($rows | Measure-Object gesture_mean_px -Average).Average
        form_mean_px = ($rows | Measure-Object form_mean_px -Average).Average
        form_length_share = ($rows | Measure-Object form_length_share -Average).Average
        hierarchy_edge_f1_change_vs_p4a1_pct = ($rows | Measure-Object hierarchy_edge_f1_change_vs_p4a1_pct -Average).Average
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
<title>ScanSketch P4-A.2 Form Hierarchy Review</title>
<style>
body{font-family:system-ui,sans-serif;margin:24px;background:#202020;color:#eee}
h1{margin-bottom:6px}.meta{color:#bbb;max-width:1000px}.case{margin:42px 0;padding-top:18px;border-top:1px solid #555}
.triple{display:grid;grid-template-columns:1fr 1fr 1fr;gap:14px;align-items:start}
figure{margin:0;background:#111;padding:8px}figure.focus{outline:2px solid #888}
img{width:100%;height:auto;display:block;background:white}
figcaption{text-align:center;padding-top:8px;font-size:14px}
table{margin-top:14px;border-collapse:collapse}th,td{border:1px solid #555;padding:6px 10px;text-align:left}
.questions{margin-top:14px;max-width:920px;color:#ddd}.questions li{margin:4px 0}
@media(max-width:1000px){.triple{grid-template-columns:1fr}}
</style>
</head>
<body>
<h1>ScanSketch P4-A.2 - medium Form hierarchy review</h1>
<p class="meta">Compare frozen P4-A.1 Gestures against P4-A.2 Gestures + Forms. Seed $(Html $Seed), max-side $(Html $MaxSize), sources $(Html $rows.Count).</p>
<p class="meta"><strong>Inspect the center column first.</strong> P4-A.2 should add useful 8-32px Form marks while leaving the long Gesture skeleton unchanged. Horizontal tonal hatching remains deliberately untouched until P4-A.3.</p>
<p class="meta">Sources with Forms: $withForms/$($rows.Count). Sources with at least two Forms: $withMultipleForms/$($rows.Count). Paths-only edge-F1 wins vs P4-A.1: $pathEdgeWins/$($rows.Count).</p>
$($cards -join [Environment]::NewLine)
</body>
</html>
"@

$htmlPath = Join-Path $destination "review.html"
$html | Set-Content -LiteralPath $htmlPath -Encoding UTF8

Write-Host ""
Write-Host "=== P4-A.2 real-pack summary ==="
$rows |
    Select-Object source_name,gesture_count,form_count,form_seed_count,gesture_mean_px,form_mean_px,form_max_px,form_length_share,hierarchy_edge_f1_change_vs_p4a1_pct |
    Format-Table -AutoSize
Write-Host ("Coverage: forms={0}/{1}; >=2 forms={2}/{1}; paths-edge-wins={3}/{1}; overlay-edge-wins={4}/{1}; white-nonworse={5}/{1}" -f $withForms,$rows.Count,$withMultipleForms,$pathEdgeWins,$overlayEdgeWins,$whiteNonWorse)
Write-Host "Aggregate JSON: $aggregatePath"
Write-Host "Aggregate CSV:  $(Join-Path $destination 'summary.csv')"
Write-Host "Visual review:  $htmlPath"
Write-Host "Inspect the P4-A.2 paths-only center column first."
