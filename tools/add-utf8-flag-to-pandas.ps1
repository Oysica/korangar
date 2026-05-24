# Inserts `<AdditionalOptions>/utf-8 %(AdditionalOptions)</AdditionalOptions>`
# into every <ClCompile> section of the Pandas server vcxproj files so MSVC
# uses UTF-8 as the execution character set. Idempotent: skips sections that
# already mention /utf-8.

$ErrorActionPreference = "Stop"

$projects = @(
    "D:\Ragnarok_Source\Zombie_Source_Code\src\login\login-server.vcxproj",
    "D:\Ragnarok_Source\Zombie_Source_Code\src\char\char-server.vcxproj",
    "D:\Ragnarok_Source\Zombie_Source_Code\src\map\map-server.vcxproj",
    "D:\Ragnarok_Source\Zombie_Source_Code\src\map\map-server-generator.vcxproj"
)

foreach ($path in $projects) {
    if (-not (Test-Path $path)) {
        Write-Warning "Skipping missing file: $path"
        continue
    }

    $content = [System.IO.File]::ReadAllText($path, [System.Text.UTF8Encoding]::new($false))

    if ($content -match "/utf-8") {
        Write-Host "Already has /utf-8: $path"
        continue
    }

    # Insert AdditionalOptions right after each <ClCompile> opening tag.
    $updated = [regex]::Replace(
        $content,
        '<ClCompile>\s*\r?\n',
        "<ClCompile>`r`n      <AdditionalOptions>/utf-8 %(AdditionalOptions)</AdditionalOptions>`r`n"
    )

    [System.IO.File]::WriteAllText($path, $updated, [System.Text.UTF8Encoding]::new($false))
    Write-Host "Added /utf-8 to: $path"
}
