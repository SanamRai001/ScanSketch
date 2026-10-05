# P3-A.1 proof: frozen P2-B.1 versus placement-aware tonal proposals.
# Same source/binary/seed/contours and exact total stroke COUNT.
# Outputs are local-only and never overwrite a prior experiment.
param(
    [Parameter(Mandatory = $true)][string]$InputImage,
    [Parameter(Mandatory = $true)][string]$OutputDir,
    [ValidateRange(32,1024)][int]$MaxSize = 512,
    [long]$Seed = 42,
    [switch]$RightsConfirmed
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
Set-Location $root

function FromRoot([string]$path) {
    if ([System.IO.Path]::IsPathRooted($path)) { return $path }
    return Join-Path $root $path
}

if ($Seed -lt 0) { throw "-Seed must be nonnegative." }
if (-not $RightsConfirmed) { throw "Pass -RightsConfirmed for an image you may process locally. Nothing is uploaded." }
$source = (Resolve-Path -LiteralPath (FromRoot $InputImage) -ErrorAction Stop).Path
if ([System.IO.Path]::GetExtension($source).ToLowerInvariant() -notin @(".png",".jpg",".jpeg")) {
    throw "Use a .png, .jpg, or .jpeg input."
}
$destination = FromRoot $OutputDir
if (Test-Path -LiteralPath $destination) {
    throw "Output already exists; choose a NEW -OutputDir: $destination"
}

$commit = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) { throw "Could not resolve Git HEAD." }
$sourceHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
New-Item -ItemType Directory -Path $destination -Force | Out-Null

$rows = @()
foreach ($mode in @("p2b1","p3a1")) {
    $png = Join-Path $destination "$mode.png"
    $json = Join-Path $destination "$mode-strokes.json"
    $report = Join-Path $destination "$mode-report.json"
    $args = @(
        "run","-p","scansketch-cli","--bin","scansketch","--",
        "--input",$source,"--output",$png,"--strokes",$json,
        "--max-size",[string]$MaxSize,"--seed",[string]$Seed
    )
    if ($mode -eq "p3a1") { $args += "--placement-aware" }

    Write-Host "Rendering $mode / max-side $MaxSize / seed $Seed"
    & cargo @args
    if ($LASTEXITCODE -ne 0) { throw "Rendering failed for $mode" }

    $measure = @(
        "run","-p","scansketch-cli","--bin","scansketch-measure","--",
        "--source",$source,"--preview",$png,"--strokes",$json,
        "--max-size",[string]$MaxSize,"--report",$report
    )
    & cargo @measure | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Measurement failed for $mode" }

    $m = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
    $st = Get-Content -LiteralPath $json -Raw | ConvertFrom-Json
    $strongNonhorizontal = 0
    foreach ($line in $st.strokes) {
        $dx = [math]::Abs([double]$line.x1 - [double]$line.x0)
        $dy = [math]::Abs([double]$line.y1 - [double]$line.y0)
        if ($dy -gt $dx * 0.70) { $strongNonhorizontal++ }
    }

    $rows += [pscustomobject]@{
        mode = $mode
        git_sha = $commit
        protocol_revision = $m.protocol_revision
        width = $m.width
        height = $m.height
        seed = $Seed
        max_side = $MaxSize
        source_sha256 = $sourceHash
        preview_sha256 = (Get-FileHash -LiteralPath $png -Algorithm SHA256).Hash
        strokes_sha256 = (Get-FileHash -LiteralPath $json -Algorithm SHA256).Hash
        strokes = $m.strokes.count
        strong_nonhorizontal_strokes = $strongNonhorizontal
        path_length_px = $m.strokes.total_path_length_px
        tone_rmse = $m.tone_rmse
        white_rmse = $m.white_region.rmse
        midtone_rmse = $m.midtone_region.rmse
        dark_rmse = $m.dark_region.rmse
        edge_precision = $m.edges.precision
        edge_recall = $m.edges.recall
        edge_f1 = $m.edges.f1
        highlight_ink = $m.unwanted_highlight_ink_fraction
        preview_path = $png
        strokes_path = $json
        report_path = $report
    }
}

if ($rows[0].strokes -ne $rows[1].strokes) {
    throw "P3-A.1 must match P2-B.1 total accepted stroke count in this experiment."
}
if ($rows[1].strong_nonhorizontal_strokes -le $rows[0].strong_nonhorizontal_strokes) {
    throw "P3-A.1 did not increase strongly nonhorizontal geometry on this fixture."
}

$manifest = [pscustomobject]@{
    algorithm_phase = "p3a1-placement-aware"
    protocol_revision = "p2c-v1"
    comparison_lane = "same-source, same-binary, exact total stroke-count match; path length/raster ink may differ"
    source_name = [System.IO.Path]::GetFileName($source)
    rights_confirmed_by_operator = $true
    source_sha256 = $sourceHash
    git_sha = $commit
    seed = $Seed
    max_side = $MaxSize
    fixtures = $rows
}
$summary = Join-Path $destination "summary.json"
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $summary -Encoding UTF8

$rows | Select-Object mode,strokes,strong_nonhorizontal_strokes,path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1,highlight_ink | Format-Table -AutoSize
Write-Host "P3-A.1 paired run complete. Local manifest: $summary"
Write-Host "Inspect p2b1.png and p3a1.png at identical 100% zoom before judging quality."
