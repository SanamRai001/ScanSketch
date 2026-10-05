# P3-G1: frozen P2-B.1 vs P3-A.2.3 generalization pack.
# Runs 3-5 permission-cleared NONPORTRAIT photographs from one local folder.
# Source images and outputs remain local. Refuses overwrite for reproducibility.
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
function Improve-Pct([double]$baseline, [double]$candidate, [bool]$HigherIsBetter) {
    if ([math]::Abs($baseline) -lt 1e-12) { return $null }
    if ($HigherIsBetter) {
        return (($candidate - $baseline) / [math]::Abs($baseline)) * 100.0
    }
    return (($baseline - $candidate) / [math]::Abs($baseline)) * 100.0
}
function Html([object]$value) {
    return [System.Net.WebUtility]::HtmlEncode([string]$value)
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
    throw "P3-G1 requires exactly 3-5 image files in the input folder; found $($images.Count). Use a dedicated folder."
}

$commit = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) {
    throw "Could not resolve Git HEAD."
}
New-Item -ItemType Directory -Path $destination -Force | Out-Null

$rows = @()
$reviewCards = @()
$index = 0

foreach ($image in $images) {
    $index++
    $slug = Safe-Slug $image.BaseName
    $caseName = "{0:D2}-{1}" -f $index, $slug
    $caseDir = Join-Path $destination $caseName

    Write-Host ""
    Write-Host ("=== P3-G1 case {0}/{1}: {2} ===" -f $index,$images.Count,$image.Name)

    $caseArgs = @{
        InputImage = $image.FullName
        OutputDir = $caseDir
        MaxSize = $MaxSize
        Seed = $Seed
        RightsConfirmed = $true
    }
    & (Join-Path $root "scripts\run-p3a23-ab.ps1") @caseArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Paired P3-A.2.3 run failed for $($image.Name)"
    }

    $summaryPath = Join-Path $caseDir "summary.json"
    if (-not (Test-Path -LiteralPath $summaryPath)) {
        throw "Missing summary for $($image.Name)"
    }
    $s = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
    $base = $s.fixtures | Where-Object { $_.mode -eq "p2b1" }
    $new = $s.fixtures | Where-Object { $_.mode -eq "p3a23" }
    if (-not $base -or -not $new) { throw "Missing comparison rows for $($image.Name)" }

    $countEqual = $base.strokes -eq $new.strokes
    $prefixExact = $s.tonal_prefix_identical_strokes -eq $s.deficit_proposal_stats.tone_count
    $slotAccounting = $s.changed_structural_slots -eq $s.deficit_proposal_stats.replacements
    if (-not $countEqual -or -not $prefixExact -or -not $slotAccounting) {
        throw "Frozen invariant failed for $($image.Name)"
    }

    $row = [pscustomobject]@{
        source_name = $image.Name
        case_name = $caseName
        width = $base.width
        height = $base.height
        strokes = $base.strokes
        tonal_prefix = $s.tonal_prefix_identical_strokes
        structural_budget = $s.deficit_proposal_stats.structural_budget
        max_replacements = $s.deficit_proposal_stats.max_replacements
        replacements = $s.deficit_proposal_stats.replacements
        baseline_retained = $s.deficit_proposal_stats.baseline_retained
        candidate_count = $s.deficit_proposal_stats.candidate_count
        mean_missing = $s.deficit_proposal_stats.mean_missing
        path_change_pct = Improve-Pct $base.path_length_px $new.path_length_px $true
        tone_improvement_pct = Improve-Pct $base.tone_rmse $new.tone_rmse $false
        midtone_improvement_pct = Improve-Pct $base.midtone_rmse $new.midtone_rmse $false
        dark_improvement_pct = Improve-Pct $base.dark_rmse $new.dark_rmse $false
        white_improvement_pct = Improve-Pct $base.white_rmse $new.white_rmse $false
        edge_f1_improvement_pct = Improve-Pct $base.edge_f1 $new.edge_f1 $true
        baseline_tone_rmse = $base.tone_rmse
        p3a23_tone_rmse = $new.tone_rmse
        baseline_midtone_rmse = $base.midtone_rmse
        p3a23_midtone_rmse = $new.midtone_rmse
        baseline_dark_rmse = $base.dark_rmse
        p3a23_dark_rmse = $new.dark_rmse
        baseline_white_rmse = $base.white_rmse
        p3a23_white_rmse = $new.white_rmse
        baseline_edge_f1 = $base.edge_f1
        p3a23_edge_f1 = $new.edge_f1
        source_sha256 = $s.source_sha256
    }
    $rows += $row

    $baseRel = "$caseName/p2b1.png"
    $newRel = "$caseName/p3a23.png"
    $reviewCards += @"
<section class="case">
<h2>$(Html $image.Name)</h2>
<div class="pair">
<figure><img src="$(Html $baseRel)" alt="P2-B.1"><figcaption>P2-B.1</figcaption></figure>
<figure><img src="$(Html $newRel)" alt="P3-A.2.3"><figcaption>P3-A.2.3</figcaption></figure>
</div>
<table>
<tr><th>Edge F1 improvement</th><td>$([math]::Round([double]$row.edge_f1_improvement_pct,3))%</td></tr>
<tr><th>Tone improvement</th><td>$([math]::Round([double]$row.tone_improvement_pct,3))%</td></tr>
<tr><th>Midtone improvement</th><td>$([math]::Round([double]$row.midtone_improvement_pct,3))%</td></tr>
<tr><th>Dark improvement</th><td>$([math]::Round([double]$row.dark_improvement_pct,3))%</td></tr>
<tr><th>Replacements</th><td>$($row.replacements) / $($row.structural_budget)</td></tr>
<tr><th>Candidates</th><td>$($row.candidate_count)</td></tr>
</table>
<p class="review">Manual review: compare at the same zoom. Note whether useful interior/object structure improves, whether changes only hug the silhouette, and whether tonal mass stays stable.</p>
</section>
"@
}

$edgeWins = @($rows | Where-Object { $_.edge_f1_improvement_pct -gt 0 }).Count
$toneWins = @($rows | Where-Object { $_.tone_improvement_pct -gt 0 }).Count
$midtoneWins = @($rows | Where-Object { $_.midtone_improvement_pct -gt 0 }).Count
$darkWins = @($rows | Where-Object { $_.dark_improvement_pct -gt 0 }).Count
$whiteNonWorse = @($rows | Where-Object { $_.p3a23_white_rmse -le $_.baseline_white_rmse + 1e-12 }).Count

$aggregate = [pscustomobject]@{
    phase = "p3g1-nonportrait-generalization"
    algorithm = "p3a23-deficit-proposals"
    git_sha = $commit
    seed = $Seed
    max_side = $MaxSize
    source_count = $rows.Count
    rights_confirmed_by_operator = $true
    nonportrait_confirmed_by_operator = $true
    simple_per_image_means_are_exploratory = $true
    wins = [pscustomobject]@{
        edge_f1 = $edgeWins
        tone_rmse = $toneWins
        midtone_rmse = $midtoneWins
        dark_rmse = $darkWins
        white_nonworse = $whiteNonWorse
    }
    mean_changes_pct = [pscustomobject]@{
        edge_f1 = ($rows | Measure-Object edge_f1_improvement_pct -Average).Average
        tone = ($rows | Measure-Object tone_improvement_pct -Average).Average
        midtone = ($rows | Measure-Object midtone_improvement_pct -Average).Average
        dark = ($rows | Measure-Object dark_improvement_pct -Average).Average
        path = ($rows | Measure-Object path_change_pct -Average).Average
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
<title>ScanSketch P3-G1 Review</title>
<style>
body{font-family:system-ui,sans-serif;margin:24px;background:#202020;color:#eee}
h1{margin-bottom:6px}.meta{color:#bbb}.case{margin:36px 0;padding-top:12px;border-top:1px solid #555}
.pair{display:grid;grid-template-columns:1fr 1fr;gap:18px;align-items:start}
figure{margin:0;background:#111;padding:10px}img{width:100%;height:auto;display:block;background:white}
figcaption{text-align:center;padding-top:8px}table{margin-top:14px;border-collapse:collapse}
th,td{border:1px solid #555;padding:6px 10px;text-align:left}.review{max-width:900px;color:#ccc}
@media(max-width:800px){.pair{grid-template-columns:1fr}}
</style>
</head>
<body>
<h1>ScanSketch P3-G1 — nonportrait generalization review</h1>
<p class="meta">Frozen P2-B.1 vs P3-A.2.3 · seed $(Html $Seed) · max-side $(Html $MaxSize) · images $(Html $rows.Count)</p>
<p class="meta">Metric wins: edge $edgeWins/$($rows.Count), tone $toneWins/$($rows.Count), midtone $midtoneWins/$($rows.Count), dark $darkWins/$($rows.Count), white non-worse $whiteNonWorse/$($rows.Count). Visual review remains mandatory.</p>
$($reviewCards -join [Environment]::NewLine)
</body>
</html>
"@
$htmlPath = Join-Path $destination "review.html"
$html | Set-Content -LiteralPath $htmlPath -Encoding UTF8

Write-Host ""
Write-Host "=== P3-G1 aggregate ==="
$rows |
    Select-Object source_name,replacements,structural_budget,candidate_count,edge_f1_improvement_pct,tone_improvement_pct,midtone_improvement_pct,dark_improvement_pct,path_change_pct |
    Format-Table -AutoSize
Write-Host ("Wins: edge={0}/{1}; tone={2}/{1}; midtone={3}/{1}; dark={4}/{1}; white-nonworse={5}/{1}" -f $edgeWins,$rows.Count,$toneWins,$midtoneWins,$darkWins,$whiteNonWorse)
Write-Host "Aggregate JSON: $aggregatePath"
Write-Host "Aggregate CSV:  $(Join-Path $destination 'summary.csv')"
Write-Host "Visual review:  $htmlPath"
Write-Host "Do not publish source photos or generated outputs without permission."
