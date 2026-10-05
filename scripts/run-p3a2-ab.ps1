# P3-A.2 proof: frozen P2-B.1 versus hybrid residual-aware structural accents.
# Same source/binary/seed and exact total accepted stroke count.
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
$hybridStats = $null
foreach ($mode in @("p2b1","p3a2")) {
    $png = Join-Path $destination "$mode.png"
    $json = Join-Path $destination "$mode-strokes.json"
    $report = Join-Path $destination "$mode-report.json"
    $args = @(
        "run","-p","scansketch-cli","--bin","scansketch","--",
        "--input",$source,"--output",$png,"--strokes",$json,
        "--max-size",[string]$MaxSize,"--seed",[string]$Seed
    )
    if ($mode -eq "p3a2") { $args += "--hybrid-structural" }

    Write-Host "Rendering $mode / max-side $MaxSize / seed $Seed"
    # Capture only the renderer's stdout. Cargo intentionally writes normal
    # compile/progress messages to stderr; merging stderr into stdout (2>&1)
    # makes Windows PowerShell 5.1 turn those harmless lines into terminating
    # NativeCommandError records when $ErrorActionPreference is "Stop".
    $renderOutput = @(& cargo @args)
    $renderExit = $LASTEXITCODE
    $renderOutput | ForEach-Object { Write-Host $_ }
    if ($renderExit -ne 0) {
        throw "Rendering failed for $mode (cargo exit $renderExit)"
    }

    if ($mode -eq "p3a2") {
        $line = $renderOutput | Where-Object { "$_" -like "P3-A.2 hybrid:*" } | Select-Object -Last 1
        if (-not $line) { throw "Missing P3-A.2 hybrid statistics line" }
        $text = "$line"
        function Read-Stat([string]$inputText, [string]$name) {
            $pattern = [regex]::Escape($name) + '=([0-9]+)'
            $match = [regex]::Match($inputText, $pattern)
            if (-not $match.Success) {
                throw "Unable to parse P3-A.2 hybrid statistic '$name' from: $inputText"
            }
            return [int]$match.Groups[1].Value
        }
        $hybridStats = [pscustomobject]@{
            tone_count = Read-Stat $text "tone"
            structural_budget = Read-Stat $text "structural-budget"
            hybrid_selected = Read-Stat $text "hybrid-selected"
            contour_fallback = Read-Stat $text "contour-fallback"
            candidate_count = Read-Stat $text "candidates"
        }
    }

    $measure = @(
        "run","-p","scansketch-cli","--bin","scansketch-measure","--",
        "--source",$source,"--preview",$png,"--strokes",$json,
        "--max-size",[string]$MaxSize,"--report",$report
    )
    & cargo @measure | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Measurement failed for $mode" }

    $m = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
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
    throw "P3-A.2 changed total stroke count relative to P2-B.1."
}
if (-not $hybridStats) { throw "P3-A.2 statistics missing" }
if ($hybridStats.hybrid_selected + $hybridStats.contour_fallback -ne $hybridStats.structural_budget) {
    throw "P3-A.2 structural budget accounting mismatch"
}

# Verify the actual JSON prefix, not only the reported count.
$base = Get-Content -LiteralPath (Join-Path $destination "p2b1-strokes.json") -Raw | ConvertFrom-Json
$hybrid = Get-Content -LiteralPath (Join-Path $destination "p3a2-strokes.json") -Raw | ConvertFrom-Json
$prefix = 0
$limit = [math]::Min($base.strokes.Count, $hybrid.strokes.Count)
for ($i=0; $i -lt $limit; $i++) {
    $a=$base.strokes[$i]; $b=$hybrid.strokes[$i]
    if ($a.x0 -ne $b.x0 -or $a.y0 -ne $b.y0 -or $a.x1 -ne $b.x1 -or $a.y1 -ne $b.y1 -or $a.width -ne $b.width -or $a.opacity -ne $b.opacity) {
        break
    }
    $prefix++
}
if ($prefix -ne $hybridStats.tone_count) {
    throw "Expected exact tonal-prefix length $($hybridStats.tone_count), observed $prefix"
}

$manifest = [pscustomobject]@{
    algorithm_phase = "p3a2-hybrid-structural"
    protocol_revision = "p2c-v1"
    comparison_lane = "same-source, same-binary, exact total stroke-count match; exact tonal prefix preserved"
    source_name = [System.IO.Path]::GetFileName($source)
    rights_confirmed_by_operator = $true
    source_sha256 = $sourceHash
    git_sha = $commit
    seed = $Seed
    max_side = $MaxSize
    tonal_prefix_identical_strokes = $prefix
    hybrid_stats = $hybridStats
    fixtures = $rows
}
$summary = Join-Path $destination "summary.json"
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $summary -Encoding UTF8

$rows | Select-Object mode,strokes,path_length_px,tone_rmse,midtone_rmse,dark_rmse,white_rmse,edge_f1,highlight_ink | Format-Table -AutoSize
Write-Host ("P3-A.2 mix: tone={0}; structural budget={1}; hybrid={2}; contour fallback={3}; candidates={4}" -f $hybridStats.tone_count,$hybridStats.structural_budget,$hybridStats.hybrid_selected,$hybridStats.contour_fallback,$hybridStats.candidate_count)
Write-Host "P3-A.2 paired run complete. Local manifest: $summary"
Write-Host "Inspect p2b1.png and p3a2.png at identical 100% zoom before judging quality."
