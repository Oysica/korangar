# Scans the korangar source for CJK characters used in UI strings, merges them
# with the existing font atlas charset, and regenerates the MSDF font atlas
# (NotoSansTC.png + NotoSansTC.csv.gz) so all referenced characters render.
#
# Usage:
#   pwsh tools/update-font-charset.ps1

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path "$PSScriptRoot/.."
$fontDir  = Join-Path $repoRoot "korangar/archive/data/font"
$tool     = Join-Path $fontDir "msdf-atlas-gen.exe"
$ttf      = Join-Path $fontDir "NotoSansTC.ttf"
$csv      = Join-Path $fontDir "NotoSansTC.csv"
$png      = Join-Path $fontDir "NotoSansTC.png"
$gz       = "$csv.gz"
$charset  = Join-Path $fontDir "charset.txt"

# Scan paths for CJK Unified Ideographs (U+4E00..U+9FFF) used anywhere in source
# strings (UI labels, localization, Rust source, etc.).
$scanRoots = @(
    (Join-Path $repoRoot "korangar/src"),
    (Join-Path $repoRoot "korangar/archive/data/languages"),
    (Join-Path $repoRoot "korangar-interface/src")
)

Write-Host "Scanning source for CJK characters..."

$chars = [System.Collections.Generic.HashSet[char]]::new()
$cjkPattern = "[" + [char]0x4e00 + "-" + [char]0x9fff + "]"
$cjk = [regex]$cjkPattern

foreach ($root in $scanRoots) {
    if (-not (Test-Path $root)) { continue }
    Get-ChildItem -Path $root -Recurse -File -Include *.rs,*.ron,*.toml | ForEach-Object {
        $content = [System.IO.File]::ReadAllText($_.FullName, [System.Text.Encoding]::UTF8)
        foreach ($m in $cjk.Matches($content)) {
            [void]$chars.Add([char]$m.Value[0])
        }
    }
}

# Merge with whatever was already baked so we never lose previously supported glyphs.
if (Test-Path $charset) {
    $existing = [System.IO.File]::ReadAllText($charset, [System.Text.Encoding]::UTF8)
    foreach ($m in $cjk.Matches($existing)) {
        [void]$chars.Add([char]$m.Value[0])
    }
}

$sorted = $chars | Sort-Object
$joined = -join $sorted

Write-Host "Found $($chars.Count) unique CJK characters."

# msdf-atlas-gen charset format: quoted literal string + numeric ranges.
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)
$charsetContent = "`"$joined`", [32, 126]`n"
[System.IO.File]::WriteAllText($charset, $charsetContent, $utf8NoBom)
Write-Host "Wrote charset to $charset"

# Regenerate the MSDF atlas.
Write-Host "Running msdf-atlas-gen..."
& $tool -charset $charset -pxrange 6 -size 32 -yorigin top -dimensions 8192 4096 -type msdf -format png -font $ttf -csv $csv -imageout $png
if ($LASTEXITCODE -ne 0) { throw "msdf-atlas-gen failed with exit code $LASTEXITCODE" }

# Re-compress the CSV alongside the PNG, matching the format expected by the loader.
if (Test-Path $gz) { Remove-Item $gz }
Add-Type -AssemblyName System.IO.Compression
$inStream  = [System.IO.File]::OpenRead($csv)
$outStream = [System.IO.File]::Create($gz)
$gzStream  = [System.IO.Compression.GZipStream]::new($outStream, [System.IO.Compression.CompressionLevel]::Optimal)
try {
    $inStream.CopyTo($gzStream)
}
finally {
    $gzStream.Dispose()
    $outStream.Dispose()
    $inStream.Dispose()
}
Remove-Item $csv

Write-Host "Done. Updated NotoSansTC.png and NotoSansTC.csv.gz."
