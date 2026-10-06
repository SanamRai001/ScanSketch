# P4-A.1 multi-image validation pack.
# Runs the frozen three-view comparison on 3-5 permission-cleared nonportrait photos.
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
    throw "P4-A.1 validation requires exactly 3-5 image files; found $($images.Count). Use a dedicated folder."
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
    Write-Host ("=== P4-A.1 case {0}/{1}: {2} ===" -f $index,$images.Count,$image.Name)

    $caseArgs = @{
        InputImage = $image.FullName
        OutputDir = $caseDir
        MaxSize = $MaxSize
        Seed = $Seed
        RightsConfirmed = $true
    }
    & (Join-Path $root "scripts\run-p4a1-ab.ps1") @caseArgs
    if ($LASTEXITCODE -ne 0) {
        throw "P4-A.1 three-view run failed for $($image.Name)"
    }

    $caseSummaryPath = Join-Path $caseDir "summary.json"
    $s = Get-Content -LiteralPath $caseSummaryPath -Raw | ConvertFrom-Json
    $base = $s.fixtures | Where-Object { $_.mode -eq "p2b1" }
    $overlay = $s.fixtures | Where-Object { $_.mode -eq "p4a1-overlay" }
    $pathsOnly = $s.fixtures | Where-Object { $_.mode -eq "p4a1-paths" }
    if (-not $base -or -not $overlay -or -not $pathsOnly) {
        throw "Missing P4-A.1 comparison row for $($image.Name)"
    }
    if (-not $s.baseline_segments_exactly_preserved -or -not $s.overlay_paths_equal_paths_only) {
        throw "Frozen P4-A.1 invariant failed for $($image.Name)"
    }

    $row = [pscustomobject]@{
        source_name = $image.Name
        case_name = $caseName
        accepted_paths = [int]$s.long_path_stats.accepted
        seed_candidates = [int]$s.long_path_stats.seeds
        baseline_segments = [int]$s.long_path_stats.baseline_segments
        total_path_length_px = [double]$s.long_path_stats.total_path
        mean_path_length_px = [double]$s.long_path_stats.mean_path
        max_path_length_px = [double]$s.long_path_stats.max_path
        overlay_edge_f1_improvement_pct = Improve-Pct $base.edge_f1 $overlay.edge_f1 $true
        overlay_tone_improvement_pct = Improve-Pct $base.tone_rmse $overlay.tone_rmse $false
        overlay_midtone_improvement_pct = Improve-Pct $base.midtone_rmse $overlay.midtone_rmse $false
        overlay_dark_improvement_pct = Improve-Pct $base.dark_rmse $overlay.dark_rmse $false
        overlay_white_improvement_pct = Improve-Pct $base.white_rmse $overlay.white_rmse $false
        baseline_white_rmse = $base.white_rmse
        overlay_white_rmse = $overlay.white_rmse
        baseline_edge_f1 = $base.edge_f1
        overlay_edge_f1 = $overlay.edge_f1
        paths_only_edge_f1 = $pathsOnly.edge_f1
        paths_only_logical_marks = $pathsOnly.logical_marks
        paths_only_segment_count = $pathsOnly.segment_count
        source_sha256 = $s.source_sha256
    }
    $rows += $row

    $baseRel = "$caseName/p2b1.png"
    $overlayRel = "$caseName/p4a1-overlay.png"
    $pathsRel = "$caseName/p4a1-paths.png"

    $cards += @"
<section class="case">
<h2>$(Html $image.Name)</h2>
<div class="triple">
<figure><img src="$(Html $baseRel)" alt="P2-B.1"><figcaption>P2-B.1 — frozen baseline</figcaption></figure>
<figure><img src="$(Html $overlayRel)" alt="P4-A.1 overlay"><figcaption>P4-A.1 — baseline + long gestures</figcaption></figure>
<figure class="focus"><img src="$(Html $pathsRel)" alt="P4-A.1 paths only"><figcaption>P4-A.1 — PATHS ONLY (inspect this first)</figcaption></figure>
</div>
<table>
<tr><th>Accepted gesture paths</th><td>$($row.accepted_paths)</td></tr>
<tr><th>Mean / max path length</th><td>$([math]::Round($row.mean_path_length_px,2))px / $([math]::Round($row.max_path_length_px,2))px</td></tr>
<tr><th>Total gesture length</th><td>$([math]::Round($row.total_path_length_px,2))px</td></tr>
<tr><th>Overlay edge-F1 change</th><td>$([math]::Round([double]$row.overlay_edge_f1_improvement_pct,3))%</td></tr>
<tr><th>Overlay tone change</th><td>$([math]::Round([double]$row.overlay_tone_improvement_pct,3))%</td></tr>
<tr><th>Paths-only edge F1</th><td>$([math]::Round([double]$row.paths_only_edge_f1,4))</td></tr>
</table>
<div class="questions">
<strong>Visual questions:</strong>
<ul>
<li>Do the paths follow meaningful silhouette or structural boundaries?</li>
<li>Do curves look continuous rather than like stitched scan fragments?</li>
<li>Are useful interior structures traced, not only the outer silhouette?</li>
<li>Do any paths wander, bridge blank paper, or cut across the object incorrectly?</li>
<li>Would these gestures look plausible as the first construction lines of a human sketch?</li>
</ul>
</div>
</section>
"@
}

$withPaths = @($rows | Where-Object { $_.accepted_paths -gt 0 }).Count
$withLong40 = @($rows | Where-Object { $_.max_path_length_px -ge 40 }).Count
$overlayEdgeWins = @($rows | Where-Object { $_.overlay_edge_f1_improvement_pct -gt 0 }).Count
$whiteNonWorse = @($rows | Where-Object { $_.overlay_white_rmse -le $_.baseline_white_rmse + 1e-12 }).Count

$aggregate = [pscustomobject]@{
    phase = "p4a1-real-pack"
    algorithm = "long-structural-gesture-paths"
    git_sha = $commit
    seed = $Seed
    max_side = $MaxSize
    source_count = $rows.Count
    rights_confirmed_by_operator = $true
    nonportrait_confirmed_by_operator = $true
    counts = [pscustomobject]@{
        sources_with_paths = $withPaths
        sources_with_max_path_at_least_40px = $withLong40
        overlay_edge_f1_wins = $overlayEdgeWins
        overlay_white_nonworse = $whiteNonWorse
    }
    means = [pscustomobject]@{
        accepted_paths = ($rows | Measure-Object accepted_paths -Average).Average
        mean_path_length_px = ($rows | Measure-Object mean_path_length_px -Average).Average
        max_path_length_px = ($rows | Measure-Object max_path_length_px -Average).Average
        overlay_edge_f1_improvement_pct = ($rows | Measure-Object overlay_edge_f1_improvement_pct -Average).Average
        overlay_tone_improvement_pct = ($rows | Measure-Object overlay_tone_improvement_pct -Average).Average
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
<title>ScanSketch P4-A.1 Long Gesture Review</title>
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
<h1>ScanSketch P4-A.1 — long structural gesture review</h1>
<p class="meta">Frozen P2-B.1 vs P4-A.1 overlay vs the exact same gesture layer alone. Seed $(Html $Seed), max-side $(Html $MaxSize), sources $(Html $rows.Count).</p>
<p class="meta"><strong>Inspect the PATHS ONLY column first.</strong> P4-A.1 is testing structural gesture tracing, not final tone. Horizontal hatching is intentionally still present in the overlay and is not reduced until P4-A.3.</p>
<p class="meta">Sources with paths: $withPaths/$($rows.Count). Sources with a >=40px path: $withLong40/$($rows.Count). Overlay edge-F1 wins: $overlayEdgeWins/$($rows.Count). White non-worse: $whiteNonWorse/$($rows.Count).</p>
$($cards -join [Environment]::NewLine)
</body>
</html>
"@

$htmlPath = Join-Path $destination "review.html"
$html | Set-Content -LiteralPath $htmlPath -Encoding UTF8

Write-Host ""
Write-Host "=== P4-A.1 real-pack summary ==="
$rows |
    Select-Object source_name,accepted_paths,seed_candidates,mean_path_length_px,max_path_length_px,total_path_length_px,overlay_edge_f1_improvement_pct,overlay_tone_improvement_pct,paths_only_edge_f1 |
    Format-Table -AutoSize
Write-Host ("Coverage: paths={0}/{1}; >=40px={2}/{1}; overlay-edge-wins={3}/{1}; white-nonworse={4}/{1}" -f $withPaths,$rows.Count,$withLong40,$overlayEdgeWins,$whiteNonWorse)
Write-Host "Aggregate JSON: $aggregatePath"
Write-Host "Aggregate CSV:  $(Join-Path $destination 'summary.csv')"
Write-Host "Visual review:  $htmlPath"
Write-Host "Inspect PATHS ONLY first. Original photos are never copied or uploaded."
