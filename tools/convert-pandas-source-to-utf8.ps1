# Converts every C/C++ source file under Pandas's src/ directory from
# BIG5 to UTF-8 (with BOM, so MSVC recognises the encoding). Files that
# already round-trip cleanly through UTF-8 are skipped but rewritten with
# a BOM so the whole tree ends up consistent.
#
# After running this script:
#   1. Add /utf-8 to the C/C++ compiler flags in every .vcxproj that
#      builds login/char/map server (or use the bundled rAthena.sln
#      project settings UI -> C/C++ -> Command Line -> Additional Options).
#   2. Recompile.
#   3. Set Pandas console codepage to UTF-8 (the setupConsoleOutputCP
#      change discussed earlier).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File .\tools\convert-pandas-source-to-utf8.ps1

$ErrorActionPreference = "Stop"

$pandasRoot = "D:\Ragnarok_Source\Zombie_Source_Code"
$srcRoot    = Join-Path $pandasRoot "src"
$extensions = @(".cpp", ".hpp", ".h", ".c", ".cc", ".cxx")

if (-not (Test-Path $srcRoot)) {
    throw "Pandas src directory not found at $srcRoot"
}

$utf8WithBom = [System.Text.UTF8Encoding]::new($true)
$big5 = [System.Text.Encoding]::GetEncoding(950)

$files = Get-ChildItem -Path $srcRoot -Recurse -File | Where-Object { $extensions -contains $_.Extension.ToLowerInvariant() }

$converted = 0
$alreadyUtf8 = 0
$skipped = 0
$failed = @()

foreach ($file in $files) {
    try {
        $bytes = [System.IO.File]::ReadAllBytes($file.FullName)
        if ($bytes.Length -eq 0) {
            $skipped++
            continue
        }

        # Already has UTF-8 BOM?
        if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
            $alreadyUtf8++
            continue
        }

        # Try strict UTF-8 decode. If it succeeds and the file contains
        # non-ASCII bytes, assume it's already UTF-8 - just add a BOM.
        $strictUtf8 = [System.Text.UTF8Encoding]::new($false, $true)
        $isUtf8 = $true
        $text = $null
        try {
            $text = $strictUtf8.GetString($bytes)
        }
        catch {
            $isUtf8 = $false
        }

        if (-not $isUtf8) {
            # Decode as BIG5 then re-encode as UTF-8.
            $text = $big5.GetString($bytes)
        }

        [System.IO.File]::WriteAllText($file.FullName, $text, $utf8WithBom)
        $converted++
        if ($converted % 25 -eq 0) {
            Write-Host "  ...converted $converted files"
        }
    }
    catch {
        $failed += [pscustomobject]@{ Path = $file.FullName; Error = $_.Exception.Message }
    }
}

Write-Host ""
Write-Host "Converted (rewritten with UTF-8 + BOM): $converted"
Write-Host "Already UTF-8 BOM (skipped):             $alreadyUtf8"
Write-Host "Empty / skipped:                          $skipped"
Write-Host "Failed:                                   $($failed.Count)"
if ($failed.Count -gt 0) {
    foreach ($entry in $failed) {
        Write-Warning "  $($entry.Path) -- $($entry.Error)"
    }
}

Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. In Visual Studio, open the Pandas solution, select all server"
Write-Host "     projects (login-server, char-server, map-server), Properties ->"
Write-Host "     C/C++ -> Command Line -> Additional Options -> add: /utf-8"
Write-Host "  2. Rebuild."
Write-Host "  3. Re-enable the SetConsoleOutputCP(CP_UTF8) change in utf8.cpp."
