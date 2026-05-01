# BuildPortable.ps1 - Creates portable version from vendor folder
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$target = Join-Path $scriptDir "LoonixTunesWin64v2_Portable"

Write-Host "Setting up Portable..." -ForegroundColor Cyan

# Create portable directory
if (!(Test-Path $target)) {
    New-Item -ItemType Directory -Path $target -Force | Out-Null
}

# Copy all files from vendor/ to portable folder
Copy-Item "$scriptDir\vendor\*" $target -Recurse -Force

# Rename app-core.bin back to .exe so it can be used
Rename-Item "$target\app-core.bin" "LoonixTunesWin64v2.exe" -Force

Write-Host "Done! You can run LoonixTunesWin64v2.exe inside: $target" -ForegroundColor Green
Pause
